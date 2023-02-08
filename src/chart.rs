use std::{
    collections::VecDeque,
    ops::Range,
    sync::{Mutex, RwLock},
    time::Duration,
};

use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Serialize)]
pub enum ChartType {
    Stack,
    PassThru(usize),
}

#[derive(Serialize, Deserialize)]
pub struct LineColor {
    r: u8,
    g: u8,
    b: u8,
}

impl LineColor {
    pub fn init(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

#[derive(Serialize)]
pub struct ChartInfo {
    caption: String,
    chart_type: ChartType,
    interval: std::time::Duration,
    y_range: Range<f64>,
}

#[derive(Serialize)]
pub struct Chart {
    info: ChartInfo,
    lines: std::collections::HashMap<String, (VecDeque<f64>, LineColor)>,
}

impl Chart {
    pub fn new(
        caption: String,
        chart_type: ChartType,
        interval: std::time::Duration,
        y_range: Range<f64>,
    ) -> Self {
        Chart {
            info: ChartInfo {
                caption,
                chart_type,
                interval,
                y_range,
            },
            lines: std::collections::HashMap::new(),
        }
    }
    fn new_label(&mut self, label: String, color: LineColor) {
        self.lines.insert(label, (VecDeque::new(), color));
    }

    fn insert_data(&mut self, label: String, data: f64) -> Result<(), Error> {
        let (line, _) = self
            .lines
            .get_mut(&label)
            .ok_or(Error::InvalidLineLabelError)?;

        match &self.info.chart_type {
            ChartType::Stack => {
                line.push_back(data);
            }
            ChartType::PassThru(viewport_size) => {
                if line.len() == *viewport_size {
                    line.pop_front();
                }
                line.push_back(data);
            }
        }

        Ok(())
    }

    fn resize_viewport(&mut self, size: usize) {
        match self.info.chart_type {
            ChartType::Stack => return,
            ChartType::PassThru(_) => self.info.chart_type = ChartType::PassThru(size),
        }
    }

    fn clear(&mut self) {
        self.lines.clear();
    }
}

pub struct Charts {
    charts: RwLock<endorphin::HashMap<usize, RwLock<Chart>, endorphin::policy::TTIPolicy>>,
}

impl Charts {
    pub fn new() -> Self {
        Charts {
            charts: RwLock::new(endorphin::HashMap::new(endorphin::policy::TTIPolicy::new())),
        }
    }

    pub fn contains(&self, index: usize) -> bool {
        let read_lock = self.charts.read().unwrap();
        read_lock.contains_key(&index)
    }

    pub fn insert_chart(&self, index: usize, chart: Chart, tti: Duration) -> Result<(), Error> {
        let mut write_lock = self.charts.write().unwrap();
        if write_lock.contains_key(&index) {
            Err(Error::AlreadyExistIndexError)
        } else {
            write_lock.insert(index, RwLock::new(chart), tti);
            Ok(())
        }
    }

    pub fn new_label(&self, index: usize, label: String, color: LineColor) -> Result<(), Error> {
        let read_lock = self.charts.read().unwrap();
        let mut chart = read_lock
            .get(&index)
            .ok_or(Error::InvalidChartNumberError)?
            .write()
            .unwrap();

        chart.new_label(label, color);

        Ok(())
    }
    pub fn insert_data(&self, index: usize, label: String, data: f64) -> Result<(), Error> {
        let read_lock = self.charts.read().unwrap();
        let mut chart = read_lock
            .get(&index)
            .ok_or(Error::InvalidChartNumberError)?
            .write()
            .unwrap();

        chart.insert_data(label, data)?;

        Ok(())
    }

    pub fn resize_viewport(&mut self, index: usize) {}

    pub fn get_lines_as_json_string(&self, index: usize) -> Result<String, Error> {
        let chart_lock = self.charts.read().unwrap();
        let chart = chart_lock
            .get(&index)
            .ok_or(Error::InvalidChartNumberError)?
            .read()
            .unwrap();

        Ok(serde_json::to_string(&chart.lines).unwrap())
    }

    pub fn get_info_as_json_string(&self, index: usize) -> Result<String, Error> {
        let chart_lock = self.charts.read().unwrap();
        let chart = chart_lock
            .get(&index)
            .ok_or(Error::InvalidChartNumberError)?
            .read()
            .unwrap();

        Ok(serde_json::to_string(&chart.info).unwrap())
    }
}
