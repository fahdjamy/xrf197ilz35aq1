mod server;
mod interceptors;
mod services;

pub use server::GrpcServer;

pub mod asset {
    tonic::include_proto!("asset_rpc");
}
