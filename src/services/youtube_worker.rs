use tokio::time::{Duration, interval};

use crate::db::Db;
use crate::services::site_settings::get_setting;
use crate::services::youtube::fetch_channel_videos;

const POLL_INTERVAL: Duration = Duration::from_secs(900); // 15 minutes

pub async fn start_youtube_worker(db: Db) {
    let mut ticker = interval(POLL_INTERVAL);

    loop {
        ticker.tick().await;

        let channel_id = get_setting(&db, "youtube_channel_id").await;
        let channel_id = if channel_id.is_empty() {
            crate::config::youtube_channel_id()
        } else {
            channel_id
        };
        let channel_name = crate::config::youtube_channel_name();

        match fetch_channel_videos(&channel_id).await {
            Ok(videos) => {
                let count = videos.len();
                for video in &videos {
                    let tags_json =
                        serde_json::to_string(&video.tags).unwrap_or_else(|_| "[]".into());
                    let result = sqlx::query(
                        r#"
                        INSERT INTO youtube_videos (video_id, channel_id, title, description, thumbnail_url, published_at, tags)
                        VALUES ($1, $2, $3, $4, $5, $6, $7)
                        ON CONFLICT (video_id) DO UPDATE SET
                            title = EXCLUDED.title,
                            description = EXCLUDED.description,
                            thumbnail_url = EXCLUDED.thumbnail_url,
                            tags = EXCLUDED.tags
                        "#,
                    )
                    .bind(&video.video_id)
                    .bind(&video.channel_id)
                    .bind(&video.title)
                    .bind(&video.description)
                    .bind(&video.thumbnail_url)
                    .bind(&video.published_at)
                    .bind(&tags_json)
                    .execute(&db)
                    .await;

                    if let Err(e) = result {
                        eprintln!("Failed to upsert video {}: {}", video.video_id, e);
                    }
                }
                println!("[YouTube Worker] {channel_name} 채널에서 {count}개 영상 동기화 완료");
            }
            Err(e) => {
                eprintln!("[YouTube Worker] {channel_name} 채널 RSS 파싱 실패: {e}");
            }
        }
    }
}
