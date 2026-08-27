//! Loopback reverse proxy: route `route.test` HTTP(S) hosts to project
//! internal ports with streaming passthrough, SSE/WebSocket support, and
//! typed responses for non-running or unknown upstreams.

use std::{net::SocketAddr, sync::Arc, time::Duration};

use http::{Request, Response, StatusCode, header};
use http_body_util::{BodyExt, Full, combinators::BoxBody};
use hyper::{
    body::{Bytes, Incoming},
    server::conn::http1,
    service::service_fn,
};
use hyper_util::rt::{TokioIo, TokioTimer};
use tokio::net::{TcpListener, TcpStream};

use crate::{control::ControlManager, lifecycle::LifecycleState, paths::AppPaths, tls};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const RETRY_AFTER_SECONDS: u64 = 2;

pub type Body = BoxBody<Bytes, hyper::Error>;

#[derive(Clone)]
pub struct ProxyContext {
    pub control: Arc<ControlManager>,
    pub paths: AppPaths,
}

fn text_body(text: &str) -> Body {
    Full::new(Bytes::copy_from_slice(text.as_bytes()))
        .map_err(|never| match never {})
        .boxed()
}

fn error_response(status: StatusCode, reason: &str, retry_after: Option<u64>) -> Response<Body> {
    let mut response = Response::new(text_body(reason));
    *response.status_mut() = status;
    if let Some(seconds) = retry_after {
        response
            .headers_mut()
            .insert(header::RETRY_AFTER, seconds.to_string().parse().unwrap());
    }
    response
}

/// Normalize a Host header into a lowercase `.test` route name.
fn route_from_host(host: Option<&str>) -> Option<String> {
    let host = host?;
    let host = host
        .split(':')
        .next()?
        .trim()
        .trim_end_matches('.')
        .to_ascii_lowercase();
    if !host.ends_with(".test") {
        return None;
    }
    Some(host.trim_end_matches(".test").to_owned())
}

/// Look up a live upstream for a route name: returns (project_id, port).
async fn resolve_upstream(control: &ControlManager, route: &str) -> Option<(uuid::Uuid, u16)> {
    let projects = control.list_registered().await;
    let project = projects
        .into_iter()
        .find(|p| p.route.as_deref() == Some(route))?;
    let snapshot = control.snapshot(project.project_id)?;
    Some((project.project_id, snapshot.port?))
}

fn header_value(value: &str) -> http::HeaderValue {
    http::HeaderValue::from_str(value).unwrap_or_else(|_| http::HeaderValue::from_static("unknown"))
}

async fn proxy_http(
    ctx: &ProxyContext,
    req: Request<Incoming>,
    proto: &str,
) -> Result<Response<Body>, std::io::Error> {
    // Reject proxy loops: never forward to a `.test` upstream or to Nerd's own
    // listener ports.
    if is_loop_attempt(&req) {
        return Ok(error_response(
            StatusCode::BAD_GATEWAY,
            "proxy loop rejected",
            None,
        ));
    }

    let host = req
        .headers()
        .get(header::HOST)
        .and_then(|value| value.to_str().ok());
    let Some(route) = route_from_host(host) else {
        return Ok(error_response(StatusCode::NOT_FOUND, "unknown host", None));
    };

    let Some((project_id, upstream_port)) = resolve_upstream(&ctx.control, &route).await else {
        if control_knows_route(&ctx.control, &route).await {
            return Ok(apply_upstream_state(&ctx.control, &route).await);
        }
        return Ok(error_response(StatusCode::NOT_FOUND, "unknown host", None));
    };
    let _ = project_id;

    let upstream_addr: SocketAddr = ([127, 0, 0, 1], upstream_port).into();
    let upstream = tokio::time::timeout(CONNECT_TIMEOUT, TcpStream::connect(upstream_addr))
        .await
        .ok()
        .and_then(|result| result.ok())
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "upstream"))?;

    // Capture the incoming request parts, then rebuild an upstream request
    // carrying our X-Forwarded-* policy.
    let method = req.method().clone();
    let uri = req.uri().clone();
    let host_value = host.map(|h| h.to_owned());
    let headers: Vec<(http::HeaderName, http::HeaderValue)> = req
        .headers()
        .iter()
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect();
    let body = req.into_body().boxed();

    let mut forwarded_req = http::Request::builder()
        .method(method)
        .uri(uri)
        .body(body)
        .map_err(|error| std::io::Error::other(error.to_string()))?;
    for (name, value) in headers {
        forwarded_req.headers_mut().insert(name, value);
    }
    set_forwarded(
        &mut forwarded_req,
        host_value.as_deref().unwrap_or(""),
        proto,
    );

    let (mut sender, conn) = hyper::client::conn::http1::handshake(TokioIo::new(upstream))
        .await
        .map_err(|error| std::io::Error::other(error.to_string()))?;
    tokio::spawn(async move {
        let _ = conn.with_upgrades().await;
    });

    let upstream_response = sender
        .send_request(forwarded_req)
        .await
        .map_err(|error| std::io::Error::other(error.to_string()))?;

    let (parts, upstream_body) = upstream_response.into_parts();
    let mut response = Response::new(upstream_body.boxed());
    *response.status_mut() = parts.status;
    for (name, value) in parts.headers {
        if let Some(name) = name {
            response.headers_mut().insert(name, value);
        }
    }
    Ok(response)
}

fn is_loop_attempt<B>(req: &Request<B>) -> bool {
    req.uri().host().is_some_and(|h| h.ends_with(".test"))
        || req
            .uri()
            .port_u16()
            .is_some_and(|p| p == 80 || p == 443 || p == 53)
}

async fn control_knows_route(control: &ControlManager, route: &str) -> bool {
    control
        .list_registered()
        .await
        .iter()
        .any(|p| p.route.as_deref() == Some(route))
}

async fn apply_upstream_state(control: &ControlManager, route: &str) -> Response<Body> {
    let projects = control.list_registered().await;
    let project = projects
        .into_iter()
        .find(|p| p.route.as_deref() == Some(route));
    let Some(project) = project else {
        return error_response(StatusCode::NOT_FOUND, "unknown host", None);
    };
    match control.snapshot(project.project_id) {
        Some(snapshot) => match snapshot.state {
            LifecycleState::Running => {
                error_response(StatusCode::BAD_GATEWAY, "upstream failed", None)
            }
            LifecycleState::StartingApp | LifecycleState::WaitingReady => error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                "project is starting",
                Some(RETRY_AFTER_SECONDS),
            ),
            _ => error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                "project is stopped; start it from nerd project start",
                None,
            ),
        },
        None => error_response(StatusCode::NOT_FOUND, "unknown host", None),
    }
}

fn set_forwarded<B>(req: &mut Request<B>, host: &str, proto: &str) {
    let host_value = header_value(host);
    req.headers_mut().insert(header::HOST, host_value.clone());
    req.headers_mut().insert("x-forwarded-host", host_value);
    req.headers_mut()
        .insert("x-forwarded-proto", header_value(proto));
    req.headers_mut()
        .insert("x-forwarded-for", header_value("127.0.0.1"));
}

pub async fn serve(
    http_addr: SocketAddr,
    https_addr: SocketAddr,
    ctx: Arc<ProxyContext>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let http_listener = TcpListener::bind(http_addr).await?;
    let https_listener = TcpListener::bind(https_addr).await?;
    let ca_key = tls::load_ca_key(&ctx.paths)?;

    let http_task = tokio::spawn(serve_http(http_listener, Arc::clone(&ctx)));
    let https_task = tokio::spawn(serve_https(https_listener, ctx, ca_key));
    let _ = tokio::try_join!(http_task, https_task)?;
    Ok(())
}

async fn serve_http(listener: TcpListener, ctx: Arc<ProxyContext>) -> Result<(), std::io::Error> {
    loop {
        let (stream, _) = listener.accept().await?;
        let ctx = Arc::clone(&ctx);
        tokio::spawn(async move {
            let io = TokioIo::new(stream);
            let service = service_fn(move |req| {
                let ctx = ctx.clone();
                async move { proxy_http(&ctx, req, "http").await }
            });
            let _ = http1::Builder::new()
                .timer(TokioTimer::new())
                .serve_connection(io, service)
                .with_upgrades()
                .await;
        });
    }
}

async fn serve_https(
    listener: TcpListener,
    ctx: Arc<ProxyContext>,
    ca_key: Arc<str>,
) -> Result<(), std::io::Error> {
    // Build the SNI resolver once; it lazily issues and caches a leaf config
    // per requested hostname so every `name.test` certificate matches.
    let acceptor = match tls::RouteCertResolver::new(ca_key).build_acceptor() {
        Ok(acceptor) => acceptor,
        Err(error) => {
            tracing::warn!(error = %error, "https proxy: cannot build TLS resolver");
            return Ok(());
        }
    };
    loop {
        let (stream, _) = listener.accept().await?;
        let ctx = Arc::clone(&ctx);
        let acceptor = acceptor.clone();
        tokio::spawn(async move {
            let tls_stream = match acceptor.accept(stream).await {
                Ok(stream) => stream,
                Err(_) => return,
            };
            let io = TokioIo::new(tls_stream);
            let service = service_fn(move |req| {
                let ctx = ctx.clone();
                async move { proxy_http(&ctx, req, "https").await }
            });
            let _ = http1::Builder::new()
                .timer(TokioTimer::new())
                .serve_connection(io, service)
                .with_upgrades()
                .await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{is_loop_attempt, route_from_host};

    #[test]
    fn normalizes_hosts() {
        assert_eq!(route_from_host(Some("Foo.Test")), Some("foo".to_owned()));
        assert_eq!(
            route_from_host(Some("foo.test:3000")),
            Some("foo".to_owned())
        );
        assert_eq!(route_from_host(Some("foo.test.")), Some("foo".to_owned()));
        assert_eq!(route_from_host(Some("example.com")), None);
        assert_eq!(route_from_host(None), None);
    }

    #[test]
    fn rejects_loop_attempts() {
        let loop_req = http::Request::builder()
            .uri("http://foo.test/x")
            .body(http_body_util::Empty::<hyper::body::Bytes>::new())
            .expect("build loop request");
        assert!(is_loop_attempt(&loop_req));

        let internal = http::Request::builder()
            .uri("http://127.0.0.1:1234/x")
            .body(http_body_util::Empty::<hyper::body::Bytes>::new())
            .expect("build normal request");
        assert!(!is_loop_attempt(&internal));
    }
}
