//! Runtime solver trait.
//!
//! Each backend implements [`ChallengeSolver`] for its provider type. The
//! provider enum in [`crate::registry`] dispatches to them statically.

use std::collections::HashMap;

use crate::provider::{JsChallengeError, JsChallengeType};

/// A stateful solver created from preprocessed player code.
pub trait ChallengeSolver {
    fn solve_n(&mut self, challenge: &str) -> Result<String, JsChallengeError>;
    fn solve_sig(&mut self, challenge: &str) -> Result<String, JsChallengeError>;

    fn solve_challenges(
        &mut self,
        req_type: JsChallengeType,
        challenges: &[String],
    ) -> Result<HashMap<String, String>, JsChallengeError> {
        let mut results = HashMap::with_capacity(challenges.len());
        for challenge in challenges {
            let result = match req_type {
                JsChallengeType::N => self.solve_n(challenge)?,
                JsChallengeType::Sig => self.solve_sig(challenge)?,
            };
            results.insert(challenge.clone(), result);
        }
        Ok(results)
    }
}
