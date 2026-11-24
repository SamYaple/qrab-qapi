//! Building the QMP and QGA registries from every pinned release.

use super::lower::VersionResult;
use super::lower::load_versions;
use super::merge::{
    check_lifetime, fix_recursive_types, materialize_field_history, materialize_variant_history,
    merge_enum, merge_struct,
};
use super::*;

/// Parse the QMP schemas of the given release tags and merge them into a
/// unified registry. Releases before 1.0, which have no QMP schema, are left
/// out. Extraction and parsing run in parallel across versions; merging is
/// sequential, in release order.
pub fn build_unified_registry(
    qemu_repo: &Path,
    version_tags: &[&str],
) -> Result<UnifiedRegistry, RegistryError> {
    build_registry(qemu_repo, version_tags, false)
}

/// Build a unified registry from the QGA schemas of the given release tags.
/// Releases that predate the guest agent's schema are left out.
pub fn build_qga_registry(
    qemu_repo: &Path,
    version_tags: &[&str],
) -> Result<UnifiedRegistry, RegistryError> {
    build_registry(qemu_repo, version_tags, true)
}

fn build_registry(
    qemu_repo: &Path,
    version_tags: &[&str],
    qga: bool,
) -> Result<UnifiedRegistry, RegistryError> {
    let protocol = if qga { "QGA" } else { "QMP" };
    merge_releases(protocol, load_versions(qemu_repo, version_tags, qga)?)
}

/// Merge lowered releases of `protocol` in release order.
pub(super) fn merge_releases(
    protocol: &'static str,
    mut version_results: Vec<VersionResult>,
) -> Result<UnifiedRegistry, RegistryError> {
    version_results.sort_by_key(|result| result.order);

    let mut registry = UnifiedRegistry::default();
    let mut prev_hash: Option<u64> = None;
    // The first merged tag of each release, which a refused lifetime names.
    let mut tags = BTreeMap::new();
    for vr in &mut version_results {
        registry.versions.push(vr.version);
        registry.warnings.append(&mut vr.warnings);

        // Skip versions with identical schemas (patch releases)
        if prev_hash == Some(vr.schema_hash) {
            continue;
        }
        prev_hash = Some(vr.schema_hash);
        tags.entry(vr.version).or_insert_with(|| vr.tag.clone());

        // In name order, so that the refused lifetime reported is always the same.
        let mut enums: Vec<_> = vr.enums.iter().collect();
        enums.sort_by_key(|(name, _)| name.as_str());
        let mut structs: Vec<_> = vr.structs.iter().collect();
        structs.sort_by_key(|(name, _)| name.as_str());
        let mut seen_types: BTreeSet<&str> = BTreeSet::new();
        for (name, e) in enums {
            seen_types.insert(name.as_str());
            check_lifetime(&registry, protocol, name, true, &vr.tag, &tags)?;
            merge_enum(&mut registry, name, e, vr.version);
        }
        for (name, s) in structs {
            seen_types.insert(name.as_str());
            check_lifetime(&registry, protocol, name, false, &vr.tag, &tags)?;
            merge_struct(&mut registry, name, s, vr.version);
        }
        for (name, ut) in registry.types.iter_mut() {
            if !seen_types.contains(name.as_str()) {
                let vr_inner = match ut {
                    UnifiedType::Enum(e) => &mut e.versions,
                    UnifiedType::Struct(s) => &mut s.versions,
                };
                if vr_inner.until.is_none() && vr_inner.since < vr.version {
                    vr_inner.until = Some(vr.version);
                }
            }
        }
    }

    materialize_variant_history(&mut registry);
    materialize_field_history(&mut registry);
    fix_recursive_types(&mut registry);
    Ok(registry)
}
