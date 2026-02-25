use crate::Embedded;
use crate::prelude::*;
#[cfg(all(not(target_arch = "wasm32"), feature = "self-update"))]
use crate::self_update::{SelfUpdateEvent, SelfUpdater};
use crate::undo::character_undo::CharacterUndoAtom;
use crate::undo::item_undo::ItemUndoAtom;
use codegridfx::Module;
use rusterix::{
    PlayerCamera, Rusterix, SceneManager, SceneManagerResult, Texture, Value, ValueContainer,
};
use shared::rusterix_utils::*;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::mpsc::Receiver;
#[cfg(all(not(target_arch = "wasm32"), feature = "self-update"))]
use std::sync::{
    Arc, Mutex,
    mpsc::{Sender, channel},
};

#[cfg(all(not(target_arch = "wasm32"), feature = "self-update"))]
use std::thread;

pub static PREVIEW_ICON: LazyLock<RwLock<(TheRGBATile, i32)>> =
    LazyLock::new(|| RwLock::new((TheRGBATile::default(), 0)));

pub static TILEPICKER: LazyLock<RwLock<TilePicker>> =
    LazyLock::new(|| RwLock::new(TilePicker::new("Main Tile Picker".to_string())));
pub static SHAPEPICKER: LazyLock<RwLock<ShapePicker>> =
    LazyLock::new(|| RwLock::new(ShapePicker::new("Main Shape Picker".to_string())));
pub static TILEMAPEDITOR: LazyLock<RwLock<TilemapEditor>> =
    LazyLock::new(|| RwLock::new(TilemapEditor::new()));
pub static SIDEBARMODE: LazyLock<RwLock<SidebarMode>> =
    LazyLock::new(|| RwLock::new(SidebarMode::Region));
pub static UNDOMANAGER: LazyLock<RwLock<UndoManager>> =
    LazyLock::new(|| RwLock::new(UndoManager::default()));
pub static TOOLLIST: LazyLock<RwLock<ToolList>> =
    LazyLock::new(|| RwLock::new(ToolList::default()));
pub static ACTIONLIST: LazyLock<RwLock<ActionList>> =
    LazyLock::new(|| RwLock::new(ActionList::default()));
// pub static PANELS: LazyLock<RwLock<Panels>> = LazyLock::new(|| RwLock::new(Panels::new()));
pub static CODEEDITOR: LazyLock<RwLock<CodeEditor>> =
    LazyLock::new(|| RwLock::new(CodeEditor::new()));
pub static PALETTE: LazyLock<RwLock<ThePalette>> =
    LazyLock::new(|| RwLock::new(ThePalette::default()));
pub static RUSTERIX: LazyLock<RwLock<Rusterix>> =
    LazyLock::new(|| RwLock::new(Rusterix::default()));
pub static CONFIGEDITOR: LazyLock<RwLock<ConfigEditor>> =
    LazyLock::new(|| RwLock::new(ConfigEditor::new()));
pub static INFOVIEWER: LazyLock<RwLock<InfoViewer>> =
    LazyLock::new(|| RwLock::new(InfoViewer::new()));
pub static CONFIG: LazyLock<RwLock<toml::Table>> =
    LazyLock::new(|| RwLock::new(toml::Table::default()));
pub static NODEEDITOR: LazyLock<RwLock<NodeEditor>> =
    LazyLock::new(|| RwLock::new(NodeEditor::new()));
pub static WORLDEDITOR: LazyLock<RwLock<WorldEditor>> =
    LazyLock::new(|| RwLock::new(WorldEditor::new()));
pub static RENDEREDITOR: LazyLock<RwLock<RenderEditor>> =
    LazyLock::new(|| RwLock::new(RenderEditor::new()));
pub static EDITCAMERA: LazyLock<RwLock<EditCamera>> =
    LazyLock::new(|| RwLock::new(EditCamera::new()));
pub static SCENEMANAGER: LazyLock<RwLock<SceneManager>> =
    LazyLock::new(|| RwLock::new(SceneManager::default()));
pub static DOCKMANAGER: LazyLock<RwLock<DockManager>> =
    LazyLock::new(|| RwLock::new(DockManager::default()));

pub static CODEGRIDFX: LazyLock<RwLock<Module>> =
    LazyLock::new(|| RwLock::new(Module::as_type(codegridfx::ModuleType::CharacterTemplate)));
pub static SHADEGRIDFX: LazyLock<RwLock<Module>> =
    LazyLock::new(|| RwLock::new(Module::as_type(codegridfx::ModuleType::Shader)));
pub static SHADERBUFFER: LazyLock<RwLock<TheRGBABuffer>> =
    LazyLock::new(|| RwLock::new(TheRGBABuffer::new(TheDim::sized(200, 200))));

pub struct Editor {
    project: Project,
    project_path: Option<PathBuf>,

    sidebar: Sidebar,
    mapeditor: MapEditor,

    server_ctx: ServerContext,

    update_tracker: UpdateTracker,
    event_receiver: Option<Receiver<TheEvent>>,

    #[cfg(all(not(target_arch = "wasm32"), feature = "self-update"))]
    self_update_rx: Receiver<SelfUpdateEvent>,
    #[cfg(all(not(target_arch = "wasm32"), feature = "self-update"))]
    self_update_tx: Sender<SelfUpdateEvent>,
    #[cfg(all(not(target_arch = "wasm32"), feature = "self-update"))]
    self_updater: Arc<Mutex<SelfUpdater>>,

    update_counter: usize,

    build_values: ValueContainer,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CompileFromEditorInputRequest {
    #[serde(default)]
    pub project_json: Option<String>,
    #[serde(default)]
    pub output_path: Option<String>,
    #[serde(default)]
    pub pretty: bool,
    #[serde(default = "default_true")]
    pub include_compiled_project_json: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CompileFromEditorInputStats {
    pub regions: usize,
    pub screens: usize,
    pub tilemaps: usize,
    pub tiles: usize,
    pub characters: usize,
    pub items: usize,
    pub assets: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CompileFromEditorInputResponse {
    pub success: bool,
    pub message: String,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub output_path: Option<String>,
    pub compiled_project_json: Option<String>,
    pub stats: CompileFromEditorInputStats,
}

fn default_true() -> bool {
    true
}

impl Editor {
    fn is_realtime_mode(&self) -> bool {
        self.server_ctx.game_mode
            || RUSTERIX.read().unwrap().server.state == rusterix::ServerState::Running
    }

    fn redraw_interval_ms(&self) -> u64 {
        let config = CONFIGEDITOR.read().unwrap();
        if self.is_realtime_mode() {
            (1000 / config.target_fps.clamp(1, 60)) as u64
        } else {
            config.game_tick_ms.max(1) as u64
        }
    }

    fn compile_project_for_runtime(project: &mut Project) -> Vec<String> {
        let mut warnings = Vec::new();

        for region in &mut project.regions {
            for (_, profile) in &mut region.map.profiles {
                profile.sanitize();
            }
            region.map.sanitize();
        }

        for (_, screen) in &mut project.screens {
            screen.map.sanitize();
        }

        if project.tiles.is_empty() {
            let tiles = project.extract_tiles();

            for (id, tile_data) in &tiles {
                let mut texture_array: Vec<Texture> = vec![];
                for buffer in &tile_data.buffer {
                    let mut texture = Texture::new(
                        buffer.pixels().to_vec(),
                        buffer.dim().width as usize,
                        buffer.dim().height as usize,
                    );
                    texture.generate_normals(true);
                    texture_array.push(texture);
                }
                let tile = rusterix::Tile {
                    id: tile_data.id,
                    role: rusterix::TileRole::from_index(tile_data.role),
                    textures: texture_array,
                    module: None,
                    blocking: tile_data.blocking,
                    scale: tile_data.scale,
                    tags: tile_data.name.clone(),
                };
                project.tiles.insert(*id, tile);
            }
            warnings.push("Project used legacy tilemap tiles; migrated to project tiles.".to_string());
        }

        for tile in project.tiles.values_mut() {
            for texture in &mut tile.textures {
                texture.generate_normals(true);
            }
        }

        for character in project.characters.values_mut() {
            if character.source.starts_with("class") {
                character.source = character.module.build(false);
                character.source_debug = character.module.build(true);
            }
        }

        for item in project.items.values_mut() {
            if item.source.starts_with("class") {
                item.source = item.module.build(false);
                item.source_debug = item.module.build(true);
            }
        }

        warnings
    }

    fn compile_stats(project: &Project) -> CompileFromEditorInputStats {
        CompileFromEditorInputStats {
            regions: project.regions.len(),
            screens: project.screens.len(),
            tilemaps: project.tilemaps.len(),
            tiles: project.tiles.len(),
            characters: project.characters.len(),
            items: project.items.len(),
            assets: project.assets.len(),
        }
    }

    pub fn compile_from_editor_input(
        &mut self,
        request: CompileFromEditorInputRequest,
    ) -> CompileFromEditorInputResponse {
        let mut response = CompileFromEditorInputResponse::default();

        let mut project = if let Some(project_json) = request.project_json.as_ref() {
            match serde_json::from_str::<Project>(project_json) {
                Ok(project) => project,
                Err(error) => {
                    response.message = "Failed to parse editor project input.".to_string();
                    response.errors.push(error.to_string());
                    return response;
                }
            }
        } else {
            self.project.clone()
        };

        response.warnings = Self::compile_project_for_runtime(&mut project);
        response.stats = Self::compile_stats(&project);

        let compiled_json = if request.pretty {
            match serde_json::to_string_pretty(&project) {
                Ok(value) => value,
                Err(error) => {
                    response.message = "Failed to serialize compiled project.".to_string();
                    response.errors.push(error.to_string());
                    return response;
                }
            }
        } else {
            match serde_json::to_string(&project) {
                Ok(value) => value,
                Err(error) => {
                    response.message = "Failed to serialize compiled project.".to_string();
                    response.errors.push(error.to_string());
                    return response;
                }
            }
        };

        if let Some(path) = request.output_path.as_ref() {
            if let Err(error) = std::fs::write(path, &compiled_json) {
                response.message = "Failed to write compiled output file.".to_string();
                response.errors.push(error.to_string());
                return response;
            }
            response.output_path = Some(path.clone());
        }

        if request.include_compiled_project_json || response.output_path.is_none() {
            response.compiled_project_json = Some(compiled_json);
        }

        if request.project_json.is_none() {
            self.project = project;
        }

        response.success = true;
        response.message = "Compile successful.".to_string();
        response
    }

    fn refresh_designer_views(&mut self, ui: &mut TheUI, ctx: &mut TheContext) {
        insert_content_into_maps(&mut self.project);
        self.sidebar
            .load_from_project(ui, ctx, &mut self.server_ctx, &mut self.project);
        self.mapeditor.load_from_project(ui, ctx, &self.project);
        *PALETTE.write().unwrap() = self.project.palette.clone();
    }

    fn unique_name<I>(&self, existing: I, base: &str) -> String
    where
        I: Iterator<Item = String>,
    {
        let names: std::collections::HashSet<String> =
            existing.map(|name| name.to_lowercase()).collect();
        if !names.contains(&base.to_lowercase()) {
            return base.to_string();
        }

        let mut index = 2usize;
        loop {
            let candidate = format!("{base} {index}");
            if !names.contains(&candidate.to_lowercase()) {
                return candidate;
            }
            index += 1;
        }
    }

    fn create_widget_window_screen(&self, base_name: &str, window_type: &str) -> Screen {
        let mut screen = Screen::new();
        screen.name = self.unique_name(
            self.project.screens.values().map(|s| s.name.clone()),
            base_name,
        );
        screen.map.name = format!("{} [{}]", screen.name, window_type);
        screen
    }

    fn validate_project_for_design(&self) -> (Vec<String>, Vec<String>) {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        if self.project.regions.is_empty() {
            errors.push("Project has no regions.".to_string());
        }
        if self.project.tiles.is_empty() && self.project.tilemaps.is_empty() {
            warnings.push("Project has no tiles or tilemaps.".to_string());
        }

        let mut region_names = std::collections::HashSet::new();
        for region in &self.project.regions {
            if region.name.trim().is_empty() {
                errors.push(format!("Region {} has an empty name.", region.id));
            }
            let key = region.name.trim().to_lowercase();
            if !key.is_empty() && !region_names.insert(key.clone()) {
                warnings.push(format!("Duplicate region name found: '{}'.", region.name));
            }
        }

        let mut character_names = std::collections::HashSet::new();
        for character in self.project.characters.values() {
            if character.name.trim().is_empty() {
                errors.push(format!("Character {} has an empty name.", character.id));
            }
            let key = character.name.trim().to_lowercase();
            if !key.is_empty() && !character_names.insert(key.clone()) {
                warnings.push(format!("Duplicate character name found: '{}'.", character.name));
            }
        }

        let mut item_names = std::collections::HashSet::new();
        for item in self.project.items.values() {
            if item.name.trim().is_empty() {
                errors.push(format!("Item {} has an empty name.", item.id));
            }
            let key = item.name.trim().to_lowercase();
            if !key.is_empty() && !item_names.insert(key.clone()) {
                warnings.push(format!("Duplicate item name found: '{}'.", item.name));
            }
        }

        (warnings, errors)
    }

    fn compile_runtime_to_project_folder(
        &mut self,
        pretty: bool,
    ) -> Result<(PathBuf, CompileFromEditorInputResponse), String> {
        let source_path = self
            .project_path
            .as_ref()
            .ok_or_else(|| "Save the project first to export compile output.".to_string())?;

        let mut output_path = source_path.clone();
        output_path.set_extension("runtime.eldiron.json");

        let response = self.compile_from_editor_input(CompileFromEditorInputRequest {
            output_path: Some(output_path.to_string_lossy().into_owned()),
            pretty,
            include_compiled_project_json: false,
            ..Default::default()
        });

        if !response.success {
            return Err(response
                .errors
                .first()
                .cloned()
                .unwrap_or_else(|| response.message.clone()));
        }

        Ok((output_path, response))
    }
}

impl TheTrait for Editor {
    fn new() -> Self
    where
        Self: Sized,
    {
        let mut project = Project::new();
        if let Some(bytes) = crate::Embedded::get("toml/config.toml") {
            if let Ok(source) = std::str::from_utf8(bytes.data.as_ref()) {
                project.config = source.to_string();
            }
        }

        #[cfg(all(not(target_arch = "wasm32"), feature = "self-update"))]
        let (self_update_tx, self_update_rx) = channel();

        #[cfg(all(
            not(target_arch = "wasm32"),
            feature = "self-update",
            not(target_os = "macos")
        ))]
        let self_updater = SelfUpdater::new("markusmoenig", "Eldiron", "eldiron-creator");
        #[cfg(all(
            not(target_arch = "wasm32"),
            feature = "self-update",
            target_os = "macos"
        ))]
        let self_updater = SelfUpdater::new("markusmoenig", "Eldiron", "Eldiron-Creator.app");

        Self {
            project,
            project_path: None,

            sidebar: Sidebar::new(),
            mapeditor: MapEditor::new(),

            server_ctx: ServerContext::default(),

            update_tracker: UpdateTracker::new(),
            event_receiver: None,

            #[cfg(all(not(target_arch = "wasm32"), feature = "self-update"))]
            self_update_rx,
            #[cfg(all(not(target_arch = "wasm32"), feature = "self-update"))]
            self_update_tx,
            #[cfg(all(not(target_arch = "wasm32"), feature = "self-update"))]
            self_updater: Arc::new(Mutex::new(self_updater)),

            update_counter: 0,

            build_values: ValueContainer::default(),
        }
    }

    fn init(&mut self, _ctx: &mut TheContext) {
        #[cfg(all(not(target_arch = "wasm32"), feature = "self-update"))]
        {
            let updater = Arc::clone(&self.self_updater);
            let tx = self.self_update_tx.clone();

            thread::spawn(move || {
                let mut updater = updater.lock().unwrap();

                if let Err(err) = updater.fetch_release_list() {
                    tx.send(SelfUpdateEvent::UpdateError(err.to_string()))
                        .unwrap();
                };
            });
        }
    }

    fn window_title(&self) -> String {
        "NexusStudio".to_string()
    }

    fn target_fps(&self) -> f64 {
        1000.0 / self.redraw_interval_ms() as f64
    }

    fn fonts_to_load(&self) -> Vec<TheFontScript> {
        vec![TheFontScript::Han]
    }

    fn default_window_size(&self) -> (usize, usize) {
        (1200, 720)
    }

    fn window_icon(&self) -> Option<(Vec<u8>, u32, u32)> {
        let file = Embedded::get("icons/vertex_splash.png")
            .or_else(|| Embedded::get("icons/window_logo.png"))
            .or_else(|| Embedded::get("window_logo.png"));
        if let Some(file) = file {
            let data = std::io::Cursor::new(file.data);

            let decoder = png::Decoder::new(data);
            if let Ok(mut reader) = decoder.read_info() {
                if let Some(buffer_size) = reader.output_buffer_size() {
                    let mut buf = vec![0; buffer_size];
                    let info = reader.next_frame(&mut buf).unwrap();
                    let bytes = &buf[..info.buffer_size()];

                    Some((bytes.to_vec(), info.width, info.height))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn init_ui(&mut self, ui: &mut TheUI, ctx: &mut TheContext) {
        RUSTERIX.write().unwrap().client.messages_font = ctx.ui.font.clone();

        // Embedded Icons
        for file in Embedded::iter() {
            let name = file.as_ref();

            if name.ends_with(".png") {
                if let Some(file) = Embedded::get(name) {
                    let data = std::io::Cursor::new(file.data);

                    let decoder = png::Decoder::new(data);
                    if let Ok(mut reader) = decoder.read_info() {
                        if let Some(buffer_size) = reader.output_buffer_size() {
                            let mut buf = vec![0; buffer_size];
                            let info = reader.next_frame(&mut buf).unwrap();
                            let bytes = &buf[..info.buffer_size()];

                            let mut cut_name = name.replace("icons/", "");
                            cut_name = cut_name.replace(".png", "");

                            ctx.ui.add_icon(
                                cut_name.to_string(),
                                TheRGBABuffer::from(bytes.to_vec(), info.width, info.height),
                            );
                        }
                    }
                }
            }
        }

        // ---

        ui.set_statusbar_name("Statusbar".to_string());

        let mut top_canvas = TheCanvas::new();
        // Keep the internal menu row visible in all builds.
        {
            let mut menu_canvas = TheCanvas::new();
            let mut menu = TheMenu::new(TheId::named("Menu"));

            let mut file_menu = TheContextMenu::named(fl!("menu_file"));
            file_menu.add(TheContextMenuItem::new(
                fl!("menu_new"),
                TheId::named("New"),
            ));
            file_menu.add_separator();
            file_menu.add(TheContextMenuItem::new_with_accel(
                fl!("menu_open"),
                TheId::named("Open"),
                TheAccelerator::new(TheAcceleratorKey::CTRLCMD, 'o'),
            ));
            file_menu.add(TheContextMenuItem::new_with_accel(
                fl!("menu_save"),
                TheId::named("Save"),
                TheAccelerator::new(TheAcceleratorKey::CTRLCMD, 's'),
            ));
            file_menu.add(TheContextMenuItem::new_with_accel(
                fl!("menu_save_as"),
                TheId::named("Save As"),
                TheAccelerator::new(TheAcceleratorKey::CTRLCMD, 'a'),
            ));
            let mut edit_menu = TheContextMenu::named(fl!("menu_edit"));
            edit_menu.add(TheContextMenuItem::new_with_accel(
                fl!("menu_undo"),
                TheId::named("Undo"),
                TheAccelerator::new(TheAcceleratorKey::CTRLCMD, 'z'),
            ));
            edit_menu.add(TheContextMenuItem::new_with_accel(
                fl!("menu_redo"),
                TheId::named("Redo"),
                TheAccelerator::new(TheAcceleratorKey::CTRLCMD | TheAcceleratorKey::SHIFT, 'z'),
            ));
            edit_menu.add_separator();
            edit_menu.add(TheContextMenuItem::new_with_accel(
                fl!("menu_cut"),
                TheId::named("Cut"),
                TheAccelerator::new(TheAcceleratorKey::CTRLCMD, 'x'),
            ));
            edit_menu.add(TheContextMenuItem::new_with_accel(
                fl!("menu_copy"),
                TheId::named("Copy"),
                TheAccelerator::new(TheAcceleratorKey::CTRLCMD, 'c'),
            ));
            edit_menu.add(TheContextMenuItem::new_with_accel(
                fl!("menu_paste"),
                TheId::named("Paste"),
                TheAccelerator::new(TheAcceleratorKey::CTRLCMD, 'v'),
            ));
            edit_menu.add_separator();
            edit_menu.add(TheContextMenuItem::new_with_accel(
                fl!("menu_apply_action"),
                TheId::named("Action Apply"),
                TheAccelerator::new(TheAcceleratorKey::CTRLCMD, 'p'),
            ));

            let mut game_menu = TheContextMenu::named(fl!("game"));
            game_menu.add(TheContextMenuItem::new_with_accel(
                fl!("menu_play"),
                TheId::named("Play"),
                TheAccelerator::new(TheAcceleratorKey::CTRLCMD, 'p'),
            ));
            game_menu.add(TheContextMenuItem::new_with_accel(
                fl!("menu_pause"),
                TheId::named("Pause"),
                TheAccelerator::new(TheAcceleratorKey::CTRLCMD, 'o'),
            ));
            game_menu.add(TheContextMenuItem::new_with_accel(
                fl!("menu_stop"),
                TheId::named("Stop"),
                TheAccelerator::new(TheAcceleratorKey::CTRLCMD | TheAcceleratorKey::SHIFT, 'p'),
            ));

            let mut project_menu = TheContextMenu::named("Project".to_string());
            project_menu.add(TheContextMenuItem::new(
                "Validate Project".to_string(),
                TheId::named("Project Validate"),
            ));
            project_menu.add(TheContextMenuItem::new(
                "Refresh Designer View".to_string(),
                TheId::named("Project Refresh View"),
            ));

            let mut build_compile_submenu = TheContextMenu::named("Compile".to_string());
            build_compile_submenu.add(TheContextMenuItem::new_with_accel(
                "Compile Runtime (Compact)".to_string(),
                TheId::named("Build Compile Runtime Compact"),
                TheAccelerator::new(TheAcceleratorKey::CTRLCMD | TheAcceleratorKey::SHIFT, 'b'),
            ));
            build_compile_submenu.add(TheContextMenuItem::new(
                "Compile Runtime (Pretty)".to_string(),
                TheId::named("Build Compile Runtime Pretty"),
            ));
            build_compile_submenu.add(TheContextMenuItem::new(
                "Compile Runtime To Project Folder".to_string(),
                TheId::named("Build Compile Runtime To File"),
            ));

            let mut build_menu = TheContextMenu::named("Build".to_string());
            build_menu.add(TheContextMenuItem::new_submenu(
                "Compile".to_string(),
                TheId::named("Build Compile Menu"),
                build_compile_submenu,
            ));
            build_menu.add(TheContextMenuItem::new(
                "Validate + Compile".to_string(),
                TheId::named("Build Validate And Compile"),
            ));

            let mut world_generate_submenu = TheContextMenu::named("Generate".to_string());
            world_generate_submenu.add(TheContextMenuItem::new(
                "Add Region".to_string(),
                TheId::named("World Add Region"),
            ));
            world_generate_submenu.add(TheContextMenuItem::new(
                "Add Dungeon Region".to_string(),
                TheId::named("World Add Dungeon"),
            ));
            world_generate_submenu.add(TheContextMenuItem::new(
                "Add UI Screen".to_string(),
                TheId::named("World Add Screen"),
            ));

            let mut world_menu = TheContextMenu::named("World".to_string());
            world_menu.add(TheContextMenuItem::new_submenu(
                "Generate".to_string(),
                TheId::named("World Generate Menu"),
                world_generate_submenu,
            ));
            world_menu.add(TheContextMenuItem::new(
                "Create Biome Starter Pack".to_string(),
                TheId::named("World Seed Biomes"),
            ));

            let mut content_create_submenu = TheContextMenu::named("Create".to_string());
            content_create_submenu.add(TheContextMenuItem::new(
                "Add Character Template".to_string(),
                TheId::named("Content Add Character"),
            ));
            content_create_submenu.add(TheContextMenuItem::new(
                "Add Item Template".to_string(),
                TheId::named("Content Add Item"),
            ));
            content_create_submenu.add(TheContextMenuItem::new(
                "Add Starter Character + Item".to_string(),
                TheId::named("Content Add Starter Pack"),
            ));

            let mut content_widgets_submenu = TheContextMenu::named("Widget Windows".to_string());
            content_widgets_submenu.add(TheContextMenuItem::new(
                "Create HUD Window".to_string(),
                TheId::named("Content Add Widget HUD"),
            ));
            content_widgets_submenu.add(TheContextMenuItem::new(
                "Create Inventory Window".to_string(),
                TheId::named("Content Add Widget Inventory"),
            ));
            content_widgets_submenu.add(TheContextMenuItem::new(
                "Create Dialogue Window".to_string(),
                TheId::named("Content Add Widget Dialogue"),
            ));
            content_widgets_submenu.add(TheContextMenuItem::new(
                "Create Settings Window".to_string(),
                TheId::named("Content Add Widget Settings"),
            ));
            content_widgets_submenu.add(TheContextMenuItem::new(
                "Create Widget Window Starter Pack".to_string(),
                TheId::named("Content Add Widget Starter Pack"),
            ));

            let mut content_menu = TheContextMenu::named("Content".to_string());
            content_menu.add(TheContextMenuItem::new_submenu(
                "Create".to_string(),
                TheId::named("Content Create Menu"),
                content_create_submenu,
            ));
            content_menu.add(TheContextMenuItem::new_submenu(
                "Widget Windows".to_string(),
                TheId::named("Content Widget Windows Menu"),
                content_widgets_submenu,
            ));

            file_menu.register_accel(ctx);
            edit_menu.register_accel(ctx);
            game_menu.register_accel(ctx);
            project_menu.register_accel(ctx);
            build_menu.register_accel(ctx);
            world_menu.register_accel(ctx);
            content_menu.register_accel(ctx);

            menu.add_context_menu(file_menu);
            menu.add_context_menu(edit_menu);
            menu.add_context_menu(game_menu);
            menu.add_context_menu(project_menu);
            menu.add_context_menu(build_menu);
            menu.add_context_menu(world_menu);
            menu.add_context_menu(content_menu);
            menu_canvas.set_widget(menu);
            top_canvas.set_top(menu_canvas);
        }

        let mut menubar = TheMenubar::new(TheId::named("Menubar"));
        menubar.limiter_mut().set_max_height(43 + 22);

        let mut logo_button = TheMenubarButton::new(TheId::named("Logo"));
        logo_button.set_icon_name("vertex_editor_logo".to_string());
        logo_button.set_status_text(&fl!("status_logo_button"));

        let mut open_button = TheMenubarButton::new(TheId::named("Open"));
        open_button.set_icon_name("icon_role_load".to_string());
        open_button.set_status_text(&fl!("status_open_button"));

        let mut save_button = TheMenubarButton::new(TheId::named("Save"));
        save_button.set_status_text(&fl!("status_save_button"));
        save_button.set_icon_name("icon_role_save".to_string());

        let mut save_as_button = TheMenubarButton::new(TheId::named("Save As"));
        save_as_button.set_icon_name("icon_role_save_as".to_string());
        save_as_button.set_status_text(&fl!("status_save_as_button"));
        save_as_button.set_icon_offset(Vec2::new(2, -5));

        let mut undo_button = TheMenubarButton::new(TheId::named("Undo"));
        undo_button.set_status_text(&fl!("status_undo_button"));
        undo_button.set_icon_name("icon_role_undo".to_string());

        let mut redo_button = TheMenubarButton::new(TheId::named("Redo"));
        redo_button.set_status_text(&fl!("status_redo_button"));
        redo_button.set_icon_name("icon_role_redo".to_string());

        let mut play_button = TheMenubarButton::new(TheId::named("Play"));
        play_button.set_status_text(&fl!("status_play_button"));
        play_button.set_icon_name("play".to_string());
        //play_button.set_fixed_size(vec2i(28, 28));

        let mut pause_button = TheMenubarButton::new(TheId::named("Pause"));
        pause_button.set_status_text(&fl!("status_pause_button"));
        pause_button.set_icon_name("play-pause".to_string());

        let mut stop_button = TheMenubarButton::new(TheId::named("Stop"));
        stop_button.set_status_text(&fl!("status_stop_button"));
        stop_button.set_icon_name("stop-fill".to_string());

        let mut input_button = TheMenubarButton::new(TheId::named("GameInput"));
        input_button.set_status_text(&fl!("status_game_input_button"));
        input_button.set_icon_name("keyboard".to_string());
        input_button.set_has_state(true);

        let mut time_slider = TheTimeSlider::new(TheId::named("Server Time Slider"));
        time_slider.set_status_text(&fl!("status_time_slider"));
        time_slider.set_continuous(true);
        time_slider.limiter_mut().set_max_width(400);
        time_slider.set_value(TheValue::Time(TheTime::default()));

        let mut patreon_button = TheMenubarButton::new(TheId::named("Patreon"));
        patreon_button.set_status_text(&fl!("status_patreon_button"));
        patreon_button.set_icon_name("patreon".to_string());
        // patreon_button.set_fixed_size(vec2i(36, 36));
        patreon_button.set_icon_offset(Vec2::new(-4, -2));

        let mut help_button = TheMenubarButton::new(TheId::named("Help"));
        help_button.set_status_text(&fl!("status_help_button"));
        help_button.set_icon_name("question-mark".to_string());
        help_button.set_has_state(true);
        // patreon_button.set_fixed_size(vec2i(36, 36));
        help_button.set_icon_offset(Vec2::new(-2, -2));

        #[cfg(all(not(target_arch = "wasm32"), feature = "self-update"))]
        let mut update_button = {
            let mut button = TheMenubarButton::new(TheId::named("Update"));
            button.set_status_text(&fl!("status_update_button"));
            button.set_icon_name("arrows-clockwise".to_string());
            button
        };

        let mut hlayout = TheHLayout::new(TheId::named("Menu Layout"));
        hlayout.set_background_color(None);
        hlayout.set_margin(Vec4::new(10, 2, 10, 1));
        hlayout.add_widget(Box::new(logo_button));
        hlayout.add_widget(Box::new(TheMenubarSeparator::new(TheId::empty())));
        hlayout.add_widget(Box::new(open_button));
        hlayout.add_widget(Box::new(save_button));
        hlayout.add_widget(Box::new(save_as_button));
        hlayout.add_widget(Box::new(TheMenubarSeparator::new(TheId::empty())));
        hlayout.add_widget(Box::new(undo_button));
        hlayout.add_widget(Box::new(redo_button));
        hlayout.add_widget(Box::new(TheMenubarSeparator::new(TheId::empty())));
        hlayout.add_widget(Box::new(play_button));
        hlayout.add_widget(Box::new(pause_button));
        hlayout.add_widget(Box::new(stop_button));
        hlayout.add_widget(Box::new(input_button));
        hlayout.add_widget(Box::new(TheMenubarSeparator::new(TheId::empty())));
        hlayout.add_widget(Box::new(time_slider));
        //hlayout.add_widget(Box::new(TheMenubarSeparator::new(TheId::empty())));

        #[cfg(all(not(target_arch = "wasm32"), feature = "self-update"))]
        {
            hlayout.add_widget(Box::new(update_button));
            hlayout.add_widget(Box::new(TheMenubarSeparator::new(TheId::empty())));
            hlayout.add_widget(Box::new(patreon_button));
            hlayout.set_reverse_index(Some(3));
        }

        #[cfg(not(all(not(target_arch = "wasm32"), feature = "self-update")))]
        {
            hlayout.add_widget(Box::new(patreon_button));
            hlayout.add_widget(Box::new(help_button));
            hlayout.set_reverse_index(Some(2));
        }

        top_canvas.set_widget(menubar);
        top_canvas.set_layout(hlayout);
        ui.canvas.set_top(top_canvas);

        // Sidebar
        self.sidebar.init_ui(ui, ctx, &mut self.server_ctx);

        // Docks
        let bottom_panels = DOCKMANAGER.write().unwrap().init(ctx);

        let mut editor_canvas: TheCanvas = TheCanvas::new();

        let mut editor_stack = TheStackLayout::new(TheId::named("Editor Stack"));
        let poly_canvas = self.mapeditor.init_ui(ui, ctx, &mut self.project);
        editor_stack.add_canvas(poly_canvas);

        // Add Dock Editors
        DOCKMANAGER
            .write()
            .unwrap()
            .add_editors_to_stack(&mut editor_stack, ctx);

        editor_canvas.set_layout(editor_stack);

        // Main V Layout
        let mut vsplitlayout = TheSharedVLayout::new(TheId::named("Shared VLayout"));
        vsplitlayout.add_canvas(editor_canvas);
        vsplitlayout.add_canvas(bottom_panels);
        vsplitlayout.set_shared_ratio(0.68);
        vsplitlayout.set_mode(TheSharedVLayoutMode::Shared);

        let mut shared_canvas = TheCanvas::new();
        shared_canvas.set_layout(vsplitlayout);

        let mut workspace_top = TheCanvas::new();
        workspace_top.set_widget(TheTraybar::new(TheId::empty()));

        let mut workspace_actions = TheHLayout::new(TheId::named("Workspace Actions"));
        workspace_actions.set_margin(Vec4::new(8, 2, 8, 2));
        workspace_actions.set_padding(2);

        let mut workspace_title = TheText::new(TheId::named("Workspace Title"));
        workspace_title.set_text("Vertex Workspace".to_string());
        workspace_title.limiter_mut().set_max_width(200);
        workspace_actions.add_widget(Box::new(workspace_title));
        workspace_actions.add_widget(Box::new(TheHDivider::new(TheId::empty())));

        let mut validate_button = TheTraybarButton::new(TheId::named("Project Validate"));
        validate_button.set_icon_name("info".to_string());
        validate_button.set_status_text("Validate project consistency");
        validate_button.set_fixed_size(true);
        workspace_actions.add_widget(Box::new(validate_button));

        let mut build_button = TheTraybarButton::new(TheId::named("Build Validate And Compile"));
        build_button.set_icon_name("export".to_string());
        build_button.set_status_text("Validate and compile runtime output");
        build_button.set_fixed_size(true);
        workspace_actions.add_widget(Box::new(build_button));

        let mut widgets_button =
            TheTraybarButton::new(TheId::named("Content Add Widget Starter Pack"));
        widgets_button.set_icon_name("treasure-chest".to_string());
        widgets_button.set_status_text("Create starter widget windows");
        widgets_button.set_fixed_size(true);
        workspace_actions.add_widget(Box::new(widgets_button));

        let mut refresh_button = TheTraybarButton::new(TheId::named("Project Refresh View"));
        refresh_button.set_icon_name("arrows-clockwise".to_string());
        refresh_button.set_status_text("Refresh workspace content");
        refresh_button.set_fixed_size(true);
        workspace_actions.add_widget(Box::new(refresh_button));

        workspace_top.set_layout(workspace_actions);
        shared_canvas.set_top(workspace_top);

        // Tool List
        let mut tool_list_canvas: TheCanvas = TheCanvas::new();

        let mut tool_list_bar_canvas = TheCanvas::new();
        tool_list_bar_canvas.set_widget(TheToolListBar::new(TheId::empty()));
        tool_list_canvas.set_top(tool_list_bar_canvas);

        let mut v_tool_list_layout = TheVLayout::new(TheId::named("Tool List Layout"));
        v_tool_list_layout.limiter_mut().set_max_width(64);
        v_tool_list_layout.set_margin(Vec4::new(4, 4, 4, 4));
        v_tool_list_layout.set_padding(2);

        TOOLLIST
            .write()
            .unwrap()
            .set_active_editor(&mut v_tool_list_layout, ctx);

        tool_list_canvas.set_layout(v_tool_list_layout);

        let mut tool_list_border_canvas = TheCanvas::new();
        let mut border_widget = TheIconView::new(TheId::empty());
        border_widget.set_border_color(Some([92, 92, 92, 255]));
        border_widget.limiter_mut().set_max_width(1);
        border_widget.limiter_mut().set_max_height(i32::MAX);
        tool_list_border_canvas.set_widget(border_widget);

        tool_list_canvas.set_right(tool_list_border_canvas);
        shared_canvas.set_left(tool_list_canvas);

        ui.canvas.set_center(shared_canvas);

        let mut status_canvas = TheCanvas::new();
        let mut statusbar = TheStatusbar::new(TheId::named("Statusbar"));
        statusbar.set_text(fl!("info_welcome"));
        status_canvas.set_widget(statusbar);

        ui.canvas.set_bottom(status_canvas);

        // -

        // ctx.ui.set_disabled("Save");
        // ctx.ui.set_disabled("Save As");
        ctx.ui.set_disabled("Undo");
        ctx.ui.set_disabled("Redo");

        // Init Rusterix

        if let Some(icon) = ctx.ui.icon("light_on") {
            let texture = Texture::from_rgbabuffer(icon);
            self.build_values.set("light_on", Value::Texture(texture));
        }
        if let Some(icon) = ctx.ui.icon("light_off") {
            let texture = Texture::from_rgbabuffer(icon);
            self.build_values.set("light_off", Value::Texture(texture));
        }
        if let Some(icon) = ctx.ui.icon("character_on") {
            let texture = Texture::from_rgbabuffer(icon);
            self.build_values
                .set("character_on", Value::Texture(texture));
        }
        if let Some(icon) = ctx.ui.icon("character_off") {
            let texture = Texture::from_rgbabuffer(icon);
            self.build_values
                .set("character_off", Value::Texture(texture));
        }
        if let Some(icon) = ctx.ui.icon("treasure_on") {
            let texture = Texture::from_rgbabuffer(icon);
            self.build_values
                .set("treasure_on", Value::Texture(texture));
        }
        if let Some(icon) = ctx.ui.icon("treasure_off") {
            let texture = Texture::from_rgbabuffer(icon);
            self.build_values
                .set("treasure_off", Value::Texture(texture));
        }

        RUSTERIX
            .write()
            .unwrap()
            .client
            .builder_d2
            .set_properties(&self.build_values);
        RUSTERIX.write().unwrap().set_d2();
        SCENEMANAGER.write().unwrap().startup();

        self.event_receiver = Some(ui.add_state_listener("Main Receiver".into()));
    }

    /// Set the command line arguments
    fn set_cmd_line_args(&mut self, args: Vec<String>, ctx: &mut TheContext) {
        if args.len() > 1 {
            #[allow(irrefutable_let_patterns)]
            if let Ok(path) = PathBuf::from_str(&args[1]) {
                ctx.ui.send(TheEvent::FileRequesterResult(
                    TheId::named("Open"),
                    vec![path],
                ));
                return;
            }
        }

        ctx.ui.send(TheEvent::StateChanged(
            TheId::named("New"),
            TheWidgetState::Clicked,
        ));
    }

    /// Handle UI events and UI state
    fn update_ui(&mut self, ui: &mut TheUI, ctx: &mut TheContext) -> bool {
        let mut redraw = false;
        let mut update_server_icons = false;

        // Make sure on first startup the active tool is properly selected
        if self.update_counter == 0 {
            let mut toollist = TOOLLIST.write().unwrap();
            let id = toollist.get_current_tool().id().uuid;

            toollist.set_tool(id, ui, ctx, &mut self.project, &mut self.server_ctx);
        }

        // Get build results from the scene manager if any
        while let Some(result) = SCENEMANAGER.write().unwrap().receive() {
            match result {
                SceneManagerResult::Startup => {
                    println!("Scene manager has started up.");
                }
                SceneManagerResult::ProcessedHeights(coord, heights) => {
                    if let Some(map) = &mut self.project.get_map_mut(&self.server_ctx) {
                        let local = map.terrain.get_chunk_coords(coord.x, coord.y);
                        if let Some(chunk) = &mut map.terrain.chunks.get_mut(&local) {
                            chunk.processed_heights = Some(heights);
                        }
                    }
                }
                SceneManagerResult::Chunk(chunk, togo, total, billboards) => {
                    if togo == 0 {
                        self.server_ctx.background_progress = None;
                    } else {
                        self.server_ctx.background_progress = Some(format!("{togo}/{total}"));
                    }

                    let mut rusterix = RUSTERIX.write().unwrap();

                    rusterix
                        .scene_handler
                        .vm
                        .execute(scenevm::Atom::RemoveChunkAt {
                            origin: chunk.origin,
                        });

                    rusterix.scene_handler.vm.execute(scenevm::Atom::AddChunk {
                        id: Uuid::new_v4(),
                        chunk: chunk,
                    });

                    // Add billboards to scene_handler (indexed by GeoId)
                    for billboard in billboards {
                        rusterix
                            .scene_handler
                            .billboards
                            .insert(billboard.geo_id, billboard);
                    }

                    ctx.ui.send(TheEvent::Custom(
                        TheId::named("Update Minimap"),
                        TheValue::Empty,
                    ));
                }
                SceneManagerResult::UpdatedBatch3D(coord, batch) => {
                    let mut rusterix = RUSTERIX.write().unwrap();
                    if let Some(chunk) = rusterix.client.scene.chunks.get_mut(&coord) {
                        chunk.terrain_batch3d = Some(batch);
                    }
                }
                SceneManagerResult::Clear => {
                    let mut rusterix = RUSTERIX.write().unwrap();
                    rusterix
                        .scene_handler
                        .vm
                        .execute(scenevm::Atom::ClearGeometry);

                    rusterix.scene_handler.billboards.clear();
                }
                SceneManagerResult::Quit => {
                    println!("Scene manager has shutdown.");
                }
            }
        }

        // Check for redraw (30fps) and tick updates
        let redraw_ms = self.redraw_interval_ms();
        let tick_ms = CONFIGEDITOR.read().unwrap().game_tick_ms.max(1) as u64;
        let (mut redraw_update, tick_update) = self.update_tracker.update(redraw_ms, tick_ms);

        // Handle queued UI events in the same update pass so input can trigger immediate redraw work.
        let mut pending_events = Vec::new();
        if let Some(receiver) = &mut self.event_receiver {
            while let Ok(event) = receiver.try_recv() {
                pending_events.push(event);
            }
        }
        if !pending_events.is_empty() {
            redraw_update = true;
        }

        if tick_update {
            RUSTERIX.write().unwrap().client.inc_animation_frame();

            self.server_ctx.animation_counter = self.server_ctx.animation_counter.wrapping_add(1);
            // To update animated minimaps (only for docks that need it)
            if DOCKMANAGER
                .read()
                .unwrap()
                .current_dock_supports_minimap_animation()
            {
                ctx.ui.send(TheEvent::Custom(
                    TheId::named("Soft Update Minimap"),
                    TheValue::Empty,
                ));
            }

            if RUSTERIX.read().unwrap().server.state == rusterix::ServerState::Running {
                INFOVIEWER
                    .write()
                    .unwrap()
                    .update(&self.project, ui, ctx, &self.server_ctx);
            }
        }

        if redraw_update && !self.project.regions.is_empty() {
            SCENEMANAGER.write().unwrap().tick();

            self.build_values.set(
                "no_rect_geo",
                Value::Bool(self.server_ctx.no_rect_geo_on_map),
            );

            extract_build_values_from_config(&mut self.build_values);

            let mut messages = Vec::new();
            let mut choices = Vec::new();

            // Update entities when the server is running
            {
                let rusterix = &mut RUSTERIX.write().unwrap();
                if rusterix.server.state == rusterix::ServerState::Running {
                    // Send a game tick to all servers
                    if tick_update {
                        rusterix.server.system_tick();
                    }

                    // Send a redraw tick to all servers
                    if redraw_update {
                        rusterix.server.redraw_tick();
                    }

                    if let Some(new_region_name) = rusterix.update_server() {
                        rusterix.client.current_map = new_region_name;
                    }
                    if rusterix.server.log_changed {
                        ui.set_widget_value(
                            "LogEdit",
                            ctx,
                            TheValue::Text(rusterix.server.get_log()),
                        );
                    }
                    for r in &mut self.project.regions {
                        rusterix.server.apply_entities_items(&mut r.map);

                        if r.id == self.server_ctx.curr_region {
                            if let Some(time) = rusterix.server.get_time(&r.map.id) {
                                rusterix.client.set_server_time(time);
                                if let Some(widget) = ui.get_widget("Server Time Slider") {
                                    widget.set_value(TheValue::Time(rusterix.client.server_time));
                                }
                            }

                            rusterix::tile_builder(&mut r.map, &mut rusterix.assets);
                            messages = rusterix.server.get_messages(&r.map.id);
                            choices = rusterix.server.get_choices(&r.map.id);

                            // Redraw the nodes
                            match &self.server_ctx.cc {
                                ContentContext::CharacterInstance(uuid) => {
                                    for entity in r.map.entities.iter() {
                                        if entity.creator_id == *uuid {
                                            CODEGRIDFX.write().unwrap().redraw_debug(
                                                ui,
                                                ctx,
                                                entity.id,
                                                &rusterix.server.debug,
                                            );
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }

            if self.server_ctx.world_mode {
                // Draw World Editor
                WORLDEDITOR.write().unwrap().draw(
                    ui,
                    ctx,
                    &mut self.project,
                    &mut self.server_ctx,
                    &mut self.build_values,
                );
            } else {
                // Draw Map
                if let Some(render_view) = ui.get_render_view("PolyView") {
                    let dim = *render_view.dim();

                    let buffer = render_view.render_buffer_mut();
                    buffer.resize(dim.width, dim.height);

                    {
                        // If we are drawing billboard vertices in the geometry overlay, update them.
                        if !self.server_ctx.game_mode
                            && self.server_ctx.editor_view_mode != EditorViewMode::D2
                            && self.server_ctx.curr_map_tool_type == MapToolType::Vertex
                        {
                            TOOLLIST.write().unwrap().update_geometry_overlay_3d(
                                &mut self.project,
                                &mut self.server_ctx,
                            );
                        }

                        let rusterix = &mut RUSTERIX.write().unwrap();
                        let is_running = rusterix.server.state == rusterix::ServerState::Running;
                        let b = &mut rusterix.client.builder_d2;

                        if is_running && self.server_ctx.game_mode {
                            for r in &mut self.project.regions {
                                if r.map.name == rusterix.client.current_map {
                                    rusterix.draw_game(&r.map, messages, choices);
                                    break;
                                }
                            }

                            rusterix
                                .client
                                .insert_game_buffer(render_view.render_buffer_mut());
                        } else {
                            if self.server_ctx.editor_view_mode != EditorViewMode::D2
                                && self.server_ctx.get_map_context() == MapContext::Region
                            {
                                RENDEREDITOR.write().unwrap().draw(
                                    render_view,
                                    ctx,
                                    &mut self.project,
                                    &mut self.server_ctx,
                                    rusterix,
                                );
                            } else
                            // Draw the region map
                            if self.server_ctx.get_map_context() == MapContext::Region
                                && self.server_ctx.editing_surface.is_none()
                            {
                                if let Some(region) =
                                    self.project.get_region(&self.server_ctx.curr_region)
                                {
                                    b.set_clip_rect(None);
                                    b.set_map_tool_type(self.server_ctx.curr_map_tool_type);
                                    if let Some(hover_cursor) = self.server_ctx.hover_cursor {
                                        b.set_map_hover_info(
                                            self.server_ctx.hover,
                                            Some(vek::Vec2::new(hover_cursor.x, hover_cursor.y)),
                                        );
                                    } else {
                                        b.set_map_hover_info(self.server_ctx.hover, None);
                                    }

                                    if let Some(camera_pos) = region.map.camera_xz {
                                        b.set_camera_info(
                                            Some(Vec3::new(camera_pos.x, 0.0, camera_pos.y)),
                                            None,
                                        );
                                    }

                                    // let start_time = ctx.get_time();

                                    if let Some(clipboard) = &self.server_ctx.paste_clipboard {
                                        // During a paste operation we use a merged map

                                        let mut map = region.map.clone();
                                        if let Some(hover) = self.server_ctx.hover_cursor {
                                            map.paste_at_position(clipboard, hover);
                                        }

                                        rusterix.set_dirty();
                                        // rusterix.build_scene(
                                        //     Vec2::new(dim.width as f32, dim.height as f32),
                                        //     &map,
                                        //     &self.build_values,
                                        //     self.server_ctx.game_mode,
                                        // );
                                        rusterix.apply_entities_items(
                                            Vec2::new(dim.width as f32, dim.height as f32),
                                            &map,
                                            &self.server_ctx.editing_surface,
                                            false,
                                        );
                                    } else {
                                        // rusterix.build_scene(
                                        //     Vec2::new(dim.width as f32, dim.height as f32),
                                        //     &region.map,
                                        //     &self.build_values,
                                        //     self.server_ctx.game_mode,
                                        // );

                                        if let Some(map) = self.project.get_map(&self.server_ctx) {
                                            rusterix.apply_entities_items(
                                                Vec2::new(dim.width as f32, dim.height as f32),
                                                map,
                                                &self.server_ctx.editing_surface,
                                                false,
                                            );
                                        }
                                    }

                                    // Prepare the messages for the region for drawing
                                    rusterix.process_messages(&region.map, messages);

                                    // let stop_time = ctx.get_time();
                                    //println!("{} ms", stop_time - start_time);
                                }

                                if let Some(map) = self.project.get_map_mut(&self.server_ctx) {
                                    rusterix.draw_scene(
                                        map,
                                        render_view.render_buffer_mut().pixels_mut(),
                                        dim.width as usize,
                                        dim.height as usize,
                                    );
                                }
                            } else if self.server_ctx.get_map_context() == MapContext::Region
                                && self.server_ctx.editing_surface.is_some()
                            {
                                b.set_map_tool_type(self.server_ctx.curr_map_tool_type);
                                if let Some(profile) = self.project.get_map_mut(&self.server_ctx) {
                                    if let Some(hover_cursor) = self.server_ctx.hover_cursor {
                                        b.set_map_hover_info(
                                            self.server_ctx.hover,
                                            Some(vek::Vec2::new(hover_cursor.x, hover_cursor.y)),
                                        );
                                    } else {
                                        b.set_map_hover_info(self.server_ctx.hover, None);
                                    }

                                    if let Some(clipboard) = &self.server_ctx.paste_clipboard {
                                        // During a paste operation we use a merged map
                                        let mut map = profile.clone();
                                        if let Some(hover) = self.server_ctx.hover_cursor {
                                            map.paste_at_position(clipboard, hover);
                                        }
                                        rusterix.set_dirty();
                                        rusterix.build_custom_scene_d2(
                                            Vec2::new(dim.width as f32, dim.height as f32),
                                            &map,
                                            &self.build_values,
                                            &self.server_ctx.editing_surface,
                                            true,
                                        );
                                        rusterix.draw_custom_d2(
                                            &map,
                                            render_view.render_buffer_mut().pixels_mut(),
                                            dim.width as usize,
                                            dim.height as usize,
                                        );
                                    } else {
                                        rusterix.build_custom_scene_d2(
                                            Vec2::new(dim.width as f32, dim.height as f32),
                                            profile,
                                            &self.build_values,
                                            &self.server_ctx.editing_surface,
                                            true,
                                        );
                                        rusterix.draw_custom_d2(
                                            profile,
                                            render_view.render_buffer_mut().pixels_mut(),
                                            dim.width as usize,
                                            dim.height as usize,
                                        );
                                    }
                                }
                            } else
                            // Draw the screen / character / item map
                            if self.server_ctx.get_map_context() == MapContext::Character
                                || self.server_ctx.get_map_context() == MapContext::Item
                                || self.server_ctx.get_map_context() == MapContext::Screen
                            {
                                b.set_map_tool_type(self.server_ctx.curr_map_tool_type);
                                if let Some(map) = self.project.get_map_mut(&self.server_ctx) {
                                    if let Some(hover_cursor) = self.server_ctx.hover_cursor {
                                        b.set_map_hover_info(
                                            self.server_ctx.hover,
                                            Some(vek::Vec2::new(hover_cursor.x, hover_cursor.y)),
                                        );
                                    } else {
                                        b.set_map_hover_info(self.server_ctx.hover, None);
                                    }

                                    if self.server_ctx.get_map_context() != MapContext::Screen {
                                        b.set_clip_rect(Some(rusterix::Rect {
                                            x: -5.0,
                                            y: -5.0,
                                            width: 10.0,
                                            height: 10.0,
                                        }));
                                    } else {
                                        let viewport = CONFIGEDITOR.read().unwrap().viewport;
                                        let grid_size =
                                            CONFIGEDITOR.read().unwrap().grid_size as f32;
                                        let w = viewport.x as f32 / grid_size;
                                        let h = viewport.y as f32 / grid_size;
                                        b.set_clip_rect(Some(rusterix::Rect {
                                            x: -w / 2.0,
                                            y: -h / 2.0,
                                            width: w,
                                            height: h,
                                        }));
                                    }

                                    if let Some(clipboard) = &self.server_ctx.paste_clipboard {
                                        // During a paste operation we use a merged map
                                        let mut map = map.clone();
                                        if let Some(hover) = self.server_ctx.hover_cursor {
                                            map.paste_at_position(clipboard, hover);
                                        }
                                        rusterix.set_dirty();
                                        rusterix.build_custom_scene_d2(
                                            Vec2::new(dim.width as f32, dim.height as f32),
                                            &map,
                                            &self.build_values,
                                            &self.server_ctx.editing_surface,
                                            true,
                                        );
                                        rusterix.draw_custom_d2(
                                            &map,
                                            render_view.render_buffer_mut().pixels_mut(),
                                            dim.width as usize,
                                            dim.height as usize,
                                        );
                                    } else {
                                        rusterix.build_custom_scene_d2(
                                            Vec2::new(dim.width as f32, dim.height as f32),
                                            map,
                                            &self.build_values,
                                            &None,
                                            true,
                                        );
                                        rusterix.draw_custom_d2(
                                            map,
                                            render_view.render_buffer_mut().pixels_mut(),
                                            dim.width as usize,
                                            dim.height as usize,
                                        );
                                    }
                                }
                            }
                        }
                    }
                    if !self.server_ctx.game_mode {
                        if let Some(map) = self.project.get_map_mut(&self.server_ctx) {
                            TOOLLIST.write().unwrap().draw_hud(
                                render_view.render_buffer_mut(),
                                map,
                                ctx,
                                &mut self.server_ctx,
                                &RUSTERIX.read().unwrap().assets,
                            );
                        }
                    }
                }
            }

            // Draw the 3D Preview if active.
            // if !self.server_ctx.game_mode
            //     && self.server_ctx.curr_map_tool_helper == MapToolHelper::Preview
            // {
            //     if let Some(region) = self.project.get_region_ctx(&self.server_ctx) {
            //         PREVIEWVIEW
            //             .write()
            //             .unwrap()
            //             .draw(region, ui, ctx, &mut self.server_ctx);
            //     }
            // }

            redraw = true;
        }

        for event in pending_events {
            if self.server_ctx.game_input_mode {
                // In game input mode send events to the game tool
                if let Some(game_tool) =
                    TOOLLIST.write().unwrap().get_game_tool_of_name("Game Tool")
                {
                    redraw = game_tool.handle_event(
                        &event,
                        ui,
                        ctx,
                        &mut self.project,
                        &mut self.server_ctx,
                    );
                }
            }
            if self
                .sidebar
                .handle_event(&event, ui, ctx, &mut self.project, &mut self.server_ctx)
            {
                redraw = true;
            }
            if TOOLLIST.write().unwrap().handle_event(
                &event,
                ui,
                ctx,
                &mut self.project,
                &mut self.server_ctx,
            ) {
                redraw = true;
            }
            if DOCKMANAGER.write().unwrap().handle_event(
                &event,
                ui,
                ctx,
                &mut self.project,
                &mut self.server_ctx,
            ) {
                redraw = true;
            }
            if self
                .mapeditor
                .handle_event(&event, ui, ctx, &mut self.project, &mut self.server_ctx)
            {
                redraw = true;
            }
            if TILEMAPEDITOR.write().unwrap().handle_event(
                &event,
                ui,
                ctx,
                &mut self.project,
                &mut self.server_ctx,
            ) {
                redraw = true;
            }
            match event {
                TheEvent::CustomUndo(id, p, n) => {
                    if id.name == "ModuleUndo" {
                        if CODEEDITOR.read().unwrap().active_panel == VisibleCodePanel::Shade {
                            let prev = Module::from_json(&p);
                            let next = Module::from_json(&n);

                            let atom = MaterialUndoAtom::ShaderEdit(prev, next);
                            UNDOMANAGER.write().unwrap().add_material_undo(atom, ctx);
                        } else if CODEEDITOR.read().unwrap().active_panel == VisibleCodePanel::Code
                        {
                            let prev = Module::from_json(&p);
                            let next = Module::from_json(&n);
                            match CODEEDITOR.read().unwrap().code_content {
                                ContentContext::CharacterTemplate(id) => {
                                    let atom =
                                        CharacterUndoAtom::TemplateModuleEdit(id, prev, next);
                                    UNDOMANAGER.write().unwrap().add_character_undo(atom, ctx);
                                }
                                ContentContext::CharacterInstance(id) => {
                                    let atom = CharacterUndoAtom::InstanceModuleEdit(
                                        self.server_ctx.curr_region,
                                        id,
                                        prev,
                                        next,
                                    );
                                    UNDOMANAGER.write().unwrap().add_character_undo(atom, ctx);
                                }
                                ContentContext::ItemTemplate(id) => {
                                    let atom: ItemUndoAtom =
                                        ItemUndoAtom::TemplateModuleEdit(id, prev, next);
                                    UNDOMANAGER.write().unwrap().add_item_undo(atom, ctx);
                                }
                                ContentContext::ItemInstance(id) => {
                                    let atom = ItemUndoAtom::InstanceModuleEdit(
                                        self.server_ctx.curr_region,
                                        id,
                                        prev,
                                        next,
                                    );
                                    UNDOMANAGER.write().unwrap().add_item_undo(atom, ctx);
                                }
                                _ => {}
                            }
                        }
                    }
                }
                TheEvent::Custom(id, value) => {
                    if id.name == "Show Help" {
                        if let TheValue::Text(url) = value {
                            _ = open::that(format!("https://www.eldiron.com/{}", url));
                            ctx.ui
                                .set_widget_state("Help".to_string(), TheWidgetState::None);
                            ctx.ui.clear_hover();
                            self.server_ctx.help_mode = false;
                            redraw = true;
                        }
                    }
                    if id.name == "Set Project Undo State" {
                        UNDOMANAGER.read().unwrap().set_undo_state_to_ui(ctx);
                    } else if id.name == "Render SceneManager Map" {
                        if self.server_ctx.pc.is_region() {
                            if self.server_ctx.editor_view_mode == EditorViewMode::D2
                                && self.server_ctx.profile_view.is_some()
                            {
                            } else {
                                crate::utils::scenemanager_render_map(
                                    &self.project,
                                    &self.server_ctx,
                                );
                                if self.server_ctx.editor_view_mode != EditorViewMode::D2 {
                                    TOOLLIST.write().unwrap().update_geometry_overlay_3d(
                                        &mut self.project,
                                        &mut self.server_ctx,
                                    );
                                }
                            }
                        }
                    } else if id.name == "Tool Changed" {
                        TOOLLIST
                            .write()
                            .unwrap()
                            .update_geometry_overlay_3d(&mut self.project, &mut self.server_ctx);
                    } else if id.name == "Update Client Properties" {
                        let mut rusterix = RUSTERIX.write().unwrap();
                        self.build_values.set(
                            "no_rect_geo",
                            rusterix::Value::Bool(self.server_ctx.no_rect_geo_on_map),
                        );
                        self.build_values.set(
                            "editing_slice",
                            rusterix::Value::Float(self.server_ctx.editing_slice),
                        );
                        rusterix
                            .client
                            .builder_d2
                            .set_properties(&self.build_values);
                        rusterix.set_dirty();
                    }
                }

                TheEvent::DialogValueOnClose(role, name, uuid, _value) => {
                    if name == "Delete Character Instance ?" {
                        if role == TheDialogButtonRole::Delete {
                            if let Some(region) =
                                self.project.get_region_mut(&self.server_ctx.curr_region)
                            {
                                let character_id = uuid;
                                if region.characters.shift_remove(&character_id).is_some() {
                                    self.server_ctx.curr_region_content = ContentContext::Unknown;
                                    region.map.selected_entity_item = None;
                                    redraw = true;

                                    // Remove from the content list
                                    if let Some(list) = ui.get_list_layout("Region Content List") {
                                        list.remove(TheId::named_with_id(
                                            "Region Content List Item",
                                            character_id,
                                        ));
                                        ui.select_first_list_item("Region Content List", ctx);
                                        ctx.ui.relayout = true;
                                    }
                                    insert_content_into_maps(&mut self.project);
                                    RUSTERIX.write().unwrap().set_dirty();
                                }
                            }
                        }
                    } else if name == "Delete Item Instance ?" {
                        if role == TheDialogButtonRole::Delete {
                            if let Some(region) =
                                self.project.get_region_mut(&self.server_ctx.curr_region)
                            {
                                let item_id = uuid;
                                if region.items.shift_remove(&item_id).is_some() {
                                    self.server_ctx.curr_region_content = ContentContext::Unknown;
                                    redraw = true;

                                    // Remove from the content list
                                    if let Some(list) = ui.get_list_layout("Region Content List") {
                                        list.remove(TheId::named_with_id(
                                            "Region Content List Item",
                                            item_id,
                                        ));
                                        ui.select_first_list_item("Region Content List", ctx);
                                        ctx.ui.relayout = true;
                                    }
                                    insert_content_into_maps(&mut self.project);
                                    RUSTERIX.write().unwrap().set_dirty();
                                }
                            }
                        }
                    } else if name == "Update NexusStudio" && role == TheDialogButtonRole::Accept {
                        #[cfg(all(not(target_arch = "wasm32"), feature = "self-update"))]
                        {
                            let updater = self.self_updater.lock().unwrap();

                            if updater.has_newer_release() {
                                let release = updater.latest_release().cloned().unwrap();

                                let updater = Arc::clone(&self.self_updater);
                                let tx = self.self_update_tx.clone();

                                self.self_update_tx
                                    .send(SelfUpdateEvent::UpdateStart(release.clone()))
                                    .unwrap();

                                thread::spawn(move || {
                                    match updater.lock().unwrap().update_latest() {
                                        Ok(status) => match status {
                                            self_update::Status::UpToDate(_) => {
                                                tx.send(SelfUpdateEvent::AlreadyUpToDate).unwrap();
                                            }
                                            self_update::Status::Updated(_) => {
                                                tx.send(SelfUpdateEvent::UpdateCompleted(release))
                                                    .unwrap();
                                            }
                                        },
                                        Err(err) => {
                                            tx.send(SelfUpdateEvent::UpdateError(err.to_string()))
                                                .unwrap();
                                        }
                                    }
                                });
                            } else {
                                self.self_update_tx
                                    .send(SelfUpdateEvent::AlreadyUpToDate)
                                    .unwrap();
                            }
                        }
                    }
                }
                TheEvent::RenderViewDrop(_id, location, drop) => {
                    if drop.id.name.starts_with("Shader") {
                        if self.server_ctx.curr_map_tool_helper == MapToolHelper::ShaderEditor
                            && CODEEDITOR.read().unwrap().active_panel == VisibleCodePanel::Shade
                        {
                            if matches!(
                                CODEEDITOR.read().unwrap().shader_content,
                                ContentContext::Sector(_)
                            ) {
                                if let Some(shader) = self.project.shaders.get(&drop.id.uuid) {
                                    let prev = SHADEGRIDFX.read().unwrap().clone();
                                    if SHADEGRIDFX.write().unwrap().insert_module(shader, location)
                                    {
                                        ctx.ui.send(TheEvent::Custom(
                                            TheId::named("ModuleChanged"),
                                            TheValue::Empty,
                                        ));
                                        ctx.ui.send(TheEvent::CustomUndo(
                                            TheId::named("ModuleUndo"),
                                            prev.to_json(),
                                            SHADEGRIDFX.read().unwrap().to_json(),
                                        ));
                                    }
                                }
                            }
                        }

                        return true;
                    }

                    let mut grid_pos = Vec2::zero();

                    if let Some(map) = self.project.get_map(&self.server_ctx) {
                        if let Some(render_view) = ui.get_render_view("PolyView") {
                            let dim = *render_view.dim();
                            grid_pos = self.server_ctx.local_to_map_cell(
                                Vec2::new(dim.width as f32, dim.height as f32),
                                Vec2::new(location.x as f32, location.y as f32),
                                map,
                                map.subdivisions,
                            );
                            grid_pos += 0.5;
                        }
                    }

                    if drop.id.name.starts_with("Character") {
                        let mut instance = Character {
                            character_id: drop.id.references,
                            position: Vec3::new(grid_pos.x, 1.5, grid_pos.y),
                            ..Default::default()
                        };

                        if let Some(bytes) = crate::Embedded::get("python/instcharacter.py") {
                            if let Ok(source) = std::str::from_utf8(bytes.data.as_ref()) {
                                instance.source = source.to_string();
                            }
                        }

                        let mut name = "Character".to_string();
                        if let Some(character) = self.project.characters.get(&drop.id.references) {
                            name.clone_from(&character.name);
                        }
                        instance.name = name.clone();

                        let atom = ProjectUndoAtom::AddRegionCharacterInstance(
                            self.server_ctx.curr_region,
                            instance,
                        );
                        atom.redo(&mut self.project, ui, ctx, &mut self.server_ctx);
                        UNDOMANAGER.write().unwrap().add_undo(atom, ctx);
                    } else if drop.id.name.starts_with("Item") {
                        let mut instance = Item {
                            item_id: drop.id.references,
                            position: Vec3::new(grid_pos.x, 1.5, grid_pos.y),
                            ..Default::default()
                        };

                        if let Some(bytes) = crate::Embedded::get("python/institem.py") {
                            if let Ok(source) = std::str::from_utf8(bytes.data.as_ref()) {
                                instance.source = source.to_string();
                            }
                        }

                        let mut name = "Item".to_string();
                        if let Some(item) = self.project.items.get(&drop.id.references) {
                            name.clone_from(&item.name);
                        }
                        instance.name = name;

                        let atom = ProjectUndoAtom::AddRegionItemInstance(
                            self.server_ctx.curr_region,
                            instance,
                        );
                        atom.redo(&mut self.project, ui, ctx, &mut self.server_ctx);
                        UNDOMANAGER.write().unwrap().add_undo(atom, ctx);
                    }
                }
                /*
                TheEvent::TileEditorDrop(_id, location, drop) => {
                    if drop.id.name.starts_with("Character") {
                        let mut instance = TheCodeBundle::new();

                        let mut init = TheCodeGrid {
                            name: "init".into(),
                            ..Default::default()
                        };
                        init.insert_atom(
                            (0, 0),
                            TheCodeAtom::Set(
                                "@self.position".to_string(),
                                TheValueAssignment::Assign,
                            ),
                        );
                        init.insert_atom(
                            (1, 0),
                            TheCodeAtom::Assignment(TheValueAssignment::Assign),
                        );
                        init.insert_atom(
                            (2, 0),
                            TheCodeAtom::Value(TheValue::Position(Vec3::new(
                                location.x as f32,
                                0.0,
                                location.y as f32,
                            ))),
                        );
                        instance.insert_grid(init);

                        // Set the character instance bundle, disabled for now

                        // self.sidebar.code_editor.set_bundle(
                        //     instance.clone(),
                        //     ctx,
                        //     self.sidebar.width,
                        // );

                        let character = Character {
                            id: instance.id,
                            character_id: drop.id.uuid,
                            instance,
                        };

                        // Add the character instance to the region content list

                        let mut name = "Character".to_string();
                        if let Some(character) = self.project.characters.get(&drop.id.uuid) {
                            name.clone_from(&character.name);
                        }

                        if let Some(list) = ui.get_list_layout("Region Content List") {
                            let mut item = TheListItem::new(TheId::named_with_id(
                                "Region Content List Item",
                                character.id,
                            ));
                            item.set_text(name);
                            item.set_state(TheWidgetState::Selected);
                            item.add_value_column(100, TheValue::Text("Character".to_string()));

                            list.deselect_all();
                            item.set_context_menu(Some(TheContextMenu {
                                items: vec![TheContextMenuItem::new(
                                    "Delete Character...".to_string(),
                                    TheId::named("Sidebar Delete Character Instance"),
                                )],
                                ..Default::default()
                            }));
                            list.add_item(item, ctx);
                            list.select_item(character.id, ctx, true);
                        }

                        // Add the character instance to the project

                        if let Some(region) =
                            self.project.get_region_mut(&self.server_ctx.curr_region)
                        {
                            region.characters.insert(character.id, character.clone());
                        }

                        // Add the character instance to the server

                        self.server_ctx.curr_character = Some(character.character_id);
                        self.server_ctx.curr_character_instance = Some(character.id);
                        self.server_ctx.curr_area = None;
                        //self.sidebar.deselect_all("Character List", ui);

                        self.server_ctx.curr_grid_id =
                            self.server.add_character_instance_to_region(
                                self.server_ctx.curr_region,
                                character,
                                None,
                            );

                        // Set the character instance debug info, disabled for now

                        // if let Some(curr_grid_id) = self.server_ctx.curr_grid_id {
                        //     let debug_module = self.server.get_region_debug_module(
                        //         self.server_ctx.curr_region,
                        //         curr_grid_id,
                        //     );

                        //     self.sidebar.code_editor.set_debug_module(debug_module, ui);
                        // }
                    } else if drop.id.name.starts_with("Item") {
                        let mut instance = TheCodeBundle::new();

                        let mut init = TheCodeGrid {
                            name: "init".into(),
                            ..Default::default()
                        };
                        init.insert_atom(
                            (0, 0),
                            TheCodeAtom::Set(
                                "@self.position".to_string(),
                                TheValueAssignment::Assign,
                            ),
                        );
                        init.insert_atom(
                            (1, 0),
                            TheCodeAtom::Assignment(TheValueAssignment::Assign),
                        );
                        init.insert_atom(
                            (2, 0),
                            TheCodeAtom::Value(TheValue::Position(Vec3::new(
                                location.x as f32,
                                0.0,
                                location.y as f32,
                            ))),
                        );
                        instance.insert_grid(init);

                        // Set the character instance bundle, disabled for now

                        // self.sidebar.code_editor.set_bundle(
                        //     instance.clone(),
                        //     ctx,
                        //     self.sidebar.width,
                        // );

                        let item = Item {
                            id: instance.id,
                            item_id: drop.id.uuid,
                            instance,
                        };

                        // Add the item instance to the region content list

                        let mut name = "Item".to_string();
                        if let Some(item) = self.project.items.get(&drop.id.uuid) {
                            name.clone_from(&item.name);
                        }

                        if let Some(list) = ui.get_list_layout("Region Content List") {
                            let mut list_item = TheListItem::new(TheId::named_with_id(
                                "Region Content List Item",
                                item.id,
                            ));
                            list_item.set_text(name);
                            list_item.set_state(TheWidgetState::Selected);
                            list_item.add_value_column(100, TheValue::Text("Item".to_string()));

                            list.deselect_all();
                            list.add_item(list_item, ctx);
                            list.select_item(item.id, ctx, true);
                        }

                        // Add the item instance to the project

                        if let Some(region) =
                            self.project.get_region_mut(&self.server_ctx.curr_region)
                        {
                            region.items.insert(item.id, item.clone());
                        }

                        // Add the character instance to the server

                        self.server_ctx.curr_character = None;
                        self.server_ctx.curr_character_instance = None;
                        self.server_ctx.curr_item = Some(item.item_id);
                        self.server_ctx.curr_item_instance = Some(item.id);
                        self.server_ctx.curr_area = None;

                        self.server_ctx.curr_grid_id = self
                            .server
                            .add_item_instance_to_region(self.server_ctx.curr_region, item);

                        // Set the character instance debug info, disabled for now

                        // if let Some(curr_grid_id) = self.server_ctx.curr_grid_id {
                        //     let debug_module = self.server.get_region_debug_module(
                        //         self.server_ctx.curr_region,
                        //         curr_grid_id,
                        //     );

                        //     self.sidebar.code_editor.set_debug_module(debug_module, ui);
                        // }
                    }
                }*/
                TheEvent::FileRequesterResult(id, paths) => {
                    // Load a palette from a file
                    if id.name == "Palette Import" {
                        for p in paths {
                            let contents = std::fs::read_to_string(p).unwrap_or("".to_string());
                            let prev = self.project.palette.clone();
                            self.project.palette.load_from_txt(contents);
                            *PALETTE.write().unwrap() = self.project.palette.clone();

                            if let Some(palette_picker) = ui.get_palette_picker("Palette Picker") {
                                let index = palette_picker.index();

                                palette_picker.set_palette(self.project.palette.clone());
                                if let Some(widget) = ui.get_widget("Palette Color Picker") {
                                    if let Some(color) = &self.project.palette[index] {
                                        widget.set_value(TheValue::ColorObject(color.clone()));
                                    }
                                }
                                if let Some(widget) = ui.get_widget("Palette Hex Edit") {
                                    if let Some(color) = &self.project.palette[index] {
                                        widget.set_value(TheValue::Text(color.to_hex()));
                                    }
                                }
                            }
                            redraw = true;

                            let undo = PaletteUndoAtom::Edit(prev, self.project.palette.clone());
                            UNDOMANAGER.write().unwrap().add_palette_undo(undo, ctx);
                        }
                    } else
                    // Open
                    if id.name == "Open" {
                        for p in paths {
                            self.project_path = Some(p.clone());
                            self.update_counter = 0;
                            self.sidebar.startup = true;

                            // ctx.ui.set_disabled("Save");
                            // ctx.ui.set_disabled("Save As");
                            ctx.ui.set_disabled("Undo");
                            ctx.ui.set_disabled("Redo");
                            *UNDOMANAGER.write().unwrap() = UndoManager::default();

                            // let contents =
                            //     std::fs::read_to_string(p.clone()).unwrap_or("".to_string());
                            // // if let Ok(contents) = std::fs::read(p) {
                            // let pr: Result<Project, serde_json::Error> =
                            //     serde_json::from_str(&contents);
                            // println!("{:?}", pr.err());
                            if let Ok(contents) = std::fs::read_to_string(p) {
                                if let Ok(project) = serde_json::from_str(&contents) {
                                    self.project = project;
                                    self.project.palette.current_index = 0;

                                    insert_content_into_maps(&mut self.project);

                                    // Rename and remove legacy attributes
                                    for r in &mut self.project.regions {
                                        for s in &mut r.map.sectors {
                                            if let Some(floor) = s.properties.get("floor_source") {
                                                s.properties.set("source", floor.clone());
                                            }

                                            if s.properties.contains("rect_rendering") {
                                                s.properties.set("rect", Value::Bool(true));
                                            }

                                            s.properties.remove("floor_source");
                                            s.properties.remove("rect_rendering");
                                            s.properties.remove("ceiling_source");
                                        }
                                    }

                                    // Map names of characters to instances
                                    let mut hash = FxHashMap::default();
                                    for c in &self.project.characters {
                                        hash.insert(c.0, c.1.name.clone());
                                    }
                                    for r in &mut self.project.regions {
                                        for c in &mut r.characters {
                                            if let Some(n) = hash.get(&c.1.character_id) {
                                                c.1.name = n.clone();
                                            }
                                        }
                                    }

                                    // Map names of items to instances
                                    let mut hash = FxHashMap::default();
                                    for c in &self.project.items {
                                        hash.insert(c.0, c.1.name.clone());
                                    }

                                    // Apply names and sanitize map and its profiles
                                    for r in &mut self.project.regions {
                                        for c in &mut r.items {
                                            if let Some(n) = hash.get(&c.1.item_id) {
                                                c.1.name = n.clone();
                                            }
                                        }
                                        for (_, p) in &mut r.map.profiles {
                                            p.sanitize();
                                        }
                                        r.map.sanitize();
                                    }

                                    // Sanitize screens
                                    for (_, screen) in &mut self.project.screens {
                                        screen.map.sanitize();
                                    }

                                    // Convert old tile refs to new tiles
                                    if self.project.tiles.is_empty() {
                                        let tiles = self.project.extract_tiles();

                                        for (id, t) in tiles.iter() {
                                            let mut texture_array: Vec<Texture> = vec![];
                                            for b in &t.buffer {
                                                let mut texture = Texture::new(
                                                    b.pixels().to_vec(),
                                                    b.dim().width as usize,
                                                    b.dim().height as usize,
                                                );
                                                texture.generate_normals(true);
                                                texture_array.push(texture);
                                            }
                                            let tile = rusterix::Tile {
                                                id: t.id,
                                                role: rusterix::TileRole::from_index(t.role),
                                                textures: texture_array.clone(),
                                                module: None,
                                                blocking: t.blocking,
                                                scale: t.scale,
                                                tags: t.name.clone(),
                                            };
                                            self.project.tiles.insert(*id, tile);
                                        }
                                    }

                                    // Generate all tile normals
                                    for (_, tile) in self.project.tiles.iter_mut() {
                                        for texture in &mut tile.textures {
                                            texture.generate_normals(true);
                                        }
                                    }

                                    // Recompile character visual codes if scripts have Python code
                                    for (_, character) in self.project.characters.iter_mut() {
                                        if character.source.starts_with("class") {
                                            character.source = character.module.build(false);
                                            character.source_debug = character.module.build(true);
                                        }
                                    }

                                    // Recompile entity visual codes if scripts have Python code
                                    for (_, item) in self.project.items.iter_mut() {
                                        if item.source.starts_with("class") {
                                            item.source = item.module.build(false);
                                            item.source_debug = item.module.build(true);
                                        }
                                    }

                                    // Set the project time to the server time slider widget
                                    if let Some(widget) = ui.get_widget("Server Time Slider") {
                                        widget.set_value(TheValue::Time(self.project.time));
                                    }

                                    // Set the server time to the client (and if running to the server)
                                    {
                                        let mut rusterix = RUSTERIX.write().unwrap();
                                        rusterix.client.set_server_time(self.project.time);
                                        rusterix.client.global = self.project.render_graph.clone();
                                        if rusterix.server.state == rusterix::ServerState::Running {
                                            if let Some(map) =
                                                self.project.get_map(&self.server_ctx)
                                            {
                                                rusterix
                                                    .server
                                                    .set_time(&map.id, self.project.time);
                                            }
                                        }
                                    }

                                    self.server_ctx.clear();
                                    if let Some(first) = self.project.regions.first() {
                                        self.server_ctx.curr_region = first.id;
                                    }

                                    self.sidebar.load_from_project(
                                        ui,
                                        ctx,
                                        &mut self.server_ctx,
                                        &mut self.project,
                                    );
                                    self.mapeditor.load_from_project(ui, ctx, &self.project);
                                    update_server_icons = true;
                                    redraw = true;

                                    // Set palette and textures
                                    *PALETTE.write().unwrap() = self.project.palette.clone();

                                    SCENEMANAGER
                                        .write()
                                        .unwrap()
                                        .set_palette(self.project.palette.clone());

                                    ctx.ui.send(TheEvent::SetStatusText(
                                        TheId::empty(),
                                        "Project loaded successfully.".to_string(),
                                    ));
                                }
                            }
                        }
                    } else if id.name == "Save As" {
                        for p in paths {
                            let json = serde_json::to_string(&self.project);
                            if let Ok(json) = json {
                                if std::fs::write(p.clone(), json).is_ok() {
                                    self.project_path = Some(p);
                                    ctx.ui.send(TheEvent::SetStatusText(
                                        TheId::empty(),
                                        "Project saved successfully.".to_string(),
                                    ))
                                } else {
                                    ctx.ui.send(TheEvent::SetStatusText(
                                        TheId::empty(),
                                        "Unable to save project!".to_string(),
                                    ))
                                }
                            }
                        }
                    }
                }
                TheEvent::StateChanged(id, state) => {
                    if id.name == "Help" {
                        self.server_ctx.help_mode = state == TheWidgetState::Clicked;
                    }
                    if id.name == "GameInput" {
                        self.server_ctx.game_input_mode = state == TheWidgetState::Clicked;
                    } else if id.name == "New" {
                        self.project_path = None;
                        self.update_counter = 0;
                        self.sidebar.startup = true;
                        self.project = Project::default();

                        if let Some(bytes) = crate::Embedded::get("starter_project.eldiron") {
                            if let Ok(project_string) = std::str::from_utf8(bytes.data.as_ref()) {
                                if let Ok(project) =
                                    serde_json::from_str(&project_string.to_string())
                                {
                                    self.project = project;
                                }
                            }
                        }

                        // ctx.ui.set_disabled("Save");
                        // ctx.ui.set_disabled("Save As");
                        ctx.ui.set_disabled("Undo");
                        ctx.ui.set_disabled("Redo");
                        *UNDOMANAGER.write().unwrap() = UndoManager::default();

                        insert_content_into_maps(&mut self.project);

                        // Set the project time to the server time slider widget
                        if let Some(widget) = ui.get_widget("Server Time Slider") {
                            widget.set_value(TheValue::Time(self.project.time));
                        }

                        // Set the server time to the client (and if running to the server)
                        {
                            let mut rusterix = RUSTERIX.write().unwrap();
                            rusterix.client.set_server_time(self.project.time);
                            if rusterix.server.state == rusterix::ServerState::Running {
                                if let Some(map) = self.project.get_map(&self.server_ctx) {
                                    rusterix.server.set_time(&map.id, self.project.time);
                                }
                            }
                        }

                        self.server_ctx.clear();
                        self.sidebar.load_from_project(
                            ui,
                            ctx,
                            &mut self.server_ctx,
                            &mut self.project,
                        );
                        self.mapeditor.load_from_project(ui, ctx, &self.project);
                        update_server_icons = true;
                        redraw = true;

                        // Set palette and textures
                        *PALETTE.write().unwrap() = self.project.palette.clone();

                        ctx.ui.send(TheEvent::SetStatusText(
                            TheId::empty(),
                            "New project successfully initialized.".to_string(),
                        ));
                    } else if id.name == "Logo" {
                        _ = open::that("https://eldiron.com");
                        ctx.ui
                            .set_widget_state("Logo".to_string(), TheWidgetState::None);
                        ctx.ui.clear_hover();
                        redraw = true;
                    } else if id.name == "Patreon" {
                        _ = open::that("https://www.patreon.com/eldiron");
                        ctx.ui
                            .set_widget_state("Patreon".to_string(), TheWidgetState::None);
                        ctx.ui.clear_hover();
                        redraw = true;
                    } else if id.name == "Update" {
                        #[cfg(all(not(target_arch = "wasm32"), feature = "self-update"))]
                        {
                            let updater = self.self_updater.lock().unwrap();

                            if updater.has_newer_release() {
                                self.self_update_tx
                                    .send(SelfUpdateEvent::UpdateConfirm(
                                        updater.latest_release().cloned().unwrap(),
                                    ))
                                    .unwrap();
                            } else {
                                if let Some(statusbar) = ui.get_widget("Statusbar") {
                                    statusbar
                                        .as_statusbar()
                                        .unwrap()
                                        .set_text(fl!("info_update_check"));
                                }

                                let updater = Arc::clone(&self.self_updater);
                                let tx = self.self_update_tx.clone();

                                thread::spawn(move || {
                                    let mut updater = updater.lock().unwrap();

                                    match updater.fetch_release_list() {
                                        Ok(_) => {
                                            if updater.has_newer_release() {
                                                tx.send(SelfUpdateEvent::UpdateConfirm(
                                                    updater.latest_release().cloned().unwrap(),
                                                ))
                                                .unwrap();
                                            } else {
                                                tx.send(SelfUpdateEvent::AlreadyUpToDate).unwrap();
                                            }
                                        }
                                        Err(err) => {
                                            tx.send(SelfUpdateEvent::UpdateError(err.to_string()))
                                                .unwrap();
                                        }
                                    }
                                });
                            }

                            ctx.ui
                                .set_widget_state("Update".to_string(), TheWidgetState::None);
                            ctx.ui.clear_hover();
                            redraw = true;
                        }
                    } else if id.name == "Open" {
                        ctx.ui.open_file_requester(
                            TheId::named_with_id(id.name.as_str(), Uuid::new_v4()),
                            "Open".into(),
                            TheFileExtension::new("Eldiron".into(), vec!["eldiron".to_string()]),
                        );
                        ctx.ui
                            .set_widget_state("Open".to_string(), TheWidgetState::None);
                        ctx.ui.clear_hover();
                        redraw = true;
                    } else if id.name == "Save" {
                        if let Some(path) = &self.project_path {
                            let mut success = false;
                            // if let Ok(output) = postcard::to_allocvec(&self.project) {
                            if let Ok(output) = serde_json::to_string(&self.project) {
                                if std::fs::write(path, output).is_ok() {
                                    ctx.ui.send(TheEvent::SetStatusText(
                                        TheId::empty(),
                                        "Project saved successfully.".to_string(),
                                    ));
                                    success = true;
                                }
                            }

                            if !success {
                                ctx.ui.send(TheEvent::SetStatusText(
                                    TheId::empty(),
                                    "Unable to save project!".to_string(),
                                ))
                            }
                        } else {
                            ctx.ui.send(TheEvent::StateChanged(
                                TheId::named("Save As"),
                                TheWidgetState::Clicked,
                            ));
                            ctx.ui
                                .set_widget_state("Save".to_string(), TheWidgetState::None);
                        }
                    } else if id.name == "Save As" {
                        ctx.ui.save_file_requester(
                            TheId::named_with_id(id.name.as_str(), Uuid::new_v4()),
                            "Save".into(),
                            TheFileExtension::new("Eldiron".into(), vec!["eldiron".to_string()]),
                        );
                        ctx.ui
                            .set_widget_state("Save As".to_string(), TheWidgetState::None);
                        ctx.ui.clear_hover();
                        redraw = true;
                    } else if id.name == "Project Validate" {
                        let (warnings, errors) = self.validate_project_for_design();
                        if errors.is_empty() {
                            ctx.ui.send(TheEvent::SetStatusText(
                                TheId::empty(),
                                format!(
                                    "Project validation passed ({} warning(s)).",
                                    warnings.len()
                                ),
                            ));
                        } else {
                            ctx.ui.send(TheEvent::SetStatusText(
                                TheId::empty(),
                                format!(
                                    "Project validation failed ({} error(s), {} warning(s)).",
                                    errors.len(),
                                    warnings.len()
                                ),
                            ));
                        }
                    } else if id.name == "Project Refresh View" {
                        self.refresh_designer_views(ui, ctx);
                        ctx.ui.send(TheEvent::SetStatusText(
                            TheId::empty(),
                            "Designer view refreshed from project data.".to_string(),
                        ));
                        redraw = true;
                    } else if id.name == "Build Compile Runtime Compact"
                        || id.name == "Build Compile Runtime Pretty"
                    {
                        let pretty = id.name == "Build Compile Runtime Pretty";
                        let response = self.compile_from_editor_input(CompileFromEditorInputRequest {
                            pretty,
                            include_compiled_project_json: true,
                            ..Default::default()
                        });
                        if response.success {
                            if let Some(compiled) = response.compiled_project_json {
                                ctx.ui.clipboard = Some(TheValue::Text(compiled));
                                ctx.ui.clipboard_app_type = Some("application/json".to_string());
                            }
                            ctx.ui.send(TheEvent::SetStatusText(
                                TheId::empty(),
                                format!(
                                    "Compiled runtime JSON ({} warning(s)); copied to clipboard.",
                                    response.warnings.len()
                                ),
                            ));
                        } else {
                            let detail = response
                                .errors
                                .first()
                                .cloned()
                                .unwrap_or_else(|| response.message.clone());
                            ctx.ui.send(TheEvent::SetStatusText(
                                TheId::empty(),
                                format!("Runtime compile failed: {detail}"),
                            ));
                        }
                    } else if id.name == "Build Compile Runtime To File" {
                        match self.compile_runtime_to_project_folder(true) {
                            Ok((path, response)) => {
                                ctx.ui.send(TheEvent::SetStatusText(
                                    TheId::empty(),
                                    format!(
                                        "Runtime output exported to {} ({} warning(s)).",
                                        path.to_string_lossy(),
                                        response.warnings.len()
                                    ),
                                ));
                            }
                            Err(error) => {
                                ctx.ui.send(TheEvent::SetStatusText(
                                    TheId::empty(),
                                    format!("Export failed: {error}"),
                                ));
                            }
                        }
                    } else if id.name == "Build Validate And Compile" {
                        let (warnings, errors) = self.validate_project_for_design();
                        if !errors.is_empty() {
                            ctx.ui.send(TheEvent::SetStatusText(
                                TheId::empty(),
                                format!(
                                    "Build blocked by validation errors ({} error(s)).",
                                    errors.len()
                                ),
                            ));
                        } else {
                            match self.compile_runtime_to_project_folder(false) {
                                Ok((path, compile_response)) => {
                                    ctx.ui.send(TheEvent::SetStatusText(
                                        TheId::empty(),
                                        format!(
                                            "Build complete: {} ({} warning(s), {} compile warning(s)).",
                                            path.to_string_lossy(),
                                            warnings.len(),
                                            compile_response.warnings.len()
                                        ),
                                    ));
                                }
                                Err(error) => {
                                    ctx.ui.send(TheEvent::SetStatusText(
                                        TheId::empty(),
                                        format!("Build failed: {error}"),
                                    ));
                                }
                            }
                        }
                    } else if id.name == "World Add Region" || id.name == "World Add Dungeon" {
                        let base = if id.name == "World Add Dungeon" {
                            "Dungeon Region"
                        } else {
                            "Region"
                        };
                        let mut region = Region::new();
                        region.name =
                            self.unique_name(self.project.regions.iter().map(|r| r.name.clone()), base);
                        self.server_ctx.curr_region = region.id;
                        self.project.regions.push(region);
                        self.refresh_designer_views(ui, ctx);
                        ctx.ui.send(TheEvent::SetStatusText(
                            TheId::empty(),
                            format!("{base} created."),
                        ));
                        redraw = true;
                    } else if id.name == "World Add Screen" {
                        let mut screen = Screen::new();
                        screen.name = self.unique_name(
                            self.project.screens.values().map(|s| s.name.clone()),
                            "Screen",
                        );
                        self.project.add_screen(screen);
                        self.refresh_designer_views(ui, ctx);
                        ctx.ui.send(TheEvent::SetStatusText(
                            TheId::empty(),
                            "Screen created.".to_string(),
                        ));
                        redraw = true;
                    } else if id.name == "World Seed Biomes" {
                        let biome_names = ["Grasslands", "Forest", "Desert", "Tundra", "Volcanic"];
                        for biome in biome_names {
                            let mut region = Region::new();
                            region.name = self.unique_name(
                                self.project.regions.iter().map(|r| r.name.clone()),
                                biome,
                            );
                            self.project.regions.push(region);
                        }
                        self.refresh_designer_views(ui, ctx);
                        ctx.ui.send(TheEvent::SetStatusText(
                            TheId::empty(),
                            "Biome starter pack generated.".to_string(),
                        ));
                        redraw = true;
                    } else if id.name == "Content Add Character" {
                        let mut character = Character::new();
                        character.name = self.unique_name(
                            self.project.characters.values().map(|c| c.name.clone()),
                            "Character",
                        );
                        self.project.add_character(character);
                        self.refresh_designer_views(ui, ctx);
                        ctx.ui.send(TheEvent::SetStatusText(
                            TheId::empty(),
                            "Character template created.".to_string(),
                        ));
                        redraw = true;
                    } else if id.name == "Content Add Item" {
                        let mut item = Item::new();
                        item.name = self.unique_name(
                            self.project.items.values().map(|i| i.name.clone()),
                            "Item",
                        );
                        self.project.add_item(item);
                        self.refresh_designer_views(ui, ctx);
                        ctx.ui.send(TheEvent::SetStatusText(
                            TheId::empty(),
                            "Item template created.".to_string(),
                        ));
                        redraw = true;
                    } else if id.name == "Content Add Starter Pack" {
                        let mut character = Character::new();
                        character.name = self.unique_name(
                            self.project.characters.values().map(|c| c.name.clone()),
                            "Hero",
                        );
                        self.project.add_character(character);

                        let mut item = Item::new();
                        item.name = self.unique_name(
                            self.project.items.values().map(|i| i.name.clone()),
                            "Potion",
                        );
                        self.project.add_item(item);

                        self.refresh_designer_views(ui, ctx);
                        ctx.ui.send(TheEvent::SetStatusText(
                            TheId::empty(),
                            "Starter content pack created.".to_string(),
                        ));
                        redraw = true;
                    } else if id.name == "Content Add Widget HUD" {
                        let screen = self.create_widget_window_screen("HUD Window", "HUD");
                        self.project.add_screen(screen);
                        self.refresh_designer_views(ui, ctx);
                        ctx.ui.send(TheEvent::SetStatusText(
                            TheId::empty(),
                            "HUD widget window created.".to_string(),
                        ));
                        redraw = true;
                    } else if id.name == "Content Add Widget Inventory" {
                        let screen =
                            self.create_widget_window_screen("Inventory Window", "Inventory");
                        self.project.add_screen(screen);
                        self.refresh_designer_views(ui, ctx);
                        ctx.ui.send(TheEvent::SetStatusText(
                            TheId::empty(),
                            "Inventory widget window created.".to_string(),
                        ));
                        redraw = true;
                    } else if id.name == "Content Add Widget Dialogue" {
                        let screen = self.create_widget_window_screen("Dialogue Window", "Dialogue");
                        self.project.add_screen(screen);
                        self.refresh_designer_views(ui, ctx);
                        ctx.ui.send(TheEvent::SetStatusText(
                            TheId::empty(),
                            "Dialogue widget window created.".to_string(),
                        ));
                        redraw = true;
                    } else if id.name == "Content Add Widget Settings" {
                        let screen = self.create_widget_window_screen("Settings Window", "Settings");
                        self.project.add_screen(screen);
                        self.refresh_designer_views(ui, ctx);
                        ctx.ui.send(TheEvent::SetStatusText(
                            TheId::empty(),
                            "Settings widget window created.".to_string(),
                        ));
                        redraw = true;
                    } else if id.name == "Content Add Widget Starter Pack" {
                        let hud = self.create_widget_window_screen("HUD Window", "HUD");
                        self.project.add_screen(hud);
                        let inventory =
                            self.create_widget_window_screen("Inventory Window", "Inventory");
                        self.project.add_screen(inventory);
                        let dialogue =
                            self.create_widget_window_screen("Dialogue Window", "Dialogue");
                        self.project.add_screen(dialogue);
                        let settings =
                            self.create_widget_window_screen("Settings Window", "Settings");
                        self.project.add_screen(settings);

                        self.refresh_designer_views(ui, ctx);
                        ctx.ui.send(TheEvent::SetStatusText(
                            TheId::empty(),
                            "Widget window starter pack created.".to_string(),
                        ));
                        redraw = true;
                    }
                    // Server
                    else if id.name == "Play" {
                        let state = RUSTERIX.read().unwrap().server.state;
                        if state == rusterix::ServerState::Paused {
                            RUSTERIX.write().unwrap().server.continue_instances();
                            update_server_icons = true;
                        } else {
                            if state == rusterix::ServerState::Off {
                                start_server(
                                    &mut RUSTERIX.write().unwrap(),
                                    &mut self.project,
                                    true,
                                );
                                let commands =
                                    setup_client(&mut RUSTERIX.write().unwrap(), &mut self.project);
                                RUSTERIX
                                    .write()
                                    .unwrap()
                                    .server
                                    .process_client_commands(commands);
                                ctx.ui.send(TheEvent::SetStatusText(
                                    TheId::empty(),
                                    "Server has been started.".to_string(),
                                ));
                                // ui.set_widget_value("LogEdit", ctx, TheValue::Text(String::new()));
                                // ctx.ui.send(TheEvent::StateChanged(
                                //     TheId::named("Debug Log"),
                                //     TheWidgetState::Clicked,
                                // ));
                                RUSTERIX.write().unwrap().player_camera = PlayerCamera::D2;
                            }
                            /*
                            self.server.start();
                            self.client.reset();
                            self.client.set_project(self.project.clone());
                            self.server_ctx.clear_interactions();
                            ctx.ui.send(TheEvent::SetStatusText(
                                TheId::empty(),
                                "Server has been started.".to_string(),
                            ));
                            self.sidebar.clear_debug_messages(ui, ctx);
                            */
                            update_server_icons = true;
                        }
                    } else if id.name == "Pause" {
                        let state = RUSTERIX.read().unwrap().server.state;
                        if state == rusterix::ServerState::Running {
                            RUSTERIX.write().unwrap().server.pause();
                            update_server_icons = true;
                        }
                        /*
                        if self.server.state == ServerState::Running {
                            self.server.state = ServerState::Paused;
                            ctx.ui.send(TheEvent::SetStatusText(
                                TheId::empty(),
                                "Server has been paused.".to_string(),
                            ));
                            update_server_icons = true;
                        } else if self.server.state == ServerState::Paused {
                            self.client.tick(
                                *ACTIVEEDITOR.lock().unwrap() == ActiveEditor::GameEditor,
                            );
                            let debug = self.server.tick();
                            if !debug.is_empty() {
                                self.sidebar.add_debug_messages(debug, ui, ctx);
                            }
                            let interactions = self.server.get_interactions();
                            self.server_ctx.add_interactions(interactions);
                        }*/
                    } else if id.name == "Stop" {
                        RUSTERIX.write().unwrap().server.stop();
                        RUSTERIX.write().unwrap().player_camera = PlayerCamera::D2;

                        ui.set_widget_value("InfoView", ctx, TheValue::Text("".into()));
                        /*
                        _ = self.server.set_project(self.project.clone());
                        self.server.stop();*/
                        insert_content_into_maps(&mut self.project);
                        update_server_icons = true;

                        ctx.ui.send(TheEvent::Custom(
                            TheId::named("Render SceneManager Map"),
                            TheValue::Empty,
                        ));
                    } else if id.name == "Undo" || id.name == "Redo" {
                        let mut refresh_action_ui = false;
                        if ui.focus_widget_supports_undo_redo(ctx) {
                            if id.name == "Undo" {
                                ui.undo(ctx);
                            } else {
                                ui.redo(ctx);
                            }
                        } else if DOCKMANAGER.read().unwrap().current_dock_supports_undo() {
                            if id.name == "Undo" {
                                DOCKMANAGER.write().unwrap().undo(
                                    ui,
                                    ctx,
                                    &mut self.project,
                                    &mut self.server_ctx,
                                );
                            } else {
                                DOCKMANAGER.write().unwrap().redo(
                                    ui,
                                    ctx,
                                    &mut self.project,
                                    &mut self.server_ctx,
                                );
                            }
                            refresh_action_ui = true;
                        } else {
                            let mut manager = UNDOMANAGER.write().unwrap();

                            if id.name == "Undo" {
                                manager.undo(&mut self.server_ctx, &mut self.project, ui, ctx);
                            } else {
                                manager.redo(&mut self.server_ctx, &mut self.project, ui, ctx);
                            }
                            refresh_action_ui = true;
                        }

                        // Keep action list and TOML params in sync only when project/dock state changed.
                        if refresh_action_ui {
                            ctx.ui.send(TheEvent::Custom(
                                TheId::named("Update Action List"),
                                TheValue::Empty,
                            ));
                            ctx.ui.send(TheEvent::Custom(
                                TheId::named("Update Action Parameters"),
                                TheValue::Empty,
                            ));
                        }
                    } else if id.name == "Cut" {
                        if ui.focus_widget_supports_clipboard(ctx) {
                            // Widget specific
                            ui.cut(ctx);
                        } else {
                            // Global
                            ctx.ui.send(TheEvent::Cut);
                        }
                    } else if id.name == "Copy" {
                        if ui.focus_widget_supports_clipboard(ctx) {
                            // Widget specific
                            ui.copy(ctx);
                        } else {
                            // Global
                            ctx.ui.send(TheEvent::Copy);
                        }
                    } else if id.name == "Paste" {
                        if ui.focus_widget_supports_clipboard(ctx) {
                            // Widget specific
                            ui.paste(ctx);
                        } else {
                            // Global
                            if let Some(value) = &ctx.ui.clipboard {
                                ctx.ui.send(TheEvent::Paste(
                                    value.clone(),
                                    ctx.ui.clipboard_app_type.clone(),
                                ));
                            } else {
                                ctx.ui.send(TheEvent::Paste(
                                    TheValue::Empty,
                                    ctx.ui.clipboard_app_type.clone(),
                                ));
                            }
                        }
                    }
                }
                TheEvent::ValueChanged(id, value) => {
                    if id.name == "Server Time Slider" {
                        if let TheValue::Time(time) = value {
                            self.project.time = time;
                            let mut rusterix = RUSTERIX.write().unwrap();
                            rusterix.client.set_server_time(time);

                            if rusterix.server.state == rusterix::ServerState::Running {
                                if let Some(map) = self.project.get_map(&self.server_ctx) {
                                    rusterix.server.set_time(&map.id, time);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        #[cfg(all(not(target_arch = "wasm32"), feature = "self-update"))]
        while let Ok(event) = self.self_update_rx.try_recv() {
            match event {
                SelfUpdateEvent::AlreadyUpToDate => {
                    let text = str!("NexusStudio is already up-to-date.");
                    let uuid = Uuid::new_v4();

                    let width = 300;
                    let height = 100;

                    let mut canvas = TheCanvas::new();
                    canvas.limiter_mut().set_max_size(Vec2::new(width, height));

                    let mut hlayout: TheHLayout = TheHLayout::new(TheId::empty());
                    hlayout.limiter_mut().set_max_width(width);

                    let mut text_widget = TheText::new(TheId::named_with_id("Dialog Value", uuid));
                    text_widget.set_text(text.to_string());
                    text_widget.limiter_mut().set_max_width(200);
                    hlayout.add_widget(Box::new(text_widget));

                    canvas.set_layout(hlayout);

                    ui.show_dialog(
                        "NexusStudio Up-to-Date",
                        canvas,
                        vec![TheDialogButtonRole::Accept],
                        ctx,
                    );
                }
                SelfUpdateEvent::UpdateCompleted(release) => {
                    if let Some(statusbar) = ui.get_widget("Statusbar") {
                        statusbar.as_statusbar().unwrap().set_text(format!(
                            "Updated to version {}. Please restart the application to enjoy the new features.",
                            release.version
                        ));
                    }
                }
                SelfUpdateEvent::UpdateConfirm(release) => {
                    let text = &format!("Update to version {}?", release.version);
                    let uuid = Uuid::new_v4();

                    let width = 300;
                    let height = 100;

                    let mut canvas = TheCanvas::new();
                    canvas.limiter_mut().set_max_size(Vec2::new(width, height));

                    let mut hlayout: TheHLayout = TheHLayout::new(TheId::empty());
                    hlayout.limiter_mut().set_max_width(width);

                    let mut text_widget = TheText::new(TheId::named_with_id("Dialog Value", uuid));
                    text_widget.set_text(text.to_string());
                    text_widget.limiter_mut().set_max_width(200);
                    hlayout.add_widget(Box::new(text_widget));

                    canvas.set_layout(hlayout);

                    ui.show_dialog(
                        "Update NexusStudio",
                        canvas,
                        vec![TheDialogButtonRole::Accept, TheDialogButtonRole::Reject],
                        ctx,
                    );
                }
                SelfUpdateEvent::UpdateError(err) => {
                    if let Some(statusbar) = ui.get_widget("Statusbar") {
                        statusbar
                            .as_statusbar()
                            .unwrap()
                            .set_text(format!("Failed to update NexusStudio: {err}"));
                    }
                }
                SelfUpdateEvent::UpdateStart(release) => {
                    if let Some(statusbar) = ui.get_widget("Statusbar") {
                        statusbar
                            .as_statusbar()
                            .unwrap()
                            .set_text(format!("Updating to version {}...", release.version));
                    }
                }
            }
        }

        if update_server_icons {
            self.update_server_state_icons(ui);
            redraw = true;
        }
        self.update_counter += 1;
        if self.update_counter > 2 {
            self.sidebar.startup = false;
        }
        redraw
    }

    /// Returns true if there are changes
    fn has_changes(&self) -> bool {
        UNDOMANAGER.read().unwrap().has_undo() || DOCKMANAGER.read().unwrap().has_dock_changes()
    }
}

pub trait EldironEditor {
    fn update_server_state_icons(&mut self, ui: &mut TheUI);
}

impl EldironEditor for Editor {
    fn update_server_state_icons(&mut self, ui: &mut TheUI) {
        let rusterix = RUSTERIX.read().unwrap();
        if rusterix.server.state == rusterix::ServerState::Running {
            if let Some(button) = ui.get_widget("Play") {
                if let Some(button) = button.as_menubar_button() {
                    button.set_icon_name("play-fill".to_string());
                }
            }
            if let Some(button) = ui.get_widget("Pause") {
                if let Some(button) = button.as_menubar_button() {
                    button.set_icon_name("play-pause".to_string());
                }
            }
            if let Some(button) = ui.get_widget("Stop") {
                if let Some(button) = button.as_menubar_button() {
                    button.set_icon_name("stop".to_string());
                }
            }
        } else if rusterix.server.state == rusterix::ServerState::Paused {
            if let Some(button) = ui.get_widget("Play") {
                if let Some(button) = button.as_menubar_button() {
                    button.set_icon_name("play".to_string());
                }
            }
            if let Some(button) = ui.get_widget("Pause") {
                if let Some(button) = button.as_menubar_button() {
                    button.set_icon_name("play-pause-fill".to_string());
                }
            }
            if let Some(button) = ui.get_widget("Stop") {
                if let Some(button) = button.as_menubar_button() {
                    button.set_icon_name("stop".to_string());
                }
            }
        } else if rusterix.server.state == rusterix::ServerState::Off {
            if let Some(button) = ui.get_widget("Play") {
                if let Some(button) = button.as_menubar_button() {
                    button.set_icon_name("play".to_string());
                }
            }
            if let Some(button) = ui.get_widget("Pause") {
                if let Some(button) = button.as_menubar_button() {
                    button.set_icon_name("play-pause".to_string());
                }
            }
            if let Some(button) = ui.get_widget("Stop") {
                if let Some(button) = button.as_menubar_button() {
                    button.set_icon_name("stop-fill".to_string());
                }
            }
        }
    }
}
