//! The process-wide default version. It is global to the process, so this
//! test binary holds the only test that sets it.

use qapi_core::{
    VersionAlreadySet, qemu_version, scoped_qemu_version, set_qemu_version, with_version,
};

#[test]
fn the_process_default_is_set_once_and_yields_to_a_scope() {
    assert_eq!(qemu_version(), (0, 0, 0));
    set_qemu_version((10, 2, 0)).unwrap();
    assert_eq!(qemu_version(), (10, 2, 0));
    assert_eq!(scoped_qemu_version(), None);
    let refused: VersionAlreadySet = set_qemu_version((10, 2, 0)).unwrap_err();
    assert_eq!(
        (refused.existing, refused.requested),
        ((10, 2, 0), (10, 2, 0))
    );
    with_version((11, 1, 3), || {
        assert_eq!(qemu_version(), (11, 1, 3));
        assert_eq!(scoped_qemu_version(), Some((11, 1, 3)));
    });
    assert_eq!(qemu_version(), (10, 2, 0));
}
