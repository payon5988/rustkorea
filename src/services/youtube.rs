use chrono::{DateTime, Utc};
use std::sync::OnceLock;

use crate::config;

/// YouTube channel RSS endpoint; channel id is appended per request.
pub const RSS_FEED_BASE: &str = "https://www.youtube.com/feeds/videos.xml";

/// Trusted thumbnail CDN prefixes (RSS-supplied URLs are untrusted input).
pub const ALLOWED_THUMB_HOSTS: [&str; 2] = ["https://i.ytimg.com/", "https://img.youtube.com/"];

const RSS_FEED_URL: &str = RSS_FEED_BASE;

#[derive(Debug, Clone)]
pub struct VideoMetadata {
    pub video_id: String,
    pub channel_id: String,
    pub title: String,
    pub description: String,
    pub thumbnail_url: String,
    pub published_at: String,
    pub tags: Vec<String>,
}

fn http_client() -> reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT
        .get_or_init(|| {
            reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(config::http_timeout_secs()))
                .user_agent(config::http_user_agent())
                .build()
                .expect("reqwest client")
        })
        .clone()
}

pub async fn fetch_channel_videos(channel_id: &str) -> Result<Vec<VideoMetadata>, reqwest::Error> {
    let url = format!("{RSS_FEED_URL}?channel_id={channel_id}");
    let response = http_client().get(&url).send().await?.text().await?;

    Ok(parse_rss_feed(&response, channel_id))
}

fn parse_rss_feed(xml: &str, channel_id: &str) -> Vec<VideoMetadata> {
    use quick_xml::XmlVersion;
    use quick_xml::escape::resolve_predefined_entity;
    use quick_xml::events::Event;
    use quick_xml::reader::Reader;

    let mut videos = Vec::new();
    // No trim_text: text around entity references arrives as separate
    // events, so values are trimmed once when the element ends.
    let mut reader = Reader::from_str(xml);

    let mut buf = Vec::new();
    let mut current_field = String::new();
    let mut text = String::new();
    let mut title = String::new();
    let mut video_id = String::new();
    let mut published = String::new();
    let mut description = String::new();
    let mut thumbnail = String::new();
    let mut in_entry = false;

    let push_video = |title: &mut String,
                      video_id: &mut String,
                      published: &mut String,
                      description: &mut String,
                      thumbnail: &mut String,
                      videos: &mut Vec<VideoMetadata>| {
        if video_id.is_empty() {
            return;
        }
        let published_at = DateTime::parse_from_rfc3339(published)
            .map(|dt| dt.with_timezone(&Utc).to_rfc3339())
            .unwrap_or_else(|_| Utc::now().to_rfc3339());
        // Thumbnails from RSS are untrusted input: only allow YouTube CDN hosts.
        let thumbnail_url = if ALLOWED_THUMB_HOSTS.iter().any(|h| thumbnail.starts_with(h)) {
            thumbnail.clone()
        } else {
            String::new()
        };
        videos.push(VideoMetadata {
            video_id: video_id.clone(),
            channel_id: channel_id.to_string(),
            title: title.clone(),
            description: description.clone(),
            thumbnail_url,
            published_at,
            tags: vec![],
        });
        title.clear();
        video_id.clear();
        published.clear();
        description.clear();
        thumbnail.clear();
    };

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                text.clear();
                match e.name().as_ref() {
                    b"entry" => {
                        in_entry = true;
                        title.clear();
                        video_id.clear();
                        published.clear();
                        description.clear();
                        thumbnail.clear();
                    }
                    _ => {
                        current_field = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                    }
                }
            }
            Ok(Event::Empty(e)) => {
                if in_entry && e.name().as_ref() == b"media:thumbnail" {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"url" {
                            if let Ok(url) = attr.normalized_value(XmlVersion::Implicit1_0) {
                                thumbnail = url.into_owned();
                            }
                        }
                    }
                }
            }
            // Element text arrives in pieces: plain text, entity references
            // (`&amp;`, `&#38;`) and CDATA. Collect them and assign on End.
            Ok(Event::Text(e)) if in_entry => {
                text.push_str(&e.decode().unwrap_or_default());
            }
            Ok(Event::GeneralRef(e)) if in_entry => {
                if let Ok(Some(ch)) = e.resolve_char_ref() {
                    text.push(ch);
                } else if let Some(s) = e
                    .decode()
                    .ok()
                    .and_then(|name| resolve_predefined_entity(&name))
                {
                    text.push_str(s);
                }
            }
            Ok(Event::CData(e)) if in_entry => {
                text.push_str(&String::from_utf8_lossy(&e.into_inner()));
            }
            Ok(Event::End(e)) if e.name().as_ref() == b"entry" => {
                in_entry = false;
                push_video(
                    &mut title,
                    &mut video_id,
                    &mut published,
                    &mut description,
                    &mut thumbnail,
                    &mut videos,
                );
            }
            Ok(Event::End(_)) => {
                if in_entry {
                    let value = text.trim().to_string();
                    match current_field.as_str() {
                        "title" if title.is_empty() => title = value,
                        "yt:videoId" => video_id = value,
                        "published" => published = value,
                        "media:description" => description = value,
                        _ => {}
                    }
                }
                current_field.clear();
                text.clear();
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    videos
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns:yt="http://www.youtube.com/xml/schemas/2015" xmlns:media="http://search.yahoo.com/mrss/">
<entry><title>Hello &amp; World</title><yt:videoId>abc123XYZ_-</yt:videoId><published>2026-01-01T00:00:00+00:00</published><media:description>desc</media:description><media:thumbnail url="https://i.ytimg.com/vi/abc123XYZ_-/hqdefault.jpg"/></entry>
<entry><title>Evil</title><yt:videoId>evil1</yt:videoId><published>2026-01-02T00:00:00+00:00</published><media:thumbnail url="https://evil.example/x.jpg"/></entry>
</feed>"#;

    #[test]
    fn parses_entries_and_filters_thumbnails() {
        let videos = parse_rss_feed(SAMPLE, "UCtest");
        assert_eq!(videos.len(), 2);
        assert_eq!(videos[0].video_id, "abc123XYZ_-");
        assert_eq!(videos[0].title, "Hello & World");
        assert!(videos[0].thumbnail_url.starts_with("https://i.ytimg.com/"));
        assert!(videos[1].thumbnail_url.is_empty());
    }

    /// Real feeds are indented and nest media fields in `media:group`;
    /// entity/char references and CDATA must join into one value.
    #[test]
    fn joins_text_pieces_in_nested_indented_feed() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns:yt="http://www.youtube.com/xml/schemas/2015" xmlns:media="http://search.yahoo.com/mrss/">
 <title>Channel</title>
 <entry>
  <yt:videoId>vid_1</yt:videoId>
  <title>Rust &#39;async&#39; &lt;tips&gt; &amp; tricks</title>
  <published>2026-02-03T04:05:06+00:00</published>
  <media:group>
   <media:title>ignored</media:title>
   <media:thumbnail url="https://i.ytimg.com/vi/vid_1/hq.jpg?a=1&amp;b=2" width="480"/>
   <media:description><![CDATA[line <b>one</b>]]> &amp; two</media:description>
  </media:group>
 </entry>
</feed>"#;
        let videos = parse_rss_feed(xml, "UCtest");
        assert_eq!(videos.len(), 1);
        let v = &videos[0];
        assert_eq!(v.video_id, "vid_1");
        assert_eq!(v.title, "Rust 'async' <tips> & tricks");
        assert_eq!(v.description, "line <b>one</b> & two");
        assert_eq!(
            v.thumbnail_url,
            "https://i.ytimg.com/vi/vid_1/hq.jpg?a=1&b=2"
        );
        assert_eq!(v.published_at, "2026-02-03T04:05:06+00:00");
    }
}
