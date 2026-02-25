use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub type UeActorId = u64;
pub type UeLevelId = String;
pub type UeNodeId = u64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UeDimensionMode {
    TwoD,
    ThreeD,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct UeVec2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct UeVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct UeTransform {
    pub location: UeVec3,
    pub rotation: UeVec3,
    pub scale: UeVec3,
}

impl Default for UeTransform {
    fn default() -> Self {
        Self {
            location: UeVec3::default(),
            rotation: UeVec3::default(),
            scale: UeVec3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UeComponent {
    Sprite2D { sprite_id: String, layer: i32 },
    Tilemap2D { tilemap_id: String, collision: bool },
    StaticMesh3D { mesh_id: String, material_id: String },
    SkeletalMesh3D { mesh_id: String, animation_set: String },
    Collider2D { size: UeVec2, trigger: bool },
    Collider3D { size: UeVec3, trigger: bool },
    Light2D { intensity: f32, radius: f32 },
    Light3D { intensity: f32, range: f32 },
    Camera2D { zoom: f32 },
    Camera3D { fov: f32, near: f32, far: f32 },
    AudioSource { cue_id: String, volume: f32, looped: bool },
    ScriptGraph { graph: UeBlueprintGraph },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UeActor {
    pub id: UeActorId,
    pub name: String,
    pub enabled: bool,
    pub transform: UeTransform,
    pub tags: Vec<String>,
    pub parent: Option<UeActorId>,
    pub children: Vec<UeActorId>,
    pub components: Vec<UeComponent>,
}

impl UeActor {
    pub fn new(id: UeActorId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            enabled: true,
            transform: UeTransform::default(),
            tags: vec![],
            parent: None,
            children: vec![],
            components: vec![],
        }
    }

    pub fn with_component(mut self, c: UeComponent) -> Self {
        self.components.push(c);
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UeLevel {
    pub id: UeLevelId,
    pub name: String,
    pub mode: UeDimensionMode,
    pub actors: HashMap<UeActorId, UeActor>,
    pub next_actor_id: UeActorId,
}

impl UeLevel {
    pub fn new(id: impl Into<String>, name: impl Into<String>, mode: UeDimensionMode) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            mode,
            actors: HashMap::new(),
            next_actor_id: 1,
        }
    }

    pub fn spawn_actor(&mut self, name: impl Into<String>) -> UeActorId {
        let id = self.next_actor_id;
        self.next_actor_id = self.next_actor_id.saturating_add(1);
        self.actors.insert(id, UeActor::new(id, name));
        id
    }

    pub fn remove_actor(&mut self, id: UeActorId) -> bool {
        let Some(actor) = self.actors.remove(&id) else {
            return false;
        };
        if let Some(parent_id) = actor.parent
            && let Some(parent) = self.actors.get_mut(&parent_id)
        {
            parent.children.retain(|c| *c != id);
        }
        for child_id in actor.children {
            if let Some(child) = self.actors.get_mut(&child_id) {
                child.parent = None;
            }
        }
        true
    }

    pub fn duplicate_actor(&mut self, source: UeActorId) -> Option<UeActorId> {
        let src = self.actors.get(&source)?.clone();
        let new_id = self.next_actor_id;
        self.next_actor_id = self.next_actor_id.saturating_add(1);
        let mut clone = src;
        clone.id = new_id;
        clone.name = format!("{} Copy", clone.name);
        clone.parent = None;
        clone.children.clear();
        self.actors.insert(new_id, clone);
        Some(new_id)
    }

    pub fn attach_actor(&mut self, child_id: UeActorId, parent_id: UeActorId) -> bool {
        if child_id == parent_id {
            return false;
        }
        if !self.actors.contains_key(&child_id) || !self.actors.contains_key(&parent_id) {
            return false;
        }
        let mut cursor = Some(parent_id);
        while let Some(id) = cursor {
            if id == child_id {
                return false;
            }
            cursor = self.actors.get(&id).and_then(|a| a.parent);
        }

        let old_parent = self.actors.get(&child_id).and_then(|a| a.parent);
        if let Some(old_parent_id) = old_parent
            && let Some(p) = self.actors.get_mut(&old_parent_id)
        {
            p.children.retain(|c| *c != child_id);
        }
        if let Some(parent) = self.actors.get_mut(&parent_id)
            && !parent.children.contains(&child_id)
        {
            parent.children.push(child_id);
        }
        if let Some(child) = self.actors.get_mut(&child_id) {
            child.parent = Some(parent_id);
        }
        true
    }

    pub fn detach_actor(&mut self, child_id: UeActorId) -> bool {
        let Some(child) = self.actors.get_mut(&child_id) else {
            return false;
        };
        let parent_id = child.parent.take();
        if let Some(parent_id) = parent_id
            && let Some(parent) = self.actors.get_mut(&parent_id)
        {
            parent.children.retain(|c| *c != child_id);
        }
        true
    }

    pub fn snap_actor_to_grid(&mut self, actor_id: UeActorId, grid: f32) -> bool {
        if grid <= 0.0 {
            return false;
        }
        let Some(actor) = self.actors.get_mut(&actor_id) else {
            return false;
        };
        actor.transform.location.x = (actor.transform.location.x / grid).round() * grid;
        actor.transform.location.y = (actor.transform.location.y / grid).round() * grid;
        if self.mode == UeDimensionMode::ThreeD {
            actor.transform.location.z = (actor.transform.location.z / grid).round() * grid;
        }
        true
    }

    pub fn actors_with_tag<'a>(&'a self, tag: &str) -> Vec<&'a UeActor> {
        let tag_l = tag.to_lowercase();
        self.actors
            .values()
            .filter(|a| a.tags.iter().any(|t| t.to_lowercase() == tag_l))
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UeBlueprintValue {
    Bool(bool),
    Int(i64),
    Float(f32),
    Vec2(UeVec2),
    Vec3(UeVec3),
    Text(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UeBlueprintNodeKind {
    EventBeginPlay,
    EventTick,
    Branch {
        condition_key: String,
    },
    MoveActorBy {
        actor_id: UeActorId,
        delta: UeVec3,
    },
    RotateActorBy {
        actor_id: UeActorId,
        delta: UeVec3,
    },
    SetTag {
        actor_id: UeActorId,
        tag: String,
    },
    PlaySfx {
        cue_id: String,
    },
    EmitEvent {
        key: String,
        value: UeBlueprintValue,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UeBlueprintNode {
    pub id: UeNodeId,
    pub kind: UeBlueprintNodeKind,
    pub next_true: Option<UeNodeId>,
    pub next_false: Option<UeNodeId>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct UeBlueprintGraph {
    pub entry: Option<UeNodeId>,
    pub nodes: HashMap<UeNodeId, UeBlueprintNode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UeBlueprintRuntimeEvent {
    TagSet {
        actor_id: UeActorId,
        tag: String,
    },
    SfxPlayed {
        cue_id: String,
    },
    Custom {
        key: String,
        value: UeBlueprintValue,
    },
}

impl UeBlueprintGraph {
    pub fn execute(
        &self,
        level: &mut UeLevel,
        vars: &HashMap<String, UeBlueprintValue>,
        tick_mode: bool,
        max_steps: usize,
    ) -> Vec<UeBlueprintRuntimeEvent> {
        let mut out = Vec::new();
        let mut current = self.entry;
        let mut steps = 0usize;
        let mut visited: HashSet<UeNodeId> = HashSet::new();

        while let Some(node_id) = current {
            if steps >= max_steps || !visited.insert(node_id) {
                break;
            }
            steps += 1;
            let Some(node) = self.nodes.get(&node_id) else {
                break;
            };

            let mut advance = node.next_true;
            match &node.kind {
                UeBlueprintNodeKind::EventBeginPlay => {
                    if tick_mode {
                        break;
                    }
                }
                UeBlueprintNodeKind::EventTick => {
                    if !tick_mode {
                        break;
                    }
                }
                UeBlueprintNodeKind::Branch { condition_key } => {
                    let condition = match vars.get(condition_key) {
                        Some(UeBlueprintValue::Bool(v)) => *v,
                        _ => false,
                    };
                    advance = if condition {
                        node.next_true
                    } else {
                        node.next_false
                    };
                }
                UeBlueprintNodeKind::MoveActorBy { actor_id, delta } => {
                    if let Some(actor) = level.actors.get_mut(actor_id) {
                        actor.transform.location.x += delta.x;
                        actor.transform.location.y += delta.y;
                        actor.transform.location.z += delta.z;
                    }
                }
                UeBlueprintNodeKind::RotateActorBy { actor_id, delta } => {
                    if let Some(actor) = level.actors.get_mut(actor_id) {
                        actor.transform.rotation.x += delta.x;
                        actor.transform.rotation.y += delta.y;
                        actor.transform.rotation.z += delta.z;
                    }
                }
                UeBlueprintNodeKind::SetTag { actor_id, tag } => {
                    if let Some(actor) = level.actors.get_mut(actor_id)
                        && !actor.tags.contains(tag)
                    {
                        actor.tags.push(tag.clone());
                    }
                    out.push(UeBlueprintRuntimeEvent::TagSet {
                        actor_id: *actor_id,
                        tag: tag.clone(),
                    });
                }
                UeBlueprintNodeKind::PlaySfx { cue_id } => {
                    out.push(UeBlueprintRuntimeEvent::SfxPlayed {
                        cue_id: cue_id.clone(),
                    });
                }
                UeBlueprintNodeKind::EmitEvent { key, value } => {
                    out.push(UeBlueprintRuntimeEvent::Custom {
                        key: key.clone(),
                        value: value.clone(),
                    });
                }
            }
            current = advance;
        }
        out
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UeEditorCommand {
    CreateLevel {
        id: UeLevelId,
        name: String,
        mode: UeDimensionMode,
    },
    SetActiveLevel(UeLevelId),
    SpawnActor {
        name: String,
    },
    RemoveActor {
        actor_id: UeActorId,
    },
    DuplicateActor {
        actor_id: UeActorId,
    },
    MoveActor {
        actor_id: UeActorId,
        delta: UeVec3,
    },
    RotateActor {
        actor_id: UeActorId,
        delta: UeVec3,
    },
    SnapActorToGrid {
        actor_id: UeActorId,
        grid: f32,
    },
    AttachActor {
        child_id: UeActorId,
        parent_id: UeActorId,
    },
    DetachActor {
        child_id: UeActorId,
    },
    AddComponent {
        actor_id: UeActorId,
        component: UeComponent,
    },
    SetPlayMode(bool),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UeEditorEvent {
    LevelCreated(UeLevelId),
    ActiveLevelChanged(UeLevelId),
    ActorSpawned(UeActorId),
    ActorRemoved(UeActorId),
    ActorDuplicated {
        source: UeActorId,
        duplicated: UeActorId,
    },
    ActorTransformed(UeActorId),
    ActorAttached {
        child_id: UeActorId,
        parent_id: UeActorId,
    },
    ActorDetached(UeActorId),
    ComponentAdded(UeActorId),
    PlayModeChanged(bool),
    Warning(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct UeEditorCommandResult {
    pub ok: bool,
    pub events: Vec<UeEditorEvent>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct UeEditorRuntime {
    pub levels: HashMap<UeLevelId, UeLevel>,
    pub active_level: Option<UeLevelId>,
    pub play_mode: bool,
    pub sim_time_seconds: f32,
}

impl UeEditorRuntime {
    pub fn apply(&mut self, cmd: UeEditorCommand) -> UeEditorCommandResult {
        let mut result = UeEditorCommandResult::default();
        match cmd {
            UeEditorCommand::CreateLevel { id, name, mode } => {
                if self.levels.contains_key(&id) {
                    result
                        .events
                        .push(UeEditorEvent::Warning(format!("Level {id} already exists.")));
                    return result;
                }
                self.levels
                    .insert(id.clone(), UeLevel::new(id.clone(), name, mode));
                self.active_level = Some(id.clone());
                result.ok = true;
                result.events.push(UeEditorEvent::LevelCreated(id.clone()));
                result.events.push(UeEditorEvent::ActiveLevelChanged(id));
            }
            UeEditorCommand::SetActiveLevel(id) => {
                if self.levels.contains_key(&id) {
                    self.active_level = Some(id.clone());
                    result.ok = true;
                    result.events.push(UeEditorEvent::ActiveLevelChanged(id));
                }
            }
            UeEditorCommand::SetPlayMode(v) => {
                self.play_mode = v;
                result.ok = true;
                result.events.push(UeEditorEvent::PlayModeChanged(v));
            }
            other => {
                let Some(active_id) = self.active_level.clone() else {
                    result
                        .events
                        .push(UeEditorEvent::Warning("No active level.".to_string()));
                    return result;
                };
                let Some(level) = self.levels.get_mut(&active_id) else {
                    result.events.push(UeEditorEvent::Warning(
                        "Active level not found.".to_string(),
                    ));
                    return result;
                };

                match other {
                    UeEditorCommand::SpawnActor { name } => {
                        let id = level.spawn_actor(name);
                        result.ok = true;
                        result.events.push(UeEditorEvent::ActorSpawned(id));
                    }
                    UeEditorCommand::RemoveActor { actor_id } => {
                        result.ok = level.remove_actor(actor_id);
                        if result.ok {
                            result.events.push(UeEditorEvent::ActorRemoved(actor_id));
                        }
                    }
                    UeEditorCommand::DuplicateActor { actor_id } => {
                        if let Some(new_id) = level.duplicate_actor(actor_id) {
                            result.ok = true;
                            result.events.push(UeEditorEvent::ActorDuplicated {
                                source: actor_id,
                                duplicated: new_id,
                            });
                        }
                    }
                    UeEditorCommand::MoveActor { actor_id, delta } => {
                        if let Some(actor) = level.actors.get_mut(&actor_id) {
                            actor.transform.location.x += delta.x;
                            actor.transform.location.y += delta.y;
                            actor.transform.location.z += delta.z;
                            result.ok = true;
                            result.events.push(UeEditorEvent::ActorTransformed(actor_id));
                        }
                    }
                    UeEditorCommand::RotateActor { actor_id, delta } => {
                        if let Some(actor) = level.actors.get_mut(&actor_id) {
                            actor.transform.rotation.x += delta.x;
                            actor.transform.rotation.y += delta.y;
                            actor.transform.rotation.z += delta.z;
                            result.ok = true;
                            result.events.push(UeEditorEvent::ActorTransformed(actor_id));
                        }
                    }
                    UeEditorCommand::SnapActorToGrid { actor_id, grid } => {
                        result.ok = level.snap_actor_to_grid(actor_id, grid);
                        if result.ok {
                            result.events.push(UeEditorEvent::ActorTransformed(actor_id));
                        }
                    }
                    UeEditorCommand::AttachActor {
                        child_id,
                        parent_id,
                    } => {
                        result.ok = level.attach_actor(child_id, parent_id);
                        if result.ok {
                            result.events.push(UeEditorEvent::ActorAttached {
                                child_id,
                                parent_id,
                            });
                        }
                    }
                    UeEditorCommand::DetachActor { child_id } => {
                        result.ok = level.detach_actor(child_id);
                        if result.ok {
                            result.events.push(UeEditorEvent::ActorDetached(child_id));
                        }
                    }
                    UeEditorCommand::AddComponent {
                        actor_id,
                        component,
                    } => {
                        if let Some(actor) = level.actors.get_mut(&actor_id) {
                            actor.components.push(component);
                            result.ok = true;
                            result.events.push(UeEditorEvent::ComponentAdded(actor_id));
                        }
                    }
                    UeEditorCommand::CreateLevel { .. }
                    | UeEditorCommand::SetActiveLevel(_)
                    | UeEditorCommand::SetPlayMode(_) => {}
                }
            }
        }
        result
    }

    pub fn simulate_tick(
        &mut self,
        dt_seconds: f32,
        vars: &HashMap<String, UeBlueprintValue>,
    ) -> Vec<UeBlueprintRuntimeEvent> {
        if !self.play_mode {
            return vec![];
        }
        self.sim_time_seconds += dt_seconds.max(0.0);
        let Some(level_id) = self.active_level.clone() else {
            return vec![];
        };
        let Some(level) = self.levels.get_mut(&level_id) else {
            return vec![];
        };

        let actor_ids: Vec<UeActorId> = level.actors.keys().copied().collect();
        let mut out = vec![];
        for actor_id in actor_ids {
            let Some(actor) = level.actors.get(&actor_id).cloned() else {
                continue;
            };
            for component in actor.components {
                if let UeComponent::ScriptGraph { graph } = component {
                    out.extend(graph.execute(level, vars, true, 64));
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_create_level_and_spawn_actor() {
        let mut rt = UeEditorRuntime::default();
        let r = rt.apply(UeEditorCommand::CreateLevel {
            id: "lvl_a".to_string(),
            name: "Level A".to_string(),
            mode: UeDimensionMode::TwoD,
        });
        assert!(r.ok);
        let r2 = rt.apply(UeEditorCommand::SpawnActor {
            name: "Hero".to_string(),
        });
        assert!(r2.ok);
        assert_eq!(rt.levels["lvl_a"].actors.len(), 1);
    }

    #[test]
    fn runtime_duplicate_attach_and_snap() {
        let mut rt = UeEditorRuntime::default();
        rt.apply(UeEditorCommand::CreateLevel {
            id: "lvl".to_string(),
            name: "Level".to_string(),
            mode: UeDimensionMode::ThreeD,
        });
        rt.apply(UeEditorCommand::SpawnActor {
            name: "Parent".to_string(),
        });
        rt.apply(UeEditorCommand::SpawnActor {
            name: "Child".to_string(),
        });

        let ids: Vec<_> = rt.levels["lvl"].actors.keys().copied().collect();
        let p = ids[0];
        let c = ids[1];

        let attach = rt.apply(UeEditorCommand::AttachActor {
            child_id: c,
            parent_id: p,
        });
        assert!(attach.ok);

        let dup = rt.apply(UeEditorCommand::DuplicateActor { actor_id: p });
        assert!(dup.ok);

        rt.levels
            .get_mut("lvl")
            .unwrap()
            .actors
            .get_mut(&p)
            .unwrap()
            .transform
            .location = UeVec3 {
            x: 2.6,
            y: 7.4,
            z: 1.2,
        };
        let snap = rt.apply(UeEditorCommand::SnapActorToGrid {
            actor_id: p,
            grid: 1.0,
        });
        assert!(snap.ok);
        let loc = rt.levels["lvl"].actors[&p].transform.location;
        assert_eq!(loc, UeVec3 { x: 3.0, y: 7.0, z: 1.0 });
    }

    #[test]
    fn blueprint_tick_moves_actor() {
        let mut rt = UeEditorRuntime::default();
        rt.apply(UeEditorCommand::CreateLevel {
            id: "lvl".to_string(),
            name: "Level".to_string(),
            mode: UeDimensionMode::TwoD,
        });
        rt.apply(UeEditorCommand::SpawnActor {
            name: "Mover".to_string(),
        });
        let actor_id = *rt.levels["lvl"].actors.keys().next().unwrap();

        let graph = UeBlueprintGraph {
            entry: Some(1),
            nodes: HashMap::from([(
                1,
                UeBlueprintNode {
                    id: 1,
                    kind: UeBlueprintNodeKind::MoveActorBy {
                        actor_id,
                        delta: UeVec3 {
                            x: 1.0,
                            y: 0.0,
                            z: 0.0,
                        },
                    },
                    next_true: None,
                    next_false: None,
                },
            )]),
        };

        rt.apply(UeEditorCommand::AddComponent {
            actor_id,
            component: UeComponent::ScriptGraph { graph },
        });
        rt.apply(UeEditorCommand::SetPlayMode(true));
        let ev = rt.simulate_tick(0.016, &HashMap::new());
        assert!(ev.is_empty());
        assert_eq!(rt.levels["lvl"].actors[&actor_id].transform.location.x, 1.0);
    }
}
