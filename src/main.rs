use rhai::plugin::*;        // a "prelude" import for macros

// My custom type
#[derive(Clone)]
pub struct TestStruct {
    pub value: i64
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
    // This global function defines a custom operator '@'.
    #[rhai_fn(name = "@", global)]
    pub fn square_add(x: i64, y: i64) -> i64 {
        x * x + y * y
    }

    // Sub-modules are ignored when the module is registered globally.
    pub mod my_sub_module {
        // This function is ignored when registered globally.
        // Otherwise it is a valid registered function under a sub-module.
        pub fn get_info() -> String {
            "hello".to_string()
        }
    }

    // Sub-modules are commonly used to put feature gates on a group of
    // functions because feature gates cannot be put on function definitions.
    // This is currently a limitation of the plugin procedural macros.
    #[cfg(feature = "advanced_functions")]
    pub mod advanced {
        // This function is ignored when registered globally.
        // Otherwise it is a valid registered function under a sub-module
        // which only exists when the 'advanced_functions' feature is used.
        pub fn advanced_calc(input: i64) -> i64 {
            input * 2
        }
    }
}


pub fn main() -> Result<(), Box<rhai::EvalAltResult>> {
    let mut engine = rhai::Engine::new();

      // The macro call creates a Rhai module from the plugin module.
    let module = exported_module!(my_module);

    // A module can simply be registered into the global namespace.
    engine.register_global_module(module.into());

    // Define a custom operator '@' with precedence of 160 (i.e. between +|- and *|/).
    engine.register_custom_operator("@", 160).unwrap();

    let script = r#"
        let s = `{"path": "/foo"}`;
        let meta = parse_json(s);
        let route = switch meta.path {
            "/" => "index.html",
            "/foo" => "foo.html",
            _ => "404.sh"
        };
        print(route);
        print(get_num());
        print(greet("Sam"));
    "#;
    engine.run(script)?;
    Ok(())
}
