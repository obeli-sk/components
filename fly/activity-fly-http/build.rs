use anyhow::Result;
use wit_bindgen_rust::Opts;

fn main() -> Result<()> {
    let path = Opts {
        generate_all: true,
        additional_derive_attributes: vec![
            "serde::Serialize".to_string(),
            "serde::Deserialize".to_string(),
        ],
        ..Default::default()
    }
    .build()
    .generate_to_out_dir(None)?;

    // TODO: Replace the generated-code regex rewriting below with wit-bindgen's
    // `additional_type_attributes` and `additional_member_attributes` once a
    // release containing https://github.com/bytecodealliance/wit-bindgen/pull/1656
    // is available (the feature is not present in 0.60.0). Use fully qualified
    // WIT selectors for the kebab-case enum/variant attributes and for
    // `machine-config.env`, then remove the `regex` build dependency.
    /*
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
    */
    let contents = std::fs::read_to_string(&path)?;
    let re = regex::Regex::new(r"(pub\s+enum\s+\w+)").unwrap();
    let contents = re
        .replace_all(&contents, "#[serde(rename_all = \"kebab-case\")]\n$1")
        .into_owned();
    let re_env = regex::Regex::new(r"(pub env: Option<_rt::Vec)").unwrap();
    let contents = re_env
        .replace(
            &contents,
            "#[serde(with = \"crate::machine::env_serde\", default)]\n    $1",
        )
        .into_owned();
    std::fs::write(&path, contents)?;

    Ok(())
}
