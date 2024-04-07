use crate::game::ui;
use bevy::app::App;
use bevy::log::error;
use bevy::prelude::{Reflect, Res, Resource};
use bevy::time::{Timer, TimerMode};
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

    /// Returns true if the human was successfully added to the store, false if there
    /// was not enough space.
    pub fn add_single(&mut self, floor: i32, patience: Duration) -> bool {
        for slot in self.slots.iter_mut() {
            if slot.0.is_none() {
                slot.0 = Some(StoredHuman {
                    destination_floor: floor,
                    patience_timer: Timer::new(patience, TimerMode::Once),
                    kind: HumanKind::Simon,
                });
                return true;
            }
        }
        false
    }
    /// The length of floors vec must be less than or equal to the number of free slots, this is
    /// checked but not enforced, surplus humans will simply cease to exist - sorry.
    pub fn add(&mut self, floors: Vec<i32>) {
        for floor in floors.iter() {
            // TODO: get a real duration
            let result = self.add_single(*floor, Duration::from_secs(10));
            if !result {
                error!(
                    "Failed to add all humans to lift store, expected there to always be enough space"
                );
            }
        }
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
    pub fn update_system(mut humans: Res<Self>, mut contexts: EguiContexts) {
        let frame = ui::default_frame();
        let text_color = Color32::WHITE;
        let size = 24.0;
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
                            ui.label(
                                RichText::new(format!(
                                    "Slot {}: {}",
                                    i,
                                    slot.0
                                        .as_ref()
                                        .map(|human| format!(
                                            "{} to {}",
                                            human.kind, human.destination_floor
                                        ))
                                        .unwrap_or_else(|| "Empty".to_string())
                                ))
                                .color(text_color)
                                .size(size),
                            );
                        }
                    })
            });
    }
}
