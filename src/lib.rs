//! lava-runtime — unified `EmbeddedRuntime` trait for in-process IaC
//! DSL evaluation.
//!
//! Magma + future orchestrators consume any DSL implementation
//! through one typed surface — same Plan flow regardless of authoring
//! language. Zero shell-out, zero IPC, zero disk-roundtrip between
//! authoring and apply.
//!
//! ## Implementations
//!
//! | Runtime | DSL | Backed by | Status |
//! |---|---|---|---|
//! | [`LavaRuntime`] | tatara-lisp | `lava-eval` (in-process) | ✓ shipped |
//! | `RubyRuntime`   | Ruby DSL    | `pangea-ruby-eval` (magnus + CRuby) | planned |
//! | `TataraRuntime` | full tatara-script | `actions/tatara-script` crate | planned |
//! | `TerraformJsonRuntime` | terraform.json | direct passthrough | ✓ shipped |
//!
//! ## Usage
//!
//! ```ignore
//! use lava_runtime::{EmbeddedRuntime, LavaRuntime, ArtifactInput};
//! let rt = LavaRuntime::new();
//! let arch = rt.evaluate(&ArtifactInput {
//!     source: tlisp_source,
//!     bindings: bindings,
//! })?;
//! let json = arch.render_terraform_json()?;
//! // magma's existing config-loading consumes `json` and applies.
//! ```

#![allow(clippy::module_name_repetitions)]

use indexmap::IndexMap;
use lava_core::Architecture;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod lava;
pub mod terraform;

pub use lava::LavaRuntime;
pub use terraform::TerraformJsonRuntime;

/// Typed input to an [`EmbeddedRuntime::evaluate`] call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactInput {
    /// Source text (`.tlisp` / `.rb` / `.tf.json` / etc).
    pub source: String,
    /// Operator-supplied input bindings; what flows in for `:inputs`
    /// the architecture declares. Scalar strings + string lists are
    /// the two universal shapes every runtime can accept.
    pub bindings: IndexMap<String, ArtifactBinding>,
    /// Optional name hint (file basename, repo slug) used in error
    /// messages + plan output.
    pub name: Option<String>,
}

/// Typed scalar | list binding. Every DSL runtime accepts the same
/// two universal value shapes — keeps the trait minimal + portable.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArtifactBinding {
    Scalar(String),
    List(Vec<String>),
}

/// The typed return: one `lava_core::Architecture`. Every runtime
/// returns this regardless of authoring language. Downstream
/// `Synthesizer<TerraformJson | MagmaPlan | CrossplaneYaml | …>`
/// turns it into the wire shape magma applies against.
#[derive(Debug, Clone)]
pub struct EvaluationResult {
    pub architecture: Architecture,
    /// Runtime-emitted diagnostic messages (info / warning / error).
    /// Magma surfaces these in plan output.
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub source_location: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticLevel {
    Info,
    Warning,
    Error,
}

/// Typed trait every embedded DSL runtime implements. Magma's
/// existing plan/apply pipeline consumes the resulting Architecture
/// the same way regardless of which runtime produced it.
pub trait EmbeddedRuntime {
    /// Stable runtime kind — `"lava"`, `"ruby"`, `"terraform-json"`.
    /// Surfaced in diagnostics + the plan receipt's `:authored-by` field.
    fn kind(&self) -> &'static str;

    /// File extension hint — `"tlisp"`, `"rb"`, `"tf.json"`. Magma's
    /// auto-detect routing dispatches inputs to the matching runtime
    /// using this hint.
    fn extension(&self) -> &'static str;

    /// Evaluate the input + produce a typed `EvaluationResult`.
    fn evaluate(&self, input: &ArtifactInput) -> Result<EvaluationResult, RuntimeError>;
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("parse: {0}")]
    Parse(String),
    #[error("evaluate: {0}")]
    Evaluate(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("upstream: {0}")]
    Upstream(String),
}

/// Auto-detect the right runtime from a file extension. Returns the
/// runtime + the loaded source. Lets magma route `.tlisp`, `.tf.json`,
/// (and future `.rb`, `.scm`) through one entry point.
#[must_use]
pub fn pick_runtime_for_path(path: &std::path::Path) -> Option<Box<dyn EmbeddedRuntime>> {
    let ext = path.extension()?.to_str()?;
    match ext {
        "tlisp" => Some(Box::new(LavaRuntime::new())),
        "json" | "tf.json" => Some(Box::new(TerraformJsonRuntime::new())),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vpc_tlisp() -> &'static str {
        r#"
        (deflava-architecture aws-vpc-tiny
          :inputs ((:cidr "10.0.0.0/16"))
          :resources (
            (aws-vpc "main"
              :cidr-block "{cidr}"
              :enable-dns-support #t)))
        "#
    }

    #[test]
    fn lava_runtime_evaluates_tlisp_to_architecture() {
        let rt = LavaRuntime::new();
        let input = ArtifactInput {
            source: vpc_tlisp().to_string(),
            bindings: IndexMap::new(),
            name: Some("aws-vpc-tiny".to_string()),
        };
        let result = rt.evaluate(&input).unwrap();
        let json = result.architecture.render_terraform_json().unwrap();
        assert_eq!(
            json["resource"]["aws_vpc"]["main"]["cidr_block"],
            "10.0.0.0/16"
        );
        assert_eq!(
            json["resource"]["aws_vpc"]["main"]["enable_dns_support"],
            true
        );
    }

    #[test]
    fn lava_runtime_accepts_scalar_overrides_via_bindings() {
        let rt = LavaRuntime::new();
        let mut bindings = IndexMap::new();
        bindings.insert(
            "cidr".to_string(),
            ArtifactBinding::Scalar("172.16.0.0/12".to_string()),
        );
        let input = ArtifactInput {
            source: vpc_tlisp().to_string(),
            bindings,
            name: None,
        };
        let result = rt.evaluate(&input).unwrap();
        let json = result.architecture.render_terraform_json().unwrap();
        assert_eq!(
            json["resource"]["aws_vpc"]["main"]["cidr_block"],
            "172.16.0.0/12"
        );
    }

    #[test]
    fn auto_detect_picks_lava_runtime_for_tlisp() {
        let p = std::path::Path::new("/tmp/foo/bar.tlisp");
        let rt = pick_runtime_for_path(p).unwrap();
        assert_eq!(rt.kind(), "lava");
    }

    #[test]
    fn auto_detect_picks_terraform_runtime_for_json() {
        let p = std::path::Path::new("/tmp/main.tf.json");
        let rt = pick_runtime_for_path(p).unwrap();
        assert_eq!(rt.kind(), "terraform-json");
    }

    #[test]
    fn auto_detect_returns_none_for_unknown_extension() {
        let p = std::path::Path::new("/tmp/main.yaml");
        assert!(pick_runtime_for_path(p).is_none());
    }
}
