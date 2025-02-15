mod server;
mod interceptors;
mod services;
mod header;

pub use header::{get_header_value, XRF_USER_FINGERPRINT};
pub use server::GrpcServer;

pub mod asset {
    tonic::include_proto!("asset_rpc");
    tonic::include_proto!("proto.contract.v1");
}
