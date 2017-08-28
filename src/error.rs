// Copyright 2016 Adam Perry. Dual-licensed MIT and Apache 2.0 (see LICENSE files for details).

use std;
use std::convert::From;
use std::io;

use diesel;
use handlebars;
use hyper;
use r2d2;
use serde_json;

pub type DashResult<T> = std::result::Result<T, DashError>;

#[derive(Debug)]
pub enum DashError {
    Hyper(hyper::error::Error),
    Io(io::Error),
    Serde(serde_json::error::Error),
    R2d2Timeout(r2d2::GetTimeout),
    DieselError(diesel::result::Error),
    Template(handlebars::TemplateRenderError),
    Misc(Option<String>),
}

impl From<handlebars::TemplateRenderError> for DashError {
    fn from(e: handlebars::TemplateRenderError) -> Self { DashError::Template(e) }
}

impl From<hyper::error::Error> for DashError {
    fn from(e: hyper::error::Error) -> Self { DashError::Hyper(e) }
}

impl From<io::Error> for DashError {
    fn from(e: io::Error) -> Self { DashError::Io(e) }
}

impl From<serde_json::error::Error> for DashError {
    fn from(e: serde_json::error::Error) -> Self { DashError::Serde(e) }
}

impl From<r2d2::GetTimeout> for DashError {
    fn from(e: r2d2::GetTimeout) -> Self { DashError::R2d2Timeout(e) }
}

impl From<diesel::result::Error> for DashError {
    fn from(e: diesel::result::Error) -> Self { DashError::DieselError(e) }
}
