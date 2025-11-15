import glob
import subprocess
import sys
import os
import shutil
from time import perf_counter

class ArgumentsTraclus:
    PATH_ITER = [
        "90_degres_3_traclus.txt",
        "montreal_to_montreal_traclus.txt",
        "small_radius_to_small_radius_traclus.txt",
        "up_the_bridges_traclus.txt" ]
    
    MAX_DIST_ITER =     [5,     10,     30,     50]
    MIN_DENSITY_ITER =  [2,     3,      3,      3]
    MAX_ANGLE_ITER =    [5,     5,      10,     10]
    SEG_SIZE_ITER =     [10,    10,     10,     10]

    def __init__(self, data_path: str):
        self.index_args = 0
        self.index_path = 0
        self.data_path = data_path
        self.load_arguments()

    def load_arguments(self):
        self.path = self.PATH_ITER[self.index_path]

        self.max_dist = self.MAX_DIST_ITER[self.index_args]
        self.min_density = self.MIN_DENSITY_ITER[self.index_args]
        self.max_angle = self.MAX_ANGLE_ITER[self.index_args]
        self.seg_size = self.SEG_SIZE_ITER[self.index_args]

    def iter_arguments(self) -> bool:
        self.index_args += 1

        if self.index_args >= len(self.MAX_DIST_ITER):
            self.index_args = 0
            self.index_path += 1
        
        if self.index_path >= 1:
            return False


        self.load_arguments()
        return True
    
    def copy(self):
        new_args = ArgumentsTraclus(self.data_path)
        new_args.index_args = self.index_args
        new_args.index_path = self.index_path
        new_args.load_arguments()
        return new_args
    
    def get_path(self) -> str:
        return self.data_path + "/" + self.get_name()

    def get_name(self) -> str:
        return self.PATH_ITER[self.index_path]
    
    def get_args(self) -> str:
        return f"[{self.max_dist}, {self.min_density}, {self.max_angle}, {self.seg_size}]"

def copy_data_folder(data_path: str, dst_folder: str):
    src_path = os.path.join(os.path.dirname(__file__), data_path)
    dst_path = os.path.join(os.path.dirname(__file__), dst_folder)
    shutil.copytree(src_path, dst_path, dirs_exist_ok=True)

def remove_data_folder(dst_folder: str):
    dst_path =  os.path.join(os.path.dirname(__file__), dst_folder)
    shutil.rmtree(dst_path, ignore_errors=True)
# ----------------------------------------
# BUILD STEP 
# Python can't "build" Python, but we can at least import once (warming).
# ----------------------------------------
def build_python_impl():
    print("=== 'Building' Python implementation (import warmup) ===")
    start = perf_counter()

    subprocess.run(
        [sys.executable, "-c", "import python_impl.program"],
        capture_output=True
    )

    end = perf_counter()
    print(f"Python warmup done in {end - start:.4f} seconds")


def build_rust_impl():
    print("=== Building Rust implementation (cargo build --release) ===")
    cwd_rust = os.path.join(os.path.dirname(__file__), "..", "rust_impl")
    start = perf_counter()

    result = subprocess.run(
        ["cargo", "build", "--release", "--quiet"],
        cwd=cwd_rust,
        capture_output=True,
        text=True
    )

    end = perf_counter()

    if result.returncode != 0:
        print("Rust build failed:\n", result.stderr)
        sys.exit(1)

    print(f"Rust build done in {end - start:.4f} seconds")


# ----------------------------------------
# RUN STEP — Pure execution timing
# ----------------------------------------
def run_python_impl_once(args: ArgumentsTraclus):   
    cmd = [
        sys.executable,
        "../python_impl/Traclus_DL.py",
        "--infile", "../python_impl/" + args.get_path(),
        "--max_dist", str(args.max_dist),
        "--min_density", str(args.min_density),
        "--max_angle", str(args.max_angle),
        "--segment_size", str(args.seg_size),
    ]

    subprocess.run(
        cmd,
        capture_output=True,
        text=True
    )

    folder_path = os.path.join("../python_impl", args.data_path)
    output_files = glob.glob(os.path.join(folder_path, "*"))
    corridor_file = next((f for f in output_files if "corridor" in f and args.get_name() in f), None)


    corridor_content = ""
    if corridor_file and os.path.exists(corridor_file):
        with open(corridor_file, "r") as f:
            corridor_content = f.read()
        os.remove(corridor_file)

    return corridor_content


def run_rust_impl_once(args: ArgumentsTraclus):
    exe_path = "../rust_impl/target/release/rust_impl"
    if os.name == "nt":
        exe_path += ".exe"

    result = subprocess.run(
        [exe_path],
        capture_output=True,
        text=True
    )
    return result.stdout.strip()

def run_impls(impl: str):
    args = ArgumentsTraclus("benchmarked_data")
    outputs = []

    if impl == "python": copy_data_folder("../inputs/benchmarked_data", "../python_impl/benchmarked_data")

    start = perf_counter()
    while True:
        if impl == "python":
            outputs.append({"args": args.copy(), "out": run_python_impl_once(args)})
        elif impl == "rust":
            outputs.append({"args": args.copy(), "out": run_rust_impl_once(args)})
        
        if args.iter_arguments() is False:
            break
    end = perf_counter()

    if impl == "python": remove_data_folder("../python_impl/benchmarked_data")

    if impl == "python":  print("=== Running Python implementation ===")
    if impl == "rust":    print("=== Running Rust implementation ===")
    print(f"Execution time: {end - start:.6f} seconds")
    
    return outputs


if __name__ == "__main__":
    build_python_impl()
    build_rust_impl()
    
    print("\n")
    py_outputs = run_impls('python')
    rs_outputs = run_impls('rust')

    print("\n=== Comparison ===")
    for (py_output, rs_output) in zip(py_outputs, rs_outputs):
        args = py_output['args']
        py_output = py_output['out']
        rs_output = rs_output['out']

        print(f"\n-- Argument Set {args.get_args()} for {args.get_name()} --")
        if py_output == rs_output:
            print("✔ Outputs match")
        else:
            print("✘ Outputs differ")
            print("Python:", py_output)
            print("Rust:  ", rs_output)
