use crate::prelude::*;
use ToolEvent::*;

pub struct PixelArtTool {
    id: TheId,
    mode_index: i32,
    palette_index: i32,
    brush_size: f32,
    opacity: f32,
    zoom: f32,
    show_grid: bool,
}

impl PixelArtTool {
    fn build_creation_menu(&self) -> TheContextMenu {
        let mut menu = TheContextMenu::named("Pixel Art".to_string());
        menu.add(TheContextMenuItem::new(
            "New Sprite".to_string(),
            TheId::named("PixelArt New Sprite"),
        ));
        menu.add(TheContextMenuItem::new(
            "New Tileset".to_string(),
            TheId::named("PixelArt New Tileset"),
        ));
        menu.add(TheContextMenuItem::new(
            "Import PNG".to_string(),
            TheId::named("PixelArt Import PNG"),
        ));
        menu.add(TheContextMenuItem::new(
            "Export PNG".to_string(),
            TheId::named("PixelArt Export PNG"),
        ));
        menu
    }

    pub fn build_creation_panel(&self) -> TheCanvas {
        let mut canvas = TheCanvas::new();

        let mut tools_layout = TheVLayout::new(TheId::named("PixelArt Panel Layout"));
        tools_layout.set_margin(Vec4::new(10, 10, 10, 10));

        let mut title = TheText::new(TheId::empty());
        title.set_text("Pixel Art Panel".to_string());
        tools_layout.add_widget(Box::new(title));

        let mut palette = TheDropdownMenu::new(TheId::named("PixelArt Panel Palette"));
        palette.add_option("Default".to_string());
        palette.add_option("Warm".to_string());
        palette.add_option("Cold".to_string());
        palette.add_option("Dungeon".to_string());
        palette.set_selected_index(self.palette_index);
        tools_layout.add_widget(Box::new(palette));

        let mut brush = TheSlider::new(TheId::named("PixelArt Panel Brush"));
        brush.set_range(TheValue::RangeF32(1.0..=64.0));
        brush.set_value(TheValue::Float(self.brush_size));
        brush.set_continuous(true);
        tools_layout.add_widget(Box::new(brush));

        canvas.set_layout(tools_layout);
        canvas
    }

    pub fn build_creation_window(&self) -> TheCanvas {
        let mut window = TheCanvas::new();
        let mut body = TheCanvas::new();
        body.set_layout(TheRGBALayout::new(TheId::named("PixelArt Tool Canvas")));
        window.set_center(body);
        window.set_top(self.build_creation_panel());
        window
    }

    fn build_palette_widget(&self) -> TheDropdownMenu {
        let mut palette = TheDropdownMenu::new(TheId::named("PixelArt Palette"));
        palette.add_option("Default".to_string());
        palette.add_option("Warm".to_string());
        palette.add_option("Cold".to_string());
        palette.add_option("Dungeon".to_string());
        palette.set_selected_index(self.palette_index);
        palette
    }
}

impl Tool for PixelArtTool {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            id: TheId::named("PixelArt Tool"),
            mode_index: 0,
            palette_index: 0,
            brush_size: 8.0,
            opacity: 1.0,
            zoom: 8.0,
            show_grid: true,
        }
    }

    fn id(&self) -> TheId {
        self.id.clone()
    }

    fn info(&self) -> String {
        "Pixel Art Tool (P). Create sprites, tilesets, and icon sheets.".to_string()
    }

    fn icon_name(&self) -> String {
        "brush".to_string()
    }

    fn accel(&self) -> Option<char> {
        Some('P')
    }

    fn help_url(&self) -> Option<String> {
        Some("docs/creator/tools/pixelart".to_string())
    }

    fn tool_event(
        &mut self,
        tool_event: ToolEvent,
        ui: &mut TheUI,
        _ctx: &mut TheContext,
        _project: &mut Project,
        server_ctx: &mut ServerContext,
    ) -> bool {
        match tool_event {
            Activate => {
                server_ctx.curr_map_tool_type = MapToolType::General;
                if let Some(layout) = ui.get_hlayout("Game Tool Params") {
                    layout.clear();

                    let mut mode = TheGroupButton::new(TheId::named("PixelArt Mode"));
                    mode.add_text_status("Draw".to_string(), "Draw pixels.".to_string());
                    mode.add_text_status("Erase".to_string(), "Erase pixels.".to_string());
                    mode.add_text_status("Fill".to_string(), "Flood fill.".to_string());
                    mode.add_text_status("Select".to_string(), "Select rectangle.".to_string());
                    mode.set_item_width(72);
                    mode.set_index(self.mode_index);
                    layout.add_widget(Box::new(mode));

                    let mut brush = TheSlider::new(TheId::named("PixelArt Brush Size"));
                    brush.set_range(TheValue::RangeF32(1.0..=64.0));
                    brush.set_value(TheValue::Float(self.brush_size));
                    brush.set_continuous(true);
                    brush.limiter_mut().set_max_width(110);
                    layout.add_widget(Box::new(brush));

                    let mut opacity = TheSlider::new(TheId::named("PixelArt Opacity"));
                    opacity.set_range(TheValue::RangeF32(0.0..=1.0));
                    opacity.set_value(TheValue::Float(self.opacity));
                    opacity.set_continuous(true);
                    opacity.limiter_mut().set_max_width(110);
                    layout.add_widget(Box::new(opacity));

                    let mut zoom = TheSlider::new(TheId::named("PixelArt Zoom"));
                    zoom.set_range(TheValue::RangeF32(1.0..=32.0));
                    zoom.set_value(TheValue::Float(self.zoom));
                    zoom.set_continuous(true);
                    zoom.limiter_mut().set_max_width(110);
                    layout.add_widget(Box::new(zoom));

                    layout.add_widget(Box::new(self.build_palette_widget()));

                    let mut grid_toggle = TheCheckButton::new(TheId::named("PixelArt Grid Toggle"));
                    if self.show_grid {
                        grid_toggle.set_state(TheWidgetState::Selected);
                    }
                    layout.add_widget(Box::new(grid_toggle));

                    let mut create_btn = TheTraybarButton::new(TheId::named("PixelArt Create"));
                    create_btn.set_text("Create".to_string());
                    create_btn.set_context_menu(Some(self.build_creation_menu()));
                    create_btn.set_status_text("Create a sprite, tileset, or icon sheet.");
                    layout.add_widget(Box::new(create_btn));

                    let mut panel_btn =
                        TheTraybarButton::new(TheId::named("PixelArt Open Panel"));
                    panel_btn.set_text("Panel".to_string());
                    panel_btn.set_status_text("Open pixel art panel window.");
                    layout.add_widget(Box::new(panel_btn));

                    layout.set_reverse_index(Some(1));
                }
                true
            }
            DeActivate => {
                if let Some(layout) = ui.get_hlayout("Game Tool Params") {
                    layout.clear();
                    layout.set_reverse_index(None);
                }
                true
            }
            _ => false,
        }
    }

    fn handle_event(
        &mut self,
        event: &TheEvent,
        _ui: &mut TheUI,
        ctx: &mut TheContext,
        _project: &mut Project,
        _server_ctx: &mut ServerContext,
    ) -> bool {
        match event {
            TheEvent::IndexChanged(id, index) => {
                if id.name == "PixelArt Mode" {
                    self.mode_index = *index as i32;
                    return true;
                } else if id.name == "PixelArt Palette" || id.name == "PixelArt Panel Palette" {
                    self.palette_index = *index as i32;
                    return true;
                }
            }
            TheEvent::ValueChanged(id, TheValue::FloatRange(v, _))
            | TheEvent::ValueChanged(id, TheValue::Float(v)) => {
                if id.name == "PixelArt Brush Size" || id.name == "PixelArt Panel Brush" {
                    self.brush_size = *v;
                    return true;
                } else if id.name == "PixelArt Opacity" {
                    self.opacity = *v;
                    return true;
                } else if id.name == "PixelArt Zoom" {
                    self.zoom = *v;
                    return true;
                }
            }
            TheEvent::StateChanged(id, state) => {
                if id.name == "PixelArt Grid Toggle" {
                    self.show_grid = *state == TheWidgetState::Selected;
                    return true;
                }
                if id.name == "PixelArt Open Panel" && *state == TheWidgetState::Clicked {
                    ctx.ui.send(TheEvent::Custom(
                        TheId::named("PixelArt Show Window"),
                        TheValue::Text("open".to_string()),
                    ));
                    return true;
                }
            }
            _ => {}
        }

        false
    }
}
