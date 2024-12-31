use anyhow::anyhow;
use std::fmt::Display;
use std::str::FromStr;
use tracing::warn;

#[derive(Debug, Clone, Copy)]
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
