use skulpin_plugin_imgui::imgui;

use std::borrow::Borrow;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use std::{rc::Rc, sync::Arc};

use crate::events;
use crate::state::Mode;
use crate::STATE;

pub mod icons;
pub mod support;

// These are before transformation by STATE.dpi (glutin scale_factor)
const TOOLBOX_OFFSET_X: f32 = 10.;
const TOOLBOX_OFFSET_Y: f32 = TOOLBOX_OFFSET_X;
const TOOLBOX_WIDTH: f32 = 55.;
const TOOLBOX_HEIGHT: f32 = 220.;

pub fn build_and_check_button(ui: &imgui::Ui, mode: Mode, icon: &[u8]) {
    STATE.with(|v| {
        let mut pop_me = None;
        if v.borrow().mode == mode {
            pop_me = Some(ui.push_style_color(imgui::StyleColor::Button, [0., 0., 0., 0.2]));
        }
        // Icons are always constant so this is not really unsafe.
        ui.button(
            unsafe { imgui::ImStr::from_utf8_with_nul_unchecked(icon) },
            [0., 30.],
        );
        if ui.is_item_clicked(imgui::MouseButton::Left) {
            v.borrow_mut().mode = mode;
        }
        if let Some(p) = pop_me {
            p.pop(ui);
        }
    });
}

pub fn build_imgui_ui(mut ui: &mut imgui::Ui) {
    STATE.with(|v| {
        let mode = v.borrow().mode;

        imgui::Window::new(imgui::im_str!("Tools"))
            .bg_alpha(1.) // See comment on fn redraw_skia
            .flags(
                #[rustfmt::skip]
                      imgui::WindowFlags::NO_RESIZE
                    | imgui::WindowFlags::NO_MOVE
                    | imgui::WindowFlags::NO_COLLAPSE,
            )
            .position(
                [TOOLBOX_OFFSET_X, TOOLBOX_OFFSET_Y],
                imgui::Condition::Always,
            )
            .size([TOOLBOX_WIDTH, TOOLBOX_HEIGHT], imgui::Condition::Always)
            .build(ui, || {
                build_and_check_button(&ui, Mode::Pan, &icons::PAN);
                build_and_check_button(&ui, Mode::Select, &icons::SELECT);
                ui.separator();
                build_and_check_button(&ui, Mode::Zoom, &icons::ZOOM);
                ui.separator();
                build_and_check_button(&ui, Mode::Pen, &icons::PEN);
            });

        let new_mode = v.borrow().mode;
        if new_mode != mode {
            events::mode_switched(mode, new_mode);
        }
    });
}

use font_kit::{
    family_name::FamilyName as FKFamilyName, handle::Handle as FKHandle, properties::Properties,
    source::SystemSource,
};

struct Font {
    data: Vec<u8>,
    path: Option<PathBuf>,
}

fn load_font(family: &[FKFamilyName]) -> Font {
    let source = SystemSource::new();
    let font = source
        .select_best_match(family, &Properties::new())
        .unwrap();
    match font {
        FKHandle::Path { path, .. } => Font {
            path: Some(path.clone()),
            data: fs::read(path).expect("Failed to open font path system specified"),
        },
        FKHandle::Memory { bytes, .. } => Font {
            path: None,
            data: Arc::try_unwrap(bytes).expect("Failed to load in-memory font"),
        },
    }
}

lazy_static! {
    static ref SYSTEMSANS: Font = load_font(&[
        FKFamilyName::Title("Segoe UI".to_string()),
        FKFamilyName::SansSerif
    ]);
    static ref SYSTEMMONO: Font = load_font(&[FKFamilyName::Monospace]);
}

use skulpin::skia_safe::Rect;

pub fn toolbox_rect() -> Rect {
    let dpi = STATE.with(|v| v.borrow().dpi) as f32;
    Rect::from_point_and_size(
        (TOOLBOX_OFFSET_X * dpi, TOOLBOX_OFFSET_Y * dpi),
        (TOOLBOX_WIDTH * dpi, TOOLBOX_HEIGHT * dpi),
    )
}
