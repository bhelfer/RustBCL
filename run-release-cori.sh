#!/bin/bash -l
#SBATCH -C haswell
#SBATCH -p debug      # change this option for non-debug runs
#SBATCH -N 2
#SBATCH -t 00:10:00   # adjust the amount of time as necessary
#SBATCH -J RustBCL
#SBATCH -o RustBCL.%j.stdout
#SBATCH -e RustBCL.%j.error
srun -N 2 -n 2 ./target/release/main
