use topcoat::{
    context::Cx,
    router::{
        Body, Layer, LayerFuture, Next, Path, header::HeaderValue, response::response_headers,
    },
};

/// Defense-in-depth HTTP security headers for every response.
///
/// CSP allows the app's inline scripts/styles (the views render inline
/// `style=`/`on*` handlers), YouTube embeds, Google Fonts, and AdSense.
/// Tighten `script-src` once inline code moves to bundled assets.
#[derive(Debug, Clone, Copy, Default)]
pub struct SecurityHeaders;

impl SecurityHeaders {
    pub const fn new() -> Self {
        Self
    }
}

/// Header values in one place (no inline literals at the call site).
pub const X_CONTENT_TYPE_OPTIONS: &str = "nosniff";
pub const X_FRAME_OPTIONS: &str = "SAMEORIGIN";
pub const REFERRER_POLICY: &str = "strict-origin-when-cross-origin";
pub const PERMISSIONS_POLICY: &str = "camera=(), microphone=(), geolocation=()";
pub const CONTENT_SECURITY_POLICY: &str = "default-src 'self'; \
     script-src 'self' 'unsafe-inline' https://www.youtube.com https://s.ytimg.com https://cdnjs.cloudflare.com https://pagead2.googlesyndication.com https://www.googletagservices.com; \
     style-src 'self' 'unsafe-inline' https://fonts.googleapis.com https://cdnjs.cloudflare.com; \
     font-src 'self' https://fonts.gstatic.com; \
     img-src 'self' data: https:; \
     media-src 'self' https:; \
     frame-src 'self' https://www.youtube.com https://www.youtube-nocookie.com https://googleads.g.doubleclick.net; \
     connect-src 'self' https:; \
     object-src 'none'; base-uri 'self'; form-action 'self'";

impl Layer for SecurityHeaders {
    fn path(&self) -> Option<&Path> {
        None
    }

    fn handle<'a>(&'a self, cx: &'a Cx, body: Body, next: Next<'a>) -> LayerFuture<'a> {
        Box::pin(async move {
            let headers = response_headers(cx);
            let set = |name: &str, value: &str| {
                if let (Ok(n), Ok(v)) = (
                    name.parse::<topcoat::router::header::HeaderName>(),
                    HeaderValue::from_str(value),
                ) {
                    headers.append(n, v);
                }
            };
            set("x-content-type-options", X_CONTENT_TYPE_OPTIONS);
            set("x-frame-options", X_FRAME_OPTIONS);
            set("referrer-policy", REFERRER_POLICY);
            set("permissions-policy", PERMISSIONS_POLICY);
            set("content-security-policy", CONTENT_SECURITY_POLICY);
            let response = next.run(cx, body).await?;
            Ok(response)
        })
    }
}
