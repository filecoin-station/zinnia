use std::fs::read_dir;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;
use tempfile::tempdir;

#[test]
pub fn it_removes_lassie_temp_on_start() {
    let _ = env_logger::builder().is_test(true).try_init();

    let temp_root = tempdir().expect("cannot create temporary directory");
    let cache_root = temp_root.path().join("cache");
    let state_root = temp_root.path().join("state");

    let mut mod_js = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    mod_js.push("tests");
    mod_js.push("fixtures");
    mod_js.push("fetch-ipfs-with-delay.js");
    assert!(mod_js.is_file(), "test JS file not found - {:?}", mod_js);

    let bin = assert_cmd::cargo::cargo_bin("zinniad");
    assert!(bin.is_file(), "zinniad not found - {:?}", bin);

    // Create a command to start zinniad
    let mut cmd = Command::new(bin);
    cmd.env("NO_COLOR", "1")
        .env("FIL_WALLET_ADDRESS", "f1test")
        .env("STATION_ID", "a".repeat(88))
        .env("CACHE_ROOT", cache_root.display().to_string())
        .env("STATE_ROOT", state_root.display().to_string())
        .args([&mod_js.as_os_str()])
        .stdout(Stdio::piped());

    if !log::log_enabled!(log::Level::Debug) {
        cmd.stderr(Stdio::null());
    }

    // Start zinniad in background
    let mut child = cmd
        .spawn()
        .unwrap_or_else(|_| panic!("cannot spawn {:?}", cmd.get_program()));
    let mut stdout_lines = BufReader::new(child.stdout.take().expect("cannot take child's stdout"))
        .lines()
        .map(|it| it.expect("cannot read from child's stdout"))
        .inspect(|ln| println!("[zinniad] {}", ln));

    // Wait until our module starts and calls `fetch('ipfs://...')`
    stdout_lines
        .by_ref()
        .take_while(|ln| !ln.contains("fetch:start"))
        .for_each(drop);

    // Wait until Lassie creates its temp file
    loop {
        std::thread::sleep(Duration::from_millis(100));
        let file_count = read_dir(cache_root.join("lassie"))
            .expect("cannot list files in Lassie's temp dir")
            .count();

        if file_count > 0 {
            break;
        }
    }

    // Stop the process
    // Note: on Unix, this sends SIGKILL signal which allows the process to shutdown gracefully
    // If we ever implement graceful shutdown for Lassie, then we may need to rework this line.
    child.kill().expect("cannot stop zinniad");

    // Read the rest of stdout, ensure the retrieval was interrupted and did not finish yet
    assert!(
        !stdout_lines.any(|ln| ln.contains("fetch-end")),
        "zinniad should have been killed before the retrieval finished"
    );

    // Assert there is at least one temp file left over
    let old_files: Vec<_> = read_dir(cache_root.join("lassie"))
        .expect("cannot list files in Lassie's temp dir")
        .map(|it| it.expect("cannot parse directory entry").path())
        .collect();
    assert!(
        !old_files.is_empty(),
        "Lassie should have left some temp files"
    );

    // Run zinniad again
    let mut child = cmd
        .spawn()
        .unwrap_or_else(|_| panic!("cannot spawn {:?}", cmd.get_program()));
    let mut stdout_lines = BufReader::new(child.stdout.take().expect("cannot take child's stdout"))
        .lines()
        .map(|it| it.expect("cannot read from child's stdout"))
        .inspect(|ln| println!("[zinniad] {}", ln));

    // Wait until our module starts and calls `fetch('ipfs://...')`
    stdout_lines
        .by_ref()
        .take_while(|ln| !ln.contains("fetch:start"))
        .for_each(drop);

    // Stop the process
    // Note: on Unix, this sends SIGKILL signal which allows the process to shutdown gracefully
    // If we ever implement graceful shutdown for Lassie, then we may need to rework this line.
    child.kill().expect("cannot stop zinniad");

    // Check that all temp files from the previous run were deleted
    let found: Vec<_> = read_dir(cache_root.join("lassie"))
        .expect("cannot list files in Lassie's temp dir")
        .map(|it| it.expect("cannot parse directory entry").path())
        .filter(|ent| old_files.contains(ent))
        .collect();

    assert_eq!(
        found.len(),
        0,
        "all files from the previous run should have been deleted"
    );
}
