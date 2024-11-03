use primitive_types::U512;

pub fn arr64_6_to_g1_sum(u: &[u64; 6]) -> U512 {
    U512([u[0], u[1], u[2], u[3], u[4], u[5], 0, 0])
}
