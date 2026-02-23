CORRIDOR_ID_INDEX = 3

# =====================================================
#               TEXT FILE MEASUREMENTS
# =====================================================

def number_of_corridors(file_path: str) -> int:
    with open(file_path, 'r') as file:
        lines = file.readlines()
        return len(lines) - 1  # Subtract 1 for the header line

def number_of_non_clustered_segments(file_path: str) -> int:
    nb_non_clustered = 0
    with open(file_path, 'r') as file:
        lines = file.readlines()
        
        for line in lines[1:]:  # Skip the header line
            columns = line.strip().split('\t')
            if columns[CORRIDOR_ID_INDEX] == '-1':  # Check the 'is_clustered' column
                nb_non_clustered += 1

        return nb_non_clustered

# =====================================================
#               STATISTICS CALCULATIONS
# =====================================================
