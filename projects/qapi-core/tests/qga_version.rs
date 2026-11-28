//! Parsing the version a guest agent reports in `guest-info`.

use qapi_core::{VersionComponent, VersionParseErrorKind, parse_qga_version};
use std::num::IntErrorKind;

#[test]
fn releases_parse_with_suffixes_and_a_fourth_component() {
    for (input, expected) in [
        ("9.1.0", (9, 1, 0)),
        ("11.1.3", (11, 1, 3)),
        ("9.1.0-rc0", (9, 1, 0)),
        ("11.1.0rc1", (11, 1, 0)),
        ("8.2.2.1", (8, 2, 2)),
        (" v10.2.0\n", (10, 2, 0)),
    ] {
        assert_eq!(parse_qga_version(input), Ok(expected), "{input:?}");
    }
}

#[test]
fn versions_without_three_components_are_refused() {
    for input in ["", "9", "9.1", "9.1.0.1.2"] {
        let error = parse_qga_version(input).unwrap_err();
        assert_eq!(error.input(), input);
        assert_eq!(error.kind(), &VersionParseErrorKind::Shape, "{input:?}");
    }
}

#[test]
fn components_that_are_not_u32_name_the_component() {
    for (input, component, kind) in [
        ("x.y.z", VersionComponent::Major, IntErrorKind::InvalidDigit),
        ("9.x.0", VersionComponent::Minor, IntErrorKind::InvalidDigit),
        ("9.1.rc0", VersionComponent::Patch, IntErrorKind::Empty),
        ("9.1.0rc1.2", VersionComponent::Patch, IntErrorKind::InvalidDigit),
        ("9.1.0.x", VersionComponent::Fourth, IntErrorKind::Empty),
        ("+9.1.0", VersionComponent::Major, IntErrorKind::InvalidDigit),
        ("9.+1.0", VersionComponent::Minor, IntErrorKind::InvalidDigit),
        ("9.1.+0.1", VersionComponent::Patch, IntErrorKind::InvalidDigit),
        ("4294967296.0.0", VersionComponent::Major, IntErrorKind::PosOverflow),
    ] {
        let error = parse_qga_version(input).unwrap_err();
        let VersionParseErrorKind::Component { which, source } = error.kind()
        else {
            panic!("{input:?}: {error}");
        };
        assert_eq!((*which, source.kind()), (component, &kind), "{input:?}");
        assert!(error.to_string().contains(&component.to_string()), "{error}");
    }
}
