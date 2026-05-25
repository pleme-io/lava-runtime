//! LavaRuntime — tatara-lisp via `lava-eval` (in-process).

use crate::{
    ArtifactBinding, ArtifactInput, EmbeddedRuntime, EvaluationResult, Interface, RuntimeError,
};
use lava_eval::{eval_architecture, eval_architecture_with_schema, EvalError, InputBindings};

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
        let b = bindings_from(input);
        let arch = eval_architecture(&input.source, &b)
            .map_err(|e| RuntimeError::Evaluate(e.to_string()))?;
        Ok(EvaluationResult {
            architecture: arch,
            diagnostics: vec![],
        })
    }

    /// Native override — routes through `lava-eval`'s
    /// `eval_architecture_with_schema` so the typed gate happens in
    /// the same pass that constructs the architecture (one validate,
    /// one walk, no rebuild of the InputBindings shape).
    fn evaluate_with_schema(
        &self,
        input: &ArtifactInput,
        iface: &Interface,
    ) -> Result<EvaluationResult, RuntimeError> {
        let b = bindings_from(input);
        let arch = eval_architecture_with_schema(&input.source, &b, iface).map_err(|e| match e {
            EvalError::Schema {
                interface,
                count,
                first,
                ..
            } => RuntimeError::Schema {
                interface,
                count,
                first,
            },
            other => RuntimeError::Evaluate(other.to_string()),
        })?;
        Ok(EvaluationResult {
            architecture: arch,
            diagnostics: vec![],
        })
    }
}

fn bindings_from(input: &ArtifactInput) -> InputBindings {
    let mut b = InputBindings::new();
    for (k, v) in &input.bindings {
        match v {
            ArtifactBinding::Scalar(s) => b.set_str(k.clone(), s.clone()),
            ArtifactBinding::List(items) => b.set_list(k.clone(), items.clone()),
        }
    }
    b
}
