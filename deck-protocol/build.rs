extern crate prost_build;
extern crate tower_grpc_build;

fn main() {
    prost_build::compile_protos(
        &[
            "proto/deck-core/core.proto",
            "proto/deck-core/build-package.proto",
            "proto/deck-core/get-build-log.proto",
            "proto/deck-core/query-package.proto",
        ],
        &["proto", "proto/deck-core"],
    ).unwrap_or_else(|e| panic!("protobuf compilation failed: {}", e));

    tower_grpc_build::Config::new()
        .enable_server(true)
        .enable_client(true)
        .build(
            &["proto/deck-builder/builder.proto"],
            &["proto", "proto/deck-builder"],
        ).unwrap_or_else(|e| panic!("protobuf compilation failed: {}", e));

    tower_grpc_build::Config::new()
        .enable_server(true)
        .enable_client(true)
        .build(
            &["proto/deck-binary-cache/binary-cache.proto"],
            &["proto", "proto/deck-binary-cache"],
        ).unwrap_or_else(|e| panic!("protobuf compilation failed: {}", e));

    #[cfg(feature = "daemon")]
    tower_grpc_build::Config::new()
        .enable_server(true)
        .enable_client(true)
        .build(
            &["proto/deck-daemon/daemon.proto"],
            &["proto", "proto/deck-daemon"],
        ).unwrap_or_else(|e| panic!("protobuf compilation failed: {}", e));
}
