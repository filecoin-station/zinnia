use std::path::Path;
use std::rc::Rc;

use deno_core::anyhow::Result;
use deno_core::error::JsError;
use deno_core::url::Url;
use deno_core::{op, OpState};
use deno_fetch::FetchPermissions;
use deno_web::TimersPermission;

use crate::{LogLevel, Reporter};

/// Hard-coded permissions
pub struct ZinniaPermissions;

impl TimersPermission for ZinniaPermissions {
    fn allow_hrtime(&mut self) -> bool {
        // Disable high-resolution time management.
        //
        // Quoting from https://v8.dev/docs/untrusted-code-mitigations
        // > A high-precision timer makes it easier to observe side channels in the SSCA
        // > vulnerability. If your product offers high-precision timers that can be accessed by
        // > untrusted JavaScript or WebAssembly code, consider making these timers more coarse or
        // > adding jitter to them.
        false
    }
    fn check_unstable(&self, _state: &deno_core::OpState, _api_name: &'static str) {}
}

impl FetchPermissions for ZinniaPermissions {
    fn check_net_url(&mut self, _url: &Url, _api_name: &str) -> Result<()> {
        Ok(())
    }
    fn check_read(&mut self, _p: &Path, _api_name: &str) -> Result<()> {
        Ok(())
    }
}

deno_core::extension!(
    zinnia_runtime,
    ops = [
        op_job_completed,
        op_info_activity,
        op_error_activity,
        op_zinnia_log,
        op_format_test_error
    ],
    esm_entry_point = "ext:zinnia_runtime/99_main.js",
    esm = [
      dir "js",
      "06_util.js",
      "90_zinnia_apis.js",
      "98_global_scope.js",
      "internals.js",
      "test.js",
      "vendored/asserts.bundle.js",
      "99_main.js",
    ],
    options = {
        reporter: Rc<dyn Reporter>,
    },
    state = |state, options| {
        state.put(ZinniaPermissions {});
        state.put(Rc::clone(&options.reporter));
    }
);

type StoredReporter = Rc<dyn Reporter>;

#[op]
fn op_job_completed(state: &mut OpState) {
    let reporter = state.borrow::<StoredReporter>();
    reporter.job_completed();
}

#[op]
fn op_info_activity(state: &mut OpState, msg: &str) {
    let reporter = state.borrow::<StoredReporter>();
    reporter.info_activity(msg);
}

#[op]
fn op_error_activity(state: &mut OpState, msg: &str) {
    let reporter = state.borrow::<StoredReporter>();
    reporter.error_activity(msg);
}

#[op]
fn op_zinnia_log(state: &mut OpState, msg: &str, level: LogLevel) {
    let reporter = state.borrow::<StoredReporter>();
    reporter.log(level, msg);
}

#[op]
fn op_format_test_error(error: JsError) -> String {
    crate::vendored::cli_tools::format_test_error(&error)
}
