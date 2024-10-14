use prost_build::Config;
use std::io::Result;

fn main() -> Result<()> {
    let mut config = Config::new();
    let protos = &["protos/motor.proto", "protos/chassis.proto"];
    let includes = &[" protos/"];

    config
        .enable_type_names()
        .type_name_domain(["."], "type.googleapis.com");

    prost_reflect_build::Builder::new()
        .file_descriptor_set_bytes("DESCRIPTOR_SET_BYTES")
        .configure(&mut config, protos, includes)?;

    config.compile_protos(protos, includes)?;
    Ok(())
}
