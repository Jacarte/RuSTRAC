extern crate anyhow;
extern crate clap;
extern crate dtw as dtw_core;
extern crate termcolor;

use anyhow::*;
use clap::Parser;
use dtw_core::parsing::TraceEncoder;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

// This code is copied and transformed from the wasm-tools repo
//
//
macro_rules! subcommands {
    ($(
        $(#[$attr:meta])*
        ($name:ident, $string:tt)
    )*) => {
        $(
            // #[cfg(feature = $string)]
            mod $name;
        )*

        #[derive(Parser)]
        // #[clap(version = version())]
        #[allow(non_camel_case_types)]
        enum DTWTools {
            $(
                // #[cfg(feature = $string)]
                $(#[$attr])*
                $name($name::Opts),
            )*
        }

        impl DTWTools {
            fn run(self, tr1: Box<dyn dtw_core::dtw::Accesor>, tr2: Box<dyn dtw_core::dtw::Accesor>, distance: Box<dyn dtw_core::dtw::Distance>) -> f64 {
                match self {
                    $(
                        // #[cfg(feature = $string)]
                        Self::$name(opts) => opts.run(tr1, tr2, distance),
                    )*
                }
            }

            fn general_opts(&self) -> &dtw_tools::GeneralOpts {
                match *self {
                    $(
                        // #[cfg(feature = $string)]
                        Self::$name(ref opts) => opts.general_opts(),
                    )*
                }
            }

            fn io(&self) -> &dtw_tools::InputOutput {
                match *self {
                    $(
                        // #[cfg(feature = $string)]
                        Self::$name(ref opts) => opts.io(),
                    )*
                }
            }

        }
    }
}

subcommands! {
    (dtw, "dtw")
    // (memodtw, "memodtw")
}

fn main() {
    let args = <DTWTools as Parser>::parse();
    args.general_opts().init_logger();
    let mut encoder = dtw_core::parsing::ToMemoryParser::default();

    log::debug!("Preprocessing as text files");
    // Check if the flag is bin or text file
    // Read the input files
    // Read from fs as text
    let trace1 = std::fs::read_to_string(&args.io().input1).expect("Could not read file");
    let trace2 = std::fs::read_to_string(&args.io().input2).expect("Could not read file");

    // Separate the tokens by endline
    // TODO replace with a custom separator
    let trace1 = trace1
        .split('\n')
        .map(|s| String::from(s))
        .collect::<Vec<_>>();
    let trace2 = trace2
        .split('\n')
        .map(|s| String::from(s))
        .collect::<Vec<_>>();

    log::debug!("Generating bin traces");

    // Get the name of the file
    let name1 = args.io().input1.file_name().unwrap().to_str().unwrap();
    let name2 = args.io().input2.file_name().unwrap().to_str().unwrap();

    let r1 = encoder.create_bin(trace1, PathBuf::from(format!("{}.trace.bin", name1)));
    let r2 = encoder.create_bin(trace2, PathBuf::from(format!("{}.trace.bin", name2)));

    log::debug!("Runnning DTW");

    let distance = dtw_core::dtw::STRACDistance::new(
        args.io().gap_cost.or(Some(1.0)).unwrap(),
        args.io().missmatch_cost.or(Some(3.0)).unwrap(),
        0.0
        );
    let distance = Box::new(distance);

    let distance = args.run(Box::new(r1), Box::new(r2), distance);

    println!("{}", distance);
}

