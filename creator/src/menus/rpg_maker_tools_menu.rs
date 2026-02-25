use crate::prelude::*;

pub struct RPGMakerToolsMenu {
    pub id: TheId,
}

impl RPGMakerToolsMenu {
    pub fn new(id: TheId) -> Self {
        Self { id }
    }

    pub fn context_menu(&self) -> TheContextMenu {
        let mut menu = TheContextMenu::named(self.id.name.clone());
        menu.add(TheContextMenuItem::new(
            "Map Editor".to_string(),
            TheId::named("RPG Tool Map Editor"),
        ));
        menu.add(TheContextMenuItem::new(
            "Character Editor".to_string(),
            TheId::named("RPG Tool Character Editor"),
        ));
        menu.add(TheContextMenuItem::new(
            "Item Editor".to_string(),
            TheId::named("RPG Tool Item Editor"),
        ));
        menu.add(TheContextMenuItem::new(
            "Event Editor".to_string(),
            TheId::named("RPG Tool Event Editor"),
        ));
        menu.add(TheContextMenuItem::new(
            "Database".to_string(),
            TheId::named("RPG Tool Database"),
        ));
        menu.add(TheContextMenuItem::new(
            "Tileset Editor".to_string(),
            TheId::named("RPG Tool Tileset Editor"),
        ));
        menu.add(TheContextMenuItem::new(
            "Script Editor".to_string(),
            TheId::named("RPG Tool Script Editor"),
        ));
        menu.add(TheContextMenuItem::new(
            "Sound Manager".to_string(),
            TheId::named("RPG Tool Sound Manager"),
        ));
        menu.add(TheContextMenuItem::new_with_accel(
            "Test Play".to_string(),
            TheId::named("RPG Tool Test Play"),
            TheAccelerator::new(TheAcceleratorKey::CTRLCMD, 'p'),
        ));
        menu
    }
}
