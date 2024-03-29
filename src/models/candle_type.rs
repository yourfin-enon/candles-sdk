use std::collections::HashSet;

use chrono::{DateTime, Datelike, Utc};
use chrono::{Duration, TimeZone};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(
    Serialize_repr,
    Deserialize_repr,
    Debug,
    Clone,
    IntoPrimitive,
    TryFromPrimitive,
    Hash,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
)]
#[repr(i32)]
pub enum CandleType {
    Minute = 0,
    Hour = 1,
    Day = 2,
    Month = 3,
    ThreeMinutes = 4,
    FiveMinutes = 5,
    FifteenMinutes = 6,
    ThirtyMinutes = 7,
    TwoHours = 8,
    FourHours = 9,
    SixHours = 10,
    EightHours = 11,
    TwelveHours = 12,
    ThreeDays = 13,
    SevenDays = 14,
}

impl CandleType {
    pub fn get_start_date(&self, datetime: DateTime<Utc>) -> DateTime<Utc> {
        let timestamp_sec = datetime.timestamp();

        match self {
            CandleType::Minute => Utc
                .timestamp_millis_opt((timestamp_sec - timestamp_sec % 60) * 1000)
                .unwrap(),
            CandleType::Hour => Utc
                .timestamp_millis_opt((timestamp_sec - timestamp_sec % 3600) * 1000)
                .unwrap(),
            CandleType::Day => Utc
                .timestamp_millis_opt((timestamp_sec - timestamp_sec % 86400) * 1000)
                .unwrap(),
            CandleType::Month => {
                let date = Utc.timestamp_millis_opt(timestamp_sec * 1000).unwrap();
                let start_of_month: DateTime<Utc> = Utc
                    .with_ymd_and_hms(date.year(), date.month(), 1, 0, 0, 0)
                    .unwrap();

                start_of_month
            }
            CandleType::ThreeMinutes => Utc
                .timestamp_millis_opt((timestamp_sec - timestamp_sec % 180) * 1000)
                .unwrap(),
            CandleType::FiveMinutes => Utc
                .timestamp_millis_opt((timestamp_sec - timestamp_sec % 300) * 1000)
                .unwrap(),
            CandleType::FifteenMinutes => Utc
                .timestamp_millis_opt((timestamp_sec - timestamp_sec % 900) * 1000)
                .unwrap(),
            CandleType::ThirtyMinutes => Utc
                .timestamp_millis_opt((timestamp_sec - timestamp_sec % 1800) * 1000)
                .unwrap(),
            CandleType::TwoHours => Utc
                .timestamp_millis_opt((timestamp_sec - timestamp_sec % 7200) * 1000)
                .unwrap(),
            CandleType::FourHours => Utc
                .timestamp_millis_opt((timestamp_sec - timestamp_sec % 14400) * 1000)
                .unwrap(),
            CandleType::SixHours => Utc
                .timestamp_millis_opt((timestamp_sec - timestamp_sec % 21600) * 1000)
                .unwrap(),
            CandleType::EightHours => Utc
                .timestamp_millis_opt((timestamp_sec - timestamp_sec % 28800) * 1000)
                .unwrap(),
            CandleType::TwelveHours => Utc
                .timestamp_millis_opt((timestamp_sec - timestamp_sec % 43200) * 1000)
                .unwrap(),
            CandleType::ThreeDays => Utc
                .timestamp_millis_opt((timestamp_sec - timestamp_sec % 604800) * 1000)
                .unwrap(),
            CandleType::SevenDays => Utc
                .timestamp_millis_opt((timestamp_sec - timestamp_sec % 1036800) * 1000)
                .unwrap(),
        }
    }

    pub fn get_start_dates(
        &self,
        datetime_from: DateTime<Utc>,
        datetime_to: DateTime<Utc>,
    ) -> HashSet<DateTime<Utc>> {
        let mut dates = HashSet::new();
        let date_from = self.get_start_date(datetime_from);
        dates.insert(date_from);
        let date_to = self.get_start_date(datetime_to);

        let mut last_date = self.get_start_date(date_from);

        while last_date < date_to {
            let next_date = self.get_start_date(last_date) + self.get_duration(last_date);
            last_date = self.get_start_date(next_date);
            dates.insert(last_date);
        }

        dates
    }


    pub fn get_end_date(
        &self,
        datetime: DateTime<Utc>
    ) -> DateTime<Utc> {
        let start = self.get_start_date(datetime);
        let duration = self.get_duration(datetime);

        start + duration
    }

    pub fn get_dates_count(&self, datetime_from: DateTime<Utc>, datetime_to: DateTime<Utc>) -> usize {
        let from = self.get_start_date(datetime_from);
        let to = self.get_end_date(datetime_to);

        match self {
            CandleType::Month =>  {
                let year_diff = to.year() - from.year();
                let month_diff = to.month() - from.month();
                let total_month_diff = year_diff * 12 + month_diff as i32;

                total_month_diff as usize
            },
            CandleType::Minute => {
                let duration = to.signed_duration_since(from);
                let minute_count = duration.num_minutes();

                minute_count as usize
            },
            _ => {
                let duration = self.get_duration(datetime_from);
                let duration_between = to - from;
                let count = duration_between.num_seconds() / duration.num_seconds();

                count as usize
            }

        }
    }

    pub fn get_duration(&self, datetime: DateTime<Utc>) -> Duration {
        let duration = match self {
            CandleType::Minute => Duration::seconds(60),
            CandleType::Hour => Duration::seconds(3600),
            CandleType::Day => Duration::seconds(86400),
            CandleType::Month => {
                let start_of_month: DateTime<Utc> = Utc
                    .with_ymd_and_hms(datetime.year(), datetime.month(), 1, 0, 0, 0)
                    .unwrap();
                let next_month = if datetime.month() == 12 {
                    1
                } else {
                    datetime.month() + 1
                };

                let next_year = if datetime.month() == 12 {
                    datetime.year() + 1
                } else {
                    datetime.year()
                };

                let end_of_month: DateTime<Utc> = Utc
                    .with_ymd_and_hms(next_year, next_month, 1, 0, 0, 0)
                    .unwrap();

                end_of_month - start_of_month
            }
            CandleType::ThreeMinutes => Duration::minutes(3),
            CandleType::FiveMinutes => Duration::minutes(5),
            CandleType::FifteenMinutes => Duration::minutes(15),
            CandleType::ThirtyMinutes => Duration::minutes(30),
            CandleType::TwoHours => Duration::hours(2),
            CandleType::FourHours => Duration::hours(4),
            CandleType::SixHours => Duration::hours(6),
            CandleType::EightHours => Duration::hours(8),
            CandleType::TwelveHours => Duration::hours(12),
            CandleType::ThreeDays => Duration::days(3),
            CandleType::SevenDays => Duration::days(7),
        };

        duration
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::models::candle_type::CandleType;
    use chrono::{DateTime, Datelike, Duration, TimeZone, Timelike, Utc};

    #[tokio::test]
    async fn count_minute() {
        let candle_type = CandleType::Minute;
        let duration = Duration::minutes(15);
        let from: DateTime<Utc> = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
        let to: DateTime<Utc> = from + duration - Duration::seconds(1);

        let count = candle_type.get_dates_count(from, to);

        assert_eq!(count, duration.num_minutes() as usize);
    }

    #[tokio::test]
    async fn count_hour() {
        let candle_type = CandleType::Hour;
        let duration = Duration::hours(15);
        let from: DateTime<Utc> = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
        let to: DateTime<Utc> = from + duration - Duration::seconds(1);

        let count = candle_type.get_dates_count(from, to);

        assert_eq!(count, duration.num_hours() as usize);
    }

    #[tokio::test]
    async fn count_day() {
        let candle_type = CandleType::Day;
        let duration = Duration::days(15);
        let from: DateTime<Utc> = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
        let to: DateTime<Utc> = from + duration - Duration::seconds(1);

        let count = candle_type.get_dates_count(from, to);

        assert_eq!(count, duration.num_days() as usize);
    }

    
    #[tokio::test]
    #[ignore]
    async fn count_month() {
        let candle_type = CandleType::Month;
        let num_months = 12;
        let from: DateTime<Utc> = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
        let to: DateTime<Utc> = Utc.with_ymd_and_hms(2000, num_months, 1, 0, 0, 0).unwrap();

        let count = candle_type.get_dates_count(from, to);

        assert_eq!(count, num_months as usize);
    }

    #[tokio::test]
    async fn get_date_for_minute() {
        let candle_type = CandleType::Minute;
        let src_date: DateTime<Utc> = Utc.with_ymd_and_hms(2000, 1, 1, 1, 1, 34).unwrap();

        let start_date = candle_type.get_start_date(src_date);

        assert_eq!(start_date.year(), src_date.year());
        assert_eq!(start_date.month(), src_date.month());
        assert_eq!(start_date.day(), src_date.day());
        assert_eq!(start_date.hour(), src_date.hour());
        assert_eq!(start_date.hour(), src_date.hour());
        assert_eq!(start_date.minute(), src_date.minute());
        assert_eq!(start_date.second(), 0);
    }

    #[tokio::test]
    async fn get_date_for_hour() {
        let candle_type = CandleType::Hour;
        let src_date: DateTime<Utc> = Utc.with_ymd_and_hms(2000, 1, 1, 1, 23, 34).unwrap();

        let start_date = candle_type.get_start_date(src_date);

        assert_eq!(start_date.year(), src_date.year());
        assert_eq!(start_date.month(), src_date.month());
        assert_eq!(start_date.day(), src_date.day());
        assert_eq!(start_date.hour(), src_date.hour());
        assert_eq!(start_date.minute(), 0);
        assert_eq!(start_date.second(), 0);
    }

    #[tokio::test]
    async fn get_date_for_day() {
        let candle_type = CandleType::Day;
        let src_date: DateTime<Utc> = Utc.with_ymd_and_hms(2000, 1, 1, 3, 23, 34).unwrap();

        let start_date = candle_type.get_start_date(src_date);

        assert_eq!(start_date.year(), src_date.year());
        assert_eq!(start_date.month(), src_date.month());
        assert_eq!(start_date.day(), src_date.day());
        assert_eq!(start_date.hour(), 0);
        assert_eq!(start_date.minute(), 0);
        assert_eq!(start_date.second(), 0);
    }

    #[tokio::test]
    async fn get_start_date_for_month() {
        let candle_type = CandleType::Month;
        let src_date: DateTime<Utc> = Utc.with_ymd_and_hms(2000, 1, 12, 3, 23, 34).unwrap();

        let start_date = candle_type.get_start_date(src_date);

        assert_eq!(start_date.year(), src_date.year());
        assert_eq!(start_date.month(), src_date.month());
        assert_eq!(start_date.day(), 1);
        assert_eq!(start_date.hour(), 0);
        assert_eq!(start_date.minute(), 0);
        assert_eq!(start_date.second(), 0);
    }

    #[tokio::test]
    async fn get_end_date_for_month() {
        let candle_type = CandleType::Month;
        let src_date: DateTime<Utc> = Utc.with_ymd_and_hms(2000, 12, 12, 3, 23, 34).unwrap();

        let end_date = candle_type.get_end_date(src_date);

        assert_eq!(end_date.year(), src_date.year() + 1);
        assert_eq!(end_date.month(), 1);
        assert_eq!(end_date.day(), 1);
        assert_eq!(end_date.hour(), 0);
        assert_eq!(end_date.minute(), 0);
        assert_eq!(end_date.second(), 0);
    }

    #[tokio::test]
    async fn get_start_dates_for_minute() {
        let duration = Duration::minutes(15);
        let from: DateTime<Utc> = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
        let to: DateTime<Utc> = from + duration;
        let candle_type = CandleType::Minute;

        let dates = candle_type.get_start_dates(from, to);
        let dates: HashSet<DateTime<Utc>> = dates.into_iter().collect();

        assert!(dates.contains(&candle_type.get_start_date(from)));
        assert!(dates.contains(&candle_type.get_start_date(to)));
        assert_eq!(dates.len(), (duration.num_minutes() + 1) as usize);

        for _ in 0..duration.num_minutes() {
            let date = candle_type.get_start_date(from) + candle_type.get_duration(from);
            assert!(dates.contains(&date));
        }
    }

    #[tokio::test]
    async fn get_start_dates_for_hour() {
        let duration = Duration::hours(15);
        let from: DateTime<Utc> = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
        let to: DateTime<Utc> = from + duration;
        let candle_type = CandleType::Hour;

        let dates = candle_type.get_start_dates(from, to);
        let dates: HashSet<DateTime<Utc>> = dates.into_iter().collect();

        assert!(dates.contains(&candle_type.get_start_date(from)));
        assert!(dates.contains(&candle_type.get_start_date(to)));
        assert_eq!(dates.len(), (duration.num_hours() + 1) as usize);

        for _ in 0..duration.num_hours() {
            let date = candle_type.get_start_date(from) + candle_type.get_duration(from);
            assert!(dates.contains(&date));
        }
    }

    #[tokio::test]
    async fn get_start_dates_for_day() {
        let duration = Duration::days(15);
        let from: DateTime<Utc> = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
        let to: DateTime<Utc> = from + duration;
        let candle_type = CandleType::Day;

        let dates = candle_type.get_start_dates(from, to);
        let dates: HashSet<DateTime<Utc>> = dates.into_iter().collect();

        assert!(dates.contains(&candle_type.get_start_date(from)));
        assert!(dates.contains(&candle_type.get_start_date(to)));
        assert_eq!(dates.len(), (duration.num_days() + 1) as usize);

        for _ in 0..duration.num_days() {
            let date = candle_type.get_start_date(from) + candle_type.get_duration(from);
            assert!(dates.contains(&date));
        }
    }

    #[tokio::test]
    async fn get_start_dates_for_month() {
        let num_months = 12;
        let from: DateTime<Utc> = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
        let to: DateTime<Utc> = Utc.with_ymd_and_hms(2000, num_months, 1, 0, 0, 0).unwrap();
        let candle_type = CandleType::Month;

        let dates = candle_type.get_start_dates(from, to);
        let dates: HashSet<DateTime<Utc>> = dates.into_iter().collect();

        assert!(dates.contains(&candle_type.get_start_date(from)));
        assert!(dates.contains(&candle_type.get_start_date(to)));
        assert_eq!(dates.len(), num_months as usize);

        for _ in 0..num_months {
            let date = candle_type.get_start_date(from) + candle_type.get_duration(from);
            assert!(dates.contains(&date));
        }
    }
}
