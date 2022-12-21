pub fn main() -> Result<(), Box<rhai::EvalAltResult>> {
    let engine = rhai::Engine::new();
    let script = "print(40 + 2);";
    engine.run(script)?;
    Ok(())
}
