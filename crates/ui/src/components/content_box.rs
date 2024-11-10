use crate::internal::prelude::*;
use gpui::{AnyElement, IntoElement, ParentElement, StyleRefinement, Styled};
use smallvec::SmallVec;

register_components!(layout, [ContentBox]);

/// A flexible container component that can hold other elements.
#[derive(IntoElement)]
pub struct ContentBox {
    base: Div,
    border: bool,
    fill: bool,
    children: SmallVec<[AnyElement; 2]>,
}

impl ContentBox {
    /// Creates a new [ContentBox].
    pub fn new() -> Self {
        Self {
            base: div(),
            border: true,
            fill: true,
            children: SmallVec::new(),
        }
    }

    /// Removes the border from the [ContentBox].
    pub fn borderless(mut self) -> Self {
        self.border = false;
        self
    }

    /// Removes the background fill from the [ContentBox].
    pub fn unfilled(mut self) -> Self {
        self.fill = false;
        self
    }
}

impl ParentElement for ContentBox {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements)
    }
}

impl Styled for ContentBox {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for ContentBox {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        // TODO:
        // Baked in padding will make scrollable views inside of content boxes awkward.
        //
        // Do we make the padding optional, or do we push to use a different component?

        self.base
            .when(self.fill, |this| {
                this.bg(cx.theme().colors().text.opacity(0.05))
            })
            .when(self.border, |this| {
                this.border_1().border_color(cx.theme().colors().border)
            })
            .rounded_md()
            .p_2()
            .children(self.children)
    }
}

impl ComponentPreview for ContentBox {
    fn description() -> impl Into<Option<&'static str>> {
        "A flexible container component that can hold other elements. It can be customized with or without a border and background fill."
    }

    fn example_label_side() -> ExampleLabelSide {
        ExampleLabelSide::Bottom
    }

    fn examples() -> Vec<ComponentExampleGroup<Self>> {
        vec![example_group(vec![
            single_example(
                "Default",
                ContentBox::new()
                    .flex_1()
                    .items_center()
                    .justify_center()
                    .h_48()
                    .child(Label::new("Default ContentBox")),
            )
            .grow(),
            single_example(
                "Without Border",
                ContentBox::new()
                    .flex_1()
                    .items_center()
                    .justify_center()
                    .h_48()
                    .borderless()
                    .child(Label::new("Borderless ContentBox")),
            )
            .grow(),
            single_example(
                "Without Fill",
                ContentBox::new()
                    .flex_1()
                    .items_center()
                    .justify_center()
                    .h_48()
                    .unfilled()
                    .child(Label::new("Unfilled ContentBox")),
            )
            .grow(),
        ])]
    }
}
