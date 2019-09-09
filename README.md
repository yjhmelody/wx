# wx(WIP)

## 前后端

### 功能

- 车场主创建新的停车场(new_parking_lot)
- 入库(entering)
- 出库(leaving)
- 设置地图来显示停车场，demo上面可以设置四个停车场。
- 其他一些常用地图功能

### 数据展示

- 显示停车场的经纬度，名称，剩余停车位数量(ParkingLot)
- 用户的车入库时间(ParkingInfo)
- 用户当前费用，每次该车场有车出入库`entering`和`leaving`时，所有该车场的用户停车费会刷新(TODO)。
- 显示停车场的一周历史平均车流量(TODO)。


### Event约定


以下是 substrate 发送的事件
```rust
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
```

相应的JSON如下

```json
{
  "ParkingLot": {
    "name": "Vec<u8>",
    "owner": "AccocuntId",
    "remain": "u32",
    "capacity": "u32",
    "min_price": "Balance",
    "max_price": "Balance",
    "latitude": "i32",
    "longitude": "i32"
  },
  "ParkingLotInfo": "ParkingLot",

  "ParkingInfo": {
    "user_id": "AccountId",
    "parking_lot_hash": "H256",
    "info_hash": "H256",
    "enter_time": "Moment",
    "current_time": "Moment",
    "current_fee": "Balance"
  },

  "EnteringInfo": "ParkingInfo",
  "LeavingInfo": "ParkingInfo"
}
```

### 一些约定

纯展示用的数据由前端来编解码，后端只负责存储。