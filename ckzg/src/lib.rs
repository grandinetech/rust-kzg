pub mod utils;
pub mod consts;
pub mod finite;
pub mod poly;
pub mod fftsettings;
pub mod kzgsettings;
pub mod fk20settings;

#[cfg(feature = "parallel")]
const RUN_PARALLEL: bool = true;
#[cfg(not(feature = "parallel"))]
const RUN_PARALLEL: bool = false;
