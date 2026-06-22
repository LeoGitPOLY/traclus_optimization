#!/bin/bash
#SBATCH --job-name=traclus_unit_tests
#SBATCH --output=logs/traclus_tests_%j.out
#SBATCH --error=logs/traclus_tests_%j.err
#SBATCH --time=00:30:00                 
#SBATCH --cpus-per-task=50
#SBATCH --mem-per-cpu=2G               

# Ensure a clean module environment
module purge
module load python/3.11

# Install Rust if not available
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Activate your virtual environment
python -m venv ~/envs/traclus
source ~/envs/traclus/bin/activate
pip install -r requirements.txt

echo "Job started at: $(date)"
echo "Running on node: $SLURMD_NODENAME"
echo "CPUs allocated: $SLURM_CPUS_PER_TASK"

mkdir -p logs
python runner_unit_tests.py -m all-can

echo "Job finished at: $(date) with exit code $?"