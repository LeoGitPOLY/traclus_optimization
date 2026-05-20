[1mdiff --git a/benchmarks/measurements.py b/benchmarks/measurements.py[m
[1mindex 217498b..961332b 100644[m
[1m--- a/benchmarks/measurements.py[m
[1m+++ b/benchmarks/measurements.py[m
[36m@@ -128,13 +128,12 @@[m [mdef compare_element(element: dict, comparison_list: list) -> tuple:[m
 [m
     for comp_element in comparison_list:[m
         is_match = ([m
[31m-            abs(element['start'][0] - comp_element['start'][0]) <= THRESHOLD and[m
[31m-            abs(element['start'][1] - comp_element['start'][1]) <= THRESHOLD[m
[32m+[m[32m            abs(element.start[0] - comp_element.start[0]) <= THRESHOLD and[m
[32m+[m[32m            abs(element.start[1] - comp_element.start[1]) <= THRESHOLD[m
         )[m
         if is_match:[m
             correspondings.append(comp_element)[m
             [m
[31m-    [m
     if len(correspondings) == 0:[m
         print(f"Element {element} is missing in comparison list.")[m
         return (0, 0, 0)  # No match found[m
[36m@@ -143,18 +142,15 @@[m [mdef compare_element(element: dict, comparison_list: list) -> tuple:[m
         return (0, 0, 0)  # Multiple matches found[m
     [m
     corresponding = correspondings[0][m
[31m-    if element['corridor_id'] != -1 and corresponding['corridor_id'] != -1:[m
[32m+[m[32m    if element.corridor_id != -1 and corresponding.corridor_id != -1:[m
         return (1, 0, 0)  # Both clustered[m
[31m-    elif element['corridor_id'] == -1 and corresponding['corridor_id'] == -1:[m
[32m+[m[32m    elif element.corridor_id == -1 and corresponding.corridor_id == -1:[m
         return (0, 1, 0)  # Both non-clustered[m
[31m-    elif element['corridor_id'] != -1 and corresponding['corridor_id'] == -1:[m
[32m+[m[32m    elif element.corridor_id != -1 and corresponding.corridor_id == -1:[m
         return (0, 0, 1)  # Only clustered in reference[m
     [m
     return (0, 0, 0)[m
 [m
[31m-# =====================================================[m
[31m-#               RUST VS RUST OUTPUT COMPARISON[m
[31m-# =====================================================[m
 # Compare two values with a numeric tolerance. Returns (is_ok, updated_offset).[m
 def _check_value_mismatch(ref_val, cmp_val, current_offset: float, context: str, max_offset: float = 10**(-1)) -> tuple[bool, float]:[m
     if str(ref_val) == str(cmp_val):[m
[36m@@ -249,25 +245,26 @@[m [mdef is_same_output_dict(reference_dict: dict, comparison_dict: dict) -> bool:[m
 #               STATISTICS CALCULATIONS[m
 # =====================================================[m
 [m
[31m-def calculate_file_information(file_path_corr_py: str, file_path_seg_py: str) -> dict:[m
[32m+[m[32mdef calculate_file_information(reference_path_corr: str, comparison_path_corr: str) -> dict:[m
     info = {[m
[31m-        "number_of_corridors": number_of_corridors(file_path_corr_py),[m
[31m-        "number_of_segments": number_of_segments(file_path_seg_py),[m
[31m-        "number_of_non_clustered_segments": number_of_non_clustered_segments(file_path_seg_py)[m
[32m+[m[32m        "number_of_corridors": number_of_corridors(reference_path_corr),[m
[32m+[m[32m        "number_of_segments": number_of_segments(comparison_path_corr),[m
[32m+[m[32m        "number_of_non_clustered_segments": number_of_non_clustered_segments(comparison_path_corr)[m
     }[m
     return info[m
 [m
[31m-def calculate_similaty_index(file_path_seg_py: str, file_path_seg_rust: str) -> dict:[m
[31m-    dict_segments_py = generate_dict_segments(file_path_seg_py)[m
[31m-    dict_segments_rust = generate_dict_segments(file_path_seg_rust)[m
[32m+[m[32mdef calculate_similarity_index(reference_path_seg: str, comparison_path_seg: str, format:str = "old_format") -> dict:[m
[32m+[m[32m    order = OLD_ORDER if format == "old_format" else NEW_ORDER[m
[32m+[m[32m    ref_dict_segments = generate_dict_segments(reference_path_seg, order=order)[m
[32m+[m[32m    comp_dict_segments = generate_dict_segments(comparison_path_seg, order=order)[m
 [m
[31m-    comparison_result_py = compare_clustered_seg_dict(dict_segments_py, dict_segments_rust)[m
[31m-    comparison_result_rust = compare_clustered_seg_dict(dict_segments_rust, dict_segments_py)[m
[32m+[m[32m    ref_results = compare_clustered_seg_dict(ref_dict_segments, comp_dict_segments)[m
[32m+[m[32m    comp_results = compare_clustered_seg_dict(comp_dict_segments, ref_dict_segments)[m
 [m
[31m-    nb_both_clustered = comparison_result_py[0] # Should be the same as comparison_result_rust[0][m
[31m-    nb_both_non_clustered = comparison_result_py[1] # Should be the same as comparison_result_rust[1][m
[31m-    nb_only_clustered_py = comparison_result_py[2] [m
[31m-    nb_only_clustered_rust = comparison_result_rust[2][m
[32m+[m[32m    nb_both_clustered = ref_results[0] # Should be the same as comparison_result_rust[0][m
[32m+[m[32m    nb_both_non_clustered = ref_results[1] # Should be the same as comparison_result_rust[1][m
[32m+[m[32m    nb_only_clustered_py = ref_results[2][m[41m [m
[32m+[m[32m    nb_only_clustered_rust = comp_results[2][m
 [m
     total_both = nb_both_clustered + nb_both_non_clustered[m
 [m
[36m@@ -286,18 +283,20 @@[m [mdef calculate_similaty_index(file_path_seg_py: str, file_path_seg_rust: str) ->[m
         "similarity_index_2": similarity_index_2[m
     }[m
     [m
[31m-def calculate_exact_output_information(file_path_seg_rust_stable: str, file_path_seg_rust_new: str):[m
[31m-    list_of_dict_rust_stable = generate_dict_segments(file_path_seg_rust_stable, order=NEW_ORDER)[m
[31m-    list_of_dict_rust_new = generate_dict_segments(file_path_seg_rust_new, order=NEW_ORDER)[m
[31m-    [m
[32m+[m[32mdef calculate_exact_output_information(reference_path_seg: str, comparison_path_seg: str, format:str = "old_format") -> None:[m
[32m+[m
     # 1. Check file output content similarity[m
[31m-    if is_same_output_file(file_path_seg_rust_stable, file_path_seg_rust_new):[m
[32m+[m[32m    if is_same_output_file(reference_path_seg, comparison_path_seg):[m
         print("✅ SAME OUTPUT: The two files have the same content.")[m
         return[m
     else:[m
         print("❌ DIFFERENT OUTPUT: The two files have different content.")[m
 [m
     # 2. Check dict output content similarity[m
[32m+[m[32m    order = OLD_ORDER if format == "old_format" else NEW_ORDER[m
[32m+[m[32m    list_of_dict_rust_stable = generate_dict_segments(reference_path_seg, order=order)[m
[32m+[m[32m    list_of_dict_rust_new = generate_dict_segments(comparison_path_seg, order=order)[m
[32m+[m
     if is_same_output_dict(list_of_dict_rust_stable, list_of_dict_rust_new):[m
         print("✅ SAME DICTS: The two dictionaries have the same content.")[m
         return[m
[36m@@ -305,4 +304,5 @@[m [mdef calculate_exact_output_information(file_path_seg_rust_stable: str, file_path[m
         print("❌ DIFFERENT DICTS: The two dictionaries have different content.")[m
 [m
 [m
[31m-    # 3. Calculate more check latter ... [m
\ No newline at end of file[m
[32m+[m[32m    # 3. Calculate more check latter ...[m[41m [m
[41m+    [m
\ No newline at end of file[m
[1mdiff --git a/benchmarks/runner_unit_tests.py b/benchmarks/runner_unit_tests.py[m
[1mindex 8bb588a..551859d 100644[m
[1m--- a/benchmarks/runner_unit_tests.py[m
[1m+++ b/benchmarks/runner_unit_tests.py[m
[36m@@ -5,7 +5,7 @@[m [mimport argparse[m
 import shutil[m
 from time import perf_counter[m
 from arguments_traclus import ArgumentsTraclus[m
[31m-from measurements import calculate_exact_output_information, calculate_file_information, calculate_similaty_index[m
[32m+[m[32mfrom measurements import calculate_exact_output_information, calculate_file_information, calculate_similarity_index[m
 [m
 # =====================================================[m
 #                 PATH CONSTANTS[m
[36m@@ -40,7 +40,7 @@[m [mdef parse_args():[m
 [m
     parser.add_argument([m
         "-m", "--mode",[m
[31m-        choices=["visual", "time", "multi-od", "verify"],[m
[32m+[m[32m        choices=["visual", "time", "multi-od", "verify", "verify-sim"],[m
         default = "time",[m
         help="Run mode [visual, time, default: time]"[m
     )[m
[36m@@ -185,17 +185,23 @@[m [mdef file_information(impl: str, mode: dict = {"name": "NONE"}) -> dict:[m
     [m
     return calculate_file_information(file_corridor, file_segment)[m
 [m
[31m-def similaty_index() -> dict:[m
[31m-    file_segment_py = PYTHON_BENCH_DST + "/" + get_files_with_all_substring(PYTHON_BENCH_DST, ["segment"])[0][m
[31m-    file_segment_rust = RUST_BENCH_DST + "/" + get_files_with_all_substring(RUST_BENCH_DST, ["segment", "old"])[0][m
[32m+[m[32mdef full_output_similarity_python_vs_rust(mode: dict = {"name": "ParallelRayon"}) -> dict:[m
[32m+[m[32m    _, file_segment_py = get_python_output_files()[m
[32m+[m[32m    _, file_segment_rust, _ = get_rust_output_files(mode['name'])[m
 [m
[31m-    return calculate_similaty_index(file_segment_py, file_segment_rust)[m
[32m+[m[32m    calculate_exact_output_information(file_segment_py, file_segment_rust)[m
[32m+[m[32m    print(calculate_similarity_index(file_segment_py, file_segment_rust))[m
[32m+[m[32m    return calculate_similarity_index(file_segment_py, file_segment_rust)[m
 [m
 def full_output_similarity_rust() -> dict:[m
[31m-    file_segment_stable = get_stable_rust_output_files("ParallelRayon")[2][m
[31m-    file_segment_new = get_rust_output_files("ParallelRayon")[2][m
[32m+[m[32m    _, _, file_segment_stable= get_stable_rust_output_files("ParallelRayon")[m
[32m+[m[32m    _, _, file_segment_new = get_rust_output_files("ParallelRayon")[m
[32m+[m
[32m+[m[32m    calculate_exact_output_information(file_segment_stable, file_segment_new, "new_format")[m
[32m+[m[32m    print(calculate_similarity_index(file_segment_stable, file_segment_new, "new_format"))[m
[32m+[m
[32m+[m
 [m
[31m-    return calculate_exact_output_information(file_segment_stable, file_segment_new)[m
 [m
 # =====================================================[m
 #                 BUILD STEP[m
[36m@@ -322,9 +328,7 @@[m [mdef run_timed_all(impl: str, args: ArgumentsTraclus, mode: dict = {"name": "NONE[m
         if args.iter_arguments() is False:[m
             break  [m
 [m
[31m-    print("=" * 20)[m
[31m-    print(f"\nTotal {impl} mode {mode['name']} execution time: {total_time:.6f} seconds \n")[m
[31m-    print("=" * 20)[m
[32m+[m[32m    print(f"\n[Total {impl} mode {mode['name']} execution time: {total_time:.6f} seconds ]\n")[m
     return outputs[m
 [m
 # =====================================================[m
[36m@@ -343,7 +347,7 @@[m [mdef visual_testing(traclus_args: ArgumentsTraclus, rust_mode: list):[m
         for mode in rust_mode: run_timed_once("rust", traclus_args, mode)[m
 [m
         # Calculate similiarity index[m
[31m-        similarity_index = similaty_index()[m
[32m+[m[32m        similarity_index = full_output_similarity_python_vs_rust()[m
         print(f"\nSimilarity Index for argument set {traclus_args.get_args()}: "[m
           f"Similarity Index 1: {similarity_index['similarity_index_1']:.6f}, "[m
           f"Similarity Index 2: {similarity_index['similarity_index_2']:.6f}\n")[m
[36m@@ -373,12 +377,19 @@[m [mdef time_testing(traclus_args: ArgumentsTraclus, rust_mode: list):[m
     for output in outputs:[m
         print(f"{output['impl']};{output['mode']};{output['args']};{output['time']:.6f}")[m
 [m
[31m-def run_averaged_multi_OD(args: dict, rust_mode: list):[m
[32m+[m[32mdef run_averaged_multi_OD():[m
[32m+[m[32m    args_values = {[m
[32m+[m[32m        'max_dist':     [600],[m
[32m+[m[32m        'max_angle':    [5,7],[m
[32m+[m[32m        'seg_size':     [3000],[m
[32m+[m[32m    }[m
[32m+[m[32m    # ,[m
[32m+[m[32m    rust_mode = [{'cmd': 'serial', 'name': 'Serial'},[m
[32m+[m[32m                {'cmd': 'parallel-rayon', 'name': 'ParallelRayon'}][m
[32m+[m[41m    [m
     base_file = "enquete_od_DL_$NB$_traclus.txt"[m
[31m-    # list_of_sizes = [1000, 2000, 3000, 4000, 5000, [m
[31m-                    #  6000, 7000, 8000, 9000, 10000, 11000, 12000, 13000, [m
[31m-                    #  14000, 15000, 16000, 17000, 18000, 19000, 20000][m
[31m-    list_of_sizes = [1000, 16000, 17000][m
[32m+[m[32m    list_of_sizes = [2000, 4000, 6000, 8000, 10000, 12000,[m[41m [m
[32m+[m[32m                     14000, 16000, 18000, 20000][m
     max_index_python = -1[m
 [m
     outputs_time = [][m
[36m@@ -388,14 +399,13 @@[m [mdef run_averaged_multi_OD(args: dict, rust_mode: list):[m
         for (index,size) in enumerate(list_of_sizes):[m
             file_name = base_file.replace("$NB$", str(size))[m
             [m
[31m-            args_copy = args.copy()[m
[32m+[m[32m            args_copy = args_values.copy()[m
             args_copy['path'] = [file_name][m
             args_copy['min_density'] = [size//3][m
             traclus_args = ArgumentsTraclus("benchmarked_data", args_copy, print_as_text=False)[m
 [m
             print(f"\n======== Running implementations for {file_name} ===========")[m
 [m
[31m-            current_outputs = {}[m
             # TESTING PYTHON[m
             if index <= max_index_python:[m
                 outputs_time += run_timed_all("python", traclus_args)[m
[36m@@ -408,7 +418,7 @@[m [mdef run_averaged_multi_OD(args: dict, rust_mode: list):[m
 [m
             # Calculate similiarity index[m
             if index <= max_index_python:[m
[31m-                similarity_index = similaty_index()[m
[32m+[m[32m                similarity_index = full_output_similarity_python_vs_rust()[m
                 outputs_similarity.append({"size":size, **similarity_index})[m
   [m
             [m
[36m@@ -426,23 +436,43 @@[m [mdef run_averaged_multi_OD(args: dict, rust_mode: list):[m
     for output in outputs_similarity:[m
         print(f"{output['size']};{output['similarity_index_1']:.6f};{output['similarity_index_2']:.6f}".replace(".", ","))[m
 [m
[32m+[m[32mdef verify_similarity_index():[m
[32m+[m[32m    args_order_verify = {[m
[32m+[m[32m        'max_dist':     [600],[m
[32m+[m[32m        'min_density':  [666],[m
[32m+[m[32m        'max_angle':    [5],[m
[32m+[m[32m        'seg_size':     [3000],[m
[32m+[m[32m        'path': ["enquete_od_DL_2000_traclus.txt" ],[m
[32m+[m[32m    }[m
[32m+[m[32m    args = ArgumentsTraclus("benchmarked_data", args_order_verify)[m
[32m+[m[32m    rust_mode = {'cmd': 'serial', 'name': 'Serial'}[m
[32m+[m
[32m+[m[32m    while True:[m
[32m+[m[32m        run_timed_once("rust", args, rust_mode)[m
[32m+[m[32m        run_timed_once("python", args)[m
[32m+[m
[32m+[m[32m        full_output_similarity_python_vs_rust(rust_mode)[m
[32m+[m
[32m+[m[32m        if args.iter_arguments() is False:[m
[32m+[m[32m            break[m[41m [m
[32m+[m
 def verify_solution_and_performance_gain():[m
     args_small_samples = {[m
         'max_dist':     [600, 800, 1000] * 2,[m
         'min_density':  [1500],[m
         'max_angle':    [5, 4, 3] * 2,[m
         'seg_size':     [1000, 900, 800] * 2,[m
[31m-        'path': ["enquete_od_DL_5000_traclus.txt" ],[m
[32m+[m[32m        'path': ["enquete_od_DL_4000_traclus.txt" ],[m
     }[m
     args_big_samples = {[m
[31m-        'max_dist':     [600] * 6,[m
[32m+[m[32m        'max_dist':     [600] * 2,[m
         'min_density':  [2666],[m
[31m-        'max_angle':    [5] * 6,[m
[31m-        'seg_size':     [3000] * 6,[m
[32m+[m[32m        'max_angle':    [5] * 2,[m
[32m+[m[32m        'seg_size':     [3000] * 2,[m
         'path': ["enquete_od_DL_9000_traclus.txt" ],[m
     }[m
 [m
[31m-    args = ArgumentsTraclus("benchmarked_data", args_small_samples)[m
[32m+[m[32m    args = ArgumentsTraclus("benchmarked_data", args_big_samples)[m
     rust_mode = {'cmd': 'parallel-rayon', 'name': 'ParallelRayon'}[m
 [m
     nb, tot_time_stable, tot_time_new = 0, 0, 0[m
[36m@@ -471,12 +501,12 @@[m [mif __name__ == "__main__":[m
     args_values = {[m
         'max_dist':     [600],[m
         'min_density':  [300],[m
[31m-        'max_angle':    [5],[m
[32m+[m[32m        'max_angle':    [5,7],[m
         'seg_size':     [1000],[m
         'path': ["enquete_od_DL_1000_traclus.txt" ],[m
     }[m
[31m-    rust_mode = [{'cmd': 'parallel-rayon', 'name': 'ParallelRayon'},[m
[31m-                 {'cmd': 'serial', 'name': 'Serial'}][m
[32m+[m[32m    rust_mode = [{'cmd': 'serial', 'name': 'Serial'},[m
[32m+[m[32m                 {'cmd': 'parallel-rayon', 'name': 'ParallelRayon'}][m
     traclus_args = ArgumentsTraclus("benchmarked_data", args_values)[m
 [m
     build_python_impl()[m
[36m@@ -490,6 +520,10 @@[m [mif __name__ == "__main__":[m
     elif args_cli.mode == "time":[m
         time_testing(traclus_args, rust_mode)[m
     elif args_cli.mode == "multi-od":[m
[31m-        run_averaged_multi_OD(args_values, rust_mode)[m
[32m+[m[32m        run_averaged_multi_OD()[m
     elif args_cli.mode == "verify":[m
[31m-        verify_solution_and_performance_gain()[m
\ No newline at end of file[m
[32m+[m[32m        verify_solution_and_performance_gain()[m
[32m+[m[32m    elif args_cli.mode == "verify-sim":[m
[32m+[m[32m        verify_similarity_index()[m
[41m+[m
[41m+   [m
\ No newline at end of file[m
[1mdiff --git a/inputs/dataset_factory.py b/inputs/dataset_factory.py[m
[1mindex ec4526e..dc0b84b 100644[m
[1m--- a/inputs/dataset_factory.py[m
[1m+++ b/inputs/dataset_factory.py[m
[36m@@ -250,20 +250,20 @@[m [mdef main():[m
 [m
     # Circle_around: lines every 30 degrees[m
     filename = BENCHMARKS_DIR / "circle_around_DL"[m
[31m-    list_of_lines = generate_desire_line_in_circle(5, SMALL_RADIUS_1.center, 1000 - EPSILON)[m
[32m+[m[32m    list_of_lines = generate_desire_line_in_circle(30, SMALL_RADIUS_1.center, 300 - EPSILON)[m
     save_to_tsv(list_of_lines, f"{filename}.tsv")[m
     save_to_traclus(list_of_lines, f"{filename}_traclus.txt")[m
     [m
     # Parallels lines: 10 lines[m
     filename = BENCHMARKS_DIR / "parallels_DL"[m
[31m-    list_of_lines = generate_vertical_parallel_lines(50, SMALL_RADIUS_1.center, 1000 - EPSILON, 10)[m
[32m+[m[32m    list_of_lines = generate_vertical_parallel_lines(15, SMALL_RADIUS_1.center, 300 - EPSILON, 6)[m
     save_to_tsv(list_of_lines, f"{filename}.tsv")[m
     save_to_traclus(list_of_lines, f"{filename}_traclus.txt")[m
 [m
     # 90 degrees lines: 10 lines[m
     filename = BENCHMARKS_DIR / "90_degrees_DL"[m
[31m-    list_of_lines = generate_horizontal_parallel_lines(50, SMALL_RADIUS_1.center, 1000 - EPSILON, 11)[m
[31m-    list_of_lines += generate_vertical_parallel_lines(50, SMALL_RADIUS_1.center, 1000 - EPSILON, 11)[m
[32m+[m[32m    list_of_lines = generate_horizontal_parallel_lines(15, SMALL_RADIUS_1.center, 300 - EPSILON, 6)[m
[32m+[m[32m    list_of_lines += generate_vertical_parallel_lines(15, SMALL_RADIUS_1.center, 300 - EPSILON, 6)[m
     for i in range(1, len(list_of_lines) + 1): list_of_lines[i-1][0] = i[m
     save_to_tsv(list_of_lines, f"{filename}.tsv")[m
     save_to_traclus(list_of_lines, f"{filename}_traclus.txt")[m
[36m@@ -273,7 +273,7 @@[m [mdef main():[m
     filename = BENCHMARKS_DIR / "enquete_od_DL"[m
     list_of_lines = convert_csv_enquete_to_list(input_file)[m
 [m
[31m-    for sample_size in [500, 1000, 2000, 3000]:[m
[32m+[m[32m    for sample_size in [10, 500, 1000, 2000, 3000]:[m
         sampled_lines = chose_random_lines(list_of_lines, sample_size)[m
         save_to_tsv(sampled_lines, f"{filename}_{sample_size}.tsv")[m
         save_to_traclus(sampled_lines, f"{filename}_{sample_size}_traclus.txt")[m
[1mdiff --git a/rust_impl/src/clustering/algorithms/base_traclusdl.rs b/rust_impl/src/clustering/algorithms/base_traclusdl.rs[m
[1mindex 07884fa..13857fd 100644[m
[1m--- a/rust_impl/src/clustering/algorithms/base_traclusdl.rs[m
[1m+++ b/rust_impl/src/clustering/algorithms/base_traclusdl.rs[m
[36m@@ -1,4 +1,3 @@[m
[31m-use std::sync::atomic::Ordering;[m
 use super::super::geometry::{segment::Segment, trajectory::Trajectory};[m
 use super::super::objects::{[m
     cluster::Cluster,[m
[36m@@ -11,6 +10,7 @@[m [muse crate::io::args::TraclusArgs;[m
 use crate::utils::events::app_events::{AppEvent, ComputationType};[m
 use crate::utils::events::event_singleton::{emit, emit_timed_perf};[m
 use crate::utils::gui_parallel_runner::StopFlag;[m
[32m+[m[32muse std::sync::atomic::Ordering;[m
 [m
 pub const TICK_EVERY: usize = 25; // how many trajectories between progress events[m
 [m
[36m@@ -99,12 +99,6 @@[m [mpub trait TraclusAlgorithm {[m
                 continue;[m
             }[m
 [m
[31m-            // emit(AppEvent::PrintInfo {[m
[31m-            //     messages: vec![format!([m
[31m-            //         "Checking distance from seed to trajectory {}, {}: ",[m
[31m-            //         nearby_traj.id, segment_id[m
[31m-            //     )],[m
[31m-            // });[m
             // Add qualifying segment as a candidate[m
             let segment: &Segment = nearby_traj.segment(segment_id).unwrap();[m
             let candidate: ClusterMember = ClusterMember::new([m
[36m@@ -248,16 +242,16 @@[m [mpub trait TraclusAlgorithm {[m
     fn emit_start_clustering(&self, raw_trajectories: &RawTrajectories) {[m
         emit(AppEvent::ComputationStart {[m
             computation_type: ComputationType::Clustering,[m
[31m-            max_progress: raw_trajectories.get_total_trajectories(),[m
[32m+[m[32m            max_progress: raw_trajectories.get_num_trajectories(),[m
         });[m
[31m-        emit_timed_perf("Clustering", true, None);[m
[32m+[m[32m        emit_timed_perf("Clustering_All", true, None);[m
     }[m
 [m
     fn emit_complete_clustering(&self) {[m
         emit(AppEvent::ComputationComplete {[m
             computation_type: ComputationType::Clustering,[m
         });[m
[31m-        emit_timed_perf("Clustering", false, None);[m
[32m+[m[32m        emit_timed_perf("Clustering_All", false, None);[m
     }[m
 [m
     fn emit_start_remove_duplicates(&self, clustered_trajectories: &ClusteredTrajectories) {[m
[1mdiff --git a/rust_impl/src/clustering/algorithms/parallel_rayon_traclusdl.rs b/rust_impl/src/clustering/algorithms/parallel_rayon_traclusdl.rs[m
[1mindex 334ac48..8c339c8 100644[m
[1m--- a/rust_impl/src/clustering/algorithms/parallel_rayon_traclusdl.rs[m
[1m+++ b/rust_impl/src/clustering/algorithms/parallel_rayon_traclusdl.rs[m
[36m@@ -89,9 +89,15 @@[m [mimpl ParallelRayonTraclusDL {[m
             let chunk_results: Vec<Vec<Cluster>> = chunk[m
                 .par_iter()[m
                 .map(|(angle_start, traj)| {[m
[32m+[m[32m                    let thread_index: Option<usize> = rayon::current_thread_index();[m
[32m+[m[32m                    emit_timed_perf("Clustering", true, thread_index);[m
[32m+[m
                     let nearby_trajs: Vec<&Trajectory> =[m
                         raw_trajectories.iter_nearby_angle(*angle_start).collect();[m
[31m-                    self.individual_trajectory_clustering(traj, &nearby_trajs)[m
[32m+[m[32m                    let clusters: Vec<Cluster> =[m
[32m+[m[32m                        self.individual_trajectory_clustering(traj, &nearby_trajs);[m
[32m+[m[32m                    emit_timed_perf("Clustering", false, thread_index);[m
[32m+[m[32m                    clusters[m
                 })[m
                 .collect();[m
 [m
[1mdiff --git a/rust_impl/src/clustering/algorithms/serial_traclusdl.rs b/rust_impl/src/clustering/algorithms/serial_traclusdl.rs[m
[1mindex b4f83fb..8a750ae 100644[m
[1m--- a/rust_impl/src/clustering/algorithms/serial_traclusdl.rs[m
[1m+++ b/rust_impl/src/clustering/algorithms/serial_traclusdl.rs[m
[36m@@ -7,6 +7,8 @@[m [muse super::super::storage::{[m
 use super::base_traclusdl::TraclusAlgorithm;[m
 [m
 use crate::io::args::TraclusArgs;[m
[32m+[m[32muse crate::utils::events::app_events::AppEvent;[m
[32m+[m[32muse crate::utils::events::event_singleton::emit;[m
 use crate::utils::gui_parallel_runner::StopFlag;[m
 [m
 pub struct SerialTraclusDL {[m
[36m@@ -47,9 +49,6 @@[m [mimpl SerialTraclusDL {[m
                     self.individual_trajectory_clustering(traj_seed, &nearby_trajs);[m
                 clustered_trajectories.add_list_cluster(clusters);[m
 [m
[31m-                // Fill all segments to be treated as non-clustered later[m
[31m-                clustered_trajectories.fill_non_clustered_segments(traj_seed);[m
[31m-[m
                 self.tick_clustering(&mut total_traj_processed);[m
 [m
                 // Check for stop signal to bail out early[m
[1mdiff --git a/rust_impl/src/clustering/main_traclusdl.rs b/rust_impl/src/clustering/main_traclusdl.rs[m
[1mindex 5f90534..7bf90b2 100644[m
[1m--- a/rust_impl/src/clustering/main_traclusdl.rs[m
[1m+++ b/rust_impl/src/clustering/main_traclusdl.rs[m
[36m@@ -37,7 +37,7 @@[m [mimpl MainTraclusDL {[m
 [m
         // Emit information about the loaded data[m
         emit(AppEvent::LoadComplete {[m
[31m-            desire_line_count: self.raw_storage.as_ref().unwrap().get_total_trajectories(),[m
[32m+[m[32m            desire_line_count: self.raw_storage.as_ref().unwrap().get_num_trajectories(),[m
             correlation_percent: directional_correlation(self.raw_storage.as_ref().unwrap()),[m
         });[m
     }[m
[1mdiff --git a/rust_impl/src/clustering/objects/cluster_member.rs b/rust_impl/src/clustering/objects/cluster_member.rs[m
[1mindex fb37301..fb2b2b3 100644[m
[1m--- a/rust_impl/src/clustering/objects/cluster_member.rs[m
[1m+++ b/rust_impl/src/clustering/objects/cluster_member.rs[m
[36m@@ -51,9 +51,16 @@[m [mimpl ClusterMember {[m
     }[m
 [m
     pub fn angle(&self) -> f64 {[m
[31m-        let dx = self.center.x - self.start.x;[m
[31m-        let dy = self.center.y - self.start.y;[m
[31m-        dy.atan2(dx).to_degrees()[m
[32m+[m[32m        let delta_x: f64 = self.center.x - self.start.x;[m
[32m+[m[32m        let delta_y: f64 = self.center.y - self.start.y;[m
[32m+[m[32m        let mut angle: f64 = delta_y.atan2(delta_x).to_degrees();[m
[32m+[m[32m        angle = (angle * 100.0).round() / 100.0;[m
[32m+[m
[32m+[m[32m        if angle < 0.0 {[m
[32m+[m[32m            angle += 360.0;[m
[32m+[m[32m        }[m
[32m+[m
[32m+[m[32m        angle[m
     }[m
 }[m
 [m
[1mdiff --git a/rust_impl/src/clustering/storage/raw_trajectories.rs b/rust_impl/src/clustering/storage/raw_trajectories.rs[m
[1mindex 7343302..4ede17a 100644[m
[1m--- a/rust_impl/src/clustering/storage/raw_trajectories.rs[m
[1m+++ b/rust_impl/src/clustering/storage/raw_trajectories.rs[m
[36m@@ -1,18 +1,6 @@[m
[31m-use std::i16::MAX;[m
[31m-[m
[31m-use eframe::App;[m
[31m-[m
[31m-use crate::utils::events::{app_events::AppEvent, event_singleton::emit};[m
[31m-[m
 use super::super::geometry::trajectory::Trajectory;[m
 [m
[31m-// TODO:[m
[31m-// - bucket size should be a fraction of the max angle threshold used in clustering[m
[31m-//   (e.g., if max angle is 5 degrees, bucket size could be 2.5 degrees to reduce sending to much neighboring buckets)[m
[31m-//     - Change constructor accordingly (easy)[m
[31m-//     - Change iter_nearby_angle accordingly (a bit more complex)[m
[31m-[m
[31m-const BUCKET_SIZE: f64 = 0.5;[m
[32m+[m[32mconst BUCKET_SIZE: f64 = 0.5; // degrees, must evenly divide 360.0[m
 [m
 pub struct Bucket {[m
     pub angle_start: f64, // (inclusive)[m
[36m@@ -37,10 +25,13 @@[m [mimpl RawTrajectories {[m
     }[m
 [m
     fn create_buckets(bucket_size: f64) -> Vec<Bucket> {[m
[32m+[m[32m        let num_buckets: usize = (360.0 / bucket_size).round() as usize;[m
         assert!(bucket_size > 0.0 && bucket_size <= 360.0);[m
[31m-        assert!(360.0 % bucket_size == 0.0, "Bucket size must be even");[m
[32m+[m[32m        assert!([m
[32m+[m[32m            (num_buckets as f64 * bucket_size - 360.0).abs() < 1e-9,[m
[32m+[m[32m            "Bucket size must evenly divide 360"[m
[32m+[m[32m        );[m
 [m
[31m-        let num_buckets: usize = (360.0 / bucket_size).ceil() as usize;[m
         let mut buckets: Vec<Bucket> = Vec::with_capacity(num_buckets);[m
 [m
         for i in 0..num_buckets {[m
[36m@@ -102,7 +93,7 @@[m [mimpl RawTrajectories {[m
             .flat_map(move |i| self.traj_buckets[i].trajectories.iter())[m
     }[m
 [m
[31m-    pub fn get_total_trajectories(&self) -> usize {[m
[32m+[m[32m    pub fn get_num_trajectories(&self) -> usize {[m
         self.traj_buckets.iter().map(|b| b.trajectories.len()).sum()[m
     }[m
 [m
[1mdiff --git a/rust_impl/src/utils/statistic.rs b/rust_impl/src/utils/statistic.rs[m
[1mindex 687508a..541e357 100644[m
[1m--- a/rust_impl/src/utils/statistic.rs[m
[1m+++ b/rust_impl/src/utils/statistic.rs[m
[36m@@ -12,7 +12,7 @@[m [muse crate::clustering::storage::raw_trajectories::RawTrajectories;[m
 /// # Arguments[m
 /// * `raw` - The raw trajectories to analyze[m
 pub fn directional_correlation(raw: &RawTrajectories) -> f64 {[m
[31m-    let total = raw.get_total_trajectories();[m
[32m+[m[32m    let total = raw.get_num_trajectories();[m
 [m
     // Edge cases[m
     if total == 0 {[m
[36m@@ -23,7 +23,7 @@[m [mpub fn directional_correlation(raw: &RawTrajectories) -> f64 {[m
     if n == 0 {[m
         return 0.0;[m
     }[m
[31m-    [m
[32m+[m
     // Only one bucket → always 1.0[m
     if n == 1 {[m
         return 1.0;[m
