use crate::db::Db;

pub async fn create_notification(
    db: &Db,
    user_id: &str,
    noti_type: &str,
    from_user_id: Option<&str>,
    post_id: Option<&str>,
    message: &str,
) {
    sqlx::query(
        "INSERT INTO notifications (user_id, noti_type, from_user_id, post_id, message, is_read, created_at)
         VALUES ($1, $2, $3, $4, $5, 0, NOW()::text)"
    )
    .bind(user_id)
    .bind(noti_type)
    .bind(from_user_id)
    .bind(post_id)
    .bind(message)
    .execute(db)
    .await
    .ok();
}

pub async fn add_points(db: &Db, user_id: &str, points: i64, reason: &str) {
    sqlx::query("UPDATE users SET points = points + $1 WHERE id = $2")
        .bind(points)
        .bind(user_id)
        .execute(db)
        .await
        .ok();
    sqlx::query(
        "INSERT INTO point_history (user_id, points, reason, created_at) VALUES ($1, $2, $3, NOW()::text)"
    )
    .bind(user_id).bind(points).bind(reason).execute(db).await.ok();
}

pub async fn check_and_award_badges(db: &Db, user_id: &str) {
    let points: i64 = sqlx::query_scalar("SELECT points FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(db)
        .await
        .unwrap_or(0);

    let post_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM posts WHERE author_id = $1")
        .bind(user_id)
        .fetch_one(db)
        .await
        .unwrap_or(0);

    let badges: Vec<(String, i64)> = sqlx::query_as("SELECT id, points_required FROM badges")
        .fetch_all(db)
        .await
        .unwrap_or_default();

    for (badge_id, required) in &badges {
        if points >= *required || (*required == 0 && post_count > 0) {
            sqlx::query("INSERT INTO user_badges (user_id, badge_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
                .bind(user_id).bind(badge_id).execute(db).await.ok();
        }
    }
}
