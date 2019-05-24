#!/bin/bash -l
#SBATCH -C haswell
#SBATCH -p debug      # change this option for non-debug runs
#SBATCH -N 2
#SBATCH -t 00:10:00   # adjust the amount of time as necessary
#SBATCH -J RustBCL
#SBATCH -o RustBCL.%j.stdout
#SBATCH -e RustBCL.%j.error

mv run-release-cori.out run-release-cori.out.old
for n in 1 2 4 8 16 32
do
#   echo "N = 1, n = $n" | tee -a "run-release-cori.out"
   srun -N 1 -n $n ./target/release/main | tee -a "run-release-cori.out"
done

#for N in 2 4 8 16 32
#do
#    echo "N = $N, n = $(($N*32))" | tee -a "run-release-cori.out"
#    srun -N $N -n $(($N*32)) ./target/release/main | tee -a "run-release-cori.out"
#done
