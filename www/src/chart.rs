use serde::Deserialize;
use std::ops::Range;

use plotters::style::RGBColor;

#[derive(Deserialize)]
pub enum ChartType {
    Stack,
    PassThru(usize),
}

#[derive(Deserialize)]
pub struct LineColor {
    r: u8,
    g: u8,
    b: u8,
}

impl LineColor {
    pub fn into_rgb_color(self) -> RGBColor {
        RGBColor(self.r, self.g, self.b)
    }
}

#[derive(Deserialize)]
pub struct ChartInfo {
    pub caption: String,
    pub chart_type: ChartType,
    pub interval: std::time::Duration,
    pub y_range: Range<f64>,
}
