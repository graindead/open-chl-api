//! Parses saved (trimmed) replies from the real server, checks query strings, and runs the
//! client against a tiny local HTTP server. Nothing here touches the network unless
//! `CHLB_TEST_LIVE=1` is set (see `live`).

use super::*;

macro_rules! sample {
    ($name:literal) => {
        include_bytes!(concat!("../tests/samples/", $name, ".json"))
    };
}

#[test]
fn scores_top() {
    let s: Scores = parse(sample!("scores_top")).unwrap();
    assert_eq!(s.summary.total_player_count, 39);
    assert!(s.summary.song.is_none(), "asked by trackHash: no song block");
    assert!(s.near.is_none());
    let first = &s.top.score_data[0];
    assert_eq!(first.placement, Some(1));
    assert_eq!((first.name.as_str(), first.artist.as_str()), ("Enter Sandman", "Metallica"));
    assert_eq!(first.track_hash.len(), 43);
    assert!(first.is_fc && first.is_pfc);
    assert_eq!(first.accuracy_pct(), Some(100.0));
    assert!(s.top.score_data.windows(2).all(|w| w[0].total_score >= w[1].total_score));
}

#[test]
fn scores_near() {
    let s: Scores = parse(sample!("scores_near")).unwrap();
    let near = s.near.unwrap();
    assert!(near.position_offset > 0);
    let you = near.score_data.iter().find(|r| r.username == "You").unwrap();
    assert_eq!(you.account_id, 1000);
    assert!(you.placement.unwrap() as u64 > near.position_offset);
    assert!(near.score_data.windows(2).all(|w| w[0].placement < w[1].placement));
}

#[test]
fn scores_by_song_has_song_block() {
    let s: Scores = parse(sample!("scores_song")).unwrap();
    let song = s.summary.song.unwrap();
    assert_eq!(song.name, "Braindead");
    assert_eq!(song.metadata_approval_status, "approved");
    assert_eq!(song.song_hash.len(), 43);
}

#[test]
fn all_scores() {
    let v: Vec<BriefScore> = parse(sample!("all_scores")).unwrap();
    assert_eq!(v.len(), 3);
    assert!(!v[0].username.is_empty());
    assert!(v.windows(2).all(|w| w[0].total_score >= w[1].total_score));
}

#[test]
fn filters() {
    let f: Filters = parse(sample!("filters")).unwrap();
    assert!(f.states.iter().any(|s| s.controller_type == "Gamepad" && s.modifiers == ["allTaps"]));
    assert!(f.states.iter().all(|s| s.speed == 100 && s.instrument == "guitar"));
}

#[test]
fn graphs() {
    let d: DurationStats = parse(sample!("duration_stats")).unwrap();
    assert_eq!(d.segment_count, 50);
    for v in [&d.missed_note_counts, &d.failed_counts, &d.fc_streak_counts] {
        assert_eq!(v.len(), d.segment_count as usize);
    }
    assert!(d.failed_counts.windows(2).all(|w| w[0] <= w[1]), "running total");

    let r: RecordProgression = parse(sample!("record_progression")).unwrap();
    assert_eq!(r.points.len(), 2);
    assert!(r.points[0].total_score < r.points[1].total_score);
    assert_eq!(r.points[0].score.total_score, r.points[0].total_score);
}

#[test]
fn latest_and_stats() {
    let l: Vec<Score> = parse(sample!("latest_scores")).unwrap();
    assert!(l[0].score_id > 0 && l[0].placement.is_none());

    let p: Vec<PlayerRank> = parse(sample!("player_leaderboard")).unwrap();
    assert!(p.windows(2).all(|w| w[0].value >= w[1].value));
    let s: Vec<SongRank> = parse(sample!("song_leaderboard")).unwrap();
    assert!(s[0].plays > 0 && !s[0].song_hash.is_empty());
    let t: SiteTotals = parse(sample!("site_totals")).unwrap();
    assert!(t.users > 0 && t.submissions > t.users && t.pfcs <= t.fcs);
}

#[test]
fn profile() {
    let p: Profile = parse(sample!("profile")).unwrap();
    assert_eq!((p.account_id, p.username.as_str()), (1000, "You"));
    assert!(p.stats.songs_played > 0 && p.stats.total_pfcs <= p.stats.total_fcs);
    assert!(p.ranks.total_score.is_some() && p.ranks.notes_hit.is_some());
    assert_eq!(p.top_instruments[0].instrument, "guitar");
    assert_eq!(p.latest_submissions[0].account_id, 1000);
    assert!(!p.most_played.is_empty() && p.top_submissions.is_empty());
    assert!(p.deleted_at.is_none());
}

#[test]
fn account_scores_paging() {
    let a: AccountScores = parse(sample!("account_scores")).unwrap();
    assert_eq!((a.page, a.page_size), (1, ACCOUNT_PAGE_SIZE));
    assert!(a.total_count > 50);
    assert!(!a.is_last_page());
    let end: AccountScores = parse(sample!("account_scores_past_end")).unwrap();
    assert!(end.scores.is_empty() && end.total_count > 0 && end.is_last_page());
    let exact = AccountScores { scores: vec![Score::default()], total_count: 100, page: 2, page_size: 50 };
    assert!(exact.is_last_page());
}

/// Charts with hidden metadata come back with asterisks for their details and `"icon": null`;
/// one such row mustn't fail the whole page.
#[test]
fn nulls_read_as_defaults() {
    let a: AccountScores = parse(sample!("account_scores_hidden_metadata")).unwrap();
    let hidden = &a.scores[0];
    assert_eq!((hidden.icon.as_str(), hidden.album.as_str(), hidden.squeezes), ("", "", 0));
    assert_eq!(hidden.name, "********");
    assert_eq!((hidden.total_score, hidden.accuracy_pct()), (210000, Some(94.7)));
    assert_eq!(a.scores[1].icon, "rb1");
    let b: BriefScore = parse(br#"{"username": null, "totalScore": null, "accuracy": null}"#).unwrap();
    assert_eq!((b.username.as_str(), b.total_score, b.accuracy), ("", 0, None));
    let p: Profile = parse(br#"{"username": "x", "stats": null, "topArtists": null, "latestSubmissions": [{"icon": null}]}"#).unwrap();
    assert!(p.top_artists.is_empty() && p.latest_submissions[0].icon.is_empty());
}

#[test]
fn search() {
    let q: QuickSearch = parse(sample!("search_quick")).unwrap();
    assert_eq!(q.query, "sandman");
    assert!(q.songs[0].play_count >= q.songs[0].player_count);
    assert!(q.users[0].account_id > 0);
    let s: Search = parse(sample!("search")).unwrap();
    assert_eq!(s.next_songs_page, Some(2));
    assert!(s.found_songs >= s.songs.len() as u64);
    let b: Banner = parse(sample!("banner")).unwrap();
    assert!(b.enabled && !b.message.is_empty());
}

#[test]
fn error_bodies() {
    let e: ApiError = parse(sample!("error_400")).unwrap();
    assert_eq!((e.name.as_str(), e.status), ("AJV_VALIDATION_ERROR", 400));
    let msg = status_error(reqwest::StatusCode::BAD_REQUEST, sample!("error_400"));
    assert!(msg.contains("400") && msg.contains("metric must be equal to one of the allowed values"), "{msg}");
    assert!(!msg.contains("Bad request on parameter"));
    let msg = status_error(reqwest::StatusCode::NOT_FOUND, sample!("error_404"));
    assert!(msg.ends_with("Account with id \"999999999\" not found."), "{msg}");
    assert_eq!(status_error(reqwest::StatusCode::BAD_GATEWAY, b"<html>"), "Clone Hero's leaderboards answered 502 Bad Gateway");
}

#[test]
fn unknown_and_missing_fields_are_fine() {
    let s: Score = parse(br#"{"totalScore": 5, "brandNewField": [1, 2]}"#).unwrap();
    assert_eq!(s.total_score, 5);
    assert!(s.accuracy.is_none() && s.accuracy_pct().is_none());
    let s: Scores = parse(br#"{}"#).unwrap();
    assert!(s.top.score_data.is_empty());
}

#[test]
fn query_strings() {
    // The game's own request, in its order (from Player.log).
    let q = ScoresQuery::track("H", "guitar", "expert").top(9).near(7, 2, 2).speed(100).controller("5Fret").sort("score");
    let s: Vec<String> = q.pairs().into_iter().map(|(k, v)| format!("{k}={v}")).collect();
    assert_eq!(
        s.join("&"),
        "topScores=9&nearHigher=2&nearLower=2&trackHash=H&accountId=7&instrument=guitar&difficulty=expert&minSpeed=100&maxSpeed=100&controllerType=5Fret&sort=score"
    );
    // The website's: repeated keys for lists.
    let mut q = ScoresQuery::song("S", "drums", "hard");
    q.controller_types = vec!["7LaneDrums".into(), "5LaneDrums".into()];
    q.modifiers = vec!["noKick".into()];
    let keys: Vec<_> = q.pairs().into_iter().map(|(k, _)| k).collect();
    assert_eq!(keys, ["songHash", "instrument", "difficulty", "controllerTypes", "controllerTypes", "modifiers"]);

    let a = AccountScoresQuery { page: 0, best_per_chart: true, search: Some(String::new()), ..Default::default() };
    assert_eq!(a.pairs(), [("page", "1".to_string()), ("bestPerChart", "true".to_string())]);
    assert_eq!(default_controller("prodrums"), "7LaneDrums");
    assert_eq!(default_controller("bassghl"), "6Fret");
    assert_eq!(default_controller("keys"), "5Fret");
}

/// Serves each connection one canned reply and remembers the request line.
async fn serve(replies: Vec<(u16, &'static [u8])>) -> (String, std::sync::Arc<std::sync::Mutex<Vec<String>>>) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}/v1", listener.local_addr().unwrap());
    let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let log = seen.clone();
    tokio::spawn(async move {
        for (status, body) in replies {
            let (mut sock, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 8192];
            let n = sock.read(&mut buf).await.unwrap();
            let req = String::from_utf8_lossy(&buf[..n]).lines().next().unwrap_or_default().to_string();
            log.lock().unwrap().push(req);
            let head = format!(
                "HTTP/1.1 {status} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
                body.len()
            );
            sock.write_all(head.as_bytes()).await.unwrap();
            sock.write_all(body).await.unwrap();
        }
    });
    (base, seen)
}

#[tokio::test]
async fn client_against_local_server() {
    let page1: &'static [u8] = br#"{"scores":[{"scoreId":1},{"scoreId":2}],"totalCount":3,"page":1,"pageSize":2}"#;
    let page2: &'static [u8] = br#"{"scores":[{"scoreId":3}],"totalCount":3,"page":2,"pageSize":2}"#;
    let (base, seen) = serve(vec![(200, page1), (200, page2), (404, sample!("error_404")), (200, b"not json")]).await;
    let _ = rustls::crypto::ring::default_provider().install_default();
    let api = Api::with_base(reqwest::Client::new(), format!("{base}/"));

    let q = AccountScoresQuery { best_per_chart: true, ..Default::default() };
    let all = api.account_scores_all(42, &q, 10).await.unwrap();
    assert_eq!(all.iter().map(|s| s.score_id).collect::<Vec<_>>(), [1, 2, 3]);

    let err = api.profile(42).await.unwrap_err();
    assert!(err.contains("404") && err.contains("not found"), "{err}");
    let err = api.site_totals().await.unwrap_err();
    assert!(err.starts_with("Unexpected answer"), "{err}");

    let seen = seen.lock().unwrap();
    assert_eq!(seen[0], "GET /v1/account/42/scores?page=1&bestPerChart=true HTTP/1.1");
    assert_eq!(seen[1], "GET /v1/account/42/scores?page=2&bestPerChart=true HTTP/1.1");
    assert_eq!(seen[2], "GET /v1/account/42/profile HTTP/1.1");
}

/// Parses a saved `/account/{id}/scores` reply: `CHLB_TEST_FILE=<file.json> cargo test saved_file`.
#[test]
fn saved_file() {
    let Ok(path) = std::env::var("CHLB_TEST_FILE") else { return };
    let a: AccountScores = parse(&std::fs::read(path).unwrap()).unwrap();
    println!("{} rows, {} with no icon", a.scores.len(), a.scores.iter().filter(|s| s.icon.is_empty()).count());
}

/// A few real requests, gently. Run with `CHLB_TEST_LIVE=1 cargo test live -- --nocapture`;
/// `CHLB_TEST_ACCOUNT=<your account number>` adds your profile and first page of scores.
#[tokio::test]
async fn live() {
    if std::env::var("CHLB_TEST_LIVE").is_err() {
        return;
    }
    let _ = rustls::crypto::ring::default_provider().install_default();
    let http = reqwest::Client::builder().user_agent("open-chl-api-tests").build().unwrap();
    let api = Api::new(http);
    let pause = || tokio::time::sleep(std::time::Duration::from_millis(500));

    // Metallica - Enter Sandman (oBLINKxONCEo), lead guitar on Expert.
    let hash = "-Nku724NgARbyAgbfSHvqhPBeskhr5rF-m_FTuv9B5s";
    let q = ScoresQuery::track(hash, "guitar", "expert").speed(100).top(5).sort("score");
    let b = api.scores(&q).await.unwrap();
    assert_eq!(b.top.score_data[0].placement, Some(1));
    println!("Enter Sandman: {} players", b.summary.total_player_count);
    pause().await;
    assert!(!api.filters(None, Some(hash)).await.unwrap().states.is_empty());
    pause().await;
    assert!(!api.all_scores(hash, "5Fret", 100).await.unwrap().is_empty());
    pause().await;
    assert!(api.site_totals().await.unwrap().users > 0);
    pause().await;
    let err = api.player_leaderboard("nope").await.unwrap_err();
    assert!(err.contains("400"), "{err}");

    if let Some(id) = std::env::var("CHLB_TEST_ACCOUNT").ok().and_then(|v| v.parse().ok()) {
        pause().await;
        let p = api.profile(id).await.unwrap();
        pause().await;
        let s = api.account_scores(id, &AccountScoresQuery { best_per_chart: true, ..Default::default() }).await.unwrap();
        println!("{}: {} best scores", p.username, s.total_count);
    }
}
