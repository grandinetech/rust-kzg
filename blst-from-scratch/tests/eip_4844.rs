#[cfg(test)]

mod tests {
    use blst_from_scratch::types::{fr::FsFr, g1::FsG1};
    use kzg_bench::tests::eip_4844::{
        g1_lincomb
    };


    #[test]
    pub fn test_g1_lincomb() {
        g1_lincomb_vienas_testas::<FsFr, FsG1>(
            &g1_lincomb
        );
    }

}
