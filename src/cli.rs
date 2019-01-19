use std::fs;
use std::io::{stdin, Read};
use std::path::PathBuf;

use clap::{_clap_count_exprs, arg_enum};
use failure::Error;
use serde_json;
use serde_value::Value;
use serde_yaml;
use structopt::StructOpt;
use toml;

arg_enum! {
    #[derive(PartialEq, Debug)]
    pub enum Format {
        Yaml,
        Toml,
        Json,
    }
}

#[derive(StructOpt, Debug)]
pub struct Opts {
    /// The format of the input data (default is guessing)
    #[structopt(
        long = "input-format",
        short = "f",
        value_name = "FORMAT",
        raw(possible_values = "&Format::variants()"),
        raw(case_insensitive = "true")
    )]
    pub input_format: Option<Format>,
    /// The output format (defaults to input format)
    #[structopt(
        long = "output-format",
        short = "F",
        value_name = "FORMAT",
        raw(possible_values = "&Format::variants()"),
        raw(case_insensitive = "true")
    )]
    pub output_format: Option<Format>,
    /// The name of the file to open (or stdin)
    #[structopt(value_name = "FILE")]
    pub file: Option<PathBuf>,
}

pub fn deserialize(input: &[u8], format: Format) -> Result<Value, Error> {
    match format {
        Format::Json => Ok(serde_json::from_slice(input)?),
        Format::Yaml => Ok(serde_yaml::from_slice(input)?),
        Format::Toml => Ok(toml::from_slice(input)?),
    }
}

pub fn serialize(value: &Value, format: Format) -> Result<String, Error> {
    match format {
        Format::Json => Ok(serde_json::to_string_pretty(&value)?),
        Format::Yaml => Ok(serde_yaml::to_string(&value)?),
        Format::Toml => Ok(toml::to_string(&value)?),
    }
}

pub fn execute() -> Result<(), Error> {
    let opts = Opts::from_args();

    let contents = match opts.file {
        None => {
            let mut buf = Vec::new();
            stdin().lock().read_to_end(&mut buf)?;
            buf
        }
        Some(filename) => fs::read(filename)?,
    };

    let parsed = deserialize(&contents, opts.input_format.unwrap_or(Format::Json))?;
    let output = serialize(&parsed, opts.output_format.unwrap_or(Format::Json))?;
    println!("{}", output);
    Ok(())
}
