use std::{hash::Hash, ops::RangeInclusive};

use bevy_egui::egui;

pub struct StepperInputRowResponse {
    decrement_clicked: bool,
    increment_clicked: bool,
    text_response: egui::Response,
}

impl StepperInputRowResponse {
    pub fn changed(&self) -> bool {
        self.text_response.changed()
    }

    pub fn lost_focus(&self) -> bool {
        self.text_response.lost_focus()
    }

    pub fn has_focus(&self) -> bool {
        self.text_response.has_focus()
    }

    pub fn decrement_clicked(&self) -> bool {
        self.decrement_clicked
    }

    pub fn increment_clicked(&self) -> bool {
        self.increment_clicked
    }
}

pub fn property_grid(ui: &mut egui::Ui, id_source: impl Hash, add_rows: impl FnOnce(&mut egui::Ui)) {
    egui::Grid::new(id_source)
        .num_columns(4)
        .spacing([8.0, 10.0])
        .show(ui, add_rows);
}

#[allow(dead_code)]
pub fn stepper_input_row(
    ui: &mut egui::Ui,
    label: &str,
    buffer: &mut String,
) -> StepperInputRowResponse {
    stepper_input_row_enabled(ui, label, buffer, true)
}

pub fn stepper_input_row_enabled(
    ui: &mut egui::Ui,
    label: &str,
    buffer: &mut String,
    enabled: bool,
) -> StepperInputRowResponse {
    ui.label(label);
    let decrement_clicked = ui.add_enabled(enabled, egui::Button::new("-").small()).clicked();
    let text_response = ui.add_enabled(
        enabled,
        egui::TextEdit::singleline(buffer)
            .desired_width(96.0)
            .horizontal_align(egui::Align::Center),
    );
    let increment_clicked = ui.add_enabled(enabled, egui::Button::new("+").small()).clicked();
    ui.end_row();

    StepperInputRowResponse {
        decrement_clicked,
        increment_clicked,
        text_response,
    }
}

#[allow(dead_code)]
pub fn slider_row<Num>(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut Num,
    range: RangeInclusive<Num>,
    text: impl Into<egui::WidgetText>,
) -> egui::Response
where
    Num: egui::emath::Numeric,
{
    ui.label(label);
    let response = ui.add(egui::Slider::new(value, range).text(text));
    ui.end_row();
    response
}