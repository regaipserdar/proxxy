use crate::error::ProxyError;
use crate::Result;
use rcgen::{
    BasicConstraints, Certificate, CertificateParams, DistinguishedName, DnType, IsCa, KeyPair,
    PKCS_ECDSA_P256_SHA256,
};
use std::fs;
use std::path::Path;
use time::{Duration, OffsetDateTime};

/// Certificate Authority for managing MITM certificates.
///
/// Handles persistence of the Root CA certificate and private key, as well as dynamic generation
/// of domain-specific (leaf) certificates signed by the Root CA.
pub struct CertificateAuthority {
    ca_cert: Certificate,
}

impl CertificateAuthority {
    /// Create a new CertificateAuthority found at the given path, or generate a new one.
    pub fn new(ca_dir: &Path) -> Result<Self> {
        let ca_cert_path = ca_dir.join("ca.pem");
        let ca_key_path = ca_dir.join("ca.key");

        if ca_cert_path.exists() && ca_key_path.exists() {
            Self::load(&ca_cert_path, &ca_key_path)
        } else {
            // Create directory if it doesn't exist
            if !ca_dir.exists() {
                fs::create_dir_all(ca_dir).map_err(|e| ProxyError::Io(e))?;
            }
            Self::generate_and_save(&ca_cert_path, &ca_key_path)
        }
    }

    /// Load existing CA certificate and private key
    fn load(cert_path: &Path, key_path: &Path) -> Result<Self> {
        let cert_pem = fs::read_to_string(cert_path).map_err(|e| ProxyError::Io(e))?;
        let key_pem = fs::read_to_string(key_path).map_err(|e| ProxyError::Io(e))?;

        Self::from_pem(&cert_pem, &key_pem)
    }

    /// Create a CertificateAuthority from PEM strings (cert and key).
    pub fn from_pem(_cert_pem: &str, key_pem: &str) -> Result<Self> {
        // Parse the private key from PEM
        let key_pair = KeyPair::from_pem(key_pem)
            .map_err(|e| ProxyError::General(format!("Failed to parse CA key: {}", e)))?;

        // Reconstruct the certificate with the loaded keypair
        // Note: rcgen doesn't support loading existing certs for signing,
        // so we recreate the CA cert with the same parameters
        let mut params = CertificateParams::default();
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, "Proxxy CA");
        dn.push(DnType::OrganizationName, "Proxxy Distributed MITM");
        params.distinguished_name = dn;
        params.is_ca = IsCa::Ca(BasicConstraints::Constrained(0));
        params.key_pair = Some(key_pair);

        let cert = Certificate::from_params(params)
            .map_err(|e| ProxyError::General(format!("Failed to recreate CA cert: {}", e)))?;

        Ok(Self { ca_cert: cert })
    }

    /// Generate a new Root CA and save it to disk
    fn generate_and_save(cert_path: &Path, key_path: &Path) -> Result<Self> {
        let mut params = CertificateParams::default();
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, "Proxxy CA");
        dn.push(DnType::OrganizationName, "Proxxy Distributed MITM");
        params.distinguished_name = dn;
        params.is_ca = IsCa::Ca(BasicConstraints::Constrained(0));
        params.key_usages = vec![
            rcgen::KeyUsagePurpose::KeyCertSign,
            rcgen::KeyUsagePurpose::CrlSign,
        ];

        // Valid for 10 years
        let not_before = OffsetDateTime::now_utc();
        let not_after = not_before + Duration::days(365 * 10);
        params.not_before = not_before;
        params.not_after = not_after;

        // Generate key pair
        let key_pair = KeyPair::generate(&PKCS_ECDSA_P256_SHA256)
            .map_err(|e| ProxyError::General(format!("Failed to generate CA key: {}", e)))?;

        params.key_pair = Some(key_pair);

        let cert = Certificate::from_params(params)
            .map_err(|e| ProxyError::General(format!("Failed to generate CA cert: {}", e)))?;

        let cert_pem = cert
            .serialize_pem()
            .map_err(|e| ProxyError::General(format!("Failed to serialize CA cert: {}", e)))?;
        let key_pem = cert.serialize_private_key_pem();

        fs::write(cert_path, &cert_pem).map_err(|e| ProxyError::Io(e))?;
        fs::write(key_path, &key_pem).map_err(|e| ProxyError::Io(e))?;

        // Export .crt format as requested
        let crt_path = cert_path.with_extension("crt");
        fs::write(crt_path, &cert_pem).map_err(|e| ProxyError::Io(e))?;

        Ok(Self { ca_cert: cert })
    }

    /// Generate a certificate for a specific domain, signed by this CA.
    ///
    /// * `domain` - The domain name (e.g., "example.com") to generate a certificate for.
    ///
    /// Returns a tuple containing `(cert_pem, key_pem)`.
    pub fn gen_cert_for_domain(&self, domain: &str) -> Result<(String, String)> {
        let mut params = CertificateParams::new(vec![domain.to_string()]);

        // Valid for 1 year
        let not_before = OffsetDateTime::now_utc() - Duration::days(1);
        let not_after = not_before + Duration::days(365);
        params.not_before = not_before;
        params.not_after = not_after;

        let cert = Certificate::from_params(params).map_err(|e| {
            ProxyError::General(format!("Failed to generate domain cert params: {}", e))
        })?;

        // Version 0.12+: verify signature.
        let cert_pem = cert
            .serialize_pem_with_signer(&self.ca_cert)
            .map_err(|e| ProxyError::General(format!("Failed to sign domain cert: {}", e)))?;

        let key_pem = cert.serialize_private_key_pem();

        Ok((cert_pem, key_pem))
    }

    /// Get the Root CA certificate in PEM format.
    pub fn get_ca_cert_pem(&self) -> Result<String> {
        self.ca_cert
            .serialize_pem()
            .map_err(|e| ProxyError::General(format!("Failed to serialize CA cert: {}", e)))
    }

    /// Get the Root CA private key in PEM format.
    pub fn get_ca_key_pem(&self) -> Result<String> {
        let mut key_pem = self.ca_cert.serialize_private_key_pem();
        // Ensure proper newline at end
        if !key_pem.ends_with('\n') {
            key_pem.push('\n');
        }
        Ok(key_pem)
    }

    /// Get the Root CA certificate in DER format (for use with rustls/hudsucker).
    pub fn get_ca_cert_der(&self) -> Result<Vec<u8>> {
        self.ca_cert
            .serialize_der()
            .map_err(|e| ProxyError::General(format!("Failed to serialize CA cert DER: {}", e)))
    }

    /// Get the Root CA private key in DER format (for use with rustls/hudsucker).
    pub fn get_ca_key_der(&self) -> Result<Vec<u8>> {
        Ok(self.ca_cert.serialize_private_key_der())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_ca_generation_and_loading() {
        let dir = tempdir().unwrap();
        let ca_dir = dir.path();

        // First creation (generate)
        let ca = CertificateAuthority::new(ca_dir).expect("Failed to create CA");
        assert!(ca_dir.join("ca.pem").exists());
        assert!(ca_dir.join("ca.key").exists());
        assert!(ca_dir.join("ca.crt").exists());

        // Second creation (load)
        let ca2 = CertificateAuthority::new(ca_dir).expect("Failed to load CA");

        // Simple check that they can both generate certs
        let (cert1, _) = ca.gen_cert_for_domain("example.com").unwrap();
        let (cert2, _) = ca2.gen_cert_for_domain("example.com").unwrap();

        assert!(!cert1.is_empty());
        assert!(!cert2.is_empty());
    }

    #[test]
    fn test_domain_certificate_generation() {
        let dir = tempdir().unwrap();
        let ca = CertificateAuthority::new(dir.path()).unwrap();

        let domain = "test.local";
        let (cert_pem, key_pem) = ca.gen_cert_for_domain(domain).unwrap();

        assert!(cert_pem.contains("BEGIN CERTIFICATE"));
        assert!(key_pem.contains("BEGIN PRIVATE KEY"));
    }
}
