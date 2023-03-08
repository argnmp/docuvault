fn main () -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../protos/upload.proto")?;
    tonic_build::compile_protos("../protos/download.proto")?;
    tonic_build::compile_protos("../protos/delete.proto")?;
    Ok(())
}
