#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		dispatch::DispatchResultWithPostInfo,
		pallet_prelude::*,
		traits::{Currency, Randomness, ReservableCurrency},
	};
	use frame_system::pallet_prelude::*;
	use sp_io::hashing::blake2_128;
	use sp_runtime::traits::{AtLeast32BitUnsigned, Bounded};

	#[derive(Encode, Decode, scale_info::TypeInfo)]
	//TypeInfo是满足什么需要？pallet_storage?
	pub struct Kitty(pub [u8; 16]); //元组结构体

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		//为何需要Self as frame_system::Config? 类型本来就继承Config
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type KittyIndex: Parameter + AtLeast32BitUnsigned + Default + Copy + Bounded;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		type ReserveOfKittyCreate: Get<BalanceOf<Self>>;
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	//#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreated(T::AccountId, T::KittyIndex),
		KittyTransfered(T::AccountId, T::AccountId, T::KittyIndex),
		KittySale(T::AccountId, T::KittyIndex, Option<BalanceOf<T>>),
		ErrorSalePrice(T::AccountId, T::KittyIndex),
	}

	#[pallet::error]
	pub enum Error<T> {
		KittiesCountOverflow,
		InvalidKittyIndex,
		NotCorrectOwner,
		BreedFromSameParent,
		NotForSale,
		NotEnoughBalance,
		KittyAlreadyOwned,
		SameOwner,
	}

	#[pallet::storage]
	#[pallet::getter(fn kitties_count)]
	pub type KittiesCount<T: Config> = StorageValue<_, T::KittyIndex>;

	#[pallet::storage]
	#[pallet::getter(fn get_kitties)]
	//todo 是否可以不加<T>
	//T是指代哪個的泛型？
	//最后一个参数ValueQuery如何使用，何时省略？
	pub type Kitties<T: Config> = StorageMap<
		_,                //
		Blake2_128Concat, //
		T::KittyIndex,    //
		Kitty,    //
	>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)]
	pub type KittyOwner<T: Config> = StorageMap<
		_,                    //
		Blake2_128Concat,     //
		T::KittyIndex,        //
		T::AccountId, //poe中有blockNumber,似乎没用到，是否可以也可以删除
	>;

	#[pallet::storage]
	#[pallet::getter(fn kitties_price)]
	pub type KittiesPrice<T: Config> =StorageMap<
		_, //
		Blake2_128Concat, //
		T::KittyIndex, //
		BalanceOf<T>//
	>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1000)]
		pub fn create(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			T::Currency::reserve(&sender, T::ReserveOfKittyCreate::get())
			.map_err(|_| Error::<T>::NotEnoughBalance)?;

			let kitty_id = match Self::kitties_count() {
				Some(id) => {
					ensure!(id != T::KittyIndex::max_value(), Error::<T>::KittiesCountOverflow);
					id
				}
				None => 1u32.into(),
			};

			let dna = Self::random_value(&sender);

			Kitties::<T>::insert(kitty_id, Kitty(dna));
			KittyOwner::<T>::insert(kitty_id, sender.clone());
			//1.into为什么不行,u32哪里实现了into trait
			KittiesCount::<T>::put(kitty_id + 1u32.into());

			Self::deposit_event(Event::KittyCreated(sender, kitty_id));

			Ok(().into())
		}

		#[pallet::weight(1000)]
		pub fn transfer(
			origin: OriginFor<T>,
			target: T::AccountId,
			kitty_id: T::KittyIndex,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			ensure!(sender != target, Error::<T>::SameOwner);

			let owner = KittyOwner::<T>::get(&kitty_id).unwrap();
			ensure!(owner == sender, Error::<T>::NotCorrectOwner);

			Self::transfer_kitty(sender, target, kitty_id);
			Ok(().into())
		}

		#[pallet::weight(1000)]
		pub fn breed(
			origin: OriginFor<T>,
			first_kitty_id: T::KittyIndex,
			second_kitty_id: T::KittyIndex,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(first_kitty_id != second_kitty_id, Error::<T>::BreedFromSameParent);

			let owner1 = Self::kitty_owner(first_kitty_id).ok_or(Error::<T>::InvalidKittyIndex)?;
			let owner2 = Self::kitty_owner(second_kitty_id).ok_or(Error::<T>::InvalidKittyIndex)?;

			ensure!(owner1 == who, Error::<T>::NotCorrectOwner);
			ensure!(owner2 == who, Error::<T>::NotCorrectOwner);

			let first_kitty_content = Self::get_kitties(first_kitty_id).ok_or(Error::<T>::InvalidKittyIndex)?;
			let second_kitty_contnet = Self::get_kitties(second_kitty_id).ok_or(Error::<T>::InvalidKittyIndex)?;

			let kitty_id = match Self::kitties_count() {
				Some(id) => {
					ensure!(id != T::KittyIndex::max_value(), Error::<T>::KittiesCountOverflow);
					id
				}
				None => 1u32.into(),
			};

			let dna_1 = first_kitty_content.0;
			let dna_2 = second_kitty_contnet.0;

			let selector = Self::random_value(&who);
			let mut new_dna = [0u8; 16];

			for i in 0..dna_1.len() {
				new_dna[i] = (selector[i] & dna_1[i]) | (!selector[i] & dna_2[i])
			}

			Kitties::<T>::insert(kitty_id, Kitty(new_dna));
			KittyOwner::<T>::insert(kitty_id, who.clone());
			KittiesCount::<T>::put(kitty_id + 1u32.into());

			Self::deposit_event(Event::KittyCreated(who, kitty_id));
			Ok(().into())
		}

		#[pallet::weight(1000)]
		pub fn sell(
			origin: OriginFor<T>,
			kitty_id: T::KittyIndex,
			price: Option<BalanceOf<T>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Some(who.clone()) == Self::kitty_owner(kitty_id), Error::<T>::NotCorrectOwner);

			KittiesPrice::<T>::mutate_exists(kitty_id, |p| *p = price);

			match price {
				Some(_) => {
					Self::deposit_event(Event::KittySale(who, kitty_id, price));
				}
				None => {
					Self::deposit_event(Event::ErrorSalePrice(who, kitty_id));
				}
			}

			// 与Ok(())区别
			Ok(().into())
		}

		#[pallet::weight(1000)]
		pub fn buy(origin: OriginFor<T>, kitty_id: T::KittyIndex) -> DispatchResult {
			let buyer = ensure_signed(origin)?;

			let owner = Self::kitty_owner(kitty_id).unwrap();
			ensure!(owner != buyer.clone(), Error::<T>::KittyAlreadyOwned);

			let price = Self::kitties_price(kitty_id).ok_or(Error::<T>::NotForSale)?;

			let reserve = T::ReserveOfKittyCreate::get();


			T::Currency::reserve(&buyer, reserve).map_err(|_| Error::<T>::NotEnoughBalance)?;


			T::Currency::unreserve(&owner, reserve);


			T::Currency::transfer(
				&buyer,
				&owner,
				price,
				frame_support::traits::ExistenceRequirement::KeepAlive,
			)?;


			KittiesPrice::<T>::remove(kitty_id);

			Self::transfer_kitty(owner, buyer, kitty_id);

			Ok(())
		}
	
	}

	impl<T: Config> Pallet<T> {
		fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload =(
				T::Randomness::random_seed(), 
				&sender, 
				//frame_system::Pallet::<T>::extrinsic_index()
				<frame_system::Pallet<T>>::extrinsic_index()
			);
			payload.using_encoded(blake2_128)
		}

		fn transfer_kitty(source: T::AccountId, target: T::AccountId, kitty_id: T::KittyIndex) {
			KittyOwner::<T>::insert(kitty_id, target.clone());
			Self::deposit_event(Event::KittyTransfered(source, target, kitty_id));

		}
	}
}
