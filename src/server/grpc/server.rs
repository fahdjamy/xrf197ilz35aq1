use crate::common::{generate_request_id, REQUEST_ID_KEY};
use crate::configs::GrpcServerConfig;
use crate::server::grpc::asset::asset_service_server::AssetServiceServer;
use crate::server::grpc::asset::contract_service_server::ContractServiceServer;
use crate::server::grpc::services::{AssetServiceManager, ContractServiceManager};
use anyhow::Context;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls_pemfile::Item;
use sqlx::PgPool;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tonic::metadata::{KeyAndValueRef, MetadataKey, MetadataValue};
use tonic::transport::{Error, Server};
use tonic::{Request, Status};
use tower::ServiceBuilder;
use tracing::{info, info_span, warn};

pub struct GrpcServer {
    timeout: Duration,
    addr: core::net::SocketAddr,
    asset_service: AssetServiceManager,
    contract_service: ContractServiceManager,
}

impl GrpcServer {
    pub fn new(pg_pool: PgPool, config: GrpcServerConfig) -> Result<Self, anyhow::Error> {
        let addr = format!("[::]:{}", config.port)
            .parse()
            .context("Failed to parse grpc server address")?;

        // Create the PgArc, so we only have one strong reference initially
        let pg_pool_arc = Arc::new(pg_pool);

        // create the services
        let asset_service = AssetServiceManager::new(pg_pool_arc.clone());
        let contract_service = ContractServiceManager::new(pg_pool_arc.clone());

        let config_timeout = config.timeout;

        Ok(Self {
            addr,
            asset_service,
            contract_service,
            timeout: Duration::from_millis(config_timeout as u64),
        })
    }

    pub async fn run_until_stopped(self) -> Result<(), Error> {
        info!("starting gRPC server :: port {}", &self.addr.port());

        // Tower: Setting up interceptors
        // Stack of middleware that the service will be wrapped in
        let tower_layers = ServiceBuilder::new()
            // Apply request-id interceptor
            .layer(tonic::service::interceptor(Self::request_id_interceptor))
            .into_inner();

        Server::builder()
            .layer(tower_layers)
            .max_connection_age(self.timeout)
            .add_service(AssetServiceServer::new(self.asset_service))
            .add_service(ContractServiceServer::new(self.contract_service))
            .serve(self.addr)
            .await
    }

    fn request_id_interceptor(mut req: Request<()>) -> Result<Request<()>, Status> {
        let req_id = generate_request_id();
        let span = info_span!("gRPC", request_id = req_id);
        let _guard = span.enter();

        req.metadata_mut()
            .insert(MetadataKey::from_static(REQUEST_ID_KEY), MetadataValue::from_str(&req_id).unwrap());

        let mut header_string = String::new();

        for key_and_value in req.metadata_mut().iter() {
            let value_str = match key_and_value {
                KeyAndValueRef::Ascii(key, val) => {
                    &format!("{}:{:?}", key.as_str(), val)
                }
                KeyAndValueRef::Binary(key, val) => {
                    &format!("{}:{:?}", key.as_str(), val)
                }
            };

            header_string.push_str(&format!("{}; ", value_str));
        }

        Ok(req)
    }
}

fn load_certs(cert_path: &Path, key_path: &Path)
              -> anyhow::Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {

    // Load certificate(s).
    let cert_file = File::open(cert_path).context("Failed to open certificate file")?;
    let mut reader = BufReader::new(cert_file);
    let mut certs = Vec::new();
    while let Ok(Some(item)) = rustls_pemfile::read_one(&mut reader) {
        if let rustls_pemfile::Item::X509Certificate(cert) = item {
            certs.push(cert.into_owned()); // Convert to CertificateDer<'static>
        } else {
            // Log and ignore other items, or handle them appropriately
            // use tracing::warn if you're using the tracing lib
            warn!("Certificate is not a valid PEM encoded X509 certificate");
        }
    }

    if certs.is_empty() {
        anyhow::bail!("No certificates found in {}", cert_path.display());
    }

    // Load private key
    let key_file = File::open(key_path).context("Failed to open private key file")?;
    let mut key_reader = BufReader::new(key_file);
    let key = match rustls_pemfile::read_one(&mut key_reader).context("Failed to read private key")? {
        None => anyhow::bail!("No private key found in {}", key_path.display()),
        Some(Item::Pkcs8Key(key)) => PrivateKeyDer::from(key.into()), // Convert to PrivateKeyDer
        Some(_) => anyhow::bail!("Found an unexpected item in the key file. Expected a PKCS#8, Sec1, or RSA key."),
    };

    Ok((certs, key))
}
