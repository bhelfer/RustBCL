# run 'build-bench-cori.sh' on the shell without interactive session.
# run 'run-bench-cori.sh' on the shell with interactive session.
# check https://bheisler.github.io/criterion.rs/book/criterion_rs.html for how to write benchmark file.

# srun -N 1 -n 4 ./target/release/benchmark

# for n in 1 2 4 8 16 32
# do
#    srun -N 1 -n $n ./target/release/main
# done

for N in 2 4 8 16 32
do
    srun -N $N -n $(($N*32)) ./target/release/main
done