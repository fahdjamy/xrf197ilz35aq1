use anyhow::anyhow;
use std::fmt::Display;
use std::str::FromStr;
use tracing::warn;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum OrderType {
    Asc,
    Desc,
}

impl Default for OrderType {
    fn default() -> Self {
        OrderType::Asc
    }
}

impl FromStr for OrderType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "asc" | "ascending" | "ASC" | "ASCENDING" => Ok(OrderType::Asc),
            "desc" | "descending" | "DESC" | "DESCENDING" => Ok(OrderType::Desc),
            _ => Err(anyhow!("invalid order type: {}", s)),
        }
    }
}

impl Display for OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderType::Asc => write!(f, "ASC"),
            OrderType::Desc => write!(f, "DESC"),
        }
    }
}

impl From<String> for OrderType {
    fn from(s: String) -> Self {
        crate::core::OrderType::from_str(&s).unwrap_or_else(|e| {
            warn!("invalid order type {} defaulting to: {}", e, OrderType::Asc);
            OrderType::Asc
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_type_from_str() {
        assert_eq!(OrderType::from_str("asc").unwrap(), OrderType::Asc);
        assert_eq!(OrderType::from_str("ASC").unwrap(), OrderType::Asc);
        assert_eq!(OrderType::from_str("ASCENDING").unwrap(), OrderType::Asc);
        assert_eq!(OrderType::from_str("ascending").unwrap(), OrderType::Asc);
        assert_eq!(OrderType::from_str("desc").unwrap(), OrderType::Desc);
        assert_eq!(OrderType::from_str("DESC").unwrap(), OrderType::Desc);
        assert_eq!(OrderType::from_str("descending").unwrap(), OrderType::Desc);
        assert_eq!(OrderType::from_str("DESCENDING").unwrap(), OrderType::Desc);
        assert!(OrderType::from_str("invalid").is_err());
    }

    #[test]
    fn test_order_type_from_string() {
        assert_eq!(OrderType::from("asc".to_string()), OrderType::Asc);
        assert_eq!(OrderType::from("DESC".to_string()), OrderType::Desc);
    }
}