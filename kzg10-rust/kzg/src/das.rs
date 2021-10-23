use crate::data_types::fr::Fr;
use crate::fk20_fft::*;

impl FFTSettings {
    pub fn das_fft_extension(&self, values: &mut Vec<Fr>) {
        if (values.len() << 1) > self.max_width {
            panic!("ftt_settings max width too small!");
        }

        self._das_fft_extension(values, 1);
        
        // just dividing every value by 1/(2**depth) aka length
        // TODO: what's faster, maybe vec[x] * vec[x], ask herumi to implement?
        let inv_length = Fr::from_int(values.len() as i32).get_inv();
        for i in 0..values.len() {
            values[i] *= &inv_length;
        }
    }

    pub fn das_fft_extension_from_slice(&self, values: &[Fr]) -> Vec<Fr>{
        if (values.len() << 1) > self.max_width {
            panic!("ftt_settings max width too small!");
        }
        let mut values = values.to_vec();
        self._das_fft_extension(&mut values, 1);
        
        // just dividing every value by 1/(2**depth) aka length
        // TODO: what's faster, maybe vec[x] * vec[x], ask herumi to implement?
        let inv_length = Fr::from_int(values.len() as i32).get_inv();
        for i in 0..values.len() {
            values[i] *= &inv_length;
        }
        values
    }


    fn _das_fft_extension(&self, values: &mut [Fr], stride: usize) {
        if values.len() == 2 {
            let (x, y) = FFTSettings::_calc_add_and_sub(&values[0], &values[1]);

            let temp = &y * &self.exp_roots_of_unity[stride];
            values[0] = &x + &temp;
            values[1] = &x - &temp;
            return;
        }

        let length = values.len();
        let half = length >> 1;
        
        // let ab_half_0s = ab[..quarter];
        // let ab_half_1s = ab[quarter..];
        for i in 0..half {
            let (add, sub) = FFTSettings::_calc_add_and_sub(&values[i], &values[half + i]);
            values[half + i] = &sub * &self.exp_roots_of_unity_rev[(i << 1) * stride];
            values[i] = add;
        }

        // left
        self._das_fft_extension(&mut values[..half], stride << 1);
        // right
        self._das_fft_extension(&mut values[half..], stride << 1);

        for i in 0..half {
            let root = &self.exp_roots_of_unity[((i << 1) + 1) * stride];
            let y_times_root = &values[half + i] * root;

            let (add, sub) = FFTSettings::_calc_add_and_sub(&values[i], &y_times_root);
            values[i] = add;
            values[i + half] = sub;
        }
    }

    fn _calc_add_and_sub(a: &Fr, b: &Fr) -> (Fr, Fr) {
        return (a + b, a - b);
    }
}