//! Boa JS Challenge Provider

use crate::provider::JsChallengeError;
use crate::trace::{debug, info};
use boa_engine::object::ObjectInitializer;
use boa_engine::property::Attribute;
use boa_engine::{Context, Source, js_string};

/// Boa-based JavaScript Challenge Provider
pub struct BoaJCP {
    context: Context,
}

impl BoaJCP {
    pub fn new(code: &str) -> Result<Self, JsChallengeError> {
        info!("Creating Boa runtime");
        let mut context = Context::default();

        let result_obj = ObjectInitializer::new(&mut context).build();
        context
            .register_global_property(js_string!("_result"), result_obj, Attribute::all())
            .map_err(|e| JsChallengeError::Runtime(format!("Failed to register _result: {}", e)))?;

        debug!(code_len = code.len(), "Evaluating preprocessed code in Boa");
        context
            .eval(Source::from_bytes(code))
            .map_err(|e| JsChallengeError::Runtime(format!("Failed to execute: {}", e)))?;

        info!("Boa code evaluation complete");
        Ok(Self { context })
    }

    pub fn solve_n(&mut self, challenge: &str) -> Result<String, JsChallengeError> {
        self.call_solver("n", challenge)
    }

    pub fn solve_sig(&mut self, challenge: &str) -> Result<String, JsChallengeError> {
        self.call_solver("sig", challenge)
    }

    fn call_solver(
        &mut self,
        func_name: &str,
        challenge: &str,
    ) -> Result<String, JsChallengeError> {
        debug!(%func_name, %challenge, "Calling solver in Boa");
        let escaped = challenge.replace('\\', "\\\\").replace('"', "\\\"");
        let call_code = format!("_result.{}(\"{}\")", func_name, escaped);

        let result = self
            .context
            .eval(Source::from_bytes(&call_code))
            .map_err(|e| {
                JsChallengeError::Runtime(format!("Failed to call {}: {}", func_name, e))
            })?;

        result
            .to_string(&mut self.context)
            .map(|s| {
                let result = s.to_std_string_escaped();
                debug!(%func_name, %challenge, %result, "Boa solver returned");
                result
            })
            .map_err(|e| JsChallengeError::Runtime(format!("Failed to convert result: {}", e)))
    }
}
