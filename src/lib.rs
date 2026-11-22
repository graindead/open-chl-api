//! Unofficial, read-only client for Clone Hero's leaderboard API (`https://api.clonehero.net/v1`),
//! the server behind <https://leaderboards.clonehero.net> and the game's in-game leaderboards.
//!
//! The API isn't documented by its authors. Everything here was mapped from the leaderboard
//! website's own code, the requests the game writes to its log, and the server's replies (see
//! `docs/clonehero-leaderboard-api.md`). It may change without notice, so every response type is
//! lenient: missing fields take their defaults and unknown fields are ignored.
//!
//! Only public reads are covered. Anything that needs a signed-in session (`/account/@me`,
//! `/auth`, `/scores/latest-user-scores`) or is for moderators and admins is left out.
//!
//! ```no_run
//! # async fn demo(http: reqwest::Client) -> Result<(), String> {
//! let api = open_chl_api::Api::new(http);
//! let q = open_chl_api::ScoresQuery::track("-Nku724NgARbyAgbfSHvqhPBeskhr5rF-m_FTuv9B5s", "guitar", "expert")
//!     .speed(100)
//!     .top(50);
//! let board = api.scores(&q).await?;
//! println!("{} players", board.summary.total_player_count);
//! # Ok(()) }
//! ```
//!
//! Errors are plain `String`s, ready to show to a player.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// The API's base address (version 1).
pub const API_URL: &str = "https://api.clonehero.net/v1";
/// The leaderboard website.
pub const SITE_URL: &str = "https://leaderboards.clonehero.net";

/// Leaderboard instrument keys, in the website's order. Drums and pro drums are separate boards
/// on the same drum track.
pub const INSTRUMENTS: &[&str] = &[
    "guitar",
    "guitarcoop",
    "rhythm",
    "bass",
    "drums",
    "prodrums",
    "keys",
    "guitarghl",
    "guitarcoopghl",
    "rhythmghl",
    "bassghl",
    "keysghl",
];
pub const DIFFICULTIES: &[&str] = &["expert", "hard", "medium", "easy"];
/// What a score was played with. "7LaneDrums" is the website's "4 Lane" (pads plus cymbals).
pub const CONTROLLER_TYPES: &[&str] = &["5Fret", "6Fret", "7LaneDrums", "5LaneDrums", "Gamepad"];
/// Modifier keys the website knows (it filters on them; the server may accept more).
pub const MODIFIERS: &[&str] = &[
    "allHOPOs",
    "allStrums",
    "allTaps",
    "brutalMode",
    "deadlyDynamics",
    "doubleKick",
    "doubleNotes",
    "droplessSustains",
    "HOPOsToTaps",
    "kickOnly",
    "lightsOut",
    "mirrorMode",
    "modFull",
    "modLite",
    "modPrep",
    "noFretGhosting",
    "noKick",
    "noteShuffle",
    "precisionMode",
    "strumlessHopos",
];
/// How `/scores` can rank.
pub const SORTS: &[&str] = &["score", "accuracy"];
/// What `/stats/player-leaderboard` can rank players by.
pub const PLAYER_METRICS: &[&str] = &["totalScore", "submissions", "totalFcs", "totalPfcs", "hoursPlayed", "starsEarned", "notesHit"];
/// The song chart tabs of `/stats/song-leaderboard`.
pub const SONG_TABS: &[&str] = &["trending", "topAllTime", "topYear", "topMonth", "topWeek"];
/// The controller groups of `/stats/song-leaderboard`.
pub const CONTROLLER_GROUPS: &[&str] = &["fiveFret", "sixFret", "drums"];
/// Rows per page of `/account/{id}/scores`.
pub const ACCOUNT_PAGE_SIZE: u32 = 50;

/// The controller type the website assumes for an instrument.
pub fn default_controller(instrument: &str) -> &'static str {
    match instrument {
        "drums" | "prodrums" => "7LaneDrums",
        i if i.ends_with("ghl") => "6Fret",
        _ => "5Fret",
    }
}

// ---------------------------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------------------------

/// One submitted score, as `/scores`, `/scores/latest-scores`, record progression and account
/// pages give it. Times are ISO 8601 strings; durations are milliseconds.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Score {
    /// Rank on the board asked for (1 = best). Only on `/scores` rows.
    pub placement: Option<u32>,
    pub score_id: u64,
    /// The exact track played (one instrument + difficulty of one chart version).
    pub track_hash: String,
    /// The chart as a whole (what the website's `/scores/{songHash}` pages use).
    pub song_hash: String,
    /// When the server first saw the chart.
    pub song_created_at: String,
    pub name: String,
    pub artist: String,
    pub album: String,
    pub genre: String,
    pub year: String,
    pub charter: String,
    /// Source icon key ("rb1", "ghm", a charter's own…).
    pub icon: String,
    /// Song length, ms.
    pub duration: u64,
    /// When this score was submitted.
    pub created_at: String,
    pub account_id: u64,
    pub username: String,
    pub controller_type: String,
    pub instrument: String,
    pub difficulty: String,
    pub modifiers: Vec<String>,
    /// Song speed, percent (100 = normal).
    pub speed: u32,
    pub is_fc: bool,
    pub is_pfc: bool,
    pub has_previous_fc: bool,
    pub has_previous_pfc: bool,
    pub failed_in_run: bool,
    pub base_score: i64,
    pub total_score: i64,
    pub solo_score: i64,
    pub note_score: i64,
    pub ghost_score: i64,
    pub sustain_score: i64,
    pub combo_score: i64,
    pub stars: u8,
    pub sp_score: i64,
    pub sp_phrase_count: u32,
    pub sp_phrases_hit: u32,
    /// Time spent in Star Power, ms.
    pub time_in_sp: u64,
    pub notes_hit: u32,
    pub note_count: u32,
    /// As the server gives it: "99.8". Read it with [`Score::accuracy_pct`].
    pub accuracy: Option<String>,
    pub max_combo: u32,
    pub excess_hits: u32,
    pub ghost_inputs: u32,
    pub ghosts_hit: u32,
    pub ghost_note_count: u32,
    pub accents_hit: u32,
    pub accent_note_count: u32,
    pub squeezes: u32,
}

impl Score {
    /// Accuracy as a number (99.8), when the server gave one.
    pub fn accuracy_pct(&self) -> Option<f32> {
        self.accuracy.as_deref()?.parse().ok()
    }
}

/// A chart, as `/scores` describes it when asked by `songHash`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Song {
    pub song_hash: String,
    pub track_hash: String,
    pub song_created_at: String,
    pub name: String,
    pub artist: String,
    pub album: String,
    pub genre: String,
    pub year: String,
    pub charter: String,
    pub icon: String,
    pub duration: u64,
    /// "approved", …
    pub metadata_approval_status: String,
}

/// The head of a `/scores` reply.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Summary {
    /// Only filled in when asked by `songHash`; `None` for `trackHash`.
    pub song: Option<Song>,
    /// Scores on the board as filtered (controller, speed, modifiers).
    pub submission_count: u64,
    /// Plays of the track over every filter.
    pub total_play_count: u64,
    /// Players of the track over every filter.
    pub total_player_count: u64,
    /// Banner art key of the board's leader.
    pub hero_banner_key: Option<String>,
}

/// A run of consecutive places on a board.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Section {
    /// Places before the first row (so its placement is `position_offset + 1`).
    pub position_offset: u64,
    pub score_data: Vec<Score>,
}

/// `GET /scores`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Scores {
    pub summary: Summary,
    pub top: Section,
    /// The rows around `accountId`'s score: only when one was asked for, the account has a
    /// score on the board, and it isn't already in `top`.
    pub near: Option<Section>,
}

/// One row of `GET /scores/all-scores` (what the game uses for its in-song ghost/target score).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct BriefScore {
    pub username: String,
    pub is_fc: bool,
    pub is_pfc: bool,
    pub has_previous_fc: bool,
    pub has_previous_pfc: bool,
    pub failed_in_run: bool,
    pub total_score: i64,
    pub stars: u8,
    pub notes_hit: u32,
    pub note_count: u32,
    pub accuracy: Option<String>,
    pub max_combo: u32,
    pub excess_hits: u32,
    pub ghost_inputs: u32,
    pub ghosts_hit: u32,
    pub ghost_note_count: u32,
    pub accents_hit: u32,
    pub accent_note_count: u32,
}

/// `GET /scores/filters`: every combination that has at least one score.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Filters {
    pub states: Vec<FilterState>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct FilterState {
    pub instrument: String,
    pub difficulty: String,
    pub controller_type: String,
    pub speed: u32,
    pub modifiers: Vec<String>,
}

/// `GET /scores/graphs/duration-stats`: the song cut into `segment_count` equal slices, with
/// counts per slice over the board's visible scores.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct DurationStats {
    pub total_visible_scores: u32,
    pub segment_count: u32,
    /// Notes missed in each slice, summed over scores.
    pub missed_note_counts: Vec<u32>,
    /// Scores that had failed by the end of each slice (running total).
    pub failed_counts: Vec<u32>,
    /// Scores still on a full combo at each slice (falls as combos break).
    pub fc_streak_counts: Vec<u32>,
}

/// `GET /scores/graphs/record-progression`: each time the board's record was beaten, oldest first.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RecordProgression {
    pub points: Vec<RecordPoint>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RecordPoint {
    pub created_at: String,
    pub username: String,
    pub total_score: i64,
    pub score: Score,
}

/// One row of `GET /stats/player-leaderboard`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct PlayerRank {
    pub account_id: u64,
    pub username: String,
    /// The metric asked for (hours for "hoursPlayed").
    pub value: u64,
}

/// One row of `GET /stats/song-leaderboard`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SongRank {
    pub song_hash: String,
    pub name: String,
    pub artist: String,
    pub plays: u64,
}

/// `GET /stats/site-totals`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SiteTotals {
    pub users: u64,
    pub submissions: u64,
    pub songs: u64,
    pub notes_hit: u64,
    pub fcs: u64,
    pub pfcs: u64,
    pub hours_played: u64,
    pub total_score: u64,
}

/// `GET /account/{id}/profile`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Profile {
    pub account_id: u64,
    pub username: String,
    pub created_at: String,
    pub pronouns: Option<String>,
    /// "provider" (picture from a linked sign-in) or "default".
    pub avatar_source_type: String,
    /// The provider ("discord", "google") or default avatar ("default-blue").
    pub avatar_source_key: String,
    pub profile_picture_url: Option<String>,
    pub title_key: Option<String>,
    pub flair_key: Option<String>,
    pub banner_key: Option<String>,
    pub submission_count: u64,
    pub stats: ProfileStats,
    pub ranks: ProfileRanks,
    pub top_artists: Vec<ArtistPlays>,
    pub top_instruments: Vec<InstrumentPlays>,
    /// Newest first (15 seen).
    pub latest_submissions: Vec<Score>,
    pub most_played: Vec<SongPlays>,
    /// Seen empty; assumed to hold scores like `latest_submissions`.
    pub top_submissions: Vec<Score>,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ProfileStats {
    pub total_pfcs: u64,
    pub total_fcs: u64,
    pub total_score: u64,
    pub songs_played: u64,
    pub total_time_played_ms: u64,
    pub notes_hit: u64,
    pub total_stars_earned: u64,
    pub highest_combo: u64,
}

/// The account's place on each `/stats/player-leaderboard` metric (1 = first). `None` when the
/// server leaves one out.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ProfileRanks {
    pub total_score: Option<u64>,
    pub submissions: Option<u64>,
    pub total_fcs: Option<u64>,
    pub total_pfcs: Option<u64>,
    pub hours_played: Option<u64>,
    pub stars_earned: Option<u64>,
    pub notes_hit: Option<u64>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ArtistPlays {
    pub artist: String,
    pub plays: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct InstrumentPlays {
    pub instrument: String,
    pub plays: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SongPlays {
    pub song_hash: String,
    pub name: String,
    pub artist: String,
    pub charter: String,
    pub plays: u64,
}

/// `GET /account/{id}/scores`: one page, newest first.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AccountScores {
    pub scores: Vec<Score>,
    /// Rows over every page, for the filters given.
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
}

impl AccountScores {
    /// Whether no page follows this one.
    pub fn is_last_page(&self) -> bool {
        self.scores.is_empty() || u64::from(self.page) * u64::from(self.page_size.max(1)) >= self.total_count
    }
}

/// A chart in search results.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SongHit {
    pub song_hash: String,
    pub name: String,
    pub artist: String,
    pub charter: String,
    pub icon: String,
    pub play_count: u64,
    pub player_count: u64,
}

/// A player in search results.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct UserHit {
    pub account_id: u64,
    pub username: String,
    pub avatar_source_type: String,
    pub avatar_source_key: String,
    pub profile_picture_url: Option<String>,
}

/// `GET /search/quick`: the header search box's suggestions (up to 5 of each).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct QuickSearch {
    pub query: String,
    pub songs: Vec<SongHit>,
    pub users: Vec<UserHit>,
}

/// `GET /search`: the full search page, songs and players paged separately.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Search {
    pub query: String,
    pub found_songs: u64,
    pub found_users: u64,
    /// `None` on the last page.
    pub next_songs_page: Option<u32>,
    pub next_users_page: Option<u32>,
    pub songs: Vec<SongHit>,
    pub users: Vec<UserHit>,
}

/// `GET /site-settings/announcement-banner`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Banner {
    pub enabled: bool,
    pub message: String,
}

/// The body of an error reply.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ApiError {
    /// "AJV_VALIDATION_ERROR", "NOT_FOUND", "UNAUTHORIZED", …
    pub name: String,
    pub message: String,
    pub status: u16,
}

// ---------------------------------------------------------------------------------------------
// Queries
// ---------------------------------------------------------------------------------------------

/// Which board `/scores` (and the graphs) should read, and how much of it.
///
/// `instrument` and `difficulty` are required, plus one of `track_hash` / `song_hash`. Anything
/// left `None` or empty isn't sent and the server's default applies.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ScoresQuery {
    /// 43 characters of URL-safe base64, no padding.
    pub track_hash: Option<String>,
    pub song_hash: Option<String>,
    pub instrument: String,
    pub difficulty: String,
    /// One controller, as the game asks (`controllerType`).
    pub controller_type: Option<String>,
    /// Several controllers, as the website asks (`controllerTypes`, repeated).
    pub controller_types: Vec<String>,
    /// Percent, 5–5000.
    pub min_speed: Option<u32>,
    pub max_speed: Option<u32>,
    /// Rows from the top, 1–500 (the server's default is 3).
    pub top_scores: Option<u32>,
    /// "score" (default) or "accuracy".
    pub sort: Option<String>,
    /// Only scores with exactly these modifiers (repeated).
    pub modifiers: Vec<String>,
    /// Fill `near` with the rows around this account's score.
    pub account_id: Option<u64>,
    /// Rows above / below that score to include.
    pub near_higher: Option<u32>,
    pub near_lower: Option<u32>,
}

impl ScoresQuery {
    /// A board by track hash (what the game and Truss use).
    pub fn track(hash: impl Into<String>, instrument: impl Into<String>, difficulty: impl Into<String>) -> Self {
        ScoresQuery { track_hash: Some(hash.into()), instrument: instrument.into(), difficulty: difficulty.into(), ..Default::default() }
    }

    /// A board by song hash (what the website uses).
    pub fn song(hash: impl Into<String>, instrument: impl Into<String>, difficulty: impl Into<String>) -> Self {
        ScoresQuery { song_hash: Some(hash.into()), instrument: instrument.into(), difficulty: difficulty.into(), ..Default::default() }
    }

    /// Only this speed.
    pub fn speed(mut self, percent: u32) -> Self {
        self.min_speed = Some(percent);
        self.max_speed = Some(percent);
        self
    }

    pub fn top(mut self, rows: u32) -> Self {
        self.top_scores = Some(rows);
        self
    }

    pub fn sort(mut self, by: impl Into<String>) -> Self {
        self.sort = Some(by.into());
        self
    }

    pub fn controller(mut self, controller_type: impl Into<String>) -> Self {
        self.controller_type = Some(controller_type.into());
        self
    }

    /// Include the rows around an account's score.
    pub fn near(mut self, account_id: u64, higher: u32, lower: u32) -> Self {
        self.account_id = Some(account_id);
        self.near_higher = Some(higher);
        self.near_lower = Some(lower);
        self
    }

    /// The query string pairs, in the order the game sends them.
    pub fn pairs(&self) -> Vec<(&'static str, String)> {
        let mut p = Vec::new();
        let mut opt = |k: &'static str, v: Option<String>| {
            if let Some(v) = v {
                p.push((k, v));
            }
        };
        opt("topScores", self.top_scores.map(|v| v.to_string()));
        opt("nearHigher", self.near_higher.map(|v| v.to_string()));
        opt("nearLower", self.near_lower.map(|v| v.to_string()));
        opt("trackHash", self.track_hash.clone());
        opt("songHash", self.song_hash.clone());
        opt("accountId", self.account_id.map(|v| v.to_string()));
        p.push(("instrument", self.instrument.clone()));
        p.push(("difficulty", self.difficulty.clone()));
        let mut opt = |k: &'static str, v: Option<String>| {
            if let Some(v) = v {
                p.push((k, v));
            }
        };
        opt("minSpeed", self.min_speed.map(|v| v.to_string()));
        opt("maxSpeed", self.max_speed.map(|v| v.to_string()));
        opt("controllerType", self.controller_type.clone());
        opt("sort", self.sort.clone());
        p.extend(self.controller_types.iter().map(|c| ("controllerTypes", c.clone())));
        p.extend(self.modifiers.iter().map(|m| ("modifiers", m.clone())));
        p
    }
}

/// Filters for `/account/{id}/scores`.
#[derive(Debug, Clone, PartialEq)]
pub struct AccountScoresQuery {
    /// From 1. Past the last page the server answers with no rows.
    pub page: u32,
    pub instrument: Option<String>,
    pub difficulty: Option<String>,
    /// Only full combos.
    pub fc_only: bool,
    /// Only the best score per chart (per track: instrument and difficulty apart).
    pub best_per_chart: bool,
    /// Matches song name or artist.
    pub search: Option<String>,
}

impl Default for AccountScoresQuery {
    fn default() -> Self {
        AccountScoresQuery { page: 1, instrument: None, difficulty: None, fc_only: false, best_per_chart: false, search: None }
    }
}

impl AccountScoresQuery {
    pub fn pairs(&self) -> Vec<(&'static str, String)> {
        let mut p = vec![("page", self.page.max(1).to_string())];
        if let Some(i) = &self.instrument {
            p.push(("instrument", i.clone()));
        }
        if let Some(d) = &self.difficulty {
            p.push(("difficulty", d.clone()));
        }
        if self.fc_only {
            p.push(("fcOnly", "true".into()));
        }
        if self.best_per_chart {
            p.push(("bestPerChart", "true".into()));
        }
        if let Some(s) = self.search.as_ref().filter(|s| !s.is_empty()) {
            p.push(("search", s.clone()));
        }
        p
    }
}

// ---------------------------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------------------------

/// The API, over a `reqwest::Client` you set up (TLS, user agent, timeouts). Cheap to clone.
#[derive(Debug, Clone)]
pub struct Api {
    http: reqwest::Client,
    base: String,
}

impl Api {
    pub fn new(http: reqwest::Client) -> Self {
        Api { http, base: API_URL.to_string() }
    }

    /// Another base address (a mirror or a local test server), without a trailing slash.
    pub fn with_base(http: reqwest::Client, base: impl Into<String>) -> Self {
        Api { http, base: base.into().trim_end_matches('/').to_string() }
    }

    async fn get<T: DeserializeOwned>(&self, path: &str, query: &[(&str, String)]) -> Result<T, String> {
        let resp = self
            .http
            .get(format!("{}{path}", self.base))
            .query(query)
            .send()
            .await
            .map_err(|e| format!("Couldn't reach Clone Hero's leaderboards: {e}"))?;
        let status = resp.status();
        let body = resp.bytes().await.map_err(|e| format!("Couldn't reach Clone Hero's leaderboards: {e}"))?;
        if !status.is_success() {
            return Err(status_error(status, &body));
        }
        parse(&body)
    }

    /// `GET /scores`: a board's top rows, and optionally the rows around one account.
    pub async fn scores(&self, q: &ScoresQuery) -> Result<Scores, String> {
        self.get("/scores", &q.pairs()).await
    }

    /// `GET /scores/all-scores`: up to 100 scores on a track for one controller and speed, best
    /// first, in brief. The game uses it for in-song comparisons.
    pub async fn all_scores(&self, track_hash: &str, controller_type: &str, speed: u32) -> Result<Vec<BriefScore>, String> {
        let q = [("trackHash", track_hash.to_string()), ("controllerType", controller_type.to_string()), ("speed", speed.to_string())];
        self.get("/scores/all-scores", &q).await
    }

    /// `GET /scores/filters`: which instrument/difficulty/controller/speed/modifier combinations
    /// have scores, for a song or a single track.
    pub async fn filters(&self, song_hash: Option<&str>, track_hash: Option<&str>) -> Result<Filters, String> {
        let mut q = Vec::new();
        if let Some(h) = song_hash {
            q.push(("songHash", h.to_string()));
        }
        if let Some(h) = track_hash {
            q.push(("trackHash", h.to_string()));
        }
        self.get("/scores/filters", &q).await
    }

    /// `GET /scores/graphs/duration-stats`: where in the song players miss, fail and drop combos.
    /// Uses the query's hash, instrument, difficulty and modifiers.
    pub async fn duration_stats(&self, q: &ScoresQuery) -> Result<DurationStats, String> {
        let mut p = Vec::new();
        if let Some(h) = &q.song_hash {
            p.push(("songHash", h.clone()));
        }
        if let Some(h) = &q.track_hash {
            p.push(("trackHash", h.clone()));
        }
        p.push(("instrument", q.instrument.clone()));
        p.push(("difficulty", q.difficulty.clone()));
        p.extend(q.modifiers.iter().map(|m| ("modifiers", m.clone())));
        self.get("/scores/graphs/duration-stats", &p).await
    }

    /// `GET /scores/graphs/record-progression`: the board's record over time, with the same
    /// filters as [`Api::scores`].
    pub async fn record_progression(&self, q: &ScoresQuery) -> Result<RecordProgression, String> {
        self.get("/scores/graphs/record-progression", &q.pairs()).await
    }

    /// `GET /scores/latest-scores`: the 100 newest scores site-wide.
    pub async fn latest_scores(&self) -> Result<Vec<Score>, String> {
        self.get("/scores/latest-scores", &[]).await
    }

    /// `GET /stats/player-leaderboard`: the top 10 players by a metric (see [`PLAYER_METRICS`]).
    pub async fn player_leaderboard(&self, metric: &str) -> Result<Vec<PlayerRank>, String> {
        self.get("/stats/player-leaderboard", &[("metric", metric.to_string())]).await
    }

    /// `GET /stats/song-leaderboard`: the 100 most played charts (see [`SONG_TABS`],
    /// [`CONTROLLER_GROUPS`]).
    pub async fn song_leaderboard(&self, tab: &str, controller_group: &str) -> Result<Vec<SongRank>, String> {
        self.get("/stats/song-leaderboard", &[("tab", tab.to_string()), ("controllerGroup", controller_group.to_string())]).await
    }

    /// `GET /stats/site-totals`.
    pub async fn site_totals(&self) -> Result<SiteTotals, String> {
        self.get("/stats/site-totals", &[]).await
    }

    /// `GET /account/{id}/profile`.
    pub async fn profile(&self, account_id: u64) -> Result<Profile, String> {
        self.get(&format!("/account/{account_id}/profile"), &[]).await
    }

    /// `GET /account/{id}/scores`: one page of an account's scores, newest first.
    pub async fn account_scores(&self, account_id: u64, q: &AccountScoresQuery) -> Result<AccountScores, String> {
        self.get(&format!("/account/{account_id}/scores"), &q.pairs()).await
    }

    /// Every page of [`Api::account_scores`] (from `q.page`), up to `max_pages`, one request at
    /// a time.
    pub async fn account_scores_all(&self, account_id: u64, q: &AccountScoresQuery, max_pages: u32) -> Result<Vec<Score>, String> {
        let mut q = q.clone();
        let mut all = Vec::new();
        for _ in 0..max_pages {
            let p = self.account_scores(account_id, &q).await?;
            let last = p.is_last_page();
            all.extend(p.scores);
            if last {
                break;
            }
            q.page += 1;
        }
        Ok(all)
    }

    /// `GET /search/quick`: up to 5 charts and 5 players.
    pub async fn quick_search(&self, text: &str) -> Result<QuickSearch, String> {
        self.get("/search/quick", &[("q", text.to_string())]).await
    }

    /// `GET /search`: charts and players, each paged on its own (`per_page` rows).
    pub async fn search(&self, text: &str, songs_page: u32, users_page: u32, per_page: u32) -> Result<Search, String> {
        let q = [
            ("q", text.to_string()),
            ("songsPage", songs_page.to_string()),
            ("usersPage", users_page.to_string()),
            ("perPage", per_page.to_string()),
        ];
        self.get("/search", &q).await
    }

    /// `GET /site-settings/announcement-banner`.
    pub async fn banner(&self) -> Result<Banner, String> {
        self.get("/site-settings/announcement-banner", &[]).await
    }
}

/// A reply body as `T`.
pub fn parse<T: DeserializeOwned>(body: &[u8]) -> Result<T, String> {
    serde_json::from_slice(body).map_err(|e| format!("Unexpected answer from Clone Hero's leaderboards: {e}"))
}

/// The message for an error reply: the server's own when it sent one.
fn status_error(status: reqwest::StatusCode, body: &[u8]) -> String {
    match serde_json::from_slice::<ApiError>(body) {
        Ok(e) if !e.message.is_empty() => {
            // Validation errors start with a line about "request.query"; the useful part follows.
            let msg = e.message.lines().filter(|l| !l.starts_with("Bad request on parameter")).collect::<Vec<_>>().join("; ");
            format!("Clone Hero's leaderboards answered {status}: {msg}")
        }
        _ => format!("Clone Hero's leaderboards answered {status}"),
    }
}

/// Links to the leaderboard website.
pub mod site {
    use super::SITE_URL;

    /// A song's page, opened on one board.
    pub fn song_page(song_hash: &str, instrument: &str, difficulty: &str, speed: u32, sort: &str) -> String {
        format!(
            "{SITE_URL}/scores/{song_hash}?instrument={instrument}&difficulty={difficulty}&minSpeed={speed}&maxSpeed={speed}&sort={sort}"
        )
    }

    /// A player's page.
    pub fn player_page(account_id: u64) -> String {
        format!("{SITE_URL}/users/{account_id}")
    }
}

#[cfg(test)]
mod tests;
