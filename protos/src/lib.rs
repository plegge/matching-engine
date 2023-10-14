// Declare the proto module
pub mod proto {
    tonic::include_proto!("matchingengine_v1");  // Includes the generated Protobuf code
}

pub use proto::*;