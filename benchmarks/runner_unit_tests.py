import subprocess
import sys
import os
import argparse
import shutil
from time import perf_counter
from unittest import result
from arguments_traclus import ArgumentsTraclus

# =====================================================
#                 PATH CONSTANTS
# =====================================================

ROOT_DIR = os.path.dirname(__file__)

PYTHON_IMPL_DIR = os.path.join(ROOT_DIR, "..", "python_impl")
RUST_IMPL_DIR   = os.path.join(ROOT_DIR, "..", "rust_impl")
INPUTS_DIR      = os.path.join(ROOT_DIR, "..", "inputs")
RESULTS_QGIS_DIR   = os.path.join(ROOT_DIR, "..", "results_qgis")

PYTHON_SCRIPT   = os.path.join(PYTHON_IMPL_DIR, "Traclus_DL.py")
RUST_EXECUTABLE = os.path.join(RUST_IMPL_DIR, "target", "release", "rust_impl")

BENCH_SRC = os.path.join(INPUTS_DIR, "benchmarked_data")
PYTHON_BENCH_DST = os.path.join(PYTHON_IMPL_DIR, "benchmarked_data")
RUST_BENCH_DST = os.path.join(RUST_IMPL_DIR, "benchmarked_data")

if os.name == "nt":
    RUST_EXECUTABLE += ".exe"



def parse_args():
    parser = argparse.ArgumentParser()

    parser.add_argument(
        "-m", "--mode",
        choices=["visual", "time"],
        default = "time",
        help="Run mode [visual, time, default: time]"
    )

    return parser.parse_args()


# =====================================================
#                 FILE OPERATIONS
# =====================================================

def copy_data_folder(src: str, dst: str):
    shutil.copytree(src, dst, dirs_exist_ok=True)

def remove_data_folder(folder: str):
    shutil.rmtree(folder, ignore_errors=True)

def copy_file(src: str, dst: str, name: str):
    create_file(src, dst, name, name)

def create_file(src: str, dst: str, name_src: str, name_dst: str):
    src_file =  os.path.join(src, name_src)
    dist_file = os.path.join(dst, name_dst)
    shutil.copy2(src_file, dist_file)

def create_empty_folder(folder: str):
    os.makedirs(folder, exist_ok=True)

def get_list_of_files_name(folder: str) -> list:
    return [f for f in os.listdir(folder) if os.path.isfile(os.path.join(folder, f))]

# =====================================================
#                 BUILD STEP
# =====================================================

def build_python_impl():
    print("=== 'Building' Python implementation (import) ===")
    start = perf_counter()

    subprocess.run(
        [sys.executable, "-c", "import python_impl.program"],
        capture_output=True
    )

    end = perf_counter()
    print(f"Python warmup done in {end - start:.4f} seconds")


def build_rust_impl():
    print("=== Building Rust implementation (cargo build --release) ===")
    start = perf_counter()

    result = subprocess.run(
        ["cargo", "build", "--release", "--quiet"],
        cwd=RUST_IMPL_DIR,
        capture_output=True,
        text=True
    )

    end = perf_counter()

    if result.returncode != 0:
        print("Rust build failed:\n", result.stderr)
        sys.exit(1)

    print(f"Rust build done in {end - start:.4f} seconds")


# =====================================================
#                 RUN STEP — Execution
# =====================================================

def run_python_impl_once(args: ArgumentsTraclus):
    cmd = [
        sys.executable,
        PYTHON_SCRIPT,
        "--infile", os.path.join(PYTHON_IMPL_DIR, args.get_path()),
        "--max_dist", args.get_args_value('max_dist'),
        "--min_density", args.get_args_value('min_density'),
        "--max_angle", args.get_args_value('max_angle'),
        "--segment_size", args.get_args_value('seg_size'),
    ]

    result = subprocess.run(cmd, capture_output=True, text=True)
    return result.stdout.strip()


def run_rust_impl_once(args: ArgumentsTraclus):
    cmd = [
        "cargo",
        "run",
        "--release",
        "--",
        "--infile", os.path.join(RUST_IMPL_DIR, args.get_path()),
        "--max_dist", args.get_args_value('max_dist'),
        "--min_density", args.get_args_value('min_density'),
        "--max_angle", args.get_args_value('max_angle'),
        "--segment_size", args.get_args_value('seg_size'),
    ]   
    
    result = subprocess.run(cmd, cwd=RUST_IMPL_DIR, capture_output=True, text=True)
    return result.stdout.strip()

# =====================================================
#                 TEST IMPLEMENTATIONS
# =====================================================

def visual_testing(traclus_args: ArgumentsTraclus):
    if not os.path.exists(RESULTS_QGIS_DIR):
        print(f"Error: Required folder to run the visual testing'{RESULTS_QGIS_DIR}' does not exist.")
        sys.exit(1)
    
    remove_data_folder(PYTHON_BENCH_DST)
    remove_data_folder(RUST_BENCH_DST)

    # Copy only the one file at the time (for python and rust)
    create_empty_folder(PYTHON_BENCH_DST)
    copy_file(BENCH_SRC, PYTHON_BENCH_DST, traclus_args.get_name())
    create_empty_folder(RUST_BENCH_DST)
    copy_file(BENCH_SRC, RUST_BENCH_DST, traclus_args.get_name())

    run_timed_once("python", traclus_args)
    run_timed_once("rust", traclus_args)

    names_py  = get_list_of_files_name(PYTHON_BENCH_DST)
    names_rust = get_list_of_files_name(RUST_BENCH_DST)

    name_py_corridor = next((name for name in names_py if "corridor" in name), None)
    name_py_segments = next((name for name in names_py if "segment" in name), None)
    name_rust_corridor = next((name for name in names_rust if "corridor" in name), None)

    create_file(BENCH_SRC, RESULTS_QGIS_DIR, traclus_args.get_name(), "DL_INPUT.txt")
    create_file(PYTHON_BENCH_DST, RESULTS_QGIS_DIR, name_py_corridor, "CORRIDOR_PY.txt")
    create_file(PYTHON_BENCH_DST, RESULTS_QGIS_DIR, name_py_segments, "SEG_PY.txt")
    create_file(RUST_BENCH_DST, RESULTS_QGIS_DIR, name_rust_corridor, "CORRIDOR_RUST.txt")


def time_testing(traclus_args: ArgumentsTraclus):
    remove_data_folder(PYTHON_BENCH_DST)
    remove_data_folder(RUST_BENCH_DST)
    
    copy_data_folder(BENCH_SRC, PYTHON_BENCH_DST)
    copy_data_folder(BENCH_SRC, RUST_BENCH_DST)

    py_outputs = run_timed_all("python", traclus_args)
    traclus_args.reset_arguments()
    rs_outputs = run_timed_all("rust", traclus_args)


    print("\n=== Comparison ===")
    for (py_output, rs_output) in zip(py_outputs, rs_outputs):
        args = py_output["args"]
        py_time = py_output["time"]
        rs_time = rs_output["time"]
        py_output = py_output["out"]
        rs_output = rs_output["out"]

        print(f"-- Argument Set {args} --")
        print(f"Python Time: {py_time:.6f} seconds // Rust Time: {rs_time:.6f} seconds\n")


def run_timed_once(impl: str, args: ArgumentsTraclus):
    run_start = perf_counter()
    
    if impl == "python":
        out = run_python_impl_once(args)
    elif impl == "rust":
        out = run_rust_impl_once(args)
    
    run_end = perf_counter()
    time = run_end - run_start

    print(f"Argument Set {args.get_args()} // Time: {time:.6f} seconds")
    return {"args": args.get_args(), "out": out, "time": time}

    
def run_timed_all(impl: str, args: ArgumentsTraclus):
    outputs = []
    total_time = 0.0

    while True:
        output = run_timed_once(impl, args)
        outputs.append(output)
        total_time += output["time"]

        if args.iter_arguments() is False:
            break  
    
    print(f"\nTotal {impl} execution time: {total_time:.6f} seconds \n")
    return outputs

# =====================================================
#                 MAIN
# =====================================================

if __name__ == "__main__":
    args_cli = parse_args()
    args_values = {
        'max_dist':     [250],
        'min_density':  [8],
        'max_angle':    [10],
        'seg_size':     [1000],
        'path':   ["up_the_bridges_DL_traclus.txt"],
    }

    traclus_args = ArgumentsTraclus("benchmarked_data", args_values)

    build_python_impl()
    build_rust_impl()

    print("\n=== Starting Benchmarks ===")

    if args_cli.mode == "visual":
        visual_testing(traclus_args)
    elif args_cli.mode == "time":
        time_testing(traclus_args)
    



 # TODO: pass this step to retrieve the correct output file 
    # to a function after the running phase
    # folder_path = os.path.join(PYTHON_IMPL_DIR, args.data_path)
    # output_files = glob.glob(os.path.join(folder_path, "*"))

    # corridor_file = next(
    #     (f for f in output_files if "corridor" in f and args.get_name() in f),
    #     None
    # )

    # if corridor_file and os.path.exists(corridor_file):
    #     with open(corridor_file, "r") as f:
    #         content = f.read()
    #     return content