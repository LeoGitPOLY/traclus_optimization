import subprocess
import sys
import os
import argparse
import shutil
from time import perf_counter
from arguments_traclus import ArgumentsTraclus
from measurements import calculate_file_information, calculate_similaty_index

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
        choices=["visual", "time", "multi-od"],
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

def remove_and_copy_input_file(traclus_args: ArgumentsTraclus):
    # remove previous input files in the implementation folders
    remove_data_folder(PYTHON_BENCH_DST)
    remove_data_folder(RUST_BENCH_DST)

    # Copy only one input file at the time (for python and rust)
    create_empty_folder(PYTHON_BENCH_DST)
    copy_file(BENCH_SRC, PYTHON_BENCH_DST, traclus_args.get_name())
    create_empty_folder(RUST_BENCH_DST)
    copy_file(BENCH_SRC, RUST_BENCH_DST, traclus_args.get_name())

def get_files_with_all_substring(folder: str, substring: list[str]) -> list:
    names_folder = get_list_of_files_name(folder)
    names_substring = [name for name in names_folder if all(sub in name for sub in substring)]
    return names_substring

def transfert_files_to_qgis_results(rust_mode: list):
    name_data = traclus_args.get_name().replace("_traclus", "").replace(".txt", ".tsv")
    create_file(BENCH_SRC, RESULTS_QGIS_DIR, name_data, "DL_INPUT.txt")

    # For python output files; only one instance is generated for each type (corridor and segment)
    name_py_seg = get_files_with_all_substring(PYTHON_BENCH_DST, ["segment"])[0]
    name_py_corr = get_files_with_all_substring(PYTHON_BENCH_DST, ["corridor"])[0]
    create_file(PYTHON_BENCH_DST, RESULTS_QGIS_DIR, name_py_corr, "CORRIDOR_PY.txt")
    create_file(PYTHON_BENCH_DST, RESULTS_QGIS_DIR, name_py_seg, "SEG_PY.txt")

    
    # For rust output files; many instances can be generated (e.g., for different modes)
    for mode in rust_mode:
        name = mode['name']
        
        rust_seg_old = get_files_with_all_substring(RUST_BENCH_DST, ["segment", "old", name])[0]
        rust_seg_new = get_files_with_all_substring(RUST_BENCH_DST, ["segment", "new", name])[0]
        rust_corr = get_files_with_all_substring(RUST_BENCH_DST, ["corridor", name])[0]
        
        create_file(RUST_BENCH_DST, RESULTS_QGIS_DIR, rust_seg_old, f"SEG_RUST_{name}.txt")
        create_file(RUST_BENCH_DST, RESULTS_QGIS_DIR, rust_seg_new, f"SEG_RUST_NEW_{name}.txt")
        create_file(RUST_BENCH_DST, RESULTS_QGIS_DIR, rust_corr, f"CORRIDOR_RUST_{name}.txt")

# =====================================================
#                 STATISTICS CALCULATIONS
# =====================================================

def file_information(impl: str) -> dict:
    file_corridor = None
    file_segment = None

    if impl == "python":
        file_corridor = PYTHON_BENCH_DST + "/" + get_files_with_all_substring(PYTHON_BENCH_DST, ["corridor"])[0]
        file_segment = PYTHON_BENCH_DST + "/" + get_files_with_all_substring(PYTHON_BENCH_DST, ["segment"])[0]
    elif impl == "rust":
        file_corridor = RUST_BENCH_DST + "/" + get_files_with_all_substring(RUST_BENCH_DST, ["corridor"])[0]
        file_segment = RUST_BENCH_DST + "/" + get_files_with_all_substring(RUST_BENCH_DST, ["segment", "old"])[0]
    
    return calculate_file_information(file_corridor, file_segment)

def similaty_index() -> dict:
    file_segment_py = PYTHON_BENCH_DST + "/" + get_files_with_all_substring(PYTHON_BENCH_DST, ["segment"])[0]
    file_segment_rust = RUST_BENCH_DST + "/" + get_files_with_all_substring(RUST_BENCH_DST, ["segment", "old"])[0]

    return calculate_similaty_index(file_segment_py, file_segment_rust)

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

    results = subprocess.run(cmd, capture_output=True, text=True)

def run_rust_impl_once(args: ArgumentsTraclus, cmd: str = "serial"):
    cmd = [
        "cargo",
        "run",
        "--release",
        "--",
        "--file", os.path.join(RUST_IMPL_DIR, args.get_path()),
        "--max_dist", args.get_args_value('max_dist'),
        "--min_density", args.get_args_value('min_density'),
        "--max_angle", args.get_args_value('max_angle'),
        "--segment_size", args.get_args_value('seg_size'),
        "--mode", cmd,
        "--interface", "performance"
    ]   
    
    results = subprocess.run(cmd, cwd=RUST_IMPL_DIR, capture_output=True, text=True)

def run_timed_once(impl: str, args: ArgumentsTraclus, mode: dict = {"name": "NONE"}):
    run_start = perf_counter()
    
    if impl == "python":
        run_python_impl_once(args)
    elif impl == "rust":
        run_rust_impl_once(args, mode['cmd'])
    
    run_end = perf_counter()
    time = run_end - run_start

    information = file_information(impl)

    print(f"Argument Set {args.get_args()} for {impl} // Time: {time:.6f} seconds")
    return {"impl": impl, "mode": mode['name'], "args": args.get_args(), "time": time, **information}
    
def run_timed_all(impl: str, args: ArgumentsTraclus, mode: dict = {"name": "NONE"}):
    outputs = []
    total_time = 0.0

    while True:
        output = run_timed_once(impl, args, mode)
        outputs.append(output)
        total_time += output["time"]

        if args.iter_arguments() is False:
            break  
    
    print(f"\nTotal {impl} mode {mode['name']} execution time: {total_time:.6f} seconds \n")
    return outputs

# =====================================================
#                 TEST IMPLEMENTATIONS
# =====================================================

def visual_testing(traclus_args: ArgumentsTraclus, rust_mode: list):
    if not os.path.exists(RESULTS_QGIS_DIR):
        print(f"Error: Required folder to run the visual testing'{RESULTS_QGIS_DIR}' does not exist.")
        sys.exit(1)
    
    while True:
        remove_and_copy_input_file(traclus_args)

        # TESTING PYTHON
        run_timed_once("python", traclus_args)
        # TESTING ALL MODE RUST
        for mode in rust_mode: run_timed_once("rust", traclus_args, mode)

        # Calculate similiarity index
        similarity_index = similaty_index()
        print(f"\nSimilarity Index for argument set {traclus_args.get_args()}: "
          f"Similarity Index 1: {similarity_index['similarity_index_1']:.6f}, "
          f"Similarity Index 2: {similarity_index['similarity_index_2']:.6f}\n")
        
        transfert_files_to_qgis_results(rust_mode)

        print(f"=== Visual results are ready for argument set {traclus_args.get_args()} ===")

        user_input = input("\nPress Enter to continue to the next argument set (or 's' to stop)...\n")
        if user_input.lower() == 's':
            break
        if traclus_args.iter_arguments() is False:
            break


def time_testing(traclus_args: ArgumentsTraclus, rust_mode: list):
    remove_and_copy_input_file(traclus_args)
    outputs = []

    # TESTING PYTHON
    outputs += run_timed_all("python", traclus_args)
    traclus_args.reset_arguments()

    # TESTING ALL MODE RUST
    for  mode in rust_mode:
        outputs += run_timed_all("rust", traclus_args, mode)
        traclus_args.reset_arguments()

    for output in outputs:
        print(f"{output['impl']};{output['mode']};{output['args']};{output['time']:.6f}")


def run_averaged_multi_OD(args: dict, rust_mode: list):
    base_file = "enquete_od_DL_$NB$_traclus.txt"
    # list_of_sizes = [1000, 2000, 3000, 4000, 5000, 
                    #  6000, 7000, 8000, 9000, 10000, 11000, 12000, 13000, 
                    #  14000, 15000, 16000, 17000, 18000, 19000, 20000]
    list_of_sizes = [1000, 16000, 17000]
    max_index_python = -1

    outputs_time = []
    outputs_similarity = []
    try: # TODO: REMOVE
        for (index,size) in enumerate(list_of_sizes):
            file_name = base_file.replace("$NB$", str(size))
            
            args_copy = args.copy()
            args_copy['path'] = [file_name]
            args_copy['min_density'] = [size//3]
            traclus_args = ArgumentsTraclus("benchmarked_data", args_copy, print_as_text=False)

            remove_and_copy_input_file(traclus_args)
            print(f"\n======== Running implementations for {file_name} ===========")

            current_outputs = {}
            # TESTING PYTHON
            if index <= max_index_python:
                outputs_time += run_timed_all("python", traclus_args)
                traclus_args.reset_arguments()

            # TESTING ALL MODE RUST
            for  mode in rust_mode:
                outputs_time += run_timed_all("rust", traclus_args, mode)
                traclus_args.reset_arguments()

            # Calculate similiarity index
            if index <= max_index_python:
                similarity_index = similaty_index()
                outputs_similarity.append({"size":size, **similarity_index})
  
            

    except Exception as e:
        print(f"An error occurred: {e}")

    print("\n=== Final Time Results (sorted by implementation and mode) ===")
    outputs_sorted = sorted(outputs_time, key=lambda x: (x['impl'], x['mode']))
    for output in outputs_sorted:
        print(f"{output['impl']};{output['mode']};{output['args']};{output['time']:.6f};"
              f"{output['number_of_corridors']};{output['number_of_segments']};{output['number_of_non_clustered_segments']}".replace(".", ","))

    print("\n=== Final Similarity Index Results (sorted by size) ===")
    for output in outputs_similarity:
        print(f"{output['size']};{output['similarity_index_1']:.6f};{output['similarity_index_2']:.6f}".replace(".", ","))

# =====================================================
#                 MAIN
# =====================================================

if __name__ == "__main__":
    args_cli = parse_args()
    args_values = {
        'max_dist':     [600],
        'min_density':  [1],
        'max_angle':    [5],
        'seg_size':     [150],
        'path': ["90_degrees_DL_traclus.txt" ],
    }
    rust_mode = [{'cmd': 'parallel-rayon', 'name': 'ParallelRayon'},
                 {'cmd': 'serial', 'name': 'Serial'}]
    traclus_args = ArgumentsTraclus("benchmarked_data", args_values)

    build_python_impl()
    build_rust_impl()

    print("\n=== Starting Benchmarks ===")

    if args_cli.mode == "visual":
        visual_testing(traclus_args, rust_mode)
    elif args_cli.mode == "time":
        time_testing(traclus_args, rust_mode)
    elif args_cli.mode == "multi-od":
        run_averaged_multi_OD(args_values, rust_mode)
    