use crate::prelude::*;

pub struct RPGMakerMenu {
    pub id: TheId,
}

impl RPGMakerMenu {
    pub fn new(id: TheId) -> Self {
        Self { id }
    }

    pub fn context_menu(&self) -> TheContextMenu {
        let mut menu = TheContextMenu::named(self.id.name.clone());
        menu.add(TheContextMenuItem::new_with_accel(
            "New Project".to_string(),
            TheId::named("RPG New Project"),
            TheAccelerator::new(TheAcceleratorKey::CTRLCMD, 'n'),
        ));
        menu.add(TheContextMenuItem::new_with_accel(
            "Open Project".to_string(),
            TheId::named("RPG Open Project"),
            TheAccelerator::new(TheAcceleratorKey::CTRLCMD, 'o'),
        ));
        menu.add(TheContextMenuItem::new_with_accel(
            "Save Project".to_string(),
            TheId::named("RPG Save Project"),
            TheAccelerator::new(TheAcceleratorKey::CTRLCMD, 's'),
        ));
        menu.add_separator();
        menu.add(TheContextMenuItem::new(
            "Import Assets".to_string(),
            TheId::named("RPG Import Assets"),
        ));
        menu.add(TheContextMenuItem::new(
            "Export Game".to_string(),
            TheId::named("RPG Export Game"),
        ));
        menu.add_separator();
        menu.add(TheContextMenuItem::new(
            "Game Settings".to_string(),
            TheId::named("RPG Game Settings"),
        ));
        menu.add(TheContextMenuItem::new_with_accel(
            "Test Play".to_string(),
            TheId::named("RPG Test Play"),
            TheAccelerator::new(TheAcceleratorKey::CTRLCMD, 't'),
        ));
        menu.add_separator();
        menu.add(TheContextMenuItem::new(
            "Exit".to_string(),
            TheId::named("RPG Exit"),
        ));
        menu
    }
}
