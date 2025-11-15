import subprocess
import sys
import os
from time import perf_counter

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
    start = perf_counter()

    result = subprocess.run(
        ["cargo", "build", "--release", "--quiet"],
        cwd="../rust_impl",
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
def run_python_impl():
    print("=== Running Python implementation ===")
    start = perf_counter()

    result = subprocess.run(
        [sys.executable, "../python_impl/program.py"],
        capture_output=True,
        text=True
    )

    end = perf_counter()
    print("Output:", result.stdout.strip())
    print(f"Execution time: {end - start:.6f} seconds")

    return result.stdout.strip()


def run_rust_impl():
    print("=== Running Rust implementation (pre-built) ===")

    # Determine where Rust executable is
    exe_path = "../rust_impl/target/release/rust_impl"
    if os.name == "nt":
        exe_path += ".exe"

    start = perf_counter()

    result = subprocess.run(
        [exe_path],
        capture_output=True,
        text=True
    )

    end = perf_counter()
    print("Output:", result.stdout.strip())
    print(f"Execution time: {end - start:.6f} seconds")

    return result.stdout.strip()



if __name__ == "__main__":
    build_python_impl()
    build_rust_impl()
 
    print("\n")
    py_output = run_python_impl()
    rs_output = run_rust_impl()

    print("\n=== Comparison ===")
    if py_output == rs_output:
        print("✔ Outputs match")
    else:
        print("✘ Outputs differ")
        print("Python:", py_output)
        print("Rust:  ", rs_output)
