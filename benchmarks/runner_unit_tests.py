import subprocess
import sys
import os
import argparse
import shutil
from time import perf_counter
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
DATA_SRC = os.path.join(INPUTS_DIR, "data")

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

def get_files_with_all_substring(folder: str, substring: list[str]) -> list:
    names_folder = get_list_of_files_name(folder)
    names_substring = [name for name in names_folder if all(sub in name for sub in substring)]
    return names_substring

def transfert_files_to_qgis_results(rust_mode: list):
    name_data = traclus_args.get_name().replace("_traclus", "").replace(".txt", ".tsv")
    create_file(DATA_SRC, RESULTS_QGIS_DIR, name_data, "DL_INPUT.txt")

    # For python output files; only one instance is generated for each type (corridor and segment)
    name_py_seg = get_files_with_all_substring(PYTHON_BENCH_DST, ["segment"])[0]
    name_py_corr = get_files_with_all_substring(PYTHON_BENCH_DST, ["corridor"])[0]
    create_file(PYTHON_BENCH_DST, RESULTS_QGIS_DIR, name_py_corr, "CORRIDOR_PY.txt")
    create_file(PYTHON_BENCH_DST, RESULTS_QGIS_DIR, name_py_seg, "SEG_PY.txt")

    
    # For rust output files; many instances can be generated (e.g., for different modes)
    for mode in rust_mode:
        rust_seg_old = get_files_with_all_substring(RUST_BENCH_DST, ["segment", "old", mode])[0]
        rust_corr_new = get_files_with_all_substring(RUST_BENCH_DST, ["corridor", "new", mode])[0]
        rust_corr = get_files_with_all_substring(RUST_BENCH_DST, ["corridor", mode])[0]
        create_file(RUST_BENCH_DST, RESULTS_QGIS_DIR, rust_seg_old, f"SEG_RUST_{mode}.txt")
        create_file(RUST_BENCH_DST, RESULTS_QGIS_DIR, rust_corr_new, f"CORRIDOR_RUST_NEW_{mode}.txt")
        create_file(RUST_BENCH_DST, RESULTS_QGIS_DIR, rust_corr, f"CORRIDOR_RUST_{mode}.txt")

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
    # print(result.stdout.strip())
    return result.stdout.strip()



def run_rust_impl_once(args: ArgumentsTraclus, mode: str = "serial"):
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
        "--mode", mode
    ]   
    
    result = subprocess.run(cmd, cwd=RUST_IMPL_DIR, capture_output=True, text=True)
    # print(result.stdout.strip())
    return result.stdout.strip()

# =====================================================
#                 TEST IMPLEMENTATIONS
# =====================================================

def visual_testing(traclus_args: ArgumentsTraclus, rust_mode: list):
    if not os.path.exists(RESULTS_QGIS_DIR):
        print(f"Error: Required folder to run the visual testing'{RESULTS_QGIS_DIR}' does not exist.")
        sys.exit(1)
    
    while True:
        remove_data_folder(PYTHON_BENCH_DST)
        remove_data_folder(RUST_BENCH_DST)

        # Copy only one input file at the time (for python and rust)
        create_empty_folder(PYTHON_BENCH_DST)
        copy_file(BENCH_SRC, PYTHON_BENCH_DST, traclus_args.get_name())
        create_empty_folder(RUST_BENCH_DST)
        copy_file(BENCH_SRC, RUST_BENCH_DST, traclus_args.get_name())

        run_timed_once("python", traclus_args)
        for mode in rust_mode: run_timed_once("rust", traclus_args, mode)

        transfert_files_to_qgis_results(rust_mode)

        print(f"=== Visual results are ready for argument set {traclus_args.get_args()} ===")
        user_input = input("\nPress Enter to continue to the next argument set (or 's' to stop)...\n")
        
        if user_input.lower() == 's':
            break
        if traclus_args.iter_arguments() is False:
            break


def time_testing(traclus_args: ArgumentsTraclus, rust_mode: list):
    remove_data_folder(PYTHON_BENCH_DST)
    remove_data_folder(RUST_BENCH_DST)

    # Copy only one input file at the time (for python and rust)
    create_empty_folder(PYTHON_BENCH_DST)
    copy_file(BENCH_SRC, PYTHON_BENCH_DST, traclus_args.get_name())
    create_empty_folder(RUST_BENCH_DST)
    copy_file(BENCH_SRC, RUST_BENCH_DST, traclus_args.get_name())

    # TESTING PYTHON
    py_outputs = run_timed_all("python", traclus_args)
    traclus_args.reset_arguments()

    # TESTING ALL MODE RUST
    rs_outputs_all_modes = {}
    for  mode in rust_mode:
        rs_outputs_all_modes[mode] = run_timed_all("rust", traclus_args, mode)
        traclus_args.reset_arguments()


    print("\n=== Comparison ===")
    for index in range(len(py_outputs)):
        time_algo = {}
        output_algo = {}
        args = py_outputs[index]["args"]
        
        time_algo["python"] = py_outputs[index]["time"]
        output_algo["python"] = py_outputs[index]["out"]

        for mode in rust_mode:
            time_algo[mode] = rs_outputs_all_modes[mode][index]["time"]
            output_algo[mode] = rs_outputs_all_modes[mode][index]["out"]

        print(f"-- Argument Set {args} --")
        for algo in time_algo:
            print(f"{algo}: {time_algo[algo]:.6f} seconds")


def run_timed_once(impl: str, args: ArgumentsTraclus, mode: str = ""):
    run_start = perf_counter()
    
    if impl == "python":
        out = run_python_impl_once(args)
    elif impl == "rust":
        out = run_rust_impl_once(args, mode)
    
    run_end = perf_counter()
    time = run_end - run_start

    print(f"Argument Set {args.get_args()} for {impl} // Time: {time:.6f} seconds")
    return {"args": args.get_args(), "out": out, "time": time}
    
def run_timed_all(impl: str, args: ArgumentsTraclus, mode: str = ""):
    outputs = []
    total_time = 0.0

    while True:
        output = run_timed_once(impl, args, mode)
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
        'max_dist':     [600],
        'min_density':  [1000],
        'max_angle':    [7],
        'seg_size':     [3000],
        'path': ["enquete_od_DL_3000_traclus.txt"],
    }
    rust_mode = ['serial', 'parallel-rayon']
    traclus_args = ArgumentsTraclus("benchmarked_data", args_values)
    #'path':   ["circle_around_DL_traclus.txt", "90_degres_3_DL_traclus.txt", "small_radius_to_small_radius_DL_traclus.txt", "up_the_bridges_DL_traclus.txt"],
    # "enquete_od_DL_500_traclus.txt"

    build_python_impl()
    build_rust_impl()

    print("\n=== Starting Benchmarks ===")

    if args_cli.mode == "visual":
        visual_testing(traclus_args, rust_mode)
    elif args_cli.mode == "time":
        time_testing(traclus_args, rust_mode)
    


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
