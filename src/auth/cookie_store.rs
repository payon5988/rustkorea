use topcoat::{
    context::Cx,
    cookie::{Cookie, Cookies, SameSite, time::Duration as TimeDuration},
    session::{Token, TokenStore, TokenStoreFuture},
};

/// Whether session cookies require HTTPS (`Secure` + `__Host-` prefix).
/// Set `SECURE_COOKIES=0` for plain-HTTP testing. Defaults to secure.
pub fn secure_cookies() -> bool {
    std::env::var("SECURE_COOKIES").as_deref() != Ok("0")
}

/// Session cookie names (Secure vs plain-HTTP modes).
pub const SESSION_COOKIE_SECURE: &str = "__Host-session";
pub const SESSION_COOKIE_PLAIN: &str = "session";

fn cookie_name() -> &'static str {
    if secure_cookies() {
        SESSION_COOKIE_SECURE
    } else {
        SESSION_COOKIE_PLAIN
    }
}

fn fallback_name() -> &'static str {
    if secure_cookies() {
        SESSION_COOKIE_PLAIN
    } else {
        SESSION_COOKIE_SECURE
    }
}

/// Session [`TokenStore`] mirroring the framework default
/// (`HttpOnly`, `SameSite=Lax`, `Path=/`) but with the `Secure` flag and
/// `__Host-` prefix controlled by `SECURE_COOKIES`.
#[derive(Debug, Clone, Copy, Default)]
pub struct EnvCookieStore;

impl TokenStore for EnvCookieStore {
    fn read<'a>(&'a self, cx: &'a Cx) -> TokenStoreFuture<'a, Option<Token>> {
        Box::pin(async move {
            let jar = topcoat::cookie::cookies(cx);
            let raw = jar.get(cookie_name()).or_else(|| jar.get(fallback_name()));
            match raw {
                Some(cookie) => Ok(Token::decode(cookie.value_trimmed()).ok()),
                None => Ok(None),
            }
        })
    }

    fn write<'a>(
        &'a self,
        cx: &'a Cx,
        token: Token,
        max_age: std::time::Duration,
    ) -> TokenStoreFuture<'a, ()> {
        Box::pin(async move {
            let max_age = TimeDuration::try_from(max_age)?;
            let mut cookie = Cookie::new(cookie_name(), token.encode());
            cookie.set_path("/");
            cookie.set_http_only(true);
            cookie.set_same_site(SameSite::Lax);
            cookie.set_max_age(max_age);
            if secure_cookies() {
                cookie.set_secure(true);
            }
            topcoat::cookie::cookies(cx).add(cookie);
            Ok(())
        })
    }

    fn delete<'a>(&'a self, cx: &'a Cx) -> TokenStoreFuture<'a, ()> {
        Box::pin(async move {
            let jar = topcoat::cookie::cookies(cx);
            jar.remove(Cookie::new(SESSION_COOKIE_SECURE, ""));
            jar.remove(Cookie::new(SESSION_COOKIE_PLAIN, ""));
            Ok(())
        })
    }
}
