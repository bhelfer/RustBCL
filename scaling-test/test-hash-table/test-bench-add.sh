#!/usr/bin/env bash
#srun -N 1 -n 6 ./target/release/main
base_scale=$((2**17))
#
#echo "strong scaling" | tee -a "test-bench-add.out"
#
#for i in 1 2 4 8 12 16 
#do
#    echo "N = $i, n = $(($i * 4)), local size = $((${base_scale} / $(($i * $i * 4))))" | tee -a "test-bench-add.out"
#    srun -N $i -n $(($i * 4)) ./target/release/main $((${base_scale} / $(($i * $i * 4)))) | tee -a "test-bench-add.out" 
#done

#echo "weak scaling" | tee -a "test-bench-add.out"

for i in 1 2 4 8 12 16
do
    echo "N = $i, n = $(($i * 4)), local size = ${base_scale}" | tee -a "test-bench-add.out"
    srun -N $i -n $(($i * 4)) ./target/release/main ${base_scale} | tee -a "test-bench-add.out" 
done

