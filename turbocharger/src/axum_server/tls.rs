// Inspired by
// https://github.com/tokio-rs/tls/blob/master/tokio-rustls/examples/server/src/main.rs
// and
// https://github.com/tokio-rs/axum/blob/3b579c721504d4d64de74b414f39c3dfb33b923a/examples/tls_rustls.rs

use std::io::BufReader;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_rustls::rustls::{self, Certificate, PrivateKey};
use tokio_rustls::TlsAcceptor;
use tracked::tracked;
use turbosql::{select, Turbosql};

#[allow(non_camel_case_types)]
#[derive(Turbosql, Default, Clone)]
struct _turbocharger_tls_cert {
 rowid: Option<i64>,
 server_name: Option<String>,
 issue_time: Option<i64>,
 cert: Option<String>,
 key: Option<String>,
}

impl _turbocharger_tls_cert {
 #[tracked]
 fn parsed_cert(&self) -> Result<Vec<Certificate>, tracked::StringError> {
  Ok(
   rustls_pemfile::certs(&mut BufReader::new(self.cert.as_ref()?.as_bytes()))
    .map(|mut certs| certs.drain(..).map(Certificate).collect())?,
  )
 }
 #[tracked]
 fn parsed_key(&self) -> Result<Vec<PrivateKey>, tracked::StringError> {
  Ok(
   rustls_pemfile::pkcs8_private_keys(&mut BufReader::new(
    tracked::Track::t(self.key.as_ref())?.as_bytes(),
   ))
   .map(|mut keys| keys.drain(..).map(PrivateKey).collect())?,
  )
 }
}

#[tracked]
pub async fn serve(addr: &SocketAddr, app: axum::routing::Router) -> tracked::Result<()> {
 let mut config = rustls::ServerConfig::builder()
  .with_safe_defaults()
  .with_no_client_auth()
  .with_cert_resolver(Arc::new(Resolver(tokio::runtime::Handle::current())));

 config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

 let acceptor = TlsAcceptor::from(Arc::new(config));
 let listener = TcpListener::bind(addr).await?;

 loop {
  let (stream, peer_addr) = listener.accept().await?;
  let acceptor = acceptor.clone();
  let app = app.clone().layer(axum::extract::Extension(axum::extract::ConnectInfo(peer_addr)));

  tokio::task::spawn_blocking(move || {
   tokio::runtime::Handle::current().block_on(async move {
    if let Ok(stream) = acceptor.accept(stream).await {
     hyper::server::conn::Http::new().serve_connection(stream, app).with_upgrades().await.ok();
    }
   })
  });
 }
}

struct Resolver(tokio::runtime::Handle);

impl rustls::server::ResolvesServerCert for Resolver {
 fn resolve(
  &self,
  client_hello: rustls::server::ClientHello<'_>,
 ) -> Option<Arc<rustls::sign::CertifiedKey>> {
  match resolve_cert(&self.0, &client_hello.server_name()?.to_ascii_lowercase()) {
   Ok(cert) => Some(Arc::new(cert)),
   Err(e) => {
    log::error!("{}", e);
    None
   }
  }
 }
}

#[tracked]
fn resolve_cert(
 handle: &tokio::runtime::Handle,
 server_name: &str,
) -> Result<rustls::sign::CertifiedKey, tracked::StringError> {
 if select!(Option<_turbocharger_tls_cert> "WHERE server_name = ?", server_name)?.is_none() {
  let cert = request_cert(handle, server_name)?;
  _turbocharger_tls_cert {
   rowid: None,
   server_name: Some(server_name.into()),
   issue_time: Some(
    std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH)?.as_secs()
     as i64,
   ),
   cert: Some(cert.certificate().to_string()),
   key: Some(cert.private_key().to_string()),
  }
  .insert()?;
 }
 let cert = select!(_turbocharger_tls_cert "WHERE server_name = ?", server_name)?;

 Ok(rustls::sign::CertifiedKey::new(
  cert.parsed_cert()?,
  rustls::sign::any_supported_type(cert.parsed_key()?.first()?)?,
 ))
}

#[tracked]
fn request_cert(
 handle: &tokio::runtime::Handle,
 server_name: &str,
) -> Result<acme_lib::Certificate, tracked::StringError> {
 log::warn!("requesting new TLS cert for {}", server_name);

 let url = acme_lib::DirectoryUrl::LetsEncrypt;
 let persist = acme_lib::persist::MemoryPersist::new();
 let dir = acme_lib::Directory::from_url(persist, url)?;
 let acc = dir.account("trevyn-git@protonmail.com")?;
 let mut ord_new = acc.new_order(server_name, &[])?;

 log::info!("proving domain ownership");

 let ord_csr = loop {
  if let Some(ord_csr) = ord_new.confirm_validations() {
   break ord_csr;
  }

  let auths = ord_new.authorizations()?;
  let chall = auths[0].http_challenge();
  let token = chall.http_token();
  let path = format!("/.well-known/acme-challenge/{}", token);
  let proof = chall.http_proof();

  let app =
   axum::routing::Router::new().route(&path, axum::routing::get(move || acme_handler(proof)));
  async fn acme_handler(proof: String) -> impl axum::response::IntoResponse {
   log::info!("served proof");
   proof
  }
  let server = handle.spawn(async move {
   log::info!("proof server spawned, path = {}", path);
   axum::Server::bind(&SocketAddr::from(([0, 0, 0, 0], 80)))
    .serve(app.into_make_service())
    .await
    .unwrap();
  });

  log::info!("confirming ownership");
  chall.validate(1000)?;
  log::info!("updating state");
  ord_new.refresh()?;
  log::info!("finalizing order");
  server.abort();
 };

 let pkey_pri = acme_lib::create_p384_key();
 let ord_cert = ord_csr.finalize_pkey(pkey_pri, 1000)?;
 log::info!("downloading certificate");
 let cert = ord_cert.download_and_save_cert()?;
 log::info!("certificate downloaded");
 Ok(cert)
}
