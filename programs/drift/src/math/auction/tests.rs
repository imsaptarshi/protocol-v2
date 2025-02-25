mod calculate_auction_prices {
    use crate::controller::position::PositionDirection;
    use crate::math::auction::calculate_auction_prices;
    use crate::math::constants::PRICE_PRECISION_I64;
    use crate::state::oracle::OraclePriceData;

    #[test]
    fn no_limit_price_long() {
        let oracle_price_data = OraclePriceData {
            price: 100 * PRICE_PRECISION_I64,
            ..OraclePriceData::default()
        };
        let position_direction = PositionDirection::Long;
        let limit_price = 0;

        let (auction_start_price, auction_end_price) =
            calculate_auction_prices(&oracle_price_data, position_direction, limit_price).unwrap();

        assert_eq!(auction_start_price, 100000000);
        assert_eq!(auction_end_price, 100500000);
    }

    #[test]
    fn no_limit_price_short() {
        let oracle_price_data = OraclePriceData {
            price: 100 * PRICE_PRECISION_I64,
            ..OraclePriceData::default()
        };
        let position_direction = PositionDirection::Short;
        let limit_price = 0;

        let (auction_start_price, auction_end_price) =
            calculate_auction_prices(&oracle_price_data, position_direction, limit_price).unwrap();

        assert_eq!(auction_start_price, 100000000);
        assert_eq!(auction_end_price, 99500000);
    }

    #[test]
    fn limit_price_much_better_than_oracle_long() {
        let oracle_price_data = OraclePriceData {
            price: 100 * PRICE_PRECISION_I64,
            ..OraclePriceData::default()
        };
        let position_direction = PositionDirection::Long;
        let limit_price = 90000000;

        let (auction_start_price, auction_end_price) =
            calculate_auction_prices(&oracle_price_data, position_direction, limit_price).unwrap();

        assert_eq!(auction_start_price, 89550000);
        assert_eq!(auction_end_price, 90000000);
    }

    #[test]
    fn limit_price_slightly_better_than_oracle_long() {
        let oracle_price_data = OraclePriceData {
            price: 100 * PRICE_PRECISION_I64,
            ..OraclePriceData::default()
        };
        let position_direction = PositionDirection::Long;
        let limit_price = 99999999;

        let (auction_start_price, auction_end_price) =
            calculate_auction_prices(&oracle_price_data, position_direction, limit_price).unwrap();

        assert_eq!(auction_start_price, 99500000);
        assert_eq!(auction_end_price, 99999999);
    }

    #[test]
    fn limit_price_much_worse_than_oracle_long() {
        let oracle_price_data = OraclePriceData {
            price: 100 * PRICE_PRECISION_I64,
            ..OraclePriceData::default()
        };
        let position_direction = PositionDirection::Long;
        let limit_price = 110000000;

        let (auction_start_price, auction_end_price) =
            calculate_auction_prices(&oracle_price_data, position_direction, limit_price).unwrap();

        assert_eq!(auction_start_price, 100000000);
        assert_eq!(auction_end_price, 100500000);
    }

    #[test]
    fn limit_price_slightly_worse_than_oracle_long() {
        let oracle_price_data = OraclePriceData {
            price: 100 * PRICE_PRECISION_I64,
            ..OraclePriceData::default()
        };
        let position_direction = PositionDirection::Long;
        let limit_price = 100400000;

        let (auction_start_price, auction_end_price) =
            calculate_auction_prices(&oracle_price_data, position_direction, limit_price).unwrap();

        assert_eq!(auction_start_price, 100000000);
        assert_eq!(auction_end_price, 100400000);
    }

    #[test]
    fn limit_price_much_better_than_oracle_short() {
        let oracle_price_data = OraclePriceData {
            price: 100 * PRICE_PRECISION_I64,
            ..OraclePriceData::default()
        };
        let position_direction = PositionDirection::Short;
        let limit_price = 110000000;

        let (auction_start_price, auction_end_price) =
            calculate_auction_prices(&oracle_price_data, position_direction, limit_price).unwrap();

        assert_eq!(auction_start_price, 110550000);
        assert_eq!(auction_end_price, 110000000);
    }

    #[test]
    fn limit_price_slightly_better_than_oracle_short() {
        let oracle_price_data = OraclePriceData {
            price: 100 * PRICE_PRECISION_I64,
            ..OraclePriceData::default()
        };
        let position_direction = PositionDirection::Short;
        let limit_price = 100000001;

        let (auction_start_price, auction_end_price) =
            calculate_auction_prices(&oracle_price_data, position_direction, limit_price).unwrap();

        assert_eq!(auction_start_price, 100500001);
        assert_eq!(auction_end_price, 100000001);
    }

    #[test]
    fn limit_price_much_worse_than_oracle_short() {
        let oracle_price_data = OraclePriceData {
            price: 100 * PRICE_PRECISION_I64,
            ..OraclePriceData::default()
        };
        let position_direction = PositionDirection::Short;
        let limit_price = 90000000;

        let (auction_start_price, auction_end_price) =
            calculate_auction_prices(&oracle_price_data, position_direction, limit_price).unwrap();

        assert_eq!(auction_start_price, 100000000);
        assert_eq!(auction_end_price, 99500000);
    }

    #[test]
    fn limit_price_slightly_worse_than_oracle_short() {
        let oracle_price_data = OraclePriceData {
            price: 100 * PRICE_PRECISION_I64,
            ..OraclePriceData::default()
        };
        let position_direction = PositionDirection::Short;
        let limit_price = 99999999;

        let (auction_start_price, auction_end_price) =
            calculate_auction_prices(&oracle_price_data, position_direction, limit_price).unwrap();

        assert_eq!(auction_start_price, 100000000);
        assert_eq!(auction_end_price, 99999999);
    }
}

mod calculate_auction_price {
    use crate::math::auction::calculate_auction_price;
    use crate::math::constants::{PRICE_PRECISION_I64, PRICE_PRECISION_U64};
    use crate::state::user::{Order, OrderType};

    #[test]
    fn long_oracle_order() {
        let tick_size = 1;

        // auction starts $.10 below oracle and ends $.1 above oracle
        let order = Order {
            order_type: OrderType::Oracle,
            auction_duration: 10,
            slot: 0,
            auction_start_price: -PRICE_PRECISION_I64 / 10,
            auction_end_price: PRICE_PRECISION_I64 / 10,
            ..Order::default()
        };
        let oracle_price = Some(PRICE_PRECISION_I64);

        let slot = 0;
        let price = calculate_auction_price(&order, slot, tick_size, oracle_price).unwrap();

        assert_eq!(price, 9 * PRICE_PRECISION_U64 / 10);

        let slot = 5;
        let price = calculate_auction_price(&order, slot, tick_size, oracle_price).unwrap();

        assert_eq!(price, PRICE_PRECISION_U64);

        let slot = 10;
        let price = calculate_auction_price(&order, slot, tick_size, oracle_price).unwrap();

        assert_eq!(price, 11 * PRICE_PRECISION_U64 / 10);

        // auction starts $.20 below oracle and ends $.1 below oracle
        let order = Order {
            order_type: OrderType::Oracle,
            auction_duration: 10,
            slot: 0,
            auction_start_price: -PRICE_PRECISION_I64 / 5,
            auction_end_price: -PRICE_PRECISION_I64 / 10,
            ..Order::default()
        };

        let slot = 0;
        let price = calculate_auction_price(&order, slot, tick_size, oracle_price).unwrap();

        assert_eq!(price, 8 * PRICE_PRECISION_U64 / 10);

        let slot = 5;
        let price = calculate_auction_price(&order, slot, tick_size, oracle_price).unwrap();

        assert_eq!(price, 85 * PRICE_PRECISION_U64 / 100);

        let slot = 10;
        let price = calculate_auction_price(&order, slot, tick_size, oracle_price).unwrap();

        assert_eq!(price, 9 * PRICE_PRECISION_U64 / 10);

        // auction starts $.10 above oracle and ends $.2 above oracle
        let order = Order {
            order_type: OrderType::Oracle,
            auction_duration: 10,
            slot: 0,
            auction_start_price: PRICE_PRECISION_I64 / 10,
            auction_end_price: PRICE_PRECISION_I64 / 5,
            ..Order::default()
        };

        let slot = 0;
        let price = calculate_auction_price(&order, slot, tick_size, oracle_price).unwrap();

        assert_eq!(price, 11 * PRICE_PRECISION_U64 / 10);

        let slot = 5;
        let price = calculate_auction_price(&order, slot, tick_size, oracle_price).unwrap();

        assert_eq!(price, 115 * PRICE_PRECISION_U64 / 100);

        let slot = 10;
        let price = calculate_auction_price(&order, slot, tick_size, oracle_price).unwrap();

        assert_eq!(price, 12 * PRICE_PRECISION_U64 / 10);
    }

    #[test]
    fn short_oracle_order() {
        let tick_size = 1;
        // auction starts $.10 above oracle and ends $.1 below oracle
        let order = Order {
            order_type: OrderType::Oracle,
            auction_duration: 10,
            slot: 0,
            auction_start_price: PRICE_PRECISION_I64 / 10,
            auction_end_price: -PRICE_PRECISION_I64 / 10,
            ..Order::default()
        };
        let oracle_price = Some(PRICE_PRECISION_I64);

        let slot = 0;
        let price = calculate_auction_price(&order, slot, tick_size, oracle_price).unwrap();

        assert_eq!(price, 11 * PRICE_PRECISION_U64 / 10);

        let slot = 5;
        let price = calculate_auction_price(&order, slot, tick_size, oracle_price).unwrap();

        assert_eq!(price, PRICE_PRECISION_U64);

        let slot = 10;
        let price = calculate_auction_price(&order, slot, tick_size, oracle_price).unwrap();

        assert_eq!(price, 9 * PRICE_PRECISION_U64 / 10);

        // auction starts $.20 above oracle and ends $.1 above oracle
        let order = Order {
            order_type: OrderType::Oracle,
            auction_duration: 10,
            slot: 0,
            auction_start_price: PRICE_PRECISION_I64 / 5,
            auction_end_price: PRICE_PRECISION_I64 / 10,
            ..Order::default()
        };

        let slot = 0;
        let price = calculate_auction_price(&order, slot, tick_size, oracle_price).unwrap();

        assert_eq!(price, 12 * PRICE_PRECISION_U64 / 10);

        let slot = 5;
        let price = calculate_auction_price(&order, slot, tick_size, oracle_price).unwrap();

        assert_eq!(price, 115 * PRICE_PRECISION_U64 / 100);

        let slot = 10;
        let price = calculate_auction_price(&order, slot, tick_size, oracle_price).unwrap();

        assert_eq!(price, 11 * PRICE_PRECISION_U64 / 10);

        // auction starts $.10 below oracle and ends $.2 below oracle
        let order = Order {
            order_type: OrderType::Oracle,
            auction_duration: 10,
            slot: 0,
            auction_start_price: -PRICE_PRECISION_I64 / 10,
            auction_end_price: -PRICE_PRECISION_I64 / 5,
            ..Order::default()
        };

        let slot = 0;
        let price = calculate_auction_price(&order, slot, tick_size, oracle_price).unwrap();

        assert_eq!(price, 9 * PRICE_PRECISION_U64 / 10);

        let slot = 5;
        let price = calculate_auction_price(&order, slot, tick_size, oracle_price).unwrap();

        assert_eq!(price, 85 * PRICE_PRECISION_U64 / 100);

        let slot = 10;
        let price = calculate_auction_price(&order, slot, tick_size, oracle_price).unwrap();

        assert_eq!(price, 8 * PRICE_PRECISION_U64 / 10);
    }
}
