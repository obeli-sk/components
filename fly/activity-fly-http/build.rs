use anyhow::Result;
use wit_bindgen_rust::Opts;

fn main() -> Result<()> {
    const KEBAB_CASE: &str = r#"#[serde(rename_all = "kebab-case")]"#;
    let type_selectors = [
        "obelisk-flyio:activity-fly-http/regions@1.0.0-beta/region",
        "obelisk-flyio:activity-fly-http/machines@1.0.0-beta/machine-state",
        "obelisk-flyio:activity-fly-http/machines@1.0.0-beta/host-status",
        "obelisk-flyio:activity-fly-http/machines@1.0.0-beta/cpu-kind",
        "obelisk-flyio:activity-fly-http/machines@1.0.0-beta/restart-policy",
        "obelisk-flyio:activity-fly-http/machines@1.0.0-beta/service-protocol",
        "obelisk-flyio:activity-fly-http/machines@1.0.0-beta/port-handler",
        "obelisk-flyio:activity-fly-http/ips@1.0.0-beta/ip-variant",
    ];

    Opts {
        generate_all: true,
        additional_derive_attributes: vec![
            "serde::Serialize".to_string(),
            "serde::Deserialize".to_string(),
        ],
        additional_type_attributes: type_selectors
            .into_iter()
            .map(|selector| (selector.to_string(), KEBAB_CASE.to_string()))
            .collect(),
        additional_member_attributes: vec![(
            "obelisk-flyio:activity-fly-http/machines@1.0.0-beta/machine-config.env".to_string(),
            r#"#[serde(with = "crate::machine::env_serde", default)]"#.to_string(),
        )],
        ..Default::default()
    }
    .build()
    .generate_to_out_dir(None)?;

    Ok(())
}
