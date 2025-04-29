// src/hugetop.rs

use clap::{arg, crate_version, value_parser, ArgAction, Command};
use std::{
    fs,
    io::{BufRead, BufReader},
    thread::sleep,
    time::Duration,
};
use uucore::{error::UResult, format_usage, help_about, help_usage};

use bytesize::ByteSize;
use chrono::Local;

const ABOUT: &str = help_about!("hugetop.md");
const USAGE: &str = help_usage!("hugetop.md");

pub struct Settings {
    delay: Option<u64>,
    human: bool,
    once: bool,
    numa: bool,
}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    todo!();
}
