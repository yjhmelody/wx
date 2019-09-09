use codec::{Decode, Encode};
use rstd::{prelude::*, result, convert::TryInto};
// sr_primitives include so many stds
use sr_primitives::traits::{Hash, CheckedMul, CheckedSub};
/// A runtime module template with necessary imports

use support::{
    decl_event, decl_module, decl_storage, dispatch::Result, ensure, traits::Currency, StorageMap,
    StorageValue,
};

use system::ensure_signed;


type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

/// The module's configuration trait.
pub trait Trait: timestamp::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Currency: Currency<Self::AccountId>;
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct ParkingLot<T: Trait> {
    name: Vec<u8>,
    owner: T::AccountId,
    remain: u32,
    capacity: u32,
    min_price: BalanceOf<T>,
    max_price: BalanceOf<T>,
    latitude: i32,
    longitude: i32,
}

pub const HOUR: u64 = 3600;



impl<T: Trait> ParkingLot<T> {
    // TODO: refactor fee model
    pub fn fee(&self, enter_time: T::Moment, exit_time: T::Moment) -> result::Result<BalanceOf<T>, &'static str> {
        let diff = (exit_time - enter_time) / to_moment::<T>(HOUR)?;
        let diff: u32 = TryInto::<u32>::try_into(diff).map_err(|_| "Time diff overflow")?;
        self.min_price.checked_mul(&BalanceOf::<T>::from(diff)).ok_or("Fee overflow")
    }

    pub fn compute_new_fee(&self, current_time: T::Moment, exit_time: T::Moment) -> result::Result<BalanceOf<T>, &'static str> {
        let current_num = self.capacity.checked_sub(self.remain.clone()).ok_or("Remained num greater than capacity")?;
        let diff_time = current_time.checked_sub(exit_time).ok_or("current time must greater than exiting time")?;
        let diff_time: u32 = TryInto::<u32>::try_into(diff_time).map_err(|_| "Time diff overflow")?;
        let diff_price = self.max_price.checked_sub(self.min_price).ok_or("Max price must be greater than min price")?;
        let diff_price: u32 = TryInto::<u32>::try_into(diff_price).map_err(|_| "Price diff overflow")?;
        Ok(
            diff_time * (current_num / self.capacity * diff_price + self.min_price)
        )
    }
}

fn to_balance<T: Trait>(val: u128) -> result::Result<BalanceOf<T>, &'static str > {
            val.try_into().map_err(|_| "Convert to Balance type overflow")
        }

fn to_moment<T: Trait>(val: u64) -> result::Result<T::Moment, &'static str > {
    val.try_into().map_err(|_| "Convert to Moment type overflow")
}


#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq)]
pub struct ParkingInfo<T: Trait> {
    pub user_id: T::AccountId,
    pub parking_lot_hash: T::Hash,
    pub info_hash: T::Hash,
    pub enter_time: T::Moment,
    pub current_time: T::Moment,
    pub current_fee: BalanceOf<T>,
}

impl<T: Trait> ParkingInfo<T> {
    pub fn new(user_id: T::AccountId, parking_lot_hash: T::Hash, info_hash: T::Hash, enter_time: T::Moment) -> Self {
        Self {
            user_id,
            parking_lot_hash,
            info_hash,
            enter_time,
            current_time: enter_time.clone(),
            current_fee: 0.into(),
        }
    }
}


#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default)]
pub struct ParkingOwnerInfo<T: Trait> {
    pub id: T::AccountId,
    pub parking_lot_count: u32,
}


// TODO: Design the interface
decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        Moment = <T as timestamp::Trait>::Moment,
        // EnteringInfo = ParkingInfo<T>,
        // LeavingInfo = ParkingInfo<T>,
    {
        SomethingStored(u32, AccountId, Moment),
        // Entering(EnteringInfo),
        // Leaving(LeavingInfo),
        // TransferFee(AccountId, AccountId),
    }
);


// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as Parking {
        Something get(something): Option<u32>;
        /// Current parking lot count of a owner
        OwnerParkingLotsCount get(owner_parking_lots_count): map T::AccountId => u64;
        /// Access the all parking lot infos
        OwnerParkingLotsArray get(owner_parking_lots_array): map (T::AccountId, u64) => T::Hash;
        /// Hash map to one parking lot
        ParkingLots get(parking_lots): map T::Hash => Option<ParkingLot<T>>;
        /// All user id of current parking lot
        CurrentParkingAccounts get(current_parking_accounts): map T::Hash => Vec<T::AccountId>;

        /// All parking lots' number
        AllParkingLotsCount get(all_parking_lots_count): u64;
        /// current parking info of users
        UserParkingInfo get(user_parking_info): map T::AccountId => Option<ParkingInfo<T>>;
        /// All parking infos
        // ParkingInfos get(parking_infos): map T::Hash => Option<ParkingInfo<T>>;
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
            let who = ensure_signed(origin)?;
            // For example: the following line stores the passed in u32 in the storage
            Something::put(something);
            // here we are raising the Som  ething event
            Self::deposit_event(RawEvent::SomethingStored(something, who, <timestamp::Module<T>>::get()));
            Ok(())
        }

        /// 经纬度需要前端把浮点数转为整数，这里只负责存储，不负责解析
        pub fn new_parking_lot(origin, name: Vec<u8>, latitude: i32, longitude: i32, capacity: u32, min_price: BalanceOf<T>, max_price: BalanceOf<T>) -> Result {
            let owner = ensure_signed(origin)?;
            ensure!(name.len() < 100, "Parking Lot name cannot be more than 100 bytes");
            let parking: ParkingLot<T> = ParkingLot {
                name,
                owner: owner.clone(),
                capacity,
                min_price,
                max_price,
                remain: capacity,
                latitude,
                longitude,
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

        /// 交停车费，在leaving之前调用
        // TODO: 跟leaving合并
        pub fn transfer_parking_fee(origin, parking_lot_hash: T::Hash) -> Result {
            let user = ensure_signed(origin)?;

            let parking_lot = Self::parking_lots(parking_lot_hash).ok_or("The parking lot has not existed")?;
            let parking_info = Self::user_parking_info(user.clone()).ok_or("User must be in the parking lot")?;
            ensure!(parking_info.user_id == user, "User must be in the parking lot");

            let owner = parking_lot.owner.clone();
            let now = <timestamp::Module<T>>::get();
            let fee = parking_lot.fee(parking_info.enter_time, now)?;

            T::Currency::transfer(&user, &owner, fee)?;

            // Self::deposit_event(RawEvent::TransferFee());
            Ok(())
        }

        /// 用户车入库
        pub fn entering(origin, parking_lot_hash: T::Hash) -> Result {
            let user = ensure_signed(origin)?;
            ensure!(!<UserParkingInfo<T>>::exists(user.clone()), "User already has entered a parking lot");

            let info_hash = <system::Module<T>>::random_seed();
            // record the entering time
            let now = <timestamp::Module<T>>::get();
            let parking_info = ParkingInfo::<T>::new(user.clone(), parking_lot_hash, info_hash, now);
            let mut parking_lot = Self::parking_lots(parking_lot_hash).ok_or("The parking lot has not existed")?;
            if parking_lot.remain <= 0 {
                return Err("The parking lot has no more position");
            }

            let mut accs = Self::current_parking_accounts::<T>();
            accs.push(user.clone());
            parking_lot.remain -= 1;


            <CurrentParkingAccounts<T>>::insert(parking_lot_hash.clone(), accs);
            <ParkingLots<T>>::insert(parking_lot_hash, parking_lot);
            <UserParkingInfo<T>>::insert(user.clone(), parking_info.clone());

            // Self::deposit_event(RawEvent::Entering(parking_info));
            Ok(())
        }

        /// 用户车出库
        pub fn leaving(origin) -> Result {
            let user = ensure_signed(origin)?;
            let parking_info = Self::user_parking_info(user.clone()).ok_or("User has not entered a parking lot")?;
            let parking_lot_hash = parking_info.parking_lot_hash.clone();
            let mut parking_lot = Self::parking_lots(parking_lot_hash).ok_or("The parking lot has not existed")?;
            let accs: Vec<_> = Self::current_parking_accounts::<T>();

            // update fees first, and then pay the fee and remove parking info
            Self::pay_parking_fee(user.clone(), &parking_lot)?;
            parking_lot.remain += 1;
            let mut new_accs = vec![];
            for acc in accs {
                if acc != user {
                    new_accs.push(acc);
                }
            }


            <CurrentParkingAccounts<T>>::insert(parking_lot_hash.clone(), new_accs);
            <ParkingLots<T>>::insert(parking_lot_hash, parking_lot.clone());
            <UserParkingInfo<T>>::remove(user);

            // Self::deposit_event(RawEvent::Leaving(parking_info));
            Ok(())
        }
    }
}


impl<T: Trait> Module<T> {
        fn pay_parking_fee(user: T::AccountId, parking_lot: &ParkingLot<T>) -> Result {
            let parking_info = Self::user_parking_info(user.clone()).ok_or("User must be in the parking lot")?;
            ensure!(parking_info.user_id == user, "User must be in the parking lot");

            let owner = parking_lot.owner.clone();
            let now = <timestamp::Module<T>>::get();
            let fee = parking_lot.fee(parking_info.enter_time, now)?;

            T::Currency::transfer(&user, &owner, fee)
        }

        /// Recompute all parking info for current parking lot
        fn recompute_all_fee(parking_lot: &ParkingLot<T>, current_time: T::Moment, exit_time: T::Moment) -> Result {
            let new_fee = parking_lot.compute_new_fee(current_time, exit_time)?;
            unimplemented!()
        }
}


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
        type Currency = balances::Module<Test>;
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
