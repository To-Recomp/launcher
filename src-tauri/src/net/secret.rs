//! The proxy password lives in the system keychain, never in `launcher.json`.

const SERVICE: &str = "org.torecomp.launcher";
const ACCOUNT: &str = "proxy";

pub trait SecretStore {
    fn get(&self) -> Result<Option<String>, String>;
    fn set(&self, secret: &str) -> Result<(), String>;
    fn delete(&self) -> Result<(), String>;
}

/// Secret Service (GNOME Keyring, KWallet) on Linux, Credential Manager on Windows, Keychain on
/// macOS.
pub struct Keychain;

fn unavailable(e: keyring::Error) -> String {
    format!(
        "The system keychain isn't available, so the password can't be kept safely: {e}. \
         The launcher never stores it in a file."
    )
}

fn entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new(SERVICE, ACCOUNT).map_err(unavailable)
}

impl SecretStore for Keychain {
    fn get(&self) -> Result<Option<String>, String> {
        match entry()?.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(unavailable(e)),
        }
    }

    fn set(&self, secret: &str) -> Result<(), String> {
        entry()?.set_password(secret).map_err(unavailable)
    }

    fn delete(&self) -> Result<(), String> {
        match entry()?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(unavailable(e)),
        }
    }
}
