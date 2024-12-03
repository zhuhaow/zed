mod code_edits_tool;
mod now_tool;

use assistant_tool::ToolRegistry;
use gpui::AppContext;

use crate::code_edits_tool::CodeEditsTool;
use crate::now_tool::NowTool;

pub fn init(cx: &mut AppContext) {
    assistant_tool::init(cx);

    let registry = ToolRegistry::global(cx);
    registry.register_tool(NowTool);
    registry.register_tool(CodeEditsTool);
}
