use qapi_macros::qapi;
use super::*;
/// QEMU Guest Agent command `guest-exec`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(returns = "GuestExec")]
#[qapi(name = "guest-exec")]
#[qapi(added = "2.5.0")]
pub struct GuestExecCommand {
    pub arg: Option<Vec<String>>,
    #[qapi(name = "capture-output")]
    #[qapi(added = "8.1.0")]
    pub capture_output: Option<GuestExecCaptureOutput>,
    #[qapi(name = "capture-output")]
    #[qapi(removed = "8.1.0")]
    pub capture_output_legacy: Option<bool>,
    pub env: Option<Vec<String>>,
    #[qapi(name = "input-data")]
    pub input_data: Option<String>,
    pub path: String,
}

/// QEMU Guest Agent command `guest-exec-status`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(returns = "GuestExecStatus")]
#[qapi(name = "guest-exec-status")]
#[qapi(added = "2.5.0")]
pub struct GuestExecStatusCommand {
    pub pid: i64,
}

/// QEMU Guest Agent command `guest-file-close`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-file-close")]
#[qapi(returns = "()")]
#[qapi(added = "0.15.0")]
pub struct GuestFileClose {
    pub handle: i64,
}

/// QEMU Guest Agent command `guest-file-flush`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-file-flush")]
#[qapi(returns = "()")]
#[qapi(added = "0.15.0")]
pub struct GuestFileFlush {
    pub handle: i64,
}

/// QEMU Guest Agent command `guest-file-open`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-file-open")]
#[qapi(returns = "i64")]
#[qapi(added = "0.15.0")]
pub struct GuestFileOpen {
    pub mode: Option<String>,
    pub path: String,
}

/// QEMU Guest Agent command `guest-file-read`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(returns = "GuestFileRead")]
#[qapi(name = "guest-file-read")]
#[qapi(added = "0.15.0")]
pub struct GuestFileReadCommand {
    pub count: Option<i64>,
    pub handle: i64,
}

/// QEMU Guest Agent command `guest-file-seek`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(returns = "GuestFileSeek")]
#[qapi(name = "guest-file-seek")]
#[qapi(added = "0.15.0")]
pub struct GuestFileSeekCommand {
    pub handle: i64,
    pub offset: i64,
    #[qapi(added = "2.6.0")]
    #[qapi(required)]
    pub whence: Option<GuestFileWhence>,
    #[qapi(name = "whence")]
    #[qapi(removed = "2.6.0")]
    #[qapi(required)]
    pub whence_legacy: Option<i64>,
}

/// QEMU Guest Agent command `guest-file-write`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(returns = "GuestFileWrite")]
#[qapi(name = "guest-file-write")]
#[qapi(added = "0.15.0")]
pub struct GuestFileWriteCommand {
    #[qapi(name = "buf-b64")]
    pub buf_b64: String,
    pub count: Option<i64>,
    pub handle: i64,
}

/// QEMU Guest Agent command `guest-fsfreeze-freeze`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-fsfreeze-freeze")]
#[qapi(returns = "i64")]
#[qapi(condition = "(CONFIG_WIN32 || CONFIG_FSFREEZE)")]
#[qapi(added = "0.15.0")]
pub struct GuestFsfreezeFreeze {}

/// QEMU Guest Agent command `guest-fsfreeze-freeze-list`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-fsfreeze-freeze-list")]
#[qapi(returns = "i64")]
#[qapi(condition = "(CONFIG_WIN32 || CONFIG_FSFREEZE)")]
#[qapi(added = "2.2.0")]
pub struct GuestFsfreezeFreezeList {
    pub mountpoints: Option<Vec<String>>,
}

/// QEMU Guest Agent command `guest-fsfreeze-status`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(returns = "GuestFsfreezeStatus")]
#[qapi(name = "guest-fsfreeze-status")]
#[qapi(condition = "(CONFIG_WIN32 || CONFIG_FSFREEZE)")]
#[qapi(added = "0.15.0")]
pub struct GuestFsfreezeStatusCommand {}

/// QEMU Guest Agent command `guest-fsfreeze-thaw`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-fsfreeze-thaw")]
#[qapi(returns = "i64")]
#[qapi(condition = "(CONFIG_WIN32 || CONFIG_FSFREEZE)")]
#[qapi(added = "0.15.0")]
pub struct GuestFsfreezeThaw {}

/// QEMU Guest Agent command `guest-fstrim`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-fstrim")]
#[qapi(returns = "GuestFilesystemTrimResponse")]
#[qapi(condition = "(CONFIG_WIN32 || CONFIG_FSTRIM)")]
#[qapi(added = "1.2.0")]
#[qapi(returns(added = "1.2.0", value = "()"))]
#[qapi(returns(added = "2.4.0", value = "GuestFilesystemTrimResponse"))]
pub struct GuestFstrim {
    pub minimum: Option<i64>,
}

/// QEMU Guest Agent command `guest-get-cpustats`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-get-cpustats")]
#[qapi(returns = "Vec<GuestCpuStats>")]
#[qapi(condition = "CONFIG_LINUX")]
#[qapi(added = "7.1.0")]
pub struct GuestGetCpustats {}

/// QEMU Guest Agent command `guest-get-devices`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-get-devices")]
#[qapi(returns = "Vec<GuestDeviceInfo>")]
#[qapi(condition = "CONFIG_WIN32")]
#[qapi(added = "5.2.0")]
pub struct GuestGetDevices {}

/// QEMU Guest Agent command `guest-get-disks`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-get-disks")]
#[qapi(returns = "Vec<GuestDiskInfo>")]
#[qapi(condition = "(CONFIG_WIN32 || CONFIG_LIBUDEV)")]
#[qapi(added = "5.2.0")]
pub struct GuestGetDisks {}

/// QEMU Guest Agent command `guest-get-diskstats`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-get-diskstats")]
#[qapi(returns = "Vec<GuestDiskStatsInfo>")]
#[qapi(condition = "CONFIG_LINUX")]
#[qapi(added = "7.1.0")]
pub struct GuestGetDiskstats {}

/// QEMU Guest Agent command `guest-get-fsinfo`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-get-fsinfo")]
#[qapi(returns = "Vec<GuestFilesystemInfo>")]
#[qapi(condition = "(CONFIG_WIN32 || CONFIG_LINUX)")]
#[qapi(added = "2.2.0")]
pub struct GuestGetFsinfo {}

/// QEMU Guest Agent command `guest-get-host-name`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-get-host-name")]
#[qapi(returns = "GuestHostName")]
#[qapi(added = "2.10.0")]
pub struct GuestGetHostName {}

/// QEMU Guest Agent command `guest-get-load`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-get-load")]
#[qapi(returns = "GuestLoadAverage")]
#[qapi(condition = "(CONFIG_WIN32 || CONFIG_GETLOADAVG)")]
#[qapi(added = "10.0.0")]
pub struct GuestGetLoad {}

/// QEMU Guest Agent command `guest-get-memory-block-info`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-get-memory-block-info")]
#[qapi(returns = "GuestMemoryBlockInfo")]
#[qapi(condition = "CONFIG_LINUX")]
#[qapi(added = "2.3.0")]
pub struct GuestGetMemoryBlockInfo {}

/// QEMU Guest Agent command `guest-get-memory-blocks`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-get-memory-blocks")]
#[qapi(returns = "Vec<GuestMemoryBlock>")]
#[qapi(condition = "CONFIG_LINUX")]
#[qapi(added = "2.3.0")]
pub struct GuestGetMemoryBlocks {}

/// QEMU Guest Agent command `guest-get-osinfo`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-get-osinfo")]
#[qapi(returns = "GuestOsInfo")]
#[qapi(added = "2.10.0")]
pub struct GuestGetOsinfo {}

/// QEMU Guest Agent command `guest-get-time`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-get-time")]
#[qapi(returns = "i64")]
#[qapi(added = "1.5.0")]
pub struct GuestGetTime {}

/// QEMU Guest Agent command `guest-get-timezone`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-get-timezone")]
#[qapi(returns = "GuestTimezone")]
#[qapi(added = "2.10.0")]
pub struct GuestGetTimezone {}

/// QEMU Guest Agent command `guest-get-users`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-get-users")]
#[qapi(returns = "Vec<GuestUser>")]
#[qapi(condition = "(CONFIG_WIN32 || HAVE_UTMPX)")]
#[qapi(added = "2.10.0")]
pub struct GuestGetUsers {}

/// QEMU Guest Agent command `guest-get-vcpus`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-get-vcpus")]
#[qapi(returns = "Vec<GuestLogicalProcessor>")]
#[qapi(condition = "(CONFIG_LINUX || CONFIG_WIN32)")]
#[qapi(added = "1.5.0")]
pub struct GuestGetVcpus {}

/// QEMU Guest Agent command `guest-info`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-info")]
#[qapi(returns = "GuestAgentInfo")]
#[qapi(added = "0.15.0")]
pub struct GuestInfo {}

/// QEMU Guest Agent command `guest-network-get-interfaces`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-network-get-interfaces")]
#[qapi(returns = "Vec<GuestNetworkInterface>")]
#[qapi(condition = "(CONFIG_WIN32 || HAVE_GETIFADDRS)")]
#[qapi(added = "1.1.0")]
pub struct GuestNetworkGetInterfaces {}

/// QEMU Guest Agent command `guest-network-get-route`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-network-get-route")]
#[qapi(returns = "Vec<GuestNetworkRoute>")]
#[qapi(condition = "(CONFIG_LINUX || CONFIG_WIN32)")]
#[qapi(added = "9.1.0")]
pub struct GuestNetworkGetRoute {}

/// QEMU Guest Agent command `guest-ping`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-ping")]
#[qapi(returns = "()")]
#[qapi(added = "0.15.0")]
pub struct GuestPing {}

/// QEMU Guest Agent command `guest-set-memory-blocks`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-set-memory-blocks")]
#[qapi(returns = "Vec<GuestMemoryBlockResponse>")]
#[qapi(condition = "CONFIG_LINUX")]
#[qapi(added = "2.3.0")]
pub struct GuestSetMemoryBlocks {
    #[qapi(name = "mem-blks")]
    pub mem_blks: Vec<GuestMemoryBlock>,
}

/// QEMU Guest Agent command `guest-set-time`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-set-time")]
#[qapi(returns = "()")]
#[qapi(added = "1.5.0")]
pub struct GuestSetTime {
    #[qapi(added = "2.0.0")]
    pub time: Option<i64>,
    #[qapi(name = "time")]
    #[qapi(removed = "2.0.0")]
    #[qapi(required)]
    pub time_legacy: Option<i64>,
}

/// QEMU Guest Agent command `guest-set-user-password`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-set-user-password")]
#[qapi(returns = "()")]
#[qapi(condition = "(CONFIG_WIN32 || CONFIG_LINUX || CONFIG_FREEBSD)")]
#[qapi(added = "2.3.0")]
pub struct GuestSetUserPassword {
    pub crypted: bool,
    pub password: String,
    pub username: String,
}

/// QEMU Guest Agent command `guest-set-vcpus`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-set-vcpus")]
#[qapi(returns = "i64")]
#[qapi(condition = "CONFIG_LINUX")]
#[qapi(added = "1.5.0")]
pub struct GuestSetVcpus {
    pub vcpus: Vec<GuestLogicalProcessor>,
}

/// QEMU Guest Agent command `guest-shutdown`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-shutdown")]
#[qapi(returns = "()")]
#[qapi(added = "0.15.0")]
#[qapi(no_success_response(added = "1.1.0"))]
pub struct GuestShutdown {
    pub mode: Option<String>,
}

/// QEMU Guest Agent command `guest-ssh-add-authorized-keys`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-ssh-add-authorized-keys")]
#[qapi(returns = "()")]
#[qapi(added = "5.2.0")]
pub struct GuestSshAddAuthorizedKeys {
    pub keys: Vec<String>,
    pub reset: Option<bool>,
    pub username: String,
}

/// QEMU Guest Agent command `guest-ssh-get-authorized-keys`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-ssh-get-authorized-keys")]
#[qapi(returns = "GuestAuthorizedKeys")]
#[qapi(added = "5.2.0")]
pub struct GuestSshGetAuthorizedKeys {
    pub username: String,
}

/// QEMU Guest Agent command `guest-ssh-remove-authorized-keys`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-ssh-remove-authorized-keys")]
#[qapi(returns = "()")]
#[qapi(added = "5.2.0")]
pub struct GuestSshRemoveAuthorizedKeys {
    pub keys: Vec<String>,
    pub username: String,
}

/// QEMU Guest Agent command `guest-suspend-disk`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-suspend-disk")]
#[qapi(returns = "()")]
#[qapi(condition = "(CONFIG_LINUX || CONFIG_WIN32)")]
#[qapi(added = "1.1.0")]
#[qapi(no_success_response)]
pub struct GuestSuspendDisk {}

/// QEMU Guest Agent command `guest-suspend-hybrid`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-suspend-hybrid")]
#[qapi(returns = "()")]
#[qapi(condition = "CONFIG_LINUX")]
#[qapi(added = "1.1.0")]
#[qapi(no_success_response)]
pub struct GuestSuspendHybrid {}

/// QEMU Guest Agent command `guest-suspend-ram`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-suspend-ram")]
#[qapi(returns = "()")]
#[qapi(condition = "(CONFIG_LINUX || CONFIG_WIN32)")]
#[qapi(added = "1.1.0")]
#[qapi(no_success_response)]
pub struct GuestSuspendRam {}

/// QEMU Guest Agent command `guest-sync`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-sync")]
#[qapi(returns = "i64")]
#[qapi(added = "0.15.0")]
pub struct GuestSync {
    pub id: i64,
}

/// QEMU Guest Agent command `guest-sync-delimited`; see the QEMU Guest Agent Protocol Reference.
#[qapi(command)]
#[qapi(name = "guest-sync-delimited")]
#[qapi(returns = "i64")]
#[qapi(added = "1.1.0")]
pub struct GuestSyncDelimited {
    pub id: i64,
}
