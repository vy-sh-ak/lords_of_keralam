use std::{ops::RangeInclusive};

use bevy_egui::egui;

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