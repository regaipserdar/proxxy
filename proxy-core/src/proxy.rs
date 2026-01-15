use crate::{
    admin::{start_admin_server, Metrics},
    ca::CertificateAuthority,
    config::{ProxyConfig, BodyCaptureConfig},
    error::ProxyError,
    handlers::LogHandler,
    Result,
};
use hudsucker::{certificate_authority::RcgenAuthority, rustls, ProxyBuilder};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{error, info};

pub struct ProxyServer {
    config: ProxyConfig,
    ca: CertificateAuthority,
    metrics: Arc<Metrics>,
    log_sender: Option<tokio::sync::mpsc::Sender<crate::pb::TrafficEvent>>,
    body_capture_config: Option<BodyCaptureConfig>,
}

impl ProxyServer {
    pub fn new(config: ProxyConfig, ca: CertificateAuthority) -> Self {
        Self {
            config,
            ca,
            metrics: Arc::new(Metrics::default()),
            log_sender: None,
            body_capture_config: None,
        }
    }

    pub fn with_log_sender(
        mut self,
        sender: tokio::sync::mpsc::Sender<crate::pb::TrafficEvent>,
    ) -> Self {
        self.log_sender = Some(sender);
        self
    }

    pub fn with_body_capture_config(mut self, config: BodyCaptureConfig) -> Self {
        self.body_capture_config = Some(config);
        self
    }

    pub async fn run(self) -> Result<()> {
        let addr = SocketAddr::from(([0, 0, 0, 0], self.config.listen_port));
        info!("Starting proxy server on {}", addr);

        // Start Admin Server
        let admin_port = self.config.admin_port;
        let metrics = self.metrics.clone();
        tokio::spawn(async move {
            if let Err(e) = start_admin_server(admin_port, metrics).await {
                error!("Admin server failed: {}", e);
            }
        });

        // Hudsucker/Rustls expects DER, not PEM.
        let ca_cert_der = self.ca.get_ca_cert_der()?;
        let ca_key_der = self.ca.get_ca_key_der()?;

        let private_key = rustls::PrivateKey(ca_key_der);
        let ca_cert = rustls::Certificate(ca_cert_der);

        let authority = RcgenAuthority::new(private_key, ca_cert, 1000).map_err(|e| {
            ProxyError::Configuration(format!("Failed to create CA authority: {}", e))
        })?;

        // Create LogHandler with body capture config if provided, otherwise use defaults
        let log_handler = match self.body_capture_config {
            Some(body_config) => LogHandler::new(self.metrics.clone(), self.log_sender, body_config),
            None => LogHandler::new_with_defaults(self.metrics.clone(), self.log_sender),
        };

        let proxy = ProxyBuilder::new()
            .with_addr(addr)
            .with_rustls_client()
            .with_ca(authority)
            .with_http_handler(log_handler)
            .build();

        proxy
            .start(std::future::pending::<()>())
            .await
            .map_err(|e| ProxyError::Network(format!("Proxy failed: {}", e)))?;

        Ok(())
    }
}

