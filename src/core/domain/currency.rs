use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::postgres::PgTypeInfo;
use sqlx::{Database, TypeInfo};
use sqlx::{Decode, Encode, Postgres, Type};
use strum_macros::EnumString;

#[derive(
    serde::Deserialize,
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
    EnumString,
    strum_macros::Display,
    Type
)]
#[sqlx(type_name = "currency_enum")]
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

// Vec<Currency> type wrapper
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CurrencyList(pub Vec<Currency>);

// 2. Implement Type for Vec<Currency> to represent currency_enum[]
impl Type<Postgres> for CurrencyList {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_currency_enum") // Note the leading underscore for array types in PostgreSQL
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        ty.name() == "_currency_enum"
    }
}

// 3. Implement Encode for Vec<Currency> to handle serialization to PostgreSQL
impl<'q> Encode<'q, Postgres> for CurrencyList {
    fn encode_by_ref(&self, buf: &mut <Postgres as Database>::ArgumentBuffer<'q>) -> Result<IsNull, BoxDynError> {
        let mut encoder = sqlx::postgres::types::PgRecordEncoder::new(buf);
        for currency in &self.0 {
            encoder.encode(currency).expect("failed to encode currency");
        }
        encoder.finish();
        Ok(IsNull::No)
    }
}

// 4. Implement Decode for Vec<Currency> to handle deserialization from PostgresSQL
impl<'r> Decode<'r, Postgres> for CurrencyList {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let mut decoder = sqlx::postgres::types::PgRecordDecoder::new(value)?;
        let mut currencies = Vec::new();

        // Loop until try_decode returns None
        loop {
            let maybe_currency: Result<Option<Currency>, BoxDynError> = decoder.try_decode();

            match maybe_currency {
                Ok(Some(currency)) => {
                    currencies.push(currency);
                },
                Ok(None) => {
                    break;
                }
                Err(e) => return Err(e.into()),
            }
        }
        Ok(CurrencyList(currencies))
    }
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

    pub fn db_string(&self) -> &'static str {
        match self {
            Currency::USD => "USD",
            Currency::EUR => "EUR",
            Currency::XRP => "XRP",
            Currency::RUB => "RUB",
            Currency::ARS => "ARS",
            Currency::BRL => "BRL",
            Currency::CNY => "CNY",
            Currency::GBP => "GBP",
            Currency::MXN => "MXN",
            Currency::QAR => "QAR",
            Currency::JPY => "JPY",
            Currency::DOGE => "DOGE",
            Currency::XRFQ => "XRFQ",
            Currency::SOLANA => "SOL",
            Currency::BITCOIN => "BTC",
            Currency::TETHER => "USDT",
            Currency::CARDANO => "ADA",
            Currency::ETHEREUM => "ETH",
            Currency::BinanceCoin => "BNB",
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::Currency;
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
