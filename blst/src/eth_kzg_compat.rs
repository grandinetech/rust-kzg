#[cfg(feature = "c_bindings")]
use crate::eip_7594::BlstBackend;

#[cfg(feature = "c_bindings")]
kzg::c_bindings_rust_eth_kzg!(BlstBackend, "../../kzg-bench/src/trusted_setup.txt");
