import glob
import subprocess
import sys
import os
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

PYTHON_SCRIPT   = os.path.join(PYTHON_IMPL_DIR, "Traclus_DL.py")
RUST_EXECUTABLE = os.path.join(RUST_IMPL_DIR, "target", "release", "rust_impl")

PYTHON_BENCH_SRC = os.path.join(INPUTS_DIR, "benchmarked_data")
PYTHON_BENCH_DST = os.path.join(PYTHON_IMPL_DIR, "benchmarked_data")

RUST_BENCH_SRC = os.path.join(INPUTS_DIR, "benchmarked_data")
RUST_BENCH_DST = os.path.join(RUST_IMPL_DIR, "benchmarked_data")

if os.name == "nt":
    RUST_EXECUTABLE += ".exe"


# =====================================================
#                 FILE OPERATIONS
# =====================================================

def copy_data_folder(src: str, dst: str):
    shutil.copytree(src, dst, dirs_exist_ok=True)

def remove_data_folder(folder: str):
    shutil.rmtree(folder, ignore_errors=True)


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


def run_impls(impl: str, args: ArgumentsTraclus):
    outputs = []

    if impl == "python":
        print("=== Running Python implementation ===")

        remove_data_folder(PYTHON_BENCH_DST)
        copy_data_folder(PYTHON_BENCH_SRC, PYTHON_BENCH_DST)
        total_time = 0.0
        while True:
            run_start = perf_counter()
            out = run_python_impl_once(args)
            run_end = perf_counter()
            total_time += run_end - run_start

            outputs.append({"args": args.get_args(), "out": out, "time": run_end - run_start})
            if args.iter_arguments() is False:
                break

        print(f"Total execution time: {total_time:.6f} seconds")

    args.reset_arguments()

    if impl == "rust":
        print("=== Running Rust implementation ===")

        remove_data_folder(RUST_BENCH_DST)
        copy_data_folder(RUST_BENCH_SRC, RUST_BENCH_DST)

        total_time = 0.0
        while True:
            run_start = perf_counter()
            out = run_rust_impl_once(args)
            run_end = perf_counter()
            total_time += run_end - run_start

            outputs.append({"args": args.get_args(), "out": out, "time": run_end - run_start})
            if args.iter_arguments() is False:
                break

        print(f"Total execution time: {total_time:.6f} seconds")

    return outputs


# =====================================================
#                 MAIN
# =====================================================

if __name__ == "__main__":
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

    print("\n")
    py_outputs = run_impls("python", traclus_args)
    rs_outputs = run_impls("rust", traclus_args)
    # py_outputs = rs_outputs

    print("\n=== Comparison ===")
    for (py_output, rs_output) in zip(py_outputs, rs_outputs):
        args = py_output["args"]
        py_time = py_output["time"]
        rs_time = rs_output["time"]
        py_output = py_output["out"]
        rs_output = rs_output["out"]

        print(f"\n-- Argument Set {args} --")
        print(f"Python Time: {py_time:.6f} seconds // Rust Time: {rs_time:.6f} seconds")
        # if py_output == rs_output:
        #     print("✔ Outputs match")
        # else:
        #     print("Python:\n", py_output)
        #     print("Rust:\n", rs_output)



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