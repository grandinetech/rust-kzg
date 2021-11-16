
// pub use super::{ZPoly, BlsScalar};
use kzg::{G1, G2, G1Mul, G2Mul, KZGSettings};
// use ff::{Field, PrimeField};

// use std::ptr;

use std::ops::{Neg, Add};
// use std::convert::TryInto;

// use blst::blst_p1_affine as P1Affine;
// use blst::blst_p1 as P1;
// use blst::blst_fr as BlstFr;

// use blst::blst_scalar;

// use crate::utils::*;

// use crate::utils::*;
pub use crate::curve::g1::G1Affine as ZkG1Affine; 
pub use crate::curve::g1::G1Projective as ZkG1Projective; 
pub use crate::curve::g2::G2Affine as ZkG2Affine; 
pub use crate::curve::g2::G2Projective as ZkG2Projective;
pub use crate::curve::fp::Fp as ZkFp;
pub use crate::curve::fp2::Fp2 as ZkFp2;
pub use crate::curve::fp12::Fp12 as ZkFp12;

// #[cfg(all(feature = "pairings", feature = "alloc"))]
pub use crate::curve::pairings::{G2Prepared, multi_miller_loop, MillerLoopResult };

// use super::G2Prepared;
// use super::multi_miller_loop;

use crate::zkfr::blsScalar;

use crate::poly::ZPoly;
use crate::fftsettings::{ZkFFTSettings};

use crate::kzg_proofs::{
	check_proof_single as check_single,
	commit_to_poly as poly_commit,
	compute_proof_single as open_single,
	eval_poly,
	new_kzg_settings,
	KZGSettings as LKZGSettings
};

 pub const G1_GENERATOR: ZkG1Projective = ZkG1Projective {
    x: ZkFp::from_raw_unchecked([0x5cb3_8790_fd53_0c16, 0x7817_fc67_9976_fff5, 0x154f_95c7_143b_a1c1, 0xf0ae_6acd_f3d0_e747, 0xedce_6ecc_21db_f440, 0x1201_7741_9e0b_fb75,] ),
    y: ZkFp::from_raw_unchecked([0xbaac_93d5_0ce7_2271, 0x8c22_631a_7918_fd8e, 0xdd59_5f13_5707_25ce, 0x51ac_5829_5040_5194, 0x0e1c_8c3f_ad00_59c0, 0x0bbc_3efc_5008_a26a,] ),
    z: ZkFp::from_raw_unchecked([0x7609_0000_0002_fffd, 0xebf4_000b_c40c_0002, 0x5f48_9857_53c7_58ba, 0x77ce_5853_7052_5745, 0x5c07_1a97_a256_ec6d, 0x15f6_5ec3_fa80_e493,] ),
	};
	
	pub const G1_NEGATIVE_GENERATOR: ZkG1Projective = ZkG1Projective {
    x: ZkFp::from_raw_unchecked([0x5cb3_8790_fd53_0c16, 0x7817_fc67_9976_fff5, 0x154f_95c7_143b_a1c1, 0xf0ae_6acd_f3d0_e747, 0xedce_6ecc_21db_f440, 0x1201_7741_9e0b_fb75,] ),
    y: ZkFp::from_raw_unchecked([0xff52_6c2a_f318_883a, 0x9289_9ce4_383b_0270, 0x89d7_738d_9fa9_d055, 0x12ca_f35b_a344_c12a, 0x3cff_1b76_964b_5317, 0x0e44_d2ed_e977_4430,] ),
    z: ZkFp::from_raw_unchecked([0x7609_0000_0002_fffd, 0xebf4_000b_c40c_0002, 0x5f48_9857_53c7_58ba, 0x77ce_5853_7052_5745, 0x5c07_1a97_a256_ec6d, 0x15f6_5ec3_fa80_e493,] ),
	};
	
	pub const G1_IDENTITY: ZkG1Projective = ZkG1Projective {
	x: ZkFp::zero(),
    y: ZkFp::one(),
    z: ZkFp::zero(),};

	
	pub const G2_GENERATOR: ZkG2Projective = ZkG2Projective {
    x: ZkFp2 {
           c0: ZkFp ( [0xf5f28fa202940a10, 0xb3f5fb2687b4961a, 0xa1a893b53e2ae580, 0x9894999d1a3caee9, 0x6f67b7631863366b, 0x058191924350bcd7,] ),
           c1: ZkFp ( [0xa5a9c0759e23f606, 0xaaa0c59dbccd60c3, 0x3bb17e18e2867806, 0x1b1ab6cc8541b367, 0xc2b6ed0ef2158547, 0x11922a097360edf3,] )
        
    },
    y: ZkFp2 {
       
           c0: ZkFp ( [0x4c730af860494c4a, 0x597cfa1f5e369c5a, 0xe7e6856caa0a635a, 0xbbefb5e96e0d495f, 0x07d3a975f0ef25a2, 0x0083fd8e7e80dae5,] ),
           c1: ZkFp ( [0xadc0fc92df64b05d, 0x18aa270a2b1461dc, 0x86adac6a3be4eba0, 0x79495c4ec93da33a, 0xe7175850a43ccaed, 0x0b2bc2a163de1bf2,] )
        
    },
    z: ZkFp2 {
        
           c0: ZkFp ( [0x760900000002fffd, 0xebf4000bc40c0002, 0x5f48985753c758ba, 0x77ce585370525745, 0x5c071a97a256ec6d, 0x15f65ec3fa80e493,] ),
           c1: ZkFp ( [0x0000000000000000, 0x0000000000000000, 0x0000000000000000, 0x0000000000000000, 0x0000000000000000, 0x0000000000000000,] )
        
    },
};

pub const G2_NEGATIVE_GENERATOR: ZkG2Projective = ZkG2Projective {
    x: ZkFp2 {
           c0: ZkFp ( [0xf5f28fa202940a10, 0xb3f5fb2687b4961a, 0xa1a893b53e2ae580, 0x9894999d1a3caee9, 0x6f67b7631863366b, 0x058191924350bcd7] ),
           c1: ZkFp ( [0xa5a9c0759e23f606, 0xaaa0c59dbccd60c3, 0x3bb17e18e2867806, 0x1b1ab6cc8541b367, 0xc2b6ed0ef2158547, 0x11922a097360edf3] )
        
    },
    y: ZkFp2 {
           c0: ZkFp ( [0x6d8bf5079fb65e61, 0xc52f05df531d63a5, 0x7f4a4d344ca692c9, 0xa887959b8577c95f, 0x4347fe40525c8734, 0x197d145bbaff0bb5] ),
           c1: ZkFp ( [0x0c3e036d209afa4e, 0x0601d8f4863f9e23, 0xe0832636bacc0a84, 0xeb2def362a476f84, 0x64044f659f0ee1e9, 0x0ed54f48d5a1caa7] )
        
    },
    z: ZkFp2 {
           c0: ZkFp ( [0x760900000002fffd, 0xebf4000bc40c0002, 0x5f48985753c758ba, 0x77ce585370525745, 0x5c071a97a256ec6d, 0x15f65ec3fa80e493] ),
           c1: ZkFp ( [0x0000000000000000, 0x0000000000000000, 0x0000000000000000, 0x0000000000000000, 0x0000000000000000, 0x0000000000000000] )
    },
};
	

impl G1 for ZkG1Projective {
	
	fn default() -> Self {
		<ZkG1Projective as Default>::default()
	}

	fn identity() -> Self {
		G1_IDENTITY
	}

	fn generator() -> Self {
		G1_GENERATOR
	}

	fn negative_generator() -> Self {
		G1_NEGATIVE_GENERATOR
	}

	fn rand() -> Self {
		todo!()
	}

	fn add_or_dbl(&mut self, b: &Self) -> Self {
		if self.eq(&b) {
			self.dbl()
		}
		else {
		
		let ret = ZkG1Projective::add(&self, &b);
		
		ret
		}
	}

	fn is_inf(&self) -> bool {
		bool::from(self.is_identity())
	}
	
	fn dbl(&self) -> Self {
		self.double()
	}
	
	fn sub(&self, b: &Self) -> Self {
		self + (-b)
	}

    fn equals(&self, b: &Self) -> bool {
		self.eq(b)
	}
}

impl G1Mul<blsScalar> for ZkG1Projective {
	fn mul(&self, b: &blsScalar) -> Self {
		self * b
	}
}

impl G2 for ZkG2Projective {
	
	fn default() -> Self {
		<ZkG2Projective as Default>::default()
	}

	fn generator() -> Self {
		G2_GENERATOR
	}

	fn negative_generator() -> Self {
		G2_NEGATIVE_GENERATOR
	}

	fn add_or_dbl(&mut self, b: &Self) -> Self {
		if self.eq(&b) {
			self.dbl()
		}
		else {self.add(b)}
	}

	fn dbl(&self) -> Self {
		self.double()
	}
	
	fn sub(&self, b: &Self) -> Self {
		self + (-b)
	}

    fn equals(&self, b: &Self) -> bool {
		self.eq(b)
	}
}

impl G2Mul<blsScalar> for ZkG2Projective {
	fn mul(&self, b: &blsScalar) -> Self {
		self * b
	}
}

pub fn pairings_verify(a1: &ZkG1Projective, a2: &ZkG2Projective, b1: &ZkG1Projective, b2: &ZkG2Projective) -> bool {
    // As an optimisation, we want to invert one of the pairings,

	let a1neg = ZkG1Projective::neg(*a1);
	
	let aa1 = ZkG1Affine::from(&a1neg);
	let bb1 = ZkG1Affine::from(b1);
	let aa2 = ZkG2Affine::from(a2);
	let bb2 = ZkG2Affine::from(b2);
	
	let aa2_prepared = G2Prepared::from(aa2);
	let bb2_prepared = G2Prepared::from(bb2);
	
	
	let loop0 = multi_miller_loop(&[(&aa1, &aa2_prepared)]);
	let loop1 = multi_miller_loop(&[(&bb1, &bb2_prepared)]);
	
	let gt_point = loop0.add(loop1);
	
		
	let new_point = MillerLoopResult::final_exponentiation(&gt_point);
	
	return ZkFp12::eq(&ZkFp12::one(), &new_point.0);
	
}

// pub fn g1_linear_combination(out: &mut ZkG1Projective, p: &Vec<ZkG1Projective>, coeffs: &Vec<blsScalar>, len: usize) {

    // // if (len < 8) { // Tunable parameter: must be at least 2 since Blst fails for 0 or 1
        // // // Direct approach
        // // g1_t tmp;
        // // *out = g1_identity;
        // // for (uint64_t i = 0; i < len; i++) {
            // // g1_mul(&tmp, &p[i], &coeffs[i]);
            // // blst_p1_add_or_double(out, out, &tmp);
        // // }       
	
	// if len < 8 {
		// let mut tmp = <ZkG1Projective as G1>::default();
		// *out = G1_IDENTITY;
		// for i in 0..len{
			// tmp = G1Mul::mul(&p[i], &coeffs[i]);
			// out.add_or_dbl(&tmp);
			// // g1_mul(&tmp, &p[i], &coeffs[i]);
            // // blst_p1_add_or_double(out, out, &tmp);
		// }	
	// }
	// else {
		// // Blst's implementation of the Pippenger method
        // //blst_p1_affine *p_affine = malloc(len * sizeof(blst_p1_affine));
        
		// let mut scratch: blst::limb_t = blst::limb_t::default();
		// unsafe {
			// scratch = blst::blst_p1s_mult_pippenger_scratch_sizeof(len).try_into().unwrap();
        // }
		
		// let mut p_affine = vec![P1Affine::default(); len];
        // //blst_scalar *scalars = malloc(len * sizeof(blst_scalar));
        // let mut scalars = vec![blst_scalar::default(); len];
        // // Transform the points to affine representation
        // //const blst_p1 *p_arg[2] = {p, NULL};
        // // let p_arg: const* = {p, null}
		
		// let p_blst = zk_g1projective_into_blst_p1(p[0]).unwrap();
		
        // let p_arg: [*const P1; 2] = [&p_blst, &P1::default()];
        // // let p_arg = p_blst;
		// // let p_arg: *const P1 = &p_blst;

        // unsafe {
            // blst::blst_p1s_to_affine(p_affine.as_mut_ptr(), p_arg.as_ptr(), len);
			// //meta errora
		// }
		// // let coeffs_blst: [BlstFr; u64] = [ ;u64];
        // // Transform the field elements to 256-bit scalars
        // for i in 0..len {
            // unsafe {
				// let coeffs_blst = zk_fr_into_blst_fr(&coeffs[i]);
                // blst::blst_scalar_from_fr(&mut scalars[i], &coeffs_blst);
            // }
        // }
        // // Call the Pippenger implementation
        // //const byte *scalars_arg[2] = {(byte *)scalars, NULL};
        // let scalars_arg: [*const blst_scalar; 2] = [scalars.as_ptr(), &blst_scalar::default()];
        // //scalars_arg[0] = &scalars;
        // //const blst_p1_affine *points_arg[2] = {p_affine, NULL};
        // let points_arg: [*const P1Affine; 2] = [p_affine.as_ptr(), &P1Affine::default()];
        // //points_arg[0] = &p_affine;
        // //void *scratch = malloc(blst_p1s_mult_pippenger_scratch_sizeof(len));
		
		// //blst_p1s_mult_pippenger(out, points_arg, len, scalars_arg, 256, scratch);
		
		// let mut out_blst = zk_g1projective_into_blst_p1(*out).unwrap();
		
        // unsafe {
            // // blst::blst_p1s_mult_pippenger(&mut out_blst, points_arg.as_ptr(), len, scalars_arg.as_ptr() as *const *const u8, 256, &mut scratch);
		// // meta errora. greiciausiai del tu [*const p1affine; 2]...
		// }
	
	// *out = blst_p1_into_zk_g1projective(&out_blst).unwrap();
	
	// }
	
// }

impl KZGSettings<blsScalar, ZkG1Projective, ZkG2Projective, ZkFFTSettings, ZPoly> for LKZGSettings {
	fn default() -> Self {
    	todo!()
    }

	fn new(secret_g1: &Vec<ZkG1Projective>, secret_g2: &Vec<ZkG2Projective>, length: usize, fs: &ZkFFTSettings) -> Result<LKZGSettings, String> {
        Ok(new_kzg_settings(secret_g1.to_vec(), secret_g2.to_vec(), length as u64, fs))
    }

	fn commit_to_poly(&self, p: &ZPoly) -> Result<ZkG1Projective, String> {
        Ok(poly_commit(&p, self).unwrap())
    }

    fn compute_proof_single(&self, p: &ZPoly, x: &blsScalar) -> Result<ZkG1Projective, String> {
    	Ok(open_single(p, x, self))
    }

    fn check_proof_single(&self, com: &ZkG1Projective, proof: &ZkG1Projective, x: &blsScalar, value: &blsScalar) -> Result<bool, String> {
    	Ok(check_single(com, proof, x, value, self))
    }

    fn compute_proof_multi(&self, _p: &ZPoly, _x: &blsScalar, _n: usize) -> Result<ZkG1Projective, String> {
    	todo!()
    }

    fn check_proof_multi(&self, _com: &ZkG1Projective, _proof: &ZkG1Projective, _x: &blsScalar, _values: &Vec<blsScalar>, _n: usize) -> Result<bool, String> {
    	todo!()
    }

    fn get_expanded_roots_of_unity_at(&self, _i: usize) -> blsScalar {
    	todo!()
    }
}

impl Clone for LKZGSettings {
    fn clone(&self) -> Self {
        LKZGSettings::new(&self.secret_g1.clone(), &self.secret_g2.clone(), self.length as usize, &self.fs.clone()).unwrap()
    }
}
