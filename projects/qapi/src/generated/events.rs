use crate as qapi;
use qapi_macros::qapi;
use super::*;
/// QMP event `ACPI_DEVICE_OST`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "ACPI_DEVICE_OST")]
#[qapi(added = "2.1.0")]
pub struct AcpiDeviceOstEvent {
    pub info: AcpiostInfo,
}

/// QMP event `BALLOON_CHANGE`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "BALLOON_CHANGE")]
#[qapi(added = "2.1.0")]
pub struct BalloonChangeEvent {
    pub actual: i64,
}

/// QMP event `BLOCK_EXPORT_DELETED`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "BLOCK_EXPORT_DELETED")]
#[qapi(added = "5.2.0")]
pub struct BlockExportDeletedEvent {
    pub id: String,
}

/// QMP event `BLOCK_IMAGE_CORRUPTED`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "BLOCK_IMAGE_CORRUPTED")]
#[qapi(added = "2.1.0")]
pub struct BlockImageCorruptedEvent {
    pub device: String,
    #[qapi(added = "2.2.0")]
    #[qapi(required)]
    pub fatal: Option<bool>,
    pub msg: String,
    #[qapi(name = "node-name")]
    #[qapi(added = "2.4.0")]
    pub node_name: Option<String>,
    pub offset: Option<i64>,
    pub size: Option<i64>,
}

/// QMP event `BLOCK_IO_ERROR`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "BLOCK_IO_ERROR")]
#[qapi(added = "2.1.0")]
pub struct BlockIoErrorEvent {
    pub action: BlockErrorAction,
    pub device: String,
    #[qapi(name = "node-name")]
    #[qapi(added = "2.12.0")]
    pub node_name: Option<String>,
    #[qapi(name = "node-name")]
    #[qapi(added = "2.8.0")]
    #[qapi(removed = "2.12.0")]
    #[qapi(required)]
    pub node_name_legacy: Option<String>,
    #[qapi(added = "2.2.0")]
    pub nospace: Option<bool>,
    pub operation: IoOperationType,
    #[qapi(name = "qom-path")]
    #[qapi(added = "9.2.0")]
    #[qapi(required)]
    pub qom_path: Option<String>,
    #[qapi(added = "2.2.0")]
    #[qapi(required)]
    pub reason: Option<String>,
}

/// QMP event `BLOCK_JOB_CANCELLED`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "BLOCK_JOB_CANCELLED")]
#[qapi(added = "2.1.0")]
pub struct BlockJobCancelledEvent {
    pub device: String,
    pub len: i64,
    pub offset: i64,
    pub speed: i64,
    #[qapi(added = "3.0.0")]
    #[qapi(required)]
    pub r#type: Option<JobType>,
    #[qapi(name = "type")]
    #[qapi(removed = "3.0.0")]
    #[qapi(required)]
    pub type_legacy: Option<BlockJobType>,
}

/// QMP event `BLOCK_JOB_COMPLETED`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "BLOCK_JOB_COMPLETED")]
#[qapi(added = "2.1.0")]
pub struct BlockJobCompletedEvent {
    pub device: String,
    pub error: Option<String>,
    pub len: i64,
    pub offset: i64,
    pub speed: i64,
    #[qapi(added = "3.0.0")]
    #[qapi(required)]
    pub r#type: Option<JobType>,
    #[qapi(name = "type")]
    #[qapi(removed = "3.0.0")]
    #[qapi(required)]
    pub type_legacy: Option<BlockJobType>,
}

/// QMP event `BLOCK_JOB_ERROR`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "BLOCK_JOB_ERROR")]
#[qapi(added = "2.1.0")]
pub struct BlockJobErrorEvent {
    pub action: BlockErrorAction,
    pub device: String,
    pub operation: IoOperationType,
}

/// QMP event `BLOCK_JOB_PENDING`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "BLOCK_JOB_PENDING")]
#[qapi(added = "2.12.0")]
pub struct BlockJobPendingEvent {
    pub id: String,
    #[qapi(added = "3.0.0")]
    #[qapi(required)]
    pub r#type: Option<JobType>,
    #[qapi(name = "type")]
    #[qapi(removed = "3.0.0")]
    #[qapi(required)]
    pub type_legacy: Option<BlockJobType>,
}

/// QMP event `BLOCK_JOB_READY`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "BLOCK_JOB_READY")]
#[qapi(added = "2.1.0")]
pub struct BlockJobReadyEvent {
    pub device: String,
    pub len: i64,
    pub offset: i64,
    pub speed: i64,
    #[qapi(added = "3.0.0")]
    #[qapi(required)]
    pub r#type: Option<JobType>,
    #[qapi(name = "type")]
    #[qapi(removed = "3.0.0")]
    #[qapi(required)]
    pub type_legacy: Option<BlockJobType>,
}

/// QMP event `BLOCK_WRITE_THRESHOLD`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "BLOCK_WRITE_THRESHOLD")]
#[qapi(added = "2.3.0")]
pub struct BlockWriteThresholdEvent {
    #[qapi(name = "amount-exceeded")]
    pub amount_exceeded: u64,
    #[qapi(name = "node-name")]
    pub node_name: String,
    #[qapi(name = "write-threshold")]
    pub write_threshold: u64,
}

/// QMP event `COLO_EXIT`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "COLO_EXIT")]
#[qapi(added = "3.1.0")]
pub struct ColoExitEvent {
    pub mode: ColoMode,
    pub reason: ColoExitReason,
}

/// QMP event `COMMAND_DROPPED`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "COMMAND_DROPPED")]
#[qapi(added = "2.12.0")]
#[qapi(removed = "4.0.0")]
pub struct CommandDroppedEvent {
    pub id: serde_json::Value,
    pub reason: CommandDropReason,
}

/// QMP event `CPU_POLARIZATION_CHANGE`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "CPU_POLARIZATION_CHANGE")]
#[qapi(added = "8.2.0")]
#[qapi(unstable(added = "8.2.0"))]
pub struct CpuPolarizationChangeEvent {
    #[qapi(added = "9.2.0")]
    #[qapi(required)]
    pub polarization: Option<S390CpuPolarization>,
    #[qapi(name = "polarization")]
    #[qapi(removed = "9.2.0")]
    #[qapi(required)]
    pub polarization_legacy: Option<CpuS390Polarization>,
}

/// QMP event `DEVICE_DELETED`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "DEVICE_DELETED")]
#[qapi(added = "2.1.0")]
pub struct DeviceDeletedEvent {
    pub device: Option<String>,
    pub path: String,
}

/// QMP event `DEVICE_TRAY_MOVED`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "DEVICE_TRAY_MOVED")]
#[qapi(added = "2.1.0")]
pub struct DeviceTrayMovedEvent {
    pub device: String,
    #[qapi(added = "2.8.0")]
    #[qapi(required)]
    pub id: Option<String>,
    #[qapi(name = "tray-open")]
    pub tray_open: bool,
}

/// QMP event `DEVICE_UNPLUG_GUEST_ERROR`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "DEVICE_UNPLUG_GUEST_ERROR")]
#[qapi(added = "6.2.0")]
pub struct DeviceUnplugGuestErrorEvent {
    pub device: Option<String>,
    pub path: String,
}

/// QMP event `DUMP_COMPLETED`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "DUMP_COMPLETED")]
#[qapi(added = "2.6.0")]
pub struct DumpCompletedEvent {
    pub error: Option<String>,
    pub result: DumpQueryResult,
}

/// QMP event `FAILOVER_NEGOTIATED`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "FAILOVER_NEGOTIATED")]
#[qapi(added = "4.2.0")]
pub struct FailoverNegotiatedEvent {
    #[qapi(name = "device-id")]
    pub device_id: String,
}

/// QMP event `GUEST_CRASHLOADED`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "GUEST_CRASHLOADED")]
#[qapi(added = "5.0.0")]
pub struct GuestCrashloadedEvent {
    pub action: GuestPanicAction,
    pub info: Option<GuestPanicInformation>,
}

/// QMP event `GUEST_PANICKED`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "GUEST_PANICKED")]
#[qapi(added = "2.1.0")]
pub struct GuestPanickedEvent {
    pub action: GuestPanicAction,
    #[qapi(added = "2.9.0")]
    pub info: Option<GuestPanicInformation>,
}

/// QMP event `GUEST_PVSHUTDOWN`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "GUEST_PVSHUTDOWN")]
#[qapi(added = "9.1.0")]
pub struct GuestPvshutdownEvent {}

/// QMP event `HV_BALLOON_STATUS_REPORT`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "HV_BALLOON_STATUS_REPORT")]
#[qapi(added = "8.2.0")]
pub struct HvBalloonStatusReportEvent {
    pub available: u64,
    pub committed: u64,
}

/// QMP event `JOB_STATUS_CHANGE`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "JOB_STATUS_CHANGE")]
#[qapi(added = "3.0.0")]
pub struct JobStatusChangeEvent {
    pub id: String,
    pub status: JobStatus,
}

/// QMP event `MEMORY_DEVICE_SIZE_CHANGE`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "MEMORY_DEVICE_SIZE_CHANGE")]
#[qapi(added = "5.1.0")]
pub struct MemoryDeviceSizeChangeEvent {
    pub id: Option<String>,
    #[qapi(name = "qom-path")]
    #[qapi(added = "6.2.0")]
    #[qapi(required)]
    pub qom_path: Option<String>,
    pub size: u64,
}

/// QMP event `MEMORY_FAILURE`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "MEMORY_FAILURE")]
#[qapi(added = "5.2.0")]
pub struct MemoryFailureEvent {
    pub action: MemoryFailureAction,
    pub flags: MemoryFailureFlags,
    pub recipient: MemoryFailureRecipient,
}

/// QMP event `MEM_UNPLUG_ERROR`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "MEM_UNPLUG_ERROR")]
#[qapi(added = "2.4.0")]
#[qapi(deprecated(added = "6.2.0"))]
#[qapi(removed = "9.1.0")]
pub struct MemUnplugErrorEvent {
    pub device: String,
    pub msg: String,
}

/// QMP event `MIGRATION`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "MIGRATION")]
#[qapi(added = "2.4.0")]
pub struct MigrationEvent {
    pub status: MigrationStatus,
}

/// QMP event `MIGRATION_PASS`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "MIGRATION_PASS")]
#[qapi(added = "2.6.0")]
pub struct MigrationPassEvent {
    pub pass: i64,
}

/// QMP event `NETDEV_STREAM_CONNECTED`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "NETDEV_STREAM_CONNECTED")]
#[qapi(added = "7.2.0")]
pub struct NetdevStreamConnectedEvent {
    pub addr: SocketAddress,
    #[qapi(name = "netdev-id")]
    pub netdev_id: String,
}

/// QMP event `NETDEV_STREAM_DISCONNECTED`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "NETDEV_STREAM_DISCONNECTED")]
#[qapi(added = "7.2.0")]
pub struct NetdevStreamDisconnectedEvent {
    #[qapi(name = "netdev-id")]
    pub netdev_id: String,
}

/// QMP event `NETDEV_VHOST_USER_CONNECTED`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "NETDEV_VHOST_USER_CONNECTED")]
#[qapi(added = "10.0.0")]
pub struct NetdevVhostUserConnectedEvent {
    #[qapi(name = "chardev-id")]
    pub chardev_id: String,
    #[qapi(name = "netdev-id")]
    pub netdev_id: String,
}

/// QMP event `NETDEV_VHOST_USER_DISCONNECTED`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "NETDEV_VHOST_USER_DISCONNECTED")]
#[qapi(added = "10.0.0")]
pub struct NetdevVhostUserDisconnectedEvent {
    #[qapi(name = "netdev-id")]
    pub netdev_id: String,
}

/// QMP event `NIC_RX_FILTER_CHANGED`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "NIC_RX_FILTER_CHANGED")]
#[qapi(added = "2.1.0")]
pub struct NicRxFilterChangedEvent {
    pub name: Option<String>,
    pub path: String,
}

/// QMP event `POWERDOWN`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "POWERDOWN")]
#[qapi(added = "2.1.0")]
pub struct PowerdownEvent {}

/// QMP event `PR_MANAGER_STATUS_CHANGED`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "PR_MANAGER_STATUS_CHANGED")]
#[qapi(added = "3.0.0")]
pub struct PrManagerStatusChangedEvent {
    pub connected: bool,
    pub id: String,
}

/// QMP event `QUORUM_FAILURE`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "QUORUM_FAILURE")]
#[qapi(added = "2.1.0")]
pub struct QuorumFailureEvent {
    pub reference: String,
    #[qapi(name = "sector-num")]
    pub sector_num: i64,
    #[qapi(name = "sectors-count")]
    pub sectors_count: i64,
}

/// QMP event `QUORUM_REPORT_BAD`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "QUORUM_REPORT_BAD")]
#[qapi(added = "2.1.0")]
pub struct QuorumReportBadEvent {
    pub error: Option<String>,
    #[qapi(name = "node-name")]
    pub node_name: String,
    #[qapi(name = "sector-num")]
    pub sector_num: i64,
    #[qapi(name = "sectors-count")]
    pub sectors_count: i64,
    #[qapi(added = "2.6.0")]
    #[qapi(required)]
    pub r#type: Option<QuorumOpType>,
}

/// QMP event `RDMA_GID_STATUS_CHANGED`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "RDMA_GID_STATUS_CHANGED")]
#[qapi(added = "4.0.0")]
#[qapi(removed = "9.1.0")]
pub struct RdmaGidStatusChangedEvent {
    #[qapi(name = "gid-status")]
    pub gid_status: bool,
    #[qapi(name = "interface-id")]
    pub interface_id: u64,
    pub netdev: String,
    #[qapi(name = "subnet-prefix")]
    pub subnet_prefix: u64,
}

/// QMP event `RESET`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "RESET")]
#[qapi(added = "2.1.0")]
pub struct ResetEvent {
    #[qapi(added = "2.10.0")]
    #[qapi(required)]
    pub guest: Option<bool>,
    #[qapi(added = "4.0.0")]
    #[qapi(required)]
    pub reason: Option<ShutdownCause>,
}

/// QMP event `RESUME`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "RESUME")]
#[qapi(added = "2.1.0")]
pub struct ResumeEvent {}

/// QMP event `RTC_CHANGE`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "RTC_CHANGE")]
#[qapi(added = "2.1.0")]
pub struct RtcChangeEvent {
    pub offset: i64,
    #[qapi(name = "qom-path")]
    #[qapi(added = "7.0.0")]
    #[qapi(required)]
    pub qom_path: Option<String>,
}

/// QMP event `SCLP_CPI_INFO_AVAILABLE`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "SCLP_CPI_INFO_AVAILABLE")]
#[qapi(added = "10.2.0")]
#[qapi(unstable(added = "10.2.0"))]
pub struct SclpCpiInfoAvailableEvent {}

/// QMP event `SHUTDOWN`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "SHUTDOWN")]
#[qapi(added = "2.1.0")]
pub struct ShutdownEvent {
    #[qapi(added = "2.10.0")]
    #[qapi(required)]
    pub guest: Option<bool>,
    #[qapi(added = "4.0.0")]
    #[qapi(required)]
    pub reason: Option<ShutdownCause>,
}

/// QMP event `SPICE_CONNECTED`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "SPICE_CONNECTED")]
#[qapi(condition = "CONFIG_SPICE")]
#[qapi(added = "2.1.0")]
pub struct SpiceConnectedEvent {
    pub client: SpiceBasicInfo,
    pub server: SpiceBasicInfo,
}

/// QMP event `SPICE_DISCONNECTED`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "SPICE_DISCONNECTED")]
#[qapi(condition = "CONFIG_SPICE")]
#[qapi(added = "2.1.0")]
pub struct SpiceDisconnectedEvent {
    pub client: SpiceBasicInfo,
    pub server: SpiceBasicInfo,
}

/// QMP event `SPICE_INITIALIZED`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "SPICE_INITIALIZED")]
#[qapi(condition = "CONFIG_SPICE")]
#[qapi(added = "2.1.0")]
pub struct SpiceInitializedEvent {
    pub client: SpiceChannel,
    pub server: SpiceServerInfo,
}

/// QMP event `SPICE_MIGRATE_COMPLETED`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "SPICE_MIGRATE_COMPLETED")]
#[qapi(condition = "CONFIG_SPICE")]
#[qapi(added = "2.1.0")]
pub struct SpiceMigrateCompletedEvent {}

/// QMP event `STOP`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "STOP")]
#[qapi(added = "2.1.0")]
pub struct StopEvent {}

/// QMP event `SUSPEND_DISK`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "SUSPEND_DISK")]
#[qapi(added = "2.1.0")]
pub struct SuspendDiskEvent {}

/// QMP event `SUSPEND`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "SUSPEND")]
#[qapi(added = "2.1.0")]
pub struct SuspendEvent {}

/// QMP event `UNPLUG_PRIMARY`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "UNPLUG_PRIMARY")]
#[qapi(added = "4.2.0")]
pub struct UnplugPrimaryEvent {
    #[qapi(name = "device-id")]
    pub device_id: String,
}

/// QMP event `VFIO_MIGRATION`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "VFIO_MIGRATION")]
#[qapi(added = "9.1.0")]
pub struct VfioMigrationEvent {
    #[qapi(name = "device-id")]
    pub device_id: String,
    #[qapi(name = "device-state")]
    #[qapi(added = "9.2.0")]
    #[qapi(required)]
    pub device_state: Option<QapiVfioMigrationState>,
    #[qapi(name = "device-state")]
    #[qapi(removed = "9.2.0")]
    #[qapi(required)]
    pub device_state_legacy: Option<VfioMigrationState>,
    #[qapi(name = "qom-path")]
    pub qom_path: String,
}

/// QMP event `VFU_CLIENT_HANGUP`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "VFU_CLIENT_HANGUP")]
#[qapi(added = "7.1.0")]
pub struct VfuClientHangupEvent {
    #[qapi(name = "dev-id")]
    pub dev_id: String,
    #[qapi(name = "dev-qom-path")]
    pub dev_qom_path: String,
    #[qapi(name = "vfu-id")]
    pub vfu_id: String,
    #[qapi(name = "vfu-qom-path")]
    pub vfu_qom_path: String,
}

/// QMP event `VNC_CONNECTED`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "VNC_CONNECTED")]
#[qapi(condition = "CONFIG_VNC")]
#[qapi(added = "2.1.0")]
pub struct VncConnectedEvent {
    pub client: VncBasicInfo,
    pub server: VncServerInfo,
}

/// QMP event `VNC_DISCONNECTED`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "VNC_DISCONNECTED")]
#[qapi(condition = "CONFIG_VNC")]
#[qapi(added = "2.1.0")]
pub struct VncDisconnectedEvent {
    pub client: VncClientInfo,
    pub server: VncServerInfo,
}

/// QMP event `VNC_INITIALIZED`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "VNC_INITIALIZED")]
#[qapi(condition = "CONFIG_VNC")]
#[qapi(added = "2.1.0")]
pub struct VncInitializedEvent {
    pub client: VncClientInfo,
    pub server: VncServerInfo,
}

/// QMP event `VSERPORT_CHANGE`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "VSERPORT_CHANGE")]
#[qapi(added = "2.1.0")]
pub struct VserportChangeEvent {
    pub id: String,
    pub open: bool,
}

/// QMP event `WAKEUP`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "WAKEUP")]
#[qapi(added = "2.1.0")]
pub struct WakeupEvent {}

/// QMP event `WATCHDOG`; see the QEMU QMP Reference Manual.
#[qapi(event)]
#[qapi(name = "WATCHDOG")]
#[qapi(added = "2.1.0")]
pub struct WatchdogEvent {
    #[qapi(added = "2.11.0")]
    #[qapi(required)]
    pub action: Option<WatchdogAction>,
    #[qapi(name = "action")]
    #[qapi(removed = "2.11.0")]
    #[qapi(required)]
    pub action_legacy: Option<WatchdogExpirationAction>,
}
