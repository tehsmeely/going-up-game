use bevy_egui::egui;
use bevy_egui::egui::Sense;

pub fn fill_bar(
    ui: &mut egui::Ui,
    bg_color: egui::Color32,
    fill_color: egui::Color32,
    fraction: f32,
) {
    let desired_size = ui.spacing().interact_size * egui::vec2(1.0, 0.2);
    let (rect, _) = ui.allocate_exact_size(desired_size, Sense::hover());
    let fill_height = rect.height() * fraction;
    let mut fill_rect = rect.clone();
    fill_rect.set_height(fill_height);

    if ui.is_rect_visible(rect) {
        ui.painter().rect_filled(rect, 0.0, bg_color);
        ui.painter().rect_filled(fill_rect, 0.0, fill_color);
    }
}
