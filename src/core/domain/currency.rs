use strum_macros::EnumString;

#[derive(
    serde::Deserialize,
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
    EnumString,
    strum_macros::Display
)]
pub enum Currency {
    #[strum(serialize = "USD", serialize = "usd", serialize = "US Dollar", serialize = "us dollar")]
    USD,
    #[strum(serialize = "EUR", serialize = "eur", serialize = "Euro", serialize = "euro")]
    EUR,
    #[strum(serialize = "XRP", serialize = "xrp", serialize = "Ripple", serialize = "ripple")]
    XRP,
    #[strum(
        serialize = "RUB",
        serialize = "rub",
        serialize = "Russian Ruble",
        serialize = "russian ruble"
    )]
    RUB,
    #[strum(
        serialize = "ARS",
        serialize = "ars",
        serialize = "Argentine Peso",
        serialize = "argentine peso"
    )]
    ARS,
    #[strum(
        serialize = "BRL",
        serialize = "brl",
        serialize = "Brazilian Real",
        serialize = "brazilian real"
    )]
    BRL,
    #[strum(
        serialize = "CNY",
        serialize = "cny",
        serialize = "Chinese Yuan",
        serialize = "chinese yuan"
    )]
    CNY,
    #[strum(
        serialize = "GBP",
        serialize = "gbp",
        serialize = "British Pound",
        serialize = "british pound",
        serialize = "Pound Sterling",
        serialize = "pound sterling"
    )]
    GBP,
    #[strum(
        serialize = "MXN",
        serialize = "mxn",
        serialize = "Mexican Peso",
        serialize = "mexican peso"
    )]
    MXN,
    #[strum(
        serialize = "QAR",
        serialize = "qar",
        serialize = "Qatari Rial",
        serialize = "qatari rial"
    )]
    QAR,
    #[strum(
        serialize = "JPY",
        serialize = "jpy",
        serialize = "Japanese Yen",
        serialize = "japanese yen"
    )]
    JPY,
    ////////// CRYPTO Currencies
    #[strum(serialize = "DOGE", serialize = "doge", serialize = "Dogecoin", serialize = "dogecoin")]
    DOGE,
    #[strum(
        serialize = "XRFQ",
        serialize = "xrfq"
    )] // Assuming this is a made-up or very specific currency code
    XRFQ,
    #[strum(serialize = "SOL", serialize = "sol", serialize = "Solana", serialize = "solana")]
    SOLANA,
    #[strum(serialize = "BTC", serialize = "btc", serialize = "Bitcoin", serialize = "bitcoin")]
    BITCOIN,
    #[strum(serialize = "ETH", serialize = "eth", serialize = "Ethereum", serialize = "ethereum")]
    ETHEREUM,
    #[strum(serialize = "ADA", serialize = "ada", serialize = "Cardano", serialize = "cardano")]
    CARDANO,
    #[strum(serialize = "USDT", serialize = "usdt", serialize = "Tether", serialize = "tether")]
    TETHER,
    #[strum(
        serialize = "BNB",
        serialize = "bnb",
        serialize = "Binance Coin",
        serialize = "binance coin"
    )]
    BinanceCoin,
}

impl Currency {
    pub fn is_crypto(&self) -> bool {
        matches!(
            self,
            Currency::XRP
                | Currency::XRFQ
                | Currency::DOGE
                | Currency::SOLANA
                | Currency::TETHER
                | Currency::CARDANO
                | Currency::BITCOIN
                | Currency::ETHEREUM
                | Currency::BinanceCoin
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use strum::ParseError;

    #[test]
    fn test_from_string_valid_values() {
        let test_cases = vec![
            ("USD", Ok(Currency::USD)),
            ("usd", Ok(Currency::USD)),
            ("EUR", Ok(Currency::EUR)),
            ("eur", Ok(Currency::EUR)),
            ("XRP", Ok(Currency::XRP)),
            ("xrp", Ok(Currency::XRP)),
            ("Ripple", Ok(Currency::XRP)),
            ("Cardano", Ok(Currency::CARDANO)),
            ("Argentine Peso", Ok(Currency::ARS)),
        ];

        for (input, expected) in test_cases {
            assert_eq!(Currency::from_str(input), expected);
            assert_eq!(input.parse::<Currency>(), expected);
        }
    }

    #[test]
    fn test_from_string_invalid_values() {
        let test_cases = vec![
            "INVALID",
            "",
            "  EUR  ", // Strum trims whitespace by default
            "YEN",
            "  USD  ", // Strum trims whitespace by default
            "  ",       // Only whitespace, but trimmed
            "US ",      // Trailing whitespace (strum only trims at beginning and end, not in the middle, to avoid matching partials)
            "EUR-",
            "USSD",
            "UsD",
            "123",      // Numbers alone are not valid (unless you add a serialize for them)
            "-$£",      // Symbols alone are not valid (unless you add a serialize for them)
            " USD ", // Leading and trailing whitespace (trimmed by strum)
            "USD EUR", // Multiple valid currencies, but that doesn't match a single variant
            ".",        // Single punctuation
            "...",      // Multiple punctuations
            "USD.",    // Valid currency followed by punctuation
        ];

        for input in test_cases {
            // Expect an error (Err) when parsing invalid input
            let result: Result<Currency, ParseError> = Currency::from_str(input);
            assert!(result.is_err(), "Expected an error for input: \"{}\"", input);

            // Or, using .parse()
            let result = input.parse::<Currency>();
            assert!(result.is_err(), "Expected an error for input: \"{}\"", input);
        }
    }
}
