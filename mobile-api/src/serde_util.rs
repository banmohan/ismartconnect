use std::str::FromStr;

use chrono::{DateTime, NaiveDateTime, SecondsFormat, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serializer};
use serde::de;
use serde_json::Value;

pub mod dec {
    use super::*;

    pub fn serialize<S: Serializer>(value: &Decimal, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::Serialize;
        let raw = serde_json::value::RawValue::from_string(value.to_string()).map_err(serde::ser::Error::custom)?;
        raw.serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Decimal, D::Error> {
        let value = Value::deserialize(deserializer)?;
        parse_required(&value).map_err(de::Error::custom)
    }
}

pub mod dec_loose {
    use super::*;

    pub fn serialize<S: Serializer>(value: &Decimal, serializer: S) -> Result<S::Ok, S::Error> {
        dec::serialize(value, serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Decimal, D::Error> {
        let value = Option::<Value>::deserialize(deserializer)?;
        match value {
            None | Some(Value::Null) => Ok(Decimal::ZERO),
            Some(value) => parse_required(&value).map_err(de::Error::custom),
        }
    }
}

pub mod dec_opt {
    use super::*;

    pub fn serialize<S: Serializer>(value: &Option<Decimal>, serializer: S) -> Result<S::Ok, S::Error> {
        match value {
            Some(value) => dec::serialize(value, serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Decimal>, D::Error> {
        let value = Option::<Value>::deserialize(deserializer)?;
        match value {
            None | Some(Value::Null) => Ok(None),
            Some(value) => parse_required(&value).map(Some).map_err(de::Error::custom),
        }
    }
}

fn parse_required(value: &Value) -> Result<Decimal, String> {
    match value {
        Value::Number(number) => Decimal::from_str(&number.to_string()).map_err(|err| err.to_string()),
        Value::String(text) => Decimal::from_str(text).map_err(|err| err.to_string()),
        other => Err(format!("expected a decimal, got {other}")),
    }
}

/// Accepts the date-time shapes System.Text.Json and PremiumCBS both emit,
/// and stores an RFC3339 UTC string so the response stays a JSON string.
pub fn flex_datetime<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let value = Value::deserialize(deserializer)?;
    let Value::String(text) = value else {
        return Err(de::Error::custom("expected a date-time string"));
    };
    let parsed = parse_datetime(&text).ok_or_else(|| de::Error::custom(format!("invalid date-time {text}")))?;
    Ok(parsed.to_rfc3339_opts(SecondsFormat::Secs, true))
}

pub fn parse_datetime(text: &str) -> Option<DateTime<Utc>> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    if let Ok(parsed) = DateTime::parse_from_rfc3339(text) {
        return Some(parsed.with_timezone(&Utc));
    }
    let naive_formats = ["%Y-%m-%dT%H:%M:%S%.f", "%Y-%m-%dT%H:%M:%S", "%Y-%m-%d %H:%M:%S", "%Y-%m-%dT%H:%M"];
    for format in naive_formats {
        if let Ok(naive) = NaiveDateTime::parse_from_str(text, format) {
            return Some(naive.and_utc());
        }
    }
    if let Ok(date) = chrono::NaiveDate::parse_from_str(text, "%Y-%m-%d") {
        return Some(date.and_hms_opt(0, 0, 0)?.and_utc());
    }
    None
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rust_decimal::Decimal;
    use serde::Serialize;

    #[test]
    fn decimal_serializes_as_json_number() {
        #[derive(Serialize)]
        struct Sample {
            #[serde(with = "super::dec")]
            amount: Decimal,
        }
        let json = serde_json::to_string(&Sample {
            amount: Decimal::from_str("10.50").unwrap(),
        })
        .unwrap();
        assert_eq!(json, r#"{"amount":10.50}"#);
    }
}
