use super::repository::InstructionRepository;
use crate::error::{Result, SupportBotError};
use crate::tools::{
    SupportTool, ToolCall, ToolContext, ToolExecutionOutcome, ToolKind, ToolResult, ToolSpec,
};
use async_trait::async_trait;
use serde::Deserialize;

#[async_trait]
impl SupportTool for InstructionRepository {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "instructions".to_string(),
            description: "Load support runbooks/instructions by Markdown document id. Omit id to load /index, then follow links like [/runbooks/example].".to_string(),
            kind: ToolKind::ReadOnly,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": ["string", "null"],
                        "description": "Instruction document id. Omit or null to load /index."
                    }
                },
                "additionalProperties": false
            }),
        }
    }

    async fn call(&self, _ctx: ToolContext, call: ToolCall) -> Result<ToolExecutionOutcome> {
        let call_id = call.id;
        let args: InstructionToolArgs =
            serde_json::from_value(call.arguments).map_err(|error| {
                SupportBotError::Tool(format!("invalid instructions arguments: {error}"))
            })?;
        let id = args
            .id
            .as_deref()
            .map(str::trim)
            .filter(|id| !id.is_empty());

        let (content, is_error) = match self.get_document(id)? {
            Some(loaded) => (
                serde_json::json!({
                    "document": {
                        "id": loaded.document.id,
                        "title": loaded.document.title
                    },
                    "content": loaded.content
                }),
                false,
            ),
            None => (
                serde_json::json!({
                    "missing": id.unwrap_or("/index")
                }),
                true,
            ),
        };

        Ok(ToolExecutionOutcome::ToolResult(ToolResult {
            call_id,
            content,
            is_error,
        }))
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct InstructionToolArgs {
    #[serde(default)]
    id: Option<String>,
}
