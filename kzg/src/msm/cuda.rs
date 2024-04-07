use core::marker::PhantomData;

use icicle_bls12_381::curve::CurveCfg;
use icicle_core::{curve::Affine, msm::{precompute_bases, MSMConfig}, traits::FieldImpl};
use icicle_cuda_runtime::{memory::HostOrDeviceSlice, device_context::{DeviceContext, DEFAULT_DEVICE_ID}};
use core::fmt::Debug;
use crate::{Fr, G1Affine, G1Fp, G1GetFp, G1Mul, Scalar256, G1};

use super::msm_impls::batch_convert;

pub struct IcicleConfig<TFr, TG1, TG1Fp, TG1Affine>
where
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
{
    affines: HostOrDeviceSlice<'static, Affine<CurveCfg>>,

    g1_marker: PhantomData<TG1>,
    g1_fp_marker: PhantomData<TG1Fp>,
    fr_marker: PhantomData<TFr>,
    g1_affine_marker: PhantomData<TG1Affine>
}

impl<
TFr: Fr,
TG1Fp: G1Fp,
TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
TG1Affine: G1Affine<TG1, TG1Fp>,
> Debug for IcicleConfig<TFr, TG1, TG1Fp, TG1Affine> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // TODO: add formatting for affines
        f.debug_struct("IcicleConfig").finish()
    }
}

impl<
TFr: Fr,
TG1Fp: G1Fp,
TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
TG1Affine: G1Affine<TG1, TG1Fp>,
> Clone for IcicleConfig<TFr, TG1, TG1Fp, TG1Affine> {
    fn clone(&self) -> Self {
        // FIXME: affines should be cloned actually
        Self { affines: HostOrDeviceSlice::Host(vec![]), g1_marker: PhantomData, g1_fp_marker: PhantomData, fr_marker: PhantomData, g1_affine_marker: PhantomData }
    }
}

const PRECOMPUTE_FACTOR: usize = 8;

impl<
        TFr: Fr,
        TG1Fp: G1Fp,
        TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
        TG1Affine: G1Affine<TG1, TG1Fp>,
    > IcicleConfig<TFr, TG1, TG1Fp, TG1Affine>
{
    pub fn new(points: &[TG1]) -> Result<Option<Self>, String> {
        let affines_raw = batch_convert::<TG1, TG1Fp, TG1Affine>(points).iter().map(|it| icicle_bls12_381::curve::G1Affine::from_limbs(it.x().to_limbs(), it.y().to_limbs())).collect::<Vec<_>>();
        // let Ok(mut affines) = HostOrDeviceSlice::<'static, Affine<CurveCfg>>::cuda_malloc(affines_raw.len()) else {
        //     return Ok(None);
        // };
        // if affines.copy_from_host(&affines_raw).is_err() {
        //     return Ok(None);
        // }
        let device_affines = HostOrDeviceSlice::on_host(affines_raw);

        let Ok(mut affines) = HostOrDeviceSlice::<'static, Affine<CurveCfg>>::cuda_malloc(points.len() * PRECOMPUTE_FACTOR) else {
            return Ok(None);
        };

        if precompute_bases(&device_affines, PRECOMPUTE_FACTOR as i32, 0, &DeviceContext::default_for_device(DEFAULT_DEVICE_ID), &mut affines).is_err() {
            return Ok(None);
        }

        Ok(Some(Self {
            affines,

            fr_marker: PhantomData,
            g1_fp_marker: PhantomData,
            g1_marker: PhantomData,
            g1_affine_marker: PhantomData
        }))
    }

    pub fn multiply_sequential(&self, _scalars: &[Scalar256]) -> TG1 {
        panic!("No sequential implementation for CUDA MSM");
    }

    #[cfg(feature = "parallel")]
    pub fn multiply_parallel(&self, scalars: &[Scalar256]) -> TG1 {
        use icicle_bls12_381::curve::ScalarField;
        use icicle_core::curve::Projective;
        use icicle_cuda_runtime::stream::CudaStream;

        let mut results = HostOrDeviceSlice::cuda_malloc(1).unwrap();
        let mut scalars_d = HostOrDeviceSlice::cuda_malloc(scalars.len()).unwrap();
        let stream = CudaStream::create().unwrap();
        scalars_d.copy_from_host_async(&scalars.iter().map(|it| ScalarField::from_bytes_le(it.as_u8())).collect::<Vec<_>>(), &stream).unwrap();
        let mut config = MSMConfig::default_for_device(DEFAULT_DEVICE_ID);
        config.precompute_factor = PRECOMPUTE_FACTOR as i32;
        config.ctx.stream = &stream;
        config.is_async = true;
        
        icicle_core::msm::msm(&scalars_d, &self.affines, &config, &mut results).unwrap();

        let mut results_h = vec![Projective::<CurveCfg>::zero(); 1];
        results.copy_to_host_async(&mut results_h, &stream);

        stream.synchronize().unwrap();
        stream.destroy().unwrap();

        let mut output = TG1::default();

        *output.x_mut() = TG1Fp::from_bytes_le(&results_h.as_slice()[0].x.to_bytes_le().try_into().unwrap());
        *output.y_mut() = TG1Fp::from_bytes_le(&results_h.as_slice()[0].y.to_bytes_le().try_into().unwrap());
        *output.z_mut() = TG1Fp::from_bytes_le(&results_h.as_slice()[0].z.to_bytes_le().try_into().unwrap());

        output
    }
}
