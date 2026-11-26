use qapi_macros::qapi;
use super::*;
/// QMP command `add_client`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "add_client")]
#[qapi(returns = "()")]
#[qapi(added = "1.3.0")]
pub struct AddClient {
    pub fdname: String,
    pub protocol: String,
    pub skipauth: Option<bool>,
    pub tls: Option<bool>,
}

/// QMP command `add-fd`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "add-fd")]
#[qapi(returns = "AddfdInfo")]
#[qapi(added = "1.2.0")]
pub struct AddFd {
    #[qapi(name = "fdset-id")]
    pub fdset_id: Option<i64>,
    pub opaque: Option<String>,
}

/// QMP command `announce-self`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "announce-self")]
#[qapi(returns = "()")]
#[qapi(added = "4.0.0")]
pub struct AnnounceSelf {
    #[qapi(added = "4.1.0")]
    pub id: Option<String>,
    pub initial: i64,
    #[qapi(added = "4.1.0")]
    pub interfaces: Option<Vec<String>>,
    pub max: i64,
    pub rounds: i64,
    pub step: i64,
}

/// QMP command `balloon`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "balloon")]
#[qapi(returns = "()")]
#[qapi(added = "1.1.0")]
pub struct Balloon {
    pub value: i64,
}

/// QMP command `block-commit`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "block-commit")]
#[qapi(returns = "()")]
#[qapi(added = "1.3.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct BlockCommit {
    #[qapi(name = "auto-dismiss")]
    #[qapi(added = "3.1.0")]
    pub auto_dismiss: Option<bool>,
    #[qapi(name = "auto-finalize")]
    #[qapi(added = "3.1.0")]
    pub auto_finalize: Option<bool>,
    #[qapi(name = "backing-file")]
    #[qapi(added = "2.1.0")]
    pub backing_file: Option<String>,
    #[qapi(name = "backing-mask-protocol")]
    #[qapi(added = "9.0.0")]
    pub backing_mask_protocol: Option<bool>,
    #[qapi(deprecated(added = "5.0.0"))]
    pub base: Option<String>,
    #[qapi(name = "base-node")]
    #[qapi(added = "3.1.0")]
    pub base_node: Option<String>,
    pub device: String,
    #[qapi(name = "filter-node-name")]
    #[qapi(added = "2.9.0")]
    pub filter_node_name: Option<String>,
    #[qapi(name = "job-id")]
    #[qapi(added = "2.7.0")]
    pub job_id: Option<String>,
    #[qapi(name = "on-error")]
    #[qapi(added = "5.0.0")]
    pub on_error: Option<BlockdevOnError>,
    pub speed: Option<i64>,
    #[qapi(added = "2.0.1")]
    #[qapi(deprecated(added = "5.0.0"))]
    pub top: Option<String>,
    #[qapi(name = "top-node")]
    #[qapi(added = "3.1.0")]
    pub top_node: Option<String>,
    #[qapi(name = "top")]
    #[qapi(removed = "2.0.1")]
    #[qapi(required)]
    pub top_legacy: Option<String>,
}

/// QMP command `blockdev-add`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "blockdev-add")]
#[qapi(returns = "()")]
#[qapi(added = "1.7.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct BlockdevAdd {
    #[qapi(added = "10.0.0")]
    pub active: Option<bool>,
    #[qapi(name = "auto-read-only")]
    #[qapi(added = "3.1.0")]
    pub auto_read_only: Option<bool>,
    #[qapi(added = "2.8.0")]
    pub cache: Option<BlockdevCacheOptions>,
    #[qapi(name = "detect-zeroes")]
    #[qapi(added = "2.8.0")]
    pub detect_zeroes: Option<BlockdevDetectZeroesOptions>,
    #[qapi(added = "2.8.0")]
    pub discard: Option<BlockdevDiscardOptions>,
    #[qapi(discriminator)]
    #[qapi(added = "2.8.0")]
    #[qapi(required)]
    pub driver: Option<BlockdevDriver>,
    #[qapi(name = "force-share")]
    #[qapi(added = "2.10.0")]
    pub force_share: Option<bool>,
    #[qapi(name = "node-name")]
    #[qapi(added = "2.8.0")]
    pub node_name: Option<String>,
    #[qapi(removed = "2.8.0")]
    #[qapi(required)]
    pub options: Option<BlockdevOptions>,
    #[qapi(name = "read-only")]
    #[qapi(added = "2.8.0")]
    pub read_only: Option<bool>,
    #[qapi(union)]
    #[qapi(flatten)]
    #[qapi(added = "2.8.0")]
    pub u: Option<BlockdevOptionsBranch>,
}

/// QMP command `blockdev-backup`; see the QEMU QMP Reference Manual.
#[qapi(name = "blockdev-backup")]
#[qapi(command)]
#[qapi(returns = "()")]
#[qapi(added = "2.3.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct BlockdevBackup {
    #[qapi(name = "auto-dismiss")]
    #[qapi(added = "2.12.0")]
    pub auto_dismiss: Option<bool>,
    #[qapi(name = "auto-finalize")]
    #[qapi(added = "2.12.0")]
    pub auto_finalize: Option<bool>,
    #[qapi(added = "3.1.0")]
    pub bitmap: Option<String>,
    #[qapi(name = "bitmap-mode")]
    #[qapi(added = "4.2.0")]
    pub bitmap_mode: Option<BitmapSyncMode>,
    #[qapi(added = "2.8.0")]
    pub compress: Option<bool>,
    pub device: String,
    #[qapi(name = "discard-source")]
    #[qapi(added = "9.1.0")]
    pub discard_source: Option<bool>,
    #[qapi(name = "filter-node-name")]
    #[qapi(added = "4.2.0")]
    pub filter_node_name: Option<String>,
    #[qapi(name = "job-id")]
    #[qapi(added = "2.7.0")]
    pub job_id: Option<String>,
    #[qapi(name = "on-cbw-error")]
    #[qapi(added = "10.1.0")]
    pub on_cbw_error: Option<OnCbwError>,
    #[qapi(name = "on-source-error")]
    pub on_source_error: Option<BlockdevOnError>,
    #[qapi(name = "on-target-error")]
    pub on_target_error: Option<BlockdevOnError>,
    pub speed: Option<i64>,
    pub sync: MirrorSyncMode,
    pub target: String,
    #[qapi(name = "x-perf")]
    #[qapi(feature = "unstable")]
    #[qapi(added = "6.0.0")]
    pub x_perf: Option<BackupPerf>,
}

/// QMP command `blockdev-change-medium`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "blockdev-change-medium")]
#[qapi(returns = "()")]
#[qapi(added = "2.5.0")]
pub struct BlockdevChangeMedium {
    #[qapi(added = "2.8.0")]
    #[qapi(deprecated(added = "5.0.0"))]
    pub device: Option<String>,
    #[qapi(name = "device")]
    #[qapi(removed = "2.8.0")]
    #[qapi(required)]
    pub device_legacy: Option<String>,
    pub filename: String,
    #[qapi(added = "7.1.0")]
    pub force: Option<bool>,
    pub format: Option<String>,
    #[qapi(added = "2.8.0")]
    pub id: Option<String>,
    #[qapi(name = "read-only-mode")]
    pub read_only_mode: Option<BlockdevChangeReadOnlyMode>,
}

/// QMP command `blockdev-close-tray`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "blockdev-close-tray")]
#[qapi(returns = "()")]
#[qapi(added = "2.5.0")]
pub struct BlockdevCloseTray {
    #[qapi(added = "2.8.0")]
    #[qapi(deprecated(added = "5.0.0"))]
    pub device: Option<String>,
    #[qapi(name = "device")]
    #[qapi(removed = "2.8.0")]
    #[qapi(required)]
    pub device_legacy: Option<String>,
    #[qapi(added = "2.8.0")]
    pub id: Option<String>,
}

/// QMP command `blockdev-create`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "blockdev-create")]
#[qapi(returns = "()")]
#[qapi(added = "3.0.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct BlockdevCreate {
    #[qapi(name = "job-id")]
    pub job_id: String,
    pub options: BlockdevCreateOptions,
}

/// QMP command `blockdev-del`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "blockdev-del")]
#[qapi(returns = "()")]
#[qapi(added = "2.9.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct BlockdevDel {
    #[qapi(name = "node-name")]
    pub node_name: String,
}

/// QMP command `blockdev-insert-medium`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "blockdev-insert-medium")]
#[qapi(returns = "()")]
#[qapi(added = "2.12.0")]
pub struct BlockdevInsertMedium {
    pub id: String,
    #[qapi(name = "node-name")]
    pub node_name: String,
}

/// QMP command `blockdev-mirror`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "blockdev-mirror")]
#[qapi(returns = "()")]
#[qapi(added = "2.6.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct BlockdevMirror {
    #[qapi(name = "auto-dismiss")]
    #[qapi(added = "3.1.0")]
    pub auto_dismiss: Option<bool>,
    #[qapi(name = "auto-finalize")]
    #[qapi(added = "3.1.0")]
    pub auto_finalize: Option<bool>,
    #[qapi(name = "buf-size")]
    pub buf_size: Option<i64>,
    #[qapi(name = "copy-mode")]
    #[qapi(added = "3.0.0")]
    pub copy_mode: Option<MirrorCopyMode>,
    pub device: String,
    #[qapi(name = "filter-node-name")]
    #[qapi(added = "2.9.0")]
    pub filter_node_name: Option<String>,
    pub granularity: Option<u32>,
    #[qapi(name = "job-id")]
    #[qapi(added = "2.7.0")]
    pub job_id: Option<String>,
    #[qapi(name = "on-source-error")]
    pub on_source_error: Option<BlockdevOnError>,
    #[qapi(name = "on-target-error")]
    pub on_target_error: Option<BlockdevOnError>,
    pub replaces: Option<String>,
    pub speed: Option<i64>,
    pub sync: MirrorSyncMode,
    pub target: String,
    #[qapi(name = "target-is-zero")]
    #[qapi(added = "10.1.0")]
    pub target_is_zero: Option<bool>,
}

/// QMP command `blockdev-open-tray`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "blockdev-open-tray")]
#[qapi(returns = "()")]
#[qapi(added = "2.5.0")]
pub struct BlockdevOpenTray {
    #[qapi(added = "2.8.0")]
    #[qapi(deprecated(added = "5.0.0"))]
    pub device: Option<String>,
    #[qapi(name = "device")]
    #[qapi(removed = "2.8.0")]
    #[qapi(required)]
    pub device_legacy: Option<String>,
    pub force: Option<bool>,
    #[qapi(added = "2.8.0")]
    pub id: Option<String>,
}

/// QMP command `blockdev-remove-medium`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "blockdev-remove-medium")]
#[qapi(returns = "()")]
#[qapi(added = "2.12.0")]
pub struct BlockdevRemoveMedium {
    pub id: String,
}

/// QMP command `blockdev-reopen`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "blockdev-reopen")]
#[qapi(returns = "()")]
#[qapi(added = "6.1.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct BlockdevReopen {
    pub options: Vec<BlockdevOptions>,
}

/// QMP command `blockdev-set-active`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "blockdev-set-active")]
#[qapi(returns = "()")]
#[qapi(added = "10.0.0")]
#[qapi(allow_preconfig)]
pub struct BlockdevSetActive {
    pub active: bool,
    #[qapi(name = "node-name")]
    pub node_name: Option<String>,
}

/// QMP command `blockdev-snapshot`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(returns = "()")]
#[qapi(name = "blockdev-snapshot")]
#[qapi(added = "2.5.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct BlockdevSnapshotCommand {
    pub node: String,
    pub overlay: String,
}

/// QMP command `blockdev-snapshot-delete-internal-sync`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "blockdev-snapshot-delete-internal-sync")]
#[qapi(returns = "SnapshotInfo")]
#[qapi(added = "1.7.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct BlockdevSnapshotDeleteInternalSync {
    pub device: String,
    pub id: Option<String>,
    pub name: Option<String>,
}

/// QMP command `blockdev-snapshot-internal-sync`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "blockdev-snapshot-internal-sync")]
#[qapi(returns = "()")]
#[qapi(added = "1.7.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct BlockdevSnapshotInternalSync {
    pub device: String,
    pub name: String,
}

/// QMP command `blockdev-snapshot-sync`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "blockdev-snapshot-sync")]
#[qapi(returns = "()")]
#[qapi(added = "1.1.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct BlockdevSnapshotSync {
    #[qapi(added = "2.0.0")]
    pub device: Option<String>,
    #[qapi(name = "device")]
    #[qapi(removed = "2.0.0")]
    #[qapi(required)]
    pub device_legacy: Option<String>,
    pub format: Option<String>,
    pub mode: Option<NewImageMode>,
    #[qapi(name = "node-name")]
    #[qapi(added = "2.0.0")]
    pub node_name: Option<String>,
    #[qapi(name = "snapshot-file")]
    pub snapshot_file: String,
    #[qapi(name = "snapshot-node-name")]
    #[qapi(added = "2.0.0")]
    pub snapshot_node_name: Option<String>,
}

/// QMP command `block-dirty-bitmap-add`; see the QEMU QMP Reference Manual.
#[qapi(name = "block-dirty-bitmap-add")]
#[qapi(command)]
#[qapi(returns = "()")]
#[qapi(added = "2.4.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct BlockDirtyBitmapAdd {
    #[qapi(added = "2.10.0")]
    #[qapi(removed = "4.2.0")]
    pub autoload: Option<bool>,
    #[qapi(added = "4.0.0")]
    pub disabled: Option<bool>,
    pub granularity: Option<u32>,
    pub name: String,
    pub node: String,
    #[qapi(added = "2.10.0")]
    pub persistent: Option<bool>,
    #[qapi(name = "x-disabled")]
    #[qapi(added = "3.0.0")]
    #[qapi(removed = "4.0.0")]
    pub x_disabled: Option<bool>,
}

/// QMP command `block-dirty-bitmap-clear`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "block-dirty-bitmap-clear")]
#[qapi(returns = "()")]
#[qapi(added = "2.4.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct BlockDirtyBitmapClear {
    pub name: String,
    pub node: String,
}

/// QMP command `block-dirty-bitmap-disable`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "block-dirty-bitmap-disable")]
#[qapi(returns = "()")]
#[qapi(added = "4.0.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct BlockDirtyBitmapDisable {
    pub name: String,
    pub node: String,
}

/// QMP command `block-dirty-bitmap-enable`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "block-dirty-bitmap-enable")]
#[qapi(returns = "()")]
#[qapi(added = "4.0.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct BlockDirtyBitmapEnable {
    pub name: String,
    pub node: String,
}

/// QMP command `block-dirty-bitmap-merge`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(returns = "()")]
#[qapi(name = "block-dirty-bitmap-merge")]
#[qapi(added = "4.0.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct BlockDirtyBitmapMergeCommand {
    #[qapi(added = "7.1.0")]
    #[qapi(required)]
    pub bitmaps: Option<Vec<BlockDirtyBitmapOrStr>>,
    #[qapi(name = "bitmaps")]
    #[qapi(removed = "4.1.0")]
    #[qapi(required)]
    pub bitmaps_legacy: Option<Vec<String>>,
    #[qapi(name = "bitmaps")]
    #[qapi(added = "4.1.0")]
    #[qapi(removed = "7.1.0")]
    #[qapi(required)]
    pub bitmaps_legacy2: Option<Vec<BlockDirtyBitmapMergeSource>>,
    pub node: String,
    pub target: String,
}

/// QMP command `block-dirty-bitmap-remove`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "block-dirty-bitmap-remove")]
#[qapi(returns = "()")]
#[qapi(added = "2.4.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct BlockDirtyBitmapRemove {
    pub name: String,
    pub node: String,
}

/// QMP command `block-export-add`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "block-export-add")]
#[qapi(returns = "()")]
#[qapi(added = "5.2.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct BlockExportAdd {
    #[qapi(name = "allow-inactive")]
    #[qapi(added = "10.0.0")]
    pub allow_inactive: Option<bool>,
    #[qapi(name = "fixed-iothread")]
    pub fixed_iothread: Option<bool>,
    pub id: String,
    #[qapi(added = "11.0.0")]
    pub iothread: Option<BlockExportIothreads>,
    #[qapi(name = "iothread")]
    #[qapi(removed = "11.0.0")]
    pub iothread_legacy: Option<String>,
    #[qapi(name = "node-name")]
    pub node_name: String,
    #[qapi(discriminator)]
    pub r#type: BlockExportType,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<BlockExportOptionsBranch>,
    pub writable: Option<bool>,
    pub writethrough: Option<bool>,
}

/// QMP command `block-export-del`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "block-export-del")]
#[qapi(returns = "()")]
#[qapi(added = "5.2.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct BlockExportDel {
    pub id: String,
    pub mode: Option<BlockExportRemoveMode>,
}

/// QMP command `block-job-cancel`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "block-job-cancel")]
#[qapi(returns = "()")]
#[qapi(added = "1.1.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct BlockJobCancel {
    pub device: String,
    #[qapi(added = "1.3.0")]
    pub force: Option<bool>,
}

/// QMP command `block-job-change`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "block-job-change")]
#[qapi(returns = "()")]
#[qapi(added = "8.2.0")]
pub struct BlockJobChange {
    pub id: String,
    #[qapi(discriminator)]
    pub r#type: JobType,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<BlockJobChangeOptionsBranch>,
}

/// QMP command `block-job-complete`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "block-job-complete")]
#[qapi(returns = "()")]
#[qapi(added = "1.3.0")]
#[qapi(deprecated(added = "10.1.0"))]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct BlockJobComplete {
    pub device: String,
}

/// QMP command `block-job-dismiss`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "block-job-dismiss")]
#[qapi(returns = "()")]
#[qapi(added = "2.12.0")]
#[qapi(deprecated(added = "10.1.0"))]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct BlockJobDismiss {
    pub id: String,
}

/// QMP command `block-job-finalize`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "block-job-finalize")]
#[qapi(returns = "()")]
#[qapi(added = "2.12.0")]
#[qapi(deprecated(added = "10.1.0"))]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct BlockJobFinalize {
    pub id: String,
}

/// QMP command `block-job-pause`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "block-job-pause")]
#[qapi(returns = "()")]
#[qapi(added = "1.3.0")]
#[qapi(deprecated(added = "10.1.0"))]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct BlockJobPause {
    pub device: String,
}

/// QMP command `block-job-resume`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "block-job-resume")]
#[qapi(returns = "()")]
#[qapi(added = "1.3.0")]
#[qapi(deprecated(added = "10.1.0"))]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct BlockJobResume {
    pub device: String,
}

/// QMP command `block-job-set-speed`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "block-job-set-speed")]
#[qapi(returns = "()")]
#[qapi(added = "1.1.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct BlockJobSetSpeed {
    pub device: String,
    pub speed: i64,
}

/// QMP command `block-latency-histogram-set`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "block-latency-histogram-set")]
#[qapi(returns = "()")]
#[qapi(added = "4.0.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct BlockLatencyHistogramSet {
    pub boundaries: Option<Vec<u64>>,
    #[qapi(name = "boundaries-flush")]
    pub boundaries_flush: Option<Vec<u64>>,
    #[qapi(name = "boundaries-read")]
    pub boundaries_read: Option<Vec<u64>>,
    #[qapi(name = "boundaries-write")]
    pub boundaries_write: Option<Vec<u64>>,
    #[qapi(name = "boundaries-zap")]
    #[qapi(added = "8.1.0")]
    pub boundaries_zap: Option<Vec<u64>>,
    pub id: String,
}

/// QMP command `block_passwd`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "block_passwd")]
#[qapi(returns = "()")]
#[qapi(added = "1.1.0")]
#[qapi(removed = "6.0.0")]
pub struct BlockPasswd {
    #[qapi(added = "2.0.0")]
    pub device: Option<String>,
    #[qapi(name = "device")]
    #[qapi(removed = "2.0.0")]
    #[qapi(required)]
    pub device_legacy: Option<String>,
    #[qapi(name = "node-name")]
    #[qapi(added = "2.0.0")]
    pub node_name: Option<String>,
    pub password: String,
}

/// QMP command `block_resize`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "block_resize")]
#[qapi(returns = "()")]
#[qapi(added = "1.1.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct BlockResize {
    #[qapi(added = "2.0.0")]
    pub device: Option<String>,
    #[qapi(name = "device")]
    #[qapi(removed = "2.0.0")]
    #[qapi(required)]
    pub device_legacy: Option<String>,
    #[qapi(name = "node-name")]
    #[qapi(added = "2.0.0")]
    pub node_name: Option<String>,
    pub size: i64,
}

/// QMP command `block_set_io_throttle`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "block_set_io_throttle")]
#[qapi(returns = "()")]
#[qapi(added = "1.1.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct BlockSetIoThrottle {
    pub bps: i64,
    #[qapi(added = "1.7.0")]
    pub bps_max: Option<i64>,
    #[qapi(added = "2.6.0")]
    pub bps_max_length: Option<i64>,
    pub bps_rd: i64,
    #[qapi(added = "1.7.0")]
    pub bps_rd_max: Option<i64>,
    #[qapi(added = "2.6.0")]
    pub bps_rd_max_length: Option<i64>,
    pub bps_wr: i64,
    #[qapi(added = "1.7.0")]
    pub bps_wr_max: Option<i64>,
    #[qapi(added = "2.6.0")]
    pub bps_wr_max_length: Option<i64>,
    #[qapi(added = "2.8.0")]
    #[qapi(deprecated(added = "5.0.0"))]
    pub device: Option<String>,
    #[qapi(name = "device")]
    #[qapi(removed = "2.8.0")]
    #[qapi(required)]
    pub device_legacy: Option<String>,
    #[qapi(added = "2.4.0")]
    pub group: Option<String>,
    #[qapi(added = "2.8.0")]
    pub id: Option<String>,
    pub iops: i64,
    #[qapi(added = "1.7.0")]
    pub iops_max: Option<i64>,
    #[qapi(added = "2.6.0")]
    pub iops_max_length: Option<i64>,
    pub iops_rd: i64,
    #[qapi(added = "1.7.0")]
    pub iops_rd_max: Option<i64>,
    #[qapi(added = "2.6.0")]
    pub iops_rd_max_length: Option<i64>,
    #[qapi(added = "1.7.0")]
    pub iops_size: Option<i64>,
    pub iops_wr: i64,
    #[qapi(added = "1.7.0")]
    pub iops_wr_max: Option<i64>,
    #[qapi(added = "2.6.0")]
    pub iops_wr_max_length: Option<i64>,
}

/// QMP command `block-set-write-threshold`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "block-set-write-threshold")]
#[qapi(returns = "()")]
#[qapi(added = "2.3.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct BlockSetWriteThreshold {
    #[qapi(name = "node-name")]
    pub node_name: String,
    #[qapi(name = "write-threshold")]
    pub write_threshold: u64,
}

/// QMP command `block-stream`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "block-stream")]
#[qapi(returns = "()")]
#[qapi(added = "1.1.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct BlockStream {
    #[qapi(name = "auto-dismiss")]
    #[qapi(added = "3.1.0")]
    pub auto_dismiss: Option<bool>,
    #[qapi(name = "auto-finalize")]
    #[qapi(added = "3.1.0")]
    pub auto_finalize: Option<bool>,
    #[qapi(name = "backing-file")]
    #[qapi(added = "2.1.0")]
    pub backing_file: Option<String>,
    #[qapi(name = "backing-mask-protocol")]
    #[qapi(added = "9.0.0")]
    pub backing_mask_protocol: Option<bool>,
    pub base: Option<String>,
    #[qapi(name = "base-node")]
    #[qapi(added = "2.8.0")]
    pub base_node: Option<String>,
    #[qapi(added = "6.0.0")]
    pub bottom: Option<String>,
    pub device: String,
    #[qapi(name = "filter-node-name")]
    #[qapi(added = "6.0.0")]
    pub filter_node_name: Option<String>,
    #[qapi(name = "job-id")]
    #[qapi(added = "2.7.0")]
    pub job_id: Option<String>,
    #[qapi(name = "on-error")]
    #[qapi(added = "1.3.0")]
    pub on_error: Option<BlockdevOnError>,
    pub speed: Option<i64>,
}

/// QMP command `calc-dirty-rate`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "calc-dirty-rate")]
#[qapi(returns = "()")]
#[qapi(added = "5.2.0")]
pub struct CalcDirtyRate {
    #[qapi(name = "calc-time")]
    pub calc_time: i64,
    #[qapi(name = "calc-time-unit")]
    #[qapi(added = "8.2.0")]
    pub calc_time_unit: Option<TimeUnit>,
    #[qapi(added = "6.2.0")]
    pub mode: Option<DirtyRateMeasureMode>,
    #[qapi(name = "sample-pages")]
    #[qapi(added = "6.1.0")]
    pub sample_pages: Option<i64>,
}

/// QMP command `cancel-vcpu-dirty-limit`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "cancel-vcpu-dirty-limit")]
#[qapi(returns = "()")]
#[qapi(added = "7.1.0")]
pub struct CancelVcpuDirtyLimit {
    #[qapi(name = "cpu-index")]
    pub cpu_index: Option<i64>,
}

/// QMP command `change`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "change")]
#[qapi(returns = "()")]
#[qapi(added = "1.1.0")]
#[qapi(deprecated(added = "5.0.0"))]
#[qapi(removed = "6.0.0")]
pub struct Change {
    pub arg: Option<String>,
    pub device: String,
    pub target: String,
}

/// QMP command `change-backing-file`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "change-backing-file")]
#[qapi(returns = "()")]
#[qapi(added = "2.1.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct ChangeBackingFile {
    #[qapi(name = "backing-file")]
    pub backing_file: String,
    pub device: String,
    #[qapi(name = "image-node-name")]
    pub image_node_name: String,
}

/// QMP command `change-vnc-password`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "change-vnc-password")]
#[qapi(returns = "()")]
#[qapi(condition = "CONFIG_VNC")]
#[qapi(added = "1.1.0")]
pub struct ChangeVncPassword {
    pub password: String,
}

/// QMP command `chardev-add`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "chardev-add")]
#[qapi(returns = "ChardevReturn")]
#[qapi(added = "1.4.0")]
pub struct ChardevAdd {
    pub backend: ChardevBackend,
    pub id: String,
}

/// QMP command `chardev-change`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "chardev-change")]
#[qapi(returns = "ChardevReturn")]
#[qapi(added = "2.10.0")]
pub struct ChardevChange {
    pub backend: ChardevBackend,
    pub id: String,
}

/// QMP command `chardev-remove`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "chardev-remove")]
#[qapi(returns = "()")]
#[qapi(added = "1.4.0")]
pub struct ChardevRemove {
    pub id: String,
}

/// QMP command `chardev-send-break`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "chardev-send-break")]
#[qapi(returns = "()")]
#[qapi(added = "2.10.0")]
pub struct ChardevSendBreak {
    pub id: String,
}

/// QMP command `client_migrate_info`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "client_migrate_info")]
#[qapi(returns = "()")]
#[qapi(added = "2.4.0")]
pub struct ClientMigrateInfo {
    #[qapi(name = "cert-subject")]
    pub cert_subject: Option<String>,
    pub hostname: String,
    pub port: Option<i64>,
    pub protocol: String,
    #[qapi(name = "tls-port")]
    pub tls_port: Option<i64>,
}

/// QMP command `closefd`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "closefd")]
#[qapi(returns = "()")]
#[qapi(added = "1.2.0")]
pub struct Closefd {
    pub fdname: String,
}

/// QMP command `cont`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "cont")]
#[qapi(returns = "()")]
#[qapi(added = "1.1.0")]
pub struct Cont {}

/// QMP command `cpu`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "cpu")]
#[qapi(returns = "()")]
#[qapi(added = "1.0.0")]
#[qapi(removed = "2.12.0")]
pub struct Cpu {
    pub index: i64,
}

/// QMP command `cpu-add`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "cpu-add")]
#[qapi(returns = "()")]
#[qapi(added = "1.5.0")]
#[qapi(deprecated(added = "5.0.0"))]
#[qapi(removed = "5.2.0")]
pub struct CpuAdd {
    pub id: i64,
}

/// QMP command `cxl-add-dynamic-capacity`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "cxl-add-dynamic-capacity")]
#[qapi(returns = "()")]
#[qapi(added = "9.1.0")]
#[qapi(unstable(added = "9.1.0"))]
pub struct CxlAddDynamicCapacity {
    pub extents: Vec<CxlDynamicCapacityExtent>,
    #[qapi(name = "host-id")]
    pub host_id: u16,
    pub path: String,
    pub region: u8,
    #[qapi(name = "selection-policy")]
    pub selection_policy: CxlExtentSelectionPolicy,
    pub tag: Option<String>,
}

/// QMP command `cxl-inject-correctable-error`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "cxl-inject-correctable-error")]
#[qapi(returns = "()")]
#[qapi(added = "8.0.0")]
pub struct CxlInjectCorrectableError {
    pub path: String,
    pub r#type: CxlCorErrorType,
}

/// QMP command `cxl-inject-dram-event`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "cxl-inject-dram-event")]
#[qapi(returns = "()")]
#[qapi(added = "8.1.0")]
pub struct CxlInjectDramEvent {
    pub bank: Option<u8>,
    #[qapi(name = "bank-group")]
    pub bank_group: Option<u8>,
    pub channel: Option<u8>,
    #[qapi(name = "cme-ev-flags")]
    #[qapi(added = "11.0.0")]
    pub cme_ev_flags: Option<u8>,
    pub column: Option<u16>,
    #[qapi(name = "component-id")]
    #[qapi(added = "11.0.0")]
    pub component_id: Option<String>,
    #[qapi(name = "correction-mask")]
    pub correction_mask: Option<Vec<u64>>,
    #[qapi(name = "cvme-count")]
    #[qapi(added = "11.0.0")]
    pub cvme_count: Option<u32>,
    pub descriptor: u8,
    pub dpa: u64,
    #[qapi(added = "11.0.0")]
    #[qapi(required)]
    pub flags: Option<u32>,
    #[qapi(name = "flags")]
    #[qapi(removed = "11.0.0")]
    #[qapi(required)]
    pub flags_legacy: Option<u8>,
    #[qapi(name = "head-id")]
    #[qapi(added = "11.0.0")]
    pub head_id: Option<u8>,
    #[qapi(name = "is-comp-id-pldm")]
    #[qapi(added = "11.0.0")]
    pub is_comp_id_pldm: Option<bool>,
    #[qapi(name = "ld-id")]
    #[qapi(added = "11.0.0")]
    pub ld_id: Option<u16>,
    pub log: CxlEventLog,
    #[qapi(name = "maint-op-class")]
    #[qapi(added = "11.0.0")]
    pub maint_op_class: Option<u8>,
    #[qapi(name = "maint-op-subclass")]
    #[qapi(added = "11.0.0")]
    pub maint_op_subclass: Option<u8>,
    #[qapi(name = "nibble-mask")]
    pub nibble_mask: Option<u32>,
    pub path: String,
    pub rank: Option<u8>,
    pub row: Option<u32>,
    #[qapi(name = "sub-channel")]
    #[qapi(added = "11.0.0")]
    pub sub_channel: Option<u8>,
    #[qapi(name = "sub-type")]
    #[qapi(added = "11.0.0")]
    #[qapi(required)]
    pub sub_type: Option<u8>,
    #[qapi(name = "transaction-type")]
    pub transaction_type: u8,
    pub r#type: u8,
}

/// QMP command `cxl-inject-general-media-event`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "cxl-inject-general-media-event")]
#[qapi(returns = "()")]
#[qapi(added = "8.1.0")]
pub struct CxlInjectGeneralMediaEvent {
    pub channel: Option<u8>,
    #[qapi(name = "cme-count")]
    #[qapi(added = "11.0.0")]
    pub cme_count: Option<u32>,
    #[qapi(name = "cme-ev-flags")]
    #[qapi(added = "11.0.0")]
    pub cme_ev_flags: Option<u8>,
    #[qapi(name = "component-id")]
    pub component_id: Option<String>,
    pub descriptor: u8,
    pub device: Option<u32>,
    pub dpa: u64,
    #[qapi(added = "11.0.0")]
    #[qapi(required)]
    pub flags: Option<u32>,
    #[qapi(name = "flags")]
    #[qapi(removed = "11.0.0")]
    #[qapi(required)]
    pub flags_legacy: Option<u8>,
    #[qapi(name = "head-id")]
    #[qapi(added = "11.0.0")]
    pub head_id: Option<u8>,
    #[qapi(name = "is-comp-id-pldm")]
    #[qapi(added = "11.0.0")]
    pub is_comp_id_pldm: Option<bool>,
    #[qapi(name = "ld-id")]
    #[qapi(added = "11.0.0")]
    pub ld_id: Option<u16>,
    pub log: CxlEventLog,
    #[qapi(name = "maint-op-class")]
    #[qapi(added = "11.0.0")]
    pub maint_op_class: Option<u8>,
    #[qapi(name = "maint-op-subclass")]
    #[qapi(added = "11.0.0")]
    pub maint_op_subclass: Option<u8>,
    pub path: String,
    pub rank: Option<u8>,
    #[qapi(name = "sub-type")]
    #[qapi(added = "11.0.0")]
    #[qapi(required)]
    pub sub_type: Option<u8>,
    #[qapi(name = "transaction-type")]
    pub transaction_type: u8,
    pub r#type: u8,
}

/// QMP command `cxl-inject-memory-module-event`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "cxl-inject-memory-module-event")]
#[qapi(returns = "()")]
#[qapi(added = "8.1.0")]
pub struct CxlInjectMemoryModuleEvent {
    #[qapi(name = "additional-status")]
    pub additional_status: u8,
    #[qapi(name = "component-id")]
    #[qapi(added = "11.0.0")]
    pub component_id: Option<String>,
    #[qapi(name = "corrected-persistent-error-count")]
    pub corrected_persistent_error_count: u32,
    #[qapi(name = "corrected-volatile-error-count")]
    pub corrected_volatile_error_count: u32,
    #[qapi(name = "dirty-shutdown-count")]
    pub dirty_shutdown_count: u32,
    #[qapi(added = "11.0.0")]
    #[qapi(required)]
    pub flags: Option<u32>,
    #[qapi(name = "flags")]
    #[qapi(removed = "11.0.0")]
    #[qapi(required)]
    pub flags_legacy: Option<u8>,
    #[qapi(name = "head-id")]
    #[qapi(added = "11.0.0")]
    pub head_id: Option<u8>,
    #[qapi(name = "health-status")]
    pub health_status: u8,
    #[qapi(name = "is-comp-id-pldm")]
    #[qapi(added = "11.0.0")]
    pub is_comp_id_pldm: Option<bool>,
    #[qapi(name = "ld-id")]
    #[qapi(added = "11.0.0")]
    pub ld_id: Option<u16>,
    #[qapi(name = "life-used")]
    pub life_used: u8,
    pub log: CxlEventLog,
    #[qapi(name = "maint-op-class")]
    #[qapi(added = "11.0.0")]
    pub maint_op_class: Option<u8>,
    #[qapi(name = "maint-op-subclass")]
    #[qapi(added = "11.0.0")]
    pub maint_op_subclass: Option<u8>,
    #[qapi(name = "media-status")]
    pub media_status: u8,
    pub path: String,
    #[qapi(name = "sub-type")]
    #[qapi(added = "11.0.0")]
    #[qapi(required)]
    pub sub_type: Option<u8>,
    pub temperature: i16,
    pub r#type: u8,
}

/// QMP command `cxl-inject-poison`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "cxl-inject-poison")]
#[qapi(returns = "()")]
#[qapi(added = "8.1.0")]
pub struct CxlInjectPoison {
    pub length: u64,
    pub path: String,
    pub start: u64,
}

/// QMP command `cxl-inject-uncorrectable-errors`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "cxl-inject-uncorrectable-errors")]
#[qapi(returns = "()")]
#[qapi(added = "8.0.0")]
pub struct CxlInjectUncorrectableErrors {
    pub errors: Vec<CxlUncorErrorRecord>,
    pub path: String,
}

/// QMP command `cxl-release-dynamic-capacity`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "cxl-release-dynamic-capacity")]
#[qapi(returns = "()")]
#[qapi(added = "9.1.0")]
#[qapi(unstable(added = "9.1.0"))]
pub struct CxlReleaseDynamicCapacity {
    pub extents: Vec<CxlDynamicCapacityExtent>,
    #[qapi(name = "forced-removal")]
    pub forced_removal: Option<bool>,
    #[qapi(name = "host-id")]
    pub host_id: u16,
    pub path: String,
    pub region: u8,
    #[qapi(name = "removal-policy")]
    pub removal_policy: CxlExtentRemovalPolicy,
    #[qapi(name = "sanitize-on-release")]
    pub sanitize_on_release: Option<bool>,
    pub tag: Option<String>,
}

/// QMP command `device_add`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "device_add")]
#[qapi(returns = "()")]
#[qapi(added = "2.8.0")]
pub struct DeviceAdd {
    /// Additional command arguments not modeled by this schema.
    #[qapi(flatten)]
    pub arguments: Option<serde_json::Map<String, serde_json::Value>>,
    #[qapi(added = "2.9.0")]
    pub bus: Option<String>,
    pub driver: String,
    #[qapi(added = "2.9.0")]
    pub id: Option<String>,
    #[qapi(name = "id")]
    #[qapi(removed = "2.9.0")]
    #[qapi(required)]
    pub id_legacy: Option<String>,
}

/// QMP command `device_del`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "device_del")]
#[qapi(returns = "()")]
#[qapi(added = "1.1.0")]
pub struct DeviceDel {
    pub id: String,
}

/// QMP command `device-list-properties`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "device-list-properties")]
#[qapi(returns = "Vec<ObjectPropertyInfo>")]
#[qapi(added = "1.2.0")]
#[qapi(returns(added = "1.2.0", value = "Vec<DevicePropertyInfo>"))]
#[qapi(returns(added = "2.12.0", value = "Vec<ObjectPropertyInfo>"))]
#[qapi(returns_enum = "DeviceListPropertiesReturns")]
pub struct DeviceListProperties {
    pub typename: String,
}

/// QMP command `device-sync-config`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "device-sync-config")]
#[qapi(returns = "()")]
#[qapi(added = "9.2.0")]
#[qapi(unstable(added = "9.2.0"))]
pub struct DeviceSyncConfig {
    pub id: String,
}

/// QMP command `display-reload`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "display-reload")]
#[qapi(returns = "()")]
#[qapi(added = "6.0.0")]
pub struct DisplayReload {
    #[qapi(discriminator)]
    pub r#type: DisplayReloadType,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<DisplayReloadOptionsBranch>,
}

/// QMP command `display-update`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "display-update")]
#[qapi(returns = "()")]
#[qapi(added = "7.1.0")]
pub struct DisplayUpdate {
    #[qapi(discriminator)]
    pub r#type: DisplayUpdateType,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<DisplayUpdateOptionsBranch>,
}

/// QMP command `drive-backup`; see the QEMU QMP Reference Manual.
#[qapi(name = "drive-backup")]
#[qapi(command)]
#[qapi(returns = "()")]
#[qapi(added = "1.6.0")]
#[qapi(deprecated(added = "6.2.0"))]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct DriveBackup {
    #[qapi(name = "auto-dismiss")]
    #[qapi(added = "2.12.0")]
    pub auto_dismiss: Option<bool>,
    #[qapi(name = "auto-finalize")]
    #[qapi(added = "2.12.0")]
    pub auto_finalize: Option<bool>,
    #[qapi(added = "2.4.0")]
    pub bitmap: Option<String>,
    #[qapi(name = "bitmap-mode")]
    #[qapi(added = "4.2.0")]
    pub bitmap_mode: Option<BitmapSyncMode>,
    #[qapi(added = "2.8.0")]
    pub compress: Option<bool>,
    pub device: String,
    #[qapi(name = "discard-source")]
    #[qapi(added = "9.1.0")]
    pub discard_source: Option<bool>,
    #[qapi(name = "filter-node-name")]
    #[qapi(added = "4.2.0")]
    pub filter_node_name: Option<String>,
    pub format: Option<String>,
    #[qapi(name = "job-id")]
    #[qapi(added = "2.7.0")]
    pub job_id: Option<String>,
    pub mode: Option<NewImageMode>,
    #[qapi(name = "on-cbw-error")]
    #[qapi(added = "10.1.0")]
    pub on_cbw_error: Option<OnCbwError>,
    #[qapi(name = "on-source-error")]
    pub on_source_error: Option<BlockdevOnError>,
    #[qapi(name = "on-target-error")]
    pub on_target_error: Option<BlockdevOnError>,
    pub speed: Option<i64>,
    pub sync: MirrorSyncMode,
    pub target: String,
    #[qapi(name = "x-perf")]
    #[qapi(feature = "unstable")]
    #[qapi(added = "6.0.0")]
    pub x_perf: Option<BackupPerf>,
}

/// QMP command `drive-mirror`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "drive-mirror")]
#[qapi(returns = "()")]
#[qapi(added = "1.3.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct DriveMirror {
    #[qapi(name = "auto-dismiss")]
    #[qapi(added = "3.1.0")]
    pub auto_dismiss: Option<bool>,
    #[qapi(name = "auto-finalize")]
    #[qapi(added = "3.1.0")]
    pub auto_finalize: Option<bool>,
    #[qapi(name = "buf-size")]
    #[qapi(added = "1.4.0")]
    pub buf_size: Option<i64>,
    #[qapi(name = "copy-mode")]
    #[qapi(added = "3.0.0")]
    pub copy_mode: Option<MirrorCopyMode>,
    pub device: String,
    pub format: Option<String>,
    #[qapi(added = "1.4.0")]
    pub granularity: Option<u32>,
    #[qapi(name = "job-id")]
    #[qapi(added = "2.7.0")]
    pub job_id: Option<String>,
    pub mode: Option<NewImageMode>,
    #[qapi(name = "node-name")]
    #[qapi(added = "2.1.0")]
    pub node_name: Option<String>,
    #[qapi(name = "on-source-error")]
    pub on_source_error: Option<BlockdevOnError>,
    #[qapi(name = "on-target-error")]
    pub on_target_error: Option<BlockdevOnError>,
    #[qapi(added = "2.1.0")]
    pub replaces: Option<String>,
    pub speed: Option<i64>,
    pub sync: MirrorSyncMode,
    pub target: String,
    #[qapi(added = "2.3.1")]
    pub unmap: Option<bool>,
}

/// QMP command `dumpdtb`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "dumpdtb")]
#[qapi(condition = "CONFIG_FDT")]
#[qapi(returns = "()")]
#[qapi(added = "7.2.0")]
pub struct Dumpdtb {
    pub filename: String,
}

/// QMP command `dump-guest-memory`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "dump-guest-memory")]
#[qapi(returns = "()")]
#[qapi(added = "1.2.0")]
pub struct DumpGuestMemory {
    pub begin: Option<i64>,
    #[qapi(added = "2.6.0")]
    pub detach: Option<bool>,
    #[qapi(added = "2.0.0")]
    pub format: Option<DumpGuestMemoryFormat>,
    pub length: Option<i64>,
    pub paging: bool,
    pub protocol: String,
}

/// QMP command `dump-skeys`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "dump-skeys")]
#[qapi(returns = "()")]
#[qapi(added = "2.5.0")]
pub struct DumpSkeys {
    pub filename: String,
}

/// QMP command `eject`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "eject")]
#[qapi(returns = "()")]
#[qapi(added = "1.1.0")]
pub struct Eject {
    #[qapi(added = "2.8.0")]
    #[qapi(deprecated(added = "5.0.0"))]
    pub device: Option<String>,
    #[qapi(name = "device")]
    #[qapi(removed = "2.8.0")]
    #[qapi(required)]
    pub device_legacy: Option<String>,
    pub force: Option<bool>,
    #[qapi(added = "2.8.0")]
    pub id: Option<String>,
}

/// QMP command `expire_password`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "expire_password")]
#[qapi(returns = "()")]
#[qapi(added = "1.1.0")]
pub struct ExpirePassword {
    #[qapi(discriminator)]
    #[qapi(added = "7.0.0")]
    #[qapi(required)]
    pub protocol: Option<DisplayProtocol>,
    #[qapi(name = "protocol")]
    #[qapi(removed = "7.0.0")]
    #[qapi(required)]
    pub protocol_legacy: Option<String>,
    pub time: String,
    #[qapi(union)]
    #[qapi(flatten)]
    #[qapi(added = "7.0.0")]
    pub u: Option<ExpirePasswordOptionsBranch>,
}

/// QMP command `getfd`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "getfd")]
#[qapi(returns = "()")]
#[qapi(condition = "CONFIG_POSIX")]
#[qapi(added = "1.2.0")]
pub struct Getfd {
    pub fdname: String,
}

/// QMP command `get-win32-socket`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "get-win32-socket")]
#[qapi(condition = "CONFIG_WIN32")]
#[qapi(returns = "()")]
#[qapi(added = "8.0.0")]
pub struct GetWin32Socket {
    pub fdname: String,
    pub info: String,
}

/// QMP command `human-monitor-command`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "human-monitor-command")]
#[qapi(returns = "String")]
#[qapi(added = "1.1.0")]
pub struct HumanMonitorCommand {
    #[qapi(name = "command-line")]
    pub command_line: String,
    #[qapi(name = "cpu-index")]
    pub cpu_index: Option<i64>,
}

/// QMP command `inject-ghes-v2-error`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "inject-ghes-v2-error")]
#[qapi(returns = "()")]
#[qapi(added = "10.2.0")]
#[qapi(unstable(added = "10.2.0"))]
pub struct InjectGhesV2Error {
    pub cper: String,
}

/// QMP command `inject-nmi`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "inject-nmi")]
#[qapi(returns = "()")]
#[qapi(added = "1.1.0")]
pub struct InjectNmi {}

/// QMP command `input-send-event`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "input-send-event")]
#[qapi(returns = "()")]
#[qapi(added = "2.6.0")]
pub struct InputSendEvent {
    pub device: Option<String>,
    pub events: Vec<InputEvent>,
    pub head: Option<i64>,
}

/// QMP command `job-cancel`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "job-cancel")]
#[qapi(returns = "()")]
#[qapi(added = "3.0.0")]
pub struct JobCancel {
    pub id: String,
}

/// QMP command `job-complete`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "job-complete")]
#[qapi(returns = "()")]
#[qapi(added = "3.0.0")]
pub struct JobComplete {
    pub id: String,
}

/// QMP command `job-dismiss`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "job-dismiss")]
#[qapi(returns = "()")]
#[qapi(added = "3.0.0")]
pub struct JobDismiss {
    pub id: String,
}

/// QMP command `job-finalize`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "job-finalize")]
#[qapi(returns = "()")]
#[qapi(added = "3.0.0")]
pub struct JobFinalize {
    pub id: String,
}

/// QMP command `job-pause`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "job-pause")]
#[qapi(returns = "()")]
#[qapi(added = "3.0.0")]
pub struct JobPause {
    pub id: String,
}

/// QMP command `job-resume`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "job-resume")]
#[qapi(returns = "()")]
#[qapi(added = "3.0.0")]
pub struct JobResume {
    pub id: String,
}

/// QMP command `memsave`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "memsave")]
#[qapi(returns = "()")]
#[qapi(added = "1.1.0")]
pub struct Memsave {
    #[qapi(name = "cpu-index")]
    pub cpu_index: Option<i64>,
    pub filename: String,
    #[qapi(added = "9.1.0")]
    #[qapi(required)]
    pub size: Option<u64>,
    #[qapi(name = "size")]
    #[qapi(removed = "9.1.0")]
    #[qapi(required)]
    pub size_legacy: Option<i64>,
    #[qapi(added = "9.1.0")]
    #[qapi(required)]
    pub val: Option<u64>,
    #[qapi(name = "val")]
    #[qapi(removed = "9.1.0")]
    #[qapi(required)]
    pub val_legacy: Option<i64>,
}

/// QMP command `migrate`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "migrate")]
#[qapi(returns = "()")]
#[qapi(added = "1.1.0")]
pub struct Migrate {
    #[qapi(deprecated(added = "8.2.0"))]
    #[qapi(removed = "9.1.0")]
    pub blk: Option<bool>,
    #[qapi(added = "8.2.0")]
    pub channels: Option<Vec<MigrationChannel>>,
    #[qapi(deprecated(added = "10.1.0"))]
    #[qapi(removed = "11.0.0")]
    pub detach: Option<bool>,
    #[qapi(deprecated(added = "8.2.0"))]
    #[qapi(removed = "9.1.0")]
    pub inc: Option<bool>,
    #[qapi(added = "3.0.0")]
    pub resume: Option<bool>,
    #[qapi(added = "8.2.2")]
    pub uri: Option<String>,
    #[qapi(name = "uri")]
    #[qapi(removed = "8.2.2")]
    #[qapi(required)]
    pub uri_legacy: Option<String>,
}

/// QMP command `migrate_cancel`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "migrate_cancel")]
#[qapi(returns = "()")]
#[qapi(added = "1.1.0")]
pub struct MigrateCancel {}

/// QMP command `migrate-continue`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "migrate-continue")]
#[qapi(returns = "()")]
#[qapi(added = "2.11.0")]
pub struct MigrateContinue {
    pub state: MigrationStatus,
}

/// QMP command `migrate-incoming`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "migrate-incoming")]
#[qapi(returns = "()")]
#[qapi(added = "2.3.0")]
pub struct MigrateIncoming {
    #[qapi(added = "8.2.0")]
    pub channels: Option<Vec<MigrationChannel>>,
    #[qapi(name = "exit-on-error")]
    #[qapi(added = "9.1.0")]
    pub exit_on_error: Option<bool>,
    #[qapi(added = "8.2.0")]
    pub uri: Option<String>,
    #[qapi(name = "uri")]
    #[qapi(removed = "8.2.0")]
    #[qapi(required)]
    pub uri_legacy: Option<String>,
}

/// QMP command `migrate-pause`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "migrate-pause")]
#[qapi(returns = "()")]
#[qapi(added = "3.0.0")]
#[qapi(allow_oob)]
pub struct MigratePause {}

/// QMP command `migrate-recover`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "migrate-recover")]
#[qapi(returns = "()")]
#[qapi(added = "3.0.0")]
#[qapi(allow_oob)]
pub struct MigrateRecover {
    pub uri: String,
}

/// QMP command `migrate-set-cache-size`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "migrate-set-cache-size")]
#[qapi(returns = "()")]
#[qapi(added = "1.2.0")]
#[qapi(deprecated(added = "5.0.0"))]
#[qapi(removed = "6.0.0")]
pub struct MigrateSetCacheSize {
    pub value: i64,
}

/// QMP command `migrate-set-capabilities`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "migrate-set-capabilities")]
#[qapi(returns = "()")]
#[qapi(added = "1.2.0")]
pub struct MigrateSetCapabilities {
    pub capabilities: Vec<MigrationCapabilityStatus>,
}

/// QMP command `migrate_set_downtime`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "migrate_set_downtime")]
#[qapi(returns = "()")]
#[qapi(added = "1.1.0")]
#[qapi(deprecated(added = "5.0.0"))]
#[qapi(removed = "6.0.0")]
pub struct MigrateSetDowntime {
    pub value: f64,
}

/// QMP command `migrate-set-parameters`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "migrate-set-parameters")]
#[qapi(returns = "()")]
#[qapi(added = "2.4.0")]
pub struct MigrateSetParameters {
    #[qapi(name = "announce-initial")]
    #[qapi(added = "4.0.0")]
    pub announce_initial: Option<u64>,
    #[qapi(name = "announce-max")]
    #[qapi(added = "4.0.0")]
    pub announce_max: Option<u64>,
    #[qapi(name = "announce-rounds")]
    #[qapi(added = "4.0.0")]
    pub announce_rounds: Option<u64>,
    #[qapi(name = "announce-step")]
    #[qapi(added = "4.0.0")]
    pub announce_step: Option<u64>,
    #[qapi(name = "avail-switchover-bandwidth")]
    #[qapi(added = "8.2.0")]
    pub avail_switchover_bandwidth: Option<u64>,
    #[qapi(name = "block-bitmap-mapping")]
    #[qapi(added = "5.2.0")]
    pub block_bitmap_mapping: Option<Vec<BitmapMigrationNodeAlias>>,
    #[qapi(name = "block-incremental")]
    #[qapi(added = "2.10.0")]
    #[qapi(deprecated(added = "8.2.0"))]
    #[qapi(removed = "9.1.0")]
    pub block_incremental: Option<bool>,
    #[qapi(name = "compress-level")]
    #[qapi(added = "6.0.0")]
    #[qapi(deprecated(added = "8.2.0"))]
    #[qapi(removed = "9.1.0")]
    pub compress_level: Option<u8>,
    #[qapi(name = "compress-threads")]
    #[qapi(added = "6.0.0")]
    #[qapi(deprecated(added = "8.2.0"))]
    #[qapi(removed = "9.1.0")]
    pub compress_threads: Option<u8>,
    #[qapi(name = "compress-wait-thread")]
    #[qapi(added = "3.1.0")]
    #[qapi(deprecated(added = "8.2.0"))]
    #[qapi(removed = "9.1.0")]
    pub compress_wait_thread: Option<bool>,
    #[qapi(name = "compress-level")]
    #[qapi(removed = "6.0.0")]
    pub compress_level_legacy: Option<i64>,
    #[qapi(name = "compress-threads")]
    #[qapi(removed = "6.0.0")]
    pub compress_threads_legacy: Option<i64>,
    #[qapi(name = "cpr-exec-command")]
    #[qapi(added = "10.2.0")]
    pub cpr_exec_command: Option<Vec<String>>,
    #[qapi(name = "cpu-throttle-increment")]
    #[qapi(added = "6.0.0")]
    pub cpu_throttle_increment: Option<u8>,
    #[qapi(name = "cpu-throttle-initial")]
    #[qapi(added = "6.0.0")]
    pub cpu_throttle_initial: Option<u8>,
    #[qapi(name = "cpu-throttle-tailslow")]
    #[qapi(added = "5.1.0")]
    pub cpu_throttle_tailslow: Option<bool>,
    #[qapi(name = "cpu-throttle-increment")]
    #[qapi(added = "2.7.0")]
    #[qapi(removed = "6.0.0")]
    pub cpu_throttle_increment_legacy: Option<i64>,
    #[qapi(name = "cpu-throttle-initial")]
    #[qapi(added = "2.7.0")]
    #[qapi(removed = "6.0.0")]
    pub cpu_throttle_initial_legacy: Option<i64>,
    #[qapi(name = "decompress-threads")]
    #[qapi(added = "6.0.0")]
    #[qapi(deprecated(added = "8.2.0"))]
    #[qapi(removed = "9.1.0")]
    pub decompress_threads: Option<u8>,
    #[qapi(name = "decompress-threads")]
    #[qapi(removed = "6.0.0")]
    pub decompress_threads_legacy: Option<i64>,
    #[qapi(name = "direct-io")]
    #[qapi(added = "9.1.0")]
    pub direct_io: Option<bool>,
    #[qapi(name = "downtime-limit")]
    #[qapi(added = "6.0.0")]
    pub downtime_limit: Option<u64>,
    #[qapi(name = "downtime-limit")]
    #[qapi(added = "2.8.0")]
    #[qapi(removed = "6.0.0")]
    pub downtime_limit_legacy: Option<i64>,
    #[qapi(name = "max-bandwidth")]
    #[qapi(added = "6.0.0")]
    pub max_bandwidth: Option<u64>,
    #[qapi(name = "max-cpu-throttle")]
    #[qapi(added = "6.0.0")]
    pub max_cpu_throttle: Option<u8>,
    #[qapi(name = "max-postcopy-bandwidth")]
    #[qapi(added = "3.0.0")]
    pub max_postcopy_bandwidth: Option<u64>,
    #[qapi(name = "max-bandwidth")]
    #[qapi(added = "2.8.0")]
    #[qapi(removed = "6.0.0")]
    pub max_bandwidth_legacy: Option<i64>,
    #[qapi(name = "max-cpu-throttle")]
    #[qapi(added = "3.1.0")]
    #[qapi(removed = "6.0.0")]
    pub max_cpu_throttle_legacy: Option<i64>,
    #[qapi(added = "8.2.0")]
    pub mode: Option<MigMode>,
    #[qapi(name = "multifd-channels")]
    #[qapi(added = "6.0.0")]
    pub multifd_channels: Option<u8>,
    #[qapi(name = "multifd-compression")]
    #[qapi(added = "5.0.0")]
    pub multifd_compression: Option<MultiFdCompression>,
    #[qapi(name = "multifd-qatzip-level")]
    #[qapi(added = "9.2.0")]
    pub multifd_qatzip_level: Option<u8>,
    #[qapi(name = "multifd-zlib-level")]
    #[qapi(added = "6.0.0")]
    pub multifd_zlib_level: Option<u8>,
    #[qapi(name = "multifd-zstd-level")]
    #[qapi(added = "6.0.0")]
    pub multifd_zstd_level: Option<u8>,
    #[qapi(name = "multifd-channels")]
    #[qapi(added = "4.0.0")]
    #[qapi(removed = "6.0.0")]
    pub multifd_channels_legacy: Option<i64>,
    #[qapi(name = "multifd-zlib-level")]
    #[qapi(added = "5.0.0")]
    #[qapi(removed = "6.0.0")]
    pub multifd_zlib_level_legacy: Option<i64>,
    #[qapi(name = "multifd-zstd-level")]
    #[qapi(added = "5.0.0")]
    #[qapi(removed = "6.0.0")]
    pub multifd_zstd_level_legacy: Option<i64>,
    #[qapi(name = "throttle-trigger-threshold")]
    #[qapi(added = "6.0.0")]
    pub throttle_trigger_threshold: Option<u8>,
    #[qapi(name = "throttle-trigger-threshold")]
    #[qapi(added = "5.0.0")]
    #[qapi(removed = "6.0.0")]
    pub throttle_trigger_threshold_legacy: Option<i64>,
    #[qapi(name = "tls-authz")]
    #[qapi(added = "4.0.0")]
    pub tls_authz: Option<StrOrNull>,
    #[qapi(name = "tls-creds")]
    #[qapi(added = "2.10.0")]
    pub tls_creds: Option<StrOrNull>,
    #[qapi(name = "tls-hostname")]
    #[qapi(added = "2.10.0")]
    pub tls_hostname: Option<StrOrNull>,
    #[qapi(name = "tls-creds")]
    #[qapi(added = "2.7.0")]
    #[qapi(removed = "2.10.0")]
    pub tls_creds_legacy: Option<String>,
    #[qapi(name = "tls-hostname")]
    #[qapi(added = "2.7.0")]
    #[qapi(removed = "2.10.0")]
    pub tls_hostname_legacy: Option<String>,
    #[qapi(name = "vcpu-dirty-limit")]
    #[qapi(added = "8.1.0")]
    pub vcpu_dirty_limit: Option<u64>,
    #[qapi(name = "x-checkpoint-delay")]
    #[qapi(feature = "unstable")]
    #[qapi(added = "6.0.0")]
    pub x_checkpoint_delay: Option<u32>,
    #[qapi(name = "x-cpu-throttle-increment")]
    #[qapi(added = "2.5.0")]
    #[qapi(removed = "2.7.0")]
    pub x_cpu_throttle_increment: Option<i64>,
    #[qapi(name = "x-cpu-throttle-initial")]
    #[qapi(added = "2.5.0")]
    #[qapi(removed = "2.7.0")]
    pub x_cpu_throttle_initial: Option<i64>,
    #[qapi(name = "x-multifd-channels")]
    #[qapi(added = "2.11.0")]
    #[qapi(removed = "4.0.0")]
    pub x_multifd_channels: Option<i64>,
    #[qapi(name = "x-multifd-page-count")]
    #[qapi(added = "2.11.0")]
    #[qapi(removed = "4.0.0")]
    pub x_multifd_page_count: Option<i64>,
    #[qapi(name = "x-rdma-chunk-size")]
    #[qapi(feature = "unstable")]
    #[qapi(added = "11.1.0")]
    pub x_rdma_chunk_size: Option<u64>,
    #[qapi(name = "x-vcpu-dirty-limit-period")]
    #[qapi(feature = "unstable")]
    #[qapi(added = "8.1.0")]
    pub x_vcpu_dirty_limit_period: Option<u64>,
    #[qapi(name = "x-checkpoint-delay")]
    #[qapi(added = "2.8.0")]
    #[qapi(removed = "6.0.0")]
    pub x_checkpoint_delay_legacy: Option<i64>,
    #[qapi(name = "xbzrle-cache-size")]
    #[qapi(added = "2.11.0")]
    pub xbzrle_cache_size: Option<u64>,
    #[qapi(name = "zero-page-detection")]
    #[qapi(added = "9.0.0")]
    pub zero_page_detection: Option<ZeroPageDetection>,
}

/// QMP command `migrate_set_speed`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "migrate_set_speed")]
#[qapi(returns = "()")]
#[qapi(added = "1.1.0")]
#[qapi(deprecated(added = "5.0.0"))]
#[qapi(removed = "6.0.0")]
pub struct MigrateSetSpeed {
    pub value: i64,
}

/// QMP command `migrate-start-postcopy`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "migrate-start-postcopy")]
#[qapi(returns = "()")]
#[qapi(added = "2.5.0")]
pub struct MigrateStartPostcopy {}

/// QMP command `nbd-server-add`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "nbd-server-add")]
#[qapi(returns = "()")]
#[qapi(added = "1.3.0")]
#[qapi(deprecated(added = "5.2.0"))]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct NbdServerAdd {
    #[qapi(added = "4.0.0")]
    pub bitmap: Option<String>,
    #[qapi(added = "5.0.0")]
    pub description: Option<String>,
    pub device: String,
    #[qapi(added = "2.12.0")]
    pub name: Option<String>,
    pub writable: Option<bool>,
}

/// QMP command `nbd-server-remove`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "nbd-server-remove")]
#[qapi(returns = "()")]
#[qapi(added = "2.12.0")]
#[qapi(deprecated(added = "5.2.0"))]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct NbdServerRemove {
    #[qapi(added = "5.2.0")]
    pub mode: Option<BlockExportRemoveMode>,
    #[qapi(name = "mode")]
    #[qapi(removed = "5.2.0")]
    pub mode_legacy: Option<NbdServerRemoveMode>,
    pub name: String,
}

/// QMP command `nbd-server-start`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "nbd-server-start")]
#[qapi(returns = "()")]
#[qapi(added = "1.3.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct NbdServerStart {
    #[qapi(added = "2.10.0")]
    #[qapi(required)]
    pub addr: Option<SocketAddressLegacy>,
    #[qapi(name = "addr")]
    #[qapi(removed = "2.10.0")]
    #[qapi(required)]
    pub addr_legacy: Option<SocketAddress>,
    #[qapi(name = "handshake-max-seconds")]
    #[qapi(added = "10.0.0")]
    pub handshake_max_seconds: Option<u32>,
    #[qapi(name = "max-connections")]
    #[qapi(added = "5.2.0")]
    pub max_connections: Option<u32>,
    #[qapi(name = "tls-authz")]
    #[qapi(added = "4.0.0")]
    pub tls_authz: Option<String>,
    #[qapi(name = "tls-creds")]
    #[qapi(added = "2.6.0")]
    pub tls_creds: Option<String>,
}

/// QMP command `nbd-server-stop`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "nbd-server-stop")]
#[qapi(returns = "()")]
#[qapi(added = "1.3.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct NbdServerStop {}

/// QMP command `netdev_add`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "netdev_add")]
#[qapi(returns = "()")]
#[qapi(added = "1.2.0")]
#[qapi(allow_preconfig(added = "6.1.0"))]
pub struct NetdevAdd {
    /// Additional command arguments not modeled by this schema.
    #[qapi(flatten)]
    #[qapi(removed = "5.0.0")]
    pub arguments: Option<serde_json::Map<String, serde_json::Value>>,
    pub id: String,
    #[qapi(removed = "2.5.0")]
    pub props: Option<serde_json::Value>,
    #[qapi(discriminator)]
    #[qapi(added = "5.0.0")]
    #[qapi(required)]
    pub r#type: Option<NetClientDriver>,
    #[qapi(name = "type")]
    #[qapi(removed = "5.0.0")]
    #[qapi(required)]
    pub type_legacy: Option<String>,
    #[qapi(union)]
    #[qapi(flatten)]
    #[qapi(added = "5.0.0")]
    pub u: Option<NetdevBranch>,
}

/// QMP command `netdev_del`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "netdev_del")]
#[qapi(returns = "()")]
#[qapi(added = "1.2.0")]
#[qapi(allow_preconfig(added = "6.1.0"))]
pub struct NetdevDel {
    pub id: String,
}

/// QMP command `object-add`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "object-add")]
#[qapi(returns = "()")]
#[qapi(added = "2.0.0")]
#[qapi(allow_preconfig(added = "6.1.0"))]
pub struct ObjectAdd {
    /// Additional command arguments not modeled by this schema.
    #[qapi(flatten)]
    #[qapi(added = "5.0.0")]
    #[qapi(removed = "6.0.0")]
    pub arguments: Option<serde_json::Map<String, serde_json::Value>>,
    /// Additional command arguments not modeled by this schema.
    #[qapi(flatten)]
    #[qapi(removed = "2.5.0")]
    pub arguments_legacy: Option<serde_json::Map<String, serde_json::Value>>,
    pub id: String,
    #[qapi(removed = "6.0.0")]
    pub props: Option<serde_json::Value>,
    #[qapi(name = "qom-type")]
    #[qapi(discriminator)]
    #[qapi(added = "6.0.0")]
    #[qapi(required)]
    pub qom_type: Option<ObjectType>,
    #[qapi(name = "qom-type")]
    #[qapi(removed = "6.0.0")]
    #[qapi(required)]
    pub qom_type_legacy: Option<String>,
    #[qapi(union)]
    #[qapi(flatten)]
    #[qapi(added = "6.0.0")]
    pub u: Option<ObjectOptionsBranch>,
}

/// QMP command `object-del`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "object-del")]
#[qapi(returns = "()")]
#[qapi(added = "2.0.0")]
#[qapi(allow_preconfig(added = "6.1.0"))]
pub struct ObjectDel {
    pub id: String,
}

/// QMP command `pmemsave`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "pmemsave")]
#[qapi(returns = "()")]
#[qapi(added = "1.1.0")]
pub struct Pmemsave {
    pub filename: String,
    #[qapi(added = "9.1.0")]
    #[qapi(required)]
    pub size: Option<u64>,
    #[qapi(name = "size")]
    #[qapi(removed = "9.1.0")]
    #[qapi(required)]
    pub size_legacy: Option<i64>,
    #[qapi(added = "9.1.0")]
    #[qapi(required)]
    pub val: Option<u64>,
    #[qapi(name = "val")]
    #[qapi(removed = "9.1.0")]
    #[qapi(required)]
    pub val_legacy: Option<i64>,
}

/// QMP command `qmp_capabilities`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "qmp_capabilities")]
#[qapi(returns = "()")]
#[qapi(added = "2.8.0")]
#[qapi(allow_preconfig(added = "3.0.0"))]
pub struct QmpCapabilities {
    #[qapi(added = "2.12.0")]
    pub enable: Option<Vec<QmpCapability>>,
}

/// QMP command `qom-get`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "qom-get")]
#[qapi(returns = "serde_json::Value")]
#[qapi(added = "1.1.0")]
#[qapi(allow_preconfig(added = "3.0.0"))]
pub struct QomGet {
    /// Additional command arguments not modeled by this schema.
    #[qapi(flatten)]
    #[qapi(removed = "2.5.0")]
    pub arguments: Option<serde_json::Map<String, serde_json::Value>>,
    pub path: String,
    pub property: String,
}

/// QMP command `qom-list`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "qom-list")]
#[qapi(returns = "Vec<ObjectPropertyInfo>")]
#[qapi(added = "1.1.0")]
#[qapi(allow_preconfig(added = "3.0.0"))]
pub struct QomList {
    pub path: String,
}

/// QMP command `qom-list-get`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "qom-list-get")]
#[qapi(returns = "Vec<ObjectPropertiesValues>")]
#[qapi(added = "10.1.0")]
#[qapi(allow_preconfig)]
pub struct QomListGet {
    pub paths: Vec<String>,
}

/// QMP command `qom-list-properties`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "qom-list-properties")]
#[qapi(returns = "Vec<ObjectPropertyInfo>")]
#[qapi(added = "2.12.0")]
#[qapi(allow_preconfig(added = "3.0.0"))]
pub struct QomListProperties {
    pub typename: String,
}

/// QMP command `qom-list-types`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "qom-list-types")]
#[qapi(returns = "Vec<ObjectTypeInfo>")]
#[qapi(added = "1.1.0")]
#[qapi(allow_preconfig(added = "3.0.0"))]
pub struct QomListTypes {
    pub r#abstract: Option<bool>,
    pub implements: Option<String>,
}

/// QMP command `qom-set`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "qom-set")]
#[qapi(returns = "()")]
#[qapi(added = "1.1.0")]
#[qapi(allow_preconfig(added = "3.0.0"))]
pub struct QomSet {
    /// Additional command arguments not modeled by this schema.
    #[qapi(flatten)]
    #[qapi(removed = "2.5.0")]
    pub arguments: Option<serde_json::Map<String, serde_json::Value>>,
    pub path: String,
    pub property: String,
    pub value: serde_json::Value,
}

/// QMP command `query-accelerators`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-accelerators")]
#[qapi(returns = "AcceleratorInfo")]
#[qapi(added = "10.2.0")]
pub struct QueryAccelerators {}

/// QMP command `query-acpi-ospm-status`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-acpi-ospm-status")]
#[qapi(returns = "Vec<AcpiostInfo>")]
#[qapi(added = "2.1.0")]
pub struct QueryAcpiOspmStatus {}

/// QMP command `query-audiodevs`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-audiodevs")]
#[qapi(returns = "Vec<Audiodev>")]
#[qapi(added = "8.0.0")]
pub struct QueryAudiodevs {}

/// QMP command `query-balloon`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-balloon")]
#[qapi(returns = "BalloonInfo")]
#[qapi(added = "1.0.0")]
pub struct QueryBalloon {}

/// QMP command `query-block`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-block")]
#[qapi(returns = "Vec<BlockInfo>")]
#[qapi(added = "1.0.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct QueryBlock {
    #[qapi(added = "11.0.0")]
    pub flat: Option<bool>,
}

/// QMP command `query-block-exports`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-block-exports")]
#[qapi(returns = "Vec<BlockExportInfo>")]
#[qapi(added = "5.2.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct QueryBlockExports {}

/// QMP command `query-block-jobs`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-block-jobs")]
#[qapi(returns = "Vec<BlockJobInfo>")]
#[qapi(added = "1.1.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct QueryBlockJobs {}

/// QMP command `query-blockstats`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-blockstats")]
#[qapi(returns = "Vec<BlockStats>")]
#[qapi(added = "1.0.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct QueryBlockstats {
    #[qapi(name = "query-nodes")]
    #[qapi(added = "2.3.0")]
    pub query_nodes: Option<bool>,
}

/// QMP command `query-chardev`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-chardev")]
#[qapi(returns = "Vec<ChardevInfo>")]
#[qapi(added = "1.0.0")]
#[qapi(allow_preconfig(added = "3.0.0"))]
pub struct QueryChardev {}

/// QMP command `query-chardev-backends`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-chardev-backends")]
#[qapi(returns = "Vec<ChardevBackendInfo>")]
#[qapi(added = "2.0.0")]
pub struct QueryChardevBackends {}

/// QMP command `query-colo-status`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-colo-status")]
#[qapi(returns = "ColoStatus")]
#[qapi(condition = "CONFIG_REPLICATION")]
#[qapi(added = "3.1.0")]
pub struct QueryColoStatus {}

/// QMP command `query-command-line-options`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-command-line-options")]
#[qapi(returns = "Vec<CommandLineOptionInfo>")]
#[qapi(added = "1.5.0")]
#[qapi(allow_preconfig(added = "3.0.0"))]
pub struct QueryCommandLineOptions {
    pub option: Option<String>,
}

/// QMP command `query-commands`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-commands")]
#[qapi(returns = "Vec<CommandInfo>")]
#[qapi(added = "1.0.0")]
#[qapi(allow_preconfig(added = "3.0.0"))]
pub struct QueryCommands {}

/// QMP command `query-cpu-definitions`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-cpu-definitions")]
#[qapi(returns = "Vec<CpuDefinitionInfo>")]
#[qapi(added = "1.2.0")]
pub struct QueryCpuDefinitions {}

/// QMP command `query-cpu-model-baseline`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-cpu-model-baseline")]
#[qapi(returns = "CpuModelBaselineInfo")]
#[qapi(added = "2.8.0")]
pub struct QueryCpuModelBaseline {
    pub modela: CpuModelInfo,
    pub modelb: CpuModelInfo,
}

/// QMP command `query-cpu-model-comparison`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-cpu-model-comparison")]
#[qapi(returns = "CpuModelCompareInfo")]
#[qapi(added = "2.8.0")]
pub struct QueryCpuModelComparison {
    pub modela: CpuModelInfo,
    pub modelb: CpuModelInfo,
}

/// QMP command `query-cpu-model-expansion`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-cpu-model-expansion")]
#[qapi(returns = "CpuModelExpansionInfo")]
#[qapi(added = "2.8.0")]
pub struct QueryCpuModelExpansion {
    pub model: CpuModelInfo,
    pub r#type: CpuModelExpansionType,
}

/// QMP command `query-cpus`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-cpus")]
#[qapi(returns = "Vec<CpuInfo>")]
#[qapi(added = "1.0.0")]
#[qapi(deprecated(added = "5.0.0"))]
#[qapi(removed = "6.0.0")]
pub struct QueryCpus {}

/// QMP command `query-cpus-fast`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-cpus-fast")]
#[qapi(returns = "Vec<CpuInfoFast>")]
#[qapi(added = "2.12.0")]
pub struct QueryCpusFast {}

/// QMP command `query-cryptodev`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-cryptodev")]
#[qapi(returns = "Vec<QCryptodevInfo>")]
#[qapi(added = "8.0.0")]
pub struct QueryCryptodev {}

/// QMP command `query-current-machine`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-current-machine")]
#[qapi(returns = "CurrentMachineParams")]
#[qapi(added = "4.0.0")]
pub struct QueryCurrentMachine {}

/// QMP command `query-dirty-rate`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-dirty-rate")]
#[qapi(returns = "DirtyRateInfo")]
#[qapi(added = "5.2.0")]
pub struct QueryDirtyRate {
    #[qapi(name = "calc-time-unit")]
    #[qapi(added = "8.2.0")]
    pub calc_time_unit: Option<TimeUnit>,
}

/// QMP command `query-display-options`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-display-options")]
#[qapi(returns = "DisplayOptions")]
#[qapi(added = "3.1.0")]
pub struct QueryDisplayOptions {}

/// QMP command `query-dump`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-dump")]
#[qapi(returns = "DumpQueryResult")]
#[qapi(added = "2.6.0")]
pub struct QueryDump {}

/// QMP command `query-dump-guest-memory-capability`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-dump-guest-memory-capability")]
#[qapi(returns = "DumpGuestMemoryCapability")]
#[qapi(added = "2.0.0")]
pub struct QueryDumpGuestMemoryCapability {}

/// QMP command `query-events`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-events")]
#[qapi(returns = "Vec<EventInfo>")]
#[qapi(added = "1.2.0")]
#[qapi(deprecated(added = "5.0.0"))]
#[qapi(removed = "6.0.0")]
pub struct QueryEvents {}

/// QMP command `query-fdsets`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-fdsets")]
#[qapi(returns = "Vec<FdsetInfo>")]
#[qapi(added = "1.2.0")]
pub struct QueryFdsets {}

/// QMP command `query-firmware-log`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-firmware-log")]
#[qapi(returns = "FirmwareLog")]
#[qapi(added = "10.2.0")]
pub struct QueryFirmwareLog {
    #[qapi(name = "max-size")]
    pub max_size: Option<u64>,
}

/// QMP command `query-gic-capabilities`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-gic-capabilities")]
#[qapi(returns = "Vec<GicCapability>")]
#[qapi(added = "2.6.0")]
pub struct QueryGicCapabilities {}

/// QMP command `query-hotpluggable-cpus`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-hotpluggable-cpus")]
#[qapi(returns = "Vec<HotpluggableCpu>")]
#[qapi(added = "2.7.0")]
#[qapi(allow_preconfig(added = "3.0.0"))]
pub struct QueryHotpluggableCpus {}

/// QMP command `query-hv-balloon-status-report`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-hv-balloon-status-report")]
#[qapi(returns = "HvBalloonInfo")]
#[qapi(added = "8.2.0")]
pub struct QueryHvBalloonStatusReport {}

/// QMP command `query-iothreads`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-iothreads")]
#[qapi(returns = "Vec<IoThreadInfo>")]
#[qapi(added = "2.0.0")]
#[qapi(allow_preconfig(added = "3.0.0"))]
pub struct QueryIothreads {}

/// QMP command `query-jobs`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-jobs")]
#[qapi(returns = "Vec<JobInfo>")]
#[qapi(added = "3.0.0")]
pub struct QueryJobs {}

/// QMP command `query-kvm`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-kvm")]
#[qapi(returns = "KvmInfo")]
#[qapi(added = "1.0.0")]
#[qapi(deprecated(added = "11.1.0"))]
pub struct QueryKvm {}

/// QMP command `query-machines`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-machines")]
#[qapi(returns = "Vec<MachineInfo>")]
#[qapi(added = "1.2.0")]
pub struct QueryMachines {
    #[qapi(name = "compat-props")]
    #[qapi(feature = "unstable")]
    #[qapi(added = "9.1.0")]
    pub compat_props: Option<bool>,
}

/// QMP command `query-memdev`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-memdev")]
#[qapi(returns = "Vec<Memdev>")]
#[qapi(added = "2.1.0")]
#[qapi(allow_preconfig(added = "3.0.0"))]
pub struct QueryMemdev {}

/// QMP command `query-memory-devices`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-memory-devices")]
#[qapi(returns = "Vec<MemoryDeviceInfo>")]
#[qapi(added = "2.1.0")]
pub struct QueryMemoryDevices {}

/// QMP command `query-memory-size-summary`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-memory-size-summary")]
#[qapi(returns = "MemoryInfo")]
#[qapi(added = "2.11.0")]
pub struct QueryMemorySizeSummary {}

/// QMP command `query-mice`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-mice")]
#[qapi(returns = "Vec<MouseInfo>")]
#[qapi(added = "1.0.0")]
pub struct QueryMice {}

/// QMP command `query-migrate`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-migrate")]
#[qapi(returns = "MigrationInfo")]
#[qapi(added = "1.0.0")]
pub struct QueryMigrate {}

/// QMP command `query-migrate-cache-size`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-migrate-cache-size")]
#[qapi(returns = "i64")]
#[qapi(added = "1.2.0")]
#[qapi(deprecated(added = "5.0.0"))]
#[qapi(removed = "6.0.0")]
pub struct QueryMigrateCacheSize {}

/// QMP command `query-migrate-capabilities`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-migrate-capabilities")]
#[qapi(returns = "Vec<MigrationCapabilityStatus>")]
#[qapi(added = "1.2.0")]
pub struct QueryMigrateCapabilities {}

/// QMP command `query-migrate-parameters`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-migrate-parameters")]
#[qapi(returns = "MigrationParameters")]
#[qapi(added = "2.4.0")]
pub struct QueryMigrateParameters {}

/// QMP command `query-migrationthreads`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-migrationthreads")]
#[qapi(returns = "Vec<MigrationThreadInfo>")]
#[qapi(added = "8.0.0")]
#[qapi(deprecated(added = "9.2.0"))]
#[qapi(removed = "11.0.0")]
pub struct QueryMigrationthreads {}

/// QMP command `query-name`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-name")]
#[qapi(returns = "NameInfo")]
#[qapi(added = "1.0.0")]
#[qapi(allow_preconfig(added = "3.0.0"))]
pub struct QueryName {}

/// QMP command `query-named-block-nodes`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-named-block-nodes")]
#[qapi(returns = "Vec<BlockDeviceInfo>")]
#[qapi(added = "2.0.0")]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct QueryNamedBlockNodes {
    #[qapi(added = "5.0.0")]
    pub flat: Option<bool>,
}

/// QMP command `query-pci`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-pci")]
#[qapi(returns = "Vec<PciInfo>")]
#[qapi(added = "1.0.0")]
pub struct QueryPci {}

/// QMP command `query-pr-managers`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-pr-managers")]
#[qapi(returns = "Vec<PrManagerInfo>")]
#[qapi(added = "3.0.0")]
#[qapi(allow_preconfig)]
pub struct QueryPrManagers {}

/// QMP command `query-qmp-schema`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-qmp-schema")]
#[qapi(returns = "Vec<SchemaInfo>")]
#[qapi(added = "2.5.0")]
#[qapi(allow_preconfig(added = "6.0.0"))]
pub struct QueryQmpSchema {
    /// Additional command arguments not modeled by this schema.
    #[qapi(flatten)]
    #[qapi(removed = "6.0.0")]
    pub arguments: Option<serde_json::Map<String, serde_json::Value>>,
}

/// QMP command `query-replay`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-replay")]
#[qapi(returns = "ReplayInfo")]
#[qapi(added = "5.2.0")]
pub struct QueryReplay {}

/// QMP command `query-rocker`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-rocker")]
#[qapi(returns = "RockerSwitch")]
#[qapi(added = "2.4.0")]
pub struct QueryRocker {
    pub name: String,
}

/// QMP command `query-rocker-of-dpa-flows`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-rocker-of-dpa-flows")]
#[qapi(returns = "Vec<RockerOfDpaFlow>")]
#[qapi(added = "2.4.0")]
pub struct QueryRockerOfDpaFlows {
    pub name: String,
    #[qapi(name = "tbl-id")]
    pub tbl_id: Option<u32>,
}

/// QMP command `query-rocker-of-dpa-groups`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-rocker-of-dpa-groups")]
#[qapi(returns = "Vec<RockerOfDpaGroup>")]
#[qapi(added = "2.4.0")]
pub struct QueryRockerOfDpaGroups {
    pub name: String,
    pub r#type: Option<u8>,
}

/// QMP command `query-rocker-ports`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-rocker-ports")]
#[qapi(returns = "Vec<RockerPort>")]
#[qapi(added = "2.4.0")]
pub struct QueryRockerPorts {
    pub name: String,
}

/// QMP command `query-rx-filter`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-rx-filter")]
#[qapi(returns = "Vec<RxFilterInfo>")]
#[qapi(added = "1.6.0")]
pub struct QueryRxFilter {
    pub name: Option<String>,
}

/// QMP command `query-s390x-cpu-polarization`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-s390x-cpu-polarization")]
#[qapi(returns = "CpuPolarizationInfo")]
#[qapi(added = "8.2.0")]
#[qapi(unstable(added = "8.2.0"))]
pub struct QueryS390xCpuPolarization {}

/// QMP command `query-sev`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-sev")]
#[qapi(returns = "SevInfo")]
#[qapi(added = "2.12.0")]
pub struct QuerySev {}

/// QMP command `query-sev-attestation-report`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-sev-attestation-report")]
#[qapi(returns = "SevAttestationReport")]
#[qapi(added = "6.1.0")]
pub struct QuerySevAttestationReport {
    pub mnonce: String,
}

/// QMP command `query-sev-capabilities`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-sev-capabilities")]
#[qapi(returns = "SevCapability")]
#[qapi(added = "2.12.0")]
pub struct QuerySevCapabilities {}

/// QMP command `query-sev-launch-measure`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-sev-launch-measure")]
#[qapi(returns = "SevLaunchMeasureInfo")]
#[qapi(added = "2.12.0")]
pub struct QuerySevLaunchMeasure {}

/// QMP command `query-sgx`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-sgx")]
#[qapi(returns = "SgxInfo")]
#[qapi(added = "6.2.0")]
pub struct QuerySgx {}

/// QMP command `query-sgx-capabilities`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-sgx-capabilities")]
#[qapi(returns = "SgxInfo")]
#[qapi(added = "6.2.0")]
pub struct QuerySgxCapabilities {}

/// QMP command `query-spice`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-spice")]
#[qapi(returns = "SpiceInfo")]
#[qapi(condition = "CONFIG_SPICE")]
#[qapi(added = "1.0.0")]
pub struct QuerySpice {}

/// QMP command `query-stats`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-stats")]
#[qapi(returns = "Vec<StatsResult>")]
#[qapi(added = "7.1.0")]
pub struct QueryStats {
    pub providers: Option<Vec<StatsRequest>>,
    #[qapi(discriminator)]
    pub target: StatsTarget,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<StatsFilterBranch>,
}

/// QMP command `query-stats-schemas`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-stats-schemas")]
#[qapi(returns = "Vec<StatsSchema>")]
#[qapi(added = "7.1.0")]
pub struct QueryStatsSchemas {
    pub provider: Option<StatsProvider>,
}

/// QMP command `query-status`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-status")]
#[qapi(returns = "StatusInfo")]
#[qapi(added = "1.0.0")]
#[qapi(allow_preconfig(added = "3.0.0"))]
pub struct QueryStatus {}

/// QMP command `query-target`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-target")]
#[qapi(returns = "QemuTargetInfo")]
#[qapi(added = "1.2.0")]
#[qapi(returns(added = "1.2.0", value = "TargetInfo"))]
#[qapi(returns(added = "10.1.0", value = "QemuTargetInfo"))]
#[qapi(returns_enum = "QueryTargetReturns")]
pub struct QueryTarget {}

/// QMP command `query-tpm`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-tpm")]
#[qapi(returns = "Vec<TpmInfo>")]
#[qapi(condition = "CONFIG_TPM")]
#[qapi(added = "1.5.0")]
pub struct QueryTpm {}

/// QMP command `query-tpm-models`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-tpm-models")]
#[qapi(returns = "Vec<TpmModel>")]
#[qapi(condition = "CONFIG_TPM")]
#[qapi(added = "1.5.0")]
pub struct QueryTpmModels {}

/// QMP command `query-tpm-types`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-tpm-types")]
#[qapi(returns = "Vec<TpmType>")]
#[qapi(condition = "CONFIG_TPM")]
#[qapi(added = "1.5.0")]
pub struct QueryTpmTypes {}

/// QMP command `query-uuid`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-uuid")]
#[qapi(returns = "UuidInfo")]
#[qapi(added = "1.0.0")]
#[qapi(allow_preconfig(added = "3.0.0"))]
pub struct QueryUuid {}

/// QMP command `query-vcpu-dirty-limit`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-vcpu-dirty-limit")]
#[qapi(returns = "Vec<DirtyLimitInfo>")]
#[qapi(added = "7.1.0")]
pub struct QueryVcpuDirtyLimit {}

/// QMP command `query-version`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-version")]
#[qapi(returns = "VersionInfo")]
#[qapi(added = "1.0.0")]
#[qapi(allow_preconfig(added = "3.0.0"))]
pub struct QueryVersion {}

/// QMP command `query-vm-generation-id`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-vm-generation-id")]
#[qapi(returns = "GuidInfo")]
#[qapi(added = "2.9.0")]
pub struct QueryVmGenerationId {}

/// QMP command `query-vnc`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-vnc")]
#[qapi(returns = "VncInfo")]
#[qapi(condition = "CONFIG_VNC")]
#[qapi(added = "1.0.0")]
pub struct QueryVnc {}

/// QMP command `query-vnc-servers`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-vnc-servers")]
#[qapi(returns = "Vec<VncInfo2>")]
#[qapi(condition = "CONFIG_VNC")]
#[qapi(added = "2.3.0")]
pub struct QueryVncServers {}

/// QMP command `query-xen-replication-status`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-xen-replication-status")]
#[qapi(returns = "ReplicationStatus")]
#[qapi(condition = "CONFIG_REPLICATION")]
#[qapi(added = "2.9.0")]
pub struct QueryXenReplicationStatus {}

/// QMP command `query-yank`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "query-yank")]
#[qapi(returns = "Vec<YankInstance>")]
#[qapi(added = "6.0.0")]
#[qapi(allow_oob)]
pub struct QueryYank {}

/// QMP command `quit`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "quit")]
#[qapi(returns = "()")]
#[qapi(added = "1.0.0")]
#[qapi(allow_preconfig(added = "6.0.0"))]
pub struct Quit {}

/// QMP command `remove-fd`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "remove-fd")]
#[qapi(returns = "()")]
#[qapi(added = "1.2.0")]
pub struct RemoveFd {
    pub fd: Option<i64>,
    #[qapi(name = "fdset-id")]
    pub fdset_id: i64,
}

/// QMP command `replay-break`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "replay-break")]
#[qapi(returns = "()")]
#[qapi(added = "5.2.0")]
pub struct ReplayBreak {
    pub icount: i64,
}

/// QMP command `replay-delete-break`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "replay-delete-break")]
#[qapi(returns = "()")]
#[qapi(added = "5.2.0")]
pub struct ReplayDeleteBreak {}

/// QMP command `replay-seek`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "replay-seek")]
#[qapi(returns = "()")]
#[qapi(added = "5.2.0")]
pub struct ReplaySeek {
    pub icount: i64,
}

/// QMP command `request-ebpf`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "request-ebpf")]
#[qapi(condition = "CONFIG_EBPF")]
#[qapi(returns = "EbpfObject")]
#[qapi(added = "9.0.0")]
pub struct RequestEbpf {
    pub id: EbpfProgramId,
}

/// QMP command `ringbuf-read`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "ringbuf-read")]
#[qapi(returns = "String")]
#[qapi(added = "1.4.0")]
pub struct RingbufRead {
    pub device: String,
    pub format: Option<DataFormat>,
    pub size: i64,
}

/// QMP command `ringbuf-write`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "ringbuf-write")]
#[qapi(returns = "()")]
#[qapi(added = "1.4.0")]
pub struct RingbufWrite {
    pub data: String,
    pub device: String,
    pub format: Option<DataFormat>,
}

/// QMP command `rtc-reset-reinjection`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "rtc-reset-reinjection")]
#[qapi(returns = "()")]
#[qapi(added = "2.1.0")]
pub struct RtcResetReinjection {}

/// QMP command `screendump`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "screendump")]
#[qapi(returns = "()")]
#[qapi(condition = "CONFIG_PIXMAN")]
#[qapi(added = "1.3.0")]
pub struct Screendump {
    #[qapi(added = "2.12.0")]
    pub device: Option<String>,
    pub filename: String,
    #[qapi(added = "7.1.0")]
    pub format: Option<ImageFormat>,
    #[qapi(added = "2.12.0")]
    pub head: Option<i64>,
}

/// QMP command `send-key`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "send-key")]
#[qapi(returns = "()")]
#[qapi(added = "1.3.0")]
pub struct SendKey {
    #[qapi(name = "hold-time")]
    pub hold_time: Option<i64>,
    pub keys: Vec<KeyValue>,
}

/// QMP command `set-action`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "set-action")]
#[qapi(returns = "()")]
#[qapi(added = "6.0.0")]
#[qapi(allow_preconfig)]
pub struct SetAction {
    pub panic: Option<PanicAction>,
    pub reboot: Option<RebootAction>,
    pub shutdown: Option<ShutdownAction>,
    pub watchdog: Option<WatchdogAction>,
}

/// QMP command `set-cpu-topology`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "set-cpu-topology")]
#[qapi(returns = "()")]
#[qapi(added = "8.2.0")]
#[qapi(unstable(added = "8.2.0"))]
pub struct SetCpuTopology {
    #[qapi(name = "book-id")]
    pub book_id: Option<u16>,
    #[qapi(name = "core-id")]
    pub core_id: u16,
    pub dedicated: Option<bool>,
    #[qapi(name = "drawer-id")]
    pub drawer_id: Option<u16>,
    #[qapi(added = "9.2.0")]
    pub entitlement: Option<S390CpuEntitlement>,
    #[qapi(name = "entitlement")]
    #[qapi(removed = "9.2.0")]
    pub entitlement_legacy: Option<CpuS390Entitlement>,
    #[qapi(name = "socket-id")]
    pub socket_id: Option<u16>,
}

/// QMP command `set_link`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "set_link")]
#[qapi(returns = "()")]
#[qapi(added = "1.1.0")]
pub struct SetLink {
    pub name: String,
    pub up: bool,
}

/// QMP command `set-numa-node`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "set-numa-node")]
#[qapi(returns = "()")]
#[qapi(added = "3.0.0")]
#[qapi(allow_preconfig)]
pub struct SetNumaNode {
    #[qapi(discriminator)]
    pub r#type: NumaOptionsType,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<NumaOptionsBranch>,
}

/// QMP command `set_password`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "set_password")]
#[qapi(returns = "()")]
#[qapi(added = "1.1.0")]
pub struct SetPassword {
    #[qapi(added = "7.0.0")]
    pub connected: Option<SetPasswordAction>,
    #[qapi(name = "connected")]
    #[qapi(removed = "7.0.0")]
    pub connected_legacy: Option<String>,
    pub password: String,
    #[qapi(discriminator)]
    #[qapi(added = "7.0.0")]
    #[qapi(required)]
    pub protocol: Option<DisplayProtocol>,
    #[qapi(name = "protocol")]
    #[qapi(removed = "7.0.0")]
    #[qapi(required)]
    pub protocol_legacy: Option<String>,
    #[qapi(union)]
    #[qapi(flatten)]
    #[qapi(added = "7.0.0")]
    pub u: Option<SetPasswordOptionsBranch>,
}

/// QMP command `set-vcpu-dirty-limit`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "set-vcpu-dirty-limit")]
#[qapi(returns = "()")]
#[qapi(added = "7.1.0")]
pub struct SetVcpuDirtyLimit {
    #[qapi(name = "cpu-index")]
    pub cpu_index: Option<i64>,
    #[qapi(name = "dirty-rate")]
    pub dirty_rate: u64,
}

/// QMP command `sev-inject-launch-secret`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "sev-inject-launch-secret")]
#[qapi(returns = "()")]
#[qapi(added = "6.0.0")]
pub struct SevInjectLaunchSecret {
    pub gpa: Option<u64>,
    #[qapi(name = "packet-header")]
    pub packet_header: String,
    pub secret: String,
}

/// QMP command `snapshot-delete`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "snapshot-delete")]
#[qapi(returns = "()")]
#[qapi(added = "6.0.0")]
pub struct SnapshotDelete {
    pub devices: Vec<String>,
    #[qapi(name = "job-id")]
    pub job_id: String,
    pub tag: String,
}

/// QMP command `snapshot-load`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "snapshot-load")]
#[qapi(returns = "()")]
#[qapi(added = "6.0.0")]
pub struct SnapshotLoad {
    pub devices: Vec<String>,
    #[qapi(name = "job-id")]
    pub job_id: String,
    pub tag: String,
    pub vmstate: String,
}

/// QMP command `snapshot-save`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "snapshot-save")]
#[qapi(returns = "()")]
#[qapi(added = "6.0.0")]
pub struct SnapshotSave {
    pub devices: Vec<String>,
    #[qapi(name = "job-id")]
    pub job_id: String,
    pub tag: String,
    pub vmstate: String,
}

/// QMP command `stop`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "stop")]
#[qapi(returns = "()")]
#[qapi(added = "1.0.0")]
pub struct Stop {}

/// QMP command `system_powerdown`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "system_powerdown")]
#[qapi(returns = "()")]
#[qapi(added = "1.0.0")]
pub struct SystemPowerdown {}

/// QMP command `system_reset`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "system_reset")]
#[qapi(returns = "()")]
#[qapi(added = "1.0.0")]
pub struct SystemReset {}

/// QMP command `system_wakeup`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "system_wakeup")]
#[qapi(returns = "()")]
#[qapi(added = "1.1.0")]
pub struct SystemWakeup {}

/// QMP command `trace-event-get-state`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "trace-event-get-state")]
#[qapi(returns = "Vec<TraceEventInfo>")]
#[qapi(added = "2.2.0")]
pub struct TraceEventGetState {
    pub name: String,
    #[qapi(added = "2.7.0")]
    #[qapi(deprecated(added = "8.1.0"))]
    #[qapi(removed = "9.1.0")]
    pub vcpu: Option<i64>,
}

/// QMP command `trace-event-set-state`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "trace-event-set-state")]
#[qapi(returns = "()")]
#[qapi(added = "2.2.0")]
pub struct TraceEventSetState {
    pub enable: bool,
    #[qapi(name = "ignore-unavailable")]
    pub ignore_unavailable: Option<bool>,
    pub name: String,
    #[qapi(added = "2.7.0")]
    #[qapi(deprecated(added = "8.1.0"))]
    #[qapi(removed = "9.1.0")]
    pub vcpu: Option<i64>,
}

/// QMP command `transaction`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "transaction")]
#[qapi(returns = "()")]
#[qapi(added = "1.1.0")]
pub struct Transaction {
    #[qapi(added = "1.6.0")]
    #[qapi(required)]
    pub actions: Option<Vec<TransactionAction>>,
    #[qapi(name = "actions")]
    #[qapi(removed = "1.6.0")]
    #[qapi(required)]
    pub actions_legacy: Option<Vec<BlockdevAction>>,
    #[qapi(added = "2.5.0")]
    pub properties: Option<TransactionProperties>,
}

/// QMP command `watchdog-set-action`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "watchdog-set-action")]
#[qapi(returns = "()")]
#[qapi(added = "2.11.0")]
pub struct WatchdogSetAction {
    pub action: WatchdogAction,
}

/// QMP command `x-accel-stats`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-accel-stats")]
#[qapi(returns = "HumanReadableText")]
#[qapi(added = "10.1.0")]
#[qapi(unstable(added = "10.1.0"))]
pub struct XAccelStats {}

/// QMP command `x-blockdev-amend`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-blockdev-amend")]
#[qapi(returns = "()")]
#[qapi(added = "5.1.0")]
#[qapi(unstable(added = "6.2.0"))]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct XBlockdevAmend {
    pub force: Option<bool>,
    #[qapi(name = "job-id")]
    pub job_id: String,
    #[qapi(name = "node-name")]
    pub node_name: String,
    pub options: BlockdevAmendOptions,
}

/// QMP command `x-blockdev-change`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-blockdev-change")]
#[qapi(returns = "()")]
#[qapi(added = "2.7.0")]
#[qapi(unstable(added = "6.2.0"))]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct XBlockdevChange {
    pub child: Option<String>,
    pub node: Option<String>,
    pub parent: String,
}

/// QMP command `x-blockdev-create`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-blockdev-create")]
#[qapi(returns = "()")]
#[qapi(added = "2.12.0")]
#[qapi(removed = "3.0.0")]
pub struct XBlockdevCreate {
    #[qapi(discriminator)]
    pub driver: BlockdevDriver,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<BlockdevCreateOptionsBranch>,
}

/// QMP command `x-blockdev-del`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-blockdev-del")]
#[qapi(returns = "()")]
#[qapi(added = "2.5.0")]
#[qapi(removed = "2.9.0")]
pub struct XBlockdevDel {
    #[qapi(removed = "2.8.0")]
    pub id: Option<String>,
    #[qapi(name = "node-name")]
    #[qapi(added = "2.8.0")]
    #[qapi(required)]
    pub node_name: Option<String>,
    #[qapi(name = "node-name")]
    #[qapi(removed = "2.8.0")]
    pub node_name_legacy: Option<String>,
}

/// QMP command `x-blockdev-insert-medium`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-blockdev-insert-medium")]
#[qapi(returns = "()")]
#[qapi(added = "2.5.0")]
#[qapi(removed = "2.12.0")]
pub struct XBlockdevInsertMedium {
    #[qapi(added = "2.8.0")]
    pub device: Option<String>,
    #[qapi(name = "device")]
    #[qapi(removed = "2.8.0")]
    #[qapi(required)]
    pub device_legacy: Option<String>,
    #[qapi(added = "2.8.0")]
    pub id: Option<String>,
    #[qapi(name = "node-name")]
    pub node_name: String,
}

/// QMP command `x-blockdev-remove-medium`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-blockdev-remove-medium")]
#[qapi(returns = "()")]
#[qapi(added = "2.5.0")]
#[qapi(removed = "2.12.0")]
pub struct XBlockdevRemoveMedium {
    #[qapi(added = "2.8.0")]
    pub device: Option<String>,
    #[qapi(name = "device")]
    #[qapi(removed = "2.8.0")]
    #[qapi(required)]
    pub device_legacy: Option<String>,
    #[qapi(added = "2.8.0")]
    pub id: Option<String>,
}

/// QMP command `x-blockdev-reopen`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-blockdev-reopen")]
#[qapi(returns = "()")]
#[qapi(added = "4.0.0")]
#[qapi(removed = "6.1.0")]
pub struct XBlockdevReopen {
    #[qapi(name = "auto-read-only")]
    pub auto_read_only: Option<bool>,
    pub cache: Option<BlockdevCacheOptions>,
    #[qapi(name = "detect-zeroes")]
    pub detect_zeroes: Option<BlockdevDetectZeroesOptions>,
    pub discard: Option<BlockdevDiscardOptions>,
    #[qapi(discriminator)]
    pub driver: BlockdevDriver,
    #[qapi(name = "force-share")]
    pub force_share: Option<bool>,
    #[qapi(name = "node-name")]
    pub node_name: Option<String>,
    #[qapi(name = "read-only")]
    pub read_only: Option<bool>,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<BlockdevOptionsBranch>,
}

/// QMP command `x-blockdev-set-iothread`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-blockdev-set-iothread")]
#[qapi(returns = "()")]
#[qapi(added = "2.12.0")]
#[qapi(unstable(added = "6.2.0"))]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct XBlockdevSetIothread {
    pub force: Option<bool>,
    pub iothread: StrOrNull,
    #[qapi(name = "node-name")]
    pub node_name: String,
}

/// QMP command `x-block-dirty-bitmap-disable`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-block-dirty-bitmap-disable")]
#[qapi(returns = "()")]
#[qapi(added = "3.0.0")]
#[qapi(removed = "4.0.0")]
pub struct XBlockDirtyBitmapDisable {
    pub name: String,
    pub node: String,
}

/// QMP command `x-block-dirty-bitmap-enable`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-block-dirty-bitmap-enable")]
#[qapi(returns = "()")]
#[qapi(added = "3.0.0")]
#[qapi(removed = "4.0.0")]
pub struct XBlockDirtyBitmapEnable {
    pub name: String,
    pub node: String,
}

/// QMP command `x-block-dirty-bitmap-merge`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-block-dirty-bitmap-merge")]
#[qapi(returns = "()")]
#[qapi(added = "3.0.0")]
#[qapi(removed = "4.0.0")]
pub struct XBlockDirtyBitmapMerge {
    pub dst_name: String,
    pub node: String,
    pub src_name: String,
}

/// QMP command `x-block-latency-histogram-set`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-block-latency-histogram-set")]
#[qapi(returns = "()")]
#[qapi(added = "2.12.0")]
#[qapi(removed = "4.0.0")]
pub struct XBlockLatencyHistogramSet {
    pub boundaries: Option<Vec<u64>>,
    #[qapi(name = "boundaries-flush")]
    pub boundaries_flush: Option<Vec<u64>>,
    #[qapi(name = "boundaries-read")]
    pub boundaries_read: Option<Vec<u64>>,
    #[qapi(name = "boundaries-write")]
    pub boundaries_write: Option<Vec<u64>>,
    pub device: String,
}

/// QMP command `x-colo-lost-heartbeat`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-colo-lost-heartbeat")]
#[qapi(returns = "()")]
#[qapi(condition = "CONFIG_REPLICATION")]
#[qapi(added = "2.8.0")]
#[qapi(unstable(added = "6.2.0"))]
pub struct XColoLostHeartbeat {}

/// QMP command `x-debug-block-dirty-bitmap-sha256`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-debug-block-dirty-bitmap-sha256")]
#[qapi(returns = "BlockDirtyBitmapSha256")]
#[qapi(added = "2.10.0")]
#[qapi(unstable(added = "6.2.0"))]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct XDebugBlockDirtyBitmapSha256 {
    pub name: String,
    pub node: String,
}

/// QMP command `x-debug-query-block-graph`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-debug-query-block-graph")]
#[qapi(returns = "XDbgBlockGraph")]
#[qapi(added = "4.0.0")]
#[qapi(unstable(added = "6.2.0"))]
#[qapi(allow_preconfig(added = "7.1.0"))]
pub struct XDebugQueryBlockGraph {}

/// QMP command `xen-colo-do-checkpoint`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "xen-colo-do-checkpoint")]
#[qapi(returns = "()")]
#[qapi(condition = "CONFIG_REPLICATION")]
#[qapi(added = "2.9.0")]
pub struct XenColoDoCheckpoint {}

/// QMP command `xen-event-inject`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "xen-event-inject")]
#[qapi(returns = "()")]
#[qapi(added = "8.0.0")]
pub struct XenEventInject {
    pub port: u32,
}

/// QMP command `xen-event-list`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "xen-event-list")]
#[qapi(returns = "Vec<EvtchnInfo>")]
#[qapi(added = "8.0.0")]
pub struct XenEventList {}

/// QMP command `xen-load-devices-state`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "xen-load-devices-state")]
#[qapi(returns = "()")]
#[qapi(added = "2.7.0")]
pub struct XenLoadDevicesState {
    pub filename: String,
}

/// QMP command `xen-save-devices-state`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "xen-save-devices-state")]
#[qapi(returns = "()")]
#[qapi(added = "1.1.0")]
pub struct XenSaveDevicesState {
    pub filename: String,
    #[qapi(added = "2.11.0")]
    pub live: Option<bool>,
}

/// QMP command `xen-set-global-dirty-log`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "xen-set-global-dirty-log")]
#[qapi(returns = "()")]
#[qapi(added = "1.3.0")]
pub struct XenSetGlobalDirtyLog {
    pub enable: bool,
}

/// QMP command `xen-set-replication`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "xen-set-replication")]
#[qapi(returns = "()")]
#[qapi(condition = "CONFIG_REPLICATION")]
#[qapi(added = "2.9.0")]
pub struct XenSetReplication {
    pub enable: bool,
    pub failover: Option<bool>,
    pub primary: bool,
}

/// QMP command `x-exit-preconfig`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-exit-preconfig")]
#[qapi(returns = "()")]
#[qapi(added = "3.0.0")]
#[qapi(unstable(added = "6.2.0"))]
#[qapi(allow_preconfig)]
pub struct XExitPreconfig {}

/// QMP command `x-input-send-event`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-input-send-event")]
#[qapi(returns = "()")]
#[qapi(added = "2.2.0")]
#[qapi(removed = "2.6.0")]
pub struct XInputSendEvent {
    pub console: Option<i64>,
    pub events: Vec<InputEvent>,
}

/// QMP command `x-nbd-server-add-bitmap`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-nbd-server-add-bitmap")]
#[qapi(returns = "()")]
#[qapi(added = "3.0.0")]
#[qapi(removed = "4.0.0")]
pub struct XNbdServerAddBitmap {
    pub bitmap: String,
    #[qapi(name = "bitmap-export-name")]
    pub bitmap_export_name: Option<String>,
    pub name: String,
}

/// QMP command `x-oob-test`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-oob-test")]
#[qapi(returns = "()")]
#[qapi(added = "2.12.0")]
#[qapi(removed = "3.0.0")]
#[qapi(allow_oob)]
pub struct XOobTest {
    pub lock: bool,
}

/// QMP command `x-query-interrupt-controllers`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-query-interrupt-controllers")]
#[qapi(returns = "HumanReadableText")]
#[qapi(added = "9.1.0")]
#[qapi(unstable(added = "9.1.0"))]
pub struct XQueryInterruptControllers {}

/// QMP command `x-query-irq`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-query-irq")]
#[qapi(returns = "HumanReadableText")]
#[qapi(added = "6.2.0")]
#[qapi(unstable(added = "6.2.0"))]
pub struct XQueryIrq {}

/// QMP command `x-query-jit`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-query-jit")]
#[qapi(condition = "CONFIG_TCG")]
#[qapi(returns = "HumanReadableText")]
#[qapi(added = "6.2.0")]
#[qapi(unstable(added = "6.2.0"))]
pub struct XQueryJit {}

/// QMP command `x-query-numa`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-query-numa")]
#[qapi(returns = "HumanReadableText")]
#[qapi(added = "6.2.0")]
#[qapi(unstable(added = "6.2.0"))]
pub struct XQueryNuma {}

/// QMP command `x-query-opcount`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-query-opcount")]
#[qapi(condition = "CONFIG_TCG")]
#[qapi(returns = "HumanReadableText")]
#[qapi(added = "6.2.0")]
#[qapi(unstable(added = "6.2.0"))]
#[qapi(removed = "10.1.0")]
pub struct XQueryOpcount {}

/// QMP command `x-query-profile`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-query-profile")]
#[qapi(returns = "HumanReadableText")]
#[qapi(condition = "CONFIG_TCG")]
#[qapi(added = "6.2.0")]
#[qapi(unstable(added = "6.2.0"))]
#[qapi(removed = "8.1.0")]
pub struct XQueryProfile {}

/// QMP command `x-query-ramblock`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-query-ramblock")]
#[qapi(returns = "HumanReadableText")]
#[qapi(added = "6.2.0")]
#[qapi(unstable(added = "6.2.0"))]
pub struct XQueryRamblock {}

/// QMP command `x-query-rdma`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-query-rdma")]
#[qapi(returns = "HumanReadableText")]
#[qapi(added = "6.2.0")]
#[qapi(unstable(added = "6.2.0"))]
#[qapi(removed = "9.1.0")]
pub struct XQueryRdma {}

/// QMP command `x-query-roms`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-query-roms")]
#[qapi(returns = "HumanReadableText")]
#[qapi(added = "6.2.0")]
#[qapi(unstable(added = "6.2.0"))]
pub struct XQueryRoms {}

/// QMP command `x-query-usb`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-query-usb")]
#[qapi(returns = "HumanReadableText")]
#[qapi(added = "6.2.0")]
#[qapi(unstable(added = "6.2.0"))]
pub struct XQueryUsb {}

/// QMP command `x-query-virtio`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-query-virtio")]
#[qapi(returns = "Vec<VirtioInfo>")]
#[qapi(added = "7.2.0")]
#[qapi(unstable(added = "7.2.0"))]
pub struct XQueryVirtio {}

/// QMP command `x-query-virtio-queue-element`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-query-virtio-queue-element")]
#[qapi(returns = "VirtioQueueElement")]
#[qapi(added = "7.2.0")]
#[qapi(unstable(added = "7.2.0"))]
pub struct XQueryVirtioQueueElement {
    pub index: Option<u16>,
    pub path: String,
    pub queue: u16,
}

/// QMP command `x-query-virtio-queue-status`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-query-virtio-queue-status")]
#[qapi(returns = "VirtQueueStatus")]
#[qapi(added = "7.2.0")]
#[qapi(unstable(added = "7.2.0"))]
pub struct XQueryVirtioQueueStatus {
    pub path: String,
    pub queue: u16,
}

/// QMP command `x-query-virtio-status`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-query-virtio-status")]
#[qapi(returns = "VirtioStatus")]
#[qapi(added = "7.2.0")]
#[qapi(unstable(added = "7.2.0"))]
pub struct XQueryVirtioStatus {
    pub path: String,
}

/// QMP command `x-query-virtio-vhost-queue-status`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "x-query-virtio-vhost-queue-status")]
#[qapi(returns = "VirtVhostQueueStatus")]
#[qapi(added = "7.2.0")]
#[qapi(unstable(added = "7.2.0"))]
pub struct XQueryVirtioVhostQueueStatus {
    pub path: String,
    pub queue: u16,
}

/// QMP command `yank`; see the QEMU QMP Reference Manual.
#[qapi(command)]
#[qapi(name = "yank")]
#[qapi(returns = "()")]
#[qapi(added = "6.0.0")]
#[qapi(allow_oob)]
pub struct Yank {
    pub instances: Vec<YankInstance>,
}
