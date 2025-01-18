use std::fmt::{Display, Formatter};

#[derive(serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Currency {
    USD,
    EUR,
    XRP,
    BITCOIN,
}

impl Display for Currency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Currency::USD => { write!(f, "USD") }
            Currency::EUR => { write!(f, "EUR") }
            Currency::XRP => { write!(f, "XRP") }
            Currency::BITCOIN => { write!(f, "BITCOIN") }
        }
    }
}

impl Currency {
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "EUR" => Some(Currency::EUR),
            "USD" => Some(Currency::USD),
            "BITCOIN" => Some(Currency::BITCOIN),
            "XRP" => Some(Currency::XRP),
            _ => None
        }
    }

    pub fn is_crypto(&self) -> bool {
        matches!(self, Currency::XRP) || matches!(self, Currency::BITCOIN)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_string_valid_values() {
        let test_cases = vec![
            ("USD", Some(Currency::USD)),
            ("usd", Some(Currency::USD)),
            ("EUR", Some(Currency::EUR)),
            ("eur", Some(Currency::EUR)),
            ("UsD", Some(Currency::USD)),
        ];

        for (input, expected) in test_cases {
            assert_eq!(Currency::from_string(input), expected);
        }
    }

    #[test]
    fn test_from_string_invalid_values() {
        let test_cases = vec![
            "INVALID",
            "",
            "  EUR  ",
            "YEN",
            "  USD  ",
            "  ", // Only whitespace
            "US ", // Trailing whitespace, not preceded or followed by space, won't be trimmed
            "EUR-",
            "USSD",
        ];

        for input in test_cases {
            assert_eq!(Currency::from_string(input), None);
        }
    }
}
