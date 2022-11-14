pub mod consts;
pub mod eip_4844;
pub mod fftsettings;
pub mod finite;
pub mod fk20settings;
pub mod kzgsettings;
pub mod poly;
pub mod utils;
pub mod fftsettings4844;
pub mod kzgsettings4844;

#[cfg(feature = "parallel")]
const RUN_PARALLEL: bool = true;
#[cfg(not(feature = "parallel"))]
const RUN_PARALLEL: bool = false;
