#!/bin/bash
#SBATCH --job-name=traclus_unit_tests
#SBATCH --output=traclus_tests_%j.out   # %j = job ID
#SBATCH --error=traclus_tests_%j.err
#SBATCH --time=03:00:00                 
#SBATCH --cpus-per-task=50
#SBATCH --mem-per-cpu=2G               

# Ensure a clean module environment
module purge

# TODO: load the modules your project needs, e.g.:
module load python/3.11

# Activate your virtual environment if you have one
source ~/envs/traclus/bin/activate

echo "Job started at: $(date)"
echo "Running on node: $SLURMD_NODENAME"
echo "CPUs allocated: $SLURM_CPUS_PER_TASK"

py runner_unit_tests.py -m all-can

echo "Job finished at: $(date) with exit code $?"