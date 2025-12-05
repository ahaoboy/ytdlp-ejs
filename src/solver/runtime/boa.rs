use super::JsRuntime;
use boa_engine::object::ObjectInitializer;
use boa_engine::property::Attribute;
use boa_engine::{Context, Source, js_string};
use std::cell::RefCell;

/// Boa-based JavaScript solver
pub struct BoaSolver {
    context: RefCell<Context>,
}

impl JsRuntime for BoaSolver {
    fn from_prepared(code: &str) -> Result<Self, String> {
        let mut context = Context::default();

        // Create _result object
        let result_obj = ObjectInitializer::new(&mut context).build();
        context
            .register_global_property(js_string!("_result"), result_obj, Attribute::all())
            .map_err(|e| format!("Failed to register _result: {}", e))?;

        // Execute the preprocessed code
        context
            .eval(Source::from_bytes(code))
            .map_err(|e| format!("Failed to execute code: {}", e))?;

        Ok(Self {
            context: RefCell::new(context),
        })
    }

    fn solve_n(&self, challenge: &str) -> Result<String, String> {
        self.call_solver("n", challenge)
    }

    fn solve_sig(&self, challenge: &str) -> Result<String, String> {
        self.call_solver("sig", challenge)
    }

    fn has_n(&self) -> bool {
        self.has_solver("n")
    }

    fn has_sig(&self) -> bool {
        self.has_solver("sig")
    }
}

impl BoaSolver {
    fn call_solver(&self, func_name: &str, challenge: &str) -> Result<String, String> {
        let mut context = self.context.borrow_mut();

        // Build the call expression: _result.n("challenge") or _result.sig("challenge")
        let escaped_challenge = challenge.replace('\\', "\\\\").replace('"', "\\\"");
        let call_code = format!("_result.{}(\"{}\")", func_name, escaped_challenge);

        let result = context
            .eval(Source::from_bytes(&call_code))
            .map_err(|e| format!("Failed to call {} function: {}", func_name, e))?;

        // Convert result to string
        result
            .to_string(&mut context)
            .map(|s| s.to_std_string_escaped())
            .map_err(|e| format!("Failed to convert result to string: {}", e))
    }

    fn has_solver(&self, func_name: &str) -> bool {
        let mut context = self.context.borrow_mut();
        let check_code = format!("typeof _result.{} === 'function'", func_name);

        context
            .eval(Source::from_bytes(&check_code))
            .map(|v| v.as_boolean().unwrap_or(false))
            .unwrap_or(false)
    }
}
