#!/bin/bash
#SBATCH --job-name=traclus_unit_tests
#SBATCH --output=logs/traclus_tests_%j.out
#SBATCH --error=logs/traclus_tests_%j.err
#SBATCH --time=12:00:00
#SBATCH --nodes=1
#SBATCH --cpus-per-task=191
#SBATCH --mem-per-cpu=2G

# Ensure a clean module environment
module purge
module load python/3.11

# Activate your virtual environment
python -m venv ~/envs/traclus
source ~/envs/traclus/bin/activate
pip install -r requirements.txt

echo "Job started at: $(date)"
echo "Running on node: $SLURMD_NODENAME"
echo "CPUs allocated: $SLURM_CPUS_PER_TASK"

mkdir -p logs

for CPUS in $(seq 1 10 191); do
    echo "=========================================="
    echo "Running benchmark with $CPUS Rayon threads"
    echo "=========================================="

    export RAYON_NUM_THREADS=$CPUS
    export OMP_NUM_THREADS=$CPUS

    BENCHMARK_INFO="Node=${SLURMD_NODENAME};RequestedCPUs=${CPUS};AllocatedCPUs=${SLURM_CPUS_PER_TASK};MemPerCPU=${SLURM_MEM_PER_CPU};TimeLimit=${SLURM_TIME_LIMIT};Date=$(date +%F)"

    python runner_unit_tests.py \
        -m all-can \
        -i "$BENCHMARK_INFO"
done

echo "Job finished at: $(date) with exit code $?"