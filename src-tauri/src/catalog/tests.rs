use std::fs;
use std::sync::{Arc, Mutex};
use std::thread;

use tempfile::TempDir;
use tiny_http::{Header, Response, Server};

use super::*;

fn catalog_json(games: &[(&str, &str, u32)]) -> String {
    let games: Vec<String> = games
        .iter()
        .map(|(id, name, year)| {
            format!(
                r#"{{"id":"{id}","name":"{name}","japan_release_year":{year},"repository":"To-Recomp/{id}",
                   "futureField":true,
                   "platforms":[{{"platform":"GameCube","regions":["France (PAL)"],"state":"playable","maturity":"beta"}}]}}"#
            )
        })
        .collect();
    format!(
        r#"{{"format":1,"generated":"2026-11-22T12:00:00Z","games":[{}]}}"#,
        games.join(",")
    )
}

struct Reply {
    status: u16,
    etag: Option<&'static str>,
    body: String,
}

fn reply(status: u16, etag: Option<&'static str>, body: &str) -> Reply {
    Reply {
        status,
        etag,
        body: body.to_string(),
    }
}

/// A fake catalog server answering the scripted replies in order, then recording what it was
/// asked (the `If-None-Match` of each request).
struct Fake {
    url: String,
    seen: Arc<Mutex<Vec<Option<String>>>>,
}

fn serve(replies: Vec<Reply>) -> Fake {
    let server = Server::http("127.0.0.1:0").unwrap();
    let url = format!(
        "http://{}/catalog.json",
        server.server_addr().to_ip().unwrap()
    );
    let seen = Arc::new(Mutex::new(Vec::new()));
    let log = Arc::clone(&seen);
    thread::spawn(move || {
        for reply in replies {
            let Ok(request) = server.recv() else { return };
            let asked = request
                .headers()
                .iter()
                .find(|h| h.field.equiv("If-None-Match"))
                .map(|h| h.value.to_string());
            log.lock().unwrap().push(asked);
            let mut response = Response::from_string(reply.body).with_status_code(reply.status);
            if let Some(etag) = reply.etag {
                response = response.with_header(Header::from_bytes("ETag", etag).unwrap());
            }
            let _ = request.respond(response);
        }
    });
    Fake { url, seen }
}

/// A URL nobody listens on.
fn dead_url() -> String {
    let server = Server::http("127.0.0.1:0").unwrap();
    let url = format!(
        "http://{}/catalog.json",
        server.server_addr().to_ip().unwrap()
    );
    drop(server);
    url
}

fn ids(catalog: &Catalog) -> Vec<&str> {
    catalog.games.iter().map(|g| g.id.as_str()).collect()
}

#[test]
fn downloads_validates_orders_and_keeps_a_copy() {
    let dir = TempDir::new().unwrap();
    let body = catalog_json(&[
        ("tov", "Tales of Vesperia", 2008),
        ("tos", "Tales of Symphonia", 2003),
    ]);
    let fake = serve(vec![reply(200, Some("\"v1\""), &body)]);

    let result = refresh(dir.path(), &fake.url);

    assert!(result.updated && result.error.is_none());
    assert_eq!(
        ids(result.catalog.as_ref().unwrap()),
        ["tos", "tov"],
        "ordered by Japanese release year"
    );
    assert_eq!(
        fs::read_to_string(dir.path().join("catalog.json")).unwrap(),
        body
    );
    assert_eq!(
        fs::read_to_string(dir.path().join("etag")).unwrap(),
        "\"v1\""
    );
    assert_eq!(ids(&load_cached(dir.path()).unwrap()), ["tos", "tov"]);
}

#[test]
fn asks_conditionally_and_keeps_the_copy_on_304() {
    let dir = TempDir::new().unwrap();
    let body = catalog_json(&[("tos", "Tales of Symphonia", 2003)]);
    let fake = serve(vec![
        reply(200, Some("\"v1\""), &body),
        reply(304, None, ""),
    ]);

    refresh(dir.path(), &fake.url);
    let result = refresh(dir.path(), &fake.url);

    assert!(!result.updated && result.error.is_none());
    assert_eq!(ids(result.catalog.as_ref().unwrap()), ["tos"]);
    assert_eq!(
        *fake.seen.lock().unwrap(),
        vec![None, Some("\"v1\"".to_string())]
    );
}

#[test]
fn a_changed_catalog_replaces_the_copy() {
    let dir = TempDir::new().unwrap();
    let fake = serve(vec![
        reply(
            200,
            Some("\"v1\""),
            &catalog_json(&[("tos", "Tales of Symphonia", 2003)]),
        ),
        reply(
            200,
            Some("\"v2\""),
            &catalog_json(&[
                ("tos", "Tales of Symphonia", 2003),
                ("toe", "Tales of Eternia", 2000),
            ]),
        ),
    ]);
    refresh(dir.path(), &fake.url);
    let result = refresh(dir.path(), &fake.url);
    assert!(result.updated);
    assert_eq!(ids(&load_cached(dir.path()).unwrap()), ["toe", "tos"]);
    assert_eq!(
        fs::read_to_string(dir.path().join("etag")).unwrap(),
        "\"v2\""
    );
}

#[test]
fn offline_uses_the_saved_copy_and_says_why() {
    let dir = TempDir::new().unwrap();
    let fake = serve(vec![reply(
        200,
        Some("\"v1\""),
        &catalog_json(&[("tos", "Tales of Symphonia", 2003)]),
    )]);
    refresh(dir.path(), &fake.url);

    let result = refresh(dir.path(), &dead_url());
    assert!(!result.updated);
    assert_eq!(ids(result.catalog.as_ref().unwrap()), ["tos"]);
    assert!(result.error.unwrap().contains("Can't reach"));
}

#[test]
fn offline_without_a_saved_copy_has_no_catalog() {
    let dir = TempDir::new().unwrap();
    let result = refresh(dir.path(), &dead_url());
    assert_eq!(result.catalog, None);
    assert!(result.error.unwrap().contains("Can't reach"));
}

#[test]
fn an_invalid_catalog_is_rejected_and_the_valid_copy_kept() {
    let good = catalog_json(&[("tos", "Tales of Symphonia", 2003)]);
    let invalid = [
        ("not json", "isn't valid JSON"),
        (
            r#"{"format":2,"generated":"2026-11-22T12:00:00Z","games":[]}"#,
            "format 1",
        ),
        (
            r#"{"format":1,"generated":"2026-11-22T12:00:00Z","games":[{"id":"tos"}]}"#,
            "format 1",
        ),
        (
            r#"{"format":1,"generated":"2026-11-22T12:00:00Z","games":[{"id":"t","name":"T","japan_release_year":2003,"repository":"a/b","platforms":[{"platform":"GC","regions":["EU"],"state":"working"}]}]}"#,
            "/games/0/platforms/0/state",
        ),
        (
            r#"{"format":1,"generated":"2026-11-22T12:00:00Z","games":[{"id":"t","name":"T","japan_release_year":2003,"repository":"a/b","platforms":[{"platform":"GC","regions":["EU"],"state":"playable"}]}]}"#,
            "maturity",
        ),
        (
            r#"{"format":1,"generated":"2026-11-22T12:00:00Z","games":[{"id":"t","name":"T","japan_release_year":2003,"repository":"a/b","platforms":[{"platform":"GC","regions":["EU"],"state":"planned","maturity":"alpha"}]}]}"#,
            "/games/0/platforms/0",
        ),
        (
            r#"{"format":1,"generated":"2026-11-22T12:00:00Z","games":[{"id":"t","name":"T","japan_release_year":2003,"repository":"a/b","platforms":[{"platform":"GC","regions":[],"state":"planned"}]}]}"#,
            "/games/0/platforms/0/regions",
        ),
        (
            &catalog_json(&[("tos", "Tales of Symphonia", 2003), ("tos", "Again", 2004)]),
            "twice",
        ),
        (
            &format!(
                r#"{{"format":1,"generated":"2026-11-22T12:00:00Z","games":[{{"id":"t","name":"T","japan_release_year":2003,"repository":"a/b","platforms":[{p},{p}]}}]}}"#,
                p = r#"{"platform":"GC","regions":["EU"],"state":"planned"}"#
            ),
            "twice",
        ),
    ];
    for (body, expected) in invalid {
        let dir = TempDir::new().unwrap();
        let fake = serve(vec![
            reply(200, Some("\"v1\""), &good),
            reply(200, Some("\"v2\""), body),
        ]);
        refresh(dir.path(), &fake.url);

        let result = refresh(dir.path(), &fake.url);

        assert!(!result.updated, "{body}");
        let error = result.error.unwrap_or_default();
        assert!(error.contains(expected), "{body}: {error}");
        assert_eq!(ids(result.catalog.as_ref().unwrap()), ["tos"], "{body}");
        assert_eq!(
            fs::read_to_string(dir.path().join("catalog.json")).unwrap(),
            good,
            "{body}"
        );
        assert_eq!(
            fs::read_to_string(dir.path().join("etag")).unwrap(),
            "\"v1\"",
            "{body}"
        );
    }
}

#[test]
fn a_server_error_keeps_the_saved_copy() {
    let dir = TempDir::new().unwrap();
    let fake = serve(vec![
        reply(
            200,
            Some("\"v1\""),
            &catalog_json(&[("tos", "Tales of Symphonia", 2003)]),
        ),
        reply(500, None, "boom"),
    ]);
    refresh(dir.path(), &fake.url);
    let result = refresh(dir.path(), &fake.url);
    assert!(result.error.unwrap().contains("500"));
    assert_eq!(ids(result.catalog.as_ref().unwrap()), ["tos"]);
}

#[test]
fn an_oversized_answer_is_refused() {
    let dir = TempDir::new().unwrap();
    let huge = format!(
        "{{\"format\":1,\"pad\":\"{}\"}}",
        "x".repeat(MAX_BYTES as usize + 10)
    );
    let fake = serve(vec![reply(200, None, &huge)]);
    let result = refresh(dir.path(), &fake.url);
    assert_eq!(result.catalog, None);
    assert!(result.error.is_some());
    assert!(!dir.path().join("catalog.json").exists());
}

#[test]
fn a_damaged_saved_copy_counts_as_none() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("catalog.json"), "{ truncated").unwrap();
    assert_eq!(load_cached(dir.path()), None);
}

#[test]
fn an_empty_catalog_is_valid_and_unknown_fields_are_ignored() {
    assert!(
        parse(r#"{"format":1,"generated":"2026-11-22T12:00:00Z","games":[],"x":1}"#)
            .unwrap()
            .games
            .is_empty()
    );
    let catalog = parse(&catalog_json(&[("tos", "Tales of Symphonia", 2003)])).unwrap();
    assert_eq!(catalog.games[0].platforms[0].state, State::Playable);
    assert_eq!(catalog.games[0].platforms[0].maturity, Some(Maturity::Beta));
}

#[test]
fn the_other_states_are_valid_and_keep_their_wire_names() {
    let raw = r#"{"format":1,"generated":"2026-11-22T12:00:00Z","games":[{"id":"t","name":"T","japan_release_year":2003,"repository":"a/b","platforms":[
        {"platform":"PS2","regions":["Japan","USA"],"state":"planned"},
        {"platform":"PS3","regions":["Japan"],"state":"in-progress"},
        {"platform":"Wii","regions":["Japan"],"state":"unsupported"}]}]}"#;
    let catalog = parse(raw).unwrap();
    let states: Vec<State> = catalog.games[0].platforms.iter().map(|p| p.state).collect();
    assert_eq!(
        states,
        [State::Planned, State::InProgress, State::Unsupported]
    );
    assert_eq!(catalog.games[0].platforms[0].maturity, None);
    let json = serde_json::to_value(&catalog).unwrap();
    let wire: Vec<&str> = (0..3)
        .map(|i| json["games"][0]["platforms"][i]["state"].as_str().unwrap())
        .collect();
    assert_eq!(wire, ["planned", "in-progress", "unsupported"]);
}

#[test]
fn json_wire_format_matches_the_frontend_types() {
    let catalog = parse(&catalog_json(&[("tos", "Tales of Symphonia", 2003)])).unwrap();
    let json = serde_json::to_value(Refresh {
        catalog: Some(catalog),
        updated: true,
        error: None,
    })
    .unwrap();
    assert_eq!(
        json,
        serde_json::json!({
            "catalog": { "games": [{
                "id": "tos", "name": "Tales of Symphonia", "japanReleaseYear": 2003,
                "repository": "To-Recomp/tos",
                "platforms": [{ "platform": "GameCube", "regions": ["France (PAL)"], "state": "playable", "maturity": "beta", "releaseYear": null }]
            }] },
            "updated": true,
            "error": null
        })
    );
}

#[test]
fn a_console_may_say_the_year_the_game_came_out_on_it() {
    let raw = r#"{"format":1,"generated":"2026-11-22T12:00:00Z","games":[{"id":"t","name":"T","japan_release_year":2003,"repository":"a/b","platforms":[
        {"platform":"GC","regions":["EU"],"state":"planned","release_year":2003},
        {"platform":"PS3","regions":["EU"],"state":"planned"}]}]}"#;
    let years: Vec<Option<u32>> = parse(raw).unwrap().games[0]
        .platforms
        .iter()
        .map(|p| p.release_year)
        .collect();
    assert_eq!(years, [Some(2003), None]);

    let too_early = raw.replace("\"release_year\":2003", "\"release_year\":1800");
    assert!(parse(&too_early).is_err());
}
