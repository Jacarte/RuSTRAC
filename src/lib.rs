// The of the tool are the trace 1 trace 2 and the distance function
//
extern crate termcolor;
use std::path::PathBuf;
use termcolor::{Ansi, ColorChoice, NoColor, StandardStream, WriteColor};

#[derive(clap::Parser)]
pub struct GeneralOpts {
    /// Use verbose output (-v info, -vv debug, -vvv trace).
    #[clap(long = "verbose", short = 'v', action = clap::ArgAction::Count)]
    verbose: u8,

    /// Use colors in output.
    #[clap(long = "color", default_value = "auto")]
    pub color: ColorChoice,
}

impl GeneralOpts {
    /// Initializes the logger based on the verbosity level.
    pub fn init_logger(&self) {
        let default = match self.verbose {
            0 => "warn",
            1 => "info",
            2 => "debug",
            _ => "trace",
        };

        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(default))
            .format_target(false)
            .init();
    }
}

// and then the methods are used to read the arguments,
#[derive(clap::Parser)]
pub struct InputOutput {
    /// Trace file1 to process.
    ///
    pub input1: PathBuf,

    /// Trace file2 to process.
    ///
    pub input2: PathBuf,

    ///// Use the trace1 as a memfile
    //trace1_memfile: bool,

    ///// Use the trace2 as a memfile
    //trace2_memfile: bool,
    #[clap(flatten)]
    output: OutputArg,

    #[clap(flatten)]
    general: GeneralOpts,

    // Make this an argument
    /// The cost of a gap
    #[arg(long)]
    pub gap_cost: Option<f64>,
    /// The cost of aligning two tokens that mismatch
    #[arg(long)]
    pub missmatch_cost: Option<f64>,
}

#[derive(clap::Parser)]
pub struct OutputArg {
    /// Where to place the alignment output.
    #[clap(long)]
    output1: Option<PathBuf>,

    #[clap(long)]
    output2: Option<PathBuf>,
    // The distance number is written to the stdout
}

impl InputOutput {
    pub fn general_opts(&self) -> &GeneralOpts {
        &self.general
    }
}
impl OutputArg {}
