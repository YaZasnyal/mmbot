use super::*;
use crate::acp::{AcpPrompt, AcpTurn};
use crate::admission::{SupportThreadAdmissionDecision, SupportThreadAdmissionHook};
use crate::config::SupportRouteConfig;
use async_trait::async_trait;

struct StaticAcp;

#[async_trait]
impl AcpRuntime for StaticAcp {
    async fn prompt(&self, _prompt: AcpPrompt) -> crate::Result<AcpTurn> {
        unreachable!()
    }
}

struct AcceptAdmissionHook;

#[async_trait]
impl SupportThreadAdmissionHook for AcceptAdmissionHook {
    async fn evaluate(
        &self,
        _thread: &thread_bot::Thread,
    ) -> crate::Result<SupportThreadAdmissionDecision> {
        Ok(SupportThreadAdmissionDecision::Accept)
    }
}

#[test]
fn builder_accepts_acp_runtime_and_admission_hook() {
    SupportBotBuilder::new(
        "support",
        SupportBotConfig {
            routes: SupportRouteConfig::default(),
        },
        Arc::new(StaticAcp),
    )
    .with_admission_hook(Arc::new(AcceptAdmissionHook))
    .build();
}
