// Inspired by
// https://github.com/tokio-rs/tls/blob/master/tokio-rustls/examples/server/src/main.rs
// and
// https://github.com/tokio-rs/axum/blob/3b579c721504d4d64de74b414f39c3dfb33b923a/examples/tls_rustls.rs

use axum::routing::Router;
use std::fs::File;
use std::io::{self, BufReader};
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_rustls::rustls::{self, Certificate, PrivateKey};
use tokio_rustls::TlsAcceptor;

fn load_certs(path: &Path) -> io::Result<Vec<Certificate>> {
 rustls_pemfile::certs(&mut BufReader::new(File::open(path)?))
  .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid cert"))
  .map(|mut certs| certs.drain(..).map(Certificate).collect())
}

fn load_keys(path: &Path) -> io::Result<Vec<PrivateKey>> {
 rustls_pemfile::pkcs8_private_keys(&mut BufReader::new(File::open(path)?))
  .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid key"))
  .map(|mut keys| keys.drain(..).map(PrivateKey).collect())
}

pub async fn serve(
 addr: &SocketAddr,
 key_path: &Path,
 cert_path: &Path,
 app: Router,
) -> Result<(), Box<dyn std::error::Error>> {
 let certs = load_certs(cert_path)?;
 let mut keys = load_keys(key_path)?;

 let mut config = rustls::ServerConfig::builder()
  .with_safe_defaults()
  .with_no_client_auth()
  .with_single_cert(certs, keys.remove(0))
  .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;

 config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

 let acceptor = TlsAcceptor::from(Arc::new(config));
 let listener = TcpListener::bind(addr).await?;

 loop {
  let (stream, _peer_addr) = listener.accept().await?;
  let acceptor = acceptor.clone();
  let app = app.clone();

  tokio::spawn(async move {
   if let Ok(stream) = acceptor.accept(stream).await {
    hyper::server::conn::Http::new().serve_connection(stream, app).with_upgrades().await.ok();
   }
  });
 }
}
