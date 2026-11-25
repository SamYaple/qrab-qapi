//! Dual-layer version gating invariants.
//!
//! QEMU fields carry `#[qapi(added = "X.Y.Z")]`. Serialization scoped to a
//! version below `X.Y.Z` refuses a value that sets such a field, naming it,
//! rather than dropping it from the JSON. On the deserialize side, older
//! QEMU versions simply do not emit these fields, so deserialization must
//! tolerate their absence and yield `None` without error.
//!
//! The tests use QEMU 10.1.0, below `10.2.0`, the `since` value of
//! `ImageInfo::limits`.

use qapi::*;

/// A peer below the `limits` field's `since = "10.2.0"`.
const BELOW_LIMITS: (u32, u32, u32) = (10, 1, 0);

/// Build a representative `ImageInfo` with the gated `limits` field
/// populated. Required fields kept minimal.
fn image_info_with_limits() -> ImageInfo {
    ImageInfo {
        actual_size: None,
        backing_filename: None,
        backing_filename_format: None,
        backing_image: None,
        cluster_size: None,
        compressed: None,
        dirty_flag: None,
        encrypted: None,
        filename: "test.qcow2".to_string(),
        format: "qcow2".to_string(),
        format_specific: None,
        full_backing_filename: None,
        limits: Some(BlockLimitsInfo {
            discard_alignment: None,
            max_discard: None,
            max_hw_iov: None,
            max_hw_transfer: None,
            max_iov: 1024,
            max_transfer: None,
            max_write_zeroes: None,
            min_mem_alignment: 512,
            opt_mem_alignment: 4096,
            opt_transfer: None,
            request_alignment: 512,
            write_zeroes_alignment: None,
        }),
        snapshots: None,
        virtual_size: 1 << 30,
    }
}

/// When the peer version is below a field's `since`, serializing a value
/// that sets the field fails with the field's name, and one that leaves it
/// unset omits it from the wire JSON.
#[test]
fn field_above_runtime_version_is_refused_on_serialize() {
    let mut info = image_info_with_limits();
    match to_value(BELOW_LIMITS, &info) {
        Err(EncodeError::Unavailable { name, version }) => {
            assert_eq!((name.as_str(), version), ("ImageInfo.limits", BELOW_LIMITS));
        }
        other => panic!("limits (added 10.2.0) must be refused at 10.1.0: {other:?}"),
    }
    info.limits = None;
    let parsed = to_value(BELOW_LIMITS, &info).unwrap();
    assert!(parsed.get("limits").is_none(), "{parsed}");
    // Sanity: required fields are still emitted.
    assert_eq!(parsed["filename"], "test.qcow2");
    assert_eq!(parsed["format"], "qcow2");
    assert_eq!(parsed["virtual-size"], 1 << 30);
}

/// When the wire JSON from an older QEMU omits a gated field, the
/// deserializer must tolerate its absence and produce `None` rather
/// than error out with "missing field".
#[test]
fn field_above_runtime_version_deserializes_as_default() {
    // JSON as an older QEMU (pre-10.2.0) would emit — no `limits` key.
    let json = r#"{"filename":"old.qcow2","format":"qcow2","virtual-size":536870912}"#;
    let info: ImageInfo = serde_json::from_str(json).unwrap();
    assert_eq!(info.filename, "old.qcow2");
    assert_eq!(info.format, "qcow2");
    assert_eq!(info.virtual_size, 536870912);
    assert!(
        info.limits.is_none(),
        "limits must default to None when absent in JSON"
    );
}

/// Round-trip companion: from the release that added it, the gated field
/// is serialized and decodes back.
#[test]
fn gated_field_round_trips_from_its_release() {
    let info = image_info_with_limits();
    let json = to_value((10, 2, 0), &info).unwrap();
    assert_eq!(json["limits"]["max-iov"], 1024);
    let back: ImageInfo = with_version((10, 2, 0), || serde_json::from_value(json)).unwrap();
    assert_eq!(back.limits.map(|limits| limits.max_iov), Some(1024));
}
