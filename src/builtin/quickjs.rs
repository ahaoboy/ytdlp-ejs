//! QuickJS JS Challenge Provider

use crate::provider::JsChallengeError;
use crate::trace::{debug, info};
use rquickjs::{Context, Function, Object, Runtime};

/// QuickJS-based JavaScript Challenge Provider
pub struct QuickJSJCP {
    context: Context,
}

impl QuickJSJCP {
    pub fn new(code: &str) -> Result<Self, JsChallengeError> {
        info!("Creating QuickJS runtime");
        let runtime = Runtime::new()
            .map_err(|e| JsChallengeError::Runtime(format!("Failed to create runtime: {}", e)))?;
        let context = Context::full(&runtime)
            .map_err(|e| JsChallengeError::Runtime(format!("Failed to create context: {}", e)))?;

        context.with(|ctx| {
            let globals = ctx.globals();
            let result_obj = Object::new(ctx.clone()).map_err(|e| {
                JsChallengeError::Runtime(format!("Failed to create object: {}", e))
            })?;
            globals
                .set("_result", result_obj)
                .map_err(|e| JsChallengeError::Runtime(format!("Failed to set _result: {}", e)))?;

            debug!(
                code_len = code.len(),
                "Evaluating preprocessed code in QuickJS"
            );
            ctx.eval::<(), _>(code).map_err(|e| {
                let err_msg = match &e {
                    rquickjs::Error::Exception => {
                        let exc = ctx.catch();
                        if exc.is_null() || exc.is_undefined() {
                            "Exception (no details)".to_string()
                        } else {
                            format!("Exception: {:?}", exc)
                        }
                    }
                    _ => format!("{:?}", e),
                };
                JsChallengeError::Runtime(format!("Failed to execute: {}", err_msg))
            })?;

            info!("QuickJS code evaluation complete");
            Ok::<(), JsChallengeError>(())
        })?;

        Ok(Self { context })
    }

    pub fn solve_n(&self, challenge: &str) -> Result<String, JsChallengeError> {
        self.call_solver("n", challenge)
    }

    pub fn solve_sig(&self, challenge: &str) -> Result<String, JsChallengeError> {
        self.call_solver("sig", challenge)
    }

    fn call_solver(&self, func_name: &str, challenge: &str) -> Result<String, JsChallengeError> {
        self.context.with(|ctx| {
            debug!(%func_name, %challenge, "Calling solver");
            let globals = ctx.globals();
            let result: Object = globals
                .get("_result")
                .map_err(|e| JsChallengeError::Runtime(format!("Failed to get _result: {}", e)))?;
            let func: Function = result.get(func_name).map_err(|e| {
                JsChallengeError::Runtime(format!("Failed to get {} function: {}", func_name, e))
            })?;

            let result: String = func.call((challenge,)).map_err(|e| {
                let err_msg = match &e {
                    rquickjs::Error::Exception => {
                        let exc = ctx.catch();
                        if exc.is_null() || exc.is_undefined() {
                            "Exception (no details)".to_string()
                        } else {
                            format!("Exception: {:?}", exc)
                        }
                    }
                    _ => format!("{:?}", e),
                };
                JsChallengeError::Runtime(format!("Failed to call {}: {}", func_name, err_msg))
            })?;
            debug!(%func_name, %challenge, result_len = result.len(), result, "Solver returned");
            Ok(result)
        })
    }
}
