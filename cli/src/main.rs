use envja::{self, DynError};
use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Envja CLI",
    about = "CLI for Jinja2-like env var interpolation"
)]
struct Config {
    /// Choose file / direct content mode
    #[structopt(subcommand)]
    mode: Mode,

    /// Redirects output into path
    #[structopt(short = "o")]
    output_path: Option<PathBuf>,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "Envja CLI mode", about = "File / direct mode")]
enum Mode {
    #[structopt(name = "file", about = "Read from file")]
    File {
        /// File path to read template content from
        #[structopt(parse(from_os_str))]
        path: PathBuf,
    },
    #[structopt(
        name = "direct",
        about = "Read directly from argument / STDIN"
    )]
    Direct {
        /// Template content to parse. Reads from STDIN if left empty.
        #[structopt()]
        content: Option<String>,
    },
}

fn main() -> Result<(), DynError> {
    let conf = Config::from_args();
    let mappings = env::vars().collect();

    let template = match conf.mode {
        Mode::File { path } => fs::read_to_string(&path)?,
        Mode::Direct { content } => {
            if let Some(content) = content {
                content
            } else {
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)?;
                buffer
            }
        }
    };

    let write_to_file = |out: &mut io::Write| -> Result<(), DynError> {
        Ok(out
            .write_all(envja::interpolate(&template, &mappings)?.as_bytes())?)
    };

    if let Some(output_path) = conf.output_path {
        let mut output_file = fs::File::create(output_path)?;
        write_to_file(&mut output_file)
    } else {
        let mut out = io::stdout();
        write_to_file(&mut out)
    }?;

    Ok(())
}
