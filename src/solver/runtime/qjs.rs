use super::JsRuntime;
use rquickjs::{Context, Function, Object, Runtime};

/// QuickJS-based JavaScript solver
pub struct QuickJsSolver {
    context: Context,
}

impl JsRuntime for QuickJsSolver {
    fn from_prepared(code: &str) -> Result<Self, String> {
        let runtime = Runtime::new().map_err(|e| format!("Failed to create runtime: {}", e))?;
        let context =
            Context::full(&runtime).map_err(|e| format!("Failed to create context: {}", e))?;

        let code_owned = code.to_string();
        context.with(|ctx| {
            let globals = ctx.globals();
            let result_obj =
                Object::new(ctx.clone()).map_err(|e| format!("Failed to create object: {}", e))?;
            globals
                .set("_result", result_obj)
                .map_err(|e| format!("Failed to set _result: {}", e))?;

            ctx.eval::<(), _>(code_owned.as_str()).map_err(|e| {
                // Try to get more detailed error information
                let err_msg = match &e {
                    rquickjs::Error::Exception => {
                        // Try to get the exception details
                        let exc = ctx.catch();
                        if exc.is_null() || exc.is_undefined() {
                            "Exception (no details available)".to_string()
                        } else {
                            format!("Exception: {:?}", exc)
                        }
                    }
                    _ => format!("{:?}", e),
                };
                format!("Failed to execute code: {}", err_msg)
            })?;

            Ok::<(), String>(())
        })?;

        Ok(Self { context })
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

impl QuickJsSolver {
    fn call_solver(&self, func_name: &str, challenge: &str) -> Result<String, String> {
        let challenge_owned = challenge.to_string();
        let func_name_owned = func_name.to_string();

        self.context.with(|ctx| {
            let globals = ctx.globals();
            let result: Object = globals
                .get("_result")
                .map_err(|e| format!("Failed to get _result: {}", e))?;
            let func: Function = result
                .get(func_name_owned.as_str())
                .map_err(|e| format!("Failed to get {} function: {}", func_name_owned, e))?;

            let result: String = func.call((challenge_owned.as_str(),)).map_err(|e| {
                let err_msg = match &e {
                    rquickjs::Error::Exception => {
                        let exc = ctx.catch();
                        if exc.is_null() || exc.is_undefined() {
                            "Exception (no details available)".to_string()
                        } else {
                            format!("Exception: {:?}", exc)
                        }
                    }
                    _ => format!("{:?}", e),
                };
                format!("Failed to call {} function: {}", func_name_owned, err_msg)
            })?;
            Ok(result)
        })
    }

    fn has_solver(&self, func_name: &str) -> bool {
        let func_name_owned = func_name.to_string();
        self.context.with(|ctx| {
            let globals = ctx.globals();
            if let Ok(result) = globals.get::<_, Object>("_result") {
                result.get::<_, Function>(func_name_owned.as_str()).is_ok()
            } else {
                false
            }
        })
    }
}
