#cargo build
#oshrun -n 4 ./target/debug/main
srun -N 1 -n 1 ./target/release/main
srun -N 2 -n 2 ./target/release/main