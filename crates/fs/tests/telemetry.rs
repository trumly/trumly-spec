use serde_json::json;
use trumly_fs::telemetry as t;

// One test: it sets environment variables, which are shared by every test in the process.
#[test]
fn nothing_is_kept_until_you_say_yes_and_off_forgets_everything() {
    let home = std::env::temp_dir().join(format!("trumly-telemetry-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&home);
    unsafe {
        std::env::set_var("TRUMLY_HOME", &home);
    }
    unsafe {
        std::env::remove_var("DO_NOT_TRACK");
    }
    unsafe {
        std::env::remove_var("TRUMLY_TELEMETRY");
    }

    assert_eq!(t::state(), t::State::Unset);
    t::track("check_run", "cli", json!({"errors": "0"}));
    assert_eq!(t::queued_count(), 0, "not asked yet: nothing recorded");
    assert!(t::install_id().is_none());

    t::set_state(t::State::On).unwrap();
    let id = t::install_id().unwrap();
    assert_eq!(id.len(), 32);
    t::track("check_run", "cli", json!({"errors": t::bucket(7)}));
    let q = t::queued(10);
    assert_eq!(q.len(), 1);
    assert_eq!(q[0]["install"], id.as_str());
    assert_eq!(q[0]["props"]["errors"], "1-9");
    assert!(q[0]["ts"].as_str().unwrap().ends_with(":00Z"), "rounded to the minute");

    // the editor's events are only taken in a fixed shape
    assert!(t::record("pack_added", json!({"pack": "system-design-lite"})));
    assert!(!t::record("Pack Added!", json!({})));
    assert!(t::record("node_created", json!({"nested": {"a": 1}})), "odd props are dropped, the event is kept");
    assert_eq!(t::queued(10)[2]["props"], json!({}));

    // sending drops what was sent and no more
    t::ack(1);
    assert_eq!(t::queued_count(), 2);

    // an outside switch always wins
    unsafe {
        std::env::set_var("DO_NOT_TRACK", "1");
    }
    assert_eq!(t::state(), t::State::Off);
    t::track("check_run", "cli", json!({}));
    assert_eq!(t::queued_count(), 2);
    unsafe {
        std::env::remove_var("DO_NOT_TRACK");
    }

    // turning it off forgets the id and the queue
    t::set_state(t::State::Off).unwrap();
    assert!(t::install_id().is_none());
    assert_eq!(t::queued_count(), 0);
    t::track("check_run", "cli", json!({}));
    assert_eq!(t::queued_count(), 0);
    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn counts_are_rounded() {
    assert_eq!(
        [t::bucket(0), t::bucket(9), t::bucket(10), t::bucket(199), t::bucket(200)],
        ["0", "1-9", "10-49", "50-199", "200+"]
    );
}
