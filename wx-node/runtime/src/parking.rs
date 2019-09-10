use codec::{Decode, Encode};
use rstd::{prelude::*, result, convert::TryInto};
// sr_primitives include so many stds
use sr_primitives::traits::{Hash, CheckedMul, CheckedSub};
use support::{
    decl_event, decl_module, decl_storage, dispatch::Result, ensure, traits::Currency, StorageMap,
    StorageValue,
};
use system::ensure_signed;


/// For Currency
type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

/// The module's configuration trait.
pub trait Trait: timestamp::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    /// The currency we use
    type Currency: Currency<Self::AccountId>;
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq)]
pub struct ParkingLot<T: Trait> {
    pub name: Vec<u8>,
    pub owner: T::AccountId,
    pub remain: u32,
    pub capacity: u32,
    pub min_price: BalanceOf<T>,
    pub max_price: BalanceOf<T>,
    pub latitude: i32,
    pub longitude: i32,
}

pub const HOUR: u64 = 3600;



impl<T: Trait> ParkingLot<T> {
    pub fn fee(&self, enter_time: T::Moment, exit_time: T::Moment) -> result::Result<BalanceOf<T>, &'static str> {
        let diff = (exit_time - enter_time) / to_moment::<T>(HOUR)?;
        let diff: u32 = TryInto::<u32>::try_into(diff).map_err(|_| "Time diff overflow")?;
        self.min_price.checked_mul(&BalanceOf::<T>::from(diff)).ok_or("Fee overflow")
    }

    pub fn compute_new_fee(&self, new_time: T::Moment, old_time: T::Moment) -> result::Result<BalanceOf<T>, &'static str> {
        let current_num = self.capacity.checked_sub(self.remain.clone()).ok_or("Remained num greater than capacity")?;
        let diff_time = new_time.checked_sub(&old_time).ok_or("current time must greater than exiting time")?;
        let diff_time: u32 = TryInto::<u32>::try_into(diff_time).map_err(|_| "Time diff overflow")?;
        let diff_price = self.max_price.checked_sub(&self.min_price).ok_or("Max price must be greater than min price")?;
        let diff_price: u32 = TryInto::<u32>::try_into(diff_price).map_err(|_| "Price diff overflow")?;
        let min_price: u32 = TryInto::<u32>::try_into(self.min_price).map_err(|_| "Min price overflow")?;
        let res = diff_time * (current_num / self.capacity * diff_price + min_price);
        Ok(res.into())
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


// TODO: Refactor
decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        Moment = <T as timestamp::Trait>::Moment,
        EnteringInfo = ParkingInfo<T>,
        LeavingInfo = ParkingInfo<T>,
        ParkingLotInfo = ParkingLot<T>,
    {
        /// Deposit a new parking lot
        NewParkingLot(Moment, ParkingLotInfo),
        /// Deposit a event that current user enter the parking lot
        Entering(Moment, EnteringInfo),
        /// Deposit a event that current user leave the parkint lot
        Leaving(Moment, AccountId, AccountId, LeavingInfo),
        
        SomethingStored(u32, AccountId, Moment),
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

        /// Total number of parking lots
        AllParkingLotsCount get(all_parking_lots_count): u64;
        /// Parking info of current user
        UserParkingInfo get(user_parking_info): map T::AccountId => Option<ParkingInfo<T>>;
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        // this is needed only if you are using events in your module
        fn deposit_event() = default;

        pub fn do_something(origin, something: u32) -> Result {
            let who = ensure_signed(origin)?;
            // For example: the following line stores the passed in u32 in the storage
            Something::put(something);
            // here we are raising the Som  ething event
            Self::deposit_event(RawEvent::SomethingStored(something, who, <timestamp::Module<T>>::get()));
            Ok(())
        }

        /// Create a new parking lot
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

            <ParkingLots<T>>::insert(parking_lot_hash, parking.clone());
            <OwnerParkingLotsArray<T>>::insert((owner.clone(), count), parking_lot_hash);
            <OwnerParkingLotsCount<T>>::insert(&owner, count+1);
            AllParkingLotsCount::put(all+1);

            Self::deposit_event(RawEvent::NewParkingLot(<timestamp::Module<T>>::get(), parking));
            Ok(())
        }

        /// User entering
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

            let mut accs = Self::current_parking_accounts(parking_lot_hash.clone());
            accs.push(user.clone());
            parking_lot.remain -= 1;

            // change states
            <CurrentParkingAccounts<T>>::insert(parking_lot_hash.clone(), accs);
            <ParkingLots<T>>::insert(parking_lot_hash, parking_lot);
            <UserParkingInfo<T>>::insert(user.clone(), parking_info.clone());

            Self::deposit_event(RawEvent::Entering(<timestamp::Module<T>>::get(), parking_info));
            Ok(())
        }

        /// User leaving
        pub fn leaving(origin) -> Result {
            let user = ensure_signed(origin)?;
            let parking_info = Self::user_parking_info(user.clone()).ok_or("User has not entered a parking lot")?;
            let parking_lot_hash = parking_info.parking_lot_hash.clone();
            let mut parking_lot = Self::parking_lots(parking_lot_hash).ok_or("The parking lot has not existed")?;
            let owner = parking_lot.owner.clone();
            let accs: Vec<_> = Self::current_parking_accounts(parking_lot_hash.clone());
            let mut new_accs = vec![];
            for acc in accs {
                if acc != user {
                    new_accs.push(acc);
                }
            }

            // update fees first, and then pay the fee and remove parking info
            // change states
            Self::pay_parking_fee(user.clone(), &parking_lot)?;
            parking_lot.remain += 1;

            <CurrentParkingAccounts<T>>::insert(parking_lot_hash.clone(), new_accs);
            <ParkingLots<T>>::insert(parking_lot_hash, parking_lot.clone());
            <UserParkingInfo<T>>::remove(user.clone());

            Self::deposit_event(RawEvent::Leaving(<timestamp::Module<T>>::get(), user, owner, parking_info));
            Ok(())
        }
    }
}


impl<T: Trait> Module<T> {
        /// Pay parking fee when user leaving
        fn pay_parking_fee(user: T::AccountId, parking_lot: &ParkingLot<T>) -> Result {
            let parking_info = Self::user_parking_info(user.clone()).ok_or("User must be in the parking lot")?;
            ensure!(parking_info.user_id == user, "User must be in the parking lot");

            let owner = parking_lot.owner.clone();
            let now = <timestamp::Module<T>>::get();
            let parking_lot_hash = parking_info.parking_lot_hash.clone();
            let old_time = parking_info.current_time.clone();

            Self::recompute_all_fee(parking_lot, parking_lot_hash, now, old_time)?;
            // Recompute all fees before paying
            let new_parking_info = Self::user_parking_info(user.clone()).expect("User must be existed. Qed");
            T::Currency::transfer(&user, &owner, new_parking_info.current_fee)
        }

        /// Recompute all parking info for current parking lot
        fn recompute_all_fee(parking_lot: &ParkingLot<T>, parking_lot_hash: T::Hash, new_time: T::Moment, old_time: T::Moment) -> Result {
            let new_fee = parking_lot.compute_new_fee(new_time, old_time)?;
            let accs: Vec<_> = Self::current_parking_accounts(parking_lot_hash);
            
            // change states
            for acc in accs {
                let mut parking_info = Self::user_parking_info(acc.clone()).ok_or("User not exists")?;
                parking_info.current_time = new_time;
                parking_info.current_fee += new_fee;
                <UserParkingInfo<T>>::insert(acc, parking_info);
            }

            Ok(())
        }
}


/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use primitives::{Blake2Hasher, H256};
    use runtime_io::{with_externalities, TestExternalities};
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

	parameter_types! {
		pub const ExistentialDeposit: u64 = 0;
		pub const TransferFee: u64 = 0;
		pub const CreationFee: u64 = 0;
		pub const TransactionBaseFee: u64 = 0;
		pub const TransactionByteFee: u64 = 0;
	}

    impl balances::Trait for Test {
        type Balance = u64;
		type OnFreeBalanceZero = ();
		type OnNewAccount = ();
		type Event = ();
		type TransactionPayment = ();
		type TransferPayment = ();
		type DustRemoval = ();
		type ExistentialDeposit = ExistentialDeposit;
		type TransferFee = TransferFee;
		type CreationFee = CreationFee;
		type TransactionBaseFee = TransactionBaseFee;
		type TransactionByteFee = TransactionByteFee;
		type WeightToFee = ();
    }

    parameter_types! {
        pub const MinimumPeriod: u64 = 1000;
    }

    impl timestamp::Trait for Test {
        type Moment = u64;
        type OnTimestampSet = ();
        type MinimumPeriod = MinimumPeriod;
    }

    impl Trait for Test {
        type Event = ();
        type Currency = balances::Module<Test>;
    }
    
    type Parking = Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> TestExternalities<Blake2Hasher> {
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

    // TODO: unit test
    fn test_new_parking_lot() {
        with_externalities(&mut new_test_ext(), || {
            assert!(true);
        })
    }

    fn test_entering() {
        with_externalities(& mut new_test_ext(), || {
            assert!(true);
        })
    }

    fn test_leaving() {
        with_externalities(& mut new_test_ext(), || {
            assert!(true);
        })
    }
}
