use crate::Result;
use async_trait::async_trait;
use thread_bot::Thread;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SupportThreadAdmissionDecision {
    Accept,
    Ignore { reason: Option<String> },
}

#[async_trait]
pub trait SupportThreadAdmissionHook: Send + Sync {
    async fn evaluate(&self, thread: &Thread) -> Result<SupportThreadAdmissionDecision>;
}

#[derive(Debug, Clone)]
pub struct FirstMessageTextAdmissionHook {
    required_substrings: Vec<String>,
}

impl FirstMessageTextAdmissionHook {
    pub fn new(required_substrings: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            required_substrings: required_substrings.into_iter().map(Into::into).collect(),
        }
    }

    pub fn required_substrings(&self) -> &[String] {
        &self.required_substrings
    }

    fn first_message<'a>(&self, thread: &'a Thread) -> Option<&'a str> {
        thread
            .messages
            .iter()
            .find(|message| message.post_id == thread.info.root_post_id)
            .or_else(|| {
                thread
                    .messages
                    .iter()
                    .min_by_key(|message| message.created_at)
            })
            .map(|message| message.message.as_str())
    }
}

#[async_trait]
impl SupportThreadAdmissionHook for FirstMessageTextAdmissionHook {
    async fn evaluate(&self, thread: &Thread) -> Result<SupportThreadAdmissionDecision> {
        if self.required_substrings.is_empty() {
            return Ok(SupportThreadAdmissionDecision::Accept);
        }

        let first_message = self.first_message(thread).unwrap_or_default();
        let missing = self
            .required_substrings
            .iter()
            .filter(|required| !first_message.contains(required.as_str()))
            .cloned()
            .collect::<Vec<_>>();

        if missing.is_empty() {
            Ok(SupportThreadAdmissionDecision::Accept)
        } else {
            Ok(SupportThreadAdmissionDecision::Ignore {
                reason: Some(format!("missing required text: {}", missing.join(", "))),
            })
        }
    }
}

#[cfg(test)]
#[path = "tests/admission.rs"]
mod tests;
