import random
import math as mat
from abc import ABC, abstractmethod

class Point:
    """Represents a 2D point"""
    def __init__(self, x: float, y: float):
        self.x = x
        self.y = y

    def __repr__(self):
        return f"Point({self.x}, {self.y})"

class RandomShape(ABC): 
    """Abstract base class for random shapes"""
    @abstractmethod
    def get_random_inside(self) -> Point:
        pass

class Circle (RandomShape): 
    """Circle defined by center (Point) and radius"""
    def __init__(self, center: Point, radius: float):
        self.center = center
        self.radius = radius

    def get_random_inside(self) -> Point:
        """Return a random point inside the circle"""
        # Random angle and distance from center
        angle = random.uniform(0, 2 * mat.pi)
        r = self.radius * (random.random() ** 0.5)  # sqrt for uniform distribution
        x = self.center.x + r * round(random.uniform(-1,1),2)
        y = self.center.y + r * round(random.uniform(-1,1),2)
        return Point(x, y)


class Quadrilateral(RandomShape):
    """Quadrilateral defined by 4 points"""
    def __init__(self, points: list[Point]):
        self.points = points[0:4]

    def get_random_inside(self) -> Point:
        """Return a random point inside the quadrilateral using bilinear interpolation"""
        # For simplicity, assume convex quadrilateral and split into two triangles
        def random_point_in_triangle(a: Point, b: Point, c: Point) -> Point:
            r1 = random.random()
            r2 = random.random()
            if r1 + r2 > 1:
                r1, r2 = 1 - r1, 1 - r2
            x = a.x + r1 * (b.x - a.x) + r2 * (c.x - a.x)
            y = a.y + r1 * (b.y - a.y) + r2 * (c.y - a.y)
            return Point(x, y)

        # Split quadrilateral into two triangles
        if random.random() < 0.5:
            return random_point_in_triangle(self.points[0], self.points[1], self.points[2])
        else:
            return random_point_in_triangle(self.points[0], self.points[2], self.points[3])
