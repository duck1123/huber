use huber_common::model::package::{Package, PackageManagement, PackageSource, PackageTargetType};

#[allow(dead_code)]
pub fn release() -> Package {
    Package {
        name: "consul".to_string(),
        source: PackageSource::Github {
            owner: "hashicorp".to_string(),
            repo: "consul".to_string(),
        },
        detail: None,
        targets: vec![
            PackageTargetType::LinuxAmd64(PackageManagement {
                artifact_templates: vec![
                    "https://releases.hashicorp.com/consul/{version}/consul_{version}_linux_amd64.zip"
                        .to_string(),
                ],
                executable_templates: None,
                executable_mappings: None,
                install_commands: None,
                uninstall_commands: None,
                upgrade_commands: None,
            }),
            PackageTargetType::LinuxArm64(PackageManagement {
                artifact_templates: vec![
                    "https://releases.hashicorp.com/consul/{version}/consul_{version}_linux_arm64.zip"
                        .to_string(),
                ],
                executable_templates: None,
                executable_mappings: None,
                install_commands: None,
                uninstall_commands: None,
                upgrade_commands: None,
            }),
            PackageTargetType::MacOS(PackageManagement {
                artifact_templates: vec![
                    "https://releases.hashicorp.com/consul/{version}/consul_{version}_darwin_amd64.zip"
                        .to_string(),
                ],
                executable_templates: None,
                executable_mappings: None,
                install_commands: None,
                uninstall_commands: None,
                upgrade_commands: None,
            }),
            PackageTargetType::Windows(PackageManagement {
                artifact_templates: vec![
                    "https://releases.hashicorp.com/consul/{version}/consul_{version}_windows_amd64.zip"
                        .to_string(),
                ],
                executable_templates: None,
                executable_mappings: None,
                install_commands: None,
                uninstall_commands: None,
                upgrade_commands: None,
            }),
        ],
        version: None,
        description: None,
        release_kind: None
    }
}
