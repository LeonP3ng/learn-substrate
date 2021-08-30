#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet{
    use frame_support::{
        dispatch::DispatchResult, 
        pallet_prelude::*,
        traits::{Randomness, Currency, ReservableCurrency, ExistenceRequirement}
    };
    use frame_system::pallet_prelude::*;
    use codec::{Encode, Decode};
    use sp_io::hashing::blake2_128;
    use sp_runtime::{
        traits::{AtLeast32BitUnsigned,Bounded}
    };

    #[derive(Encode, Decode)]
    pub struct Kitty(pub [u8;16]);

	type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    //定义配置接口
    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
        type KittyIndex: Parameter + Member + AtLeast32BitUnsigned + Default + Copy;
        type KittyReserveMoney: Get<BalanceOf<Self>>;
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config>{
        KittyCreate(T::AccountId, T::KittyIndex),
        Kittytransfer(T::AccountId, T::AccountId, T::KittyIndex),
        //KittyBuy(who, owner, kitty_id, kitty_price)
        KittyOnSale(T::AccountId, T::KittyIndex, BalanceOf<T>),
        KittyBuy(T::AccountId, T::KittyIndex, BalanceOf<T>),
    }

    //Kitty数量
    #[pallet::storage]
    #[pallet::getter(fn kitties_count)]
    pub type KittiesCount<T: Config> = StorageValue<_, T::KittyIndex>;

    //Kitty实体
    #[pallet::storage]
    #[pallet::getter(fn kitties)]
    pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<Kitty>, ValueQuery>;
     
    //Kitty持有者
    #[pallet::storage]
    #[pallet::getter(fn kitty_owner)]
    pub type Owner<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<T::AccountId>, ValueQuery>;

    //Kitty售价
    #[pallet::storage]
    #[pallet::getter(fn kitties_price)]
    pub type KittiesPrice<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<BalanceOf<T>>, ValueQuery>;

    #[pallet::error]
    pub enum Error<T>{
        //Kitty数量溢出
        KittiesCountOverflow,
        //不是拥有者
        NotOwner,
        //Kitty父母相同
        SameParentIndex,
        //无效的KittyIndex
        InvalidKittyIndex,
        //Kitty购买者和拥有者相同
        InvalidKittyBuyer,
        //Kitty处于非卖状态
        KittyIsNotOnSale,
        //金额不够
        MoneyIsNotEnough
    }


    #[pallet::call]
    impl<T: Config> Pallet<T>{
        #[pallet::weight(0)]
        //创建Kitty
        pub fn create(origin: OriginFor<T>) -> DispatchResult{
            let who = ensure_signed(origin)?;
        
            let kitty_id = match Self::kitties_count() {
                Some(id) => {
                    ensure!(id != T::KittyIndex::max_value(), Error::<T>::KittiesCountOverflow);
                    id.into()
                },
                None => {
                    1u32.into()
                }
            };
            //质押代币,
            T::Currency::reserve(&who, T::KittyReserveMoney::get()).map_err(|_| Error::<T>::MoneyIsNotEnough)?;

            let dna = Self::random_value(&who);
            
            Self::insert_kitty(who.clone(), kitty_id, Kitty(dna));

            Self::deposit_event(Event::KittyCreate(who, kitty_id));

            Ok(())
        }

        //转移Kitty所有权
        #[pallet::weight(0)]
        pub fn transfer(origin: OriginFor<T>, new_owner: T::AccountId, kitty_id: T::KittyIndex) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(Some(who.clone()) == Owner::<T>::get(kitty_id), Error::<T>::NotOwner);

            Self::transfer_kitty(who, new_owner, kitty_id);
    
            Ok(())
        }

        //出售Kitty
        #[pallet::weight(0)]
        pub fn  sell_kitty(origin: OriginFor<T>, kitty_id: T::KittyIndex, kitty_price: BalanceOf<T>) -> DispatchResult{
            let who = ensure_signed(origin)?;

            let owner = Self::kitty_owner(kitty_id).ok_or(Error::<T>::InvalidKittyIndex)?;

            ensure!(owner == who.clone(), Error::<T>::NotOwner);

            KittiesPrice::<T>::insert(kitty_id, Some(kitty_price));
            
            Self::deposit_event(Event::KittyOnSale(who, kitty_id, kitty_price));
            
            Ok(())
        }

        //购买Kitty
        #[pallet::weight(0)]
        pub fn buy_kitty(origin: OriginFor<T>, kitty_id: T::KittyIndex, pay_value: BalanceOf<T>) -> DispatchResult{
            let who = ensure_signed(origin)?;

            let owner = Self::kitty_owner(kitty_id).ok_or(Error::<T>::InvalidKittyIndex)?;

            ensure!(owner != who.clone(), Error::<T>::InvalidKittyBuyer);

            //判断购买的Kitty是否存在
            let kitty_price = Self::kitties_price(kitty_id).ok_or(Error::<T>::KittyIsNotOnSale)?;

            //判断金额
            ensure!(pay_value >= kitty_price, Error::<T>::MoneyIsNotEnough);

            //转账
			T::Currency::transfer(&who, &owner, kitty_price, ExistenceRequirement::KeepAlive).map_err(|_| Error::<T>::MoneyIsNotEnough)?;

            T::Currency::unreserve(&owner, T::KittyReserveMoney::get());
            //删除Kitty，防止重复购买
            KittiesPrice::<T>::remove(kitty_id);

            //转移Kitty所有权
            Self::transfer_kitty(owner, who.clone(), kitty_id);

            Self::deposit_event(Event::KittyBuy(who, kitty_id, kitty_price));

            Ok(())
        }

        //繁殖Kitty
        #[pallet::weight(0)]
        pub fn breed(origin: OriginFor<T>, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameParentIndex);

            let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyIndex)?;
            let kitty2 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyIndex)?;

            let kitty_id = match Self::kitties_count() {
                Some(id) => {
                    ensure!(id != T::KittyIndex::max_value(), Error::<T>::KittiesCountOverflow);
                    id.into()
                },
                None => {
                    1u32.into()
                }
            };
            
            let dna_1 = kitty1.0;
            let dna_2 = kitty2.0;

            let selector = Self::random_value(&who);

            let mut new_dna = [0u8; 16];

            for i in 0..dna_1.len() {
                new_dna[i] = (selector[i] & dna_1[i]) | (!selector[i] & dna_2[i]);
            }

            Self::insert_kitty(who.clone(), kitty_id, Kitty(new_dna));

            Self::deposit_event(Event::KittyCreate(who, kitty_id));

            Ok(())
        }
    }

    //提取公共代码
    impl<T: Config> Pallet<T>{
        fn random_value(sender: &T::AccountId) -> [u8; 16] {
            let payload = (
                T::Randomness::random_seed(),
                &sender,
                <frame_system::Pallet<T>>::extrinsic_index(),
            );

            payload.using_encoded(blake2_128)
        }


        fn transfer_kitty(owner: T::AccountId, new_owner: T::AccountId, kitty_id: T::KittyIndex){
            Owner::<T>::insert(kitty_id, Some(new_owner.clone()));

            Self::deposit_event(Event::Kittytransfer(owner, new_owner, kitty_id));
        }

        fn insert_kitty(owner: T::AccountId, kitty_id: T::KittyIndex, kitty: Kitty){
            Kitties::<T>::insert(kitty_id, Some(kitty));

            Owner::<T>::insert(kitty_id, Some(owner));

            KittiesCount::<T>::put(kitty_id + 1u32.into());
        }

       
    }
}