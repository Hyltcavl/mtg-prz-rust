use log::warn;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorNumber {
    raw_value: String,
    cleaned_value: String,
}

impl CollectorNumber {
    pub fn new(collector_number: &str) -> Result<Self, String> {
        let c_num = collector_number.trim();

        if Self::is_only_digits(c_num)
            || Self::is_dash_separated_string(c_num)
            || Self::is_underscore_and_dash_separated_string(c_num)
        {
            Ok(Self {
                raw_value: c_num.to_string(),
                cleaned_value: c_num.to_lowercase(),
            })
        } else {
            warn!("{} is not a valid collector number", collector_number);
            Err("Collector number must be 2 to 8 digits long, or 12 characters long with a dash between the first and last character".to_string())
        }
    }

    fn is_underscore_and_dash_separated_string(num: &str) -> bool {
        num.len() >= 4
            && num.len() <= 12
            && num.contains('-')
            && num
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    }

    fn is_only_digits(num: &str) -> bool {
        num.len() >= 2 && num.len() <= 8 && num.chars().all(|c| c.is_digit(10))
    }

    fn is_dash_separated_string(num: &str) -> bool {
        num.len() >= 4
            && num.len() <= 12
            && num.contains('-')
            && num.chars().all(|c| c.is_alphanumeric() || c == '-')
    }
}

impl PartialEq for CollectorNumber {
    fn eq(&self, other: &Self) -> bool {
        self.cleaned_value == other.cleaned_value
    }
}

// impl PartialOrd for CollectorNumber {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         self.cleaned_value.partial_cmp(&other.cleaned_value)
//     }
// }
