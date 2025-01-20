mod asset_service;
mod server;
pub mod interceptors;

pub use server::GrpcServer;

pub mod asset {
    tonic::include_proto!("asset_rpc");
}
