use std::collections::HashMap;

#[cfg(feature = "qjs")]
pub mod qjs;

pub mod deno;

#[cfg(feature = "boa")]
pub mod boa;

pub mod node;

pub mod bun;

/// Runtime type for JavaScript execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RuntimeType {
    #[default]
    Deno,
    #[cfg(feature = "qjs")]
    QuickJS,
    #[cfg(feature = "boa")]
    Boa,
    Node,
    Bun,
}

impl RuntimeType {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            #[cfg(feature = "qjs")]
            "qjs" | "quickjs" => Some(Self::QuickJS),
            "deno" => Some(Self::Deno),
            #[cfg(feature = "boa")]
            "boa" => Some(Self::Boa),
            "node" | "nodejs" => Some(Self::Node),
            "bun" => Some(Self::Bun),
            _ => None,
        }
    }

    #[allow(clippy::vec_init_then_push)]
    pub fn available_runtimes() -> Vec<&'static str> {
        let mut runtimes = Vec::new();
        #[cfg(feature = "qjs")]
        runtimes.push("qjs");
        runtimes.push("deno");
        #[cfg(feature = "boa")]
        runtimes.push("boa");
        runtimes.push("node");
        runtimes.push("bun");
        runtimes
    }
}

/// Trait for JavaScript runtime implementations
pub trait JsRuntime {
    fn from_prepared(code: &str) -> Result<Self, String>
    where
        Self: Sized;

    fn solve_n(&self, challenge: &str) -> Result<String, String>;
    fn solve_sig(&self, challenge: &str) -> Result<String, String>;
    fn has_n(&self) -> bool;
    fn has_sig(&self) -> bool;

    fn solve_challenges(
        &self,
        req_type: &str,
        challenges: &[String],
    ) -> Result<HashMap<String, String>, String> {
        let mut results = HashMap::new();
        for challenge in challenges {
            let result = match req_type {
                "n" => self.solve_n(challenge)?,
                "sig" => self.solve_sig(challenge)?,
                _ => return Err(format!("Unknown request type: {}", req_type)),
            };
            results.insert(challenge.clone(), result);
        }
        Ok(results)
    }
}

/// Create a solver with the specified runtime
pub fn create_solver(runtime_type: RuntimeType, code: &str) -> Result<Box<dyn JsRuntime>, String> {
    match runtime_type {
        #[cfg(feature = "qjs")]
        RuntimeType::QuickJS => Ok(Box::new(qjs::QuickJsSolver::from_prepared(code)?)),
        RuntimeType::Deno => Ok(Box::new(deno::DenoSolver::from_prepared(code)?)),
        #[cfg(feature = "boa")]
        RuntimeType::Boa => Ok(Box::new(boa::BoaSolver::from_prepared(code)?)),
        RuntimeType::Node => Ok(Box::new(node::NodeSolver::from_prepared(code)?)),
        RuntimeType::Bun => Ok(Box::new(bun::BunSolver::from_prepared(code)?)),
    }
}
