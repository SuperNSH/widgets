use nsh_widgets::{Store, WidgetError, store};
use serde_json::{Value, json};

fn temp() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::new(dir.path());
    (dir, store)
}

#[test]
fn data_starts_empty_and_round_trips_in_an_envelope() {
    let (dir, store) = temp();
    assert_eq!(store.data("Counter").unwrap(), json!({}));
    store.set_data("Counter", &json!({ "count": 3 })).unwrap();
    assert_eq!(store.data("Counter").unwrap(), json!({ "count": 3 }));
    let raw: Value =
        serde_json::from_str(&std::fs::read_to_string(dir.path().join("nsh/widgets/Counter/data.json")).unwrap())
            .unwrap();
    assert_eq!(raw, json!({ "schema": 1, "data": { "count": 3 } }));
    assert!(!dir.path().join("nsh/widgets/Counter/data.json.tmp").exists());
}

#[test]
fn update_keeps_what_the_widget_changed() {
    let (_dir, store) = temp();
    store.set_data("Tasks", &json!({ "t1": false, "label": "a" })).unwrap();
    store.apply("Tasks", "toggle:t1").unwrap();
    let data = store.update_data("Tasks", &json!({ "label": "b" })).unwrap();
    assert_eq!(data, json!({ "t1": true, "label": "b" }));
}

#[test]
fn data_must_be_an_object() {
    let (_dir, store) = temp();
    assert_eq!(
        store.set_data("Counter", &json!([1])).unwrap_err().code(),
        "invalid_json"
    );
    assert_eq!(
        store.update_data("Counter", &json!(1)).unwrap_err().code(),
        "invalid_json"
    );
}

#[test]
fn kinds_are_checked() {
    let (_dir, store) = temp();
    for bad in ["", "1abc", "../x", "a/b", "a b"] {
        assert!(matches!(store.data(bad), Err(WidgetError::InvalidKind(_))), "{bad}");
    }
    assert!(store::check_kind("Daily_steps-2").is_ok());
}

#[test]
fn actions_run_the_vocabulary_and_queue_events_once() {
    let (_dir, store) = temp();
    store.apply("Counter", "inc:count").unwrap();
    store.apply("Counter", "inc:count").unwrap();
    assert_eq!(store.apply("Counter", "dec:count").unwrap(), json!({ "count": 1 }));
    assert_eq!(store.apply("Counter", "toggle:done").unwrap()["done"], json!(true));
    let events = store.take_actions().unwrap();
    let actions: Vec<_> = events.iter().map(|e| e["action"].as_str().unwrap()).collect();
    assert_eq!(actions, ["inc:count", "inc:count", "dec:count", "toggle:done"]);
    assert!(events.iter().all(|e| e["kind"] == "Counter" && e["id"].is_string()));
    assert!(store.take_actions().unwrap().is_empty());
}

#[test]
fn unknown_actions_are_refused() {
    let (_dir, store) = temp();
    for bad in ["inc", "mul:x", "inc:", "inc:1x", "toggle:a b"] {
        assert_eq!(
            store.apply("Counter", bad).unwrap_err().code(),
            "invalid_action",
            "{bad}"
        );
    }
}

#[test]
fn snapshots_are_checked_before_they_are_written() {
    let (_dir, store) = temp();
    let good = json!({ "v": 1, "root": { "t": "col", "children": [
        { "t": "text", "text": "{count}" },
        { "t": "button", "action": "inc:count", "text": "+1" },
        { "t": "button", "action": "open:app://home", "text": "Open" }
    ]}});
    store.set_ui("Counter", &good).unwrap();
    assert_eq!(store.snapshot("Counter").unwrap(), Some(good));
    let bad = [
        json!({ "v": 2, "root": { "t": "col" } }),
        json!({ "v": 1 }),
        json!({ "v": 1, "root": { "t": "canvas" } }),
        json!({ "v": 1, "root": { "t": "col", "children": [{ "t": "button", "action": "rm:x" }] } }),
    ];
    for snapshot in bad {
        assert!(store.set_ui("Counter", &snapshot).is_err(), "{snapshot}");
    }
}

#[test]
fn the_change_token_moves_when_data_is_written() {
    let (_dir, store) = temp();
    assert_eq!(store.change_token("Counter"), 0);
    store.set_data("Counter", &json!({ "count": 1 })).unwrap();
    let first = store.change_token("Counter");
    assert_ne!(first, 0);
    std::thread::sleep(std::time::Duration::from_millis(20));
    store.apply("Counter", "inc:count").unwrap();
    assert_ne!(store.change_token("Counter"), first);
}

#[test]
fn the_lock_screen_tree_is_checked_too() {
    let (_dir, store) = temp();
    let root = json!({ "t": "text", "text": "a" });
    store
        .set_ui(
            "Steps",
            &json!({ "v": 1, "root": root, "accessory": { "t": "text", "text": "{steps}" } }),
        )
        .unwrap();
    let bad = json!({ "v": 1, "root": root, "accessory": { "t": "canvas" } });
    assert!(store.set_ui("Steps", &bad).is_err());
}
