use std::rc::Rc;

use gpui::{ClickEvent, FocusHandle};
use ui::{prelude::*, IconButtonShape, Tooltip};

use crate::context::{ContextKind, ContextSnapshot};
use crate::{AcceptSuggestedContext, RemoveFocusedContext};

#[derive(IntoElement)]
pub enum ContextPill {
    Added {
        context: ContextSnapshot,
        dupe_name: bool,
        focused: bool,
        on_click: Option<Rc<dyn Fn(&ClickEvent, &mut WindowContext)>>,
        on_remove: Option<Rc<dyn Fn(&ClickEvent, &mut WindowContext)>>,
        focus_handle: FocusHandle,
    },
    Suggested {
        name: SharedString,
        icon_path: Option<SharedString>,
        kind: ContextKind,
        focused: bool,
        on_click: Option<Rc<dyn Fn(&ClickEvent, &mut WindowContext)>>,
        focus_handle: FocusHandle,
    },
}

impl ContextPill {
    pub fn new_added(
        context: ContextSnapshot,
        dupe_name: bool,
        focused: bool,
        on_remove: Option<Rc<dyn Fn(&ClickEvent, &mut WindowContext)>>,
        focus_handle: FocusHandle,
    ) -> Self {
        Self::Added {
            context,
            dupe_name,
            on_remove,
            focused,
            on_click: None,
            focus_handle,
        }
    }

    pub fn new_suggested(
        name: SharedString,
        icon_path: Option<SharedString>,
        kind: ContextKind,
        focused: bool,
        focus_handle: FocusHandle,
    ) -> Self {
        Self::Suggested {
            name,
            icon_path,
            kind,
            focused,
            on_click: None,
            focus_handle,
        }
    }

    pub fn on_click(mut self, listener: Rc<dyn Fn(&ClickEvent, &mut WindowContext)>) -> Self {
        match &mut self {
            ContextPill::Added { on_click, .. } => {
                *on_click = Some(listener);
            }
            ContextPill::Suggested { on_click, .. } => {
                *on_click = Some(listener);
            }
        }
        self
    }

    pub fn id(&self) -> ElementId {
        match self {
            Self::Added { context, .. } => {
                ElementId::NamedInteger("context-pill".into(), context.id.0)
            }
            Self::Suggested { .. } => "suggested-context-pill".into(),
        }
    }

    pub fn icon(&self) -> Icon {
        match self {
            Self::Added { context, .. } => match &context.icon_path {
                Some(icon_path) => Icon::from_path(icon_path),
                None => Icon::new(context.kind.icon()),
            },
            Self::Suggested {
                icon_path: Some(icon_path),
                ..
            } => Icon::from_path(icon_path),
            Self::Suggested {
                kind,
                icon_path: None,
                ..
            } => Icon::new(kind.icon()),
        }
    }
}

impl RenderOnce for ContextPill {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let color = cx.theme().colors();

        let base_pill = h_flex()
            .id(self.id())
            .px_1()
            .pb(px(1.))
            .border_1()
            .rounded_md()
            .gap_1()
            .child(self.icon().size(IconSize::XSmall).color(Color::Muted));

        match self {
            ContextPill::Added {
                context,
                dupe_name,
                focused,
                on_remove,
                focus_handle,
                on_click,
            } => base_pill
                .bg(color.element_background)
                .border_color(if focused {
                    color.border_focused
                } else {
                    color.border.opacity(0.5)
                })
                // .pr(if on_remove.is_some() { px(2.) } else { px(4.) })
                // .pr_1()
                .child(
                    h_flex()
                        .id("context-data")
                        .gap_1()
                        .child(Label::new(context.name.clone()).size(LabelSize::Small))
                        .when_some(context.parent.as_ref(), |element, parent_name| {
                            if dupe_name {
                                element.child(
                                    Label::new(parent_name.clone())
                                        .size(LabelSize::XSmall)
                                        .color(Color::Muted),
                                )
                            } else {
                                element
                            }
                        })
                        .when_some(context.tooltip.clone(), |element, tooltip| {
                            element.tooltip(move |cx| Tooltip::text(tooltip.clone(), cx))
                        }),
                )
                .child(
                    div()
                        .w_3p5()
                        .when(focused, {
                            let focus_handle = focus_handle.clone();
                            move |element| {
                                element.children(
                                    ui::KeyBinding::for_action_in(
                                        &RemoveFocusedContext,
                                        &focus_handle,
                                        cx,
                                    )
                                    .map(|binding| binding.into_element()),
                                )
                            }
                        })
                        .when(!focused, |element| {
                            element.when_some(on_remove, |element, on_remove| {
                                let focus_handle = focus_handle.clone();
                                element.child(
                                    IconButton::new(("remove", context.id.0), IconName::Close)
                                        .shape(IconButtonShape::Square)
                                        .icon_size(IconSize::XSmall)
                                        .tooltip(move |cx| {
                                            Tooltip::for_action_in(
                                                "Remove Context",
                                                &RemoveFocusedContext,
                                                &focus_handle,
                                                cx,
                                            )
                                        })
                                        .on_click(move |event, cx| on_remove(event, cx)),
                                )
                            })
                        }),
                )
                .when_some(on_click, |element, on_click| {
                    element.on_click(move |event, cx| on_click(event, cx))
                }),
            ContextPill::Suggested {
                name,
                kind,
                focused,
                focus_handle,
                on_click,
                ..
            } => base_pill
                .cursor_pointer()
                .pr_1()
                .border_color(if focused {
                    color.border_focused
                } else {
                    color.border_variant.opacity(0.5)
                })
                .hover(|style| style.bg(color.element_hover.opacity(0.5)))
                .child(Label::new(name).size(LabelSize::Small).color(Color::Muted))
                .child(
                    div().px_0p5().child(
                        Label::new(match kind {
                            ContextKind::File => "Active Tab",
                            ContextKind::Thread
                            | ContextKind::Directory
                            | ContextKind::FetchedUrl => "Active",
                        })
                        .size(LabelSize::XSmall)
                        .color(Color::Muted),
                    ),
                )
                .when(focused, {
                    let focus_handle = focus_handle.clone();
                    move |element| {
                        element.children(
                            ui::KeyBinding::for_action_in(
                                &AcceptSuggestedContext,
                                &focus_handle,
                                cx,
                            )
                            .map(|binding| binding.into_element()),
                        )
                    }
                })
                .tooltip({
                    let focus_handle = focus_handle.clone();
                    move |cx| {
                        Tooltip::for_action_in(
                            "Add Suggested Context",
                            &AcceptSuggestedContext,
                            &focus_handle,
                            cx,
                        )
                    }
                })
                .when_some(on_click, |element, on_click| {
                    element.on_click(move |event, cx| on_click(event, cx))
                }),
        }
    }
}
