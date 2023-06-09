// The of the tool are the trace 1 trace 2 and the distance function
//
extern crate termcolor;
use std::path::PathBuf;
use termcolor::ColorChoice;

#[derive(clap::Parser, Clone)]
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
#[derive(clap::Parser, Clone)]
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
    pub output: OutputArg,

    #[clap(flatten)]
    general: GeneralOpts,

    // Make this an argument
    /// The cost of a gap
    #[arg(long)]
    pub gap_cost: Option<f64>,
    /// The cost of aligning two tokens that mismatch
    #[arg(long)]
    pub missmatch_cost: Option<f64>,

    /// Separator as a regular expression
    #[arg(long, default_value = "\n")]
    pub separator: String,

    /// Cleaner of the traces. After spliting the document with the regex separator, the tokens will be cleaned out. This will be helpful for removing jumps and other noise.
    // set thos optional
    #[clap(flatten)]
    pub cleaner: CleanerArg,

    /// If the output alignemtn flag is set, then the cleaned trace is outputted
    #[arg(long, default_value="false")]
    pub output_cleaned_trace: bool
}


#[derive(clap::Parser, Clone)]
pub struct CleanerArg {
    /// Cleaner regex
    #[arg(long)]
    pub cleaner_regex: Option<String>,

    /// Extracted groupd
    #[arg(long, default_value = "0")]
    pub cleaner_extract: Option<usize>,
}

#[derive(clap::Parser, Clone)]
pub struct OutputArg {
    /// Where to place the alignment output.
    #[arg(long)]
    pub output_alignment: Option<PathBuf>,

    #[arg(long, default_value = "-")]
    pub gap_symbol: char,
}

impl InputOutput {
    pub fn general_opts(&self) -> &GeneralOpts {
        &self.general
    }
}
impl OutputArg {}
