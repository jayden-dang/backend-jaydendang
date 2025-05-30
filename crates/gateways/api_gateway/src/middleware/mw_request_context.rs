use axum::{
  extract::{ConnectInfo, Request},
  http::HeaderMap,
  middleware::Next,
  response::Response,
};
use std::net::SocketAddr;
use tracing::{info, instrument};

use crate::error::RequestContext;

/// Middleware to extract and setup RequestContext for the entire request lifecycle
#[instrument(skip(req, next), fields(request_id, trace_id))]
pub async fn mw_request_context(
  ConnectInfo(addr): ConnectInfo<SocketAddr>,
  mut req: Request,
  next: Next,
) -> Response {
  // Extract client IP
  let client_ip = extract_client_ip(req.headers(), addr);

  // Create RequestContext from headers and client info
  let mut context = RequestContext::from_headers(req.headers());
  context = context.with_client_ip(client_ip);

  // Set tracing fields
  let request_id = context.request_id.clone().unwrap_or_default();
  let trace_id = context.trace_id.clone().unwrap_or_default();

  tracing::Span::current()
    .record("request_id", &request_id)
    .record("trace_id", &trace_id);

  info!(
      request_id = %request_id,
      trace_id = %trace_id,
      client_ip = %context.client_ip.as_deref().unwrap_or("unknown"),
      user_agent = %context.user_agent.as_deref().unwrap_or("unknown"),
      method = %req.method(),
      path = %req.uri().path(),
      "Request started"
  );

  // Store context in request extensions for other middleware to access
  req.extensions_mut().insert(context.clone());

  // Run the rest of the request with the context in task-local storage
  context.run_with_context(next.run(req)).await
}

/// Extract client IP from various headers with fallback to socket address
fn extract_client_ip(headers: &HeaderMap, socket_addr: SocketAddr) -> String {
  // Try various headers in order of preference
  let possible_headers = [
    "x-forwarded-for",     // Most common proxy header
    "x-real-ip",           // Nginx
    "cf-connecting-ip",    // Cloudflare
    "x-cluster-client-ip", // GCP Load Balancer
    "x-forwarded",         // General forwarded header
    "forwarded-for",       // Alternative
    "forwarded",           // RFC 7239
  ];

  for header_name in possible_headers {
    if let Some(header_value) = headers.get(header_name) {
      if let Ok(header_str) = header_value.to_str() {
        // Handle comma-separated IPs (take the first one)
        if let Some(first_ip) = header_str.split(',').next() {
          let trimmed_ip = first_ip.trim();
          if !trimmed_ip.is_empty() && trimmed_ip != "unknown" {
            return trimmed_ip.to_string();
          }
        }
      }
    }
  }

  // Fallback to socket address
  socket_addr.ip().to_string()
}

/// Helper to get RequestContext from request extensions
pub fn get_request_context_from_extensions(req: &Request) -> Option<RequestContext> {
  req.extensions().get::<RequestContext>().cloned()
}

#[cfg(test)]
mod tests {
  use super::*;
  use axum::http::{HeaderMap, HeaderValue};
  use std::net::{IpAddr, Ipv4Addr, SocketAddr};

  #[test]
  fn test_extract_client_ip_from_headers() {
    let mut headers = HeaderMap::new();
    let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 8080);

    // Test x-forwarded-for header
    headers.insert("x-forwarded-for", HeaderValue::from_static("203.0.113.195"));
    assert_eq!(extract_client_ip(&headers, socket_addr), "203.0.113.195");

    // Test comma-separated IPs (should take first)
    headers.insert(
      "x-forwarded-for",
      HeaderValue::from_static("203.0.113.195, 70.41.3.18, 150.172.238.178"),
    );
    assert_eq!(extract_client_ip(&headers, socket_addr), "203.0.113.195");

    // Test fallback to socket address
    headers.clear();
    assert_eq!(extract_client_ip(&headers, socket_addr), "192.168.1.1");
  }

  #[test]
  fn test_request_context_from_headers() {
    let mut headers = HeaderMap::new();
    headers.insert("x-request-id", HeaderValue::from_static("test-request-123"));
    headers.insert("x-trace-id", HeaderValue::from_static("test-trace-456"));
    headers.insert("user-agent", HeaderValue::from_static("test-agent/1.0"));

    let context = RequestContext::from_headers(&headers);

    assert_eq!(context.request_id, Some("test-request-123".to_string()));
    assert_eq!(context.trace_id, Some("test-trace-456".to_string()));
    assert_eq!(context.user_agent, Some("test-agent/1.0".to_string()));
    assert!(context.user_id.is_none());
    assert!(context.client_ip.is_none());
  }
}
