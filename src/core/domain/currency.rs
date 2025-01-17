use std::fmt::{Display, Formatter};

#[derive(serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Crypto {
    Bitcoin,
    Ethereum,
    Xrp,
}

impl Display for Crypto {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Crypto::Bitcoin => { write!(f, "Bitcoin") }
            Crypto::Ethereum => { write!(f, "Ethereum") }
            Crypto::Xrp => { write!(f, "Xrp") }
        }
    }
}

#[derive(serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Fiat {
    USD,
    EUR,
}

impl Display for Fiat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Fiat::USD => { write!(f, "USD") }
            Fiat::EUR => { write!(f, "EUR") }
        }
    }
}

impl Fiat {
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "USD" => Some(Fiat::USD),
            "EUR" => Some(Fiat::EUR),
            _ => None
        }
    }
}

#[derive(serde::Deserialize, Clone, Debug)]
pub enum Currency {
    Crypto,
    Fiat,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_string_valid_values() {
        let test_cases = vec![
            ("USD", Some(Fiat::USD)),
            ("usd", Some(Fiat::USD)),
            ("EUR", Some(Fiat::EUR)),
            ("eur", Some(Fiat::EUR)),
            ("UsD", Some(Fiat::USD)),
        ];

        for (input, expected) in test_cases {
            assert_eq!(Fiat::from_string(input), expected);
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
            assert_eq!(Fiat::from_string(input), None);
        }
    }
}
