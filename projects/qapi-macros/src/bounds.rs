//! `QEMU_MIN_VERSION`/`QEMU_MAX_VERSION` compile-time bounds.

use crate::support::Version;
use proc_macro2::Span;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Default)]
pub(crate) struct VersionBounds {
    pub(crate) min: Option<Version>,
    pub(crate) max: Option<Version>,
}

/// Read one bound from the value of its variable. An unset or empty
/// variable is no bound; any other value that is not a version fails the
/// build, naming the variable.
pub(crate) fn bound(
    variable: &str,
    value: Result<String, std::env::VarError>,
) -> syn::Result<Option<Version>> {
    let invalid = |value: &dyn std::fmt::Debug| {
        syn::Error::new(
            Span::call_site(),
            format!("qapi: {variable} must be a QEMU version such as 9.2.0, found {value:?}"),
        )
    };
    match value {
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(value)) => Err(invalid(&value)),
        Ok(value) if value.is_empty() => Ok(None),
        Ok(value) => Version::parse(&value)
            .map(Some)
            .ok_or_else(|| invalid(&value)),
    }
}

/// Whether an invalid bound has been reported in this compilation.
static REPORTED: AtomicBool = AtomicBool::new(false);

/// The bounds this build was configured with. Invalid bounds fail the build
/// with one error, which the first expansion reports; every expansion then
/// proceeds without bounds, so that no other error follows from it.
pub(crate) fn version_bounds() -> (VersionBounds, Option<syn::Error>) {
    let read = |variable| bound(variable, std::env::var(variable));
    let bounds = read("QEMU_MIN_VERSION").and_then(|min| {
        checked_bounds(VersionBounds {
            min,
            max: read("QEMU_MAX_VERSION")?,
        })
    });
    report_once(bounds, &REPORTED)
}

/// Turn an error into unbounded expansion, keeping the error only the first
/// time `reported` sees one.
pub(crate) fn report_once(
    bounds: syn::Result<VersionBounds>,
    reported: &AtomicBool,
) -> (VersionBounds, Option<syn::Error>) {
    match bounds {
        Ok(bounds) => (bounds, None),
        Err(error) => (
            VersionBounds::default(),
            (!reported.swap(true, Ordering::Relaxed)).then_some(error),
        ),
    }
}

/// Reject a minimum above the maximum.
pub(crate) fn checked_bounds(bounds: VersionBounds) -> syn::Result<VersionBounds> {
    if let (Some(min), Some(max)) = (bounds.min, bounds.max)
        && min > max
    {
        return Err(syn::Error::new(
            Span::call_site(),
            format!("qapi: QEMU_MIN_VERSION {min} is later than QEMU_MAX_VERSION {max}"),
        ));
    }
    Ok(bounds)
}

/// Whether a type, field or variant with this lifetime exists in no release
/// of the bounds, so that the build leaves it out: it was added after the
/// maximum, or removed at or before the minimum.
pub(crate) fn excluded_by_bounds(
    bounds: &VersionBounds,
    since: Option<Version>,
    until: Option<Version>,
) -> bool {
    let after_max = matches!((bounds.max, since), (Some(max), Some(since)) if since > max);
    let before_min = matches!((bounds.min, until), (Some(min), Some(until)) if until <= min);
    after_max || before_min
}

/// Whether a deprecation from `added` until `removed` (or for good) gets
/// Rust `#[deprecated]`: when it covers every release the build supports,
/// starting at or before `QEMU_MIN_VERSION` and lasting past
/// `QEMU_MAX_VERSION`. Items deprecated only in some of those releases are
/// still valid for the others.
pub(crate) fn rust_deprecated_by_bounds(
    bounds: &VersionBounds,
    added: Version,
    removed: Option<Version>,
) -> bool {
    let from_min = bounds.min.is_some_and(|min| added <= min);
    let to_max = match (removed, bounds.max) {
        (None, _) => true,
        (Some(removed), Some(max)) => removed > max,
        (Some(_), None) => false,
    };
    from_min && to_max
}
