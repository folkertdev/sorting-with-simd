mod lib;

use rand::{distributions::Uniform, Rng}; // 0.8.0

fn random_numbers() -> Vec<i32> {
    let range = Uniform::from(i32::MIN..i32::MAX);
    rand::thread_rng().sample_iter(&range).take(1_000).collect()
}

fn main() {
    for _ in 0..1000 {
        let input = random_numbers();

        let mut a = input.clone();
        gueron2015::sort(&mut a);

        let mut b = input;
        b.sort_unstable();

        assert!(a == b);
    }
}
