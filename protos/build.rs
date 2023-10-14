fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .compile(
            &["./protos/matching_engine/service.proto"],
            &[
                "protos/matching_engine",
                // "protos/third_party/googleapis",
            ],
        )?;
    Ok(())
}