use qapi_macros::qapi;
/// QEMU Guest Agent struct `GuestAgentCommandInfo`; see the QEMU Guest Agent Protocol Reference.
#[qapi(added = "1.1.0")]
pub struct GuestAgentCommandInfo {
    pub enabled: bool,
    pub name: String,
    #[qapi(name = "success-response")]
    #[qapi(added = "1.7.0")]
    #[qapi(required)]
    pub success_response: Option<bool>,
}

/// QEMU Guest Agent struct `GuestAgentInfo`; see the QEMU Guest Agent Protocol Reference.
#[qapi(added = "0.15.0")]
pub struct GuestAgentInfo {
    #[qapi(added = "1.1.0")]
    #[qapi(required)]
    pub supported_commands: Option<Vec<GuestAgentCommandInfo>>,
    pub version: String,
}

/// QEMU Guest Agent struct `GuestAuthorizedKeys`; see the QEMU Guest Agent Protocol Reference.
#[qapi(added = "5.2.0")]
pub struct GuestAuthorizedKeys {
    pub keys: Vec<String>,
}

/// QEMU Guest Agent struct `GuestCCWAddress`; see the QEMU Guest Agent Protocol Reference.
#[qapi(name = "GuestCCWAddress")]
#[qapi(condition = "(CONFIG_WIN32 || CONFIG_LINUX)")]
#[qapi(added = "6.0.0")]
pub struct GuestCcwAddress {
    pub cssid: i64,
    pub devno: i64,
    pub ssid: i64,
    pub subchno: i64,
}

/// QEMU Guest Agent union `GuestCpuStats`; see the QEMU Guest Agent Protocol Reference.
#[qapi(condition = "CONFIG_LINUX")]
#[qapi(added = "7.1.0")]
pub struct GuestCpuStats {
    #[qapi(discriminator)]
    pub r#type: GuestCpuStatsType,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<GuestCpuStatsBranch>,
}

#[qapi(union_branch)]
#[qapi(condition = "CONFIG_LINUX")]
#[qapi(added = "7.1.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum GuestCpuStatsBranch {
    #[qapi(name = "linux")]
    Linux(GuestLinuxCpuStats),
}

/// QEMU Guest Agent enum `GuestCpuStatsType`; see the QEMU Guest Agent Protocol Reference.
#[qapi(condition = "CONFIG_LINUX")]
#[qapi(added = "7.1.0")]
pub enum GuestCpuStatsType {
    #[qapi(name = "linux")]
    Linux,
}

/// QEMU Guest Agent union `GuestDeviceId`; see the QEMU Guest Agent Protocol Reference.
#[qapi(condition = "CONFIG_WIN32")]
#[qapi(added = "5.2.0")]
pub struct GuestDeviceId {
    #[qapi(discriminator)]
    pub r#type: GuestDeviceType,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<GuestDeviceIdBranch>,
}

#[qapi(union_branch)]
#[qapi(condition = "CONFIG_WIN32")]
#[qapi(added = "5.2.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum GuestDeviceIdBranch {
    #[qapi(name = "pci")]
    Pci(GuestDeviceIdPci),
}

/// QEMU Guest Agent struct `GuestDeviceIdPCI`; see the QEMU Guest Agent Protocol Reference.
#[qapi(name = "GuestDeviceIdPCI")]
#[qapi(condition = "CONFIG_WIN32")]
#[qapi(added = "5.2.0")]
pub struct GuestDeviceIdPci {
    #[qapi(name = "device-id")]
    pub device_id: u16,
    #[qapi(name = "vendor-id")]
    pub vendor_id: u16,
}

/// QEMU Guest Agent struct `GuestDeviceInfo`; see the QEMU Guest Agent Protocol Reference.
#[qapi(condition = "CONFIG_WIN32")]
#[qapi(added = "5.2.0")]
pub struct GuestDeviceInfo {
    #[qapi(name = "driver-date")]
    pub driver_date: Option<i64>,
    #[qapi(name = "driver-name")]
    pub driver_name: String,
    #[qapi(name = "driver-version")]
    pub driver_version: Option<String>,
    pub id: Option<GuestDeviceId>,
}

/// QEMU Guest Agent enum `GuestDeviceType`; see the QEMU Guest Agent Protocol Reference.
#[qapi(condition = "CONFIG_WIN32")]
#[qapi(added = "5.2.0")]
pub enum GuestDeviceType {
    #[qapi(name = "pci")]
    Pci,
}

/// QEMU Guest Agent struct `GuestDiskAddress`; see the QEMU Guest Agent Protocol Reference.
#[qapi(condition = "(CONFIG_WIN32 || CONFIG_LINUX)")]
#[qapi(added = "2.2.0")]
pub struct GuestDiskAddress {
    pub bus: i64,
    #[qapi(name = "bus-type")]
    pub bus_type: GuestDiskBusType,
    #[qapi(name = "ccw-address")]
    #[qapi(added = "6.0.0")]
    pub ccw_address: Option<GuestCcwAddress>,
    #[qapi(added = "3.1.0")]
    pub dev: Option<String>,
    #[qapi(name = "pci-controller")]
    pub pci_controller: GuestPciAddress,
    #[qapi(added = "3.1.0")]
    pub serial: Option<String>,
    pub target: i64,
    pub unit: i64,
}

/// QEMU Guest Agent enum `GuestDiskBusType`; see the QEMU Guest Agent Protocol Reference.
#[qapi(condition = "(CONFIG_WIN32 || CONFIG_LINUX)")]
#[qapi(added = "2.2.0")]
pub enum GuestDiskBusType {
    #[qapi(name = "fdc")]
    Fdc,
    #[qapi(name = "fibre")]
    #[qapi(added = "2.4.0")]
    Fibre,
    #[qapi(name = "file-backed-virtual")]
    #[qapi(added = "2.4.0")]
    FileBackedVirtual,
    #[qapi(name = "ide")]
    Ide,
    #[qapi(name = "ieee1394")]
    #[qapi(added = "2.4.0")]
    Ieee1394,
    #[qapi(name = "iscsi")]
    #[qapi(added = "2.4.0")]
    Iscsi,
    #[qapi(name = "mmc")]
    #[qapi(added = "2.4.0")]
    Mmc,
    #[qapi(name = "nvme")]
    #[qapi(added = "7.1.0")]
    Nvme,
    #[qapi(name = "raid")]
    #[qapi(added = "2.4.0")]
    Raid,
    #[qapi(name = "sas")]
    #[qapi(added = "2.4.0")]
    Sas,
    #[qapi(name = "sata")]
    Sata,
    #[qapi(name = "scsi")]
    Scsi,
    #[qapi(name = "sd")]
    Sd,
    #[qapi(name = "ssa")]
    #[qapi(added = "2.4.0")]
    Ssa,
    #[qapi(name = "uml")]
    Uml,
    #[qapi(name = "unknown")]
    #[qapi(added = "2.4.0")]
    Unknown,
    #[qapi(name = "usb")]
    Usb,
    #[qapi(name = "virtio")]
    Virtio,
    #[qapi(name = "virtual")]
    #[qapi(added = "2.4.0")]
    Virtual,
    #[qapi(name = "xen")]
    Xen,
}

/// QEMU Guest Agent struct `GuestDiskInfo`; see the QEMU Guest Agent Protocol Reference.
#[qapi(condition = "(CONFIG_WIN32 || CONFIG_LIBUDEV)")]
#[qapi(added = "5.2.0")]
pub struct GuestDiskInfo {
    pub address: Option<GuestDiskAddress>,
    pub alias: Option<String>,
    pub dependencies: Option<Vec<String>>,
    pub name: String,
    pub partition: bool,
    #[qapi(added = "7.1.0")]
    pub smart: Option<GuestDiskSmart>,
}

/// QEMU Guest Agent union `GuestDiskSmart`; see the QEMU Guest Agent Protocol Reference.
#[qapi(condition = "(CONFIG_WIN32 || CONFIG_LIBUDEV)")]
#[qapi(added = "7.1.0")]
pub struct GuestDiskSmart {
    #[qapi(discriminator)]
    pub r#type: GuestDiskBusType,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<GuestDiskSmartBranch>,
}

#[qapi(union_branch)]
#[qapi(condition = "(CONFIG_WIN32 || CONFIG_LIBUDEV)")]
#[qapi(added = "7.1.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum GuestDiskSmartBranch {
    #[qapi(name = "fdc")]
    Fdc,
    #[qapi(name = "fibre")]
    Fibre,
    #[qapi(name = "file-backed-virtual")]
    FileBackedVirtual,
    #[qapi(name = "ide")]
    Ide,
    #[qapi(name = "ieee1394")]
    Ieee1394,
    #[qapi(name = "iscsi")]
    Iscsi,
    #[qapi(name = "mmc")]
    Mmc,
    #[qapi(name = "nvme")]
    Nvme(GuestNvMeSmart),
    #[qapi(name = "raid")]
    Raid,
    #[qapi(name = "sas")]
    Sas,
    #[qapi(name = "sata")]
    Sata,
    #[qapi(name = "scsi")]
    Scsi,
    #[qapi(name = "sd")]
    Sd,
    #[qapi(name = "ssa")]
    Ssa,
    #[qapi(name = "uml")]
    Uml,
    #[qapi(name = "unknown")]
    Unknown,
    #[qapi(name = "usb")]
    Usb,
    #[qapi(name = "virtio")]
    Virtio,
    #[qapi(name = "virtual")]
    Virtual,
    #[qapi(name = "xen")]
    Xen,
}

/// QEMU Guest Agent struct `GuestDiskStats`; see the QEMU Guest Agent Protocol Reference.
#[qapi(condition = "CONFIG_LINUX")]
#[qapi(added = "7.1.0")]
pub struct GuestDiskStats {
    #[qapi(name = "discard-ios")]
    pub discard_ios: Option<u64>,
    #[qapi(name = "discard-merges")]
    pub discard_merges: Option<u64>,
    #[qapi(name = "discard-sectors")]
    pub discard_sectors: Option<u64>,
    #[qapi(name = "discard-ticks")]
    pub discard_ticks: Option<u64>,
    #[qapi(name = "flush-ios")]
    pub flush_ios: Option<u64>,
    #[qapi(name = "flush-ticks")]
    pub flush_ticks: Option<u64>,
    #[qapi(name = "ios-pgr")]
    pub ios_pgr: Option<u64>,
    #[qapi(name = "read-ios")]
    pub read_ios: Option<u64>,
    #[qapi(name = "read-merges")]
    pub read_merges: Option<u64>,
    #[qapi(name = "read-sectors")]
    pub read_sectors: Option<u64>,
    #[qapi(name = "read-ticks")]
    pub read_ticks: Option<u64>,
    #[qapi(name = "total-ticks")]
    pub total_ticks: Option<u64>,
    #[qapi(name = "weight-ticks")]
    pub weight_ticks: Option<u64>,
    #[qapi(name = "write-ios")]
    pub write_ios: Option<u64>,
    #[qapi(name = "write-merges")]
    pub write_merges: Option<u64>,
    #[qapi(name = "write-sectors")]
    pub write_sectors: Option<u64>,
    #[qapi(name = "write-ticks")]
    pub write_ticks: Option<u64>,
}

/// QEMU Guest Agent struct `GuestDiskStatsInfo`; see the QEMU Guest Agent Protocol Reference.
#[qapi(condition = "CONFIG_LINUX")]
#[qapi(added = "7.1.0")]
pub struct GuestDiskStatsInfo {
    pub major: u64,
    pub minor: u64,
    pub name: String,
    pub stats: GuestDiskStats,
}

/// QEMU Guest Agent struct `GuestExec`; see the QEMU Guest Agent Protocol Reference.
#[qapi(added = "2.5.0")]
pub struct GuestExec {
    pub pid: i64,
}

/// QEMU Guest Agent alternate `GuestExecCaptureOutput`; see the QEMU Guest Agent Protocol Reference.
#[qapi(added = "8.1.0")]
pub enum GuestExecCaptureOutput {
    #[qapi(name = "flag")]
    Flag(bool),
    #[qapi(name = "mode")]
    Mode(GuestExecCaptureOutputMode),
}

/// QEMU Guest Agent enum `GuestExecCaptureOutputMode`; see the QEMU Guest Agent Protocol Reference.
#[qapi(added = "8.1.0")]
pub enum GuestExecCaptureOutputMode {
    #[qapi(name = "merged")]
    #[qapi(condition = "!CONFIG_WIN32")]
    Merged,
    #[qapi(name = "none")]
    None,
    #[qapi(name = "separated")]
    Separated,
    #[qapi(name = "stderr")]
    Stderr,
    #[qapi(name = "stdout")]
    Stdout,
}

/// QEMU Guest Agent struct `GuestExecStatus`; see the QEMU Guest Agent Protocol Reference.
#[qapi(added = "2.5.0")]
pub struct GuestExecStatus {
    #[qapi(name = "err-data")]
    pub err_data: Option<String>,
    #[qapi(name = "err-truncated")]
    pub err_truncated: Option<bool>,
    pub exitcode: Option<i64>,
    pub exited: bool,
    #[qapi(name = "out-data")]
    pub out_data: Option<String>,
    #[qapi(name = "out-truncated")]
    pub out_truncated: Option<bool>,
    pub signal: Option<i64>,
}

/// QEMU Guest Agent struct `GuestFileRead`; see the QEMU Guest Agent Protocol Reference.
#[qapi(added = "0.15.0")]
pub struct GuestFileRead {
    #[qapi(name = "buf-b64")]
    pub buf_b64: String,
    pub count: i64,
    pub eof: bool,
}

/// QEMU Guest Agent struct `GuestFileSeek`; see the QEMU Guest Agent Protocol Reference.
#[qapi(added = "0.15.0")]
pub struct GuestFileSeek {
    pub eof: bool,
    pub position: i64,
}

/// QEMU Guest Agent struct `GuestFilesystemInfo`; see the QEMU Guest Agent Protocol Reference.
#[qapi(condition = "(CONFIG_WIN32 || CONFIG_LINUX)")]
#[qapi(added = "2.2.0")]
pub struct GuestFilesystemInfo {
    pub disk: Vec<GuestDiskAddress>,
    pub mountpoint: String,
    pub name: String,
    #[qapi(name = "total-bytes")]
    #[qapi(added = "3.0.0")]
    pub total_bytes: Option<u64>,
    #[qapi(name = "total-bytes-privileged")]
    #[qapi(added = "9.1.0")]
    pub total_bytes_privileged: Option<u64>,
    pub r#type: String,
    #[qapi(name = "used-bytes")]
    #[qapi(added = "3.0.0")]
    pub used_bytes: Option<u64>,
}

/// QEMU Guest Agent struct `GuestFilesystemTrimResponse`; see the QEMU Guest Agent Protocol Reference.
#[qapi(condition = "(CONFIG_WIN32 || CONFIG_FSTRIM)")]
#[qapi(added = "2.4.0")]
pub struct GuestFilesystemTrimResponse {
    pub paths: Vec<GuestFilesystemTrimResult>,
}

/// QEMU Guest Agent struct `GuestFilesystemTrimResult`; see the QEMU Guest Agent Protocol Reference.
#[qapi(condition = "(CONFIG_WIN32 || CONFIG_FSTRIM)")]
#[qapi(added = "2.4.0")]
pub struct GuestFilesystemTrimResult {
    pub error: Option<String>,
    pub minimum: Option<i64>,
    pub path: String,
    pub trimmed: Option<i64>,
}

/// QEMU Guest Agent alternate `GuestFileWhence`; see the QEMU Guest Agent Protocol Reference.
#[qapi(added = "2.6.0")]
pub enum GuestFileWhence {
    #[qapi(name = "name")]
    Name(QgaSeek),
    #[qapi(name = "value")]
    Value(i64),
}

/// QEMU Guest Agent struct `GuestFileWrite`; see the QEMU Guest Agent Protocol Reference.
#[qapi(added = "0.15.0")]
pub struct GuestFileWrite {
    pub count: i64,
    pub eof: bool,
}

/// QEMU Guest Agent enum `GuestFsfreezeStatus`; see the QEMU Guest Agent Protocol Reference.
#[qapi(condition = "(CONFIG_WIN32 || CONFIG_FSFREEZE)")]
#[qapi(added = "0.15.0")]
pub enum GuestFsfreezeStatus {
    #[qapi(name = "error")]
    #[qapi(removed = "1.1.0")]
    Error,
    #[qapi(name = "frozen")]
    Frozen,
    #[qapi(name = "thawed")]
    Thawed,
}

/// QEMU Guest Agent struct `GuestHostName`; see the QEMU Guest Agent Protocol Reference.
#[qapi(added = "2.10.0")]
pub struct GuestHostName {
    #[qapi(name = "host-name")]
    pub host_name: String,
}

/// QEMU Guest Agent struct `GuestIpAddress`; see the QEMU Guest Agent Protocol Reference.
#[qapi(condition = "(CONFIG_WIN32 || HAVE_GETIFADDRS)")]
#[qapi(added = "1.1.0")]
pub struct GuestIpAddress {
    #[qapi(name = "ip-address")]
    pub ip_address: String,
    #[qapi(name = "ip-address-type")]
    pub ip_address_type: GuestIpAddressType,
    pub prefix: i64,
}

/// QEMU Guest Agent enum `GuestIpAddressType`; see the QEMU Guest Agent Protocol Reference.
#[qapi(condition = "(CONFIG_WIN32 || HAVE_GETIFADDRS)")]
#[qapi(added = "1.1.0")]
pub enum GuestIpAddressType {
    #[qapi(name = "ipv4")]
    Ipv4,
    #[qapi(name = "ipv6")]
    Ipv6,
}

/// QEMU Guest Agent struct `GuestLinuxCpuStats`; see the QEMU Guest Agent Protocol Reference.
#[qapi(condition = "CONFIG_LINUX")]
#[qapi(added = "7.1.0")]
pub struct GuestLinuxCpuStats {
    pub cpu: i64,
    pub guest: Option<u64>,
    pub guestnice: Option<u64>,
    pub idle: u64,
    pub iowait: Option<u64>,
    pub irq: Option<u64>,
    pub nice: u64,
    pub softirq: Option<u64>,
    pub steal: Option<u64>,
    pub system: u64,
    pub user: u64,
}

/// QEMU Guest Agent struct `GuestLoadAverage`; see the QEMU Guest Agent Protocol Reference.
#[qapi(condition = "(CONFIG_WIN32 || CONFIG_GETLOADAVG)")]
#[qapi(added = "10.0.0")]
pub struct GuestLoadAverage {
    pub load15m: f64,
    pub load1m: f64,
    pub load5m: f64,
}

/// QEMU Guest Agent struct `GuestLogicalProcessor`; see the QEMU Guest Agent Protocol Reference.
#[qapi(condition = "(CONFIG_LINUX || CONFIG_WIN32)")]
#[qapi(added = "1.5.0")]
pub struct GuestLogicalProcessor {
    #[qapi(name = "can-offline")]
    pub can_offline: Option<bool>,
    #[qapi(name = "logical-id")]
    pub logical_id: i64,
    pub online: bool,
}

/// QEMU Guest Agent struct `GuestMemoryBlock`; see the QEMU Guest Agent Protocol Reference.
#[qapi(condition = "CONFIG_LINUX")]
#[qapi(added = "2.3.0")]
pub struct GuestMemoryBlock {
    #[qapi(name = "can-offline")]
    pub can_offline: Option<bool>,
    pub online: bool,
    #[qapi(name = "phys-index")]
    pub phys_index: u64,
}

/// QEMU Guest Agent struct `GuestMemoryBlockInfo`; see the QEMU Guest Agent Protocol Reference.
#[qapi(condition = "CONFIG_LINUX")]
#[qapi(added = "2.3.0")]
pub struct GuestMemoryBlockInfo {
    pub size: u64,
}

/// QEMU Guest Agent struct `GuestMemoryBlockResponse`; see the QEMU Guest Agent Protocol Reference.
#[qapi(condition = "CONFIG_LINUX")]
#[qapi(added = "2.3.0")]
pub struct GuestMemoryBlockResponse {
    #[qapi(name = "error-code")]
    pub error_code: Option<i64>,
    #[qapi(name = "phys-index")]
    pub phys_index: u64,
    pub response: GuestMemoryBlockResponseType,
}

/// QEMU Guest Agent enum `GuestMemoryBlockResponseType`; see the QEMU Guest Agent Protocol Reference.
#[qapi(condition = "CONFIG_LINUX")]
#[qapi(added = "2.3.0")]
pub enum GuestMemoryBlockResponseType {
    #[qapi(name = "not-found")]
    NotFound,
    #[qapi(name = "operation-failed")]
    OperationFailed,
    #[qapi(name = "operation-not-supported")]
    OperationNotSupported,
    #[qapi(name = "success")]
    Success,
}

/// QEMU Guest Agent struct `GuestNetworkInterface`; see the QEMU Guest Agent Protocol Reference.
#[qapi(condition = "(CONFIG_WIN32 || HAVE_GETIFADDRS)")]
#[qapi(added = "1.1.0")]
pub struct GuestNetworkInterface {
    #[qapi(name = "hardware-address")]
    pub hardware_address: Option<String>,
    #[qapi(name = "ip-addresses")]
    pub ip_addresses: Option<Vec<GuestIpAddress>>,
    pub name: String,
    #[qapi(added = "2.11.0")]
    pub statistics: Option<GuestNetworkInterfaceStat>,
}

/// QEMU Guest Agent struct `GuestNetworkInterfaceStat`; see the QEMU Guest Agent Protocol Reference.
#[qapi(condition = "(CONFIG_WIN32 || HAVE_GETIFADDRS)")]
#[qapi(added = "2.11.0")]
pub struct GuestNetworkInterfaceStat {
    #[qapi(name = "rx-bytes")]
    pub rx_bytes: u64,
    #[qapi(name = "rx-dropped")]
    pub rx_dropped: u64,
    #[qapi(name = "rx-errs")]
    pub rx_errs: u64,
    #[qapi(name = "rx-packets")]
    pub rx_packets: u64,
    #[qapi(name = "tx-bytes")]
    pub tx_bytes: u64,
    #[qapi(name = "tx-dropped")]
    pub tx_dropped: u64,
    #[qapi(name = "tx-errs")]
    pub tx_errs: u64,
    #[qapi(name = "tx-packets")]
    pub tx_packets: u64,
}

/// QEMU Guest Agent struct `GuestNetworkRoute`; see the QEMU Guest Agent Protocol Reference.
#[qapi(condition = "(CONFIG_LINUX || CONFIG_WIN32)")]
#[qapi(added = "9.1.0")]
pub struct GuestNetworkRoute {
    pub desprefixlen: Option<String>,
    pub destination: String,
    pub flags: Option<u64>,
    pub gateway: Option<String>,
    pub iface: String,
    pub irtt: Option<i64>,
    pub mask: Option<String>,
    pub metric: i64,
    pub mtu: Option<i64>,
    pub nexthop: Option<String>,
    pub refcnt: Option<i64>,
    pub source: Option<String>,
    pub srcprefixlen: Option<String>,
    pub r#use: Option<i64>,
    pub version: i64,
    pub window: Option<i64>,
}

/// QEMU Guest Agent struct `GuestNVMeSmart`; see the QEMU Guest Agent Protocol Reference.
#[qapi(name = "GuestNVMeSmart")]
#[qapi(condition = "(CONFIG_WIN32 || CONFIG_LIBUDEV)")]
#[qapi(added = "7.1.0")]
pub struct GuestNvMeSmart {
    #[qapi(name = "available-spare")]
    pub available_spare: i64,
    #[qapi(name = "available-spare-threshold")]
    pub available_spare_threshold: i64,
    #[qapi(name = "controller-busy-time-hi")]
    pub controller_busy_time_hi: u64,
    #[qapi(name = "controller-busy-time-lo")]
    pub controller_busy_time_lo: u64,
    #[qapi(name = "critical-warning")]
    pub critical_warning: i64,
    #[qapi(name = "data-units-read-hi")]
    pub data_units_read_hi: u64,
    #[qapi(name = "data-units-read-lo")]
    pub data_units_read_lo: u64,
    #[qapi(name = "data-units-written-hi")]
    pub data_units_written_hi: u64,
    #[qapi(name = "data-units-written-lo")]
    pub data_units_written_lo: u64,
    #[qapi(name = "host-read-commands-hi")]
    pub host_read_commands_hi: u64,
    #[qapi(name = "host-read-commands-lo")]
    pub host_read_commands_lo: u64,
    #[qapi(name = "host-write-commands-hi")]
    pub host_write_commands_hi: u64,
    #[qapi(name = "host-write-commands-lo")]
    pub host_write_commands_lo: u64,
    #[qapi(name = "media-errors-hi")]
    pub media_errors_hi: u64,
    #[qapi(name = "media-errors-lo")]
    pub media_errors_lo: u64,
    #[qapi(name = "number-of-error-log-entries-hi")]
    pub number_of_error_log_entries_hi: u64,
    #[qapi(name = "number-of-error-log-entries-lo")]
    pub number_of_error_log_entries_lo: u64,
    #[qapi(name = "percentage-used")]
    pub percentage_used: i64,
    #[qapi(name = "power-cycles-hi")]
    pub power_cycles_hi: u64,
    #[qapi(name = "power-cycles-lo")]
    pub power_cycles_lo: u64,
    #[qapi(name = "power-on-hours-hi")]
    pub power_on_hours_hi: u64,
    #[qapi(name = "power-on-hours-lo")]
    pub power_on_hours_lo: u64,
    pub temperature: i64,
    #[qapi(name = "unsafe-shutdowns-hi")]
    pub unsafe_shutdowns_hi: u64,
    #[qapi(name = "unsafe-shutdowns-lo")]
    pub unsafe_shutdowns_lo: u64,
}

/// QEMU Guest Agent struct `GuestOSInfo`; see the QEMU Guest Agent Protocol Reference.
#[qapi(name = "GuestOSInfo")]
#[qapi(added = "2.10.0")]
pub struct GuestOsInfo {
    pub id: Option<String>,
    #[qapi(name = "kernel-release")]
    pub kernel_release: Option<String>,
    #[qapi(name = "kernel-version")]
    pub kernel_version: Option<String>,
    pub machine: Option<String>,
    pub name: Option<String>,
    #[qapi(name = "pretty-name")]
    pub pretty_name: Option<String>,
    pub variant: Option<String>,
    #[qapi(name = "variant-id")]
    pub variant_id: Option<String>,
    pub version: Option<String>,
    #[qapi(name = "version-id")]
    pub version_id: Option<String>,
}

/// QEMU Guest Agent struct `GuestPCIAddress`; see the QEMU Guest Agent Protocol Reference.
#[qapi(name = "GuestPCIAddress")]
#[qapi(condition = "(CONFIG_WIN32 || CONFIG_LINUX)")]
#[qapi(added = "2.2.0")]
pub struct GuestPciAddress {
    pub bus: i64,
    pub domain: i64,
    pub function: i64,
    pub slot: i64,
}

/// QEMU Guest Agent struct `GuestTimezone`; see the QEMU Guest Agent Protocol Reference.
#[qapi(added = "2.10.0")]
pub struct GuestTimezone {
    pub offset: i64,
    pub zone: Option<String>,
}

/// QEMU Guest Agent struct `GuestUser`; see the QEMU Guest Agent Protocol Reference.
#[qapi(condition = "(CONFIG_WIN32 || HAVE_UTMPX)")]
#[qapi(added = "2.10.0")]
pub struct GuestUser {
    pub domain: Option<String>,
    #[qapi(name = "login-time")]
    pub login_time: f64,
    pub user: String,
}

/// QEMU Guest Agent enum `QGASeek`; see the QEMU Guest Agent Protocol Reference.
#[qapi(name = "QGASeek")]
#[qapi(added = "2.6.0")]
pub enum QgaSeek {
    #[qapi(name = "cur")]
    Cur,
    #[qapi(name = "end")]
    End,
    #[qapi(name = "set")]
    Set,
}

pub mod commands;
pub mod events;
pub mod metadata;
