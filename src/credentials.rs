use ini::Ini;
use std::path::PathBuf;
use std::env;

#[derive(Clone,Debug)]
pub struct Credentials {
    pub key: Option<String>,
    pub secret: Option<String>,
    path: String,
    profile: String,
}

impl<'a> Credentials {
    pub fn new() -> Credentials {
        Credentials{
            key: None,
            secret: None,
            path: get_profile_path(),
            profile: get_default_profile(),
        }
    }

    pub fn path(mut self, path: &str) -> Credentials {
        self.path = get_absolute_path(path);
        self
    }

    pub fn profile(mut self, profile: &str) -> Credentials {
        self.profile = String::from(profile);
        self
    }

    /// Load access and secret keys from environment or config file
    ///
    /// Behaviour is as follows:
    /// 1. If environment variable is present, use that.
    /// 2. Otherwise, use the profile:
    /// 2.1. If profile is set, use that.
    /// 2.2. Otherwise use default profile.
    ///
    /// Behaviour is copied from boto.
    pub fn load(mut self) -> Credentials {
        if let Ok(conf) = Ini::load_from_file(&self.path) {
            if let Some(section) = conf.section(Some(&self.profile)) {
                if let Some(key) = section.get("aws_access_key_id") {
                    self.key = Some(key.to_string())
                };
                if let Some(secret) = section.get("aws_secret_access_key") {
                    self.secret = Some(secret.to_string())
                }
            }
        };
        if let Ok(key) = env::var("AWS_ACCESS_KEY_ID") {
            self.key = Some(key.to_string())
        };

        if let Ok(secret) = env::var("AWS_SECRET_ACCESS_KEY") {
            self.secret = Some(secret.to_string())
        };
        self
    }
}

fn get_default_profile() -> String {
    match env::var("AWS_PROFILE") {
        Err(_) => "default".to_string(),
        Ok(s) => s.to_string(),
    }
}

fn get_profile_path() -> String {
    let home = match env::var("HOME") {
        // hell if i know what not having home set means
        Err(_) => "/root".to_string(),
        Ok(s) => s,
    };
    let mut p = PathBuf::from(&home);
    p.push(".aws");
    p.push("credentials");
    p.to_str().unwrap().to_string()
}

fn get_absolute_path(val: &str) -> String {
    let mut p = PathBuf::from(val);
    if !p.is_absolute() {
        p = env::current_dir().unwrap();
        p.push(val);
    }
    p.to_str().unwrap().to_string()
}

#[cfg(test)]
mod test {
    use super::Credentials;
    use std::env;

    // Tests depend on Credentials being resolved correctly. Since they are
    // executed in parallel, the explicit environment tests can mess up
    // other tests running at the same time.
    //
    // * Write lock needs to be acquired if the tests are changing environment
    //   variables.
    // * Read lock needs to be acquired if tests depend on environment
    //   variables (but do not change them).
    use std::sync::{StaticRwLock, RW_LOCK_INIT};
    static LOCK: StaticRwLock = RW_LOCK_INIT;

    #[test]
    fn test_defaults() {
        let _g = LOCK.read().unwrap();
        let cred = Credentials::new().path("/my/credentials/file");
        assert_eq!(cred.path, "/my/credentials/file")
    }

    #[test]
    fn test_profile() {
        let _g = LOCK.read().unwrap();
        let cred = Credentials::new().profile("new");
        assert_eq!(cred.profile, "new")
    }

    #[test]
    fn test_load_default() {
        let _g = LOCK.read().unwrap();
        // the path is relative from where cargo is running, so the root of the project
        let cred = Credentials::new().path("fixtures/credentials.ini").load();
        assert_eq!(cred.key.unwrap(), "12345")
    }

    #[test]
    fn test_load_specific() {
        let _g = LOCK.read().unwrap();
        let cred = Credentials::new().path("fixtures/credentials.ini").profile("first").load();
        assert_eq!(cred.key.unwrap(), "zxspectrum")
    }

    #[test]
    fn test_env_first_success() {
        let _g = LOCK.write().unwrap();
        env::set_var("AWS_ACCESS_KEY_ID", "envaccess");
        env::set_var("AWS_SECRET_ACCESS_KEY", "envsecret");
        let cred = Credentials::new().path("fixtures/credentials.ini").load();
        env::remove_var("AWS_ACCESS_KEY_ID");
        env::remove_var("AWS_SECRET_ACCESS_KEY");
        assert_eq!(cred.key.unwrap(), "envaccess");
        assert_eq!(cred.secret.unwrap(), "envsecret");
    }

    #[test]
    fn test_env_first_fail() {
        let _g = LOCK.write().unwrap();
        env::set_var("AWS_SECRET_ACCESS_KEY", "envsecret");
        let cred = Credentials::new().path("fixtures/credentials.ini").load();
        env::remove_var("AWS_SECRET_ACCESS_KEY");
        assert_eq!(cred.key.unwrap(), "12345");
        assert_eq!(cred.secret.unwrap(), "envsecret")
    }
}
