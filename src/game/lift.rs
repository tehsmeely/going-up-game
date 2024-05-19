use crate::game::ui;
use crate::loading::TextureAssets;
use bevy::app::App;
use bevy::log::error;
use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;
use bevy_egui::egui::{Align2, Color32, RichText};
use bevy_egui::{egui, EguiContexts};
use std::fmt::{Display, Formatter};
use std::time::Duration;

#[derive(Resource, Debug, Reflect)]
pub struct LiftHumanStore {
    slots: Vec<StoredHumanSlot>,
    max_size: usize,
}

const EGUI_UI_ENABLED: bool = true;
#[derive(Debug, Reflect, Clone, Copy)]
pub enum HumanKind {
    Simon,
}
impl Display for HumanKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HumanKind::Simon => write!(f, "Simon"),
        }
    }
}

#[derive(Debug, Reflect, Clone)]
pub struct StoredHuman {
    destination_floor: i32,
    patience_timer: Timer,
    kind: HumanKind,
}
#[derive(Debug, Reflect, Clone)]
pub struct StoredHumanSlot(Option<StoredHuman>);

impl StoredHumanSlot {
    fn ui_component(&self, ui: &mut egui::Ui, texture_ids: &(egui::TextureId, egui::TextureId)) {
        let text_color = Color32::WHITE;
        let size = 24.0;

        // TODO: The below is broken because the patience timer is hard
        /*
        match &self.0 {
            Some(human) => {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.add(egui::Image::new(egui::load::SizedTexture::new(
                            texture_ids.0,
                            [20., 30.],
                        )));
                        ui.label(
                            RichText::new(human.destination_floor.to_string())
                                .color(text_color)
                                .size(size),
                        );
                    });
                    crate::ui_widgets::fill_bar(
                        ui,
                        Color32::BLACK,
                        Color32::GREEN,
                        human.patience_timer.fraction_remaining(),
                    );
                });
            }
            None => {
                ui.add(egui::Image::new(egui::load::SizedTexture::new(
                    texture_ids.1,
                    [20., 30.],
                )));
            }
        }
        */
        let (texture, dest_str) = match &self.0 {
            Some(human) => (texture_ids.0, human.destination_floor.to_string()),
            None => (texture_ids.1, "".into()),
        };
        ui.horizontal(|ui| {
            ui.add(egui::Image::new(egui::load::SizedTexture::new(
                texture,
                [20., 30.],
            )));
            ui.label(RichText::new(dest_str).color(text_color).size(size));
        });
    }
}

pub fn add(app: &mut App) {
    app.register_type::<LiftHumanStore>()
        .register_type::<HumanKind>()
        .register_type::<StoredHuman>()
        .register_type::<StoredHumanSlot>()
        .insert_resource(LiftHumanStore::create());
}

impl LiftHumanStore {
    pub fn create() -> Self {
        let max_size = 6;
        let slots = vec![StoredHumanSlot(None); max_size];
        Self { slots, max_size }
    }

    fn sort_slots(&mut self) {
        self.slots.sort_by_key(|slot| {
            if let Some(human) = &slot.0 {
                human.destination_floor
            } else {
                i32::MAX
            }
        })
    }

    fn add_single_(&mut self, floor: i32, patience: Duration, sort_on_insert: bool) -> bool {
        let mut inserted = false;
        for slot in self.slots.iter_mut() {
            if slot.0.is_none() {
                slot.0 = Some(StoredHuman {
                    destination_floor: floor,
                    patience_timer: Timer::new(patience, TimerMode::Once),
                    kind: HumanKind::Simon,
                });
                inserted = true;
                break;
            }
        }
        if inserted && sort_on_insert {
            self.sort_slots();
        }
        return inserted;
    }
    /// Returns true if the human was successfully added to the store, false if there
    /// was not enough space.
    pub fn add_single(&mut self, floor: i32, patience: Duration) -> bool {
        self.add_single_(floor, patience, true)
    }
    /// The length of floors vec must be less than or equal to the number of free slots, this is
    /// checked but not enforced, surplus humans will simply cease to exist - sorry.
    pub fn add(&mut self, floors: Vec<i32>) {
        for floor in floors.iter() {
            // TODO: get a real duration
            let result = self.add_single_(*floor, Duration::from_secs(10), false);
            if !result {
                error!(
                    "Failed to add all humans to lift store, expected there to always be enough space"
                );
            }
        }
        self.sort_slots();
    }

    pub fn take_for_floor(&mut self, floor_num: i32) -> Vec<Duration> {
        let mut taken = vec![];
        for slot in self.slots.iter_mut() {
            let mut clear = false;
            if let Some(stored_human) = &slot.0 {
                if stored_human.destination_floor == floor_num {
                    taken.push(stored_human.patience_timer.remaining());
                    clear = true;
                }
            }
            if clear {
                slot.0 = None;
            }
        }
        taken
    }

    pub fn free_capacity(&self) -> usize {
        self.slots.iter().filter(|slot| slot.0.is_none()).count()
    }

    pub fn update_system(mut humans: ResMut<Self>, time: Res<Time>) {
        for slot in humans.slots.iter_mut() {
            if let Some(human) = &mut slot.0 {
                human.patience_timer.tick(time.delta());
            }
        }
    }
    pub fn draw_system(
        mut humans: Res<Self>,
        mut contexts: EguiContexts,
        texture_assets: Res<TextureAssets>,
        mut texture_ids: Local<(egui::TextureId, egui::TextureId)>,
        mut is_initialized: Local<bool>,
    ) {
        if EGUI_UI_ENABLED {
            if !*is_initialized {
                *is_initialized = true;
                *texture_ids = (
                    contexts.add_image(texture_assets.human_icon_on.clone_weak()),
                    contexts.add_image(texture_assets.human_icon_off.clone_weak()),
                );
            }
            let frame = ui::default_frame();
            let num_columns = 3;
            egui::Window::new("Held Humans")
                .movable(false)
                //.resizable(false)
                .anchor(Align2::RIGHT_TOP, egui::Vec2::ZERO)
                .title_bar(false)
                .frame(frame)
                .show(contexts.ctx_mut(), |ui| {
                    egui::Grid::new("held human slots")
                        .num_columns(num_columns)
                        .show(ui, |ui| {
                            for (i, slot) in humans.slots.iter().enumerate() {
                                if i % num_columns == 0 {
                                    ui.end_row();
                                }
                                slot.ui_component(ui, &texture_ids);
                            }
                        });
                });
        }
    }
}
