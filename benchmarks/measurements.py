from collections import defaultdict
import pandas as pd


Point = tuple[int, int]

class SegmentInfo:
    def __init__(self, traj_id: int, segment_id: int, 
                 angle: float, start: Point, end: Point, corridor_id: int):
        self.traj_id = traj_id
        self.segment_id = segment_id
        self.angle = angle
        self.start = start
        self.end = end
        self.corridor_id = corridor_id

    def __repr__(self):
        return f"SegmentInfo(traj_id={self.traj_id}, segment_id={self.segment_id}, angle={self.angle}, start={self.start}, end={self.end}, corridor_id={self.corridor_id})"

OLD_ORDER = {"segment_id": 0, "angle": 2, "corridor_id": 3, "coordinate": 4}
NEW_ORDER = {"corridor_id": 0, "traj_id": 1, "segment_id": 2, "angle": 4, "coordinate": 5}

# =====================================================
#               TEXT FILE MEASUREMENTS
# =====================================================

# Return the number of corridors: number of lines in the file - 1 (header)
def number_of_corridors(file_path_corridors: str) -> int:
    with open(file_path_corridors, 'r') as file:
        lines = file.readlines()
        return len(lines) - 1  # Subtract 1 for the header line

# Return the number of segments: number of lines in the file - 1 (header)
def number_of_segments(file_path_segments: str) -> int:
    with open(file_path_segments, 'r') as file:
        lines = file.readlines()
        return len(lines) - 1  # Subtract 1 for the header line

# Return the number of non-clustered segments: 
# Count the segments where the 'corridor_id' column is '-1'
def number_of_non_clustered_segments(file_path_segments: str, old_order: dict = OLD_ORDER) -> int:
    nb_non_clustered = 0
    with open(file_path_segments, 'r') as file:
        lines = file.readlines()
        
        for line in lines[1:]:  # Skip the header line
            columns = line.strip().split('\t')
            if columns[old_order["corridor_id"]] == '-1':  # Check the 'corridor_id' column
                nb_non_clustered += 1

        return nb_non_clustered

# Generate a list of dictionary containing segments informations
# Return structure: dict of lists, where each list contains dicts with keys:
# traj_id (int), angle (float), start (tuple[int, int]), end (tuple[int, int]), corridor_id (int)
def generate_dict_segments(file_path_segments: str, order:dict = OLD_ORDER) -> dict:
    dict_of_list = defaultdict(list)

    with open(file_path_segments, 'r') as file:
        lines = file.readlines()
        
        for line in lines[1:]:  # Skip the header line
            columns = line.strip().split('\t')
            segment_id_full = columns[order["segment_id"]]
            full_coordinates = columns[order["coordinate"]]

            angle = float(columns[order["angle"]])
            corridor_id = int(columns[order["corridor_id"]])

            if order == OLD_ORDER:
                traj_id = int(segment_id_full.split(':')[0])
                segment_id = None
            else:
                traj_id = int(columns[order["traj_id"]])
                segment_id = int(segment_id_full)

            # Extract end point from LINESTRING(start.x start.y, end.x end.y) in full_coordinates
            linestring = full_coordinates.replace('LINESTRING(', '').replace(')', '')
            
            start_part = linestring.split(',')[0]
            start_coords = start_part.strip().split(' ')
            end_part = linestring.split(',')[1]
            end_coords = end_part.strip().split(' ')

            start_x = float(start_coords[0].replace(',', '.'))
            start_y = float(start_coords[1].replace(',', '.'))
            end_x = float(end_coords[0].replace(',', '.'))
            end_y = float(end_coords[1].replace(',', '.'))

            dict_of_list[traj_id].append(
                SegmentInfo(traj_id, segment_id, angle, 
                            (start_x, start_y), (end_x, end_y), corridor_id)
            )

    return dict_of_list

# Give the count of: 
# - segments that are clustered in both implementations
# - segments that are non-clustered in both implementations
# - segments that are only clustered in reference but non-clustered in comparison
def compare_clustered_seg_dict(reference_dict: dict, comparison_dict: dict) -> tuple:
    nb_corr_both_clustered = 0
    nb_corr_both_non_clustered = 0
    only_clustered_reference = 0
    
    for key in reference_dict:
        if key not in comparison_dict:
            print(f"Key {key} is missing in comparison dictionary.")
            continue

        if len(reference_dict[key]) != len(comparison_dict[key]):
            print(f"Length mismatch for key {key}: {len(reference_dict[key])} vs {len(comparison_dict[key])}.")
            continue

        for element in reference_dict[key]:
            result = compare_element(element, comparison_dict[key])
            nb_corr_both_clustered += result[0]
            nb_corr_both_non_clustered += result[1]
            only_clustered_reference += result[2]
    
    return (nb_corr_both_clustered, nb_corr_both_non_clustered, only_clustered_reference)

# Compare one segment element given a threshold for matching coordinates
# For python vs Rust
def compare_element(element: dict, comparison_list: list) -> tuple:
    correspondings = []
    THRESHOLD = 5  # Define a threshold for matching coordinates

    for comp_element in comparison_list:
        is_match = (
            abs(element.start[0] - comp_element.start[0]) <= THRESHOLD and
            abs(element.start[1] - comp_element.start[1]) <= THRESHOLD
        )
        if is_match:
            correspondings.append(comp_element)
            
    if len(correspondings) == 0:
        print(f"Element {element} is missing in comparison list.")
        return (0, 0, 0)  # No match found
    if len(correspondings) > 1:
        print(f"Multiple matches found for element {element} in comparison list.")
        return (0, 0, 0)  # Multiple matches found
    
    corresponding = correspondings[0]
    if element.corridor_id != -1 and corresponding.corridor_id != -1:
        return (1, 0, 0)  # Both clustered
    elif element.corridor_id == -1 and corresponding.corridor_id == -1:
        return (0, 1, 0)  # Both non-clustered
    elif element.corridor_id != -1 and corresponding.corridor_id == -1:
        return (0, 0, 1)  # Only clustered in reference
    
    return (0, 0, 0)

# Compare two values with a numeric tolerance. Returns (is_ok, updated_offset).
def _check_value_mismatch(ref_val, cmp_val, current_offset: float, context: str, max_offset: float = 10**(-1)) -> tuple[bool, float]:
    if str(ref_val) == str(cmp_val):
        return True, current_offset

    try:
        delta = abs(float(ref_val) - float(cmp_val))
        new_offset = max(current_offset, delta)
        if new_offset > max_offset:
            print(f"\tNot equal. Required offset {new_offset} exceeds max_offset {max_offset}.")
            print(f"\t{context}: reference={ref_val!r}, comparison={cmp_val!r}")
            return False, new_offset
        return True, new_offset
    except (TypeError, ValueError):
        print(f"\tNot equal (non-numeric mismatch).")
        print(f"\t{context}: reference={ref_val!r}, comparison={cmp_val!r}")
        return False, current_offset
    
# Check if both segments files have the same content
# Compute a numeric delta for precision mismatches below a given threshold (max_offset)
def is_same_output_file(reference_path: str, comparison_path: str) -> bool:

    ref_df = pd.read_csv(reference_path, sep='\t')
    cmp_df = pd.read_csv(comparison_path, sep='\t')

    # 1. Check column names and order
    if list(ref_df.columns) != list(cmp_df.columns):
        print(f"\tColumn mismatch:\n  reference : {list(ref_df.columns)}\n  comparison: {list(cmp_df.columns)}")
        return False

    # 2. Check line by line
    for i in range(max(len(ref_df), len(cmp_df))):
        ref_row = ref_df.iloc[i] if i < len(ref_df) else None
        cmp_row = cmp_df.iloc[i] if i < len(cmp_df) else None

        for col in ref_df.columns:
            ref_val = ref_row[col] if ref_row is not None else None
            cmp_val = cmp_row[col] if cmp_row is not None else None

            if str(ref_val) != str(cmp_val):
                print(f"\tRow {i} column '{col}' mismatch:")
                print(f"\tReference: {ref_val!r} and comparison: {cmp_val!r}")
                return False
    return True

def is_same_output_dict(reference_dict: dict, comparison_dict: dict) -> bool:
    ref_keys = set(reference_dict.keys())
    cmp_keys = set(comparison_dict.keys())

    current_offset = 0
    # 1. Iterate over trajectories
    for traj_id in sorted(ref_keys | cmp_keys):
        ref_segs = reference_dict.get(traj_id, [])
        cmp_segs = comparison_dict.get(traj_id, [])

        ref_segs = sorted(ref_segs, key=lambda s: (s.segment_id is None, s.segment_id))
        cmp_segs = sorted(cmp_segs, key=lambda s: (s.segment_id is None, s.segment_id))

        # 2. Iterate over segments
        for j in range(max(len(ref_segs), len(cmp_segs))):
            ref_seg = ref_segs[j] if j < len(ref_segs) else None
            cmp_seg = cmp_segs[j] if j < len(cmp_segs) else None

            if ref_seg is None or cmp_seg is None:
                print(f"\tSegment count mismatch at traj_id={traj_id}, segment index {j}: reference={ref_seg}, comparison={cmp_seg}")
                return False

            # 4. Compare all fields
            fields = {
                "traj_id":    (ref_seg.traj_id,    cmp_seg.traj_id),
                "segment_id": (ref_seg.segment_id, cmp_seg.segment_id),
                "angle":      (ref_seg.angle,      cmp_seg.angle),
                "start_x":      (ref_seg.start[0],      cmp_seg.start[0]),
                "start_y":      (ref_seg.start[1],      cmp_seg.start[1]),
                "end_x":        (ref_seg.end[0],         cmp_seg.end[0]),
                "end_y":        (ref_seg.end[1],         cmp_seg.end[1]),
                "corridor_id":(ref_seg.corridor_id,cmp_seg.corridor_id),
            }

            for field, (ref_val, cmp_val) in fields.items():
                context = f"traj_id={traj_id}, segment_index={j}, field='{field}'"
                ok, current_offset = _check_value_mismatch(ref_val, cmp_val, current_offset, context)

                if not ok: return False

    if current_offset > 0:
        print(f"\tDicts are equal with a maximum offset of {current_offset}.")
    return True


# =====================================================
#               STATISTICS CALCULATIONS
# =====================================================

def calculate_file_information(reference_path_corr: str, comparison_path_corr: str) -> dict:
    info = {
        "number_of_corridors": number_of_corridors(reference_path_corr),
        "number_of_segments": number_of_segments(comparison_path_corr),
        "number_of_non_clustered_segments": number_of_non_clustered_segments(comparison_path_corr)
    }
    return info

def calculate_similarity_index(reference_path_seg: str, comparison_path_seg: str, format:str = "old_format") -> dict:
    order = OLD_ORDER if format == "old_format" else NEW_ORDER
    ref_dict_segments = generate_dict_segments(reference_path_seg, order=order)
    comp_dict_segments = generate_dict_segments(comparison_path_seg, order=order)

    ref_results = compare_clustered_seg_dict(ref_dict_segments, comp_dict_segments)
    comp_results = compare_clustered_seg_dict(comp_dict_segments, ref_dict_segments)

    nb_both_clustered = ref_results[0] # Should be the same as comparison_result_rust[0]
    nb_both_non_clustered = ref_results[1] # Should be the same as comparison_result_rust[1]
    nb_only_clustered_py = ref_results[2] 
    nb_only_clustered_rust = comp_results[2]

    total_both = nb_both_clustered + nb_both_non_clustered

    if nb_both_clustered + nb_only_clustered_py + nb_only_clustered_rust == 0:
        similarity_index_1 = 1.0  # If there are no segments, we consider them as perfectly similar
    else:
        similarity_index_1 = nb_both_clustered / (nb_both_clustered + nb_only_clustered_py + nb_only_clustered_rust)

    if total_both + nb_only_clustered_rust + nb_only_clustered_py == 0:
        similarity_index_2 = 1.0  # If there are no segments, we consider them as perfectly similar
    else:
        similarity_index_2 = total_both / (total_both + nb_only_clustered_rust + nb_only_clustered_py)

    return {
        "similarity_index_1": similarity_index_1,
        "similarity_index_2": similarity_index_2
    }
    
def calculate_exact_output_information(reference_path_seg: str, comparison_path_seg: str, format:str = "old_format") -> None:

    # 1. Check file output content similarity
    if is_same_output_file(reference_path_seg, comparison_path_seg):
        print("✅ SAME OUTPUT: The two files have the same content.")
        return
    else:
        print("❌ DIFFERENT OUTPUT: The two files have different content.")

    # 2. Check dict output content similarity
    order = OLD_ORDER if format == "old_format" else NEW_ORDER
    list_of_dict_rust_stable = generate_dict_segments(reference_path_seg, order=order)
    list_of_dict_rust_new = generate_dict_segments(comparison_path_seg, order=order)

    if is_same_output_dict(list_of_dict_rust_stable, list_of_dict_rust_new):
        print("✅ SAME DICTS: The two dictionaries have the same content.")
        return
    else:
        print("❌ DIFFERENT DICTS: The two dictionaries have different content.")


    # 3. Calculate more check latter ... 
    