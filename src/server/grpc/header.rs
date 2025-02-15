use tonic::metadata::{MetadataKey, MetadataMap};
use tracing::error;

pub const XRF_USER_FINGERPRINT: &str = "xrf-user-fp";

pub fn get_header_value(metadata_map: &MetadataMap, header_name: &str) -> Option<String> {
    let header_key = match MetadataKey::from_bytes(header_name.as_bytes()) {
        Ok(key) => key,
        Err(_) => {
            error!("Invalid header name: {}", header_name);
            return None;
        }
    };

    if let Some(header_value) = metadata_map.get(&header_key) {
        match header_value.to_str() {
            Ok(value_str) => Some(value_str.to_string()),
            Err(err) => {
                error!("Error decoding header value: {}", err);
                None
            }
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_header_value_exists() {
        let mut metadata_map = MetadataMap::new();
        metadata_map.insert("header2", "value2".parse().unwrap());
        metadata_map.insert(XRF_USER_FINGERPRINT, "my-value".parse().unwrap());
        let result_one = get_header_value(&metadata_map, XRF_USER_FINGERPRINT);
        assert!(result_one.is_some());
        let result_two = get_header_value(&metadata_map, "header2");
        assert!(result_two.is_some());
    }

    #[test]
    fn test_get_header_value_not_exists() {
        let metadata_map = MetadataMap::new();
        let result = get_header_value(&metadata_map, XRF_USER_FINGERPRINT);
        assert!(result.is_none());
    }
}
