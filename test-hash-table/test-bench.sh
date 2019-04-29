#srun -N 1 -n 6 ./target/release/main
base_scale=$((2**17))

echo "strong scaling" | tee -a "test-bench.out"

for i in 1 2 4 8 12 16 32
do
    echo "N = 1, n = $i, local size = $((${base_scale} / $i))" | tee -a "test-bench.out"
    srun -N 1 -n $i ./target/release/main $((${base_scale} / $i)) | tee -a "test-bench.out" 
done

for i in 1 2 4 8 12 16 32
do
    echo "N = $i, n = 32, local size = $((${base_scale} / $(($i * 32))))" | tee -a "test-bench.out"
    srun -N $i -n 32 ./target/release/main $((${base_scale} / $(($i * 32)))) | tee -a "test-bench.out" 
done


echo "weak scaling" | tee -a "test-bench.out"

for i in 1 2 4 8 12 16 32
do
    echo "N = 1, n = $i, local size = ${base_scale}" | tee -a "test-bench.out"
    srun -N 1 -n $i ./target/release/main ${base_scale} | tee -a "test-bench.out" 
done

for i in 1 2 4 8 12 16 32
do
    echo "N = $i, n = 32, local size = ${base_scale}" | tee -a "test-bench.out"
    srun -N $i -n 32 ./target/release/main ${base_scale} | tee -a "test-bench.out" 
done

