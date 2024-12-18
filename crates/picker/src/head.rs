use std::sync::Arc;

use editor::{Editor, EditorEvent};
use gpui::{prelude::*, AppContext, FocusHandle, FocusableView, View};
use ui::prelude::*;

/// The head of a [`Picker`](crate::Picker).
pub(crate) enum Head {
    /// Picker has an editor that allows the user to filter the list.
    Editor(View<Editor>),

    /// Picker has no head, it's just a list of items.
    Empty(View<EmptyHead>),
}

impl Head {
    pub fn editor<V: 'static>(
        placeholder_text: Arc<str>,
        mut edit_handler: impl FnMut(&mut V, View<Editor>, &EditorEvent, &mut Window, &mut ViewContext<'_, V>)
            + 'static,
        window: &Window,
        cx: &mut ViewContext<V>,
    ) -> Self {
        let editor = cx.new_view(|cx| {
            let mut editor = Editor::single_line(cx);
            editor.set_placeholder_text(placeholder_text, cx);
            editor
        });

        let window_handle = window.handle();

        cx.subscribe(&editor, move |view, editor, event, cx| {
            window_handle.update(cx, |_, window, cx| {
                edit_handler(view, editor, event, window, todo!());
            });
        })
        .detach();
        Self::Editor(editor)
    }

    pub fn empty<V: 'static>(
        blur_handler: impl FnMut(&mut V, &mut Window, &mut ViewContext<'_, V>) + 'static,
        cx: &mut ViewContext<V>,
    ) -> Self {
        let head = cx.new_view(EmptyHead::new);
        cx.on_blur(&head.focus_handle(cx), blur_handler).detach();
        Self::Empty(head)
    }
}

/// An invisible element that can hold focus.
pub(crate) struct EmptyHead {
    focus_handle: FocusHandle,
}

impl EmptyHead {
    fn new(cx: &mut ViewContext<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Render for EmptyHead {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div().track_focus(&self.focus_handle(cx))
    }
}

impl FocusableView for EmptyHead {
    fn focus_handle(&self, _: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}
