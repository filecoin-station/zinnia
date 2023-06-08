use std::sync::{Arc, OnceLock};

pub fn lassie_daemon() -> Arc<lassie::Daemon> {
    static LASSIE_DAEMON: OnceLock<Result<Arc<lassie::Daemon>, lassie::StartError>> =
        OnceLock::new();

    let result = LASSIE_DAEMON
        .get_or_init(|| lassie::Daemon::start(lassie::DaemonConfig::default()).map(Arc::new));

    match result {
        Ok(ptr) => Arc::clone(ptr),
        Err(err) => panic!("could not start Lassie daemon: {err}"),
    }
}
