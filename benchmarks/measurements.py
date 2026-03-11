from collections import defaultdict

SEGMENT_ID_INDEX = 0
CORRIDOR_ID_INDEX = 3

# =====================================================
#               TEXT FILE MEASUREMENTS
# =====================================================

def number_of_corridors(file_path_corridors: str) -> int:
    with open(file_path_corridors, 'r') as file:
        lines = file.readlines()
        return len(lines) - 1  # Subtract 1 for the header line

def number_of_segments(file_path_segments: str) -> int:
    with open(file_path_segments, 'r') as file:
        lines = file.readlines()
        return len(lines) - 1  # Subtract 1 for the header line

def number_of_non_clustered_segments(file_path_segments: str) -> int:
    nb_non_clustered = 0
    with open(file_path_segments, 'r') as file:
        lines = file.readlines()
        
        for line in lines[1:]:  # Skip the header line
            columns = line.strip().split('\t')
            if columns[CORRIDOR_ID_INDEX] == '-1':  # Check the 'is_clustered' column
                nb_non_clustered += 1

        return nb_non_clustered


def generate_dict_segments(file_path_segments: str) -> dict:
    dict_of_list = defaultdict(list)

    with open(file_path_segments, 'r') as file:
        lines = file.readlines()
        
        for line in lines[1:]:  # Skip the header line
            columns = line.strip().split('\t')
            segment_id_full = columns[SEGMENT_ID_INDEX]
            corridor_id = columns[CORRIDOR_ID_INDEX]

            traj_id = segment_id_full.split(':')[0]
            start_x = int(float(segment_id_full.split(':')[1].replace(',', '.')))
            start_y = int(float(segment_id_full.split(':')[2].replace(',', '.')))

            dict_of_list[traj_id].append({
                "traj_id": traj_id,
                "start_x": start_x,
                "start_y": start_y,
                "corridor_id": corridor_id
            })

    return dict_of_list

def compare_clustered_seg_dict(reference_dict: dict, comparison_dict: dict) -> None:
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


def compare_element(element: dict, comparison_list: list) -> tuple:
    correspondings = []
    THRESHOLD = 5  # Define a threshold for matching coordinates

    for comp_element in comparison_list:
        is_match = (abs(element['start_x'] - comp_element['start_x']) <= THRESHOLD and 
                    abs(element['start_y'] - comp_element['start_y']) <= THRESHOLD)
        if is_match:
            correspondings.append(comp_element)
            
    
    if len(correspondings) == 0:
        print(f"Element {element} is missing in comparison list.")
        return (0, 0, 0)  # No match found
    if len(correspondings) > 1:
        print(f"Multiple matches found for element {element} in comparison list.")
        return (0, 0, 0)  # Multiple matches found
    
    corresponding = correspondings[0]
    if element['corridor_id'] != '-1' and corresponding['corridor_id'] != '-1':
        return (1, 0, 0)  # Both clustered
    elif element['corridor_id'] == '-1' and corresponding['corridor_id'] == '-1':
        return (0, 1, 0)  # Both non-clustered
    elif element['corridor_id'] != '-1' and corresponding['corridor_id'] == '-1':
        return (0, 0, 1)  # Only clustered in reference
    
    return (0, 0, 0)



# =====================================================
#               STATISTICS CALCULATIONS
# =====================================================

def calculate_file_information(file_path_corr_py: str, file_path_seg_py: str) -> dict:
    info = {
        "number_of_corridors": number_of_corridors(file_path_corr_py),
        "number_of_segments": number_of_segments(file_path_seg_py),
        "number_of_non_clustered_segments": number_of_non_clustered_segments(file_path_seg_py)
    }
    return info

def calculate_similaty_index(file_path_seg_py: str, file_path_seg_rust: str) -> dict:
    dict_segments_py = generate_dict_segments(file_path_seg_py)
    dict_segments_rust = generate_dict_segments(file_path_seg_rust)

    comparison_result_py = compare_clustered_seg_dict(dict_segments_py, dict_segments_rust)
    comparison_result_rust = compare_clustered_seg_dict(dict_segments_rust, dict_segments_py)

    nb_both_clustered = comparison_result_py[0] # Should be the same as comparison_result_rust[0]
    nb_both_non_clustered = comparison_result_py[1] # Should be the same as comparison_result_rust[1]
    nb_only_clustered_py = comparison_result_py[2] 
    nb_only_clustered_rust = comparison_result_rust[2]

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
    
