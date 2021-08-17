#![cfg_attr(not(feature = "std"), no_std)]

// A module for proof of existence
pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

//定义功能模块
#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResultWithPostInfo, 
        pallet_prelude::*
    };
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec; 
    
    //定义配置接口
    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type ClaimSize: Get<usize>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);
    
    //定义存储单元
    #[pallet::storage]
    #[pallet::getter(fn proofs)]
    pub type Proofs<T: Config> = StorageMap<
        _, 
        Blake2_128Concat, 
        Vec<u8>, 
        (T::AccountId, T::BlockNumber)
    >;   

    //定义事件
    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config>{
        ClaimCreated(T::AccountId, Vec<u8>),
        ClaimRevoked(T::AccountId, Vec<u8>),
        ClaimTransferred(T::AccountId, Vec<u8>, T::AccountId),
    }
    
    //定义错误信息
    #[pallet::error]   // <-- Step 4. code block will replace this.
    pub enum Error<T>{
        ProofAlreadyExist,
        ClaimNotExist,
        NotProofOwner,
        ClaimSizeTooLarge,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}
    
    //定义可调用函数
    #[pallet::call]
    impl<T: Config> Pallet<T>{
        //创建存证
        #[pallet::weight(0)]
        pub fn create_claim(
            origin: OriginFor<T>,
            claim: Vec<u8>,
        ) -> DispatchResultWithPostInfo {

            let sender = ensure_signed(origin)?;
             
            ensure!(!Proofs::<T>::contains_key(&claim), Error::<T>::ProofAlreadyExist);

            //检查存证内容大小，是否小于ClaimSize上限
            ensure!(claim.len() <= T::ClaimSize::get(), Error::<T>::ClaimSizeTooLarge);

            // Get the block number from the FRAME System module.
            let current_block = <frame_system::Pallet<T>>::block_number();

            // Store the claim with the sender and block number.
            Proofs::<T>::insert(
                &claim, 
                (sender.clone(), current_block)
            );

            // Emit an event that the claim was created.
            Self::deposit_event(Event::ClaimCreated(sender, claim));

            Ok(().into())
        }
        
        //撤销存证
        #[pallet::weight(0)]
        pub fn revoke_claim(
            origin: OriginFor<T>,
            claim: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            
            let sender = ensure_signed(origin)?;

            // Get owner of the claim.
            let (owner, _) = Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?;

            // Verify that sender of the current call is the claim owner.
            ensure!(sender == owner, Error::<T>::NotProofOwner);

            // Remove claim from storage.
            Proofs::<T>::remove(&claim);

            // Emit an event that the claim was erased.
            Self::deposit_event(Event::ClaimRevoked(sender, claim));

            Ok(().into())
        }
        

        //转移存证
        #[pallet::weight(0)]
        pub fn transfer_claim(
            origin: OriginFor<T>,
            claim: Vec<u8>,
            receiver: T::AccountId,
        )-> DispatchResultWithPostInfo{
            //判断身份是否有效
            let sender = ensure_signed(origin)?;
        
            //查询存证拥有者
            let (owner, _) = Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?;

            //判断调用者是否是存证拥有者
            ensure!(sender == owner, Error::<T>::NotProofOwner);

            //得到当前区块高度
            let current_block = <frame_system::Pallet<T>>::block_number();

            // 覆盖origin的value
            Proofs::<T>::insert(
                &claim, 
                (receiver.clone(), current_block)
            );
            //触发事件ClaimTransferred
            Self::deposit_event(Event::ClaimTransferred(owner, claim, receiver));

            Ok(().into())
        }

    }
}