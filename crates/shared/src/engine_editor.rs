use crate::rpg2d::{AudioSystem, GameSettings, RuntimeOptions, WorldAtlas, WorldCreationError};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineFeatureFlags {
    pub world_generation: bool,
    pub battle_system: bool,
    pub quest_system: bool,
    pub dialog_system: bool,
    pub scripting: bool,
    pub multiplayer: bool,
    pub tool_menus: bool,
    pub audio_editor: bool,
    pub video_options: bool,
    pub keybinding_editor: bool,
}

impl Default for EngineFeatureFlags {
    fn default() -> Self {
        Self {
            world_generation: true,
            battle_system: true,
            quest_system: true,
            dialog_system: true,
            scripting: true,
            multiplayer: false,
            tool_menus: true,
            audio_editor: true,
            video_options: true,
            keybinding_editor: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorPanelKind {
    WorldTree,
    Inspector,
    TilePalette,
    ScriptEditor,
    AudioEditor,
    Console,
    Playtest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorPanel {
    pub id: String,
    pub title: String,
    pub kind: EditorPanelKind,
    pub visible: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorLayoutPreset {
    Minimal,
    Classic,
    Creator,
    Debug,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorWorkspace {
    pub preset: EditorLayoutPreset,
    pub top_menu_title: String,
    pub panels: Vec<EditorPanel>,
}

impl Default for EditorWorkspace {
    fn default() -> Self {
        Self {
            preset: EditorLayoutPreset::Creator,
            top_menu_title: "NexusStudio".to_string(),
            panels: vec![
                EditorPanel {
                    id: "world_tree".to_string(),
                    title: "World Tree".to_string(),
                    kind: EditorPanelKind::WorldTree,
                    visible: true,
                },
                EditorPanel {
                    id: "inspector".to_string(),
                    title: "Inspector".to_string(),
                    kind: EditorPanelKind::Inspector,
                    visible: true,
                },
                EditorPanel {
                    id: "script_editor".to_string(),
                    title: "Script Editor".to_string(),
                    kind: EditorPanelKind::ScriptEditor,
                    visible: true,
                },
                EditorPanel {
                    id: "audio_editor".to_string(),
                    title: "8-bit Audio FX".to_string(),
                    kind: EditorPanelKind::AudioEditor,
                    visible: true,
                },
            ],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorToolCategory {
    Build,
    Paint,
    Logic,
    Audio,
    Debug,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub id: String,
    pub title: String,
    pub category: EditorToolCategory,
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ToolRegistry {
    pub tools: BTreeMap<String, ToolDefinition>,
}

impl ToolRegistry {
    pub fn add(&mut self, tool: ToolDefinition) {
        self.tools.insert(tool.id.clone(), tool);
    }

    pub fn set_enabled(&mut self, id: &str, enabled: bool) -> bool {
        if let Some(tool) = self.tools.get_mut(id) {
            tool.enabled = enabled;
            true
        } else {
            false
        }
    }

    pub fn all_enabled(&self) -> Vec<&ToolDefinition> {
        self.tools.values().filter(|t| t.enabled).collect()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EngineProjectMetadata {
    pub id: String,
    pub name: String,
    pub author: String,
    pub version: String,
    pub tags: Vec<String>,
}

impl Default for EngineProjectMetadata {
    fn default() -> Self {
        Self {
            id: "project_default".to_string(),
            name: "New RPG Project".to_string(),
            author: "Unknown".to_string(),
            version: "0.1.0".to_string(),
            tags: vec!["rpg".to_string(), "2d".to_string()],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GameEngineProject {
    pub metadata: EngineProjectMetadata,
    pub features: EngineFeatureFlags,
    pub settings: GameSettings,
    pub runtime_options: RuntimeOptions,
    pub audio: AudioSystem,
    pub atlas: WorldAtlas,
    pub workspace: EditorWorkspace,
    pub tools: ToolRegistry,
    pub script_modules: HashMap<String, String>,
}

impl Default for GameEngineProject {
    fn default() -> Self {
        let mut tools = ToolRegistry::default();
        tools.add(ToolDefinition {
            id: "terrain_brush".to_string(),
            title: "Terrain Brush".to_string(),
            category: EditorToolCategory::Paint,
            enabled: true,
        });
        tools.add(ToolDefinition {
            id: "script_node".to_string(),
            title: "Script Node".to_string(),
            category: EditorToolCategory::Logic,
            enabled: true,
        });
        tools.add(ToolDefinition {
            id: "sfx_designer".to_string(),
            title: "8-bit SFX Designer".to_string(),
            category: EditorToolCategory::Audio,
            enabled: true,
        });
        Self {
            metadata: EngineProjectMetadata::default(),
            features: EngineFeatureFlags::default(),
            settings: GameSettings::default(),
            runtime_options: RuntimeOptions::default(),
            audio: AudioSystem::default(),
            atlas: WorldAtlas::default(),
            workspace: EditorWorkspace::default(),
            tools,
            script_modules: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineCommand {
    RenameProject(String),
    SetAuthor(String),
    SetVersion(String),
    ToggleFeature { name: String, enabled: bool },
    SetTopMenuTitle(String),
    ShowPanel(String),
    HidePanel(String),
    AddScriptModule { id: String, source: String },
    RemoveScriptModule { id: String },
    EnableTool(String),
    DisableTool(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorEngineEvent {
    ProjectRenamed(String),
    AuthorUpdated(String),
    VersionUpdated(String),
    FeatureUpdated { name: String, enabled: bool },
    WorkspaceUpdated,
    ScriptModuleUpdated(String),
    ToolUpdated(String),
    ValidationWarning(String),
    ValidationError(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EngineCommandResult {
    pub applied: bool,
    pub events: Vec<EditorEngineEvent>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct EngineRuntime {
    pub project: GameEngineProject,
    pub command_history: Vec<EngineCommand>,
}

impl EngineRuntime {
    pub fn new(project: GameEngineProject) -> Self {
        Self {
            project,
            command_history: vec![],
        }
    }

    pub fn apply(&mut self, command: EngineCommand) -> EngineCommandResult {
        let mut result = EngineCommandResult::default();
        match &command {
            EngineCommand::RenameProject(name) => {
                self.project.metadata.name = name.clone();
                result.applied = true;
                result
                    .events
                    .push(EditorEngineEvent::ProjectRenamed(name.clone()));
            }
            EngineCommand::SetAuthor(author) => {
                self.project.metadata.author = author.clone();
                result.applied = true;
                result
                    .events
                    .push(EditorEngineEvent::AuthorUpdated(author.clone()));
            }
            EngineCommand::SetVersion(version) => {
                self.project.metadata.version = version.clone();
                result.applied = true;
                result
                    .events
                    .push(EditorEngineEvent::VersionUpdated(version.clone()));
            }
            EngineCommand::SetTopMenuTitle(title) => {
                self.project.workspace.top_menu_title = title.clone();
                result.applied = true;
                result.events.push(EditorEngineEvent::WorkspaceUpdated);
            }
            EngineCommand::ShowPanel(id) => {
                if let Some(panel) = self.project.workspace.panels.iter_mut().find(|p| &p.id == id) {
                    panel.visible = true;
                    result.applied = true;
                    result.events.push(EditorEngineEvent::WorkspaceUpdated);
                }
            }
            EngineCommand::HidePanel(id) => {
                if let Some(panel) = self.project.workspace.panels.iter_mut().find(|p| &p.id == id) {
                    panel.visible = false;
                    result.applied = true;
                    result.events.push(EditorEngineEvent::WorkspaceUpdated);
                }
            }
            EngineCommand::AddScriptModule { id, source } => {
                self.project
                    .script_modules
                    .insert(id.clone(), source.clone());
                result.applied = true;
                result
                    .events
                    .push(EditorEngineEvent::ScriptModuleUpdated(id.clone()));
            }
            EngineCommand::RemoveScriptModule { id } => {
                if self.project.script_modules.remove(id).is_some() {
                    result.applied = true;
                    result
                        .events
                        .push(EditorEngineEvent::ScriptModuleUpdated(id.clone()));
                }
            }
            EngineCommand::EnableTool(id) => {
                if self.project.tools.set_enabled(id, true) {
                    result.applied = true;
                    result.events.push(EditorEngineEvent::ToolUpdated(id.clone()));
                }
            }
            EngineCommand::DisableTool(id) => {
                if self.project.tools.set_enabled(id, false) {
                    result.applied = true;
                    result.events.push(EditorEngineEvent::ToolUpdated(id.clone()));
                }
            }
            EngineCommand::ToggleFeature { name, enabled } => {
                let applied = set_feature_flag(&mut self.project.features, name, *enabled);
                result.applied = applied;
                if applied {
                    result.events.push(EditorEngineEvent::FeatureUpdated {
                        name: name.clone(),
                        enabled: *enabled,
                    });
                }
            }
        }

        if result.applied {
            self.command_history.push(command);
        }
        result
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }

    pub fn from_json(text: &str) -> Result<Self, String> {
        serde_json::from_str(text).map_err(|e| e.to_string())
    }

    pub fn validate(&self) -> EngineCommandResult {
        let mut result = EngineCommandResult::default();
        let project = &self.project;

        if project.metadata.name.trim().is_empty() {
            result
                .events
                .push(EditorEngineEvent::ValidationError(
                    "Project name is empty.".to_string(),
                ));
        }
        if project.metadata.author.trim().is_empty() {
            result.events.push(EditorEngineEvent::ValidationWarning(
                "Project author is empty.".to_string(),
            ));
        }
        if project.workspace.panels.is_empty() {
            result.events.push(EditorEngineEvent::ValidationWarning(
                "No editor panels configured.".to_string(),
            ));
        }
        if !project.features.audio_editor && project.tools.tools.contains_key("sfx_designer") {
            result.events.push(EditorEngineEvent::ValidationWarning(
                "SFX tool is enabled while audio editor feature is disabled.".to_string(),
            ));
        }

        result.applied = !result
            .events
            .iter()
            .any(|e| matches!(e, EditorEngineEvent::ValidationError(_)));
        result
    }

    pub fn create_zone_stub(
        &mut self,
        continent_id: &str,
        country_id: &str,
        zone: crate::rpg2d::Zone,
    ) -> Result<(), WorldCreationError> {
        self.project.atlas.add_zone(zone, continent_id, country_id)
    }
}

fn set_feature_flag(flags: &mut EngineFeatureFlags, name: &str, enabled: bool) -> bool {
    match name {
        "world_generation" => flags.world_generation = enabled,
        "battle_system" => flags.battle_system = enabled,
        "quest_system" => flags.quest_system = enabled,
        "dialog_system" => flags.dialog_system = enabled,
        "scripting" => flags.scripting = enabled,
        "multiplayer" => flags.multiplayer = enabled,
        "tool_menus" => flags.tool_menus = enabled,
        "audio_editor" => flags.audio_editor = enabled,
        "video_options" => flags.video_options = enabled,
        "keybinding_editor" => flags.keybinding_editor = enabled,
        _ => return false,
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_apply_project_commands() {
        let mut runtime = EngineRuntime::new(GameEngineProject::default());
        let r1 = runtime.apply(EngineCommand::RenameProject("Dragon Core".to_string()));
        assert!(r1.applied);
        assert_eq!(runtime.project.metadata.name, "Dragon Core");

        let r2 = runtime.apply(EngineCommand::SetTopMenuTitle("Vertex Engine Editor".to_string()));
        assert!(r2.applied);
        assert_eq!(runtime.project.workspace.top_menu_title, "Vertex Engine Editor");
    }

    #[test]
    fn runtime_feature_toggle_and_tool_enable_disable() {
        let mut runtime = EngineRuntime::new(GameEngineProject::default());
        let r = runtime.apply(EngineCommand::ToggleFeature {
            name: "multiplayer".to_string(),
            enabled: true,
        });
        assert!(r.applied);
        assert!(runtime.project.features.multiplayer);

        let d = runtime.apply(EngineCommand::DisableTool("sfx_designer".to_string()));
        assert!(d.applied);
        assert!(
            !runtime
                .project
                .tools
                .tools
                .get("sfx_designer")
                .unwrap()
                .enabled
        );
    }

    #[test]
    fn runtime_roundtrip_json() {
        let runtime = EngineRuntime::new(GameEngineProject::default());
        let json = runtime.to_json().unwrap();
        let decoded = EngineRuntime::from_json(&json).unwrap();
        assert_eq!(decoded.project.metadata.name, "New RPG Project");
    }

    #[test]
    fn runtime_validation_flags_errors_and_warnings() {
        let mut runtime = EngineRuntime::new(GameEngineProject::default());
        runtime.project.metadata.name.clear();
        runtime.project.metadata.author.clear();
        let result = runtime.validate();
        assert!(!result.applied);
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, EditorEngineEvent::ValidationError(_))));
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, EditorEngineEvent::ValidationWarning(_))));
    }
}
