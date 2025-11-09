use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct Relays {
    pub relays: Vec<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PublishedEvent {
    pub event_id: String,
    pub published_at: String,
    pub kind: u16,
    pub hash: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PublishedRegistry {
    #[serde(flatten)]
    pub articles: HashMap<String, PublishedEvent>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use toml;
    
    // Tests for Relays struct
    #[test]
    fn test_relays_deserialization_valid() {
        let toml_content = r#"
            relays = [
                "wss://relay1.example.com",
                "wss://relay2.example.com",
                "wss://relay3.example.com"
            ]
        "#;
        
        let relays: Result<Relays, _> = toml::from_str(toml_content);
        assert!(relays.is_ok());
        
        let relays = relays.unwrap();
        assert_eq!(relays.relays.len(), 3);
        assert_eq!(relays.relays[0], "wss://relay1.example.com");
        assert_eq!(relays.relays[1], "wss://relay2.example.com");
        assert_eq!(relays.relays[2], "wss://relay3.example.com");
    }

    #[test]
    fn test_relays_deserialization_empty() {
        let toml_content = r#"
            relays = []
        "#;
        
        let relays: Result<Relays, _> = toml::from_str(toml_content);
        assert!(relays.is_ok());
        
        let relays = relays.unwrap();
        assert_eq!(relays.relays.len(), 0);
    }

    #[test]
    fn test_relays_deserialization_single() {
        let toml_content = r#"
            relays = ["wss://single-relay.example.com"]
        "#;
        
        let relays: Result<Relays, _> = toml::from_str(toml_content);
        assert!(relays.is_ok());
        
        let relays = relays.unwrap();
        assert_eq!(relays.relays.len(), 1);
        assert_eq!(relays.relays[0], "wss://single-relay.example.com");
    }

    #[test]
    fn test_relays_serialization() {
        let relays = Relays {
            relays: vec![
                "wss://relay1.example.com".to_string(),
                "wss://relay2.example.com".to_string(),
            ]
        };
        
        let serialized = toml::to_string(&relays);
        assert!(serialized.is_ok());
        
        let serialized = serialized.unwrap();
        assert!(serialized.contains("relay1.example.com"));
        assert!(serialized.contains("relay2.example.com"));
    }

    #[test]
    fn test_relays_deserialization_invalid_toml() {
        let invalid_toml = r#"
            invalid_key = "value"
        "#;
        
        let relays: Result<Relays, _> = toml::from_str(invalid_toml);
        assert!(relays.is_err());
    }

    #[test]
    fn test_relays_deserialization_wrong_type() {
        let invalid_toml = r#"
            relays = "should_be_array"
        "#;
        
        let relays: Result<Relays, _> = toml::from_str(invalid_toml);
        assert!(relays.is_err());
    }

    // Test round-trip serialization/deserialization
    #[test]
    fn test_relays_round_trip() {
        let original = Relays {
            relays: vec![
                "wss://relay1.example.com".to_string(),
                "wss://relay2.example.com".to_string(),
                "wss://relay3.example.com".to_string(),
            ]
        };
        
        let serialized = toml::to_string(&original).unwrap();
        let deserialized: Relays = toml::from_str(&serialized).unwrap();
        
        assert_eq!(original.relays.len(), deserialized.relays.len());
        for (orig, deser) in original.relays.iter().zip(deserialized.relays.iter()) {
            assert_eq!(orig, deser);
        }
    }
}