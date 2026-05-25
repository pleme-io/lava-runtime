//! LavaRuntime — tatara-lisp via `lava-eval` (in-process).

use crate::{ArtifactBinding, ArtifactInput, EmbeddedRuntime, EvaluationResult, RuntimeError};
use lava_eval::{eval_architecture, InputBindings};

/// Embedded `.tlisp` runtime. Wraps `lava-eval` so magma loads
/// architectures in-process — no shell-out, no IPC.
#[derive(Debug, Default, Clone)]
pub struct LavaRuntime;

impl LavaRuntime {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl EmbeddedRuntime for LavaRuntime {
    fn kind(&self) -> &'static str {
        "lava"
    }

    fn extension(&self) -> &'static str {
        "tlisp"
    }

    fn evaluate(&self, input: &ArtifactInput) -> Result<EvaluationResult, RuntimeError> {
        let mut b = InputBindings::new();
        for (k, v) in &input.bindings {
            match v {
                ArtifactBinding::Scalar(s) => b.set_str(k.clone(), s.clone()),
                ArtifactBinding::List(items) => b.set_list(k.clone(), items.clone()),
            }
        }
        let arch = eval_architecture(&input.source, &b)
            .map_err(|e| RuntimeError::Evaluate(e.to_string()))?;
        Ok(EvaluationResult {
            architecture: arch,
            diagnostics: vec![],
        })
    }
}
