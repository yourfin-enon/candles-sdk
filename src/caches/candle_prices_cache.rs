use std::{collections::{BTreeMap}};
use chrono::{DateTime, Utc};
use crate::models::{candle_type::CandleType, candle_data::CandleData};

#[derive(Debug, Clone)]
pub struct CandlePricesCache{
    pub candle_type: CandleType,
    pub prices_by_date: BTreeMap<i64, CandleData>
}

impl CandlePricesCache {
    pub fn new(candle_type: CandleType) -> Self{
        Self { candle_type, prices_by_date: BTreeMap::new() }
    }

    pub fn init(&mut self, candle: CandleData){
        self.prices_by_date.insert(candle.datetime.timestamp(), candle);
    }

    pub fn update(&mut self, datetime: DateTime<Utc>, rate: f64, volume: f64){
        let candle_date = self.candle_type.get_start_date(datetime);
        let timestamp_sec = candle_date.timestamp();
        let target_candle = self.prices_by_date.get_mut(&timestamp_sec);

        match target_candle {
            Some(candle) => candle.update(datetime, rate, volume),
            None => {
                let candle_model = CandleData::new(candle_date, rate, volume);
                self.prices_by_date.insert(timestamp_sec, candle_model);
            },
        }
    }

    pub fn get_by_date_range(&self, date_from: DateTime<Utc>, date_to: DateTime<Utc>) -> Vec<CandleData>{
        let mut result = Vec::new();
        let timestamp_from = date_from.timestamp();
        let timestamp_to = date_to.timestamp();

        for (_date, candle) in self.prices_by_date.range(timestamp_from..timestamp_to){
            result.push(candle.clone());
        }

        result
    }

    pub fn clear(&mut self) {
        self.prices_by_date.clear()
    }
}