extern crate prost_build;
extern crate tower_grpc_build;

fn main() {
    tower_grpc_build::Config::new()
        .enable_server(true)
        .enable_client(true)
        .build(
            &["proto/daemon/v1alpha1/daemon.proto"],
            &["proto/daemon/v1alpha1"],
        ).unwrap_or_else(|e| panic!("protobuf compilation failed: {}", e));
}
