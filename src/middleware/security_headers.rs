use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::header::HeaderValue;
use actix_web::Error;
use std::future::{ready, Ready};
use std::pin::Pin;

pub struct SecurityHeaders;

impl<S, B> Transform<S, ServiceRequest> for SecurityHeaders
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SecurityHeadersMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SecurityHeadersMiddleware { service }))
    }
}

pub struct SecurityHeadersMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for SecurityHeadersMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;

            let headers = res.headers_mut();
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-content-type-options"),
                HeaderValue::from_static("nosniff"),
            );
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-frame-options"),
                HeaderValue::from_static("DENY"),
            );
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-xss-protection"),
                HeaderValue::from_static("1; mode=block"),
            );
            headers.insert(
                actix_web::http::header::HeaderName::from_static("strict-transport-security"),
                HeaderValue::from_static("max-age=31536000; includeSubDomains"),
            );
            headers.insert(
                actix_web::http::header::HeaderName::from_static("referrer-policy"),
                HeaderValue::from_static("strict-origin-when-cross-origin"),
            );
            headers.insert(
                actix_web::http::header::HeaderName::from_static(
                    "content-security-policy",
                ),
                HeaderValue::from_static(
                    "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self' data:; connect-src 'self'; frame-ancestors 'none';",
                ),
            );
            headers.insert(
                actix_web::http::header::HeaderName::from_static("permissions-policy"),
                HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
            );

            Ok(res)
        })
    }
}