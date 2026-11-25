use gpui_kit::base::dock::{Panel, PanelEvent};
use gpui_kit::component::{ActiveTheme, dock};
use gpui_kit::{
    App, Context, EventEmitter, FocusHandle, Focusable, IntoElement, ParentElement, Render, Styled,
    Window, div,
};

pub struct Layers {
    focus: FocusHandle,
}

impl Layers {
    pub fn new(focus: FocusHandle) -> Self {
        Self { focus }
    }
}

impl EventEmitter<PanelEvent> for Layers {}

impl Focusable for Layers {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Panel for Layers {
    fn panel_name(&self) -> &'static str {
        "layers"
    }

    fn closable(&self, _: &App) -> bool {
        false
    }
}

impl dock::Panel for Layers {
    fn title(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        "Layers"
    }
}

impl Render for Layers {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().p_2().child(
            div()
                .flex()
                .justify_between()
                .rounded_md()
                .bg(cx.theme().secondary)
                .px_2()
                .py_1()
                .text_sm()
                .child("0")
                .child(
                    div()
                        .text_color(cx.theme().muted_foreground)
                        .child("Current"),
                ),
        )
    }
}
