use std::env;

/// Reads a required environment variable; panics with a clear message at startup if it is missing.
pub fn required(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| panic!("{name} must be configured."))
}

/// Reads a required environment variable holding a number of seconds.
pub fn required_u64(name: &str) -> u64 {
    required(name)
        .parse()
        .unwrap_or_else(|_| panic!("Cannot parse {name} as u64"))
}
