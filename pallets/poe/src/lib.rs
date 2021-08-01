#![cfg_attr(not(feature = "std"), no_std)]

// A module for proof of existence
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResultWithPostInfo, 
        pallet_prelude::*
    };
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec; // Step 3.1 will include this in `Cargo.toml`
    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);
    
    #[pallet::storage]
    #[pallet::getter(fn proofs)]
    pub type Proofs<T: Config> = StorageMap<
        _, 
        Blake2_128Concat, 
        Vec<u8>, 
        (T::AccountId, T::BlockNumber)
    >;   
    #[pallet::event]   // <-- Step 3. code block will replace this.
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config>{
        ClaimCreated(T::AccountId, Vec<u8>),
        ClaimRevoked(T::AccountId, Vec<u8>),
        ClaimTransferred(T::AccountId, Vec<u8>, T::AccountId),
    }
    
    #[pallet::error]   // <-- Step 4. code block will replace this.
    pub enum Error<T>{
        ProofAlreadyExist,
        ClaimNotExist,
        NotProofOwner,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}
    
    #[pallet::call]   // <-- Step 6. code block will replace this.
    impl<T: Config> Pallet<T>{
        //创建存证
        #[pallet::weight(0)]
        pub fn create_claim(
            origin: OriginFor<T>,
            claim: Vec<u8>,
        ) -> DispatchResultWithPostInfo {

            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let sender = ensure_signed(origin)?;
        
            // Verify that the specified claim has not already been claimed.         
            ensure!(!Proofs::<T>::contains_key(&claim), Error::<T>::ProofAlreadyExist);

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
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let sender = ensure_signed(origin)?;

            // Verify that the specified proof has been claimed.
            //ensure!(Proofs::<T>::contains_key(&claim), Error::<T>::ClaimNotExist);

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