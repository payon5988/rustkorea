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

fn cookie_name() -> &'static str {
    if secure_cookies() {
        "__Host-session"
    } else {
        "session"
    }
}

fn fallback_name() -> &'static str {
    if secure_cookies() {
        "session"
    } else {
        "__Host-session"
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
            jar.remove(Cookie::new("__Host-session", ""));
            jar.remove(Cookie::new("session", ""));
            Ok(())
        })
    }
}
