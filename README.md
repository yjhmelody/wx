# wx(WIP)

## 前后端

### 功能

- 车场主创建新的停车场(new_parking_lot)
- 入库(entering)
- 出库(leaving)
- 付费(transfer_parking_fee)
- 设置地图来显示停车场，demo上面可以设置四个停车场。
- 其他一些常用地图功能

### 数据展示

- 显示停车场的经纬度，名称，剩余停车位数量(ParkingLot)
- 用户的车入库时间(ParkingInfo)
- 用户当前费用，每次该车场有车出入库`entering`和`leaving`时，所有该车场的用户停车费会刷新(TODO)。
- 显示停车场的一周历史平均车流量(TODO)。


### Event约定

TODO

### 一些约定

纯展示用的数据由前端来编解码，后端只负责存储。