#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResult, pallet_prelude::*, dispatch::DispatchResultWithPostInfo};
	use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

    #[pallet::config]
	pub trait Config: frame_system::Config {
        //为何需要Self as frame_system::Config? 类型本来就继承Config
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type MaxLen:Get<usize>;
	}

    #[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

    #[pallet::event]
    //#[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T:Config>{
        ClaimCreated(T::AccountId,Vec<u8>),
        ClaimRevoked(T::AccountId,Vec<u8>),
        ClaimTransfered(T::AccountId,Vec<u8>),
    }

    #[pallet::error]
    pub enum Error<T> {
        ProofExist,
        ProofNotExist,
        InconsistentOwner,
        ProofLengthTooLong,  
    }

    #[pallet::storage]
	#[pallet::getter(fn siyou_proofs)]
	pub type Proofs<T: Config> = StorageMap<
		_,                              //
		Blake2_128Concat,               //
		Vec<u8>,                        //
		(T::AccountId, T::BlockNumber), //
	>;

    // #[pallet::hooks]
    // // where T=Config
    // impl<T:Conifg> Hooks<BlockNumberFor<T>> for Pallet<T> {        
    // }

    #[pallet::call]
    impl<T:Config> Pallet<T>{
        #[pallet::weight(1000)]
        pub fn create_claim(
            origin:OriginFor<T>,
            claim:Vec<u8>
        )->DispatchResultWithPostInfo{
            let sender=ensure_signed(origin)?;
            ensure!(!Proofs::<T>::contains_key(&claim),Error::<T>::ProofExist);
            Proofs::<T>::insert(&claim,(sender.clone(),frame_system::Pallet::<T>::block_number()));
            //是否可以不加Self,直接deposit_event
            Self::deposit_event(Event::ClaimCreated(sender,claim));
            Ok(().into())
        }

        #[pallet::weight(1000)]
        pub fn revoke_claim(
            origin:OriginFor<T>,
            claim:Vec<u8>
        )->DispatchResultWithPostInfo{
            let sender=ensure_signed(origin)?;
            let (owner,_)=Proofs::<T>::get(&claim).ok_or(Error::<T>::ProofNotExist)?;
            ensure!(owner==sender,Error::<T>::InconsistentOwner);
            Proofs::<T>::remove(&claim);
            //为何Event不需要Event<T>
            Self::deposit_event(Event::ClaimRevoked(sender,claim));
            Ok(().into())
        }

        #[pallet::weight(1000)]
        pub fn transfer_claim(
            origin:OriginFor<T>,
            target:T::AccountId,
            claim:Vec<u8>
        )->DispatchResultWithPostInfo{
            let sender=ensure_signed(origin)?;
            let (owner,_)=Proofs::<T>::get(&claim).ok_or(Error::<T>::ProofNotExist)?;
            ensure!(owner==sender,Error::<T>::InconsistentOwner);
            Proofs::<T>::insert(&claim,(target.clone(),frame_system::Pallet::<T>::block_number()));
            Self::deposit_event(Event::ClaimTransfered(target,claim));
            Ok(().into())
        }

        #[pallet::weight(1000)]
        pub fn create_claim_with_check(
            origin:OriginFor<T>,
            claim:Vec<u8>
        )->DispatchResultWithPostInfo{
            ensure!(claim.len()<=T::MaxLen::get(),Error::<T>::ProofLengthTooLong);
            return Self::create_claim(origin, claim);
        }
    }
}