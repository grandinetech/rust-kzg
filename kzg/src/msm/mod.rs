pub mod arkmsm;
pub mod cell;
pub mod msm_impls;
pub mod precompute;
#[cfg(feature = "parallel")]
pub mod thread_pool;
#[cfg(feature = "parallel")]
pub mod tiling_parallel_pippenger;
pub mod tiling_pippenger_ops;
pub mod types;

#[cfg(feature = "parallel")]
mod parallel_pippenger_utils;
mod pippenger_utils;

#[cfg(all(feature = "bgmw", any(not(feature = "arkmsm"), feature = "parallel")))]
mod bgmw;

#[cfg(feature = "cuda")]
mod cuda;

#[cfg(all(feature = "cuda", feature = "bgmw"))]
compile_error!{"features `cuda` and `bgmw` are mutally exclusive"}
#[cfg(all(feature = "cuda", not(feature = "parallel")))]
compile_error!{"feature `cuda` requires feature `parallel`"}
