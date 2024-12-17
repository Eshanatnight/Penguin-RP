use anyhow::anyhow;
use async_trait::async_trait;
use pingora::server::Server;
use pingora_core::{prelude::HttpPeer, Result};
use pingora_proxy::{http_proxy_service, ProxyHttp, Session};

struct ReverseProxy;

#[async_trait]
impl ProxyHttp for ReverseProxy {
    type CTX = ();

    fn new_ctx(&self) -> Self::CTX {}

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        Ok(Box::from(HttpPeer::new(
            String::from("192.168.100.26:8080"),
            false,
            "".to_string(),
        )))
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let mut server = Server::new(None).map_err(|err| anyhow!(err))?;
    let mut proxy = http_proxy_service(&server.configuration, ReverseProxy);

    proxy.add_tcp("0.0.0.0:6188");
    server.add_service(proxy);
    server.run_forever();
}
