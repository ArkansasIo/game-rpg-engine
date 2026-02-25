#[cfg(not(target_arch = "wasm32"))]
#[macro_use]
mod macros;

pub mod actionlist;
pub mod actions;
pub mod codeeditor;
pub mod configeditor;
pub mod dockmanager;
pub mod docks;
pub mod editcamera;
pub mod editor;
pub mod editor_tools;
pub mod effectpicker;
pub mod hud;
#[cfg(not(target_arch = "wasm32"))]
pub mod i18n;
pub mod infoviewer;
#[cfg(not(target_arch = "wasm32"))]
pub mod menu_sfx;
pub mod mapeditor;
pub mod minimap;
pub mod misc;
pub mod nodeeditor;
pub mod panels;
pub mod rendereditor;
#[cfg(all(not(target_arch = "wasm32"), feature = "self-update"))]
pub mod self_update;
pub mod shapepicker;
pub mod sidebar;
pub mod tilemapeditor;
pub mod tilepicker;
pub mod toollist;
pub mod tools;
pub mod undo;
pub mod utils;
pub mod worldeditor;
pub mod menus;

use rust_embed::RustEmbed;
#[derive(RustEmbed)]
#[folder = "embedded/"]
#[exclude = "*.DS_Store"]
pub struct Embedded;

pub const DEFAULT_VLAYOUT_RATIO: f32 = 0.62;

#[allow(ambiguous_glob_reexports)]
pub mod prelude {
    pub use ::serde::{Deserialize, Serialize};

    pub use codegridfx::prelude::*;
    pub use shared::prelude::*;
    pub use std::sync::{LazyLock, RwLock};
    pub use theframework::prelude::*;

    pub use crate::codeeditor::*;
    // pub use crate::effectpicker::*;
    pub use crate::mapeditor::*;
    pub use crate::misc::*;
    pub use crate::panels::*;
    // pub use crate::previewview::*;
    pub use crate::actionlist::*;
    pub use crate::shapepicker::*;
    pub use crate::sidebar::*;
    pub use crate::tilemapeditor::*;
    pub use crate::tilepicker::*;
    pub use crate::toollist::*;
    pub use crate::undo::material_undo::*;
    pub use crate::undo::palette_undo::*;
    pub use crate::undo::project_atoms::*;
    pub use crate::undo::project_helper::*;
    pub use crate::undo::project_undo::*;
    pub use crate::undo::region_undo::*;
    pub use crate::undo::*;
    pub use crate::utils::*;

    pub use crate::tools::game::GameTool;
    pub use crate::tools::linedef::LinedefTool;
    pub use crate::tools::pixelart::PixelArtTool;
    pub use crate::tools::sector::SectorTool;
    pub use crate::tools::selection::SelectionTool;
    // pub use crate::tools::tileset::TilesetTool;
    pub use crate::tools::vertex::VertexTool;

    pub use crate::docks::tiles::*;

    pub use crate::actions::*;
    pub use crate::docks::*;
    pub use crate::editor_tools::*;
    pub use crate::tools::*;

    pub use crate::configeditor::ConfigEditor;
    pub use crate::editcamera::{CustomMoveAction, EditCamera};
    pub use crate::infoviewer::InfoViewer;
    pub use crate::nodeeditor::{NodeContext, NodeEditor};
    pub use crate::rendereditor::{RenderEditor, RenderMoveAction};
    pub use crate::worldeditor::WorldEditor;

    pub use crate::dockmanager::{DockManager, DockManagerState};

    pub use toml::Table;

    pub const KEY_ESCAPE: u32 = 0;
    pub const KEY_RETURN: u32 = 1;
    pub const KEY_DELETE: u32 = 2;
    pub const KEY_UP: u32 = 3;
    pub const KEY_RIGHT: u32 = 4;
    pub const KEY_DOWN: u32 = 5;
    pub const KEY_LEFT: u32 = 6;
    pub const KEY_SPACE: u32 = 7;
    pub const KEY_TAB: u32 = 8;
}

// --- FFI exports for the Xcode static library build ---

#[cfg(feature = "staticlib")]
mod ffi {
    use super::editor::{CompileFromEditorInputRequest, CompileFromEditorInputResponse, Editor};
    use super::prelude::*;

    use std::ffi::{CStr, CString};
    use std::os::raw::c_char;
    use std::ptr;

    use lazy_static::lazy_static;
    use std::sync::Mutex;

    lazy_static! {
        static ref APP: Mutex<Editor> = Mutex::new(Editor::new());
        static ref CTX: Mutex<TheContext> = Mutex::new(TheContext::new(800, 600, 1.0));
        static ref UI: Mutex<TheUI> = Mutex::new(TheUI::new());
        static ref BOOT_STATE: Mutex<EditorBootState> = Mutex::new(EditorBootState::default());
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
    enum EditorBootStage {
        Boot,
        TitleMain,
        Loading,
        Ready,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct EditorScreenDescriptor {
        id: String,
        title: String,
        subtitle: String,
        details: String,
        accent_hex: String,
        progress: f32,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct EditorBootState {
        api_version: String,
        engine_name: String,
        editor_name: String,
        stage: EditorBootStage,
        active_screen_id: String,
        loading_message: String,
        loading_progress: f32,
        screens: Vec<EditorScreenDescriptor>,
    }

    impl Default for EditorBootState {
        fn default() -> Self {
            let mut state = Self {
                api_version: "1".to_string(),
                engine_name: "Vertex Engine".to_string(),
                editor_name: "NexusStudio".to_string(),
                stage: EditorBootStage::Boot,
                active_screen_id: "boot".to_string(),
                loading_message: "Booting editor runtime...".to_string(),
                loading_progress: 0.0,
                screens: vec![
                    EditorScreenDescriptor {
                        id: "boot".to_string(),
                        title: "Vertex Engine Editor".to_string(),
                        subtitle: "Boot Screen".to_string(),
                        details: "Preparing editor kernel and project services.".to_string(),
                        accent_hex: "#00D18F".to_string(),
                        progress: 0.0,
                    },
                    EditorScreenDescriptor {
                        id: "title_main".to_string(),
                        title: "NexusStudio".to_string(),
                        subtitle: "Main Title".to_string(),
                        details: "Create, load, and build RPG worlds.".to_string(),
                        accent_hex: "#FFD166".to_string(),
                        progress: 1.0,
                    },
                    EditorScreenDescriptor {
                        id: "loading".to_string(),
                        title: "Loading Project".to_string(),
                        subtitle: "Please Wait".to_string(),
                        details: "Building assets and initializing tools.".to_string(),
                        accent_hex: "#4CC9F0".to_string(),
                        progress: 0.0,
                    },
                ],
            };
            state.sync_for_stage();
            state
        }
    }

    impl EditorBootState {
        fn set_stage_from_u32(&mut self, stage: u32) {
            self.stage = match stage {
                0 => EditorBootStage::Boot,
                1 => EditorBootStage::TitleMain,
                2 => EditorBootStage::Loading,
                _ => EditorBootStage::Ready,
            };
            self.sync_for_stage();
        }

        fn stage_as_u32(&self) -> u32 {
            match self.stage {
                EditorBootStage::Boot => 0,
                EditorBootStage::TitleMain => 1,
                EditorBootStage::Loading => 2,
                EditorBootStage::Ready => 3,
            }
        }

        fn sync_for_stage(&mut self) {
            self.active_screen_id = match self.stage {
                EditorBootStage::Boot => "boot".to_string(),
                EditorBootStage::TitleMain => "title_main".to_string(),
                EditorBootStage::Loading => "loading".to_string(),
                EditorBootStage::Ready => "title_main".to_string(),
            };
            for screen in &mut self.screens {
                if screen.id == "loading" {
                    screen.progress = self.loading_progress;
                    screen.details = self.loading_message.clone();
                } else if screen.id == "boot" {
                    screen.progress = if self.stage == EditorBootStage::Boot {
                        self.loading_progress
                    } else {
                        1.0
                    };
                }
            }
        }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn rust_init() {
        {
            let mut boot = BOOT_STATE.lock().unwrap();
            boot.set_stage_from_u32(2);
            boot.loading_progress = 0.2;
            boot.loading_message = "Initializing UI context...".to_string();
            boot.sync_for_stage();
        }
        UI.lock().unwrap().init(&mut CTX.lock().unwrap());
        APP.lock().unwrap().init(&mut CTX.lock().unwrap());
        {
            let mut boot = BOOT_STATE.lock().unwrap();
            boot.loading_progress = 0.65;
            boot.loading_message = "Building editor interface...".to_string();
            boot.sync_for_stage();
        }
        APP.lock()
            .unwrap()
            .init_ui(&mut UI.lock().unwrap(), &mut CTX.lock().unwrap());

        // Load the starter project, same as the winit version does via set_cmd_line_args
        CTX.lock().unwrap().ui.send(TheEvent::StateChanged(
            TheId::named("New"),
            TheWidgetState::Clicked,
        ));
        {
            let mut boot = BOOT_STATE.lock().unwrap();
            boot.loading_progress = 1.0;
            boot.loading_message = "Editor ready.".to_string();
            boot.set_stage_from_u32(1);
            boot.sync_for_stage();
        }
    }

    /// # Safety
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn rust_draw(pixels: *mut u8, width: u32, height: u32) {
        let length = width as usize * height as usize * 4;
        let slice = unsafe { std::slice::from_raw_parts_mut(pixels, length) };

        CTX.lock().unwrap().width = width as usize;
        CTX.lock().unwrap().height = height as usize;

        UI.lock().unwrap().draw(slice, &mut CTX.lock().unwrap());
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn rust_update() -> bool {
        UI.lock().unwrap().update(&mut CTX.lock().unwrap());
        APP.lock()
            .unwrap()
            .update_ui(&mut UI.lock().unwrap(), &mut CTX.lock().unwrap());
        APP.lock().unwrap().update(&mut CTX.lock().unwrap())
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn rust_target_fps() -> u32 {
        APP.lock().unwrap().target_fps().clamp(1.0, 120.0) as u32
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn rust_hover(x: f32, y: f32) -> bool {
        UI.lock().unwrap().hover(x, y, &mut CTX.lock().unwrap())
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn rust_touch_down(x: f32, y: f32) -> bool {
        UI.lock()
            .unwrap()
            .touch_down(x, y, &mut CTX.lock().unwrap())
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn rust_touch_dragged(x: f32, y: f32) -> bool {
        UI.lock()
            .unwrap()
            .touch_dragged(x, y, &mut CTX.lock().unwrap())
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn rust_touch_up(x: f32, y: f32) -> bool {
        UI.lock().unwrap().touch_up(x, y, &mut CTX.lock().unwrap())
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn rust_touch_wheel(x: f32, y: f32) -> bool {
        UI.lock()
            .unwrap()
            .mouse_wheel((x as i32, y as i32), &mut CTX.lock().unwrap())
    }

    /// # Safety
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn rust_key_down(p: *const c_char) -> bool {
        let c_str = unsafe { CStr::from_ptr(p) };
        if let Ok(key) = c_str.to_str() {
            if let Some(ch) = key.chars().next() {
                return UI
                    .lock()
                    .unwrap()
                    .key_down(Some(ch), None, &mut CTX.lock().unwrap());
            }
        }
        false
    }

    /// # Safety
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn rust_key_up(p: *const c_char) -> bool {
        let c_str = unsafe { CStr::from_ptr(p) };
        if let Ok(key) = c_str.to_str() {
            if let Some(ch) = key.chars().next() {
                return UI
                    .lock()
                    .unwrap()
                    .key_up(Some(ch), None, &mut CTX.lock().unwrap());
            }
        }
        false
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn rust_special_key_down(key: u32) -> bool {
        if key == KEY_ESCAPE {
            UI.lock()
                .unwrap()
                .key_down(None, Some(TheKeyCode::Escape), &mut CTX.lock().unwrap())
        } else if key == KEY_RETURN {
            UI.lock()
                .unwrap()
                .key_down(None, Some(TheKeyCode::Return), &mut CTX.lock().unwrap())
        } else if key == KEY_DELETE {
            UI.lock()
                .unwrap()
                .key_down(None, Some(TheKeyCode::Delete), &mut CTX.lock().unwrap())
        } else if key == KEY_UP {
            UI.lock()
                .unwrap()
                .key_down(None, Some(TheKeyCode::Up), &mut CTX.lock().unwrap())
        } else if key == KEY_RIGHT {
            UI.lock()
                .unwrap()
                .key_down(None, Some(TheKeyCode::Right), &mut CTX.lock().unwrap())
        } else if key == KEY_DOWN {
            UI.lock()
                .unwrap()
                .key_down(None, Some(TheKeyCode::Down), &mut CTX.lock().unwrap())
        } else if key == KEY_LEFT {
            UI.lock()
                .unwrap()
                .key_down(None, Some(TheKeyCode::Left), &mut CTX.lock().unwrap())
        } else if key == KEY_SPACE {
            UI.lock()
                .unwrap()
                .key_down(None, Some(TheKeyCode::Space), &mut CTX.lock().unwrap())
        } else {
            UI.lock()
                .unwrap()
                .key_down(None, Some(TheKeyCode::Tab), &mut CTX.lock().unwrap())
        }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn rust_key_modifier_changed(
        shift: bool,
        ctrl: bool,
        alt: bool,
        logo: bool,
    ) -> bool {
        UI.lock()
            .unwrap()
            .modifier_changed(shift, ctrl, alt, logo, &mut CTX.lock().unwrap());
        APP.lock().unwrap().modifier_changed(shift, ctrl, alt, logo)
    }

    /// # Safety
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn rust_dropped_file(p: *const c_char) {
        let path_str = unsafe { CStr::from_ptr(p) };
        if let Ok(path) = path_str.to_str() {
            APP.lock().unwrap().dropped_file(path.to_string());
        }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn rust_new() {
        CTX.lock().unwrap().ui.send(TheEvent::StateChanged(
            TheId::named("New"),
            TheWidgetState::Clicked,
        ));
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn rust_play() {
        CTX.lock().unwrap().ui.send(TheEvent::StateChanged(
            TheId::named("Play"),
            TheWidgetState::Clicked,
        ));
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn rust_pause() {
        CTX.lock().unwrap().ui.send(TheEvent::StateChanged(
            TheId::named("Pause"),
            TheWidgetState::Clicked,
        ));
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn rust_stop() {
        CTX.lock().unwrap().ui.send(TheEvent::StateChanged(
            TheId::named("Stop"),
            TheWidgetState::Clicked,
        ));
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn rust_open() {
        CTX.lock().unwrap().ui.send(TheEvent::StateChanged(
            TheId::named("Open"),
            TheWidgetState::Clicked,
        ));
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn rust_save() {
        CTX.lock().unwrap().ui.send(TheEvent::StateChanged(
            TheId::named("Save"),
            TheWidgetState::Clicked,
        ));
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn rust_save_as() {
        CTX.lock().unwrap().ui.send(TheEvent::StateChanged(
            TheId::named("Save As"),
            TheWidgetState::Clicked,
        ));
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn rust_cut() -> *mut c_char {
        CTX.lock().unwrap().ui.send(TheEvent::StateChanged(
            TheId::named("Cut"),
            TheWidgetState::Clicked,
        ));
        APP.lock()
            .unwrap()
            .update_ui(&mut UI.lock().unwrap(), &mut CTX.lock().unwrap());

        if let Some(TheValue::Text(text)) = &CTX.lock().unwrap().ui.clipboard {
            return CString::new(text.clone()).unwrap().into_raw();
        }
        ptr::null_mut()
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn rust_copy() -> *mut c_char {
        CTX.lock().unwrap().ui.send(TheEvent::StateChanged(
            TheId::named("Copy"),
            TheWidgetState::Clicked,
        ));
        APP.lock()
            .unwrap()
            .update_ui(&mut UI.lock().unwrap(), &mut CTX.lock().unwrap());

        if let Some(TheValue::Text(text)) = &CTX.lock().unwrap().ui.clipboard {
            return CString::new(text.clone()).unwrap().into_raw();
        }
        ptr::null_mut()
    }

    /// # Safety
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn rust_paste(p: *const c_char) {
        let text_str = unsafe { CStr::from_ptr(p) };
        if let Ok(text) = text_str.to_str() {
            {
                let mut ctx = CTX.lock().unwrap();
                ctx.ui.clipboard = Some(TheValue::Text(text.to_string()));
                ctx.ui.clipboard_app_type = Some("text/plain".to_string());
            }

            CTX.lock().unwrap().ui.send(TheEvent::StateChanged(
                TheId::named("Paste"),
                TheWidgetState::Clicked,
            ));

            APP.lock()
                .unwrap()
                .update_ui(&mut UI.lock().unwrap(), &mut CTX.lock().unwrap());
        }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn rust_undo() {
        CTX.lock().unwrap().ui.send(TheEvent::StateChanged(
            TheId::named("Undo"),
            TheWidgetState::Clicked,
        ));
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn rust_redo() {
        CTX.lock().unwrap().ui.send(TheEvent::StateChanged(
            TheId::named("Redo"),
            TheWidgetState::Clicked,
        ));
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn rust_has_changes() -> bool {
        APP.lock().unwrap().has_changes()
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn rust_editor_boot_stage() -> u32 {
        BOOT_STATE.lock().unwrap().stage_as_u32()
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn rust_editor_boot_set_stage(stage: u32) {
        BOOT_STATE.lock().unwrap().set_stage_from_u32(stage);
    }

    /// # Safety
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn rust_editor_boot_set_loading(
        progress: f32,
        message: *const c_char,
    ) {
        let mut boot = BOOT_STATE.lock().unwrap();
        boot.loading_progress = progress.clamp(0.0, 1.0);
        if !message.is_null() {
            let c = unsafe { CStr::from_ptr(message) };
            if let Ok(msg) = c.to_str() {
                boot.loading_message = msg.to_string();
            }
        }
        if boot.stage == EditorBootStage::Loading || boot.stage == EditorBootStage::Boot {
            boot.sync_for_stage();
        }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn rust_editor_boot_reset() {
        *BOOT_STATE.lock().unwrap() = EditorBootState::default();
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn rust_editor_boot_get_json() -> *mut c_char {
        let payload =
            serde_json::to_string(&*BOOT_STATE.lock().unwrap()).unwrap_or_else(|_| "{}".to_string());
        to_ffi_c_string(payload)
    }

    fn ffi_error_response(message: impl Into<String>) -> CompileFromEditorInputResponse {
        CompileFromEditorInputResponse {
            success: false,
            message: message.into(),
            warnings: vec![],
            errors: vec![],
            output_path: None,
            compiled_project_json: None,
            stats: Default::default(),
        }
    }

    fn to_ffi_c_string(value: String) -> *mut c_char {
        match CString::new(value) {
            Ok(cstring) => cstring.into_raw(),
            Err(_) => CString::new("{\"success\":false,\"message\":\"Invalid string for FFI output.\",\"warnings\":[],\"errors\":[],\"output_path\":null,\"compiled_project_json\":null,\"stats\":{\"regions\":0,\"screens\":0,\"tilemaps\":0,\"tiles\":0,\"characters\":0,\"items\":0,\"assets\":0}}")
                .map(CString::into_raw)
                .unwrap_or(ptr::null_mut()),
        }
    }

    /// # Safety
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn rust_compile_game_from_editor_input(
        input_json: *const c_char,
    ) -> *mut c_char {
        if input_json.is_null() {
            let response = ffi_error_response("Input JSON pointer is null.");
            let payload = serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string());
            return to_ffi_c_string(payload);
        }

        let raw_input = unsafe { CStr::from_ptr(input_json) };
        let input_str = match raw_input.to_str() {
            Ok(value) => value,
            Err(error) => {
                let mut response = ffi_error_response("Input JSON is not valid UTF-8.");
                response.errors.push(error.to_string());
                let payload = serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string());
                return to_ffi_c_string(payload);
            }
        };

        let request = match serde_json::from_str::<CompileFromEditorInputRequest>(input_str) {
            Ok(request) => request,
            Err(error) => {
                let mut response = ffi_error_response("Failed to parse compile request JSON.");
                response.errors.push(error.to_string());
                let payload = serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string());
                return to_ffi_c_string(payload);
            }
        };

        let response = APP.lock().unwrap().compile_from_editor_input(request);
        let payload = serde_json::to_string(&response).unwrap_or_else(|error| {
            let mut fallback = ffi_error_response("Failed to serialize compile response.");
            fallback.errors.push(error.to_string());
            serde_json::to_string(&fallback).unwrap_or_else(|_| "{}".to_string())
        });

        to_ffi_c_string(payload)
    }

    /// # Safety
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn rust_free_string(value: *mut c_char) {
        if value.is_null() {
            return;
        }
        unsafe {
            let _ = CString::from_raw(value);
        }
    }
}
