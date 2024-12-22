use std::time::Duration;

use anyhow::anyhow;
use async_trait::async_trait;
use pingora::{listeners::TcpSocketOptions, protocols::TcpKeepalive, server::Server};
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
    proxy.threads = Some(4);

    let keep_alive = TcpKeepalive {
        count: 16384, // using an arbitary large number for probe count. might need to be bigger
        idle: Duration::from_secs(60 * 60 * 24), // using 24 hours for now
        interval: Duration::from_millis(5), // do a probe every 5ms
    };

    let mut tcp_settings = TcpSocketOptions::default();
    tcp_settings.ipv6_only = Some(false);
    tcp_settings.tcp_keepalive = Some(keep_alive);

    proxy.add_tcp_with_settings("0.0.0.0:6188", tcp_settings);
    // proxy.add_tcp();
    server.add_service(proxy);
    server.run_forever();
}
