import subprocess
import sys
import os
import argparse
import shutil
from time import perf_counter
from turtle import pd
from datetime import datetime
import pandas as pd
from openpyxl import load_workbook
from arguments_traclus import ArgumentsBenchmark, ArgumentsTraclus
from measurements import calculate_exact_output_information, calculate_file_information, calculate_similarity_index

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

ALLIANCE_CAN_BENCH_DST = os.path.join(ROOT_DIR, "..", "..", "alliance_can_data")

newest_version_exe = "" # Will be set after building the Rust implementation

if os.name == "nt":
    RUST_EXECUTABLE += ".exe"



def parse_args():
    parser = argparse.ArgumentParser()

    parser.add_argument(
        "-m", "--mode",
        choices=["visual", "time", "verify", "verify-sim", "all-can-t", "all-can-od"],
        default = "time",
        help="Run mode [visual, time, default: time]"
    )
    parser.add_argument(
        "-i", "--info",
        type=str,
        default="",
        required=False,
        help="Benchmark information string"
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

def transfert_files_to_qgis_results(traclus_args: ArgumentsTraclus, rust_mode: list, include_stable: bool = False):
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
        rust_seg_new = get_files_with_all_substring(RUST_BENCH_DST, ["segment", name], exclude=["old"])[0]
        rust_corr = get_files_with_all_substring(RUST_BENCH_DST, ["corridor", name])[0]
        
        create_file(RUST_BENCH_DST, RESULTS_QGIS_DIR, rust_seg_old, f"SEG_RUST_{name}.txt")
        create_file(RUST_BENCH_DST, RESULTS_QGIS_DIR, rust_seg_new, f"SEG_RUST_NEW_{name}.txt")
        create_file(RUST_BENCH_DST, RESULTS_QGIS_DIR, rust_corr, f"CORRIDOR_RUST_{name}.txt")
    
    if include_stable:
        name_stable_seg_old = get_files_with_all_substring(RUST_STABLE_BENCH_DST, ["segment", "old", "ParallelRayon"])[0]
        name_stable_seg_new = get_files_with_all_substring(RUST_STABLE_BENCH_DST, ["segment", "ParallelRayon"], exclude=["old"])[0]
        name_stable_corr = get_files_with_all_substring(RUST_STABLE_BENCH_DST, ["corridor", "ParallelRayon"])[0]

        create_file(RUST_STABLE_BENCH_DST, RESULTS_QGIS_DIR, name_stable_seg_old, f"SEG_RUST_STABLE_OLD.txt")
        create_file(RUST_STABLE_BENCH_DST, RESULTS_QGIS_DIR, name_stable_seg_new, f"SEG_RUST_STABLE_NEW.txt")
        create_file(RUST_STABLE_BENCH_DST, RESULTS_QGIS_DIR, name_stable_corr, f"CORRIDOR_RUST_STABLE.txt")

def get_newest_rust_executable(version: str = None) -> str:
    exe_files = [
        f for f in os.listdir(RUST_STABLE_DIR)
        if f.startswith("rust_impl_V") and f.endswith(".exe" if os.name == "nt" else "")
    ]

    if not exe_files:
        raise FileNotFoundError("No Rust executable files found in RUST_STABLE_DIR")

    if version:
        exe_files = [f for f in exe_files if version in f]
        if not exe_files:
            raise FileNotFoundError(f"No Rust executable files found matching version: {version}")

    def version_key(f):
        try:
            name = f.replace("rust_impl_V", "").replace(".exe", "")
            return tuple(int(x) for x in name.split("."))
        except:
            return (0,)

    exe_files.sort(key=version_key, reverse=True)
    exe_path = os.path.join(RUST_STABLE_DIR, exe_files[0])
    return exe_path

def save_outputs_to_excel(outputs: dict, sheet_name: str):
    create_empty_folder("outputs")
    file_name = "outputs/benchmark_results.xlsx"

    df = pd.DataFrame(outputs)

    # Excel sheet names are limited to 31 characters
    sheet_name = sheet_name[:31]

    file_exists = os.path.exists(file_name)

    if file_exists:
        with pd.ExcelWriter(
            file_name,
            engine="openpyxl",
            mode="a",
            if_sheet_exists="replace",
        ) as writer:
            df.to_excel(writer, sheet_name=sheet_name, index=False)
    else:
        with pd.ExcelWriter(
            file_name,
            engine="openpyxl",
            mode="w",
        ) as writer:
            df.to_excel(writer, sheet_name=sheet_name, index=False)

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

def get_perf_info_from_stout(stdout: str) -> dict:
    perf_lines = []
    total_time = None

    for line in stdout.splitlines():
        stripped_line = line.strip()

        if not stripped_line.startswith("[PERF]"):
            continue

        if "TOTAL" in stripped_line.split():
            parts = stripped_line.split(":")
            total_str = parts[-1].strip().split()[0]
            total_time = float(total_str) / 1000
            continue

        cleaned_line = stripped_line[len("[PERF]"):].strip()
        perf_lines.append(cleaned_line)

    return {"total_time_perf": total_time, "complete_perf": "\n".join(perf_lines)}

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

def full_output_similarity_python_vs_rust(mode: dict = {"name": "ParallelRayon"}) -> dict:
    print(f"\n=> PYTHON similimarity vs STABLE_RUST ===")
    _, file_segment_py = get_python_output_files()
    _, file_segment_rust, _ = get_rust_output_files(mode['name'])

    calculate_exact_output_information(file_segment_py, file_segment_rust)
    sim_results = calculate_similarity_index(file_segment_py, file_segment_rust)
    print(sim_results)
    return sim_results

def full_output_similarity_rust() -> dict:
    print(f"\n=> RUST similimarity vs STABLE_RUST ===")
    _, _, file_segment_stable= get_stable_rust_output_files("ParallelRayon")
    _, _, file_segment_new = get_rust_output_files("ParallelRayon")

    calculate_exact_output_information(file_segment_stable, file_segment_new, "new_format")
    sim_results = calculate_similarity_index(file_segment_stable, file_segment_new, "new_format")
    print(sim_results)
    return sim_results


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

def set_newest_rust_executable(version: str = None):
    print("=== Finding Newest Rust Executable ===")

    global newest_version_exe
    newest_version_exe = get_newest_rust_executable(version)
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
    return results.stdout

def run_rust_impl_once(args: ArgumentsTraclus, bench: ArgumentsBenchmark, mode: str = "serial"):
    exe = os.path.join(RUST_IMPL_DIR, "target", "release", "traclusdl_cli" + (".exe" if os.name == "nt" else ""))
    cmd_list = [
        exe,
        "--file", os.path.join(RUST_IMPL_DIR, args.get_path()),
        "--max_dist", args.get_args_value('max_dist'),
        "--min_density", args.get_args_value('min_density'),
        "--max_angle", args.get_args_value('max_angle'),
        "--segment_size", args.get_args_value('seg_size'),
        "--mode", mode,
        "--interface", bench.interface,
        "--max_threads", str(bench.nb_cores)
    ]   
    
    results = subprocess.run(cmd_list, capture_output=True, text=True, encoding="utf-8", errors="replace")
    return results.stdout
    
def run_stable_rust_impl_once(args: ArgumentsTraclus, bench: ArgumentsBenchmark, mode: str = "serial"):
    cmd_list = [
        newest_version_exe,
        "--file", os.path.join(RUST_STABLE_DIR, args.get_path()),
        "--max_dist", args.get_args_value('max_dist'),
        "--min_density", args.get_args_value('min_density'),
        "--max_angle", args.get_args_value('max_angle'),
        "--segment_size", args.get_args_value('seg_size'),
        "--mode", mode,
        "--interface", bench.interface
    ]
    print(newest_version_exe[-5:-4])
    if int(newest_version_exe[-5:-4]) > 2:
        cmd_list += ["--max_threads", str(bench.nb_cores)]

    results = subprocess.run(cmd_list, capture_output=True, text=True, encoding="utf-8", errors="replace")
    return results.stdout

def run_timed_once(impl: str, args: ArgumentsTraclus, bench: ArgumentsBenchmark, mode: dict = {"name": "NONE"}) -> dict:
    remove_and_copy_input_file(args, impl)
    run_start = perf_counter()
    
    stdout = ""
    if impl == "python":
        stdout = run_python_impl_once(args)
    elif impl == "rust":
        stdout = run_rust_impl_once(args, bench, mode['cmd'])
    elif impl == "stable_rust":
        stdout = run_stable_rust_impl_once(args, bench, mode['cmd'])

    run_end = perf_counter()
    time = run_end - run_start

    perf = get_perf_info_from_stout(stdout)
    information = file_information(impl, mode)
    print(stdout)

    print(f"\n=> {impl.upper()} implementation ({mode['name']}) ===")
    print(f"\tArgument Set {args.get_args()}")
    print(f"\t\033[1;32mExecution time: {time:.6f} seconds\033[0m")
    print(f"\tNb of corr: {information['number_of_corridors']}, "
          f"Nb of seg: {information['number_of_segments']}, "
          f"Nb of non-clustered seg: {information['number_of_non_clustered_segments']}")
    return {"impl": impl, "mode": mode['name'], "args": args.get_args(), "time": time, **perf, **information}
    
def run_timed_all(impl: str, args: ArgumentsTraclus, mode: dict = {"name": "NONE"}):
    outputs = []
    total_time = 0.0

    while True:
        output = run_timed_once(impl, args, mode)
        outputs.append(output)
        total_time += output["time"]

        if args.iter_arguments() is False:
            break  

    print(f"\n[Total {impl} mode {mode['name']} execution time: {total_time:.6f} seconds ]\n")
    return outputs

# =====================================================
#                 TEST IMPLEMENTATIONS
# =====================================================

def visual_testing():
    args_values = {
        'max_dist':     [600],
        'min_density':  [7, 10, 15, 20],
        'max_angle':    [7],
        'seg_size':     [2000],
        'path': ["donnes_taxi_DL_2000_traclus.txt" ],
    }
    rust_mode = [{'cmd': 'parallel-rayon', 'name': 'ParallelRayon'}]
    traclus_args = ArgumentsTraclus("benchmarked_data", args_values)
    
    if not os.path.exists(RESULTS_QGIS_DIR):
        print(f"Error: Required folder to run the visual testing'{RESULTS_QGIS_DIR}' does not exist.")
        sys.exit(1)
    
    while True:
        # TESTING PYTHON
        # run_timed_once("python", traclus_args)
        # TESTING ALL MODE RUST
        for mode in rust_mode: run_timed_once("rust", traclus_args, mode)
        
        transfert_files_to_qgis_results(traclus_args, rust_mode)

        print(f"=== Visual results are ready for argument set {traclus_args.get_args()} ===")

        user_input = input("\nPress Enter to continue to the next argument set (or 's' to stop)...\n")
        if user_input.lower() == 's':
            break
        if traclus_args.iter_arguments() is False:
            break

def time_testing():
    args= {
        'max_dist':     [600],
        'min_density':  [333],
        'max_angle':    [5],
        'seg_size':     [3000],
        'path': ["enquete_od_DL_1000_traclus.txt"],
    }

    traclus_args = ArgumentsTraclus("benchmarked_data", args)
    rust_mode = [{'cmd': 'parallel-rayon', 'name': 'ParallelRayon'}]
    
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

def verify_similarity_index():
    args_order_verify = {
        'max_dist':     [600],
        'min_density':  [30],
        'max_angle':    [6],
        'seg_size':     [1000],
        'path': ["enquete_od_DL_10_traclus.txt"],
    }

    args = ArgumentsTraclus("benchmarked_data", args_order_verify)
    rust_mode = [{'cmd': 'parallel-rayon', 'name': 'ParallelRayon'}]

    set_newest_rust_executable("V1.0.0")
    
    outputs_python_vs_rust = []
    outputs_rust_vs_stable = []
    
    while True:
        run_timed_once("rust", args, rust_mode[0])
        run_timed_once("python", args)
        run_timed_once("stable_rust", args, rust_mode[0])

        outputs_python_vs_rust.append(full_output_similarity_python_vs_rust(rust_mode[0]))
        outputs_rust_vs_stable.append(full_output_similarity_rust())

        transfert_files_to_qgis_results(rust_mode, True)
        input("\nPress Enter to continue to the next argument set (or 's' to stop)...\n")

        if args.iter_arguments() is False:
            break
    
    print("\n=== Similarity Index Results for Python vs Rust ===")
    average_sim = (0,0,0)
    for output in outputs_python_vs_rust:
        print(f"Similarity Index 1: {output['similarity_index_1']:.6f} "
              f"Similarity Index 2: {output['similarity_index_2']:.6f} "
              f"Relative Difference: {output['Relative_Difference']:.6f}")
        average_sim = (average_sim[0] + output['similarity_index_1'], average_sim[1] + output['similarity_index_2'], average_sim[2] + output['Relative_Difference'])
    print(f"\nAverage Similarity Index 1: {average_sim[0]/len(outputs_python_vs_rust):.6f}, "
          f"Average Similarity Index 2: {average_sim[1]/len(outputs_python_vs_rust):.6f}, "
          f"Average Relative Difference: {average_sim[2]/len(outputs_python_vs_rust):.6f}")

    average_sim = (0,0,0)
    for output in outputs_rust_vs_stable:
        print(f"Similarity Index 1: {output['similarity_index_1']:.6f}"
              f"Similarity Index 2: {output['similarity_index_2']:.6f}"
              f"Relative Difference: {output['Relative_Difference']:.6f}")
        average_sim = (average_sim[0] + output['similarity_index_1'], average_sim[1] + output['similarity_index_2'], average_sim[2] + output['Relative_Difference'])
    print(f"\nAverage Similarity Index 1: {average_sim[0]/len(outputs_rust_vs_stable):.6f}, "
          f"Average Similarity Index 2: {average_sim[1]/len(outputs_rust_vs_stable):.6f}, "
          f"Average Relative Difference: {average_sim[2]/len(outputs_rust_vs_stable):.6f}")

def verify_solution_and_performance_gain():
    args_small_samples = {
        'max_dist':     [600, 800, 1000] * 2,
        'min_density':  [150],
        'max_angle':    [5, 4, 3] * 2,
        'seg_size':     [1000, 900, 800] * 2,
        'path': ["enquete_od_DL_2000_traclus.txt" ],
    }
    args_big_samples = {
        'max_dist':     [600] * 2,
        'min_density':  [2666],
        'max_angle':    [5] * 2,
        'seg_size':     [3000] * 2,
        'path': ["donnes_taxi_DL_98000_traclus.txt" ],
    }

    set_newest_rust_executable("V1.0.1")

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

def alliance_canada_over_OD(info: str):
    # Overwrite the SRC_DIRECTORY with the alliance canada data
    global BENCH_SRC
    BENCH_SRC = ALLIANCE_CAN_BENCH_DST

    args_values = {
        'max_dist':     [600],
        'max_angle':    [7],
        'seg_size':     [2000],
    }
    rust_mode = [{'cmd': 'serial', 'name': 'Serial'},
                {'cmd': 'parallel-rayon', 'name': 'ParallelRayon'}]
    base_file = "donnes_taxi_DL_$NB$_traclus.txt"
    
    # start, step, n = 2000, 8000, 17
    # list_of_sizes = [start + i * step for i in range(n)] 
    list_of_sizes = [350000, 400000, 450000, 500000, 550000] 
    
    time_last_run = [0.0, 0.0, 0.0] # For python, rust serial, rust parallel
    MAX_TIME_SEC = 5 * 60 * 60 # seconds

    sheet_name = datetime.now().strftime("%Y-%m-%d_%H-%M-%S")

    outputs = []
    for (_, size) in enumerate(list_of_sizes):
        # SETTING ARGUMENTS
        args_copy = args_values.copy()
        
        file_name = base_file.replace("$NB$", str(size))
        args_copy['path'] = [file_name]
        args_copy['min_density'] = [size//250]
        traclus_args = ArgumentsTraclus("benchmarked_data", args_copy, print_as_text=False)
        bench_args = ArgumentsBenchmark(interface="perf-timer", nb_cores=200)

        print(f"\n======== Running implementations for {file_name} ===========")

        while True: # Iter over all combinations of arguments
            o_python, o_rust_serial, o_rust_parallel = None, None, None
            
            # TESTING PYTHON
            if time_last_run[0] < MAX_TIME_SEC and False:
                o_python = run_timed_once("python", traclus_args)
                time_last_run[0] = o_python["time"]

            # TESTING ALL MODE RUST
            if time_last_run[1] < MAX_TIME_SEC and False:
                o_rust_serial = run_timed_once("rust", traclus_args, bench_args, rust_mode[0])
                time_last_run[1] = o_rust_serial["time"]
            if time_last_run[2] < MAX_TIME_SEC:
                o_rust_parallel = run_timed_once("rust", traclus_args, bench_args, rust_mode[1])
                time_last_run[2] = o_rust_parallel["time"]

            # CALCULATE SIMILIARITY INDEX
            if time_last_run[0] < MAX_TIME_SEC and False:
                similarity_index = full_output_similarity_python_vs_rust()
                o_python = o_python | similarity_index
                o_rust_serial = o_rust_serial | similarity_index
                o_rust_parallel = o_rust_parallel | similarity_index
            
            # STORE OUTPUTS
            if o_python is not None: outputs.append(o_python | {"info": info})
            if o_rust_serial is not None: outputs.append(o_rust_serial | {"info": info})
            if o_rust_parallel is not None: outputs.append(o_rust_parallel | {"info": info})

            # STORE OUTPUTS TO EXCEL FILE (avoid losing data)
            outputs_sorted = sorted(outputs, key=lambda x: (x['impl'], x['mode']))
            save_outputs_to_excel(outputs_sorted, sheet_name)

            if traclus_args.iter_arguments() is False:
                break   

def alliance_canada_over_threads(info: str):
    # Overwrite the SRC_DIRECTORY with the alliance canada data
    global BENCH_SRC
    BENCH_SRC = ALLIANCE_CAN_BENCH_DST

    args_values = {
        'max_dist':     [600],
        'max_angle':    [5,7],
        'min_density':  [520],
        'seg_size':     [2000],
        'path': ["donnes_taxi_DL_130000_traclus.txt"],
    }
    rust_mode = {'cmd': 'parallel-rayon', 'name': 'ParallelRayon'}
    sheet_name = datetime.now().strftime("%Y-%m-%d_%H-%M-%S")
    

    start, step, n = 1, 10, 19
    list_of_max_threads = [start + i * step for i in range(n)] 
    
    outputs = []
    for (_, max_threads) in enumerate(list_of_max_threads):
        # SETTING ARGUMENTS
        args_copy = args_values.copy()
        bench_args = ArgumentsBenchmark(interface="perf-timer", nb_cores=max_threads)
        traclus_args = ArgumentsTraclus("benchmarked_data", args_copy, print_as_text=False)
        
        while True: # Iter over all combinations of arguments
            o_rust_parallel = run_timed_once("rust", traclus_args, bench_args, rust_mode)
            info_and_threads = info + f";max_threads={max_threads}"
            outputs.append(o_rust_parallel | {"info": info_and_threads})


            if traclus_args.iter_arguments() is False:
                break   

        # STORE OUTPUTS TO EXCEL FILE (avoid losing data)
        outputs_sorted = sorted(outputs, key=lambda x: (x['impl'], x['mode']))
        save_outputs_to_excel(outputs_sorted, sheet_name)

# =====================================================
#                 MAIN
# =====================================================

if __name__ == "__main__":
    args_cli = parse_args()
   
    build_python_impl()
    build_rust_impl()
    set_newest_rust_executable()

    if args_cli.mode == "visual":
        visual_testing()
    elif args_cli.mode == "time":
        time_testing()
    elif args_cli.mode == "verify":
        verify_solution_and_performance_gain()
    elif args_cli.mode == "verify-sim":
        verify_similarity_index()
    elif args_cli.mode == "all-can-t":
        alliance_canada_over_threads(args_cli.info)
    elif args_cli.mode == "all-can-od":
        alliance_canada_over_OD(args_cli.info)

   