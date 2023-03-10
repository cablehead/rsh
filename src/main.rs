use std::sync::Arc;
use std::sync::Mutex;

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
    // we need to wrap the BufRead in an Arc/Mutex as rhai requires objects to be Clone-able
    // there might be around that I haven't come across yet...
    reader: Arc<Mutex<Box<dyn std::io::BufRead>>>,
}

impl Reader {
    fn new(reader: Box<dyn std::io::BufRead>) -> Reader {
        Reader {
            reader: Arc::new(Mutex::new(reader)),
        }
    }
}

#[export_module]
mod my_module {
    pub fn stdin() -> Reader {
        let stdin = std::io::stdin().lock();
        Reader::new(Box::new(stdin))
    }

    pub fn open(path: String) -> Reader {
        let file = std::fs::File::open(path).unwrap();
        let file = std::io::BufReader::new(file);
        Reader::new(Box::new(file))
    }

    #[rhai_fn(global)]
    pub fn line(ts: &mut Reader) -> rhai::Dynamic {
        let mut buffer = String::new();
        let n = ts.reader.lock().unwrap().read_line(&mut buffer).unwrap();
        if n > 0 {
            buffer.into()
        } else {
            ().into()
        }
    }

    #[rhai_fn(global)]
    // todo: surface parse errors
    pub fn json(ts: &mut Reader) -> rhai::Dynamic {
        let mut reader = ts.reader.lock().unwrap();
        // by using serde_json's into_iter, we only consume Reader until the end the next Value,
        // preserving the remainder of Reader for additional reads / redirection
        let deserializer = serde_json::Deserializer::from_reader(&mut *reader);
        let mut iterator = deserializer.into_iter::<rhai::Dynamic>();
        let more = iterator.next();
        match more {
            Some(result) => result.unwrap().into(),
            None => ().into(),
        }
    }
}

pub fn main() -> Result<(), Box<rhai::EvalAltResult>> {
    let mut engine = rhai::Engine::new();
    let module = exported_module!(my_module);
    engine.register_static_module("sh", module.into());

    let args = Args::parse();
    engine.run_file(args.path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_reader_line() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "1").unwrap();
        writeln!(file, "2").unwrap();
        writeln!(file, "3").unwrap();

        let mut engine = rhai::Engine::new();
        let module = exported_module!(my_module);
        engine.register_static_module("sh", module.into());

        let script = format!(
            r#"
                let reader = sh::open("{}");
                let got = [];
                loop {{
                    let line = reader.line();
                    got += line;
                    if line == () {{ break; }}
                }}

                got == ["1\n", "2\n", "3\n", ()]
                "#,
            file.path().display(),
        );
        let ok = engine.eval::<bool>(&script).unwrap();
        assert!(ok);
    }

    #[test]
    fn test_reader_json() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"foo": "bar"}}"#).unwrap();

        let mut engine = rhai::Engine::new();
        let module = exported_module!(my_module);
        engine.register_static_module("sh", module.into());

        let script = format!(
            r#"
            let reader = sh::open("{}");
            if reader.json().foo != "bar" {{ return false; }}
            reader.json() == ()
            "#,
            file.path().display(),
        );
        let ok = engine.eval::<bool>(&script).unwrap();
        assert!(ok);
    }
}
