//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as Template;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

const TEST_WASM_CODE: [u8; 41] = [
	0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x60,
	0x00, 0x01, 0x7e, 0x03, 0x02, 0x01, 0x00, 0x07, 0x08, 0x01, 0x04, 0x6d,
	0x61, 0x69, 0x6e, 0x00, 0x00, 0x0a, 0x09, 0x01, 0x07, 0x00, 0x42, 0x2a,
	0x10, 0x00, 0x42, 0x01, 0x0b
];

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn save_cloud_function() {
		let caller: T::AccountId = whitelisted_caller();

		#[extrinsic_call]
		save_cloud_function(RawOrigin::Signed(caller.clone()), TEST_WASM_CODE.to_vec());

		let account = Pallet::<T>::generate_keyless_account(&TEST_WASM_CODE.to_vec());
		assert!(pallet::CloudFunctions::<T>::contains_key(account));
	}

	#[benchmark]
	fn run_algo_for() {
		let caller: T::AccountId = whitelisted_caller();

		let code = TEST_WASM_CODE.to_vec();
		let _ = Pallet::<T>::save_cloud_function(RawOrigin::Signed(caller.clone()).into(), code.clone());
		let account = Pallet::<T>::generate_keyless_account(&code);

		#[extrinsic_call]
		run_algo_for(RawOrigin::Signed(caller), account);

	}

	#[benchmark]
	fn run_code(s: Linear<1, 1024>) {
		let caller: T::AccountId = whitelisted_caller();

		let mut code = TEST_WASM_CODE.to_vec();

		if s as usize > code.len() {
			code.resize(s as usize, 0);
		}

		let account = Pallet::<T>::generate_keyless_account(&code);

		pallet::CloudFunctions::<T>::insert(account.clone(), pallet::Algorithm { code: code.clone() });

		#[block]
		{
			let _ = Pallet::<T>::run_code(
				RawOrigin::Signed(caller).into(),
				account,
				code
			);
		}
	}

	impl_benchmark_test_suite!(
        Template,
        crate::mock::new_test_ext(),
        crate::mock::Test
    );
}