# Clone Hero leaderboard API (unofficial notes)

> **Unofficial.** Clone Hero's developers haven't published this API. These notes were put
> together from the leaderboard website's own JavaScript (<https://leaderboards.clonehero.net>),
> the requests the game writes to its log (`Player.log`), and the server's replies, in
> November 2026. It can change at any time without notice. Be gentle with it: a few requests at a
> time, with pauses, and cache what you can.

Base address: `https://api.clonehero.net/v1` (the website also uses `/v2`, but only for sign-in).

Each endpoint is marked:

- **verified**: called and the reply seen (with the parameters listed as verified);
- **site code only**: seen in the website's code, not called.

All the public reads below were verified. The client in `src/lib.rs` has one function for each.

## Conventions

- **Method**: every endpoint here is `GET` and needs no sign-in. The website sends
  `Authorization: Bearer <session token>` when someone is signed in; the public reads don't need it.
- **Replies** are JSON (`application/json; charset=utf-8`), fields in camelCase.
- **Times** are ISO 8601 UTC strings (`"2026-11-07T23:03:12.834Z"`). **Durations** are
  milliseconds. **Accuracy** is a string with one decimal (`"99.8"`).
- **Hashes** are 32 bytes of URL-safe base64 without padding: exactly 43 characters matching
  `^[A-Za-z0-9\-_]{43}$`.
  - `trackHash`: one instrument + difficulty of one exact chart version (scan-chart's track
    hash). What the game asks with.
  - `songHash`: the chart as a whole. What the website asks with (its pages are
    `/scores/{songHash}`).
- **Lists in the query** are sent by repeating the key (`controllerTypes=5Fret&controllerTypes=Gamepad`).
- **Unknown query parameters are ignored** (no error).
- **Booleans** in the query must be `true` or `false`.

### Allowed values

These come from the server's own validation errors (verified), except modifiers (site code).

| Name | Values |
|---|---|
| `instrument` | `guitar`, `guitarcoop`, `rhythm`, `bass`, `drums`, `prodrums`, `keys`, `guitarghl`, `guitarcoopghl`, `rhythmghl`, `bassghl`, `keysghl` |
| `difficulty` | `expert`, `hard`, `medium`, `easy` |
| `controllerType` | `5Fret`, `6Fret`, `7LaneDrums` (the site's "4 Lane"), `5LaneDrums`, `Gamepad` |
| `sort` | `score`, `accuracy` |
| `modifiers` (site code) | `allHOPOs`, `allStrums`, `allTaps`, `brutalMode`, `deadlyDynamics`, `doubleKick`, `doubleNotes`, `droplessSustains`, `HOPOsToTaps`, `kickOnly`, `lightsOut`, `mirrorMode`, `modFull`, `modLite`, `modPrep`, `noFretGhosting`, `noKick`, `noteShuffle`, `precisionMode`, `strumlessHopos` |
| speeds | percent, 5–5000 (100 = normal) |

`drums` and `prodrums` are separate boards on the same drum track. The website picks a default
controller per instrument: `7LaneDrums` for drums, `6Fret` for the `…ghl` instruments, otherwise
`5Fret`.

### Errors

Errors come back with a JSON body (verified):

```json
{"name": "AJV_VALIDATION_ERROR",
 "message": "Bad request on parameter \"request.query\".\nGetPlayerLeaderboardModel.metric must be equal to one of the allowed values. Given value: \"x\"",
 "status": 400,
 "errors": [{"dataPath": ".metric", "message": "must be equal to one of the allowed values",
             "params": {"allowedValues": ["totalScore", "submissions", "…"]}, "…": "…"}]}
```

| Status | `name` | When |
|---|---|---|
| 400 | `AJV_VALIDATION_ERROR` | A parameter is missing, of the wrong type, out of range or not an allowed value. `errors[]` has one entry per problem, with `params.allowedValues`, `params.limit` or `params.missingProperty`. |
| 400 | (plain message) | `/scores` and friends with neither `songHash` nor `trackHash`: `"Either songHash or trackHash must be defined."` |
| 401 | `UNAUTHORIZED` | A signed-in endpoint without a token: `"Authorization header not provided."` |
| 404 | `NOT_FOUND` | `/account/{id}/…` for an account that doesn't exist: `"Account with id \"…\" not found."` |

No rate-limit headers were seen; that doesn't mean there's no limit.

### The score object

Most endpoints return scores in the same shape (verified on `/scores`, `/scores/latest-scores`,
record progression, profiles and account pages). Trimmed:

```json
{"placement": 1,
 "scoreId": 4835932,
 "trackHash": "-Nku724NgARbyAgbfSHvqhPBeskhr5rF-m_FTuv9B5s",
 "songHash": "6dMOF705iGBmAXpFuEISH4gGHKG9yCkQFCTJLLjx4AI",
 "songCreatedAt": "2026-11-02T20:25:15.025Z",
 "name": "Enter Sandman", "artist": "Metallica", "album": "Metallica", "genre": "Thrash Metal",
 "year": "1991", "charter": "oBLINKxONCEo", "icon": "oblinkxonceo", "duration": 334596,
 "createdAt": "2026-11-07T23:03:12.834Z",
 "accountId": 81, "username": "21st Century Guy",
 "controllerType": "5Fret", "instrument": "guitar", "difficulty": "expert",
 "modifiers": [], "speed": 100,
 "isFc": true, "isPfc": true, "hasPreviousFc": true, "hasPreviousPfc": true, "failedInRun": false,
 "baseScore": 81467, "totalScore": 426378, "soloScore": 28700, "noteScore": 76350,
 "ghostScore": 2596, "sustainScore": 5309, "comboScore": 241687, "stars": 7,
 "spScore": 71736, "spPhraseCount": 13, "spPhrasesHit": 13, "timeInSp": 61452,
 "notesHit": 1298, "noteCount": 1298, "accuracy": "100.0", "maxCombo": 1298,
 "excessHits": 0, "ghostInputs": 0, "ghostsHit": 0, "ghostNoteCount": 0,
 "accentsHit": 0, "accentNoteCount": 0, "squeezes": 0}
```

- Always present in every reply seen: every field above except `placement`.
- `placement` (1 = best) is only on `/scores` rows (`top` and `near`).
- `ghostsHit` / `ghostNoteCount` / `accentsHit` / `accentNoteCount` are drum dynamics; 0 elsewhere.
- `songCreatedAt` is when the server first saw the chart, `createdAt` when the score was sent.
- `hasPreviousFc` / `hasPreviousPfc`: the player had already full-comboed this track before.
- Older Truss notes say `accuracy` was missing on some runs; the client treats it as optional.

---

## Scores

### `GET /scores`: one board — **verified**

A board's top rows, optionally with the rows around one account's score. The game calls it for
the in-game leaderboard; the website for every song page.

| Parameter | Type | Required | Default | Notes |
|---|---|---|---|---|
| `trackHash` | hash | one of these two | – | What the game sends. |
| `songHash` | hash | one of these two | – | What the website sends; also fills `summary.song`. |
| `instrument` | enum | yes | – | |
| `difficulty` | enum | yes | – | |
| `topScores` | int 1–500 | no | **3** (observed) | Rows from the top. The website asks 50, the game 9. |
| `minSpeed`, `maxSpeed` | int 5–5000 | no | no filter (observed) | The website and game send both; same value = one speed. The game sends `minSpeed=101&maxSpeed=5000` for "any faster". |
| `controllerType` | enum | no | no filter (observed) | One controller (the game). |
| `controllerTypes` | enum, repeated | no | no filter | Several (the website). |
| `modifiers` | enum, repeated | no | none | Only scores with these modifiers (verified with `allTaps`). |
| `sort` | `score`/`accuracy` | no | `score` | Verified both. |
| `accountId` | int | no | – | Fill `near` for this account. |
| `nearHigher`, `nearLower` | int ≥ 0 | no | – | Rows above / below that account's score (the game sends 2 and 2). No upper limit reported. |
| `mySubmissions` | `true` | site code only | – | Signed-in only; out of scope. |

The game's exact request (from `Player.log`):

```
/v1/scores?topScores=9&nearHigher=2&nearLower=2&trackHash=…&accountId=…&instrument=guitar&difficulty=expert&minSpeed=100&maxSpeed=100&controllerType=5Fret&sort=score
```

Reply (trimmed):

```json
{"summary": {"song": null, "submissionCount": 142980, "totalPlayCount": 200330,
             "totalPlayerCount": 5241, "heroBannerKey": "Classic"},
 "top":  {"positionOffset": 0,   "scoreData": [{"placement": 1, "…": "…"}]},
 "near": {"positionOffset": 227, "scoreData": [{"placement": 228, "…": "…"},
                                               {"placement": 230, "username": "<your name>", "…": "…"}]}}
```

- `summary.submissionCount` counts scores on the board *as filtered*; `totalPlayCount` and
  `totalPlayerCount` cover the track over every filter (verified: the first changes with
  `controllerType`, the others don't).
- `summary.song` is `null` when asked by `trackHash`; asked by `songHash` it's
  `{songHash, trackHash, songCreatedAt, name, artist, album, genre, year, charter, icon, duration, metadataApprovalStatus}`.
- `summary.heroBannerKey`: banner art of the board's leader (may be absent; treat as optional).
- `near` is **`null`** unless `accountId` is given, that account has a score on the board, *and*
  it isn't already within `top`. Its `positionOffset` is the number of places before its first row.
- **Paging**: none. One request gives up to 500 top rows plus the `near` window; to find your
  rank, use `accountId` and read your row's `placement`.

### `GET /scores/all-scores`: brief scores for one track — **verified**

What the game fetches when a song starts, for its in-song comparison.

| Parameter | Type | Required | Notes |
|---|---|---|---|
| `trackHash` | hash | yes | |
| `controllerType` | enum | yes | |
| `speed` | int ≥ 5 | yes | One speed. |

Reply: an array, best score first, **100 rows** on a busy track (looks like a cap):

```json
[{"username": "GiometriQ", "isFc": true, "isPfc": true, "hasPreviousFc": true,
  "hasPreviousPfc": true, "failedInRun": false, "totalScore": 55554, "stars": 7,
  "notesHit": 161, "noteCount": 161, "accuracy": "100.0", "maxCombo": 161, "excessHits": 0,
  "ghostInputs": 0, "ghostsHit": 0, "ghostNoteCount": 0, "accentsHit": 0, "accentNoteCount": 0}]
```

No account ids, chart details or placements. No paging.

### `GET /scores/filters`: which boards have scores — **verified**

| Parameter | Type | Required | Notes |
|---|---|---|---|
| `songHash` | hash | one of these two | Every track of the chart. |
| `trackHash` | hash | one of these two | One track. |

Neither gives 400 `"Either songHash or trackHash must be defined."`. Reply:

```json
{"states": [
  {"instrument": "guitar", "difficulty": "expert", "controllerType": "5Fret",   "speed": 100, "modifiers": []},
  {"instrument": "guitar", "difficulty": "expert", "controllerType": "Gamepad", "speed": 100, "modifiers": ["allTaps"]}]}
```

One entry per combination that has at least one score. No paging.

### `GET /scores/graphs/duration-stats`: where players struggle — **verified**

| Parameter | Type | Required | Notes |
|---|---|---|---|
| `songHash` or `trackHash` | hash | one | The website sends `songHash`; `trackHash` works too (verified). |
| `instrument` | enum | yes | |
| `difficulty` | enum | yes | |
| `modifiers` | enum, repeated | no | |

```json
{"totalVisibleScores": 45, "segmentCount": 50,
 "missedNoteCounts": [17, 13, 13, "… 50 numbers"],
 "failedCounts":     [0, 6, 8, "… running total, 50 numbers"],
 "fcStreakCounts":   [28, 24, 23, "… falling, 50 numbers"]}
```

The song is cut into `segmentCount` equal slices. Per slice: notes missed (summed over scores),
scores failed by then (running total), and scores still on a full combo.

### `GET /scores/graphs/record-progression`: the record over time — **verified**

Same parameters as `/scores` (the website sends the same set, `topScores` included).

```json
{"points": [
  {"createdAt": "2026-11-02T20:25:15.025Z", "username": "jojo the hobo", "totalScore": 249306, "score": {"…": "score object"}},
  {"createdAt": "2026-11-04T…", "username": "…", "totalScore": 336911, "score": {"…": "…"}}]}
```

Oldest first, one point per new record. Verified with both `songHash` and `trackHash`.

### `GET /scores/latest-scores`: newest scores site-wide — **verified**

No parameters (a `limit` is ignored). Reply: an array of **100** score objects, newest first,
without `placement`. No paging.

### `GET /scores/latest-user-scores` — out of scope

Needs sign-in (401 `"Authorization header not provided."` without it, verified). The website
sends `limit`.

---

## Stats

### `GET /stats/player-leaderboard`: top players — **verified**

| Parameter | Type | Required | Values |
|---|---|---|---|
| `metric` | enum | yes | `totalScore`, `submissions`, `totalFcs`, `totalPfcs`, `hoursPlayed`, `starsEarned`, `notesHit` |

Reply: the top **10**, best first (verified with `totalScore`):

```json
[{"accountId": 265, "username": "Ydnar445", "value": 3885917451}]
```

No paging. A profile's `ranks` gives any one player's place on each of these metrics.

### `GET /stats/song-leaderboard`: most played charts — **verified**

| Parameter | Type | Required | Values |
|---|---|---|---|
| `tab` | enum | yes | `trending`, `topAllTime`, `topYear`, `topMonth`, `topWeek` |
| `controllerGroup` | enum | yes | `fiveFret`, `sixFret`, `drums` |

Reply: **100** rows, most plays first (verified with `topWeek` + `fiveFret`):

```json
[{"songHash": "24fOWKgXgpl4AVNdDLUFa1b-XhfbYdKAQ6O3cbf14Wo", "name": "Through The Fire & Flames", "artist": "Dragonforce", "plays": 777}]
```

### `GET /stats/site-totals` — **verified**

No parameters.

```json
{"users": 96543, "submissions": 5151816, "songs": 244215, "notesHit": 5745919057,
 "fcs": 465005, "pfcs": 456079, "hoursPlayed": 385565, "totalScore": 1483723524575}
```

---

## Accounts

Only your own account was queried while mapping these. Your account number is in the game's log:
every leaderboard request it makes carries `accountId=<number>`
(`%USERPROFILE%\AppData\LocalLow\srylain Inc_\Clone Hero\Player.log`).

### `GET /account/{id}/profile` — **verified**

No query parameters. Reply (trimmed, with your details left out):

```json
{"accountId": 1000, "username": "<your name>", "createdAt": "…",
 "pronouns": null, "avatarSourceType": "provider", "avatarSourceKey": "discord",
 "profilePictureUrl": "https://cdn.discordapp.com/avatars/…", "titleKey": null,
 "flairKey": "guitar-player", "bannerKey": "Grains", "submissionCount": 1315,
 "stats": {"totalPfcs": 183, "totalFcs": 183, "totalScore": 394639667, "songsPlayed": 793,
           "totalTimePlayedMs": 328150656, "notesHit": 2274236, "totalStarsEarned": 5182, "highestCombo": 4619},
 "ranks": {"totalScore": 203, "submissions": 334, "totalFcs": 338, "totalPfcs": 333,
           "hoursPlayed": 390, "starsEarned": 258, "notesHit": 231},
 "topArtists": [{"artist": "…", "plays": 41}],
 "topInstruments": [{"instrument": "guitar", "plays": 1315}],
 "latestSubmissions": ["… 15 score objects, newest first"],
 "mostPlayed": [{"songHash": "…", "name": "…", "artist": "…", "charter": "…", "plays": 12}],
 "topSubmissions": [],
 "deletedAt": null}
```

- Always present: every key above. Nullable: `pronouns`, `profilePictureUrl`, `titleKey`,
  `flairKey`, `bannerKey`, `deletedAt`.
- `ranks` is the account's place (1 = first) on each `/stats/player-leaderboard` metric. Whether
  a rank can be `null` (say, with no scores) isn't known; treat each as optional.
- `topArtists` / `mostPlayed` had 5 and 15 rows; `latestSubmissions` 15.
- `topSubmissions` was empty; its row shape is assumed to be the score object.
- `avatarSourceType` is `"provider"` (picture from a linked Discord/Google sign-in) or
  `"default"` (with `avatarSourceKey` like `"default-blue"` and no picture URL).
- Unknown account: 404 `NOT_FOUND`.

### `GET /account/{id}/scores`: an account's score history — **verified**

| Parameter | Type | Required | Default | Notes |
|---|---|---|---|---|
| `page` | int ≥ 1 | yes (site always sends it) | – | |
| `instrument` | enum | no | all | |
| `difficulty` | enum | no | all | |
| `fcOnly` | bool | no | `false` | Only full combos. |
| `bestPerChart` | bool | no | `false` | One row per track: its best score. |
| `search` | string | no | – | Matches song name/artist (verified with `dragon`). |

A `pageSize` parameter is ignored. Reply:

```json
{"scores": ["… up to 50 score objects, newest first"], "totalCount": 1315, "page": 1, "pageSize": 50}
```

- **Paging**: 50 rows a page. It's the last page when `page × pageSize ≥ totalCount` (or
  `scores` comes back empty). Past the end the server answers 200 with `"scores": []` and the
  same `totalCount`.
- `totalCount` follows the filters (verified: 1315 all, 793 with `bestPerChart`, 2 with
  `fcOnly` + `search`).

---

## Search and site

### `GET /search/quick` — **verified**

| Parameter | Type | Required |
|---|---|---|
| `q` | string | yes |

Up to 5 charts and 5 players:

```json
{"query": "sandman",
 "songs": [{"songHash": "HrOuPeqOi1hKwYuELSMSsFIo63syfQ4DfYJYK5WwSpo", "name": "Enter Sandman", "artist": "Metallica",
            "charter": "Harmonix", "icon": "rb1", "playCount": 3796, "playerCount": 2103}],
 "users": [{"accountId": 76050, "username": "Sandman", "avatarSourceType": "default",
            "avatarSourceKey": "default-blue", "profilePictureUrl": null}]}
```

### `GET /search` — **verified**

| Parameter | Type | Required | Default (site) |
|---|---|---|---|
| `q` | string | yes | |
| `songsPage` | int | no | 1 |
| `usersPage` | int | no | 1 |
| `perPage` | int | no | 20 |

```json
{"foundSongs": 50, "foundUsers": 6, "nextSongsPage": 2, "nextUsersPage": 2, "query": "sandman",
 "songs": ["… as /search/quick"], "users": ["…"]}
```

Charts and players page separately. `nextSongsPage` / `nextUsersPage` give the next page; they're
expected to be `null` on the last one (not seen). `foundSongs` may be capped (50 seen for "sandman").

### `GET /site-settings/announcement-banner` — **verified**

```json
{"enabled": true, "message": "Be careful with your setup! …"}
```

---

## Out of scope

Seen in the website's code, not called: they need a signed-in session, change things, or are for
moderators and admins.

- `/v1/account/@me`, `/v1/account/@me/settings` (GET/PUT), `/email-change`, `/email-change/confirm`,
  `/email-change/cancel`, `/sessions`, `/sessions/{id}/end`, `/delete`, `/oauth/{provider}/unlink`
- `/v1/scores/latest-user-scores`, and `mySubmissions=true` on `/scores`
- `/v2/auth/…`: `start`, `callback`, `status`, `complete`, `create-account`, `authorization-url`,
  `link/start`, `refresh`, `logout`, `username-available/{name}`
- `/v1/moderation/…` (bans, warnings, reports, logs, stats, evidence images, song uncensoring)
- `/v1/administration/…` (accounts, roles, audit log, announcement banner)

The game itself also opens a "leaderboard session" while you play (its log shows session ids
being started, paused and cancelled); those calls aren't in its log and aren't covered here.
