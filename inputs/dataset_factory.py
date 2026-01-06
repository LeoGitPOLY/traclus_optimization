from pathlib import Path
import random
import os
from random_shapes import Circle, Point, Quadrilateral, RandomShape

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

        weight = random.randint(1000, 2000)

        list_of_lines.append([i, weight, start_point, end_point])
    
    return list_of_lines

def generate_desire_line_in_circle(lines: int, center: Point, radius: float, angle_interval: float) -> list[list]:
    pass


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

    # Montreal_to_Montreal: 500 lines
    # filename = DATA_DIR / "montreal_to_montreal_DL"
    # list_of_lines = generate_desire_line(500, [MONTREAL_QUAD], [MONTREAL_QUAD])
    # save_to_tsv(list_of_lines, f"{filename}.tsv")
    # save_to_traclus(list_of_lines, f"{filename}_traclus.txt")

    # Small_radius_to_Small_radius: 150 lines
    # filename = DATA_DIR / "small_radius_to_small_radius_DL"
    # list_of_lines = generate_desire_line(150, [SMALL_RADIUS_1], [SMALL_RADIUS_2])
    # save_to_tsv(list_of_lines, f"{filename}.tsv")
    # save_to_traclus(list_of_lines, f"{filename}_traclus.txt")

    # Up_the_bridges: 500 lines
    # filename = DATA_DIR / "up_the_bridges_DL"
    # list_of_lines = generate_desire_line(500, [LAVAL_POS, RIVE_SUD_POS], [MONTREAL_QUAD])
    # save_to_tsv(list_of_lines, f"{filename}.tsv")
    # save_to_traclus(list_of_lines, f"{filename}_traclus.txt")



if __name__ == "__main__":
    main()
