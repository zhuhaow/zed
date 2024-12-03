mod code_edits_tool;
pub mod patch;

use assistant_tool::ToolRegistry;
use gpui::AppContext;

pub use crate::edits::code_edits_tool::CodeEditsTool;

pub fn init(cx: &mut AppContext) {
    let tool_registry = ToolRegistry::global(cx);
    tool_registry.register_tool(CodeEditsTool);
}
