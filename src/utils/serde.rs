/// Custom serialization and deserialization config for [DateTime<Utc>] to
/// string having the format `%Y-%m-%dT%H:%M:%SZ`.
///
/// ```
/// use chrono::{DateTime, Utc};
/// use crate::utils::serde::json_date_format;
/// use serde::{ Deserialize, Serialize };
///
/// #[derive(Serialize, Deserialize)]
/// struct Data {
///     #[serde(with = "json_date_format")]
///     date: DateTime<Utc>
/// }
/// ```
pub mod json_date_format {
    use chrono::{DateTime, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &'static str = "%Y-%m-%dT%H:%M:%SZ";

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DateTime::parse_from_rfc3339(&s)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(serde::de::Error::custom)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct DateHolder {
            #[serde(with = "super")]
            date: DateTime<Utc>,
        }

        #[test]
        fn accepts_aws_utc_timestamps() {
            let date: DateHolder =
                serde_json::from_str(r#"{"date":"2026-07-28T15:19:32Z"}"#).unwrap();

            assert_eq!(date.date.to_rfc3339(), "2026-07-28T15:19:32+00:00");
        }

        #[test]
        fn accepts_aws_timestamps_with_fractional_seconds() {
            let date: DateHolder =
                serde_json::from_str(r#"{"date":"2026-07-28T15:19:32.000Z"}"#).unwrap();

            assert_eq!(date.date.to_rfc3339(), "2026-07-28T15:19:32+00:00");
        }
    }
}
