// gonna have to change ZkFFTSettings to something different, because of lib.rs trait 'ZkFFTSettings'

// use blst::blst_fr as BlstFr;
use crate::consts::*;
use crate::zkfr::blsScalar;
use crate::fft_fr::*;
use crate::utils::is_power_of_two;
use crate::poly::*;

// use blst::blst_fr_from_uint64;
use kzg::{Fr, FFTFr, FFTSettings, FFTSettingsPoly};

#[derive(Clone)]
pub struct ZkFFTSettings {
    pub max_width: usize,
    pub root_of_unity: blsScalar,
    pub expanded_roots_of_unity: Vec<blsScalar>,
    pub reverse_roots_of_unity: Vec<blsScalar>,
}

impl ZkFFTSettings {
    pub fn das_fft_extension_stride(&self, ab: &mut [blsScalar], stride: usize) {
        if ab.len() < 2 {
            return;
        } else if ab.len() == 2 {
            let x = ab[0].add(&ab[1]);
            let y = ab[0].sub(&ab[1]);
            let tmp = y.mul(&self.expanded_roots_of_unity[stride]);

            ab[0] = x.add(&tmp);
            ab[1] = x.sub(&tmp);

            return;
        }else {
            let half = ab.len();
            let halfhalf = half / 2;

            for i in 0..halfhalf{
                let tmp1 = ab[i].add(&ab[halfhalf+i]);
                let tmp2 = ab[i].sub(&ab[halfhalf+i]);
                ab[halfhalf+i] = tmp2.mul(&self.reverse_roots_of_unity[i * 2 * stride]);
                ab[i] = tmp1;
            }

            self.das_fft_extension_stride(&mut ab[..halfhalf], stride * 2);
            self.das_fft_extension_stride(&mut ab[halfhalf..], stride * 2);

            for i in 0..halfhalf{
                let x = ab[i];
                let y = ab[halfhalf+i];
                let y_times_root = y.mul(&self.expanded_roots_of_unity[(1 + 2 * i) * stride]);
                ab[i] = x.add(&y_times_root);
                ab[halfhalf+i] = x.sub(&y_times_root);
            }
        }
    }
}


impl FFTSettingsPoly<blsScalar, ZPoly, ZkFFTSettings> for ZkFFTSettings {
    fn poly_mul_fft(a: &ZPoly, b: &ZPoly, len: usize, _fs: Option<&ZkFFTSettings>) -> Result<ZPoly, String> {
			
		// for i in 0..3 {	
			// println!("a(fftsettings_mul_fft) = {:?}", a.get_coeff_at(i));
		// }
		// for i in 0..3 {	
			// println!("b(fftsettings_mul_fft) = {:?}", b.get_coeff_at(i));
		// }
		
		poly_mul_fft(len, a, b)
	}
	
}

impl FFTFr<blsScalar> for ZkFFTSettings {
	
	fn fft_fr(&self, data: &[blsScalar], inverse: bool) -> Result<Vec<blsScalar>, String> {
		if data.len() > self.max_width {
			return Err(String::from( "The supplied list is longer than the available max width",
			));
		}
		else if !is_power_of_two(data.len()) {
			return Err(String::from("A list with power-of-two length is expected"));
		}
	
	// In case more roots are provided with fft_settings, use a larger stride
        let stride = self.max_width / data.len();
        let mut ret = vec![<blsScalar as Fr>::default(); data.len()];

        // Inverse is same as regular, but all constants are reversed and results are divided by n
        // This is a property of the DFT matrix
        let roots = if inverse {
            &self.reverse_roots_of_unity
        } else {
            &self.expanded_roots_of_unity
        };
        fft_fr_fast(&mut ret, data, 1, roots, stride);

        if inverse {
            let mut inv_len: blsScalar = blsScalar::from_u64(data.len() as u64);
            inv_len = inv_len.inverse();
            for i in 0..data.len() {
                ret[i] = ret[i].mul(&inv_len);
            }
        }

        return Ok(ret);
	
	
	}
	
	
}

impl ZkFFTSettings {
	pub fn from_scale(max_scale: usize) -> Result<ZkFFTSettings, String> {
        if max_scale >= SCALE2_ROOT_OF_UNITY.len() {
            return Err(String::from("Scale is expected to be within root of unity matrix row size"));
        }
        let max_width: usize = 1 << max_scale;

        Ok(ZkFFTSettings {
            max_width: max_width,
            ..FFTSettings::default()
        })
    }

}


impl FFTSettings<blsScalar> for ZkFFTSettings {

	fn default() -> ZkFFTSettings {
        ZkFFTSettings {
            max_width: 0,
            root_of_unity: blsScalar::zero(),
            expanded_roots_of_unity: Vec::new(),
            reverse_roots_of_unity: Vec::new(),
        }
    }
	
	fn new(scale: usize) -> Result<ZkFFTSettings, String> {
        if scale >= SCALE2_ROOT_OF_UNITY.len() {
            return Err(String::from("Scale is expected to be within root of unity matrix row size"));
        }

        // max_width = 2 ^ max_scale
        let max_width: usize = 1 << scale;
		//let mut ret = blsScalar::default();
		//blsScalar::from_raw(SCALE2_ROOT_OF_UNITY[scale]);
        let root_of_unity = blsScalar::from_u64_arr(&SCALE2_ROOT_OF_UNITY[scale]);

        // create max_width of roots & store them reversed as well
        let expanded_roots_of_unity = expand_root_of_unity(&root_of_unity, max_width).unwrap();
        let mut reverse_roots_of_unity = expanded_roots_of_unity.clone();
        reverse_roots_of_unity.reverse();

        Ok(ZkFFTSettings {
            max_width,
            root_of_unity,
            expanded_roots_of_unity,
            reverse_roots_of_unity,
        })
    }
	
	fn get_max_width(&self) -> usize {
        self.max_width
    }

    fn get_expanded_roots_of_unity_at(&self, i: usize) -> blsScalar {
        self.expanded_roots_of_unity[i]
    }

    fn get_expanded_roots_of_unity(&self) -> &[blsScalar] {
        &self.expanded_roots_of_unity
    }

    fn get_reverse_roots_of_unity_at(&self, i: usize) -> blsScalar {
        self.reverse_roots_of_unity[i]
    }

    fn get_reversed_roots_of_unity(&self) -> &[blsScalar] {
        &self.reverse_roots_of_unity
    }
}


pub fn new_fft_settings(_max_scale: usize) -> ZkFFTSettings {
    ZkFFTSettings::default()
}
