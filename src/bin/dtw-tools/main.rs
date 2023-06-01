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

        #[derive(Parser, Clone)]
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
            fn run(self, tr1: Box<dyn dtw_core::dtw::Accesor>, tr2: Box<dyn dtw_core::dtw::Accesor>, distance: Box<dyn dtw_core::dtw::Distance>) -> dtw_core::dtw::DTWResult {
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
    (memodtw, "memodtw")
    (fastdtw, "fastdtw")
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
    let argsclone = args.clone();
    let name1 = argsclone.io().input1.file_name().unwrap().to_str().unwrap();
    let name2 = argsclone.io().input2.file_name().unwrap().to_str().unwrap();
    let n1clone = name1.clone();
    let n2clone = name2.clone();

    let r1 = encoder.create_bin(trace1, PathBuf::from(format!("{}.trace.bin", name1.clone())));
    let r2 = encoder.create_bin(trace2, PathBuf::from(format!("{}.trace.bin", name2.clone())));

    log::debug!("Runnning DTW");

    let distance = dtw_core::dtw::STRACDistance::new(
        args.io().gap_cost.or(Some(1.0)).unwrap(),
        args.io().missmatch_cost.or(Some(3.0)).unwrap(),
        0.0,
    );
    let output_alignment = args.io().output.output_alignment.clone();
    let gap_symbol = args.io().output.gap_symbol.clone();

    let distance = Box::new(distance);
    let (distance, wp) = args.run(Box::new(r1.clone()), Box::new(r2.clone()), distance);

    // Now we create the alignment using the warping path
    if let Some(wp) = wp {
        if let Some(pb) = output_alignment {
            // Open the file for writing
            let mut file = std::fs::File::create(pb).unwrap();

            let mut tr1p: Vec<Option<usize>> = vec![];
            let mut tr2p: Vec<Option<usize>> = vec![];
            // Traverse the warping path in reverse order

            for index in 0..wp.len() - 1 {
                
                let reversed = index;
                let i2 = wp[reversed as usize];
                let i1 = wp[(reversed + 1) as usize];
                
                if i2.0 > i1.0 && i2.1 > i1.1 {
                    // Write the alignment
                    tr1p.push(Some(i1.0));
                    tr2p.push(Some(i1.1));
                } else if i2.1 > i1.1 {
                    tr1p.push(None);
                    tr2p.push(Some(i1.1));
                } else if i2.0 > i1.0 {
                    tr2p.push(None);
                    tr1p.push(Some(i1.0));
                }

            }

            assert_eq!(tr1p.len(), tr2p.len());

            writeln!(file, "Alignment: {} --- {}", n1clone, n2clone).unwrap();

            for (i1, i2) in tr1p.iter().rev().zip(tr2p.iter().rev()) {
                match (i1, i2) {
                    (Some(i1), Some(i2)) => {
                        let t1 = r1.get(*i1).unwrap();
                        let t2 = r2.get(*i2).unwrap();
                        let t1 = encoder.id_to_token(*t1);
                        let t2 = encoder.id_to_token(*t2);
                        let eq = if t1 == t2 { "o" } else { "m" };
                        writeln!(file, "{} {} --- {}",eq, t1, t2).unwrap();
                    }
                    (None, Some(i2)) => {
                        let t2 = r2.get(*i2).unwrap();
                        let t2 = encoder.id_to_token(*t2);
                        writeln!(file, "g {} --- {}", gap_symbol, t2).unwrap();
                    }
                    (Some(i1), None) => {
                        let t1 = r1.get(*i1).unwrap();
                        let t1 = encoder.id_to_token(*t1);
                        writeln!(file, "g {} --- {}", t1, gap_symbol).unwrap();
                    }
                    _ => {}
                }
            }
        }
    }

    println!("{}", distance);
}
