pub mod array;
pub mod comm;
pub mod config;
pub mod global_pointer;
pub mod hash_table;
pub mod queue;
pub mod shmemx;
pub mod utils;

#[cfg(test)]
mod tests {

//    use config::Config;
//    use array::Array;
//    use comm;

//    #[test]
//    fn it_works() {
//        let mut config = Config::init(1);
//        let rankn = config.rankn;
//
//        if config.rank == 0 { println!("\n\n------------Array's test------------\n"); }
//        let rankn = config.rankn;
//
//        let mut arr = Array::<char>::init(&mut config, rankn);
//        arr.write(('a' as u8 + config.rank as u8) as char, config.rank);
//
//        comm::barrier();
//
//        if config.rank == 0 {
//            for i in 0..config.rankn {
//                println!("{}: {}", i, arr.read(i));
//            }
//        }
//    }

}
