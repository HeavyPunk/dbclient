mod service_generator;

fn main() {
    service_generator::configure()
        .compile_protos(
            &[
                "src/core/proto/dbclient/connections.proto",
                "src/core/proto/dbclient/objects.proto",
                "src/core/proto/dbclient/queries.proto",
                "src/core/proto/dbclient/common.proto",
            ],
            &["src/core/proto/dbclient"],
        )
        .unwrap();
}
