fn main() -> Result<(), Box<dyn std::error::Error>> {
    prost_build::compile_protos(
        &["../../shared/schemas/messages.proto"],
        &["../../shared/schemas/"],
    )?;
    Ok(())
}
