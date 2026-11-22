# clonehero-leaderboard

An unofficial, read-only Rust client for Clone Hero's leaderboard API
(`https://api.clonehero.net/v1`), the server behind <https://leaderboards.clonehero.net> and the
game's in-game leaderboards. Written for [Truss](https://github.com/graindead/truss).

The API isn't documented by Clone Hero's developers; [docs/clonehero-leaderboard-api.md](docs/clonehero-leaderboard-api.md)
describes every public read endpoint as it was mapped (parameters, allowed values, replies,
paging, errors). It can change at any time, so the response types are lenient.

```rust
use clonehero_leaderboard::{Api, ScoresQuery, AccountScoresQuery};

let api = Api::new(reqwest::Client::new()); // bring your own client (TLS, user agent, timeouts)

// Top 50 at normal speed on one track, plus the rows around your score.
let q = ScoresQuery::track(track_hash, "guitar", "expert").speed(100).top(50).near(my_account, 2, 2);
let board = api.scores(&q).await?;

// Your best score on every chart.
let q = AccountScoresQuery { best_per_chart: true, ..Default::default() };
let mine = api.account_scores_all(my_account, &q, 200).await?;
```

Covered: `/scores`, `/scores/all-scores`, `/scores/filters`, `/scores/graphs/duration-stats`,
`/scores/graphs/record-progression`, `/scores/latest-scores`, `/stats/player-leaderboard`,
`/stats/song-leaderboard`, `/stats/site-totals`, `/account/{id}/profile`, `/account/{id}/scores`,
`/search/quick`, `/search`, `/site-settings/announcement-banner`. Nothing that needs signing in.

Errors are `String`s ready to show to a player.

## Tests

`cargo test` parses saved, trimmed replies in `tests/samples/` and runs the client against a
local stand-in server; it doesn't touch the network. A short live check (5 requests) runs with
`CHLB_TEST_LIVE=1` (add `CHLB_TEST_ACCOUNT=<your account number>` for your profile and scores):

```
CHLB_TEST_LIVE=1 cargo test live -- --nocapture
```

Please be gentle with the server: a few requests at a time, and cache what you can.

## License

GPL-3.0, like Truss.
