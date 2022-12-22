use clap::{AppSettings, Parser};

use rhai::plugin::*;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(global_setting(AppSettings::DisableHelpSubcommand))]
struct Args {
    #[clap(value_parser)]
    path: std::path::PathBuf,
}

// My custom type
#[derive(Clone)]
pub struct TestStruct {
    pub value: i64,
}

pub struct Reader<'a> {
    reader: &'a mut (dyn std::io::BufRead + 'a),
    buffer: String,
}

impl<'a> Reader<'a> {
    fn new(reader: &'a mut (dyn std::io::BufRead + 'a)) -> Reader<'a> {
        Reader {
            reader: reader,
            buffer: String::new(),
        }
    }

    fn line<'f>(&'f mut self) -> &'f str {
        self.buffer.clear();
        self.reader.read_line(&mut self.buffer).unwrap();
        self.buffer.as_str()
    }
}

#[export_module]
mod my_module {
    // This type alias will register the friendly name 'ABC' for the
    // custom type 'TestStruct'.
    pub type ABC = TestStruct;

    // This constant will be registered as the constant variable 'MY_NUMBER'.
    // Ignored when registered as a global module.
    pub const MY_NUMBER: i64 = 42;

    // This function will be registered as 'greet'
    // but is only available with the 'greetings' feature.
    pub fn greet(name: &str) -> String {
        format!("hello, {}!", name)
    }
    /// This function will be registered as 'get_num'.
    ///
    /// If this is a Rust doc-comment, then it is included in the metadata.
    pub fn get_num() -> i64 {
        mystic_number()
    }
    /// This function will be registered as 'create_abc'.
    pub fn create_abc(value: i64) -> ABC {
        ABC { value }
    }
    /// This function will be registered as the 'value' property of type 'ABC'.
    #[rhai_fn(get = "value")]
    pub fn get_value(ts: &mut ABC) -> i64 {
        ts.value
    }
    // This function will be registered as 'increment'.
    // It will also be exposed to the global namespace since 'global' is set.
    #[rhai_fn(global)]
    pub fn increment(ts: &mut ABC) {
        ts.value += 1;
    }
    // This function is not 'pub', so NOT registered.
    fn mystic_number() -> i64 {
        42
    }

    // Sub-modules are ignored when the module is registered globally.
    pub mod my_sub_module {
        // This function is ignored when registered globally.
        // Otherwise it is a valid registered function under a sub-module.
        pub fn get_info() -> String {
            "say what".to_string()
        }
    }
}

pub fn main() -> Result<(), Box<rhai::EvalAltResult>> {
    let mut binding = std::io::stdin().lock();
    let mut stdin = Reader::new(&mut binding);
    print!("{}", stdin.line());

    let args = Args::parse();

    let mut engine = rhai::Engine::new();

    let module = exported_module!(my_module);
    engine.register_static_module("sh", module.into());

    engine.run_file(args.path)?;

    Ok(())
}
