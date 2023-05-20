use std::{error::Error};

fn main() -> Result<(), Box<dyn Error>> {
    let protos: Vec<&str> = vec![r#"src/proto/messages.proto"#];

    let includes: Vec<&str> = vec![r#"src/"#];

    prost_build::compile_protos(&protos, &includes)?;
    Ok(())
}
