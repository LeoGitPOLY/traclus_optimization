from pathlib import Path
import random
import os
import csv
from random_shapes import Circle, Point, Quadrilateral, RandomShape
from math import cos, sin, radians

MONTREAL_QUAD = Quadrilateral(
    [Point(287175,5039383), 
     Point(297225,5055425), 
     Point(303942,5053921), 
     Point(297464,5035132)])

SMALL_RADIUS_1 = Circle(Point(293680,5040290), 500)
SMALL_RADIUS_2 = Circle(Point(299157,5049817), 500)

RIVE_SUD_POS = Circle(Point(305156,5042535), 30)
LAVAL_POS = Circle(Point(287256,5045801), 30)

def generate_desire_line_shape(lines: int, start_shapes: list[RandomShape], end_shapes: list[RandomShape]) -> list[list]:
    """
    Generate desire lines with a random weight and coordinates inside a shape
    """
    list_of_lines = []

    for i in range(lines):
        start_shape = start_shapes[i % len(start_shapes)]
        end_shape = end_shapes[i % len(end_shapes)]

        start_point = start_shape.get_random_inside()
        end_point = end_shape.get_random_inside()

        weight = 1

        list_of_lines.append([i, weight, start_point, end_point])
    
    return list_of_lines

def generate_desire_line_in_circle(angle_interval: float, center: Point, radius: float) -> list[list]:
    """
    Generate desire lines starting from the center and ending on a circle.
    angle_interval is the spacing between lines in DEGREES.
    """
    list_of_lines = []

    if angle_interval <= 0:
        return list_of_lines

    num_lines = int(360 / angle_interval)

    for i in range(num_lines):
        angle_deg = i * angle_interval
        angle_rad = radians(angle_deg)

        start_point = center
        end_point = Point(
            center.x + radius * cos(angle_rad),
            center.y + radius * sin(angle_rad)
        )

        weight = 1

        list_of_lines.append([i, weight, start_point, end_point])

    return list_of_lines

def generate_vertical_parallel_lines(spacing: float, center: Point, height: float, num_lines: int) -> list[list]:
    """
    Generate vertical parallel lines spaced by `spacing`.

    - spacing: horizontal distance between lines
    - center: center point of the whole set
    - height: total height of each line
    - num_lines: number of lines
    """

    if spacing <= 0 or num_lines <= 0:
        return []

    half_h = height / 2
    lines = []

    start_x = center.x - (num_lines - 1) * spacing / 2

    for i in range(num_lines):
        x = start_x + i * spacing

        start_point = Point(x, center.y - half_h)
        end_point   = Point(x, center.y + half_h)

        weight = 1
        lines.append([i, weight, start_point, end_point])

    return lines

def convert_csv_enquete_to_list(input_file) -> list[list]:
    """
    Convert Enquête CSV format to a list of desire lines.
    """
    list_of_lines = []
    
    with open(input_file, 'r') as infile:
        reader = csv.DictReader(infile, delimiter=';')

        for row in reader:
            id_val = int(row['id'])
            weight = round(float(row['facper']))
            xorig = float(row['xorig'])
            yorig = float(row['yorig'])
            xdest = float(row['xdest'])
            ydest = float(row['ydest'])
            
            start_point = Point(xorig, yorig)
            end_point = Point(xdest, ydest)
            
            list_of_lines.append([id_val, weight, start_point, end_point])
    
    return list_of_lines

def chose_random_lines(list_lines: list[list], num_lines: int) -> list[list]:
    """
    Choose a random subset of lines from the given list.
    """
    if num_lines >= len(list_lines):
        return list_lines.copy()
    
    return random.sample(list_lines, num_lines)


def save_to_tsv(list_lines: list[str], filename: str):
    """
    Save generated desire lines to a TSV file.
    """
    header = "name\tweight\tcoordinates\n"
    os.makedirs(os.path.dirname(filename), exist_ok=True)

    with open(filename, "w") as f:
        f.write(header)

        for i, weight, start_point, end_point in list_lines:
            line_str = f"LINESTRING({start_point.x} {start_point.y}, {end_point.x} {end_point.y})"
            f.write(f"{i}\t{weight}\t{line_str}\n")

def save_to_traclus(list_lines: list[str], filename: str):
    """
    Save generated desire lines to a Traclus format file.
    """
    os.makedirs(os.path.dirname(filename), exist_ok=True)

    with open(filename, "w") as f:
        for i, weight, start_point, end_point in list_lines:
             f.write(f"{i}\t{weight}\t{start_point.x}\t{start_point.y}\t{end_point.x}\t{end_point.y}\n")



def main():
    random.seed(42)  # fixed seed for reproducibility

    # Base path: always points to the root of the project
    ROOT_DIR = Path(__file__).resolve().parent.parent
    DATA_DIR = ROOT_DIR / "inputs" / "data"

    DATA_DIR.mkdir(parents=True, exist_ok=True)

    # # Montreal_to_Montreal: 500 lines
    # filename = DATA_DIR / "montreal_to_montreal_DL"
    # list_of_lines = generate_desire_line_shape(500, [MONTREAL_QUAD], [MONTREAL_QUAD])
    # save_to_tsv(list_of_lines, f"{filename}.tsv")
    # save_to_traclus(list_of_lines, f"{filename}_traclus.txt")

    # # Small_radius_to_Small_radius: 150 lines
    # filename = DATA_DIR / "small_radius_to_small_radius_DL"
    # list_of_lines = generate_desire_line_shape(150, [SMALL_RADIUS_1], [SMALL_RADIUS_2])
    # save_to_tsv(list_of_lines, f"{filename}.tsv")
    # save_to_traclus(list_of_lines, f"{filename}_traclus.txt")

    # # Up_the_bridges: 500 lines
    # filename = DATA_DIR / "up_the_bridges_DL"
    # list_of_lines = generate_desire_line_shape(500, [LAVAL_POS, RIVE_SUD_POS], [MONTREAL_QUAD])
    # save_to_tsv(list_of_lines, f"{filename}.tsv")
    # save_to_traclus(list_of_lines, f"{filename}_traclus.txt")

    # # Circle_around: lines every 30 degrees
    # filename = DATA_DIR / "circle_around_DL"
    # list_of_lines = generate_desire_line_in_circle(5, SMALL_RADIUS_1.center, 1000)
    # save_to_tsv(list_of_lines, f"{filename}.tsv")
    # save_to_traclus(list_of_lines, f"{filename}_traclus.txt")
    
    # # Parallels lines: 10 lines
    # filename = DATA_DIR / "parallels_DL"
    # list_of_lines = generate_vertical_parallel_lines(20, SMALL_RADIUS_1.center, 1000, 10)
    # save_to_tsv(list_of_lines, f"{filename}.tsv")
    # save_to_traclus(list_of_lines, f"{filename}_traclus.txt")

    # Convert Enquête format to Traclus format
    input_file = DATA_DIR / "traclus_od_sample_3k_south_shore_to_montreal.csv"
    filename = DATA_DIR / "enquete_od_DL"
    list_of_lines = convert_csv_enquete_to_list(input_file)

    for sample_size in [500, 1000, 3000]:
        sampled_lines = chose_random_lines(list_of_lines, sample_size)
        save_to_tsv(sampled_lines, f"{filename}_{sample_size}.tsv")
        save_to_traclus(sampled_lines, f"{filename}_{sample_size}_traclus.txt")
    


if __name__ == "__main__":
    main()
