//! TerraformJsonRuntime — passthrough for pre-rendered terraform.json.
//!
//! Magma already accepts terraform.json; this runtime turns it into a
//! typed `Architecture` so the same Plan flow consumes both lava-eval
//! .tlisp output AND legacy pangea-rendered terraform.json output
//! through ONE typed surface (no special-casing in magma).

use crate::{ArtifactInput, EmbeddedRuntime, EvaluationResult, RuntimeError};
use indexmap::IndexMap;
use lava_core::{Architecture, Resource, Value};

#[derive(Debug, Default, Clone)]
pub struct TerraformJsonRuntime;

impl TerraformJsonRuntime {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl EmbeddedRuntime for TerraformJsonRuntime {
    fn kind(&self) -> &'static str {
        "terraform-json"
    }

    fn extension(&self) -> &'static str {
        "tf.json"
    }

    fn evaluate(&self, input: &ArtifactInput) -> Result<EvaluationResult, RuntimeError> {
        let value: serde_json::Value = serde_json::from_str(&input.source)
            .map_err(|e| RuntimeError::Parse(format!("json: {e}")))?;
        let arch_name = input
            .name
            .clone()
            .unwrap_or_else(|| "imported-terraform".to_string());
        let mut arch = Architecture::new(arch_name);

        // Lift each `resource.<type>.<name>` entry into a typed Resource.
        if let Some(resource_section) = value.get("resource").and_then(|v| v.as_object()) {
            for (type_id, named_map) in resource_section {
                let Some(named) = named_map.as_object() else {
                    continue;
                };
                for (name, body) in named {
                    let mut attrs = IndexMap::new();
                    if let Some(body_obj) = body.as_object() {
                        for (k, v) in body_obj {
                            // from_json lifts nested structure into the typed
                            // tree and recovers `${type.name.attr}` strings as
                            // typed references — so an imported terraform.json
                            // arrives with its dependency graph intact rather
                            // than as opaque interpolation strings.
                            attrs.insert(k.clone(), Value::from_json(v.clone()));
                        }
                    }
                    arch.resources.push(Resource {
                        type_id: type_id.clone(),
                        name: name.clone(),
                        attributes: attrs,
                        depends_on: vec![],
                        provider: None,
                        multiplicity: None,
                    });
                }
            }
        }

        // Lift outputs too — preserves the consumer's typed surface.
        if let Some(output_section) = value.get("output").and_then(|v| v.as_object()) {
            for (k, body) in output_section {
                if let Some(v) = body.get("value") {
                    arch.outputs.insert(k.clone(), Value::from_json(v.clone()));
                }
            }
        }

        Ok(EvaluationResult {
            architecture: arch,
            diagnostics: vec![],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ArtifactInput;
    use indexmap::IndexMap;

    #[test]
    fn imports_pangea_rendered_terraform_json() {
        let json = r#"{
            "resource": {
                "aws_vpc": {
                    "main-vpc": {
                        "cidr_block": "10.0.0.0/16",
                        "enable_dns_support": true
                    }
                }
            },
            "output": {
                "vpc_id": { "value": "${aws_vpc.main-vpc.id}" }
            }
        }"#;
        let rt = TerraformJsonRuntime::new();
        let input = ArtifactInput {
            source: json.to_string(),
            bindings: IndexMap::new(),
            name: Some("imported".to_string()),
        };
        let result = rt.evaluate(&input).unwrap();
        assert_eq!(result.architecture.resources.len(), 1);
        let r = &result.architecture.resources[0];
        assert_eq!(r.type_id, "aws_vpc");
        assert_eq!(r.name, "main-vpc");
        assert_eq!(result.architecture.outputs.len(), 1);
        assert!(result.architecture.outputs.contains_key("vpc_id"));
    }
}
