use anyhow::Result;
use std::ops::Range;

use language::{Anchor, BufferSnapshot, EditBehaviorProvider};

pub struct TsxEditBehaviorProvider;

pub struct TsxTagCompletionState {
    edit_index: usize,
    open_tag_range: Range<usize>,
}

impl EditBehaviorProvider for TsxEditBehaviorProvider {
    type AutoEditState = Vec<TsxTagCompletionState>;

    fn should_auto_edit(
        &self,
        buffer: &BufferSnapshot,
        edited_ranges: &[Range<usize>],
    ) -> Option<Self::AutoEditState> {
        let mut to_auto_edit = vec![];
        for (index, edited_range) in edited_ranges.iter().enumerate() {
            let text = buffer
                .text_for_range(edited_range.clone())
                .collect::<String>();
            if dbg!(!text.ends_with(">")) {
                continue;
            }
            let Some(layer) = dbg!(buffer.syntax_layer_at(edited_range.start)) else {
                continue;
            };
            let language_name = dbg!(layer.language.name());
            if dbg!(
                !(language_name.as_ref().eq_ignore_ascii_case("jsx")
                    || language_name.as_ref().eq_ignore_ascii_case("tsx"))
            ) {
                continue;
            }
            dbg!(layer.node().to_sexp());
            // todo! if buffer.settings_at
            let Some(node) = dbg!(layer
                .node()
                .descendant_for_byte_range(edited_range.start, edited_range.end))
            else {
                continue;
            };
            let mut jsx_open_tag_node = node;
            if node.grammar_name() != "jsx_opening_element" {
                if let Some(parent) = node.parent() {
                    if parent.grammar_name() == "jsx_opening_element" {
                        jsx_open_tag_node = parent;
                    }
                }
            }
            if dbg!(jsx_open_tag_node.grammar_name()) != "jsx_opening_element" {
                continue;
            }

            if jsx_open_tag_node.has_error() {
                let mut chars = buffer
                    .text_for_range(jsx_open_tag_node.byte_range())
                    .flat_map(|chunk| chunk.chars());
                if chars.next() == Some('<') && chars.next() == Some('/') {
                    continue;
                }
            }

            to_auto_edit.push(TsxTagCompletionState {
                edit_index: index,
                open_tag_range: jsx_open_tag_node.byte_range(),
            });

            dbg!(node.parent());
        }
        dbg!(&to_auto_edit.len());
        if to_auto_edit.is_empty() {
            return None;
        } else {
            return Some(to_auto_edit);
        }
        // dbg!(edited_ranges
        //     .iter()
        //     .map(|range| (
        //         range,
        //         buffer.text_for_range(range.clone()).collect::<String>()
        //     ))
        //     .collect::<Vec<_>>());
        // None
    }

    fn auto_edit(
        &self,
        buffer: BufferSnapshot,
        ranges: &[Range<usize>],
        state: Self::AutoEditState,
    ) -> Result<Vec<(Range<Anchor>, String)>> {
        let mut edits = Vec::with_capacity(state.len());
        for auto_edit in state {
            let edited_range = ranges[auto_edit.edit_index].clone();
            let Some(layer) = buffer.syntax_ancestor(edited_range.clone()) else {
                continue;
            };
            let Some(open_tag) = layer.descendant_for_byte_range(
                auto_edit.open_tag_range.start,
                auto_edit.open_tag_range.end,
            ) else {
                continue;
            };
            assert!(open_tag.grammar_name() == "jsx_opening_element");
            let tag_name_range = open_tag
                .child_by_field_name("name")
                .map_or(0..0, |node| node.byte_range());

            let tag_name = buffer.text_for_range(tag_name_range).collect::<String>();
            dbg!(&tag_name);
            {
                let mut tree_root_node = open_tag;
                // todo! child_with_descendant
                while let Some(parent) = tree_root_node.parent() {
                    tree_root_node = parent;
                    if parent.is_error()
                        || (parent.kind() != "jsx_element"
                            && parent.kind() != "jsx_opening_element")
                    {
                        break;
                    }
                }

                dbg!(tree_root_node);

                let mut unclosed_open_tag_count: i32 = 0;

                let mut stack = Vec::with_capacity(tree_root_node.descendant_count());
                stack.push(tree_root_node);

                let mut cursor = tree_root_node.walk();

                // todo! use cursor for more efficient traversal
                // if child -> go to child
                // else if next sibling -> go to next sibling
                // else -> go to parent
                // if parent == tree_root_node -> break
                while let Some(node) = stack.pop() {
                    if node.kind() == "jsx_opening_element" {
                        if node.child_by_field_name("name").map_or(false, |node| {
                            buffer
                                .text_for_range(node.byte_range())
                                .equals_str(&tag_name)
                        }) {
                            dbg!("found open");
                            unclosed_open_tag_count += 1;
                        }
                        continue;
                    } else if node.kind() == "jsx_closing_element" {
                        if node.child_by_field_name("name").map_or(false, |node| {
                            buffer
                                .text_for_range(node.byte_range())
                                .equals_str(&tag_name)
                        }) {
                            dbg!("found close");
                            unclosed_open_tag_count -= 1;
                        }
                        continue;
                    } else if node.kind() == "jsx_self_closing_element" {
                        // don't recurse into jsx self-closing elements
                        continue;
                    } else if node.kind() == "jsx_expression" {
                        // don't recurse into jsx expressions (it forms a new scope)
                        continue;
                    }

                    stack.extend(node.children(&mut cursor));
                }

                if unclosed_open_tag_count <= 0 {
                    // skip if already closed
                    continue;
                }
            }
            let edit_anchor = buffer.anchor_after(edited_range.end);
            let edit_range = edit_anchor..edit_anchor;
            edits.push((edit_range, format!("</{}>", tag_name)));
        }
        return Ok(edits);
        // Ok(vec![])
    }
}
