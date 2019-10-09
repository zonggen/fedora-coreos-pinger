mod platform;
mod os_release;
mod instance_type;

use crate::config::inputs;
use crate::rpm_ostree;
use failure::{Fallible, ResultExt};
use serde::Serialize;
use std::collections::HashMap;
use maplit;

/// Kernel arguments location
static KERNEL_ARGS_FILE: &str = "/proc/cmdline";
/// aleph version file
static OS_ALEPH_VERSION_FILE: &str = "/.coreos-aleph-version.json";
/// Afterburn cloud metadata location
static AFTERBURN_METADATA: &str = "/run/metadata/afterburn";

/// Agent identity.
#[derive(Debug, Serialize)]
pub(crate) struct Identity {
    /// Collecting level
    pub(crate) level: String,
    /// OS platform
    pub(crate) platform: String,
    /// Original OS version
    pub(crate) original_os_version: String,
    /// Current OS version
    pub(crate) current_os_version: String,
    /// Instance type if on cloud platform
    pub(crate) instance_type: Option<String>,
}

impl Identity {
    /// Create from configuration.
    pub(crate) fn new(cfg: &inputs::CollectingInput) -> Fallible<Self> {
        let collecting_level = &cfg.level;
        let id = match collecting_level.as_str() {
            level @ "minimal" | level @ "full" => Self::try_default(level).context(format!("failed to build '{}' identity", level))?,
            &_ => Self::try_default("minimal").context("failed to build 'minimal' identity")?,
        };

        Ok(id)
    }

    /// Try to fetch default data
    pub fn try_default(level: &str) -> Fallible<Self> {
        let platform = platform::get_platform(KERNEL_ARGS_FILE)?;
        let original_os_version = os_release::read_original_os_version(OS_ALEPH_VERSION_FILE)?;
        let current_os_version = rpm_ostree::booted()?.version;
        let instance_type: Option<String> = match platform.as_str() {
            "aliyun" | "aws" | "azure" | "gcp" | "openstack" => Some(instance_type::read_instance_type(AFTERBURN_METADATA, platform.as_str())?),
            _ => None,
        };

        let id = match level {
                    "minimal" | "full" => Self {
                                    level: level.to_string(),
                                    platform,
                                    original_os_version,
                                    current_os_version,
                                    instance_type,
                                },
                    &_ => Self {
                                    level: "minimal".to_string(),
                                    platform,
                                    original_os_version,
                                    current_os_version,
                                    instance_type,
                                },
                };

        Ok(id)
    }

    /// Getter for collected data, returned as a HashMap
    pub fn get_data(&self) -> HashMap<String, String> {
        let vars = maplit::hashmap!{
            "level".to_string() => self.level.clone(),
            "platform".to_string() => self.platform.clone(),
            "original_os_version".to_string() => self.original_os_version.clone(),
            "current_os_version".to_string() => self.current_os_version.clone(),
            "instance_type".to_string() => match &self.instance_type {
                Some(v) => v.clone(),
                None => "".to_string(),
            },
        };

        // TODO: Insert data specific to different levels
        match self.level.as_str() {
            "minimal" | "full" => (),
            &_ => (),
        };

        vars
    }

    #[cfg(test)]
    pub(crate) fn mock_default(level: &str) -> Self {
        match level {
            "minimal" => return Self {
                            level: String::from("minimal"),
                            platform: "mock-qemu".to_string(),
                            original_os_version: "30.20190923.dev.2-2".to_string(),
                            current_os_version: "mock-os-version".to_string(),
                            instance_type: Some("mock-instance-type".to_string()),
                        },
            "full" => return Self {
                            level: String::from("full"),
                            platform: "mock-gcp".to_string(),
                            original_os_version: "30.20190923.dev.2-2".to_string(),
                            current_os_version: "mock-os-version".to_string(),
                            instance_type: Some("mock-instance-type".to_string()),
                        },
            &_ => return Self {
                            level: String::from("minimal"),
                            platform: "mock-qemu".to_string(),
                            original_os_version: "30.20190923.dev.2-2".to_string(),
                            current_os_version: "mock-os-version".to_string(),
                            instance_type: Some("mock-instance-type".to_string()),
                        },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal() {
        let id = Identity::mock_default("minimal");
        let vars = id.get_data();

        // check if the keys exist
        assert!(vars.contains_key("level"));
        assert!(vars.contains_key("platform"));
        assert!(vars.contains_key("original_os_version"));
        assert!(vars.contains_key("current_os_version"));
        assert!(vars.contains_key("instance_type"));

        // check if the values match
        assert_eq!(vars.get("level"), Some(&"minimal".to_string()));
        assert_eq!(vars.get("platform"), Some(&"mock-qemu".to_string()));
        assert_eq!(vars.get("original_os_version"), Some(&"30.20190923.dev.2-2".to_string()));
        assert_eq!(vars.get("current_os_version"), Some(&"mock-os-version".to_string()));
        assert_eq!(vars.get("instance_type"), Some(&"mock-instance-type".to_string()));
    }

    #[test]
    fn test_full() {
        let id = Identity::mock_default("full");
        let vars = id.get_data();

        // check if the keys exist
        assert!(vars.contains_key("level"));
        assert!(vars.contains_key("platform"));
        assert!(vars.contains_key("original_os_version"));
        assert!(vars.contains_key("current_os_version"));
        assert!(vars.contains_key("instance_type"));

        // check if the values match
        assert_eq!(vars.get("level"), Some(&"full".to_string()));
        assert_eq!(vars.get("platform"), Some(&"mock-gcp".to_string()));
        assert_eq!(vars.get("original_os_version"), Some(&"30.20190923.dev.2-2".to_string()));
        assert_eq!(vars.get("current_os_version"), Some(&"mock-os-version".to_string()));
        assert_eq!(vars.get("instance_type"), Some(&"mock-instance-type".to_string()));
    }
}
