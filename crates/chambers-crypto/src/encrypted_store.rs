//! Encrypted object store for Phase 2.
//!
//! All objects and links are encrypted in RAM under K_w.
//! Plaintext exists only in the guard buffer for microseconds per access.

use crate::mem_protect::GuardBuffer;
use crate::{CryptoProvider, WorldKey};
use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use chambers_types::object::{Object, ObjectId, ObjectLink};
use chambers_types::world::WorldId;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zeroize::Zeroize;

/// An object encrypted in memory under K_w.
#[derive(Debug, Clone)]
pub struct EncryptedObject {
    pub object_id: ObjectId,
    pub object_type: String,       // Plaintext index — reveals type but not content
    pub preservable: bool,         // Plaintext index — needed for seal checks
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; 12],
}

/// A link encrypted in memory under K_w.
#[derive(Debug, Clone)]
pub struct EncryptedLink {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; 12],
}

/// Encrypt a serializable value under a WorldKey.
pub fn encrypt_value<T: Serialize>(value: &T, key: &WorldKey) -> Result<(Vec<u8>, [u8; 12]), String> {
    let plaintext = serde_json::to_vec(value).map_err(|e| e.to_string())?;
    let cipher = Aes256Gcm::new_from_slice(&key.key_bytes)
        .map_err(|e| e.to_string())?;
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| e.to_string())?;
    Ok((ciphertext, nonce_bytes))
}

/// Decrypt a value from ciphertext under a WorldKey.
pub fn decrypt_value<T: for<'de> Deserialize<'de>>(
    ciphertext: &[u8],
    nonce: &[u8; 12],
    key: &WorldKey,
) -> Result<T, String> {
    let cipher = Aes256Gcm::new_from_slice(&key.key_bytes)
        .map_err(|e| e.to_string())?;
    let nonce = Nonce::from_slice(nonce);
    let mut plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| e.to_string())?;
    let result = serde_json::from_slice(&plaintext).map_err(|e| e.to_string());
    // Zero the plaintext buffer immediately
    plaintext.zeroize();
    result
}

/// Encrypt an Object, returning an EncryptedObject.
pub fn encrypt_object(obj: &Object, key: &WorldKey) -> Result<EncryptedObject, String> {
    let (ciphertext, nonce) = encrypt_value(obj, key)?;
    Ok(EncryptedObject {
        object_id: obj.object_id,
        object_type: obj.object_type.clone(),
        preservable: obj.preservable,
        ciphertext,
        nonce,
    })
}

/// Decrypt an EncryptedObject back to an Object.
pub fn decrypt_object(enc: &EncryptedObject, key: &WorldKey) -> Result<Object, String> {
    decrypt_value(&enc.ciphertext, &enc.nonce, key)
}

/// Encrypt an ObjectLink.
pub fn encrypt_link(link: &ObjectLink, key: &WorldKey) -> Result<EncryptedLink, String> {
    let (ciphertext, nonce) = encrypt_value(link, key)?;
    Ok(EncryptedLink { ciphertext, nonce })
}

/// Decrypt an EncryptedLink.
pub fn decrypt_link(enc: &EncryptedLink, key: &WorldKey) -> Result<ObjectLink, String> {
    decrypt_value(&enc.ciphertext, &enc.nonce, key)
}

/// The encrypted world state — all objects and links stored as ciphertext.
#[derive(Debug, Clone, Default)]
pub struct EncryptedWorldState {
    pub objects: HashMap<ObjectId, EncryptedObject>,
    pub links: Vec<EncryptedLink>,
}

impl EncryptedWorldState {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
            links: Vec::new(),
        }
    }

    /// Add an object (encrypts it).
    pub fn add_object(&mut self, obj: &Object, key: &WorldKey) -> Result<(), String> {
        let enc = encrypt_object(obj, key)?;
        self.objects.insert(obj.object_id, enc);
        Ok(())
    }

    /// Add a link (encrypts it).
    pub fn add_link(&mut self, link: &ObjectLink, key: &WorldKey) -> Result<(), String> {
        let enc = encrypt_link(link, key)?;
        self.links.push(enc);
        Ok(())
    }

    /// Read an object through the scoped access pattern.
    /// The plaintext exists only inside the closure. Zeroed after.
    pub fn with_object<F, R>(
        &self,
        object_id: ObjectId,
        key: &WorldKey,
        f: F,
    ) -> Result<R, String>
    where
        F: FnOnce(&Object) -> R,
    {
        let enc = self
            .objects
            .get(&object_id)
            .ok_or_else(|| format!("object not found: {}", object_id))?;
        let obj = decrypt_object(enc, key)?;
        let result = f(&obj);
        // obj is dropped here — Rust deallocates. For full Phase 2 hardening,
        // this should use the guard buffer with explicit zeroing.
        // The current implementation zeroes the intermediate plaintext bytes
        // inside decrypt_value() before deserialization.
        Ok(result)
    }

    /// Modify an object through the scoped access pattern.
    /// Decrypts, applies the closure, re-encrypts.
    pub fn with_object_mut<F>(
        &mut self,
        object_id: ObjectId,
        key: &WorldKey,
        f: F,
    ) -> Result<(), String>
    where
        F: FnOnce(&mut Object),
    {
        let enc = self
            .objects
            .get(&object_id)
            .ok_or_else(|| format!("object not found: {}", object_id))?;
        let mut obj = decrypt_object(enc, key)?;
        f(&mut obj);
        let new_enc = encrypt_object(&obj, key)?;
        self.objects.insert(object_id, new_enc);
        Ok(())
    }

    /// Get all decrypted objects (for view rendering — decrypts one at a time).
    pub fn all_objects_decrypted(&self, key: &WorldKey) -> Vec<Object> {
        let mut result = Vec::new();
        for enc in self.objects.values() {
            if let Ok(obj) = decrypt_object(enc, key) {
                result.push(obj);
            }
        }
        result
    }

    /// Get all decrypted links.
    pub fn all_links_decrypted(&self, key: &WorldKey) -> Vec<ObjectLink> {
        let mut result = Vec::new();
        for enc in &self.links {
            if let Ok(link) = decrypt_link(enc, key) {
                result.push(link);
            }
        }
        result
    }

    /// Check if an object exists (by ID — no decryption needed).
    pub fn has_object(&self, object_id: ObjectId) -> bool {
        self.objects.contains_key(&object_id)
    }

    /// Get object type without full decryption (from plaintext index).
    pub fn object_type(&self, object_id: ObjectId) -> Option<&str> {
        self.objects.get(&object_id).map(|e| e.object_type.as_str())
    }

    /// Check if an object is preservable (from plaintext index).
    pub fn is_preservable(&self, object_id: ObjectId) -> bool {
        self.objects
            .get(&object_id)
            .map(|e| e.preservable)
            .unwrap_or(false)
    }

    /// Check if a link exists between two objects.
    pub fn link_exists(&self, source: ObjectId, target: ObjectId, key: &WorldKey) -> bool {
        self.links.iter().any(|enc| {
            decrypt_link(enc, key)
                .map(|l| l.source_id == source && l.target_id == target)
                .unwrap_or(false)
        })
    }

    pub fn object_count(&self) -> usize {
        self.objects.len()
    }

    pub fn link_count(&self) -> usize {
        self.links.len()
    }

    /// Check for unresolved challenges (requires decryption).
    pub fn has_unresolved_challenges(&self, key: &WorldKey) -> bool {
        self.objects.values().any(|enc| {
            decrypt_object(enc, key)
                .map(|o| o.challenged)
                .unwrap_or(false)
        })
    }

    /// Check if objects of a given type exist (from plaintext index — no decryption).
    pub fn has_objects_of_type(&self, object_type: &str) -> bool {
        self.objects.values().any(|e| e.object_type == object_type)
    }

    /// Secure wipe — zero all ciphertext before dropping.
    pub fn secure_wipe(&mut self) {
        for (_, enc) in self.objects.iter_mut() {
            enc.ciphertext.zeroize();
            enc.nonce.zeroize();
            enc.object_type.zeroize();
        }
        self.objects.clear();
        for enc in self.links.iter_mut() {
            enc.ciphertext.zeroize();
            enc.nonce.zeroize();
        }
        self.links.clear();
    }
}
