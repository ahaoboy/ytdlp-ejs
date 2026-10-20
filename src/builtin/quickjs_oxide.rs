//! quickjs-oxide JS Challenge Provider
//!
//! The same interface as the `qjs` provider, backed by the pure-Rust
//! [quickjs-oxide](https://crates.io/crates/quickjs-oxide) engine.

use quickjs_oxide::engine::api::core::{Context, JsString, Runtime, RuntimeError, Value};
use quickjs_oxide_host::SystemHost;

use crate::provider::JsChallengeError;
use crate::runtime::ChallengeSolver;
use crate::trace::{debug, info};

/// quickjs-oxide-based JavaScript Challenge Provider
pub struct QuickJsOxideJCP {
    context: Context,
}

impl QuickJsOxideJCP {
    pub fn new(code: &str) -> Result<Self, JsChallengeError> {
        info!("Creating quickjs-oxide runtime");
        let runtime = Runtime::new_with_host_services(SystemHost::default());
        let context = runtime
            .new_context()
            .map_err(|e| JsChallengeError::Runtime(format!("Failed to create context: {}", e)))?;

        // The preprocessed player assigns its solvers to `_result.n` and
        // `_result.sig`, so the object has to exist before it runs.
        let global = context.global_object().map_err(|e| {
            JsChallengeError::Runtime(format!("Failed to get global object: {}", e))
        })?;
        let result_obj = context
            .new_object()
            .map_err(|e| JsChallengeError::Runtime(format!("Failed to create object: {}", e)))?;
        let result_key = runtime.intern_property_key("_result").map_err(|e| {
            JsChallengeError::Runtime(format!("Failed to create _result key: {}", e))
        })?;
        if let Err(error) = context.set_property(&global, &result_key, Value::Object(result_obj)) {
            return Err(runtime_error(&context, error, "Failed to set _result"));
        }

        debug!(
            code_len = code.len(),
            "Evaluating preprocessed code in quickjs-oxide"
        );
        if let Err(error) = context.eval(code) {
            return Err(runtime_error(&context, error, "Failed to execute"));
        }

        info!("quickjs-oxide code evaluation complete");
        Ok(Self { context })
    }

    fn call_solver(&self, func_name: &str, challenge: &str) -> Result<String, JsChallengeError> {
        debug!(%func_name, %challenge, "Calling solver");

        // `Context` is a cheap handle to the provider's realm.
        let context = self.context.clone();
        let runtime = context.runtime().clone();
        let global = context.global_object().map_err(|e| {
            JsChallengeError::Runtime(format!("Failed to get global object: {}", e))
        })?;

        let result_key = runtime.intern_property_key("_result").map_err(|e| {
            JsChallengeError::Runtime(format!("Failed to create _result key: {}", e))
        })?;
        let result = context
            .get_property(&global, &result_key)
            .map_err(|e| JsChallengeError::Runtime(format!("Failed to get _result: {}", e)))?;
        let Value::Object(result_obj) = result else {
            return Err(JsChallengeError::Runtime("_result is not an object".into()));
        };

        let func_key = runtime.intern_property_key(func_name).map_err(|e| {
            JsChallengeError::Runtime(format!("Failed to create {} key: {}", func_name, e))
        })?;
        let function = context.get_property(&result_obj, &func_key).map_err(|e| {
            JsChallengeError::Runtime(format!("Failed to get {} function: {}", func_name, e))
        })?;
        let Value::Object(function_obj) = function else {
            return Err(JsChallengeError::Runtime(format!(
                "{} is not a function",
                func_name
            )));
        };
        let Some(function) = context.as_callable(&function_obj).map_err(|e| {
            JsChallengeError::Runtime(format!("Failed to read {} function: {}", func_name, e))
        })?
        else {
            return Err(JsChallengeError::Runtime(format!(
                "{} is not a function",
                func_name
            )));
        };

        let argument = Value::String(JsString::try_from_utf8(challenge).map_err(|e| {
            JsChallengeError::Runtime(format!("Failed to convert challenge: {}", e))
        })?);
        let value = context
            .call(&function, Value::Undefined, &[argument])
            .map_err(|e| runtime_error(&context, e, &format!("Failed to call {}", func_name)))?;
        let result = context.to_js_string(&value).map_err(|e| {
            runtime_error(
                &context,
                e,
                &format!("Failed to convert {} result", func_name),
            )
        })?;
        let result = result.to_utf8_lossy();

        debug!(%func_name, %challenge, result_len = result.len(), result, "Solver returned");
        Ok(result)
    }
}

/// Formats an engine error, including the value of an uncaught exception.
fn runtime_error(context: &Context, error: RuntimeError, action: &str) -> JsChallengeError {
    let detail = match error {
        RuntimeError::Exception(exception) => {
            let text = context
                .print_value(&exception)
                .unwrap_or_else(|_| format!("{:?}", exception));
            format!("Exception: {}", text)
        }
        error => error.to_string(),
    };
    JsChallengeError::Runtime(format!("{}: {}", action, detail))
}

impl ChallengeSolver for QuickJsOxideJCP {
    fn solve_n(&mut self, challenge: &str) -> Result<String, JsChallengeError> {
        self.call_solver("n", challenge)
    }

    fn solve_sig(&mut self, challenge: &str) -> Result<String, JsChallengeError> {
        self.call_solver("sig", challenge)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::ChallengeSolver;

    #[test]
    fn solves_challenges() {
        let mut provider = QuickJsOxideJCP::new(
            "_result.n = function (s) { return s + '!'; }; \
             _result.sig = function (s) { return s.split('').reverse().join(''); };",
        )
        .expect("provider");
        assert_eq!(provider.solve_n("x").expect("n"), "x!");
        assert_eq!(provider.solve_sig("abc").expect("sig"), "cba");
    }

    #[test]
    fn reports_uncaught_exceptions() {
        let mut provider =
            QuickJsOxideJCP::new("_result.n = function () { throw new Error('boom'); };")
                .expect("provider");
        let error = provider.solve_n("x").expect_err("solver throws");
        assert!(error.to_string().contains("boom"), "{error}");
    }
}
