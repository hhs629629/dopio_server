mod chart;

use std::{
    collections::{HashMap, VecDeque},
    ops::Range,
};

use plotters::{
    chart::ChartState,
    coord::{types::RangedCoordf64, Shift},
    drawing,
    prelude::*,
    series,
};
use plotters_canvas::CanvasBackend;
use wasm_bindgen::{prelude::*, JsCast};

use crate::chart::{ChartInfo, LineColor};

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    Ok(())
}

#[wasm_bindgen]
pub struct Plot {
    info: ChartInfo,
    drawing_area: DrawingArea<CanvasBackend, Shift>,
    chart: ChartState<Cartesian2d<RangedCoordf64, RangedCoordf64>>,
}

#[wasm_bindgen]
impl Plot {
    pub fn get_interval_as_millis(&self) -> u64 {
        self.info.interval.as_millis() as u64
    }
    pub fn init(canvas_id: &str, info: String) -> Result<Plot, JsError> {
        let info: ChartInfo = serde_json::from_str(&info).expect("data deserialization failed");

        let backend =
            CanvasBackend::new(canvas_id).expect(&format!("Invalid canvas_id: {}", canvas_id));
        let drawing_area = backend.into_drawing_area();
        let font: FontDesc = ("sans-serif", 20.0).into();

        drawing_area.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&drawing_area)
            .caption(info.caption.clone(), font)
            .x_label_area_size(30u32)
            .y_label_area_size(30u32)
            .margin_left(10)
            .build_cartesian_2d(0.0..100.0, info.y_range.clone())?;

        chart
            .configure_mesh()
            .disable_mesh()
            .x_label_formatter(&|_| "".to_string())
            .draw()
            .unwrap();

        Ok(Plot {
            info,
            chart: chart.into_chart_state(),
            drawing_area,
        })
    }

    pub fn update(&mut self, data: String) -> Result<(), JsValue> {
        let data: HashMap<String, (VecDeque<f64>, LineColor)> =
            serde_json::from_str(&data).expect("data deserialization failed");
        let state = self.chart.clone();
        let mut chart = state.restore(&self.drawing_area);

        chart.plotting_area().fill(&WHITE).unwrap();

        let mut data: Vec<(String, VecDeque<f64>, LineColor)> = data
            .into_iter()
            .map(|(label, (lines, color))| (label, lines, color))
            .collect();
        data.sort_by(|a, b| a.0.cmp(&b.0));

        match self.info.chart_type {
            chart::ChartType::Stack => Self::draw_stack_chart(&mut chart, data),
            chart::ChartType::PassThru(viewport_size) => {
                Self::draw_pass_thru_chart(&mut chart, data, viewport_size)
            }
        };

        chart
            .configure_series_labels()
            .position(SeriesLabelPosition::UpperRight)
            .margin(20)
            .legend_area_size(5)
            .label_font(("Calibri", 15))
            .draw()
            .unwrap();

        self.chart = chart.into_chart_state();

        self.drawing_area.present().unwrap();

        Ok(())
    }

    fn draw_pass_thru_chart(
        chart: &mut ChartContext<CanvasBackend, Cartesian2d<RangedCoordf64, RangedCoordf64>>,
        data: Vec<(String, VecDeque<f64>, LineColor)>,
        viewport_size: usize,
    ) {
        let d = (chart.x_range().end - chart.x_range().start) / (viewport_size - 1) as f64;

        for (label, lines, color) in data {
            let mut i = viewport_size - lines.len();
            let color = color.into_rgb_color();

            chart
                .draw_series(LineSeries::new(
                    lines.iter().map(|y| {
                        let ret = (i as f64 * d, *y);
                        i = i + 1;
                        ret
                    }),
                    color,
                ))
                .unwrap()
                .label(label)
                .legend(move |(x, y)| Rectangle::new([(x - 10, y + 1), (x, y)], color));
        }
    }

    fn draw_stack_chart(
        chart: &mut ChartContext<CanvasBackend, Cartesian2d<RangedCoordf64, RangedCoordf64>>,
        data: Vec<(String, VecDeque<f64>, LineColor)>,
    ) {
        let len = data[0].1.len();
        let d = (chart.x_range().end - chart.x_range().start) / (len - 1) as f64;

        for (label, lines, color) in data {
            let mut i = 0;
            let color = color.into_rgb_color();

            chart
                .draw_series(LineSeries::new(
                    lines.iter().map(|y| {
                        let ret = (i as f64 * d, *y);
                        i = i + 1;
                        ret
                    }),
                    color,
                ))
                .unwrap()
                .label(label)
                .legend(move |(x, y)| Rectangle::new([(x - 10, y + 1), (x, y)], color));
        }
    }
}
