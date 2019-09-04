use codec::{Decode, Encode};
use rstd::{prelude::*, result};
use sr_primitives::traits::Hash;
/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references

/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/srml/example/src/lib.rs
use support::{
    decl_event, decl_module, decl_storage, dispatch::Result, ensure, traits::Currency, StorageMap,
    StorageValue,
};
use system::ensure_signed;

/// The module's configuration trait.
pub trait Trait: balances::Trait {
    // TODO: Add other types and constants required configure this module.

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct ParkingLot<T: Trait> {
    name: Vec<u8>,
    owner: T::AccountId,
    pub remain: u32,
    capacity: u32,
    min_price: T::Balance,
    max_price: T::Balance,
}

impl<T: Trait> ParkingLot<T> {
    pub fn fee(&self, _cur_time: u64) -> u64 {
        unimplemented!()
    }
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq)]
pub struct ParkingInfo<T: Trait> {
    user_id: T::AccountId,
    parking_lot_hash: T::Hash,
    info_hash: T::Hash,
}

impl<T: Trait> ParkingInfo<T> {
    pub fn new(user_id: T::AccountId, parking_lot_hash: T::Hash, info_hash: T::Hash) -> Self {
        Self {
            user_id,
            parking_lot_hash,
            info_hash,
        }
    }
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default)]
pub struct ParkingOwnerInfo<T: Trait> {
    id: T::AccountId,
    parking_lot_count: u32,
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as Parking {
        Something get(something): Option<u32>;

        // parking lot info
        OwnerParkingLotsCount get(owner_parking_lots_count): map T::AccountId => u64;
        OwnerParkingLotsArray get(owner_parking_lots_array): map (T::AccountId, u64) => T::Hash;
        ParkingLots get(parking_lots): map T::Hash => Option<ParkingLot<T>>;
        AllParkingLotsCount get(all_parking_lots_count): u64;

        // user choose a parking lot
        UserParkingInfo get(user_parking_info): map T::AccountId => Option<ParkingInfo<T>>;
        ParkingInfos get(parking_infos): map T::Hash => Option<ParkingInfo<T>>;
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        // this is needed only if you are using events in your module
        fn deposit_event() = default;

        // Just a dummy entry point.
        // function that can be called by the external world as an extrinsics call
        // takes a parameter of the type `AccountId`, stores it and emits an event
        pub fn do_something(origin, something: u32) -> Result {
            // TODO: You only need this if you want to check it was signed.
            let who = ensure_signed(origin)?;

            // TODO: Code to execute when something calls this.
            // For example: the following line stores the passed in u32 in the storage
            Something::put(something);

            // here we are raising the Something event
            Self::deposit_event(RawEvent::SomethingStored(something, who));
            Ok(())
        }

        pub fn new_parking_lot(origin, name: Vec<u8>, capacity: u32, min_price: T::Balance, max_price: T::Balance) -> Result {
            let owner = ensure_signed(origin)?;
            ensure!(name.len() < 100, "Parking Lot name cannot be more than 100 bytes");
            let parking: ParkingLot<T> = ParkingLot {
                name,
                owner: owner.clone(),
                capacity,
                min_price,
                max_price,
                remain: capacity,
            };

            let count = Self::owner_parking_lots_count(owner.clone());
            let all = Self::all_parking_lots_count();

            let parking_lot_hash = (<system::Module<T>>::random_seed(), &owner, count, all)
                .using_encoded(<T as system::Trait>::Hashing::hash);

            <ParkingLots<T>>::insert(parking_lot_hash, parking);
            <OwnerParkingLotsArray<T>>::insert((owner.clone(), count), parking_lot_hash);
            <OwnerParkingLotsCount<T>>::insert(&owner, count+1);
            AllParkingLotsCount::put(all+1);
            Ok(())
        }

        pub fn transfer_parking_fee(origin, parking_lot_hash: T::Hash) -> Result {
            let user = ensure_signed(origin)?;

            let parking_lot = Self::parking_lots(parking_lot_hash).ok_or("The parking lot has not existed")?;
            // TODO: set a price
            let price = parking_lot.min_price;
            let owner = parking_lot.owner;

            <balances::Module<T> as Currency<_>>::transfer(&user, &owner, price)?;

            Ok(())
        }

        pub fn entering(origin, parking_lot_hash: T::Hash) -> Result {
            let user = ensure_signed(origin)?;
            ensure!(!<UserParkingInfo<T>>::exists(user.clone()), "User already has entered a parking lot");

            let info_hash = <system::Module<T>>::random_seed();
            let parking_info = ParkingInfo::<T>::new(user.clone(), parking_lot_hash, info_hash);
            let mut parking_lot = Self::parking_lots(parking_lot_hash).ok_or("The parking lot has not existed")?;
            if parking_lot.remain <= 0 {
                return Err("The parking lot has no more position");
            }

            parking_lot.remain -= 1;
            <ParkingLots<T>>::insert(parking_lot_hash, parking_lot);
            <UserParkingInfo<T>>::insert(user.clone(), parking_info.clone());


            Self::deposit_event(RawEvent::Entering(parking_info));
            Ok(())
        }

        pub fn leaving(origin) -> Result {
            let user = ensure_signed(origin)?;
            let parking_info = Self::user_parking_info(user.clone()).ok_or("User has not entered a parking lot")?;
            let parking_lot_hash = parking_info.parking_lot_hash.clone();
            let mut parking_lot = Self::parking_lots(parking_lot_hash).ok_or("The parking lot has not existed")?;

            parking_lot.remain += 1;
            <ParkingLots<T>>::insert(parking_lot_hash, parking_lot.clone());
            <UserParkingInfo<T>>::remove(user);

            Self::deposit_event(RawEvent::Leaving(parking_info));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        EnteringInfo = ParkingInfo<T>,
        LeavingInfo = ParkingInfo<T>,
    {
        SomethingStored(u32, AccountId),
        Entering(EnteringInfo),
        Leaving(LeavingInfo),
    }
);

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use primitives::{Blake2Hasher, H256};
    use runtime_io::with_externalities;
    use sr_primitives::weights::Weight;
    use sr_primitives::Perbill;
    use sr_primitives::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
    };
    use support::{assert_ok, impl_outer_origin, parameter_types};

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const MaximumBlockWeight: Weight = 1024;
        pub const MaximumBlockLength: u32 = 2 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    }
    impl system::Trait for Test {
        type Origin = Origin;
        type Call = ();
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type WeightMultiplierUpdate = ();
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type Version = ();
    }
    impl Trait for Test {
        type Event = ();
    }
    type Parking = Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap()
            .into()
    }

    #[test]
    fn it_works_for_default_value() {
        with_externalities(&mut new_test_ext(), || {
            // Just a dummy test for the dummy funtion `do_something`
            // calling the `do_something` function with a value 42
            assert_ok!(Parking::do_something(Origin::signed(1), 42));
            // asserting that the stored value is equal to what we stored
            assert_eq!(Parking::something(), Some(42));
        });
    }
}
