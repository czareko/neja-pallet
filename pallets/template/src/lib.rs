#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
    use log;
    use frame_support::{dispatch, dispatch::*, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use scale_info::prelude;
    use sp_runtime::{FixedI64, FixedPointNumber, Rounding};
    use wasmi::{self, core::F64};
    use sp_runtime::Vec;
    use sp_runtime::traits::Hash;
    use wasmi::{Func, Caller};

    use super::*;

    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct Algorithm<T: Config> {
        pub schema_hashes: Vec<T::Hash>,
        pub code: Vec<u8>,
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type Hashing: Hash<Output = Self::Hash>;
        type WeightInfo: WeightInfo;
    }

    #[pallet::storage]
    pub type Algorithms<T: Config> =
    StorageMap<_, Blake2_128Concat, u64 /*algoId*/, Algorithm<T>, OptionQuery>;

    #[pallet::type_value]
    pub fn DefaultNextAlgoId<T: Config>() -> u64 { 100u64 }

    #[pallet::storage]
    pub type NextAlgoId<T: Config> = StorageValue<_, u64, ValueQuery, DefaultNextAlgoId<T>>;

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        AlgorithmAdded {
            algorithm_id: u64,
        },
        AlgoResult {
            result: i64,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        AlgoNotFound,
        AttestationNotFound,
        AlgoError1,
        AlgoError2,
        AlgoError3,
        AlgoError4,
        AlgoError5,
        AlgoError6,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(1)]
        #[pallet::weight(100_000)]
        pub fn save_algo(origin: OriginFor<T>, schema_hashes: Vec<T::Hash>, code: Vec<u8>) -> DispatchResult {
            let who = ensure_signed(origin)?;


            let id = NextAlgoId::<T>::get();
            NextAlgoId::<T>::set(id + 1);

            Algorithms::<T>::insert(id, Algorithm {
                schema_hashes,
                code,
            });

            Self::deposit_event(Event::AlgorithmAdded {
                algorithm_id: id,
            });

            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(100_000)]
        pub fn run_algo_for(origin: OriginFor<T>, issuer_hash: T::Hash, account_id: Vec<u8>, algorithm_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;


            let algorithm = Algorithms::<T>::get(algorithm_id).ok_or(Error::<T>::AlgoNotFound)?;



            return Pallet::<T>::run_code(algorithm.code);
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn run_code(code: Vec<u8>) -> DispatchResult {
            let engine = wasmi::Engine::default();

            let module =
                wasmi::Module::new(&engine, code.as_slice()).map_err(|_| Error::<T>::AlgoError1)?;

            type HostState = u32;
            let mut store = wasmi::Store::new(&engine, 42);
            let host_print = wasmi::Func::wrap(
                &mut store,
                |caller: wasmi::Caller<'_, HostState>, param: i32| {
                    log::debug!(target: "algo", "Message:{:?}", param);
                },
            );
            let abort_func = wasmi::Func::wrap(
              &mut store,
              |_: Caller<'_, HostState>, msg_id: i32, filename: i32, line: i32, col: i32| {
                  log::error!(
                      target: "algo",
                      "Abort called: msg_id={}, file={}, line={}, col={}",
                      msg_id, filename, line, col
                  );
                  // Err(wasmi::Trap::new(wasmi::TrapKind::Unreachable))
              },
            );

            let mut linker = <wasmi::Linker<HostState>>::new(&engine);
            linker.define("host", "print", host_print).map_err(|_| Error::<T>::AlgoError2)?;
      
            // Define the abort function in the linker
            linker.define("env", "abort", abort_func).map_err(|_| Error::<T>::AlgoError2)?;

            let instance = linker
                .instantiate(&mut store, &module)
                .map_err(|e| {
                    log::error!(target: "algo", "Algo3 {:?}", e);
                    Error::<T>::AlgoError3
                })?
                .start(&mut store)
                .map_err(|_| Error::<T>::AlgoError4)?;

            let calc = instance
                .get_typed_func::<(), i64>(&store, "calc")
                .map_err(|_| Error::<T>::AlgoError5)?;

            // And finally we can call the wasm!
            let result = calc.call(&mut store, ()).map_err(|e| {
                log::error!(target: "algo", "Algo6 {:?}", e);
                Error::<T>::AlgoError6
            })?;
            Self::deposit_event(Event::AlgoResult {
                result,
            });

            Ok(())
        }
    }
}
