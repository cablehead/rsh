pub fn main() -> Result<(), Box<rhai::EvalAltResult>> {
    let engine = rhai::Engine::new();
    let script = r#"
        let s = `{"path": "/foo"}`;
        let meta = parse_json(s);
        let route = switch meta.path {
            "/" => "index.html",
            "/foo" => "foo.html",
            _ => "404.sh"
        };
        print(route);
    "#;
    engine.run(script)?;
    Ok(())
}
