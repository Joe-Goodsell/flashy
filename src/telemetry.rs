use std::sync::Mutex;

pub fn initialise_subscriber() {
    let logfile = std::fs::File::create("logfile.log").expect("Failed to create logfile"); 
    let subscriber = tracing_subscriber::fmt()
        .with_writer(Mutex::new(logfile))
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set global subscriber");
}