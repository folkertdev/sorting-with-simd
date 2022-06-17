#![feature(stdsimd)]

mod lib;

use lib::partition_avx512;
use rand::{distributions::Uniform, Rng}; // 0.8.0

fn random_numbers() -> Vec<i32> {
    let range = Uniform::from(i32::MIN..i32::MAX);
    rand::thread_rng().sample_iter(&range).take(1_000).collect()
}

fn main() {
    let mut elements = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 8];
    let mut scratchpad = [0; 16];

    let x = unsafe { partition_avx512(&mut elements, &mut scratchpad) };

    dbg!(x);

    //    for _ in 0..1000 {
    //        let input = random_numbers();
    //
    //        //    let mut a = input.clone();
    //        //    gueron2015::sort_old(&mut a);
    //
    //        let mut a = input.clone();
    //        gueron2015::sort(&mut a);
    //
    //        let mut b = input;
    //        b.sort_unstable();
    //
    //        assert!(a == b);
    //    }
}
