#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod layers;
mod viewport;

use gpui_kit::component::{
    Theme, ThemeMode,
    dock::{DockLayout, DockPlacement, DockSkin, panel_handle},
};
use gpui_kit::{
    App, AppContext, Bounds, Focusable, TitlebarOptions, WindowBounds, WindowOptions, px, size,
};

use layers::Layers;
use viewport::Viewport;

fn main() {
    gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        .run(|cx| {
            gpui_kit::init(cx);
            Theme::change(ThemeMode::Dark, None, cx);

            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1100.0), px(720.0)),
                    cx,
                ))),
                window_min_size: Some(size(px(600.0), px(360.0))),
                app_id: Some("com.github.ivanl1213.civilcad".into()),
                titlebar: Some(TitlebarOptions {
                    title: Some("CivilCAD".into()),
                    ..Default::default()
                }),
                ..Default::default()
            };

            gpui_kit::open_window(options, cx, |window, cx| {
                let viewport = cx.new(|cx| Viewport::new(window, cx));
                let layers = cx.new(|cx| Layers::new(cx.focus_handle()));
                viewport.read(cx).focus_handle(cx).focus(window, cx);

                let (dock, _) = DockSkin::dock_area("workspace", None, window, cx);
                dock.update(cx, |dock, cx| {
                    dock.set_center(
                        DockLayout::tabs().panel_view(panel_handle(viewport), cx),
                        window,
                        cx,
                    );
                    dock.set_dock(
                        DockPlacement::Left,
                        DockLayout::tabs().panel_view(panel_handle(layers), cx),
                        window,
                        cx,
                    );
                    dock.set_dock_size(DockPlacement::Left, px(200.0), window, cx);
                });
                dock
            })
            .expect("failed to open the CivilCAD workspace");

            cx.on_window_closed(|cx: &mut App, _| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            })
            .detach();
            cx.activate(true);
        });
}
