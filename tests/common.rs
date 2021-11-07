use rand::{rngs::StdRng, RngCore, SeedableRng};

pub fn data_region(size: usize, seed: u64) -> Vec<u8> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut res = vec![0; size];
    rng.fill_bytes(&mut res);
    res
}
