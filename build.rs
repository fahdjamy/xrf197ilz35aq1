fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::compile_protos("proto/asset.proto")?;
    tonic_prost_build::compile_protos("proto/contract/v1/contract.proto")?;
    Ok(())
}
