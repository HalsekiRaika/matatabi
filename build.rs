fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos(r#"protos/cage.proto"#)?;
    tonic_build::compile_protos(r#"protos/salmon.proto"#)?;
    Ok(())
}