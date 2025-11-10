# Chapter 13: Security Enhancements

## Overview

Security is not an afterthought—it's a fundamental requirement for production web applications. A single security vulnerability can lead to data breaches, service disruptions, or compromised user accounts. This chapter implements multiple layers of defense to protect your application against common web vulnerabilities.

You'll implement rate limiting to prevent abuse, HTML sanitization to stop XSS attacks, comprehensive security headers to control browser behavior, and request size limits to prevent resource exhaustion. By the end, you'll understand the OWASP Top 10 vulnerabilities and how Rust and Actix Web help you build secure applications by default.

## Prerequisites

### Completed Chapters
- Chapter 0: Prerequisites and Environment Setup
- Chapter 1: Core Application Setup
- Chapter 3: Error Handling and Middleware
- Chapter 7: Service Layer
- Chapter 8: REST API Handlers
- Chapter 12: Web Page Handlers

### Required Knowledge
- HTTP security concepts
- Common web vulnerabilities (XSS, CSRF, DDoS)
- Middleware patterns in Actix Web
- Understanding of HTTP headers

### System Requirements
- Running application from previous chapters
- HTTP client for testing (curl, Postman, or browser)

## Learning Objectives

By the end of this chapter, you will be able to:

1. Implement IP-based rate limiting to prevent abuse
2. Sanitize user input to prevent XSS attacks
3. Configure comprehensive security headers
4. Understand and implement Content Security Policy
5. Set up HSTS for production HTTPS enforcement
6. Configure request size limits
7. Test security features with attack vectors
8. Create a security checklist for production deployment

## Concepts Covered

### Defense in Depth Strategy

**Defense in depth** is a layered security approach where multiple independent security measures protect your application. If one layer is bypassed, others remain to prevent exploitation.

```
┌─────────────────────────────────────────────────────┐
│  Layer 1: Rate Limiting (Prevent Abuse)             │
│  ├─ IP-based throttling                             │
│  └─ Burst protection                                │
└─────────────────────────────────────────────────────┘
              ↓
┌─────────────────────────────────────────────────────┐
│  Layer 2: Input Validation (Reject Bad Data)        │
│  ├─ Type checking (compile-time)                    │
│  ├─ Validation rules (validator crate)              │
│  └─ Size limits                                     │
└─────────────────────────────────────────────────────┘
              ↓
┌─────────────────────────────────────────────────────┐
│  Layer 3: Sanitization (Clean User Input)           │
│  ├─ HTML sanitization (ammonia)                     │
│  └─ SQL injection prevention (SeaORM)               │
└─────────────────────────────────────────────────────┘
              ↓
┌─────────────────────────────────────────────────────┐
│  Layer 4: Security Headers (Control Browser)        │
│  ├─ CSP (limit resource loading)                    │
│  ├─ HSTS (force HTTPS)                              │
│  ├─ X-Frame-Options (prevent clickjacking)          │
│  └─ X-Content-Type-Options (prevent MIME sniffing)  │
└─────────────────────────────────────────────────────┘
```

Each layer addresses different attack vectors. An attacker must bypass all layers to exploit a vulnerability.

### Rate Limiting with actix-governor

**Rate limiting** restricts how many requests a client can make in a time window, preventing:
- **DDoS attacks**: Overwhelming server with requests
- **Brute force attacks**: Trying many passwords
- **API abuse**: Scraping or resource exhaustion
- **Cost attacks**: Expensive operations repeated

**actix-governor** implements the **token bucket algorithm**:

```
Token Bucket (per IP address):
┌────────────────────────────┐
│  Bucket Size: 100 tokens   │ ← Maximum burst
│  Refill: 1 token/600ms     │ ← Steady rate (100/min)
└────────────────────────────┘

Request arrives:
  ├─ Tokens available? → Allow request, consume 1 token
  └─ No tokens? → Reject with 429 Too Many Requests
```

**Why token bucket?**
- Allows bursts (user can make 100 quick requests if tokens saved)
- Fair steady-state rate (100 requests per minute sustained)
- Per-IP tracking (each IP has its own bucket)

**Configuration parameters**:
```rust
milliseconds_per_request(600)  // 600ms per request = 100 req/min
burst_size(100)                // Allow 100 tokens in bucket
```

**Calculation**:
- 1 request every 600ms = 1000ms / 600ms ≈ 1.67 req/sec
- 1.67 req/sec × 60 sec = 100 req/min

### HTML Sanitization with ammonia

**HTML sanitization** removes dangerous HTML/JavaScript from user input, preventing **Cross-Site Scripting (XSS)** attacks.

**XSS attack example**:
```javascript
// User submits memo title:
"<script>fetch('/api/delete-all').then(alert('Deleted!'))</script>"

// Without sanitization → Stored XSS
// When other users view this memo:
// - JavaScript executes in their browser
// - Script has full access to cookies, localStorage, can make requests
// - Can steal session tokens, perform actions as victim
```

**How ammonia works**:
```
Input HTML → Parse → Whitelist Filter → Safe HTML Output
                          ↓
              Allowed: <p>, <strong>, <em>, <ul>, <li>, <a>
              Blocked: <script>, <iframe>, <object>, event handlers
```

**ammonia's approach** (Mozilla's whitelist):
1. **Parse HTML**: Convert to DOM tree
2. **Walk tree**: Check each element and attribute
3. **Whitelist check**: Allow only safe tags (p, strong, em, ul, li, a, etc.)
4. **Attribute check**: Remove event handlers (onclick, onerror, etc.)
5. **URL validation**: Check href/src for javascript: protocol
6. **Serialize**: Convert back to safe HTML string

**Example transformations**:
```html
Input:  <script>alert('xss')</script>Hello
Output: Hello

Input:  <img src=x onerror=alert('xss')>
Output: <img src="x">

Input:  <p>Hello <strong>World</strong></p>
Output: <p>Hello <strong>World</strong></p>  (unchanged)
```

**Where to sanitize**: At the **service layer** (Chapter 7), not handlers. This ensures:
- All entry points (REST API, web forms) are protected
- Business logic always works with clean data
- Single source of truth for sanitization

### Security Headers Explained

**Security headers** instruct browsers how to handle your content, preventing various attacks.

#### Content-Security-Policy (CSP)

**What it prevents**: XSS, code injection, clickjacking

**How it works**: Browser only loads resources from allowed sources.

```http
Content-Security-Policy: default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'
```

**Directive breakdown**:
- `default-src 'self'`: Only load resources from same origin (no CDNs unless specified)
- `script-src 'self' 'unsafe-inline'`: JavaScript from same origin + inline `<script>` tags
  - `'unsafe-inline'` needed for our vanilla JS in templates (acceptable trade-off)
  - Production alternative: use nonces or hashes for specific inline scripts
- `style-src 'self' 'unsafe-inline'`: CSS from same origin + inline styles
- `img-src 'self' data:`: Images from same origin + data URIs
- `connect-src 'self'`: AJAX/fetch only to same origin
- `frame-ancestors 'none'`: Cannot be embedded in iframes (clickjacking prevention)

**Example attack prevented**:
```html
<!-- Attacker injects this via XSS: -->
<script src="https://evil.com/steal-cookies.js"></script>

<!-- Browser blocks it (violates CSP script-src 'self') -->
<!-- Console: Refused to load script from 'https://evil.com/'
     because it violates CSP directive "script-src 'self'" -->
```

#### Strict-Transport-Security (HSTS)

**What it prevents**: Man-in-the-middle attacks, protocol downgrade attacks

**How it works**: Browser remembers to always use HTTPS for this site.

```http
Strict-Transport-Security: max-age=31536000; includeSubDomains
```

**Parameters**:
- `max-age=31536000`: Remember for 1 year (31,536,000 seconds)
- `includeSubDomains`: Apply to all subdomains (api.example.com, www.example.com)

**Attack scenario without HSTS**:
```
1. User types: http://example.com
2. Server redirects: https://example.com
3. Attacker intercepts step 1 (MITM on public WiFi)
4. Attacker serves fake page, steals credentials
```

**With HSTS**:
```
1. User types: http://example.com
2. Browser: "I remember this site requires HTTPS"
3. Browser automatically upgrades to https://example.com
4. No insecure request sent, attacker can't intercept
```

**First visit problem**: HSTS only protects after first HTTPS visit. Solution: **HSTS preload list** (browser ships with list of HTTPS-only domains).

#### X-Frame-Options

**What it prevents**: Clickjacking attacks

**How it works**: Prevents site from being embedded in `<iframe>`.

```http
X-Frame-Options: DENY
```

**Options**:
- `DENY`: Never allow framing
- `SAMEORIGIN`: Allow framing only from same origin
- `ALLOW-FROM uri`: Allow from specific URI (deprecated, use CSP instead)

**Clickjacking attack example**:
```html
<!-- Attacker's page -->
<iframe src="https://yourbank.com/transfer" style="opacity:0.001; position:absolute; top:0; left:0;">
</iframe>
<button style="position:absolute; top:100px; left:100px;">
  Click to win a prize!
</button>

<!-- User clicks button, actually clicks invisible iframe
     underneath, authorizing bank transfer -->
```

**With X-Frame-Options: DENY**: Browser refuses to load yourbank.com in iframe, attack fails.

#### X-Content-Type-Options

**What it prevents**: MIME sniffing attacks

**How it works**: Browser respects Content-Type header, doesn't guess.

```http
X-Content-Type-Options: nosniff
```

**Attack without nosniff**:
```
1. Attacker uploads file named "image.jpg"
2. File actually contains: <script>alert('xss')</script>
3. Server responds: Content-Type: image/jpeg
4. Browser "sniffs" content, detects HTML
5. Browser renders as HTML, executes JavaScript
```

**With nosniff**:
```
4. Browser sees nosniff header
5. Browser strictly treats as image/jpeg
6. Fails to render, attack prevented
```

#### X-XSS-Protection

**What it prevents**: Reflected XSS attacks

```http
X-XSS-Protection: 1; mode=block
```

**Parameters**:
- `1`: Enable XSS filter
- `mode=block`: Block entire page if XSS detected (don't try to sanitize)

**Note**: Deprecated in modern browsers (Chrome, Firefox removed it). CSP is preferred. We include it for older browser support.

**Reflected XSS example**:
```
URL: https://example.com/search?q=<script>alert('xss')</script>

Without filter:
  Server: <h1>Results for: <script>alert('xss')</script></h1>
  Browser: Executes script

With XSS filter:
  Browser: Detects script in URL reflected in page
  Browser: Blocks page rendering
  User: Sees blank page (safe, though confusing)
```

#### Referrer-Policy

**What it prevents**: Information leakage via Referer header

```http
Referrer-Policy: strict-origin-when-cross-origin
```

**Policy meanings**:
- `strict-origin-when-cross-origin`:
  - Same-origin: Send full URL
  - Cross-origin HTTPS→HTTPS: Send origin only
  - HTTPS→HTTP: Send nothing
- `no-referrer`: Never send Referer
- `origin`: Always send origin only (https://example.com)

**Why it matters**:
```
User visits: https://yourapp.com/dashboard/private-doc-id-12345
Clicks link to: https://external-site.com

Without referrer-policy:
  Referer: https://yourapp.com/dashboard/private-doc-id-12345
  (external site learns private document ID)

With strict-origin-when-cross-origin:
  Referer: https://yourapp.com
  (external site only knows origin, not path)
```

#### Permissions-Policy

**What it prevents**: Abuse of browser features

```http
Permissions-Policy: geolocation=(), microphone=(), camera=()
```

**Format**: `feature=(allowed-origins)`
- `()` = disabled for everyone
- `self` = allowed for same origin
- `https://example.com` = allowed for specific origin

**Our policy**: Disable geolocation, microphone, camera (memo app doesn't need these).

**Why it matters**: Even if XSS occurs, attacker can't access device features.

### Request Size Limits

**What it prevents**: Resource exhaustion attacks

**How it works**: Reject requests exceeding size limit before processing.

```rust
.app_data(web::JsonConfig::default().limit(262_144))  // 256 KB
.app_data(web::PayloadConfig::default().limit(262_144))
```

**Attack scenario**:
```
Attacker sends: POST /api/v1/memos
Body: 500 MB of JSON data

Without limit:
  - Server reads entire 500 MB into memory
  - Parser tries to deserialize
  - Server runs out of memory, crashes
  - Repeat → Denial of Service

With limit:
  - Server reads first 256 KB
  - Detects size exceeded
  - Returns 413 Payload Too Large
  - Closes connection, frees memory
```

**Why 256 KB?**
- Large enough for legitimate requests (memo with 1000-char title + description)
- Small enough to prevent abuse
- Adjust based on your application's needs

### CORS Security Considerations

**CORS (Cross-Origin Resource Sharing)** controls which origins can access your API from JavaScript.

**Same-Origin Policy** (browser default):
```
Page at https://yourapp.com can only make AJAX to https://yourapp.com
Requests to https://api.other.com blocked by browser
```

**CORS allows exceptions**:
```rust
// Permissive (development only)
Cors::permissive()  // Allow from any origin

// Restrictive (production)
Cors::default()
    .allowed_origin("https://yourapp.com")
    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
```

**Security note**: Our app serves both API and UI from same origin, so CORS is not critical. But if you separate frontend to different domain, configure CORS carefully:
- Never use `Cors::permissive()` in production
- Whitelist specific origins
- Limit allowed methods and headers
- Set appropriate `max_age` for preflight caching

### SQL Injection Prevention

**SQL injection** is prevented by **SeaORM's query builder**:

```rust
// SAFE: SeaORM uses parameterized queries
Entity::find()
    .filter(Column::Title.contains(&user_input))
    .all(db)
    .await?

// Compiles to:
// SELECT * FROM memos WHERE title LIKE $1
// Parameters: [user_input]
```

**Why it's safe**:
- User input is sent as parameter, not concatenated into SQL string
- Database driver escapes special characters
- No way for attacker to inject SQL commands

**Vulnerable approach** (raw SQL, never do this):
```rust
// DANGEROUS: SQL injection vulnerability
let query = format!("SELECT * FROM memos WHERE title = '{}'", user_input);
// If user_input = "'; DROP TABLE memos; --"
// SQL becomes: SELECT * FROM memos WHERE title = ''; DROP TABLE memos; --'
```

**Rust + SeaORM benefit**: Type-safe query builder makes SQL injection nearly impossible. You'd have to deliberately write raw SQL strings.

### Common Vulnerabilities Reference

**OWASP Top 10 (2021) and our mitigations**:

1. **Broken Access Control**: Not implemented yet (authentication/authorization not included in this tutorial)
2. **Cryptographic Failures**: HTTPS required (HSTS), passwords hashed (future)
3. **Injection**: ✓ SQL injection prevented (SeaORM), XSS prevented (sanitization)
4. **Insecure Design**: ✓ Defense in depth, principle of least privilege
5. **Security Misconfiguration**: ✓ Security headers, no directory listing
6. **Vulnerable Components**: ✓ Dependency scanning (cargo audit, dependabot)
7. **Authentication Failures**: Not applicable yet (no auth system)
8. **Software & Data Integrity**: ✓ Checksums via Cargo.lock, no eval()
9. **Logging Failures**: ✓ Tracing with structured logs (Chapter 1)
10. **Server-Side Request Forgery**: Not applicable (no external requests)

## Step-by-Step Instructions

### Step 1: Verify Dependencies

All security dependencies should already be in `Cargo.toml`:

```toml
[dependencies]
actix-governor = "0.10"  # Rate limiting
ammonia = "4.1"          # HTML sanitization
```

If not present, add them and run:

```bash
cargo build
```

### Step 2: Verify Rate Limiting Configuration

Check that rate limiting is configured in `src/main.rs`:

```rust
use actix_governor::{Governor, GovernorConfigBuilder};

// In main() function:
tracing::info!("Configuring rate limiting: 100 requests per minute per IP");
let governor_conf = GovernorConfigBuilder::default()
    .milliseconds_per_request(600)   // 600ms per request
    .burst_size(100)                  // Allow 100 tokens in bucket
    .finish()
    .unwrap();

HttpServer::new(move || {
    let rate_limiter = Governor::new(&governor_conf);

    App::new()
        // ... other middleware ...
        .wrap(rate_limiter)  // Add rate limiter to middleware stack
        // ... services ...
})
```

**Configuration explained**:
- `milliseconds_per_request(600)`: 1 request per 600ms = 100 req/min
- `burst_size(100)`: Bucket holds 100 tokens, allows bursts
- **Per-IP tracking**: actix-governor automatically extracts IP from connection

**Middleware order matters**: Rate limiter should be early in the stack to reject requests before expensive processing.

### Step 3: Verify HTML Sanitization

Check `src/utils/sanitize.rs` exists with sanitization functions:

```rust
/// Sanitize HTML input to prevent XSS attacks
///
/// Uses ammonia's whitelist approach:
/// - Allows safe tags: <p>, <strong>, <em>, <ul>, <li>, <a>, etc.
/// - Removes dangerous tags: <script>, <iframe>, <object>
/// - Strips event handlers: onclick, onerror, onload, etc.
/// - Validates URLs: removes javascript: protocol
pub fn sanitize_html(input: &str) -> String {
    ammonia::clean(input)
}

/// Sanitize optional HTML input
pub fn sanitize_optional_html(input: Option<&str>) -> Option<String> {
    input.map(sanitize_html)
}
```

**ammonia defaults** (no configuration needed):
- Whitelists: a, b, blockquote, br, code, div, em, h1-h6, hr, i, img, li, ol, p, pre, span, strong, sub, sup, table, tbody, td, tfoot, th, thead, tr, ul
- Allowed attributes: href (for a), src (for img), class, id, title
- URL protocols: http, https, mailto (javascript: blocked)

**Testing sanitization**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_html() {
        let input = "<script>alert('xss')</script>Hello World";
        let result = sanitize_html(input);
        assert!(!result.contains("<script>"));
        assert!(result.contains("Hello World"));
    }

    #[test]
    fn test_sanitize_html_removes_javascript() {
        let input = "<img src=x onerror=alert('xss')>";
        let result = sanitize_html(input);
        assert!(!result.contains("onerror"));
        assert!(!result.contains("alert"));
    }

    #[test]
    fn test_sanitize_html_allows_safe_tags() {
        let input = "<p>Hello <strong>World</strong></p>";
        let result = sanitize_html(input);
        assert!(result.contains("<p>"));
        assert!(result.contains("<strong>"));
        assert!(result.contains("</strong>"));
    }
}
```

Run tests:
```bash
cargo test sanitize
```

### Step 4: Verify Service Layer Sanitization

Check `src/services/memo_service.rs` uses sanitization:

```rust
use crate::utils::{sanitize_html, sanitize_optional_html};

impl MemoService {
    pub async fn create_memo(&self, dto: CreateMemoDto) -> Result<MemoResponseDto, AppError> {
        // Sanitize inputs BEFORE database
        let sanitized_title = sanitize_html(&dto.title);
        let sanitized_description = sanitize_optional_html(dto.description.as_deref());

        tracing::debug!(title = %sanitized_title, "Creating new memo with sanitized input");

        let memo = self
            .repository
            .create(
                sanitized_title,
                sanitized_description,
                dto.date_to,
            )
            .await?;

        Ok(memo.into())
    }

    pub async fn update_memo(
        &self,
        id: Uuid,
        dto: UpdateMemoDto,
    ) -> Result<MemoResponseDto, AppError> {
        let sanitized_title = sanitize_html(&dto.title);
        let sanitized_description = sanitize_optional_html(dto.description.as_deref());

        tracing::debug!("Updating memo with sanitized input");

        let memo = self
            .repository
            .update(
                id,
                sanitized_title,
                sanitized_description,
                dto.date_to,
                dto.completed,
            )
            .await?;

        Ok(memo.into())
    }

    pub async fn patch_memo(
        &self,
        id: Uuid,
        dto: PatchMemoDto,
    ) -> Result<MemoResponseDto, AppError> {
        let title = dto
            .title
            .as_ref()
            .map(|t| sanitize_html(&t))
            .or(None);

        let description = match dto.description {
            Some(d) => sanitize_optional_html(Some(&d)),
            None => None,
        };

        tracing::debug!("Patching memo with sanitized input");

        // ... rest of method
    }
}
```

**Why sanitize in service layer**:
1. **Single responsibility**: Service layer handles business logic, including data cleaning
2. **All entry points protected**: REST API, web forms, future GraphQL all go through services
3. **Testable**: Easy to unit test sanitization in isolation
4. **No duplication**: Don't sanitize in every handler

**What we sanitize**:
- `title`: Always sanitized (required field)
- `description`: Sanitized if provided (optional field)
- `date_to`: No sanitization needed (DateTime type, validated by deserialization)
- `completed`: No sanitization needed (boolean type)

### Step 5: Verify Security Headers Middleware

Check `src/middleware/security_headers.rs` exists:

```rust
use actix_web::Error;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::http::header::HeaderValue;
use std::future::{Ready, ready};
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

            // Prevent MIME sniffing
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-content-type-options"),
                HeaderValue::from_static("nosniff"),
            );

            // Prevent clickjacking
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-frame-options"),
                HeaderValue::from_static("DENY"),
            );

            // XSS filter for older browsers
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-xss-protection"),
                HeaderValue::from_static("1; mode=block"),
            );

            // Force HTTPS
            headers.insert(
                actix_web::http::header::HeaderName::from_static("strict-transport-security"),
                HeaderValue::from_static("max-age=31536000; includeSubDomains"),
            );

            // Control Referer information
            headers.insert(
                actix_web::http::header::HeaderName::from_static("referrer-policy"),
                HeaderValue::from_static("strict-origin-when-cross-origin"),
            );

            // Content Security Policy
            headers.insert(
                actix_web::http::header::HeaderName::from_static("content-security-policy"),
                HeaderValue::from_static(
                    "default-src 'self'; \
                     script-src 'self' 'unsafe-inline'; \
                     style-src 'self' 'unsafe-inline'; \
                     img-src 'self' data:; \
                     font-src 'self' data:; \
                     connect-src 'self'; \
                     frame-ancestors 'none';"
                ),
            );

            // Disable dangerous browser features
            headers.insert(
                actix_web::http::header::HeaderName::from_static("permissions-policy"),
                HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
            );

            Ok(res)
        })
    }
}
```

**Middleware pattern explained**:
1. **Transform trait**: Factory for creating middleware instances
2. **Service trait**: Actual middleware logic
3. **forward_ready!** macro: Delegate readiness check to inner service
4. **Pin<Box<dyn Future>>**: Wraps async operation for middleware chain

### Step 6: Verify Middleware Registration

Check `src/main.rs` includes SecurityHeaders middleware:

```rust
use actix_web_template::middleware::SecurityHeaders;

HttpServer::new(move || {
    App::new()
        .app_data(web::Data::new(app_state.clone()))

        // Middleware stack (order matters!)
        .wrap(prometheus.clone())           // Metrics collection
        .wrap(Compress::default())          // Response compression
        .wrap(SecurityHeaders)              // Security headers (this chapter)
        .wrap(rate_limiter)                 // Rate limiting (this chapter)
        .wrap(cors)                         // CORS
        .wrap(TracingLogger::default())     // HTTP request logging

        // Services...
})
```

**Middleware order explanation**:
1. **Prometheus**: Outermost, measures everything including errors
2. **Compress**: Before security headers (compresses response body)
3. **SecurityHeaders**: Adds headers to all responses
4. **Rate limiter**: Rejects abusive requests early
5. **CORS**: Handles preflight requests
6. **TracingLogger**: Logs all requests (innermost, sees final request)

**Why order matters**: Middleware wraps each other like Russian dolls. Inner middleware runs first on request, last on response.

### Step 7: Verify Request Size Limits

Check `src/main.rs` configures request size limits:

```rust
App::new()
    .app_data(web::Data::new(state.clone()))
    .app_data(web::JsonConfig::default().limit(state.config.api.max_request_size))
    .app_data(web::PayloadConfig::default().limit(state.config.api.max_request_size))
    // ... middleware and services ...
```

**Config sources**:
```rust
// src/config/settings.rs
pub struct ApiConfig {
    pub max_request_size: usize,  // Default: 262_144 (256 KB)
}
```

**Environment variable** (`.env`):
```bash
MAX_REQUEST_SIZE=262144  # 256 KB in bytes
```

**What's limited**:
- **JsonConfig**: Limits JSON request body size
- **PayloadConfig**: Limits all request body size (forms, files, etc.)

**Behavior when exceeded**:
```
Request: POST /api/v1/memos
Body: 300 KB JSON

Server:
  - Reads first 256 KB
  - Detects size exceeded
  - Returns: 413 Payload Too Large
  - Closes connection without deserializing
```

### Step 8: Test Rate Limiting

Test that rate limiter works by exceeding limit:

```bash
# Rapid fire requests (should be rate limited)
for i in {1..110}; do
  curl -s -o /dev/null -w "%{http_code}\n" http://localhost:3737/health
done
```

**Expected output**:
```
200  # First 100 requests succeed
200
...
200
429  # Request 101+ get rate limited
429
...
```

**Response body for 429**:
```json
{
  "error": "Too Many Requests",
  "message": "Rate limit exceeded. Try again later."
}
```

**Test with longer delay** (should all succeed):
```bash
for i in {1..10}; do
  curl -s http://localhost:3737/health
  sleep 1  # Wait 1 second between requests
done
# All should return 200
```

**Per-IP isolation test**:
```bash
# Terminal 1: Your IP
for i in {1..50}; do curl -s http://localhost:3737/health; done

# Terminal 2: Different IP (if available)
curl -s http://localhost:3737/health  # Should still work
```

Each IP address has its own rate limit bucket.

### Step 9: Test HTML Sanitization

Test that XSS attempts are blocked:

**Test 1: Create memo with XSS attempt**:
```bash
curl -X POST http://localhost:3737/api/v1/memos \
  -H "Content-Type: application/json" \
  -d '{
    "title": "<script>alert(\"xss\")</script>Memo Title",
    "description": "<img src=x onerror=alert(\"xss\")>",
    "date_to": "2025-12-31T23:59:00Z"
  }'
```

**Check response** (should have script tags removed):
```json
{
  "id": "...",
  "title": "Memo Title",
  "description": "<img src=\"x\">",
  "date_to": "2025-12-31T23:59:00Z",
  "completed": false,
  ...
}
```

**Verify in database**:
```bash
psql $DATABASE_URL -c "SELECT title, description FROM memos ORDER BY created_at DESC LIMIT 1;"
```

**Expected**:
```
      title       |    description
------------------+-------------------
 Memo Title       | <img src="x">
```

No `<script>` tags, no `onerror` attributes.

**Test 2: Allowed HTML**:
```bash
curl -X POST http://localhost:3737/api/v1/memos \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Important: <strong>Read this</strong>",
    "description": "<p>Hello <em>world</em></p>",
    "date_to": "2025-12-31T23:59:00Z"
  }'
```

**Expected response** (safe tags preserved):
```json
{
  "title": "Important: <strong>Read this</strong>",
  "description": "<p>Hello <em>world</em></p>",
  ...
}
```

Safe HTML tags are allowed through.

### Step 10: Test Security Headers

Verify security headers are present on responses:

```bash
curl -I http://localhost:3737/
```

**Expected headers**:
```http
HTTP/1.1 200 OK
content-type: text/html; charset=utf-8
x-content-type-options: nosniff
x-frame-options: DENY
x-xss-protection: 1; mode=block
strict-transport-security: max-age=31536000; includeSubDomains
referrer-policy: strict-origin-when-cross-origin
content-security-policy: default-src 'self'; script-src 'self' 'unsafe-inline'; ...
permissions-policy: geolocation=(), microphone=(), camera=()
```

**Test each header**:

```bash
# Check specific header
curl -s -I http://localhost:3737/ | grep -i "x-frame-options"
# Output: x-frame-options: DENY

curl -s -I http://localhost:3737/ | grep -i "strict-transport-security"
# Output: strict-transport-security: max-age=31536000; includeSubDomains

curl -s -I http://localhost:3737/ | grep -i "content-security-policy"
# Output: content-security-policy: default-src 'self'; ...
```

**Test on different endpoints**:
```bash
curl -I http://localhost:3737/api/v1/memos
curl -I http://localhost:3737/health
curl -I http://localhost:3737/static/css/style.css
```

All should have security headers (middleware applies to all responses).

### Step 11: Test Request Size Limit

Test that oversized requests are rejected:

**Create large payload** (300 KB, exceeds 256 KB limit):
```bash
# Generate 300 KB of JSON
python3 << EOF
import json
payload = {
    "title": "Test",
    "description": "A" * 300000,  # 300 KB of 'A' characters
    "date_to": "2025-12-31T23:59:00Z"
}
print(json.dumps(payload))
EOF > large.json

# Send it
curl -X POST http://localhost:3737/api/v1/memos \
  -H "Content-Type: application/json" \
  -d @large.json
```

**Expected response**:
```
413 Payload Too Large
```

**Test normal-sized request** (should work):
```bash
curl -X POST http://localhost:3737/api/v1/memos \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Normal memo",
    "description": "This is fine",
    "date_to": "2025-12-31T23:59:00Z"
  }'
```

**Expected**: 201 Created

### Step 12: Security Audit with Browser DevTools

Open application in browser and check security:

**1. Open DevTools** (F12)

**2. Check Console for CSP violations**:
```javascript
// Try to load external script (should fail)
var script = document.createElement('script');
script.src = 'https://evil.com/malicious.js';
document.head.appendChild(script);

// Console output:
// Refused to load the script 'https://evil.com/malicious.js' because it
// violates the following Content Security Policy directive: "script-src 'self' 'unsafe-inline'"
```

**3. Check Security tab** (Chrome DevTools):
- Navigate to Security tab
- Should show: "This page is secure (valid HTTPS)" (if using HTTPS)
- Should list security headers

**4. Check Network tab**:
- Reload page
- Click on any request
- Headers tab → Response Headers
- Verify all security headers present

**5. Test iframe embedding** (should fail):
```html
<!-- Create test.html -->
<iframe src="http://localhost:3737/"></iframe>

<!-- Open test.html in browser -->
<!-- Console should show: -->
Refused to display 'http://localhost:3737/' in a frame because it set 'X-Frame-Options' to 'deny'.
```

### Step 13: CORS Testing

Test CORS configuration:

**Same-origin request** (should work):
```bash
curl -X GET http://localhost:3737/api/v1/memos \
  -H "Origin: http://localhost:3737"
```

**Expected**: 200 OK, response includes data

**Cross-origin request** (should respect CORS config):
```bash
curl -X GET http://localhost:3737/api/v1/memos \
  -H "Origin: https://evil.com" \
  -v
```

**Check response headers**:
- If `Cors::permissive()`: Includes `access-control-allow-origin: *`
- If restricted CORS: No CORS headers (request blocked by browser)

**Preflight request** (OPTIONS):
```bash
curl -X OPTIONS http://localhost:3737/api/v1/memos \
  -H "Origin: http://localhost:3737" \
  -H "Access-Control-Request-Method: POST" \
  -H "Access-Control-Request-Headers: content-type" \
  -v
```

**Expected**: 200 OK with CORS headers:
```http
access-control-allow-origin: http://localhost:3737
access-control-allow-methods: GET, POST, PUT, DELETE, PATCH
access-control-allow-headers: content-type
access-control-max-age: 3600
```

## Checkpoint

At this point, you should have:

**Security features implemented**:
- ✓ Rate limiting (100 req/min per IP)
- ✓ HTML sanitization (XSS prevention)
- ✓ Security headers (CSP, HSTS, X-Frame-Options, etc.)
- ✓ Request size limits (256 KB)
- ✓ CORS configuration
- ✓ SQL injection prevention (SeaORM)

**Verification commands**:
```bash
# 1. Check application runs
cargo run

# 2. Test rate limiting
for i in {1..110}; do curl -s -o /dev/null -w "%{http_code}\n" http://localhost:3737/health; done | tail -10
# Should see 429 responses

# 3. Check security headers
curl -I http://localhost:3737/ | grep -i "x-frame-options"

# 4. Test XSS prevention
curl -X POST http://localhost:3737/api/v1/memos \
  -H "Content-Type: application/json" \
  -d '{"title":"<script>alert(1)</script>Test","date_to":"2025-12-31T23:59:00Z"}' | jq .title
# Should output: "Test" (script removed)

# 5. Run sanitization tests
cargo test sanitize
```

**What works**:
- Requests are rate limited per IP
- HTML is sanitized before storage
- All responses include security headers
- Oversized requests rejected
- SQL injection impossible (type-safe queries)

**What doesn't work yet**:
- Authentication (future chapter)
- Authorization (future chapter)
- CSRF protection (not needed without sessions)

## Common Issues and Solutions

### Issue: Rate limiting not working

**Symptom**: Can make unlimited requests without 429 errors.

**Cause**: Rate limiter not registered or wrong configuration.

**Solution**:
1. **Check middleware registration**:
   ```rust
   .wrap(Governor::new(&governor_conf))
   ```

2. **Verify governor_conf creation**:
   ```rust
   let governor_conf = GovernorConfigBuilder::default()
       .milliseconds_per_request(600)
       .burst_size(100)
       .finish()
       .unwrap();
   ```

3. **Check you're testing from same IP**: Rate limits are per-IP. Different terminals on same machine use same IP.

4. **Wait for bucket refill**: After exhausting tokens, wait 60 seconds before testing again.

### Issue: HTML sanitization not removing scripts

**Symptom**: `<script>` tags appear in database.

**Cause**: Sanitization not called or called after database insert.

**Solution**:
1. **Check service layer** calls sanitization:
   ```rust
   let sanitized_title = sanitize_html(&dto.title);
   ```

2. **Verify import**:
   ```rust
   use crate::utils::{sanitize_html, sanitize_optional_html};
   ```

3. **Check sanitization happens BEFORE repository call**:
   ```rust
   // ✓ Correct order
   let sanitized = sanitize_html(&input);
   repository.create(sanitized, ...).await?;

   // ✗ Wrong order (too late)
   let memo = repository.create(input, ...).await?;
   let sanitized = sanitize_html(&memo.title);
   ```

4. **Run tests**:
   ```bash
   cargo test sanitize
   ```

### Issue: Security headers not appearing

**Symptom**: `curl -I` doesn't show security headers.

**Cause**: Middleware not registered or wrong order.

**Solution**:
1. **Check middleware registration**:
   ```rust
   .wrap(SecurityHeaders)
   ```

2. **Verify middleware order** (SecurityHeaders should be before services):
   ```rust
   .wrap(SecurityHeaders)
   .service(handlers::index)
   ```

3. **Check middleware module export** in `src/middleware/mod.rs`:
   ```rust
   pub mod security_headers;
   pub use security_headers::SecurityHeaders;
   ```

4. **Import in main.rs**:
   ```rust
   use actix_web_template::middleware::SecurityHeaders;
   ```

### Issue: HSTS warning in browser

**Symptom**: Browser shows HTTPS warning despite HSTS header.

**Cause**: Using HTTP in development, HSTS requires HTTPS.

**Solution**:
1. **Development**: HSTS only works over HTTPS. Use HTTP without HSTS, or set up local HTTPS.

2. **Conditional HSTS** (only in production):
   ```rust
   if settings.app.env == "production" {
       headers.insert(
           HeaderName::from_static("strict-transport-security"),
           HeaderValue::from_static("max-age=31536000; includeSubDomains"),
       );
   }
   ```

3. **Local HTTPS setup** (advanced):
   ```bash
   # Generate self-signed certificate
   openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes

   # Configure Actix Web for HTTPS
   # (requires actix-web HTTPS feature)
   ```

### Issue: CSP blocking inline scripts

**Symptom**: Browser console shows CSP violation for inline JavaScript.

**Cause**: Our CSP allows `'unsafe-inline'` for scripts, but other CSP directives may conflict.

**Solution**:
1. **Check CSP directive** includes `'unsafe-inline'`:
   ```rust
   script-src 'self' 'unsafe-inline';
   ```

2. **Use nonces for stricter CSP** (production):
   ```rust
   // Generate random nonce per request
   let nonce = generate_random_nonce();

   // CSP header
   script-src 'self' 'nonce-{nonce}';

   // Template
   <script nonce="{{ nonce }}">
   ```

3. **External script files** (no inline):
   ```html
   <!-- Instead of inline -->
   <script>function doSomething() { ... }</script>

   <!-- Use external file -->
   <script src="/static/js/app.js"></script>
   ```

### Issue: Request size limit too restrictive

**Symptom**: Legitimate requests get 413 Payload Too Large.

**Cause**: max_request_size too small for your use case.

**Solution**:
1. **Increase limit** in `.env`:
   ```bash
   MAX_REQUEST_SIZE=524288  # 512 KB
   ```

2. **Calculate needed size**:
   ```
   Max memo size:
   - Title: 200 chars = 200 bytes
   - Description: 1000 chars = 1000 bytes
   - Metadata: ~200 bytes
   - Total: ~1400 bytes
   - Safety margin: 256 KB is plenty
   ```

3. **Different limits for different endpoints** (advanced):
   ```rust
   web::scope("/api/v1/memos")
       .app_data(web::JsonConfig::default().limit(256_000))

   web::scope("/api/v1/uploads")
       .app_data(web::JsonConfig::default().limit(10_000_000))  // 10 MB for uploads
   ```

### Issue: Rate limit too aggressive

**Symptom**: Legitimate users hit rate limit.

**Cause**: Limit too low for expected usage.

**Solution**:
1. **Increase burst size** (allows more requests in short period):
   ```rust
   .burst_size(200)  // Was 100
   ```

2. **Decrease request interval** (allow more requests per minute):
   ```rust
   .milliseconds_per_request(300)  // 200 req/min instead of 100
   ```

3. **Different limits per endpoint**:
   ```rust
   let lenient = GovernorConfigBuilder::default()
       .milliseconds_per_request(100)
       .burst_size(1000)
       .finish()?;

   let strict = GovernorConfigBuilder::default()
       .milliseconds_per_request(10000)
       .burst_size(10)
       .finish()?;

   App::new()
       .service(
           web::scope("/api/v1/memos")
               .wrap(Governor::new(&lenient))
       )
       .service(
           web::scope("/api/v1/auth")
               .wrap(Governor::new(&strict))  // Stricter for auth endpoints
       )
   ```

## Code Review

Let's review the complete security implementation.

### Principles Demonstrated

**Defense in Depth**
- Multiple independent security layers
- If one layer is bypassed, others protect
- Rate limiting + sanitization + headers + validation

**Secure by Default**
- Rust's type system prevents many bugs
- SeaORM prevents SQL injection
- Compile-time template checking
- No unsafe code in our application

**Principle of Least Privilege**
- CSP restricts resource loading
- Permissions-Policy disables unnecessary features
- CORS limits which origins can access API

**Fail Securely**
- Rate limit exceeded → 429, not crash
- Oversized request → 413, not OOM
- Validation failed → 400, not internal error

**Security is Everyone's Job**
- Validated at DTO level (handlers)
- Sanitized at service level (business logic)
- Protected at middleware level (infrastructure)
- Type-safe at compiler level (Rust)

### Architecture Review

**Security layers from outside to inside**:

```
Internet Request
    ↓
┌─────────────────────────────────────┐
│  1. TLS/HTTPS (not in app)          │ ← Network layer security
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│  2. Rate Limiting (actix-governor)  │ ← Prevent abuse
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│  3. Request Size Limit              │ ← Prevent resource exhaustion
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│  4. Input Validation (validator)    │ ← Reject malformed data
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│  5. Sanitization (ammonia)          │ ← Remove dangerous content
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│  6. Business Logic (service)        │ ← Safe processing
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│  7. Database (SeaORM)               │ ← SQL injection prevention
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│  8. Security Headers                │ ← Control browser behavior
└─────────────────────────────────────┘
    ↓
Response to Client
```

**Attack surface analysis**:

```
Attack Vectors Addressed:
├─ DDoS / Brute Force → Rate limiting
├─ XSS (Cross-Site Scripting) → Sanitization + CSP
├─ SQL Injection → SeaORM parameterized queries
├─ Clickjacking → X-Frame-Options + CSP frame-ancestors
├─ MIME Sniffing → X-Content-Type-Options
├─ Protocol Downgrade → HSTS
├─ Information Leakage → Referrer-Policy
├─ Unauthorized Features → Permissions-Policy
└─ Resource Exhaustion → Request size limits

Attack Vectors Not Yet Addressed:
├─ CSRF (Cross-Site Request Forgery) → No sessions/cookies yet
├─ Broken Access Control → No authentication yet
├─ Sensitive Data Exposure → No PII in memos
└─ Insecure Deserialization → Rust prevents this
```

### Security Checklist

Use this checklist for production deployment:

**Infrastructure**:
- [ ] HTTPS enabled (TLS certificate)
- [ ] HSTS enabled (force HTTPS)
- [ ] HTTP → HTTPS redirect configured
- [ ] Firewall rules configured (only 80/443 open)
- [ ] Rate limiting enabled (not just in app, also at nginx/cloudflare)

**Application**:
- [ ] Security headers middleware enabled
- [ ] CSP configured appropriately (no 'unsafe-inline' in production)
- [ ] Rate limiting configured for all endpoints
- [ ] Request size limits set
- [ ] HTML sanitization on all user input
- [ ] CORS configured restrictively (no `Cors::permissive()`)

**Database**:
- [ ] Database credentials not in source code
- [ ] Database connection uses TLS
- [ ] Database user has minimal permissions (not superuser)
- [ ] Database backups encrypted

**Dependencies**:
- [ ] Run `cargo audit` regularly
- [ ] Dependencies up to date
- [ ] No known vulnerabilities (check Rustsec)
- [ ] Dependabot enabled (GitHub)

**Logging & Monitoring**:
- [ ] Security events logged (failed auth, rate limits)
- [ ] Logs don't contain sensitive data (passwords, tokens)
- [ ] Monitoring alerts configured (spike in 429, 5xx)
- [ ] Incident response plan documented

**Code**:
- [ ] No `unwrap()` in production code paths
- [ ] No panics on invalid input
- [ ] Error messages don't leak implementation details
- [ ] No secrets in environment variables (use secrets manager)

**Testing**:
- [ ] Security tests pass
- [ ] Penetration testing performed (if possible)
- [ ] OWASP ZAP or Burp Suite scan clean
- [ ] Load testing with security enabled

## Testing

### Security Test Suite

Create `tests/security_tests.rs`:

```rust
mod common;

use actix_web::{test, web, App};
use actix_web_template::{
    handlers,
    middleware::SecurityHeaders,
    state::AppState,
};
use common::setup_test_state;

#[actix_web::test]
async fn test_security_headers_present() {
    let state = setup_test_state().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .wrap(SecurityHeaders)
            .service(handlers::index),
    )
    .await;

    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);

    // Check all security headers present
    let headers = resp.headers();

    assert!(headers.contains_key("x-content-type-options"));
    assert_eq!(
        headers.get("x-content-type-options").unwrap(),
        "nosniff"
    );

    assert!(headers.contains_key("x-frame-options"));
    assert_eq!(headers.get("x-frame-options").unwrap(), "DENY");

    assert!(headers.contains_key("x-xss-protection"));
    assert_eq!(
        headers.get("x-xss-protection").unwrap(),
        "1; mode=block"
    );

    assert!(headers.contains_key("strict-transport-security"));
    assert!(headers
        .get("strict-transport-security")
        .unwrap()
        .to_str()
        .unwrap()
        .contains("max-age=31536000"));

    assert!(headers.contains_key("content-security-policy"));
    let csp = headers
        .get("content-security-policy")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(csp.contains("default-src 'self'"));
    assert!(csp.contains("script-src 'self' 'unsafe-inline'"));

    assert!(headers.contains_key("referrer-policy"));
    assert!(headers.contains_key("permissions-policy"));
}

#[actix_web::test]
async fn test_xss_sanitization() {
    let state = setup_test_state().await;
    let service = actix_web_template::services::MemoService::new(state.db.clone());

    // Attempt XSS in title
    let dto = actix_web_template::dto::CreateMemoDto {
        title: "<script>alert('xss')</script>Memo".to_string(),
        description: Some("<img src=x onerror=alert('xss')>".to_string()),
        date_to: chrono::Utc::now(),
    };

    let memo = service.create_memo(dto).await.unwrap();

    // Check script tags removed
    assert!(!memo.title.contains("<script>"));
    assert!(!memo.title.contains("alert"));
    assert_eq!(memo.title, "Memo");

    // Check event handlers removed
    assert!(memo.description.is_some());
    let desc = memo.description.unwrap();
    assert!(!desc.contains("onerror"));
    assert!(!desc.contains("alert"));

    // Cleanup
    service.delete_memo(memo.id).await.ok();
}

#[actix_web::test]
async fn test_allowed_html_preserved() {
    let state = setup_test_state().await;
    let service = actix_web_template::services::MemoService::new(state.db.clone());

    let dto = actix_web_template::dto::CreateMemoDto {
        title: "Important: <strong>Read</strong>".to_string(),
        description: Some("<p>Hello <em>world</em></p>".to_string()),
        date_to: chrono::Utc::now(),
    };

    let memo = service.create_memo(dto).await.unwrap();

    // Safe tags preserved
    assert!(memo.title.contains("<strong>"));
    assert!(memo.title.contains("</strong>"));

    let desc = memo.description.unwrap();
    assert!(desc.contains("<p>"));
    assert!(desc.contains("<em>"));
    assert!(desc.contains("</em>"));
    assert!(desc.contains("</p>"));

    // Cleanup
    service.delete_memo(memo.id).await.ok();
}
```

Run security tests:
```bash
cargo test security_tests
```

### Manual Security Testing

**Test XSS attempts**:

1. **Stored XSS** (persisted in database):
   ```bash
   curl -X POST http://localhost:3737/api/v1/memos \
     -H "Content-Type: application/json" \
     -d '{
       "title": "<script>alert(document.cookie)</script>",
       "description": "<img src=x onerror=alert(localStorage)>",
       "date_to": "2025-12-31T23:59:00Z"
     }'
   ```

   **Expected**: Script tags removed, memo created safely.

2. **Reflected XSS** (in response):
   ```bash
   curl http://localhost:3737/api/v1/memos?search=<script>alert(1)</script>
   ```

   **Expected**: No script execution (search parameter validated).

**Test rate limiting**:
```bash
# Rapid requests
ab -n 150 -c 10 http://localhost:3737/health

# Check for 429 responses
```

**Test request size limit**:
```bash
# Generate 300 KB payload
dd if=/dev/zero bs=1024 count=300 | base64 > /tmp/large.txt

curl -X POST http://localhost:3737/api/v1/memos \
  -H "Content-Type: application/json" \
  -d "{\"title\":\"test\",\"description\":\"$(cat /tmp/large.txt)\",\"date_to\":\"2025-12-31T23:59:00Z\"}"
```

**Expected**: 413 Payload Too Large

### Security Scanning Tools

**1. OWASP ZAP** (Zed Attack Proxy):
```bash
# Install OWASP ZAP
# Download from https://www.zaproxy.org/download/

# Automated scan
zap-cli quick-scan http://localhost:3737

# Check report for vulnerabilities
```

**2. cargo-audit** (dependency vulnerabilities):
```bash
cargo install cargo-audit

cargo audit

# Expected output:
# Fetching advisory database from `https://github.com/RustSec/advisory-db.git`
# Scanning Cargo.lock for vulnerabilities
# Success: No vulnerable packages found
```

**3. cargo-deny** (check licenses, bans, advisories):
```bash
cargo install cargo-deny

cargo deny check
```

**4. Burp Suite** (professional tool):
- Set up proxy
- Browse application through proxy
- Run active scan
- Check for SQLi, XSS, CSRF, etc.

## Summary

You've successfully implemented comprehensive security features for your web application:

**Key achievements**:
1. **Rate limiting**: Prevents DDoS and brute force attacks (100 req/min per IP)
2. **HTML sanitization**: Blocks XSS attacks with whitelist-based cleaning
3. **Security headers**: 8 headers controlling browser behavior (CSP, HSTS, etc.)
4. **Request limits**: Prevents resource exhaustion (256 KB max)
5. **CORS**: Controls cross-origin access (configurable per environment)
6. **SQL injection prevention**: Type-safe queries with SeaORM
7. **Defense in depth**: Multiple independent security layers

**Security patterns learned**:
- **Defense in depth**: Layered security approach
- **Secure by default**: Safe defaults, opt-in to less secure options
- **Fail securely**: Errors don't leak information or crash
- **Principle of least privilege**: Minimal permissions, disable unused features
- **Validation + Sanitization**: Reject bad data, clean suspicious data

**How this fits into the application**:
- **Middleware** (this chapter) wraps all requests/responses
- **Services** (Chapter 7) sanitize data before storage
- **DTOs** (Chapter 5) validate input format
- **ORM** (Chapter 2) prevents SQL injection
- **Type system** (Rust) prevents memory safety issues

Your application is now production-ready from a security perspective. The next chapter adds comprehensive testing to verify all features work correctly.

## Next Steps

In **Chapter 14: Testing Strategy**, you'll:
- Create a comprehensive test suite
- Write unit tests for business logic
- Write integration tests for API endpoints
- Test security features systematically
- Measure test coverage
- Set up test fixtures and utilities
- Learn testing best practices for Actix Web

Your application is secure. Next chapter ensures it's reliable through thorough testing.

## Additional Resources

### Official Documentation
- [actix-governor Documentation](https://docs.rs/actix-governor/) - Rate limiting
- [ammonia Documentation](https://docs.rs/ammonia/) - HTML sanitization
- [OWASP Top 10](https://owasp.org/www-project-top-ten/) - Common vulnerabilities
- [Content Security Policy (MDN)](https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP) - CSP guide

### Security Concepts
- [OWASP Cheat Sheet Series](https://cheatsheetseries.owasp.org/) - Security best practices
- [Web Security Academy](https://portswigger.net/web-security) - Free security training
- [Mozilla Web Security](https://infosec.mozilla.org/guidelines/web_security) - Guidelines
- [Security Headers](https://securityheaders.com/) - Check your headers

### Tools
- [OWASP ZAP](https://www.zaproxy.org/) - Web application security scanner
- [Burp Suite](https://portswigger.net/burp) - Professional security testing
- [cargo-audit](https://github.com/rustsec/rustsec/tree/main/cargo-audit) - Dependency auditing
- [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) - Linter for dependencies

### Rust Security
- [Rust Security Working Group](https://www.rust-lang.org/governance/wgs/wg-security) - Official security WG
- [RustSec Advisory Database](https://rustsec.org/) - Known vulnerabilities
- [Secure Rust Guidelines](https://anssi-fr.github.io/rust-guide/) - ANSSI security guide

### HTTP Security
- [HTTP Security Response Headers](https://www.netsparker.com/blog/web-security/http-security-headers/) - Comprehensive guide
- [CSP Evaluator](https://csp-evaluator.withgoogle.com/) - Check CSP policy
- [Mozilla Observatory](https://observatory.mozilla.org/) - Security scanner
