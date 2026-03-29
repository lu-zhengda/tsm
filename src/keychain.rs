use crate::error::Error;

pub const SERVICE_NAME: &str = "tsm";
pub const KEYCHAIN_SENTINEL: &str = "keychain:";

#[cfg(target_os = "macos")]
pub fn store_password(service: &str, account: &str, password: &str) -> Result<(), Error> {
    if service.is_empty() || account.is_empty() {
        return Err(Error::Keychain(
            "Service and account must not be empty".to_string(),
        ));
    }

    let output = std::process::Command::new("/usr/bin/security")
        .args([
            "add-generic-password",
            "-U", // update if exists
            "-s",
            service,
            "-a",
            account,
            "-w",
            password,
        ])
        .output()
        .map_err(|e| Error::Keychain(format!("Failed to run security command: {e}")))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(Error::Keychain(format!(
            "Failed to store password in Keychain: {stderr}"
        )))
    }
}

#[cfg(target_os = "macos")]
pub fn find_password(service: &str, account: &str) -> Result<String, Error> {
    let output = std::process::Command::new("/usr/bin/security")
        .args(["find-generic-password", "-s", service, "-a", account, "-w"])
        .output()
        .map_err(|e| Error::Keychain(format!("Failed to run security command: {e}")))?;

    if output.status.success() {
        let password = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(password)
    } else {
        Err(Error::Keychain(format!(
            "Password not found in Keychain for profile \"{account}\". \
             Run `tsm login --keychain` to store credentials, \
             or set TSM_PASSWORD env var."
        )))
    }
}

#[cfg(not(target_os = "macos"))]
pub fn store_password(_service: &str, _account: &str, _password: &str) -> Result<(), Error> {
    Err(Error::Keychain(
        "Keychain support is macOS only".to_string(),
    ))
}

#[cfg(not(target_os = "macos"))]
pub fn find_password(_service: &str, _account: &str) -> Result<String, Error> {
    Err(Error::Keychain(
        "Keychain support is macOS only".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sentinel_constant() {
        assert_eq!(KEYCHAIN_SENTINEL, "keychain:");
    }

    #[test]
    fn test_service_name() {
        assert_eq!(SERVICE_NAME, "tsm");
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_store_empty_service_rejected() {
        let result = store_password("", "account", "pass");
        assert!(result.is_err());
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_store_empty_account_rejected() {
        let result = store_password("service", "", "pass");
        assert!(result.is_err());
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_find_nonexistent_returns_error() {
        let result = find_password("tsm-test-nonexistent", "no-such-profile");
        assert!(result.is_err());
    }

    #[test]
    #[cfg(target_os = "macos")]
    #[ignore] // requires Keychain access, run manually
    fn test_store_and_find_roundtrip() {
        let service = "tsm-test";
        let account = "test-roundtrip";
        let password = "s3cret!@#$%";

        store_password(service, account, password).unwrap();
        let retrieved = find_password(service, account).unwrap();
        assert_eq!(retrieved, password);

        // Cleanup
        let _ = std::process::Command::new("/usr/bin/security")
            .args(["delete-generic-password", "-s", service, "-a", account])
            .output();
    }
}
