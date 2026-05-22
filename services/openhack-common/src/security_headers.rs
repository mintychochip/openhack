use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use std::future::{ready, Future, Ready};
use std::pin::Pin;
use std::rc::Rc;

/// Security headers middleware.
///
/// Adds security-related HTTP response headers to all outgoing responses.
///
/// # Expected Behavior
///
/// Wraps every response and adds the following headers:
/// - `X-Content-Type-Options: nosniff` - Prevents MIME type sniffing
/// - `X-Frame-Options: DENY` - Prevents clickjacking by disallowing framing
/// - `X-XSS-Protection: 0` - Disables legacy XSS filter (modern browsers handle this)
/// - `Strict-Transport-Security: max-age=63072000; includeSubDomains; preload` - Forces HTTPS
/// - `Referrer-Policy: strict-origin-when-cross-origin` - Controls referrer leakage
/// - `Content-Security-Policy: default-src 'self'` - Restricts resource loading
/// - `Permissions-Policy: camera=(), microphone=(), geolocation=()` - Disables sensitive APIs
///
/// # Side Effects
///
/// - Modifies response headers for every HTTP response
/// - No I/O, database, or network operations
#[derive(Default)]
pub struct SecurityHeadersMiddleware;

impl SecurityHeadersMiddleware {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl<S, B> Transform<S, ServiceRequest> for SecurityHeadersMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SecurityHeadersMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SecurityHeadersMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct SecurityHeadersMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for SecurityHeadersMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        Box::pin(async move {
            let mut res = svc.call(req).await?;

            res.headers_mut().insert(
                actix_web::http::header::X_CONTENT_TYPE_OPTIONS,
                "nosniff".parse().unwrap(),
            );

            res.headers_mut().insert(
                actix_web::http::header::X_FRAME_OPTIONS,
                "DENY".parse().unwrap(),
            );

            res.headers_mut().insert(
                actix_web::http::header::X_XSS_PROTECTION,
                "0".parse().unwrap(),
            );

            res.headers_mut().insert(
                actix_web::http::header::STRICT_TRANSPORT_SECURITY,
                "max-age=63072000; includeSubDomains; preload".parse().unwrap(),
            );

            res.headers_mut().insert(
                actix_web::http::header::REFERRER_POLICY,
                "strict-origin-when-cross-origin".parse().unwrap(),
            );

            res.headers_mut().insert(
                actix_web::http::header::CONTENT_SECURITY_POLICY,
                "default-src 'self'".parse().unwrap(),
            );

            res.headers_mut().insert(
                actix_web::http::header::PERMISSIONS_POLICY,
                "camera=(), microphone=(), geolocation=()".parse().unwrap(),
            );

            Ok(res)
        })
    }
}
