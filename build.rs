fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(false)
        // needed for older protoc packages:
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_protos(
            &[
                "proto/coolercontrol/models/v1/device.proto",
                "proto/coolercontrol/device_service/v1/device_service.proto",
            ],
            &["proto"],
        )?;
    Ok(())
}
