//! UUID Helper Functions

use uuid::Uuid;

/// Generate new UUID v4
pub fn generate_uuid() -> Uuid {
    Uuid::new_v4()
}

/// Generate UUID from string
pub fn uuid_from_string(s: &str) -> Result<Uuid, uuid::Error> {
    Uuid::parse_str(s)
}

/// Convert UUID to string without hyphens
pub fn uuid_to_compact(uuid: &Uuid) -> String {
    uuid.to_string().replace('-', "")
}

/// Parse UUID from compact format (without hyphens)
pub fn uuid_from_compact(s: &str) -> Result<Uuid, String> {
    // Add hyphens back
    if s.len() != 32 {
        return Err(format!("Invalid length: expected 32, got {}", s.len()));
    }

    let formatted = format!(
        "{}-{}-{}-{}-{}",
        &s[0..8],
        &s[8..12],
        &s[12..16],
        &s[16..20],
        &s[20..32]
    );

    Uuid::parse_str(&formatted).map_err(|e| e.to_string())
}

/// Check if string is valid UUID
pub fn is_valid_uuid(s: &str) -> bool {
    Uuid::parse_str(s).is_ok()
}

/// Generate multiple UUIDs
pub fn generate_uuids(count: usize) -> Vec<Uuid> {
    (0..count).map(|_| Uuid::new_v4()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_uuid() {
        let uuid = generate_uuid();
        assert!(uuid.get_version_num() == 4);
    }

    #[test]
    fn test_uuid_from_string() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let uuid = uuid_from_string(uuid_str).unwrap();
        assert_eq!(uuid.to_string(), uuid_str);
    }

    #[test]
    fn test_uuid_compact() {
        let uuid = generate_uuid();
        let compact = uuid_to_compact(&uuid);
        assert_eq!(compact.len(), 32);
        assert!(!compact.contains('-'));

        let parsed = uuid_from_compact(&compact).unwrap();
        assert_eq!(parsed, uuid);
    }

    #[test]
    fn test_is_valid_uuid() {
        assert!(is_valid_uuid("550e8400-e29b-41d4-a716-446655440000"));
        assert!(!is_valid_uuid("invalid-uuid"));
        assert!(!is_valid_uuid(""));
    }

    #[test]
    fn test_generate_uuids() {
        let uuids = generate_uuids(5);
        assert_eq!(uuids.len(), 5);

        // Check all UUIDs are unique
        for i in 0..uuids.len() {
            for j in i + 1..uuids.len() {
                assert_ne!(uuids[i], uuids[j]);
            }
        }
    }
}
