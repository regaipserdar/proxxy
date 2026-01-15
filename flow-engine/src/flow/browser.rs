//! Browser Management Module
//!
//! Handles Chromium browser lifecycle, configuration, and proxy integration.

use crate::error::{FlowEngineError, FlowResult};
use chromiumoxide::{Browser, BrowserConfig};
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use std::path::PathBuf;
use uuid::Uuid;

/// Proxy configuration for browser
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    /// Proxy host (e.g., "127.0.0.1")
    pub host: String,
    /// Proxy port (e.g., 8080)
    pub port: u16,
    /// Optional username for proxy auth
    pub username: Option<String>,
    /// Optional password for proxy auth
    pub password: Option<String>,
}

impl ProxyConfig {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            username: None,
            password: None,
        }
    }

    pub fn with_auth(mut self, username: impl Into<String>, password: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self.password = Some(password.into());
        self
    }

    /// Convert to proxy URL format
    pub fn to_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}

/// Browser launch options
#[derive(Debug, Clone)]
pub struct BrowserOptions {
    /// Run in headless mode (no visible UI)
    pub headless: bool,
    /// Proxy configuration
    pub proxy: Option<ProxyConfig>,
    /// Additional Chrome arguments
    pub extra_args: Vec<String>,
    /// User data directory (for persistent sessions)
    pub user_data_dir: Option<String>,
    /// Ignore SSL certificate errors (useful when proxying)
    pub ignore_ssl_errors: bool,
    /// Window size
    pub window_size: Option<(u32, u32)>,
    /// CA certificate path for SSL trust (Proxxy CA cert)
    pub ca_cert_path: Option<String>,
}

impl Default for BrowserOptions {
    fn default() -> Self {
        Self {
            headless: true,
            proxy: None,
            extra_args: Vec::new(),
            user_data_dir: None,
            ignore_ssl_errors: true,
            window_size: Some((1920, 1080)),
            ca_cert_path: None,
        }
    }
}

impl BrowserOptions {
    /// Create options for headed (visible) browser
    pub fn headed() -> Self {
        Self {
            headless: false,
            ..Default::default()
        }
    }

    /// Create options with proxy
    pub fn with_proxy(mut self, proxy: ProxyConfig) -> Self {
        self.proxy = Some(proxy);
        self
    }

    /// Set headless mode
    pub fn headless(mut self, headless: bool) -> Self {
        self.headless = headless;
        self
    }
}

/// Managed browser instance
pub struct ManagedBrowser {
    browser: Browser,
    options: BrowserOptions,
    user_data_dir: Option<PathBuf>,
}

impl ManagedBrowser {
    /// Get the underlying browser
    pub fn browser(&self) -> &Browser {
        &self.browser
    }

    /// Check if browser is still connected
    pub async fn is_connected(&self) -> bool {
        // Chromiumoxide doesn't expose a direct method, but we can try a simple operation
        self.browser.new_page("about:blank").await.is_ok()
    }

    /// Close the browser
    pub async fn close(self) -> FlowResult<()> {
        // Browser will be closed when dropped
        drop(self.browser);
        info!("Browser closed");

        // Cleanup user data dir if we created one
        if let Some(path) = self.user_data_dir {
            if path.exists() {
                 info!("Cleaning up browser profile: {:?}", path);
                 if let Err(e) = std::fs::remove_dir_all(&path) {
                     warn!("Failed to remove browser profile dir: {:?}", e);
                 }
            }
        }
        Ok(())
    }
}

/// Browser launcher and manager
pub struct BrowserManager {
    active_browser: Arc<RwLock<Option<ManagedBrowser>>>,
}

impl BrowserManager {
    pub fn new() -> Self {
        Self {
            active_browser: Arc::new(RwLock::new(None)),
        }
    }

    /// Launch a new browser instance
    pub async fn launch(&self, options: BrowserOptions) -> FlowResult<Arc<RwLock<Option<ManagedBrowser>>>> {
        // Close existing browser if any
        self.close().await?;

        // Build browser configuration
        let mut config_builder = BrowserConfig::builder();
        
        // Use a unique user data directory to avoid SingletonLock errors
        let user_data_dir = std::env::temp_dir().join(format!("proxxy_browser_{}", Uuid::new_v4()));
        config_builder = config_builder.user_data_dir(&user_data_dir);

        // Set headed mode (with_head makes browser visible)
        // Note: chromiumoxide defaults to headless, with_head() makes it visible
        if !options.headless {
            config_builder = config_builder.with_head();
        }

        // Add proxy if configured
        if let Some(ref proxy) = options.proxy {
            config_builder = config_builder.arg(format!("--proxy-server={}", proxy.to_url()));
        }

        // Ignore SSL errors (important for MITM proxy)
        if options.ignore_ssl_errors {
            config_builder = config_builder.arg("--ignore-certificate-errors");
            config_builder = config_builder.arg("--ignore-ssl-errors");
        }

        // Set window size
        if let Some((width, height)) = options.window_size {
            config_builder = config_builder.arg(format!("--window-size={},{}", width, height));
        }

        // Add custom arguments
        for arg in &options.extra_args {
            config_builder = config_builder.arg(arg);
        }

        // Standard args for automation
        config_builder = config_builder
            .arg("--disable-blink-features=AutomationControlled")
            .arg("--disable-infobars")
            .arg("--no-first-run")
            .arg("--no-default-browser-check");

        // Build config
        let config = config_builder
            .build()
            .map_err(|e| FlowEngineError::BrowserLaunch(e.to_string()))?;

        // Launch browser
        let (browser, mut handler) = Browser::launch(config)
            .await
            .map_err(|e| FlowEngineError::BrowserLaunch(format!("Failed to launch browser: {}", e)))?;

        // Spawn handler task
        tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if let Err(e) = event {
                    warn!("Browser event error: {:?}", e);
                }
            }
        });

        info!(
            "Browser launched (headless: {}, proxy: {:?})",
            options.headless,
            options.proxy.as_ref().map(|p| p.to_url())
        );

        // Store managed browser
        let managed = ManagedBrowser {
            browser,
            options,
            user_data_dir: Some(user_data_dir),
        };

        let mut guard = self.active_browser.write().await;
        *guard = Some(managed);

        Ok(self.active_browser.clone())
    }

    /// Get the active browser
    pub async fn get_browser(&self) -> Option<Arc<RwLock<Option<ManagedBrowser>>>> {
        let guard = self.active_browser.read().await;
        if guard.is_some() {
            Some(self.active_browser.clone())
        } else {
            None
        }
    }

    /// Close the active browser
    pub async fn close(&self) -> FlowResult<()> {
        let mut guard = self.active_browser.write().await;
        if let Some(browser) = guard.take() {
            browser.close().await?;
        }
        Ok(())
    }
}

impl Default for BrowserManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_config() {
        let proxy = ProxyConfig::new("127.0.0.1", 8080);
        assert_eq!(proxy.to_url(), "http://127.0.0.1:8080");

        let proxy_with_auth = ProxyConfig::new("localhost", 3128)
            .with_auth("user", "pass");
        assert_eq!(proxy_with_auth.username, Some("user".to_string()));
    }

    #[test]
    fn test_browser_options() {
        let opts = BrowserOptions::default();
        assert!(opts.headless);
        assert!(opts.ignore_ssl_errors);

        let headed = BrowserOptions::headed();
        assert!(!headed.headless);
    }
}
