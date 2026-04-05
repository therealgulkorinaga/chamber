//! Policy engine for Chambers.
//!
//! Loads grammar definitions and enforces:
//! - permitted object classes
//! - permitted primitive calls per epoch
//! - permitted views
//! - permitted state transitions
//! - preservation-law checks
//! - termination-law checks

use chambers_types::error::{SubstrateError, SubstrateResult};
use chambers_types::grammar::*;
use chambers_types::primitive::Primitive;
use chambers_types::world::{LifecyclePhase, TerminationMode};
use std::collections::HashMap;
use std::sync::RwLock;

/// The policy engine — grammar-driven rule enforcement.
#[derive(Debug)]
pub struct PolicyEngine {
    grammars: RwLock<HashMap<String, ChamberGrammar>>,
}

impl PolicyEngine {
    pub fn new() -> Self {
        Self {
            grammars: RwLock::new(HashMap::new()),
        }
    }

    /// Load a grammar definition.
    pub fn load_grammar(&self, grammar: ChamberGrammar) -> SubstrateResult<()> {
        let id = grammar.grammar_id.clone();
        self.grammars.write().unwrap().insert(id, grammar);
        Ok(())
    }

    /// Check if an object type is allowed in a grammar.
    pub fn is_object_type_allowed(&self, grammar_id: &str, object_type: &str) -> SubstrateResult<bool> {
        let grammars = self.grammars.read().unwrap();
        let grammar = grammars
            .get(grammar_id)
            .ok_or_else(|| SubstrateError::GrammarNotFound(grammar_id.to_string()))?;
        Ok(grammar.object_types.contains_key(object_type))
    }

    /// Check if a primitive is allowed in the current lifecycle phase.
    pub fn is_primitive_allowed(
        &self,
        grammar_id: &str,
        primitive: Primitive,
        phase: LifecyclePhase,
    ) -> SubstrateResult<bool> {
        let grammars = self.grammars.read().unwrap();
        let grammar = grammars
            .get(grammar_id)
            .ok_or_else(|| SubstrateError::GrammarNotFound(grammar_id.to_string()))?;
        let phase_key = LifecyclePhaseKey::from(phase);

        if let Some(allowed) = grammar.phase_primitives.get(&phase_key) {
            Ok(allowed.contains(&primitive))
        } else {
            Ok(false)
        }
    }

    /// Check if an object type is preservable under the grammar's preservation law.
    pub fn can_preserve_object(&self, grammar_id: &str, object_type: &str) -> SubstrateResult<bool> {
        let grammars = self.grammars.read().unwrap();
        let grammar = grammars
            .get(grammar_id)
            .ok_or_else(|| SubstrateError::GrammarNotFound(grammar_id.to_string()))?;
        Ok(grammar.preservable_classes.contains(&object_type.to_string()))
    }

    /// Validate a termination mode against the grammar's termination law.
    pub fn validate_termination(
        &self,
        grammar_id: &str,
        mode: TerminationMode,
        has_artifact: bool,
    ) -> SubstrateResult<()> {
        let grammars = self.grammars.read().unwrap();
        let grammar = grammars
            .get(grammar_id)
            .ok_or_else(|| SubstrateError::GrammarNotFound(grammar_id.to_string()))?;

        let law = grammar
            .termination_modes
            .iter()
            .find(|t| t.mode == mode)
            .ok_or_else(|| SubstrateError::PolicyViolation(format!(
                "termination mode {:?} not permitted by grammar",
                mode
            )))?;

        if law.requires_artifact && !has_artifact {
            return Err(SubstrateError::NoArtifactForPreservation);
        }

        Ok(())
    }

    /// Check if a lifecycle transition is legal under the grammar.
    pub fn is_transition_legal(
        &self,
        _grammar_id: &str,
        current: LifecyclePhase,
        target: LifecyclePhase,
    ) -> SubstrateResult<bool> {
        Ok(current.can_transition_to(target))
    }

    /// Check if a view is allowed.
    pub fn is_view_allowed(&self, grammar_id: &str, view: &str) -> SubstrateResult<bool> {
        let grammars = self.grammars.read().unwrap();
        let grammar = grammars
            .get(grammar_id)
            .ok_or_else(|| SubstrateError::GrammarNotFound(grammar_id.to_string()))?;
        Ok(grammar.allowed_views.contains(&view.to_string()))
    }

    /// Check if a link type is permitted between source and target types.
    pub fn is_link_permitted(
        &self,
        grammar_id: &str,
        link_type: &str,
        source_type: &str,
        target_type: &str,
    ) -> SubstrateResult<bool> {
        let grammars = self.grammars.read().unwrap();
        let grammar = grammars
            .get(grammar_id)
            .ok_or_else(|| SubstrateError::GrammarNotFound(grammar_id.to_string()))?;
        Ok(grammar.permitted_links.iter().any(|l| {
            l.link_type == link_type
                && l.source_types.contains(&source_type.to_string())
                && l.target_types.contains(&target_type.to_string())
        }))
    }

    /// Get preservable classes for a grammar.
    pub fn get_preservable_classes(&self, grammar_id: &str) -> SubstrateResult<Vec<String>> {
        let grammars = self.grammars.read().unwrap();
        let grammar = grammars
            .get(grammar_id)
            .ok_or_else(|| SubstrateError::GrammarNotFound(grammar_id.to_string()))?;
        Ok(grammar.preservable_classes.clone())
    }

    /// Get object types for a grammar.
    pub fn get_object_types(&self, grammar_id: &str) -> SubstrateResult<Vec<String>> {
        let grammars = self.grammars.read().unwrap();
        let grammar = grammars
            .get(grammar_id)
            .ok_or_else(|| SubstrateError::GrammarNotFound(grammar_id.to_string()))?;
        Ok(grammar.object_types.keys().cloned().collect())
    }

    /// Get the grammar (cloned, since we can't return a reference through RwLock).
    pub fn get_grammar(&self, grammar_id: &str) -> SubstrateResult<ChamberGrammar> {
        let grammars = self.grammars.read().unwrap();
        grammars
            .get(grammar_id)
            .cloned()
            .ok_or_else(|| SubstrateError::GrammarNotFound(grammar_id.to_string()))
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}
