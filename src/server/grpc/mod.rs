mod server;
mod interceptors;
mod services;
mod header;

pub use header::{get_xrf_user_auth_header, XRF_USER_FINGERPRINT};
pub use server::GrpcServer;

pub mod asset {
    tonic::include_proto!("asset_rpc");
    tonic::include_proto!("proto.contract.v1");
}
