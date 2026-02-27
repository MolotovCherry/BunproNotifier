use std::{collections::HashMap, sync::LazyLock};

use jiff::{Timestamp, civil::Date, tz::TimeZone};
use serde::{Deserialize, Deserializer};

static CURRENT_TZ: LazyLock<TimeZone> = LazyLock::new(TimeZone::system);

pub type CardCount = u32;

#[derive(Debug, Copy, Clone, Deserialize)]
pub struct TotalDue {
    pub total_due_grammar: CardCount,
    pub total_due_vocab: CardCount,
}

#[expect(unused)]
#[derive(Debug, Deserialize)]
pub struct ForecastDaily {
    pub grammar: ForecastDailyObject,
    pub vocab: ForecastDailyObject,
}

#[derive(Debug, Deserialize)]
pub struct ForecastHourly {
    pub grammar: ForecastHourlyObject,
    pub vocab: ForecastHourlyObject,
}

#[derive(Debug, Deserialize)]
pub struct ForecastHourlyObject {
    #[serde(flatten)]
    pub rest: HashMap<Zoned, CardCount>,
}

#[expect(unused)]
#[derive(Debug, Deserialize)]
pub struct ForecastDailyObject {
    pub later: CardCount,
    pub tomorrow: CardCount,
    #[serde(flatten)]
    pub rest: HashMap<Date, CardCount>,
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Deserialize, Hash)]
#[serde(transparent)]
pub struct Zoned(#[serde(deserialize_with = "timestamp_to_zoned")] pub jiff::Zoned);

fn timestamp_to_zoned<'de, D>(de: D) -> Result<jiff::Zoned, D::Error>
where
    D: Deserializer<'de>,
{
    let ts = Timestamp::deserialize(de)?.to_zoned(CURRENT_TZ.clone());

    Ok(ts)
}
