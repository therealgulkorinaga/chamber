//! Object engine for Chambers.
//!
//! Handles object creation, schema validation, lifecycle-class tagging,
//! transform-set binding, and payload enforcement.

use chambers_types::error::{SubstrateError, SubstrateResult};
use chambers_types::grammar::ObjectTypeSpec;
use chambers_types::object::*;
use chambers_types::world::WorldId;
use chrono::Utc;
use std::collections::HashMap;

/// The object engine validates and creates typed objects.
#[derive(Debug)]
pub struct ObjectEngine {
    /// Schema registry: object_type -> spec.
    schemas: HashMap<String, ObjectTypeSpec>,
}

impl ObjectEngine {
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
        }
    }

    /// Register object type schemas from a grammar.
    pub fn register_schemas(&mut self, specs: HashMap<String, ObjectTypeSpec>) {
        self.schemas = specs;
    }

    /// Create a new object, validating against the schema.
    pub fn create_object(
        &self,
        world_id: WorldId,
        object_type: String,
        payload: serde_json::Value,
        lifecycle_class: LifecycleClass,
        preservable: bool,
    ) -> SubstrateResult<Object> {
        // Validate type exists in schema
        let spec = self
            .schemas
            .get(&object_type)
            .ok_or_else(|| SubstrateError::UnknownObjectType(object_type.clone()))?;

        // Validate payload is not binary/blob
        self.validate_no_binary(&object_type, &payload)?;

        // Validate payload size
        let payload_bytes = serde_json::to_vec(&payload).unwrap_or_default();
        if payload_bytes.len() > spec.max_payload_bytes {
            return Err(SubstrateError::InvalidPayload {
                object_type: object_type.clone(),
                reason: format!(
                    "payload size {} exceeds max {} bytes",
                    payload_bytes.len(),
                    spec.max_payload_bytes
                ),
            });
        }

        // Validate preservable flag
        if preservable && !spec.can_be_preservable {
            return Err(SubstrateError::NotPreservable {
                object_type: object_type.clone(),
            });
        }

        let now = Utc::now();
        let object = Object {
            object_id: ObjectId::new(),
            world_id,
            object_type,
            lifecycle_class,
            payload,
            transform_set: spec.transform_set.clone(),
            preservable,
            capability_requirements: Vec::new(),
            created_at: now,
            last_modified_at: now,
            challenged: false,
            challenge_text: None,
            rank: None,
        };

        Ok(object)
    }

    /// Reject binary payloads, Base64-encoded blobs, and external references.
    fn validate_no_binary(
        &self,
        object_type: &str,
        payload: &serde_json::Value,
    ) -> SubstrateResult<()> {
        match payload {
            serde_json::Value::String(s) => {
                // Reject Base64-like strings over a threshold (heuristic)
                if s.len() > 1000 && Self::looks_like_base64(s) {
                    return Err(SubstrateError::BinaryPayloadRejected {
                        object_type: object_type.to_string(),
                    });
                }
                Ok(())
            }
            serde_json::Value::Object(map) => {
                // Check for external blob references
                if map.contains_key("$blob") || map.contains_key("$ref") || map.contains_key("$binary") {
                    return Err(SubstrateError::BinaryPayloadRejected {
                        object_type: object_type.to_string(),
                    });
                }
                // Recurse into values
                for v in map.values() {
                    self.validate_no_binary(object_type, v)?;
                }
                Ok(())
            }
            serde_json::Value::Array(arr) => {
                for v in arr {
                    self.validate_no_binary(object_type, v)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Heuristic: does this string look like Base64-encoded binary?
    fn looks_like_base64(s: &str) -> bool {
        if s.len() < 100 {
            return false;
        }
        let base64_chars = s
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '+' || *c == '/' || *c == '=')
            .count();
        let ratio = base64_chars as f64 / s.len() as f64;
        ratio > 0.95
    }

    /// Check if an object type is known.
    pub fn is_known_type(&self, object_type: &str) -> bool {
        self.schemas.contains_key(object_type)
    }

    /// Check if an object type is preservable.
    pub fn is_preservable_type(&self, object_type: &str) -> bool {
        self.schemas
            .get(object_type)
            .map(|s| s.can_be_preservable)
            .unwrap_or(false)
    }

    /// Get the transform set for an object type.
    pub fn get_transform_set(&self, object_type: &str) -> Option<&[chambers_types::primitive::Primitive]> {
        self.schemas.get(object_type).map(|s| s.transform_set.as_slice())
    }
}

impl Default for ObjectEngine {
    fn default() -> Self {
        Self::new()
    }
}
