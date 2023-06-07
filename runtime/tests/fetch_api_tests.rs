// Integration tests making Fetch API request against a custom HTTP server

use std::rc::Rc;

use anyhow::{Context, Result};
use assert_fs::prelude::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use zinnia_runtime::{anyhow, deno_core, run_js_module, BootstrapOptions, RecordingReporter};

#[tokio::test]
async fn fetch_reports_user_agent() -> Result<()> {
    let _ = env_logger::builder().is_test(true).try_init();

    let user_agent = "zinnia_fetch_api_tests agent/007";
    let server_port = start_echo_server().await?;

    let mod_js = assert_fs::NamedTempFile::new("fetch-test.js")?;
    mod_js.write_str(&format!(
        r#"
import {{ assertArrayIncludes }} from "zinnia:assert";
const response = await (await fetch("http://127.0.0.1:{server_port}/echo")).text();
console.log("RESPONSE\n%s", response);
const request_lines = response.split(/\r\n/g);
assertArrayIncludes(request_lines, ["user-agent: {user_agent}"]);
"#,
    ))?;

    let main_module = deno_core::resolve_path(
        &mod_js.to_string_lossy(),
        &std::env::current_dir().context("unable to get current working directory")?,
    )?;
    let reporter = Rc::new(RecordingReporter::new());
    let config = BootstrapOptions::new(user_agent.into(), reporter.clone(), None);
    run_js_module(&main_module, &config).await?;
    // the test passes when the JavaScript code does not throw
    Ok(())
}

// TODO: return something that will allow the caller to stop the server
async fn start_echo_server() -> Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .context("cannot listen on localhost")?;
    let port = listener.local_addr()?.port();
    tokio::spawn(echo_server(listener));
    Ok(port)
}

async fn echo_server(listener: TcpListener) {
    println!("[server] Listening on: {:?}", listener.local_addr());
    loop {
        let (mut socket, _) = listener
            .accept()
            .await
            .expect("cannot accept incoming connection");
        println!("[server] connection accepted");

        tokio::spawn(async move {
            let mut header_sent = false;
            let mut buf = vec![0; 1024];
            loop {
                let n = socket
                    .read(&mut buf)
                    .await
                    .expect("failed to read data from socket");
                println!("[server] Read {n} bytes");

                if !header_sent {
                    header_sent = true;
                    socket
                        .write_all(
                            vec![
                                "HTTP/1.1 200 OK\r\n",
                                "Connection: close\r\n",
                                "\r\n", // an empty line delimits response header from the body
                            ]
                            .join("")
                            .as_bytes(),
                        )
                        .await
                        .expect("cannot write response header");
                    println!("[server] Response header sent");
                }

                if n == 0 {
                    break;
                }

                socket
                    .write_all(&buf[0..n])
                    .await
                    .expect("failed to write data to socket");
                println!("[server] Echoed {n} bytes");

                // This is not very robust. If these four bytes are split across chunk boundaries,
                // e.g. by the underlying TCP protocol, then our detection fails.
                if buf[0..n].ends_with(b"\r\n\r\n") {
                    println!("[server] Detected end of request headers, stopping the echo loop");
                    break;
                }
            }
            socket
                .shutdown()
                .await
                .expect("cannot shutdown incoming connection");
            println!("[server] Request handled")
        });
    }
}
