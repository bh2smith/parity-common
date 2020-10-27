// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Primitive types shared by Substrate and Parity Ethereum.
//!
//! Those are uint types `U128`, `U256` and `U512`, and fixed hash types `H160`,
//! `H256` and `H512`, with optional serde serialization, parity-scale-codec and
//! rlp encoding.

#![cfg_attr(not(feature = "std"), no_std)]

use core::convert::TryFrom;
use fixed_hash::{construct_fixed_hash, impl_fixed_hash_conversions};
#[cfg(feature = "scale-info")]
use scale_info::TypeInfo;
use uint::{construct_uint, uint_full_mul_reg};

/// Error type for conversion.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
	/// Overflow encountered.
	Overflow,
}

construct_uint! {
	/// 128-bit unsigned integer.
	#[cfg_attr(feature = "scale-info", derive(TypeInfo))]
	pub struct U128(2);
}
construct_uint! {
	/// 256-bit unsigned integer.
	#[cfg_attr(feature = "scale-info", derive(TypeInfo))]
	pub struct U256(4);
}
construct_uint! {
	/// 512-bits unsigned integer.
	#[cfg_attr(feature = "scale-info", derive(TypeInfo))]
	pub struct U512(8);
}

construct_fixed_hash! {
	/// Fixed-size uninterpreted hash type with 20 bytes (160 bits) size.
	#[cfg_attr(feature = "scale-info", derive(TypeInfo))]
	pub struct H160(20);
}
construct_fixed_hash! {
	/// Fixed-size uninterpreted hash type with 32 bytes (256 bits) size.
	#[cfg_attr(feature = "scale-info", derive(TypeInfo))]
	pub struct H256(32);
}
construct_fixed_hash! {
	/// Fixed-size uninterpreted hash type with 64 bytes (512 bits) size.
	#[cfg_attr(feature = "scale-info", derive(TypeInfo))]
	pub struct H512(64);
}

#[cfg(feature = "impl-serde")]
mod serde {
	use super::*;
	use impl_serde::{impl_fixed_hash_serde, impl_uint_serde};

	impl_uint_serde!(U128, 2);
	impl_uint_serde!(U256, 4);
	impl_uint_serde!(U512, 8);

	impl_fixed_hash_serde!(H160, 20);
	impl_fixed_hash_serde!(H256, 32);
	impl_fixed_hash_serde!(H512, 64);
}

#[cfg(feature = "impl-codec")]
mod codec {
	use super::*;
	use impl_codec::{impl_fixed_hash_codec, impl_uint_codec};

	impl_uint_codec!(U128, 2);
	impl_uint_codec!(U256, 4);
	impl_uint_codec!(U512, 8);

	impl_fixed_hash_codec!(H160, 20);
	impl_fixed_hash_codec!(H256, 32);
	impl_fixed_hash_codec!(H512, 64);
}

#[cfg(feature = "impl-rlp")]
mod rlp {
	use super::*;
	use impl_rlp::{impl_fixed_hash_rlp, impl_uint_rlp};

	impl_uint_rlp!(U128, 2);
	impl_uint_rlp!(U256, 4);
	impl_uint_rlp!(U512, 8);

	impl_fixed_hash_rlp!(H160, 20);
	impl_fixed_hash_rlp!(H256, 32);
	impl_fixed_hash_rlp!(H512, 64);
}

impl_fixed_hash_conversions!(H256, H160);

impl U256 {
	/// Multiplies two 256-bit integers to produce full 512-bit integer
	/// No overflow possible
	#[inline(always)]
	pub fn full_mul(self, other: U256) -> U512 {
		U512(uint_full_mul_reg!(U256, 4, self, other))
	}

	/// Lossy saturating conversion from a `f64` to a `U256`.
	///
	/// The conversion follows roughly the same rules as converting `f64` to other
	/// primitive integer types. Namely, the conversion of `value: f64` behaves as
	/// follows:
	/// - `NaN` => `0`
	/// - `(-∞, 0]` => `0`
	/// - `(0, u256::MAX]` => `value as u256`
	/// - `(u256::MAX, +∞)` => `u256::MAX`
	pub fn from_f64_lossy(value: f64) -> U256 {
		if value >= 1.0 {
			let bits = value.to_bits();
			// NOTE: Don't consider the sign or check that the subtraction will
			//   underflow since we already checked that the value is greater
			//   than 1.0.
			let exponent = ((bits >> 52) & 0x7ff) - 1023;
			let mantissa = (bits & 0x0f_ffff_ffff_ffff) | 0x10_0000_0000_0000;
			if exponent <= 52 {
				U256::from(mantissa >> (52 - exponent))
			} else if exponent >= 256 {
				U256::MAX
			} else {
				U256::from(mantissa) << U256::from(exponent - 52)
			}
		} else {
			0.into()
		}
	}

	#[cfg(feature = "std")]
	pub fn to_f64_lossy(self) -> f64 {
		let (res, factor) = match self {
			U256([_, _, 0, 0]) => (self, 1.0),
			U256([_, _, _, 0]) => (self >> 64, 2.0f64.powi(64)),
			U256([_, _, _, _]) => (self >> 128, 2.0f64.powi(128)),
		};
		(res.low_u128() as f64) * factor
	}
}

impl From<U256> for U512 {
	fn from(value: U256) -> U512 {
		let U256(ref arr) = value;
		let mut ret = [0; 8];
		ret[0] = arr[0];
		ret[1] = arr[1];
		ret[2] = arr[2];
		ret[3] = arr[3];
		U512(ret)
	}
}

impl TryFrom<U256> for U128 {
	type Error = Error;

	fn try_from(value: U256) -> Result<U128, Error> {
		let U256(ref arr) = value;
		if arr[2] | arr[3] != 0 {
			return Err(Error::Overflow);
		}
		let mut ret = [0; 2];
		ret[0] = arr[0];
		ret[1] = arr[1];
		Ok(U128(ret))
	}
}

impl TryFrom<U512> for U256 {
	type Error = Error;

	fn try_from(value: U512) -> Result<U256, Error> {
		let U512(ref arr) = value;
		if arr[4] | arr[5] | arr[6] | arr[7] != 0 {
			return Err(Error::Overflow);
		}
		let mut ret = [0; 4];
		ret[0] = arr[0];
		ret[1] = arr[1];
		ret[2] = arr[2];
		ret[3] = arr[3];
		Ok(U256(ret))
	}
}

impl TryFrom<U512> for U128 {
	type Error = Error;

	fn try_from(value: U512) -> Result<U128, Error> {
		let U512(ref arr) = value;
		if arr[2] | arr[3] | arr[4] | arr[5] | arr[6] | arr[7] != 0 {
			return Err(Error::Overflow);
		}
		let mut ret = [0; 2];
		ret[0] = arr[0];
		ret[1] = arr[1];
		Ok(U128(ret))
	}
}

impl From<U128> for U512 {
	fn from(value: U128) -> U512 {
		let U128(ref arr) = value;
		let mut ret = [0; 8];
		ret[0] = arr[0];
		ret[1] = arr[1];
		U512(ret)
	}
}

impl From<U128> for U256 {
	fn from(value: U128) -> U256 {
		let U128(ref arr) = value;
		let mut ret = [0; 4];
		ret[0] = arr[0];
		ret[1] = arr[1];
		U256(ret)
	}
}

impl<'a> From<&'a U256> for U512 {
	fn from(value: &'a U256) -> U512 {
		let U256(ref arr) = *value;
		let mut ret = [0; 8];
		ret[0] = arr[0];
		ret[1] = arr[1];
		ret[2] = arr[2];
		ret[3] = arr[3];
		U512(ret)
	}
}

impl<'a> TryFrom<&'a U512> for U256 {
	type Error = Error;

	fn try_from(value: &'a U512) -> Result<U256, Error> {
		let U512(ref arr) = *value;
		if arr[4] | arr[5] | arr[6] | arr[7] != 0 {
			return Err(Error::Overflow);
		}
		let mut ret = [0; 4];
		ret[0] = arr[0];
		ret[1] = arr[1];
		ret[2] = arr[2];
		ret[3] = arr[3];
		Ok(U256(ret))
	}
}
