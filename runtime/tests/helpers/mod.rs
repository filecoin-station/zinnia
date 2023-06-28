use std::sync::{Arc, OnceLock};

use zinnia_runtime::lassie_config;

pub fn lassie_daemon() -> Arc<lassie::Daemon> {
    static LASSIE_DAEMON: OnceLock<Result<Arc<lassie::Daemon>, lassie::StartError>> =
        OnceLock::new();

    let result = LASSIE_DAEMON.get_or_init(|| lassie::Daemon::start(lassie_config()).map(Arc::new));

    match result {
        Ok(ptr) => Arc::clone(ptr),
        Err(err) => panic!("could not start Lassie daemon: {err}"),
    }
}
