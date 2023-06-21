use assert_fs::prelude::*;
use std::fs::read_dir;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::time::Duration;
use tempfile::tempdir;

#[test]
pub fn it_removes_lassie_temp_on_start() {
    let temp_root = tempdir().expect("cannot create temporary directory");
    let cache_root = temp_root.path().join("cache");
    let state_root = temp_root.path().join("state");

    let mod_js = assert_fs::NamedTempFile::new("retriever.js")
        .expect("cannot create temporary retriever.js");
    mod_js
        .write_str(
            r#"
// Signal that we are going to start the retrieval
Zinnia.activity.info("fetch-start");
await fetch('ipfs://QmdmQXB2mzChmMeKY47C43LxUdg1NDJ5MWcKMKxDu7RgQm');
// Signal that the retrieval has finished. If this happens then the test
// did not kill the zinniad process quickly enough.
Zinnia.activity.info("fetch-end");
"#,
        )
        .expect(&format!("cannot write to {}", mod_js.display()));

    let bin = assert_cmd::cargo::cargo_bin("zinniad");
    let bin_str = bin.display().to_string();
    assert!(bin.is_file(), "{} not found", bin_str);

    // Start zinniad in background
    let mut cmd = Command::new(bin);
    cmd.env("NO_COLOR", "1")
        .env("FIL_WALLET_ADDRESS", "f1test")
        .env("CACHE_ROOT", cache_root.display().to_string())
        .env("STATE_ROOT", state_root.display().to_string())
        .args([&mod_js.path().display().to_string()])
        .stdout(Stdio::piped());

    let mut child = cmd.spawn().expect(&format!("cannot spawn {}", bin_str));
    let mut stdout_lines = BufReader::new(child.stdout.take().expect("cannot take child's stdout"))
        .lines()
        .map(|it| it.expect("cannot read from child's stdout"))
        .inspect(|ln| println!("[zinniad] {}", ln));

    // Wait until our module starts and calls `fetch('ipfs://...')`
    stdout_lines
        .by_ref()
        .take_while(|ln| !ln.contains("fetch-start"))
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
        stdout_lines.find(|ln| ln.contains("fetch-end")).is_none(),
        "zinniad should have been killed before the retrieval finished"
    );

    // Assert there is at least one temp file left over
    let old_files: Vec<_> = read_dir(cache_root.join("lassie"))
        .expect("cannot list files in Lassie's temp dir")
        .map(|it| it.expect("cannot parse directory entry").path())
        .collect();
    assert!(
        old_files.len() > 0,
        "Lassie should have left some temp files"
    );

    // Run zinniad again
    let mut child = cmd.spawn().expect(&format!("cannot spawn {}", bin_str));
    let mut stdout_lines = BufReader::new(child.stdout.take().expect("cannot take child's stdout"))
        .lines()
        .map(|it| it.expect("cannot read from child's stdout"))
        .inspect(|ln| println!("[zinniad] {}", ln));

    // Wait until our module starts and calls `fetch('ipfs://...')`
    stdout_lines
        .by_ref()
        .take_while(|ln| !ln.contains("fetch-start"))
        .for_each(drop);

    // Stop the process
    // Note: on Unix, this sends SIGKILL signal which allows the process to shutdown gracefully
    // If we ever implement graceful shutdown for Lassie, then we may need to rework this line.
    child.kill().expect("cannot stop zinniad");

    // Check that all temp files from the previous run were deleted
    let found: Vec<_> = read_dir(cache_root.join("lassie"))
        .expect("cannot list files in Lassie's temp dir")
        .map(|it| it.expect("cannot parse directory entry").path())
        .filter(|ent| old_files.contains(&ent))
        .collect();

    assert_eq!(
        found.len(),
        0,
        "all files from the previous run should have been deleted"
    );
}
