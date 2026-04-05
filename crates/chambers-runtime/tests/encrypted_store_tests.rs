//! Phase 2: Encrypted memory pool integration tests.
//!
//! Verifies: objects encrypted in RAM, scoped decryption works,
//! ciphertext is not plaintext, burn makes data unrecoverable.

use chambers_crypto::encrypted_store::*;
use chambers_crypto::{CryptoProvider, WorldKey};
use chambers_types::object::*;
use chambers_types::primitive::Primitive;
use chambers_types::world::WorldId;
use chrono::Utc;

fn test_key() -> WorldKey {
    WorldKey {
        key_bytes: [42u8; 32],
    }
}

fn test_object(text: &str) -> Object {
    let now = Utc::now();
    Object {
        object_id: ObjectId::new(),
        world_id: WorldId::new(),
        object_type: "premise".into(),
        lifecycle_class: LifecycleClass::Temporary,
        payload: serde_json::json!({"statement": text}),
        transform_set: vec![],
        preservable: false,
        capability_requirements: vec![],
        created_at: now,
        last_modified_at: now,
        challenged: false,
        challenge_text: None,
        rank: None,
    }
}

#[test]
fn test_encrypt_decrypt_roundtrip() {
    let key = test_key();
    let obj = test_object("TOP SECRET: Project Lazarus costs $4.2M");

    let enc = encrypt_object(&obj, &key).unwrap();

    // Ciphertext should not contain the plaintext
    let cipher_str = String::from_utf8_lossy(&enc.ciphertext);
    assert!(
        !cipher_str.contains("TOP SECRET"),
        "ciphertext must not contain plaintext"
    );
    assert!(
        !cipher_str.contains("Lazarus"),
        "ciphertext must not contain plaintext"
    );

    // Decrypt should recover the original
    let decrypted = decrypt_object(&enc, &key).unwrap();
    assert_eq!(decrypted.payload, obj.payload);
    assert_eq!(decrypted.object_type, obj.object_type);
}

#[test]
fn test_encrypted_store_scoped_access() {
    let key = test_key();
    let obj = test_object("Sensitive decision data");
    let oid = obj.object_id;

    let mut store = EncryptedWorldState::new();
    store.add_object(&obj, &key).unwrap();

    // Scoped read — plaintext only inside closure
    let payload_text = store
        .with_object(oid, &key, |o| {
            o.payload["statement"].as_str().unwrap().to_string()
        })
        .unwrap();

    assert_eq!(payload_text, "Sensitive decision data");

    // The store itself holds only ciphertext
    let enc = store.objects.get(&oid).unwrap();
    let cipher_str = String::from_utf8_lossy(&enc.ciphertext);
    assert!(
        !cipher_str.contains("Sensitive"),
        "stored data must be ciphertext"
    );
}

#[test]
fn test_encrypted_store_mutation() {
    let key = test_key();
    let obj = test_object("Original text");
    let oid = obj.object_id;

    let mut store = EncryptedWorldState::new();
    store.add_object(&obj, &key).unwrap();

    // Modify through scoped access
    store
        .with_object_mut(oid, &key, |o| {
            o.payload = serde_json::json!({"statement": "Modified text"});
        })
        .unwrap();

    // Read back
    let text = store
        .with_object(oid, &key, |o| {
            o.payload["statement"].as_str().unwrap().to_string()
        })
        .unwrap();

    assert_eq!(text, "Modified text");
}

#[test]
fn test_wrong_key_fails() {
    let key1 = WorldKey { key_bytes: [1u8; 32] };
    let key2 = WorldKey { key_bytes: [2u8; 32] };
    let obj = test_object("Secret");

    let enc = encrypt_object(&obj, &key1).unwrap();

    // Decrypt with wrong key should fail
    let result = decrypt_object(&enc, &key2);
    assert!(result.is_err(), "decryption with wrong key must fail");
}

#[test]
fn test_encrypted_link_roundtrip() {
    let key = test_key();
    let link = ObjectLink {
        source_id: ObjectId::new(),
        target_id: ObjectId::new(),
        link_type: "risks".into(),
        world_id: WorldId::new(),
    };

    let enc = encrypt_link(&link, &key).unwrap();
    let decrypted = decrypt_link(&enc, &key).unwrap();

    assert_eq!(decrypted.link_type, "risks");
    assert_eq!(decrypted.source_id, link.source_id);
    assert_eq!(decrypted.target_id, link.target_id);
}

#[test]
fn test_secure_wipe() {
    let key = test_key();
    let mut store = EncryptedWorldState::new();

    for i in 0..10 {
        store
            .add_object(&test_object(&format!("Object {}", i)), &key)
            .unwrap();
    }

    assert_eq!(store.object_count(), 10);

    store.secure_wipe();

    assert_eq!(store.object_count(), 0);
    assert_eq!(store.link_count(), 0);
}

#[test]
fn test_plaintext_index_available_without_decryption() {
    let key = test_key();
    let mut obj = test_object("Content doesn't matter for this test");
    obj.object_type = "decision_summary".into();
    obj.preservable = true;
    let oid = obj.object_id;

    let mut store = EncryptedWorldState::new();
    store.add_object(&obj, &key).unwrap();

    // These checks use the plaintext index — no decryption needed
    assert_eq!(store.object_type(oid), Some("decision_summary"));
    assert!(store.is_preservable(oid));
    assert!(store.has_objects_of_type("decision_summary"));
    assert!(!store.has_objects_of_type("premise"));
}
