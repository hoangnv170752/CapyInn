use std::path::PathBuf;

pub const APP_NAME: &str = "CapyInn";
pub const APP_RUNTIME_DIR: &str = "CapyInn";
pub const APP_DATABASE_FILENAME: &str = "capyinn.db";
pub const APP_GATEWAY_LOCKFILE: &str = ".gateway-port";
pub const APP_BUNDLE_IDENTIFIER: &str = "io.capyinn.app";

pub fn runtime_root() -> PathBuf {
    runtime_root_opt().expect("Cannot find home directory")
}

pub fn runtime_root_opt() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(APP_RUNTIME_DIR))
}

pub fn database_path() -> PathBuf {
    runtime_root().join(APP_DATABASE_FILENAME)
}

pub fn database_path_opt() -> Option<PathBuf> {
    runtime_root_opt().map(|root| root.join(APP_DATABASE_FILENAME))
}

pub fn scans_dir() -> PathBuf {
    runtime_root().join("Scans")
}

pub fn scans_dir_opt() -> Option<PathBuf> {
    runtime_root_opt().map(|root| root.join("Scans"))
}

pub fn models_dir() -> PathBuf {
    runtime_root().join("models")
}

pub fn models_dir_opt() -> Option<PathBuf> {
    runtime_root_opt().map(|root| root.join("models"))
}

pub fn exports_dir() -> PathBuf {
    runtime_root().join("exports")
}

pub fn exports_dir_opt() -> Option<PathBuf> {
    runtime_root_opt().map(|root| root.join("exports"))
}

pub fn gateway_lockfile() -> PathBuf {
    runtime_root().join(APP_GATEWAY_LOCKFILE)
}

pub fn gateway_lockfile_opt() -> Option<PathBuf> {
    runtime_root_opt().map(|root| root.join(APP_GATEWAY_LOCKFILE))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_capyinn_runtime_names() {
        let root = runtime_root();
        assert!(root.ends_with(APP_RUNTIME_DIR));
        assert_eq!(database_path(), root.join(APP_DATABASE_FILENAME));
        assert_eq!(scans_dir(), root.join("Scans"));
        assert_eq!(models_dir(), root.join("models"));
        assert_eq!(exports_dir(), root.join("exports"));
        assert_eq!(gateway_lockfile(), root.join(APP_GATEWAY_LOCKFILE));
        assert_eq!(APP_NAME, "CapyInn");
        assert_eq!(APP_BUNDLE_IDENTIFIER, "io.capyinn.app");
    }
}
