import subprocess
import sys
import os
import argparse
import shutil
from time import perf_counter
from arguments_traclus import ArgumentsTraclus
from measurements import calculate_exact_output_information, calculate_file_information, calculate_similaty_index

# =====================================================
#                 PATH CONSTANTS
# =====================================================

ROOT_DIR = os.path.dirname(__file__)

PYTHON_IMPL_DIR = os.path.join(ROOT_DIR, "..", "python_impl")
RUST_IMPL_DIR   = os.path.join(ROOT_DIR, "..", "rust_impl")
RUST_STABLE_DIR = os.path.join(ROOT_DIR, "rust_versions")
INPUTS_DIR      = os.path.join(ROOT_DIR, "..", "inputs")
RESULTS_QGIS_DIR   = os.path.join(ROOT_DIR, "..", "results_qgis")

PYTHON_SCRIPT   = os.path.join(PYTHON_IMPL_DIR, "Traclus_DL.py")
RUST_EXECUTABLE = os.path.join(RUST_IMPL_DIR, "target", "release", "rust_impl")

BENCH_SRC = os.path.join(INPUTS_DIR, "benchmarked_data")

PYTHON_BENCH_DST = os.path.join(PYTHON_IMPL_DIR, "benchmarked_data")
RUST_BENCH_DST = os.path.join(RUST_IMPL_DIR, "benchmarked_data")
RUST_STABLE_BENCH_DST = os.path.join(RUST_STABLE_DIR, "benchmarked_data")

newest_version_exe = "" # Will be set after building the Rust implementation

if os.name == "nt":
    RUST_EXECUTABLE += ".exe"



def parse_args():
    parser = argparse.ArgumentParser()

    parser.add_argument(
        "-m", "--mode",
        choices=["visual", "time", "multi-od", "verify"],
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

def remove_and_copy_input_file(traclus_args: ArgumentsTraclus, impl: str = "both"):
    # remove previous input files in the implementation folders
    # Copy only one input file at the time (for python and rust)

    if impl in ["both", "python"]:
        remove_data_folder(PYTHON_BENCH_DST)
        create_empty_folder(PYTHON_BENCH_DST)
        copy_file(BENCH_SRC, PYTHON_BENCH_DST, traclus_args.get_name())

    if impl in ["both", "rust"]:
        remove_data_folder(RUST_BENCH_DST)
        create_empty_folder(RUST_BENCH_DST)
        copy_file(BENCH_SRC, RUST_BENCH_DST, traclus_args.get_name())
        
    if impl in ["both", "stable_rust"]:
        remove_data_folder(RUST_STABLE_BENCH_DST)
        create_empty_folder(RUST_STABLE_BENCH_DST)
        copy_file(BENCH_SRC, RUST_STABLE_BENCH_DST, traclus_args.get_name())

def get_files_with_all_substring(folder: str, substring: list[str], exclude: list[str] = []) -> list:
    names_folder = get_list_of_files_name(folder)
    names_substring = [name for name in names_folder if all(sub in name for sub in substring) and not any(exc in name for exc in exclude)]
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

def get_newest_rust_executable() -> str:
    exe_files = [
        f for f in os.listdir(RUST_STABLE_DIR)
        if f.startswith("rust_impl_V") and f.endswith(".exe" if os.name == "nt" else "")
    ]

    def version_key(f):
        try:
            name = f.replace("rust_impl_V", "").replace(".exe", "")
            return tuple(int(x) for x in name.split("."))
        except:
            return (0,)

    exe_files.sort(key=version_key, reverse=True)
    exe_path = os.path.join(RUST_STABLE_DIR, exe_files[0])
    return exe_path

# =====================================================
#                 GET OUTPUT FILES
# =====================================================
def get_python_output_files():
    try:
        file_corridor = PYTHON_BENCH_DST + "/" + get_files_with_all_substring(PYTHON_BENCH_DST, ["corridor"])[0]
        file_segment = PYTHON_BENCH_DST + "/" + get_files_with_all_substring(PYTHON_BENCH_DST, ["segment"])[0]
    except IndexError:
        print("Error: Could not find required output files for Python implementation.")
        sys.exit(1)
    return file_corridor, file_segment

def get_rust_output_files(mode: str):
    try:
        file_corridor = RUST_BENCH_DST + "/" + get_files_with_all_substring(RUST_BENCH_DST, ["corridor", mode])[0]
        file_segment_old = RUST_BENCH_DST + "/" + get_files_with_all_substring(RUST_BENCH_DST, ["segment", "old", mode])[0]
        file_segment_new = RUST_BENCH_DST + "/" + get_files_with_all_substring(RUST_BENCH_DST, ["segment", mode], exclude=["old"])[0]
    except IndexError:
        print("Error: Could not find required output files for Rust implementation.")
        sys.exit(1)
    return file_corridor, file_segment_old, file_segment_new

def get_stable_rust_output_files(mode: str):
    try:
        file_corridor = RUST_STABLE_BENCH_DST + "/" + get_files_with_all_substring(RUST_STABLE_BENCH_DST, ["corridor", mode])[0]
        file_segment_old = RUST_STABLE_BENCH_DST + "/" + get_files_with_all_substring(RUST_STABLE_BENCH_DST, ["segment", "old", mode])[0]
        file_segment_new = RUST_STABLE_BENCH_DST + "/" + get_files_with_all_substring(RUST_STABLE_BENCH_DST, ["segment", mode], exclude=["old"])[0]
    except IndexError:
        print("Error: Could not find required output files for stable Rust implementation.")
        sys.exit(1)
    return file_corridor, file_segment_old, file_segment_new

# =====================================================
#                 STATISTICS CALCULATIONS
# =====================================================

def file_information(impl: str, mode: dict = {"name": "NONE"}) -> dict:
    file_corridor = None
    file_segment = None

    if impl == "python":
        file_corridor, file_segment = get_python_output_files()
    elif impl == "rust":
        file_corridor, file_segment, _ = get_rust_output_files(mode['name'])
    elif impl == "stable_rust":
        file_corridor, file_segment, _ = get_stable_rust_output_files(mode['name'])
    
    return calculate_file_information(file_corridor, file_segment)

def similaty_index() -> dict:
    file_segment_py = PYTHON_BENCH_DST + "/" + get_files_with_all_substring(PYTHON_BENCH_DST, ["segment"])[0]
    file_segment_rust = RUST_BENCH_DST + "/" + get_files_with_all_substring(RUST_BENCH_DST, ["segment", "old"])[0]

    return calculate_similaty_index(file_segment_py, file_segment_rust)

def full_output_similarity_rust() -> dict:
    file_segment_stable = get_stable_rust_output_files("ParallelRayon")[2]
    file_segment_new = get_rust_output_files("ParallelRayon")[2]

    return calculate_exact_output_information(file_segment_stable, file_segment_new)

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

def set_newest_rust_executable():
    print("=== Finding Newest Rust Executable ===")

    global newest_version_exe
    newest_version_exe = get_newest_rust_executable()
    version_name = os.path.basename(newest_version_exe)
    print(f"Newest Rust executable found: {version_name}")

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

def run_rust_impl_once(args: ArgumentsTraclus, mode: str = "serial"):
    exe = os.path.join(RUST_IMPL_DIR, "target", "release", "rust_impl" + (".exe" if os.name == "nt" else ""))
    cmd = [
        exe,
        "--file", os.path.join(RUST_IMPL_DIR, args.get_path()),
        "--max_dist", args.get_args_value('max_dist'),
        "--min_density", args.get_args_value('min_density'),
        "--max_angle", args.get_args_value('max_angle'),
        "--segment_size", args.get_args_value('seg_size'),
        "--mode", mode,
        "--interface", "performance"
    ]   
    
    results = subprocess.run(cmd, capture_output=True, text=True)
    
def run_stable_rust_impl_once(args: ArgumentsTraclus, cmd: str = "serial"):
    cmd_list = [
        newest_version_exe,
        "--file", os.path.join(RUST_STABLE_DIR, args.get_path()),
        "--max_dist", args.get_args_value('max_dist'),
        "--min_density", args.get_args_value('min_density'),
        "--max_angle", args.get_args_value('max_angle'),
        "--segment_size", args.get_args_value('seg_size'),
        "--mode", cmd,
        "--interface", "performance"
    ]
    
    results = subprocess.run(cmd_list, capture_output=True, text=True)

def run_timed_once(impl: str, args: ArgumentsTraclus, mode: dict = {"name": "NONE"}):
    remove_and_copy_input_file(args, impl)
    run_start = perf_counter()
    
    if impl == "python":
        run_python_impl_once(args)
    elif impl == "rust":
        run_rust_impl_once(args, mode['cmd'])
    elif impl == "stable_rust":
        run_stable_rust_impl_once(args, mode['cmd'])

    run_end = perf_counter()
    time = run_end - run_start

    information = file_information(impl, mode)

    print(f"\n=> {impl.upper()} implementation ({mode['name']}) ===")
    print(f"\tArgument Set {args.get_args()}")
    print(f"\t\033[1;32mExecution time: {time:.6f} seconds\033[0m")
    print(f"\tNb of corr: {information['number_of_corridors']}, "
          f"Nb of seg: {information['number_of_segments']}, "
          f"Nb of non-clustered seg: {information['number_of_non_clustered_segments']}")
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

    print("=" * 20)
    print(f"\nTotal {impl} mode {mode['name']} execution time: {total_time:.6f} seconds \n")
    print("=" * 20)
    return outputs

# =====================================================
#                 TEST IMPLEMENTATIONS
# =====================================================

def visual_testing(traclus_args: ArgumentsTraclus, rust_mode: list):
    if not os.path.exists(RESULTS_QGIS_DIR):
        print(f"Error: Required folder to run the visual testing'{RESULTS_QGIS_DIR}' does not exist.")
        sys.exit(1)
    
    while True:
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

    try: # Keep the benchmarking results even if an error occurs during the process
        for (index,size) in enumerate(list_of_sizes):
            file_name = base_file.replace("$NB$", str(size))
            
            args_copy = args.copy()
            args_copy['path'] = [file_name]
            args_copy['min_density'] = [size//3]
            traclus_args = ArgumentsTraclus("benchmarked_data", args_copy, print_as_text=False)

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

def verify_solution_and_performance_gain():
    args_small_samples = {
        'max_dist':     [600, 800, 1000] * 2,
        'min_density':  [1500],
        'max_angle':    [5, 4, 3] * 2,
        'seg_size':     [1000, 900, 800] * 2,
        'path': ["enquete_od_DL_5000_traclus.txt" ],
    }
    args_big_samples = {
        'max_dist':     [600] * 6,
        'min_density':  [2666],
        'max_angle':    [5] * 6,
        'seg_size':     [3000] * 6,
        'path': ["enquete_od_DL_9000_traclus.txt" ],
    }

    args = ArgumentsTraclus("benchmarked_data", args_small_samples)
    rust_mode = {'cmd': 'parallel-rayon', 'name': 'ParallelRayon'}

    nb, tot_time_stable, tot_time_new = 0, 0, 0
    while True:
        output_rust_new = run_timed_once("rust", args, rust_mode)
        tot_time_new += output_rust_new["time"]
        output_rust_stable = run_timed_once("stable_rust", args, rust_mode)
        tot_time_stable += output_rust_stable["time"]

        full_output_similarity_rust()
        print(f"\n")

        nb += 1
        if args.iter_arguments() is False:
            break 
    
    print(f"Average execution time for stable Rust: {tot_time_stable/nb:.6f} seconds")
    print(f"Average execution time for new Rust: {tot_time_new/nb:.6f} seconds")

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
        'path': ["enquete_od_DL_1000_traclus.txt" ],
    }
    rust_mode = [{'cmd': 'parallel-rayon', 'name': 'ParallelRayon'},
                 {'cmd': 'serial', 'name': 'Serial'}]
    traclus_args = ArgumentsTraclus("benchmarked_data", args_values)

    build_python_impl()
    build_rust_impl()
    set_newest_rust_executable()

    print("\n=== Starting Benchmarks ===")

    if args_cli.mode == "visual":
        visual_testing(traclus_args, rust_mode)
    elif args_cli.mode == "time":
        time_testing(traclus_args, rust_mode)
    elif args_cli.mode == "multi-od":
        run_averaged_multi_OD(args_values, rust_mode)
    elif args_cli.mode == "verify":
        verify_solution_and_performance_gain()