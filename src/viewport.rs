use std::{cell::Cell, rc::Rc};

use gpui_kit::base::dock::{Panel, PanelEvent};
use gpui_kit::component::{ActiveTheme, Sizable, button::Button, dock};
use gpui_kit::{
    App, Bounds, Context, EventEmitter, FocusHandle, Focusable, InteractiveElement, IntoElement,
    MouseButton, ParentElement, Pixels, Point, Render, StatefulInteractiveElement, Styled, Window,
    canvas, div, fill, point, px, rgb, size,
};

const INITIAL_SCALE: f32 = 32.0;

#[derive(Clone, Copy)]
struct Camera {
    offset: Point<Pixels>,
    scale: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            offset: Point::default(),
            scale: INITIAL_SCALE,
        }
    }
}

impl Camera {
    fn origin(self, bounds: Bounds<Pixels>) -> Point<Pixels> {
        bounds.center() + self.offset
    }

    fn zoom(&mut self, bounds: Bounds<Pixels>, anchor: Point<Pixels>, factor: f32) {
        let scale = (self.scale * factor).clamp(0.25, 4096.0);
        self.offset += (anchor - self.origin(bounds)) * (1.0 - scale / self.scale);
        self.scale = scale;
    }
}

pub struct Viewport {
    focus: FocusHandle,
    camera: Camera,
    bounds: Rc<Cell<Bounds<Pixels>>>,
    cursor: Option<Point<Pixels>>,
    drag: Option<(MouseButton, Point<Pixels>)>,
    space_held: bool,
}

impl Viewport {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus = cx.focus_handle();
        cx.on_focus_out(&focus, window, |this, _, _, _| {
            this.space_held = false;
            this.drag = None;
        })
        .detach();
        Self {
            focus,
            camera: Camera::default(),
            bounds: Rc::default(),
            cursor: None,
            drag: None,
            space_held: false,
        }
    }

    fn zoom(&mut self, at: Point<Pixels>, factor: f32, cx: &mut Context<Self>) {
        self.camera.zoom(self.bounds.get(), at, factor);
        self.cursor = Some(at);
        cx.notify();
    }
}

impl EventEmitter<PanelEvent> for Viewport {}

impl Focusable for Viewport {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Panel for Viewport {
    fn panel_name(&self) -> &'static str {
        "drawing"
    }

    fn closable(&self, _: &App) -> bool {
        false
    }
}

impl dock::Panel for Viewport {
    fn title(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        "Drawing"
    }

    fn title_bar(&self, _: &App) -> bool {
        false
    }

    fn inner_padding(&self, _: &App) -> bool {
        false
    }
}

impl Render for Viewport {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let camera = self.camera;
        let bounds = self.bounds.clone();
        let coordinates = self.cursor.map_or_else(
            || "X        —   Y        —".to_owned(),
            |cursor| {
                let position = (cursor - camera.origin(bounds.get())) * (1.0 / camera.scale);
                format!(
                    "X {:>8.2}   Y {:>8.2}",
                    f32::from(position.x),
                    -f32::from(position.y)
                )
            },
        );

        let drawing = div()
            .id("drawing-canvas")
            .flex_1()
            .min_h_0()
            .overflow_hidden()
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, event: &gpui_kit::KeyDownEvent, _, _| {
                if event.keystroke.key == "space" {
                    this.space_held = true;
                }
            }))
            .on_key_up(cx.listener(|this, event: &gpui_kit::KeyUpEvent, _, _| {
                if event.keystroke.key == "space" {
                    this.space_held = false;
                    if matches!(this.drag, Some((MouseButton::Left, _))) {
                        this.drag = None;
                    }
                }
            }))
            .on_mouse_down(
                MouseButton::Middle,
                cx.listener(|this, event: &gpui_kit::MouseDownEvent, window, cx| {
                    this.focus.focus(window, cx);
                    this.drag = Some((MouseButton::Middle, event.position));
                }),
            )
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &gpui_kit::MouseDownEvent, window, cx| {
                    this.focus.focus(window, cx);
                    if this.space_held {
                        this.drag = Some((MouseButton::Left, event.position));
                    }
                }),
            )
            .capture_any_mouse_up(cx.listener(|this, event: &gpui_kit::MouseUpEvent, _, _| {
                if this.drag.is_some_and(|(button, _)| button == event.button) {
                    this.drag = None;
                }
            }))
            .on_mouse_move(
                cx.listener(|this, event: &gpui_kit::MouseMoveEvent, _, cx| {
                    if let Some((button, previous)) = this.drag {
                        if event.pressed_button == Some(button) {
                            this.camera.offset += event.position - previous;
                            this.drag = Some((button, event.position));
                        } else {
                            this.drag = None;
                        }
                    }
                    this.cursor = Some(event.position);
                    cx.notify();
                }),
            )
            .on_hover(cx.listener(|this, hovered: &bool, _, cx| {
                if !hovered {
                    this.cursor = None;
                    this.drag = None;
                    cx.notify();
                }
            }))
            .on_scroll(cx.listener(|this, event: &gpui_kit::ScrollEvent, _, cx| {
                let delta = f32::from(event.delta.pixel_delta(px(32.0)).y);
                this.zoom(event.position, (delta * 0.004).exp(), cx);
            }))
            .on_pinch(cx.listener(|this, event: &gpui_kit::PinchEvent, _, cx| {
                this.zoom(event.position, 1.0 + event.delta, cx);
            }))
            .child(
                canvas(
                    move |area, _, _| bounds.set(area),
                    move |area, (), window, _| {
                        paint_grid(window, area, camera);
                    },
                )
                .size_full(),
            );

        div()
            .flex()
            .flex_col()
            .size_full()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .px_2()
                    .h(px(36.0))
                    .text_xs()
                    .child(
                        Button::new("reset-view")
                            .small()
                            .label("Reset view")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.camera = Camera::default();
                                this.focus.focus(window, cx);
                                cx.notify();
                            })),
                    )
                    .child(
                        div()
                            .text_color(cx.theme().muted_foreground)
                            .child("Middle drag or Space + drag to pan · Scroll or pinch to zoom"),
                    ),
            )
            .child(drawing)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .px_2()
                    .h(px(26.0))
                    .font_family("monospace")
                    .text_xs()
                    .child(coordinates)
                    .child(format!("{:.0}%", camera.scale / INITIAL_SCALE * 100.0)),
            )
    }
}

fn paint_grid(window: &mut Window, bounds: Bounds<Pixels>, camera: Camera) {
    window.paint_quad(fill(bounds, rgb(0x14181e)));
    let origin = camera.origin(bounds);

    // Double or halve the world spacing as zoom changes to bound the grid line count.
    let step = camera.scale * 2.0_f32.powf((INITIAL_SCALE / camera.scale).log2().ceil());
    let first_x = f32::from(origin.x - bounds.left()).rem_euclid(step);
    let first_y = f32::from(origin.y - bounds.top()).rem_euclid(step);
    let vertical = |x| Bounds {
        origin: point(x, bounds.top()),
        size: size(px(1.0), bounds.size.height),
    };
    let horizontal = |y| Bounds {
        origin: point(bounds.left(), y),
        size: size(bounds.size.width, px(1.0)),
    };

    for column in 0..(f32::from(bounds.size.width) / step).ceil() as u32 {
        window.paint_quad(fill(
            vertical(bounds.left() + px(first_x + column as f32 * step)),
            rgb(0x29313b),
        ));
    }
    for row in 0..(f32::from(bounds.size.height) / step).ceil() as u32 {
        window.paint_quad(fill(
            horizontal(bounds.top() + px(first_y + row as f32 * step)),
            rgb(0x29313b),
        ));
    }
    if origin.x >= bounds.left() && origin.x < bounds.right() {
        window.paint_quad(fill(vertical(origin.x), rgb(0x619978)));
    }
    if origin.y >= bounds.top() && origin.y < bounds.bottom() {
        window.paint_quad(fill(horizontal(origin.y), rgb(0xac666a)));
    }
}
