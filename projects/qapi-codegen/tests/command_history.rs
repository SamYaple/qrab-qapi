//! Compile a command whose execution flags have two separate lifetimes.
mod common;
use common::{generate, git, run_fixture, write_module};
use std::path::Path;

#[test]
fn generated_command_flags_preserve_a_removed_interval() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("schema");
    std::fs::create_dir_all(repo.join("qapi")).unwrap();
    git(&repo, &["init", "-q"]);
    for (tag, schema) in [
        (
            "v6.1.0",
            "{'command':'flip','allow-oob':true,'allow-preconfig':true,'success-response':false}",
        ),
        ("v8.0.0", "{'command':'flip'}"),
        (
            "v11.1.3",
            "{'command':'flip','allow-oob':true,'allow-preconfig':true,'success-response':false}",
        ),
    ] {
        let feature = if tag == "v8.0.0" {
            "[]"
        } else {
            "['unstable']"
        };
        let initial = if tag == "v6.1.0" {
            "['unstable']"
        } else {
            "[]"
        };
        let surface = if tag == "v8.0.0" {
            ""
        } else {
            "{'command':'foo','data':'Foo','returns':'Foo'}\n{'command':'ping'}\n{'event':'FOO','data':'Foo'}"
        };
        let late = if tag == "v11.1.3" {
            "{'command':'late','data':'Late'}"
        } else {
            ""
        };
        let renamed = match tag {
            "v6.1.0" => "{'command':'renamed-1'}\n{'event':'RENAMED_BAR'}",
            "v11.1.3" => "{'command':'renamed1'}\n{'event':'RENAMED__BAR'}",
            _ => "",
        };
        let schema = format!(
            "{schema}\n{{'command':'feature-flip','features':{feature}}}\n{{'command':'stabilized','features':{initial}}}\n{{'struct':'Foo','data':{{'value':'str'}}}}\n{{'struct':'Late','data':{{'value':'str'}}}}\n{surface}\n{late}\n{renamed}"
        );
        std::fs::write(repo.join("qapi/qapi-schema.json"), schema).unwrap();
        git(&repo, &["add", "."]);
        git(
            &repo,
            &[
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.invalid",
                "commit",
                "-qm",
                tag,
            ],
        );
        git(&repo, &["tag", tag]);
    }
    let output = generate(&repo, &["v6.1.0", "v8.0.0", "v11.1.3"]).unwrap();
    let fixture = temp.path().join("fixture");
    write_module(&fixture.join("src/generated"), output);
    run_fixture(
        &workspace,
        &fixture,
        r#"
pub use qapi_runtime::{PreconfigSafe, QapiCommand, QapiEvent, QapiFlatUnion, qemu_version, scoped_qemu_version, serde, serde_json, unavailable_error, unavailable_type_message, with_version};
mod generated;
pub use generated::*;
fn main() {
    assert_eq!(commands::Renamed1::NAME, "renamed-1");
    assert_eq!(commands::Renamed1::REMOVED_IN, Some((8,0,0)));
    assert_eq!(commands::Renamed1Generated::NAME, "renamed1");
    assert_eq!(commands::Renamed1Generated::AVAILABLE_SINCE, (11,1,3));
    assert_eq!(events::RenamedBarEvent::NAME, "RENAMED_BAR");
    assert_eq!(events::RenamedBarEventGenerated::NAME, "RENAMED__BAR");
    assert_eq!(commands::LateCommand::AVAILABLE_SINCE,(11,1,3));
    with_version((8,0,0), || {
        assert!(serde_json::to_value(Late {value:"data".into()}).is_ok());
        assert!(serde_json::to_value(commands::LateCommand {value:"request".into()}).is_err());
    });
    // The first run keeps the plain names; the run that returns is named after it.
    with_version((6,1,0), || {
        let command = commands::FooCommand { value:"old".into() };
        assert_eq!(commands::FooCommand::NAME, "foo");
        assert_eq!(commands::FooCommand::REMOVED_IN, Some((8,0,0)));
        assert_eq!(serde_json::to_value(command).unwrap(),serde_json::json!({"value":"old"}));
        assert_eq!(commands::Ping::NAME,"ping");
    });
    with_version((11,1,3), || {
        let command = commands::FooGenerated { value:"new".into() };
        assert_eq!(commands::FooGenerated::NAME,"foo");
        assert_eq!(commands::FooGenerated::AVAILABLE_SINCE,(11,1,3));
        assert_eq!(serde_json::to_value(command).unwrap(),serde_json::json!({"value":"new"}));
        assert_eq!(commands::PingGenerated::NAME,"ping");
    });
    with_version((8,0,0), || {
        assert!(serde_json::to_value(Foo {value:"data remains valid".into()}).is_ok());
        assert!(serde_json::to_value(commands::FooCommand {value:"old".into()}).is_err());
        assert!(serde_json::to_value(commands::FooGenerated {value:"new".into()}).is_err());
        assert!(serde_json::to_value(events::FooEvent {value:"old".into()}).is_err());
        assert!(serde_json::to_value(events::FooEventGenerated {value:"new".into()}).is_err());
        assert!(serde_json::from_value::<events::FooEventGenerated>(serde_json::json!({"value":"new"})).is_err());
        assert!(serde_json::from_value::<events::FooEvent>(serde_json::json!({"value":"old"})).is_err());

    });
    for (version, enabled) in [((6,1,0),true), ((8,0,0),false), ((11,1,3),true)] {
        assert_eq!(commands::Flip::allows_oob(version), enabled);
        assert_eq!(commands::Flip::allows_preconfig(version), enabled);
        assert_eq!(commands::Flip::sends_success_response(version), !enabled);
        assert_eq!(commands::FeatureFlip::is_unstable(version), enabled);
        assert_eq!(commands::Stabilized::is_unstable(version), version == (6,1,0));
    }
    assert_eq!(commands::Flip::PRECONFIG_SINCE, Some((6,1,0)));
    assert!(commands::Flip::ALLOW_OOB && commands::Flip::ALLOW_PRECONFIG);
    assert!(!commands::Flip::SUCCESS_RESPONSE);
    assert!(commands::FeatureFlip::UNSTABLE);
    assert!(!commands::Stabilized::UNSTABLE);
    let command = metadata::COMMANDS.iter().find(|command| command.wire_name() == "flip").unwrap();
    let feature_flip = metadata::COMMANDS.iter().find(|command| command.wire_name() == "feature-flip").unwrap();
    let stabilized = metadata::COMMANDS.iter().find(|command| command.wire_name() == "stabilized").unwrap();
    for (version, enabled) in [((6,1,0),true), ((8,0,0),false), ((11,1,3),true)] {
        assert_eq!(command.allows_oob(version), enabled);
        assert_eq!(command.allows_preconfig(version), enabled);
        assert_eq!(command.sends_success_response(version), !enabled);
        assert_eq!(feature_flip.is_unstable(version), enabled);
        assert_eq!(stabilized.is_unstable(version), version == (6,1,0));
    }
}
"#,
    );
}
