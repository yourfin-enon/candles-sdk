use crate::models::{candle::BidAskCandle, candle_data::CandleData, candle_type::CandleType};
use ahash::AHashMap;
use chrono::{DateTime, Utc};
use compact_str::{ToCompactString};

pub struct CandlesCache {
    candles_by_ids: AHashMap<String, BidAskCandle>,
    pub candle_types: Vec<CandleType>,
    pub last_update_date: Option<DateTime<Utc>>,
}

impl CandlesCache {
    pub fn new(candle_types: Vec<CandleType>) -> Self {
        let mut candle_types = candle_types;
        candle_types.dedup();
        candle_types.sort();

        Self {
            candles_by_ids: AHashMap::new(),
            candle_types,
            last_update_date: None,
        }
    }

    pub fn get_all(&self) -> &AHashMap<String, BidAskCandle> {
        &self.candles_by_ids
    }

    pub fn len(&self) -> usize {
        self.candles_by_ids.len()
    }

    pub fn contains(&self, candle_id: &str) -> bool {
        self.candles_by_ids.contains_key(candle_id)
    }

    pub fn insert(&mut self, candle: BidAskCandle) {
        #[cfg(feature = "console-log")]
        println!(
            "insert candle {}: {} {}; {} total count",
            candle.instrument,
            candle.datetime.to_rfc3339(),
            candle.get_id(),
            self.candles_by_ids.len() + 1
        );

        self.candles_by_ids.insert(candle.get_id(), candle);
    }

    pub fn create_or_update(
        &mut self,
        datetime: DateTime<Utc>,
        instrument: &str,
        bid: f64,
        ask: f64,
        bid_vol: f64,
        ask_vol: f64,
    ) {
        for candle_type in self.candle_types.iter() {
            let candle_datetime = candle_type.get_start_date(datetime);
            let id = BidAskCandle::generate_id(instrument, candle_type, candle_datetime);
            let candle = self.candles_by_ids.get_mut(&id);

            if let Some(candle) = candle {
                candle.update(datetime, bid, ask, bid_vol, ask_vol);
            } else {
                #[cfg(feature = "console-log")]
                println!(
                    "create candle {}: {} {}; {} total count",
                    instrument.to_owned(),
                    datetime.to_rfc3339(),
                    id,
                    self.candles_by_ids.len() + 1
                );

                self.candles_by_ids.insert(
                    id,
                    BidAskCandle {
                        ask_data: CandleData::new(datetime, ask, ask_vol),
                        bid_data: CandleData::new(datetime, bid, bid_vol),
                        candle_type: candle_type.clone(),
                        instrument: instrument.to_compact_string(),
                        datetime: candle_datetime,
                    },
                );
            }
        }
        
        self.last_update_date.replace(Utc::now());
    }

    /// Gets candles with date bigger or equals specified date
    pub fn get_after(&self, datetime: DateTime<Utc>) -> Option<Vec<&BidAskCandle>> {
        if self.candles_by_ids.len() == 0 {
            return None;
        }

        let candle_dates = self.calculate_candle_dates(datetime);

        let candles = self
            .candles_by_ids
            .iter()
            .filter_map(|(_id, candle)| {
                let current_date = candle_dates.get(&candle.candle_type).expect("wrong calculate_candle_dates");

                if candle.datetime >= *current_date {
                    Some(candle)
                } else {
                    None
                }
            })
            .collect();

        Some(candles)
    }

    /// Removes candles with date less or equals specified date
    pub fn remove_before(&mut self, datetime: DateTime<Utc>, candle_type: Option<CandleType>) -> i32 {
        let mut removed_count = 0;

        if let Some(candle_type) = candle_type {
            self.candles_by_ids.retain(|_id, candle| {
                let current_date = candle_type.get_start_date(datetime);

                if candle.datetime <= current_date && candle.candle_type == candle_type {
                    removed_count += 1;
                    false
                } else {
                    true
                }
            });
        } else {
            let dates = self.calculate_candle_dates(datetime);

            self.candles_by_ids.retain(|_id, candle| {
                let current_date = dates.get(&candle.candle_type).expect("Wrong calculate_candle_dates");

                if candle.datetime <= *current_date {
                    removed_count += 1;
                    false
                } else {
                    true
                }
            });
        }

        removed_count
    }

    pub fn get(&self, id: &str) -> Option<&BidAskCandle> {
        self.candles_by_ids.get(id)
    }

    fn calculate_candle_dates(&self, datetime: DateTime<Utc>) -> AHashMap<CandleType, DateTime<Utc>> {
        let mut dates = AHashMap::with_capacity(self.candle_types.len());

        for candle_type in self.candle_types.iter() {
            dates.insert(candle_type.to_owned(), candle_type.get_start_date(datetime));
        }

        dates
    }
}

#[cfg(test)]
mod tests {
    use crate::models::candle_type::CandleType;
    use chrono::{DateTime, TimeZone, Utc};
    use crate::caches::candles_cache::CandlesCache;

    #[tokio::test]
    async fn calculate_candle_dates() {
        let candle_types = [
            CandleType::Minute,
            CandleType::ThreeMinutes,
            CandleType::FiveMinutes,
            CandleType::FifteenMinutes,
            CandleType::ThirtyMinutes,
            CandleType::Hour,
            CandleType::TwoHours,
            CandleType::FourHours,
            CandleType::SixHours,
            CandleType::EightHours,
            CandleType::TwelveHours,
            CandleType::Day,
            CandleType::ThreeDays,
            CandleType::SevenDays,
            CandleType::Month,
        ];
        let cache = CandlesCache::new(candle_types.to_vec());
        let initial_date: DateTime<Utc> = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
        let dates = cache.calculate_candle_dates(initial_date);

        assert_eq!(candle_types.len(), dates.len());

        for candle_type in candle_types.iter() {
            let date = dates.get(&candle_type);
            assert_eq!(date, Some(&candle_type.get_start_date(initial_date)))
        }
    }
}
