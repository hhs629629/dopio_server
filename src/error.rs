use thiserror::Error;

pub enum Error {
    InvalidLineLabelError,
    InvalidChartNumberError,
    AlreadyExistIndexError,
}
