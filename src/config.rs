use std::sync::Arc;
use std::sync::OnceLock;

use rustls::ClientConfig;
use rustls::RootCertStore;

static TLS_CONFIG: OnceLock<Arc<ClientConfig>> = OnceLock::new();
pub struct Config;
impl Config {
    pub(crate) fn tls_settings() -> Arc<ClientConfig> {
        TLS_CONFIG
            .get_or_init(|| {
                let root_store = RootCertStore {
                    roots: webpki_roots::TLS_SERVER_ROOTS.into(),
                };
                let mut config = rustls::ClientConfig::builder()
                    .with_root_certificates(root_store)
                    .with_no_client_auth();
                config.key_log = Arc::new(rustls::KeyLogFile::new());
                Arc::new(config)
            })
            .clone()
    }
}
