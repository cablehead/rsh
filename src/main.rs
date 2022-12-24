use std::sync::{Arc, Mutex};

use clap::{AppSettings, Parser};

use rhai::plugin::*;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(global_setting(AppSettings::DisableHelpSubcommand))]
struct Args {
    #[clap(value_parser)]
    path: std::path::PathBuf,
}

#[derive(Clone)]
pub struct Reader {
    reader: Arc<Mutex<Box<dyn std::io::BufRead>>>,
}

impl Reader {
    fn new(reader: Box<dyn std::io::BufRead>) -> Reader {
        Reader {
            reader: Arc::new(Mutex::new(reader)),
        }
    }

    pub fn line(&mut self) -> rhai::ImmutableString {
        let mut buffer = String::new();
        self.reader.lock().unwrap().read_line(&mut buffer).unwrap();
        buffer.into()
    }
}

#[export_module]
mod my_module {
    pub fn stdin() -> Reader {
        let stdin = std::io::stdin().lock();
        Reader::new(Box::new(stdin))
    }

    #[rhai_fn(global)]
    pub fn line(ts: &mut Reader) -> rhai::ImmutableString {
        let mut buffer = String::new();
        ts.reader.lock().unwrap().read_line(&mut buffer).unwrap();
        buffer.into()
    }
}

pub fn main() -> Result<(), Box<rhai::EvalAltResult>> {
    let args = Args::parse();

    let mut engine = rhai::Engine::new();

    let module = exported_module!(my_module);
    engine.register_static_module("sh", module.into());

    engine.run_file(args.path)?;

    Ok(())
}
