use std::sync::Arc;

use anyhow::{anyhow, Result};
use assistant_tool::Tool;
use gpui::{Task, WeakView, WindowContext};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CodeEditsToolInput {
    /// A high-level description of the code changes. This should be as short as possible, possibly using common abbreviations.
    pub title: String,
    /// An array of edits to be applied.
    pub edits: Vec<Edit>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Edit {
    /// The path to the file that this edit will change.
    pub path: String,
    /// The type of change that should occur at the given range of the file.
    pub operation: Operation,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Operation {
    /// Replaces the entire range with the new text.
    Update {
        /// An excerpt from the file's current contents that uniquely identifies a range within the file where the edit should occur.
        old_text: String,
        /// The new text to insert into the file.
        new_text: String,
        /// An arbitrarily-long comment that describes the purpose of this edit.
        description: Option<String>,
    },
    /// Inserts the new text before the range.
    InsertBefore {
        /// An excerpt from the file's current contents that uniquely identifies a range within the file where the edit should occur.
        old_text: String,
        /// The new text to insert into the file.
        new_text: String,
        /// An arbitrarily-long comment that describes the purpose of this edit.
        description: Option<String>,
    },
    /// Inserts new text after the range.
    InsertAfter {
        /// An excerpt from the file's current contents that uniquely identifies a range within the file where the edit should occur.
        old_text: String,
        /// The new text to insert into the file.
        new_text: String,
        /// An arbitrarily-long comment that describes the purpose of this edit.
        description: Option<String>,
    },
    /// Creates a new file with the given path and the new text.
    Create {
        /// An arbitrarily-long comment that describes the purpose of this edit.
        description: Option<String>,
        /// The new text to insert into the file.
        new_text: String,
    },
    /// Deletes the specified range from the file.
    Delete {
        /// An excerpt from the file's current contents that uniquely identifies a range within the file where the edit should occur.
        old_text: String,
    },
}

pub struct CodeEditsTool;

impl CodeEditsTool {
    pub const TOOL_NAME: &str = "zed_code_edits";
}

impl Tool for CodeEditsTool {
    fn name(&self) -> String {
        Self::TOOL_NAME.to_string()
    }

    fn description(&self) -> String {
        // Anthropic's best practices for tool descriptions:
        // https://docs.anthropic.com/en/docs/build-with-claude/tool-use#best-practices-for-tool-definitions
        include_str!("code_edits_tool_description.txt").to_string()
    }

    fn input_schema(&self) -> serde_json::Value {
        let schema = schemars::schema_for!(CodeEditsToolInput);

        serde_json::to_value(&schema).unwrap()
    }

    fn run(
        self: Arc<Self>,
        input: serde_json::Value,
        _workspace: WeakView<workspace::Workspace>,
        _cx: &mut WindowContext,
    ) -> Task<Result<String>> {
        let input: CodeEditsToolInput = match serde_json::from_value(input) {
            Ok(input) => input,
            Err(err) => return Task::ready(Err(anyhow!(err))),
        };

        Task::ready(serde_json::to_string(&input).map_err(|err| anyhow!(err)))
    }
}
