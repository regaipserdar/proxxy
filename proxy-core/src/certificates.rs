//! Certificate management functionality

use crate::{ProxyError, Result};
use rcgen::{Certificate, CertificateParams, DistinguishedName};
use std::collections::HashMap;

/// Certificate manager for handling SSL/TLS certificates
pub struct CertificateManager {
    root_cert: Certificate,
    generated_certs: HashMap<String, Certificate>,
}

impl CertificateManager {
    /// Create a new certificate manager
    pub fn new() -> Result<Self> {
        let mut params = CertificateParams::new(vec!["localhost".to_string()]);
        params.distinguished_name = DistinguishedName::new();

        let root_cert = Certificate::from_params(params).map_err(|e| {
            ProxyError::Certificate(format!("Failed to create root certificate: {}", e))
        })?;

        Ok(Self {
            root_cert,
            generated_certs: HashMap::new(),
        })
    }

    /// Generate a certificate for the given domain
    pub fn generate_cert_for_domain(&mut self, domain: &str) -> Result<&Certificate> {
        if !self.generated_certs.contains_key(domain) {
            let mut params = CertificateParams::new(vec![domain.to_string()]);
            params.distinguished_name = DistinguishedName::new();

            let cert = Certificate::from_params(params).map_err(|e| {
                ProxyError::Certificate(format!(
                    "Failed to create certificate for {}: {}",
                    domain, e
                ))
            })?;

            self.generated_certs.insert(domain.to_string(), cert);
        }

        Ok(self.generated_certs.get(domain).unwrap())
    }

    /// Get the root certificate
    pub fn root_certificate(&self) -> &Certificate {
        &self.root_cert
    }
}
