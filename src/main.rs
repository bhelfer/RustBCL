mod shmemx;

fn main() {
    // The statements here will be executed when the compiled binary is called

    // Print text to the console
    shmemx::init();
    println!("Hello World! I am process {} out of {}",
             shmemx::my_pe(), shmemx::n_pes());
    shmemx::barrier();
    shmemx::finalize();
}
