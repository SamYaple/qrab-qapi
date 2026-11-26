use qapi_macros::qapi;
use self::commands::*;
/// QMP struct `Abort`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.6.0")]
pub struct Abort {}

/// QMP struct `AbortWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct AbortWrapper {
    pub data: Abort,
}

/// QMP enum `Accelerator`; see the QEMU QMP Reference Manual.
#[qapi(added = "10.2.0")]
pub enum Accelerator {
    #[qapi(name = "hvf")]
    Hvf,
    #[qapi(name = "kvm")]
    Kvm,
    #[qapi(name = "mshv")]
    Mshv,
    #[qapi(name = "nvmm")]
    Nvmm,
    #[qapi(name = "qtest")]
    Qtest,
    #[qapi(name = "tcg")]
    Tcg,
    #[qapi(name = "whpx")]
    Whpx,
    #[qapi(name = "xen")]
    Xen,
}

/// QMP struct `AcceleratorInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "10.2.0")]
pub struct AcceleratorInfo {
    pub enabled: Accelerator,
    pub present: Vec<Accelerator>,
}

/// QMP struct `AcpiGenericInitiatorProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "9.0.0")]
pub struct AcpiGenericInitiatorProperties {
    pub node: u32,
    #[qapi(name = "pci-dev")]
    pub pci_dev: String,
}

/// QMP struct `AcpiGenericPortProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "9.2.0")]
pub struct AcpiGenericPortProperties {
    pub node: u32,
    #[qapi(name = "pci-bus")]
    pub pci_bus: String,
}

/// QMP struct `ACPIOSTInfo`; see the QEMU QMP Reference Manual.
#[qapi(name = "ACPIOSTInfo")]
#[qapi(added = "2.1.0")]
pub struct AcpiostInfo {
    pub device: Option<String>,
    pub slot: String,
    #[qapi(name = "slot-type")]
    pub slot_type: AcpiSlotType,
    pub source: i64,
    pub status: i64,
}

/// QMP enum `ACPISlotType`; see the QEMU QMP Reference Manual.
#[qapi(name = "ACPISlotType")]
#[qapi(added = "2.1.0")]
pub enum AcpiSlotType {
    #[qapi(name = "CPU")]
    #[qapi(added = "2.7.0")]
    Cpu,
    #[qapi(name = "DIMM")]
    Dimm,
}

/// QMP struct `AcpiTableOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.5.0")]
pub struct AcpiTableOptions {
    pub asl_compiler_id: Option<String>,
    pub asl_compiler_rev: Option<u32>,
    pub data: Option<String>,
    pub file: Option<String>,
    pub oem_id: Option<String>,
    pub oem_rev: Option<u32>,
    pub oem_table_id: Option<String>,
    pub rev: Option<u8>,
    pub sig: Option<String>,
}

/// QMP enum `ActionCompletionMode`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.5.0")]
pub enum ActionCompletionMode {
    #[qapi(name = "grouped")]
    Grouped,
    #[qapi(name = "individual")]
    Individual,
}

/// QMP struct `AddfdInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.2.0")]
pub struct AddfdInfo {
    pub fd: i64,
    #[qapi(name = "fdset-id")]
    pub fdset_id: i64,
}

/// QMP enum `AFXDPMode`; see the QEMU QMP Reference Manual.
#[qapi(name = "AFXDPMode")]
#[qapi(condition = "CONFIG_AF_XDP")]
#[qapi(added = "8.2.0")]
pub enum AfxdpMode {
    #[qapi(name = "native")]
    Native,
    #[qapi(name = "skb")]
    Skb,
}

/// QMP struct `AnnounceParameters`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.0.0")]
pub struct AnnounceParameters {
    #[qapi(added = "4.1.0")]
    pub id: Option<String>,
    pub initial: i64,
    #[qapi(added = "4.1.0")]
    pub interfaces: Option<Vec<String>>,
    pub max: i64,
    pub rounds: i64,
    pub step: i64,
}

/// QMP union `Audiodev`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.0.0")]
pub struct Audiodev {
    #[qapi(discriminator)]
    pub driver: AudiodevDriver,
    pub id: String,
    #[qapi(name = "timer-period")]
    pub timer_period: Option<u32>,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<AudiodevBranch>,
}

/// QMP struct `AudiodevAlsaOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.0.0")]
pub struct AudiodevAlsaOptions {
    pub r#in: Option<AudiodevAlsaPerDirectionOptions>,
    pub out: Option<AudiodevAlsaPerDirectionOptions>,
    pub threshold: Option<u32>,
}

/// QMP struct `AudiodevAlsaPerDirectionOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.0.0")]
pub struct AudiodevAlsaPerDirectionOptions {
    #[qapi(name = "buffer-length")]
    pub buffer_length: Option<u32>,
    pub channels: Option<u32>,
    pub dev: Option<String>,
    #[qapi(name = "fixed-settings")]
    pub fixed_settings: Option<bool>,
    pub format: Option<AudioFormat>,
    pub frequency: Option<u32>,
    #[qapi(name = "mixing-engine")]
    #[qapi(added = "4.2.0")]
    pub mixing_engine: Option<bool>,
    #[qapi(name = "period-length")]
    pub period_length: Option<u32>,
    #[qapi(name = "try-poll")]
    pub try_poll: Option<bool>,
    pub voices: Option<u32>,
}

#[qapi(union_branch)]
#[qapi(added = "4.0.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum AudiodevBranch {
    #[qapi(name = "alsa")]
    #[qapi(condition = "CONFIG_AUDIO_ALSA")]
    Alsa(AudiodevAlsaOptions),
    #[qapi(name = "coreaudio")]
    #[qapi(condition = "CONFIG_AUDIO_COREAUDIO")]
    Coreaudio(AudiodevCoreaudioOptions),
    #[qapi(name = "dbus")]
    #[qapi(condition = "CONFIG_DBUS_DISPLAY")]
    #[qapi(added = "10.0.0")]
    Dbus(AudiodevDBusOptions),
    #[qapi(condition = "CONFIG_DBUS_DISPLAY")]
    #[qapi(name = "dbus")]
    #[qapi(added = "7.0.0")]
    #[qapi(removed = "10.0.0")]
    DbusLegacy(AudiodevGenericOptions),
    #[qapi(name = "dsound")]
    #[qapi(condition = "CONFIG_AUDIO_DSOUND")]
    Dsound(AudiodevDsoundOptions),
    #[qapi(name = "jack")]
    #[qapi(condition = "CONFIG_AUDIO_JACK")]
    #[qapi(added = "5.1.0")]
    Jack(AudiodevJackOptions),
    #[qapi(name = "none")]
    None(AudiodevGenericOptions),
    #[qapi(name = "oss")]
    #[qapi(condition = "CONFIG_AUDIO_OSS")]
    Oss(AudiodevOssOptions),
    #[qapi(name = "pa")]
    #[qapi(condition = "CONFIG_AUDIO_PA")]
    Pa(AudiodevPaOptions),
    #[qapi(name = "pipewire")]
    #[qapi(condition = "CONFIG_AUDIO_PIPEWIRE")]
    #[qapi(added = "8.1.0")]
    Pipewire(AudiodevPipewireOptions),
    #[qapi(name = "sdl")]
    #[qapi(condition = "CONFIG_AUDIO_SDL")]
    #[qapi(added = "6.0.0")]
    Sdl(AudiodevSdlOptions),
    #[qapi(name = "sdl")]
    #[qapi(removed = "6.0.0")]
    SdlLegacy(AudiodevGenericOptions),
    #[qapi(name = "sndio")]
    #[qapi(condition = "CONFIG_AUDIO_SNDIO")]
    #[qapi(added = "7.2.0")]
    Sndio(AudiodevSndioOptions),
    #[qapi(name = "spice")]
    #[qapi(condition = "CONFIG_SPICE")]
    Spice(AudiodevGenericOptions),
    #[qapi(name = "wav")]
    Wav(AudiodevWavOptions),
}

/// QMP struct `AudiodevCoreaudioOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.0.0")]
pub struct AudiodevCoreaudioOptions {
    pub r#in: Option<AudiodevCoreaudioPerDirectionOptions>,
    pub out: Option<AudiodevCoreaudioPerDirectionOptions>,
}

/// QMP struct `AudiodevCoreaudioPerDirectionOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.0.0")]
pub struct AudiodevCoreaudioPerDirectionOptions {
    #[qapi(name = "buffer-count")]
    pub buffer_count: Option<u32>,
    #[qapi(name = "buffer-length")]
    pub buffer_length: Option<u32>,
    pub channels: Option<u32>,
    #[qapi(name = "fixed-settings")]
    pub fixed_settings: Option<bool>,
    pub format: Option<AudioFormat>,
    pub frequency: Option<u32>,
    #[qapi(name = "mixing-engine")]
    #[qapi(added = "4.2.0")]
    pub mixing_engine: Option<bool>,
    pub voices: Option<u32>,
}

/// QMP struct `AudiodevDBusOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "10.0.0")]
pub struct AudiodevDBusOptions {
    pub r#in: Option<AudiodevPerDirectionOptions>,
    pub nsamples: Option<u32>,
    pub out: Option<AudiodevPerDirectionOptions>,
}

/// QMP enum `AudiodevDriver`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.0.0")]
pub enum AudiodevDriver {
    #[qapi(name = "alsa")]
    #[qapi(condition = "CONFIG_AUDIO_ALSA")]
    Alsa,
    #[qapi(name = "coreaudio")]
    #[qapi(condition = "CONFIG_AUDIO_COREAUDIO")]
    Coreaudio,
    #[qapi(name = "dbus")]
    #[qapi(condition = "CONFIG_DBUS_DISPLAY")]
    #[qapi(added = "7.0.0")]
    Dbus,
    #[qapi(name = "dsound")]
    #[qapi(condition = "CONFIG_AUDIO_DSOUND")]
    Dsound,
    #[qapi(name = "jack")]
    #[qapi(condition = "CONFIG_AUDIO_JACK")]
    #[qapi(added = "5.1.0")]
    Jack,
    #[qapi(name = "none")]
    None,
    #[qapi(name = "oss")]
    #[qapi(condition = "CONFIG_AUDIO_OSS")]
    Oss,
    #[qapi(name = "pa")]
    #[qapi(condition = "CONFIG_AUDIO_PA")]
    Pa,
    #[qapi(name = "pipewire")]
    #[qapi(condition = "CONFIG_AUDIO_PIPEWIRE")]
    #[qapi(added = "8.1.0")]
    Pipewire,
    #[qapi(name = "sdl")]
    #[qapi(condition = "CONFIG_AUDIO_SDL")]
    Sdl,
    #[qapi(name = "sndio")]
    #[qapi(condition = "CONFIG_AUDIO_SNDIO")]
    #[qapi(added = "7.2.0")]
    Sndio,
    #[qapi(name = "spice")]
    #[qapi(condition = "CONFIG_SPICE")]
    Spice,
    #[qapi(name = "wav")]
    Wav,
}

/// QMP struct `AudiodevDsoundOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.0.0")]
pub struct AudiodevDsoundOptions {
    pub r#in: Option<AudiodevPerDirectionOptions>,
    pub latency: Option<u32>,
    pub out: Option<AudiodevPerDirectionOptions>,
}

/// QMP struct `AudiodevGenericOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.0.0")]
pub struct AudiodevGenericOptions {
    pub r#in: Option<AudiodevPerDirectionOptions>,
    pub out: Option<AudiodevPerDirectionOptions>,
}

/// QMP struct `AudiodevJackOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "5.1.0")]
pub struct AudiodevJackOptions {
    pub r#in: Option<AudiodevJackPerDirectionOptions>,
    pub out: Option<AudiodevJackPerDirectionOptions>,
}

/// QMP struct `AudiodevJackPerDirectionOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "5.1.0")]
pub struct AudiodevJackPerDirectionOptions {
    #[qapi(name = "buffer-length")]
    pub buffer_length: Option<u32>,
    pub channels: Option<u32>,
    #[qapi(name = "client-name")]
    pub client_name: Option<String>,
    #[qapi(name = "connect-ports")]
    pub connect_ports: Option<String>,
    #[qapi(name = "exact-name")]
    pub exact_name: Option<bool>,
    #[qapi(name = "fixed-settings")]
    pub fixed_settings: Option<bool>,
    pub format: Option<AudioFormat>,
    pub frequency: Option<u32>,
    #[qapi(name = "mixing-engine")]
    pub mixing_engine: Option<bool>,
    #[qapi(name = "server-name")]
    pub server_name: Option<String>,
    #[qapi(name = "start-server")]
    pub start_server: Option<bool>,
    pub voices: Option<u32>,
}

/// QMP struct `AudiodevOssOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.0.0")]
pub struct AudiodevOssOptions {
    #[qapi(name = "dsp-policy")]
    pub dsp_policy: Option<u32>,
    pub exclusive: Option<bool>,
    pub r#in: Option<AudiodevOssPerDirectionOptions>,
    pub out: Option<AudiodevOssPerDirectionOptions>,
    #[qapi(name = "try-mmap")]
    pub try_mmap: Option<bool>,
}

/// QMP struct `AudiodevOssPerDirectionOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.0.0")]
pub struct AudiodevOssPerDirectionOptions {
    #[qapi(name = "buffer-count")]
    pub buffer_count: Option<u32>,
    #[qapi(name = "buffer-length")]
    pub buffer_length: Option<u32>,
    pub channels: Option<u32>,
    pub dev: Option<String>,
    #[qapi(name = "fixed-settings")]
    pub fixed_settings: Option<bool>,
    pub format: Option<AudioFormat>,
    pub frequency: Option<u32>,
    #[qapi(name = "mixing-engine")]
    #[qapi(added = "4.2.0")]
    pub mixing_engine: Option<bool>,
    #[qapi(name = "try-poll")]
    pub try_poll: Option<bool>,
    pub voices: Option<u32>,
}

/// QMP struct `AudiodevPaOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.0.0")]
pub struct AudiodevPaOptions {
    pub r#in: Option<AudiodevPaPerDirectionOptions>,
    pub out: Option<AudiodevPaPerDirectionOptions>,
    pub server: Option<String>,
}

/// QMP struct `AudiodevPaPerDirectionOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.0.0")]
pub struct AudiodevPaPerDirectionOptions {
    #[qapi(name = "buffer-length")]
    pub buffer_length: Option<u32>,
    pub channels: Option<u32>,
    #[qapi(name = "fixed-settings")]
    pub fixed_settings: Option<bool>,
    pub format: Option<AudioFormat>,
    pub frequency: Option<u32>,
    pub latency: Option<u32>,
    #[qapi(name = "mixing-engine")]
    #[qapi(added = "4.2.0")]
    pub mixing_engine: Option<bool>,
    pub name: Option<String>,
    #[qapi(name = "stream-name")]
    #[qapi(added = "4.2.0")]
    pub stream_name: Option<String>,
    pub voices: Option<u32>,
}

/// QMP struct `AudiodevPerDirectionOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.0.0")]
pub struct AudiodevPerDirectionOptions {
    #[qapi(name = "buffer-length")]
    pub buffer_length: Option<u32>,
    pub channels: Option<u32>,
    #[qapi(name = "fixed-settings")]
    pub fixed_settings: Option<bool>,
    pub format: Option<AudioFormat>,
    pub frequency: Option<u32>,
    #[qapi(name = "mixing-engine")]
    #[qapi(added = "4.2.0")]
    pub mixing_engine: Option<bool>,
    pub voices: Option<u32>,
}

/// QMP struct `AudiodevPipewireOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.1.0")]
pub struct AudiodevPipewireOptions {
    pub r#in: Option<AudiodevPipewirePerDirectionOptions>,
    pub out: Option<AudiodevPipewirePerDirectionOptions>,
}

/// QMP struct `AudiodevPipewirePerDirectionOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.1.0")]
pub struct AudiodevPipewirePerDirectionOptions {
    #[qapi(name = "buffer-length")]
    pub buffer_length: Option<u32>,
    pub channels: Option<u32>,
    #[qapi(name = "fixed-settings")]
    pub fixed_settings: Option<bool>,
    pub format: Option<AudioFormat>,
    pub frequency: Option<u32>,
    pub latency: Option<u32>,
    #[qapi(name = "mixing-engine")]
    pub mixing_engine: Option<bool>,
    pub name: Option<String>,
    #[qapi(name = "stream-name")]
    pub stream_name: Option<String>,
    pub voices: Option<u32>,
}

/// QMP struct `AudiodevSdlOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct AudiodevSdlOptions {
    pub r#in: Option<AudiodevSdlPerDirectionOptions>,
    pub out: Option<AudiodevSdlPerDirectionOptions>,
}

/// QMP struct `AudiodevSdlPerDirectionOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct AudiodevSdlPerDirectionOptions {
    #[qapi(name = "buffer-count")]
    pub buffer_count: Option<u32>,
    #[qapi(name = "buffer-length")]
    pub buffer_length: Option<u32>,
    pub channels: Option<u32>,
    #[qapi(name = "fixed-settings")]
    pub fixed_settings: Option<bool>,
    pub format: Option<AudioFormat>,
    pub frequency: Option<u32>,
    #[qapi(name = "mixing-engine")]
    pub mixing_engine: Option<bool>,
    pub voices: Option<u32>,
}

/// QMP struct `AudiodevSndioOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.2.0")]
pub struct AudiodevSndioOptions {
    pub dev: Option<String>,
    pub r#in: Option<AudiodevPerDirectionOptions>,
    pub latency: Option<u32>,
    pub out: Option<AudiodevPerDirectionOptions>,
}

/// QMP struct `AudiodevWavOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.0.0")]
pub struct AudiodevWavOptions {
    pub r#in: Option<AudiodevPerDirectionOptions>,
    pub out: Option<AudiodevPerDirectionOptions>,
    pub path: Option<String>,
}

/// QMP enum `AudioFormat`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.0.0")]
pub enum AudioFormat {
    #[qapi(name = "f32")]
    #[qapi(added = "5.0.0")]
    F32,
    #[qapi(name = "s16")]
    S16,
    #[qapi(name = "s32")]
    S32,
    #[qapi(name = "s8")]
    S8,
    #[qapi(name = "u16")]
    U16,
    #[qapi(name = "u32")]
    U32,
    #[qapi(name = "u8")]
    U8,
}

/// QMP struct `AuthZListFileProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct AuthZListFileProperties {
    pub filename: String,
    pub refresh: Option<bool>,
}

/// QMP struct `AuthZListProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct AuthZListProperties {
    pub policy: Option<QAuthZListPolicy>,
    pub rules: Option<Vec<QAuthZListRule>>,
}

/// QMP struct `AuthZPAMProperties`; see the QEMU QMP Reference Manual.
#[qapi(name = "AuthZPAMProperties")]
#[qapi(added = "6.0.0")]
pub struct AuthZpamProperties {
    pub service: String,
}

/// QMP struct `AuthZSimpleProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct AuthZSimpleProperties {
    pub identity: String,
}

/// QMP struct `BackupCommon`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.2.0")]
pub struct BackupCommon {
    #[qapi(name = "auto-dismiss")]
    pub auto_dismiss: Option<bool>,
    #[qapi(name = "auto-finalize")]
    pub auto_finalize: Option<bool>,
    pub bitmap: Option<String>,
    #[qapi(name = "bitmap-mode")]
    pub bitmap_mode: Option<BitmapSyncMode>,
    pub compress: Option<bool>,
    pub device: String,
    #[qapi(name = "discard-source")]
    #[qapi(added = "9.1.0")]
    pub discard_source: Option<bool>,
    #[qapi(name = "filter-node-name")]
    pub filter_node_name: Option<String>,
    #[qapi(name = "job-id")]
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
    #[qapi(name = "x-perf")]
    #[qapi(feature = "unstable")]
    #[qapi(added = "6.0.0")]
    pub x_perf: Option<BackupPerf>,
}

/// QMP struct `BackupPerf`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct BackupPerf {
    #[qapi(name = "max-chunk")]
    pub max_chunk: Option<i64>,
    #[qapi(name = "max-workers")]
    pub max_workers: Option<i64>,
    #[qapi(name = "min-cluster-size")]
    #[qapi(added = "9.2.0")]
    pub min_cluster_size: Option<u64>,
    #[qapi(name = "use-copy-range")]
    pub use_copy_range: Option<bool>,
}

/// QMP struct `BalloonInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.0.0")]
pub struct BalloonInfo {
    pub actual: i64,
    #[qapi(removed = "1.4.0")]
    pub free_mem: Option<i64>,
    #[qapi(removed = "1.4.0")]
    pub major_page_faults: Option<i64>,
    #[qapi(removed = "1.4.0")]
    pub mem_swapped_in: Option<i64>,
    #[qapi(removed = "1.4.0")]
    pub mem_swapped_out: Option<i64>,
    #[qapi(removed = "1.4.0")]
    pub minor_page_faults: Option<i64>,
    #[qapi(removed = "1.4.0")]
    pub total_mem: Option<i64>,
}

/// QMP enum `BiosAtaTranslation`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.0.0")]
pub enum BiosAtaTranslation {
    #[qapi(name = "auto")]
    Auto,
    #[qapi(name = "large")]
    Large,
    #[qapi(name = "lba")]
    Lba,
    #[qapi(name = "none")]
    None,
    #[qapi(name = "rechs")]
    Rechs,
}

/// QMP struct `BitmapMigrationBitmapAlias`; see the QEMU QMP Reference Manual.
#[qapi(added = "5.2.0")]
pub struct BitmapMigrationBitmapAlias {
    pub alias: String,
    pub name: String,
    #[qapi(added = "6.0.0")]
    pub transform: Option<BitmapMigrationBitmapAliasTransform>,
}

/// QMP struct `BitmapMigrationBitmapAliasTransform`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct BitmapMigrationBitmapAliasTransform {
    pub persistent: Option<bool>,
}

/// QMP struct `BitmapMigrationNodeAlias`; see the QEMU QMP Reference Manual.
#[qapi(added = "5.2.0")]
pub struct BitmapMigrationNodeAlias {
    pub alias: String,
    pub bitmaps: Vec<BitmapMigrationBitmapAlias>,
    #[qapi(name = "node-name")]
    pub node_name: String,
}

/// QMP enum `BitmapSyncMode`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.2.0")]
pub enum BitmapSyncMode {
    #[qapi(name = "always")]
    Always,
    #[qapi(name = "never")]
    Never,
    #[qapi(name = "on-success")]
    OnSuccess,
}

/// QMP enum `BlkdebugEvent`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.0.0")]
pub enum BlkdebugEvent {
    #[qapi(name = "cluster_alloc")]
    ClusterAlloc,
    #[qapi(name = "cluster_alloc_bytes")]
    ClusterAllocBytes,
    #[qapi(name = "cluster_alloc_space")]
    #[qapi(added = "4.1.0")]
    ClusterAllocSpace,
    #[qapi(name = "cluster_free")]
    ClusterFree,
    #[qapi(name = "cor_write")]
    #[qapi(added = "2.11.0")]
    CorWrite,
    #[qapi(name = "cow_read")]
    CowRead,
    #[qapi(name = "cow_write")]
    CowWrite,
    #[qapi(name = "empty_image_prepare")]
    #[qapi(added = "2.2.0")]
    EmptyImagePrepare,
    #[qapi(name = "flush_to_disk")]
    FlushToDisk,
    #[qapi(name = "flush_to_os")]
    FlushToOs,
    #[qapi(name = "l1_grow_activate_table")]
    #[qapi(added = "2.6.0")]
    L1GrowActivateTable,
    #[qapi(name = "l1_grow_alloc_table")]
    #[qapi(added = "2.6.0")]
    L1GrowAllocTable,
    #[qapi(name = "l1_grow_write_table")]
    #[qapi(added = "2.6.0")]
    L1GrowWriteTable,
    #[qapi(name = "l1_shrink_free_l2_clusters")]
    #[qapi(added = "2.11.0")]
    L1ShrinkFreeL2Clusters,
    #[qapi(name = "l1_shrink_write_table")]
    #[qapi(added = "2.11.0")]
    L1ShrinkWriteTable,
    #[qapi(name = "l1_update")]
    L1Update,
    #[qapi(name = "l1_grow.activate_table")]
    #[qapi(removed = "2.6.0")]
    L1GrowActivateTableLegacy,
    #[qapi(name = "l1_grow.alloc_table")]
    #[qapi(removed = "2.6.0")]
    L1GrowAllocTableLegacy,
    #[qapi(name = "l1_grow.write_table")]
    #[qapi(removed = "2.6.0")]
    L1GrowWriteTableLegacy,
    #[qapi(name = "l2_alloc_cow_read")]
    #[qapi(added = "2.6.0")]
    L2AllocCowRead,
    #[qapi(name = "l2_alloc_write")]
    #[qapi(added = "2.6.0")]
    L2AllocWrite,
    #[qapi(name = "l2_load")]
    L2Load,
    #[qapi(name = "l2_update")]
    L2Update,
    #[qapi(name = "l2_update_compressed")]
    L2UpdateCompressed,
    #[qapi(name = "l2_alloc.cow_read")]
    #[qapi(removed = "2.6.0")]
    L2AllocCowReadLegacy,
    #[qapi(name = "l2_alloc.write")]
    #[qapi(removed = "2.6.0")]
    L2AllocWriteLegacy,
    #[qapi(name = "none")]
    #[qapi(added = "4.1.0")]
    None,
    #[qapi(name = "pwritev")]
    #[qapi(added = "2.2.0")]
    Pwritev,
    #[qapi(name = "pwritev_done")]
    #[qapi(added = "2.2.0")]
    PwritevDone,
    #[qapi(name = "pwritev_rmw_after_head")]
    #[qapi(added = "2.6.0")]
    PwritevRmwAfterHead,
    #[qapi(name = "pwritev_rmw_after_tail")]
    #[qapi(added = "2.6.0")]
    PwritevRmwAfterTail,
    #[qapi(name = "pwritev_rmw_head")]
    #[qapi(added = "2.6.0")]
    PwritevRmwHead,
    #[qapi(name = "pwritev_rmw_tail")]
    #[qapi(added = "2.6.0")]
    PwritevRmwTail,
    #[qapi(name = "pwritev_zero")]
    #[qapi(added = "2.2.0")]
    PwritevZero,
    #[qapi(name = "pwritev_rmw.after_head")]
    #[qapi(added = "2.2.0")]
    #[qapi(removed = "2.6.0")]
    PwritevRmwAfterHeadLegacy,
    #[qapi(name = "pwritev_rmw.after_tail")]
    #[qapi(added = "2.2.0")]
    #[qapi(removed = "2.6.0")]
    PwritevRmwAfterTailLegacy,
    #[qapi(name = "pwritev_rmw.head")]
    #[qapi(added = "2.2.0")]
    #[qapi(removed = "2.6.0")]
    PwritevRmwHeadLegacy,
    #[qapi(name = "pwritev_rmw.tail")]
    #[qapi(added = "2.2.0")]
    #[qapi(removed = "2.6.0")]
    PwritevRmwTailLegacy,
    #[qapi(name = "read_aio")]
    ReadAio,
    #[qapi(name = "read_backing_aio")]
    ReadBackingAio,
    #[qapi(name = "read_compressed")]
    ReadCompressed,
    #[qapi(name = "refblock_alloc")]
    RefblockAlloc,
    #[qapi(name = "refblock_alloc_hookup")]
    #[qapi(added = "2.6.0")]
    RefblockAllocHookup,
    #[qapi(name = "refblock_alloc_switch_table")]
    #[qapi(added = "2.6.0")]
    RefblockAllocSwitchTable,
    #[qapi(name = "refblock_alloc_write")]
    #[qapi(added = "2.6.0")]
    RefblockAllocWrite,
    #[qapi(name = "refblock_alloc_write_blocks")]
    #[qapi(added = "2.6.0")]
    RefblockAllocWriteBlocks,
    #[qapi(name = "refblock_alloc_write_table")]
    #[qapi(added = "2.6.0")]
    RefblockAllocWriteTable,
    #[qapi(name = "refblock_load")]
    RefblockLoad,
    #[qapi(name = "refblock_update")]
    RefblockUpdate,
    #[qapi(name = "refblock_update_part")]
    RefblockUpdatePart,
    #[qapi(name = "refblock_alloc.hookup")]
    #[qapi(removed = "2.6.0")]
    RefblockAllocHookupLegacy,
    #[qapi(name = "refblock_alloc.switch_table")]
    #[qapi(removed = "2.6.0")]
    RefblockAllocSwitchTableLegacy,
    #[qapi(name = "refblock_alloc.write_blocks")]
    #[qapi(removed = "2.6.0")]
    RefblockAllocWriteBlocksLegacy,
    #[qapi(name = "refblock_alloc.write")]
    #[qapi(removed = "2.6.0")]
    RefblockAllocWriteLegacy,
    #[qapi(name = "refblock_alloc.write_table")]
    #[qapi(removed = "2.6.0")]
    RefblockAllocWriteTableLegacy,
    #[qapi(name = "reftable_grow")]
    ReftableGrow,
    #[qapi(name = "reftable_load")]
    ReftableLoad,
    #[qapi(name = "reftable_update")]
    ReftableUpdate,
    #[qapi(name = "vmstate_load")]
    VmstateLoad,
    #[qapi(name = "vmstate_save")]
    VmstateSave,
    #[qapi(name = "write_aio")]
    WriteAio,
    #[qapi(name = "write_compressed")]
    WriteCompressed,
}

/// QMP struct `BlkdebugInjectErrorOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.0.0")]
pub struct BlkdebugInjectErrorOptions {
    #[qapi(name = "delay-ns")]
    #[qapi(added = "11.0.1")]
    pub delay_ns: Option<i64>,
    #[qapi(name = "delay-ns")]
    #[qapi(added = "10.0.10")]
    #[qapi(removed = "10.1.0")]
    pub delay_ns_legacy_v10_0: Option<i64>,
    #[qapi(name = "delay-ns")]
    #[qapi(added = "10.2.3")]
    #[qapi(removed = "11.0.0")]
    pub delay_ns_legacy_v10_2: Option<i64>,
    pub errno: Option<i64>,
    pub event: BlkdebugEvent,
    pub immediately: Option<bool>,
    #[qapi(added = "4.1.0")]
    pub iotype: Option<BlkdebugIoType>,
    pub once: Option<bool>,
    pub sector: Option<i64>,
    pub state: Option<i64>,
}

/// QMP enum `BlkdebugIOType`; see the QEMU QMP Reference Manual.
#[qapi(name = "BlkdebugIOType")]
#[qapi(added = "4.1.0")]
pub enum BlkdebugIoType {
    #[qapi(name = "block-status")]
    BlockStatus,
    #[qapi(name = "discard")]
    Discard,
    #[qapi(name = "flush")]
    Flush,
    #[qapi(name = "read")]
    Read,
    #[qapi(name = "write")]
    Write,
    #[qapi(name = "write-zeroes")]
    WriteZeroes,
}

/// QMP struct `BlkdebugSetStateOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.0.0")]
pub struct BlkdebugSetStateOptions {
    pub event: BlkdebugEvent,
    pub new_state: i64,
    pub state: Option<i64>,
}

/// QMP struct `BlockChildInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.0.0")]
pub struct BlockChildInfo {
    pub info: BlockGraphInfo,
    pub name: String,
}

/// QMP union `BlockdevAction`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.1.0")]
#[qapi(removed = "1.6.0")]
#[qapi(simple_union)]
pub struct BlockdevAction {
    #[qapi(discriminator)]
    #[qapi(name = "type")]
    pub r_type: BlockdevActionKind,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<BlockdevActionBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "1.1.0")]
#[qapi(removed = "1.6.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum BlockdevActionBranch {
    #[qapi(name = "blockdev-snapshot-sync")]
    BlockdevSnapshotSync(BlockdevSnapshot),
}

#[qapi(added = "1.1.0")]
#[qapi(removed = "1.6.0")]
pub enum BlockdevActionKind {
    #[qapi(name = "blockdev-snapshot-sync")]
    BlockdevSnapshotSync,
}

/// QMP enum `BlockdevAioOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.7.0")]
pub enum BlockdevAioOptions {
    #[qapi(name = "io_uring")]
    #[qapi(condition = "CONFIG_LINUX_IO_URING")]
    #[qapi(added = "5.0.0")]
    IoUring,
    #[qapi(name = "native")]
    Native,
    #[qapi(name = "threads")]
    Threads,
}

/// QMP union `BlockdevAmendOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "5.1.0")]
pub struct BlockdevAmendOptions {
    #[qapi(discriminator)]
    pub driver: BlockdevDriver,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<BlockdevAmendOptionsBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "5.1.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum BlockdevAmendOptionsBranch {
    #[qapi(name = "blkdebug")]
    Blkdebug,
    #[qapi(name = "blklogwrites")]
    Blklogwrites,
    #[qapi(name = "blkreplay")]
    Blkreplay,
    #[qapi(name = "blkverify")]
    Blkverify,
    #[qapi(name = "bochs")]
    Bochs,
    #[qapi(name = "cloop")]
    Cloop,
    #[qapi(name = "compress")]
    Compress,
    #[qapi(name = "copy-before-write")]
    #[qapi(added = "6.2.0")]
    CopyBeforeWrite,
    #[qapi(name = "copy-on-read")]
    CopyOnRead,
    #[qapi(name = "dmg")]
    Dmg,
    #[qapi(name = "file")]
    File,
    #[qapi(name = "ftp")]
    Ftp,
    #[qapi(name = "ftps")]
    Ftps,
    #[qapi(name = "gluster")]
    #[qapi(deprecated(added = "9.2.0"))]
    #[qapi(removed = "11.1.0")]
    Gluster,
    #[qapi(name = "host_cdrom")]
    #[qapi(condition = "HAVE_HOST_BLOCK_DEVICE")]
    HostCdrom,
    #[qapi(name = "host_device")]
    #[qapi(condition = "HAVE_HOST_BLOCK_DEVICE")]
    HostDevice,
    #[qapi(name = "http")]
    Http,
    #[qapi(name = "https")]
    Https,
    #[qapi(name = "io_uring")]
    #[qapi(condition = "CONFIG_BLKIO")]
    #[qapi(added = "7.2.0")]
    IoUring,
    #[qapi(name = "iscsi")]
    Iscsi,
    #[qapi(name = "luks")]
    Luks(BlockdevAmendOptionsLuks),
    #[qapi(name = "nbd")]
    Nbd,
    #[qapi(name = "nfs")]
    Nfs,
    #[qapi(name = "null-aio")]
    NullAio,
    #[qapi(name = "null-co")]
    NullCo,
    #[qapi(name = "nvme")]
    Nvme,
    #[qapi(name = "nvme-io_uring")]
    #[qapi(condition = "CONFIG_BLKIO")]
    #[qapi(added = "7.2.0")]
    NvmeIoUring,
    #[qapi(name = "parallels")]
    Parallels,
    #[qapi(name = "preallocate")]
    #[qapi(added = "6.0.0")]
    Preallocate,
    #[qapi(name = "qcow")]
    Qcow,
    #[qapi(name = "qcow2")]
    Qcow2(BlockdevAmendOptionsQcow2),
    #[qapi(name = "qed")]
    Qed,
    #[qapi(name = "quorum")]
    Quorum,
    #[qapi(name = "raw")]
    Raw,
    #[qapi(name = "rbd")]
    Rbd,
    #[qapi(name = "replication")]
    #[qapi(condition = "CONFIG_REPLICATION")]
    Replication,
    #[qapi(name = "sheepdog")]
    #[qapi(removed = "6.1.0")]
    Sheepdog,
    #[qapi(name = "snapshot-access")]
    #[qapi(added = "7.0.0")]
    SnapshotAccess,
    #[qapi(name = "ssh")]
    Ssh,
    #[qapi(name = "throttle")]
    Throttle,
    #[qapi(name = "vdi")]
    Vdi,
    #[qapi(name = "vhdx")]
    Vhdx,
    #[qapi(name = "virtio-blk-vfio-pci")]
    #[qapi(condition = "CONFIG_BLKIO")]
    #[qapi(added = "7.2.0")]
    VirtioBlkVfioPci,
    #[qapi(name = "virtio-blk-vhost-user")]
    #[qapi(condition = "CONFIG_BLKIO")]
    #[qapi(added = "7.2.0")]
    VirtioBlkVhostUser,
    #[qapi(name = "virtio-blk-vhost-vdpa")]
    #[qapi(condition = "CONFIG_BLKIO")]
    #[qapi(added = "7.2.0")]
    VirtioBlkVhostVdpa,
    #[qapi(name = "vmdk")]
    Vmdk,
    #[qapi(name = "vpc")]
    Vpc,
    #[qapi(name = "vvfat")]
    Vvfat,
}

/// QMP struct `BlockdevAmendOptionsLUKS`; see the QEMU QMP Reference Manual.
#[qapi(name = "BlockdevAmendOptionsLUKS")]
#[qapi(added = "5.1.0")]
pub struct BlockdevAmendOptionsLuks {
    #[qapi(name = "iter-time")]
    pub iter_time: Option<i64>,
    pub keyslot: Option<i64>,
    #[qapi(name = "new-secret")]
    pub new_secret: Option<String>,
    #[qapi(name = "old-secret")]
    pub old_secret: Option<String>,
    pub secret: Option<String>,
    pub state: QCryptoBlockLuksKeyslotState,
}

/// QMP struct `BlockdevAmendOptionsQcow2`; see the QEMU QMP Reference Manual.
#[qapi(added = "5.1.0")]
pub struct BlockdevAmendOptionsQcow2 {
    pub encrypt: Option<QCryptoBlockAmendOptions>,
}

/// QMP struct `BlockdevBackupWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct BlockdevBackupWrapper {
    pub data: BlockdevBackup,
}

/// QMP struct `BlockdevCacheInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.3.0")]
pub struct BlockdevCacheInfo {
    pub direct: bool,
    #[qapi(name = "no-flush")]
    pub no_flush: bool,
    pub writeback: bool,
}

/// QMP struct `BlockdevCacheOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.7.0")]
pub struct BlockdevCacheOptions {
    pub direct: Option<bool>,
    #[qapi(name = "no-flush")]
    pub no_flush: Option<bool>,
    #[qapi(removed = "2.6.0")]
    pub writeback: Option<bool>,
}

/// QMP enum `BlockdevChangeReadOnlyMode`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.5.0")]
pub enum BlockdevChangeReadOnlyMode {
    #[qapi(name = "read-only")]
    ReadOnly,
    #[qapi(name = "read-write")]
    ReadWrite,
    #[qapi(name = "retain")]
    Retain,
}

/// QMP struct `BlockdevChild`; see the QEMU QMP Reference Manual.
#[qapi(added = "10.1.0")]
pub struct BlockdevChild {
    pub child: String,
    #[qapi(name = "node-name")]
    pub node_name: String,
}

/// QMP struct `BlockdevCreateNotSupported`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
#[qapi(removed = "3.0.0")]
pub struct BlockdevCreateNotSupported {}

/// QMP union `BlockdevCreateOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
pub struct BlockdevCreateOptions {
    #[qapi(discriminator)]
    pub driver: BlockdevDriver,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<BlockdevCreateOptionsBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "2.12.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum BlockdevCreateOptionsBranch {
    #[qapi(name = "blkdebug")]
    #[qapi(added = "3.0.0")]
    Blkdebug,
    #[qapi(name = "blkdebug")]
    #[qapi(removed = "3.0.0")]
    BlkdebugLegacy(BlockdevCreateNotSupported),
    #[qapi(name = "blklogwrites")]
    #[qapi(added = "3.0.0")]
    Blklogwrites,
    #[qapi(name = "blkreplay")]
    #[qapi(added = "4.2.0")]
    Blkreplay,
    #[qapi(name = "blkverify")]
    #[qapi(added = "3.0.0")]
    Blkverify,
    #[qapi(name = "blkverify")]
    #[qapi(removed = "3.0.0")]
    BlkverifyLegacy(BlockdevCreateNotSupported),
    #[qapi(name = "bochs")]
    #[qapi(added = "3.0.0")]
    Bochs,
    #[qapi(name = "bochs")]
    #[qapi(removed = "3.0.0")]
    BochsLegacy(BlockdevCreateNotSupported),
    #[qapi(name = "cloop")]
    #[qapi(added = "3.0.0")]
    Cloop,
    #[qapi(name = "cloop")]
    #[qapi(removed = "3.0.0")]
    CloopLegacy(BlockdevCreateNotSupported),
    #[qapi(name = "compress")]
    #[qapi(added = "5.0.0")]
    Compress,
    #[qapi(name = "copy-before-write")]
    #[qapi(added = "6.2.0")]
    CopyBeforeWrite,
    #[qapi(name = "copy-on-read")]
    #[qapi(added = "3.0.0")]
    CopyOnRead,
    #[qapi(name = "dmg")]
    #[qapi(added = "3.0.0")]
    Dmg,
    #[qapi(name = "dmg")]
    #[qapi(removed = "3.0.0")]
    DmgLegacy(BlockdevCreateNotSupported),
    #[qapi(name = "file")]
    File(BlockdevCreateOptionsFile),
    #[qapi(name = "ftp")]
    #[qapi(added = "3.0.0")]
    Ftp,
    #[qapi(name = "ftp")]
    #[qapi(removed = "3.0.0")]
    FtpLegacy(BlockdevCreateNotSupported),
    #[qapi(name = "ftps")]
    #[qapi(added = "3.0.0")]
    Ftps,
    #[qapi(name = "ftps")]
    #[qapi(removed = "3.0.0")]
    FtpsLegacy(BlockdevCreateNotSupported),
    #[qapi(name = "gluster")]
    #[qapi(removed = "11.1.0")]
    Gluster(BlockdevCreateOptionsGluster),
    #[qapi(name = "host_cdrom")]
    #[qapi(condition = "HAVE_HOST_BLOCK_DEVICE")]
    #[qapi(added = "3.0.0")]
    HostCdrom,
    #[qapi(name = "host_device")]
    #[qapi(condition = "HAVE_HOST_BLOCK_DEVICE")]
    #[qapi(added = "3.0.0")]
    HostDevice,
    #[qapi(name = "host_cdrom")]
    #[qapi(removed = "3.0.0")]
    HostCdromLegacy(BlockdevCreateNotSupported),
    #[qapi(name = "host_device")]
    #[qapi(removed = "3.0.0")]
    HostDeviceLegacy(BlockdevCreateNotSupported),
    #[qapi(name = "http")]
    #[qapi(added = "3.0.0")]
    Http,
    #[qapi(name = "http")]
    #[qapi(removed = "3.0.0")]
    HttpLegacy(BlockdevCreateNotSupported),
    #[qapi(name = "https")]
    #[qapi(added = "3.0.0")]
    Https,
    #[qapi(name = "https")]
    #[qapi(removed = "3.0.0")]
    HttpsLegacy(BlockdevCreateNotSupported),
    #[qapi(name = "io_uring")]
    #[qapi(condition = "CONFIG_BLKIO")]
    #[qapi(added = "7.2.0")]
    IoUring,
    #[qapi(name = "iscsi")]
    #[qapi(added = "3.0.0")]
    Iscsi,
    #[qapi(name = "iscsi")]
    #[qapi(removed = "3.0.0")]
    IscsiLegacy(BlockdevCreateNotSupported),
    #[qapi(name = "luks")]
    Luks(BlockdevCreateOptionsLuks),
    #[qapi(name = "nbd")]
    #[qapi(added = "3.0.0")]
    Nbd,
    #[qapi(name = "nbd")]
    #[qapi(removed = "3.0.0")]
    NbdLegacy(BlockdevCreateNotSupported),
    #[qapi(name = "nfs")]
    Nfs(BlockdevCreateOptionsNfs),
    #[qapi(name = "null-aio")]
    #[qapi(added = "3.0.0")]
    NullAio,
    #[qapi(name = "null-co")]
    #[qapi(added = "3.0.0")]
    NullCo,
    #[qapi(name = "null-aio")]
    #[qapi(removed = "3.0.0")]
    NullAioLegacy(BlockdevCreateNotSupported),
    #[qapi(name = "null-co")]
    #[qapi(removed = "3.0.0")]
    NullCoLegacy(BlockdevCreateNotSupported),
    #[qapi(name = "nvme")]
    #[qapi(added = "3.0.0")]
    Nvme,
    #[qapi(name = "nvme-io_uring")]
    #[qapi(condition = "CONFIG_BLKIO")]
    #[qapi(added = "7.2.0")]
    NvmeIoUring,
    #[qapi(name = "nvme")]
    #[qapi(removed = "3.0.0")]
    NvmeLegacy(BlockdevCreateNotSupported),
    #[qapi(name = "parallels")]
    Parallels(BlockdevCreateOptionsParallels),
    #[qapi(name = "preallocate")]
    #[qapi(added = "6.0.0")]
    Preallocate,
    #[qapi(name = "qcow")]
    Qcow(BlockdevCreateOptionsQcow),
    #[qapi(name = "qcow2")]
    Qcow2(BlockdevCreateOptionsQcow2),
    #[qapi(name = "qed")]
    Qed(BlockdevCreateOptionsQed),
    #[qapi(name = "quorum")]
    #[qapi(added = "3.0.0")]
    Quorum,
    #[qapi(name = "quorum")]
    #[qapi(removed = "3.0.0")]
    QuorumLegacy(BlockdevCreateNotSupported),
    #[qapi(name = "raw")]
    #[qapi(added = "3.0.0")]
    Raw,
    #[qapi(name = "raw")]
    #[qapi(removed = "3.0.0")]
    RawLegacy(BlockdevCreateNotSupported),
    #[qapi(name = "rbd")]
    Rbd(BlockdevCreateOptionsRbd),
    #[qapi(name = "replication")]
    #[qapi(condition = "CONFIG_REPLICATION")]
    #[qapi(added = "3.0.0")]
    Replication,
    #[qapi(name = "replication")]
    #[qapi(removed = "3.0.0")]
    ReplicationLegacy(BlockdevCreateNotSupported),
    #[qapi(name = "sheepdog")]
    #[qapi(removed = "6.1.0")]
    Sheepdog(BlockdevCreateOptionsSheepdog),
    #[qapi(name = "snapshot-access")]
    #[qapi(added = "7.0.0")]
    SnapshotAccess,
    #[qapi(name = "ssh")]
    Ssh(BlockdevCreateOptionsSsh),
    #[qapi(name = "throttle")]
    #[qapi(added = "3.0.0")]
    Throttle,
    #[qapi(name = "throttle")]
    #[qapi(removed = "3.0.0")]
    ThrottleLegacy(BlockdevCreateNotSupported),
    #[qapi(name = "vdi")]
    Vdi(BlockdevCreateOptionsVdi),
    #[qapi(name = "vhdx")]
    Vhdx(BlockdevCreateOptionsVhdx),
    #[qapi(name = "virtio-blk-vfio-pci")]
    #[qapi(condition = "CONFIG_BLKIO")]
    #[qapi(added = "7.2.0")]
    VirtioBlkVfioPci,
    #[qapi(name = "virtio-blk-vhost-user")]
    #[qapi(condition = "CONFIG_BLKIO")]
    #[qapi(added = "7.2.0")]
    VirtioBlkVhostUser,
    #[qapi(name = "virtio-blk-vhost-vdpa")]
    #[qapi(condition = "CONFIG_BLKIO")]
    #[qapi(added = "7.2.0")]
    VirtioBlkVhostVdpa,
    #[qapi(name = "vmdk")]
    #[qapi(added = "4.0.0")]
    Vmdk(BlockdevCreateOptionsVmdk),
    #[qapi(name = "vmdk")]
    #[qapi(removed = "3.0.0")]
    VmdkLegacy(BlockdevCreateNotSupported),
    #[qapi(name = "vmdk")]
    #[qapi(added = "3.0.0")]
    #[qapi(removed = "4.0.0")]
    VmdkLegacy2,
    #[qapi(name = "vpc")]
    Vpc(BlockdevCreateOptionsVpc),
    #[qapi(name = "vvfat")]
    #[qapi(added = "3.0.0")]
    Vvfat,
    #[qapi(name = "vvfat")]
    #[qapi(removed = "3.0.0")]
    VvfatLegacy(BlockdevCreateNotSupported),
    #[qapi(name = "vxhs")]
    #[qapi(added = "3.0.0")]
    #[qapi(removed = "5.1.0")]
    Vxhs,
    #[qapi(name = "vxhs")]
    #[qapi(removed = "3.0.0")]
    VxhsLegacy(BlockdevCreateNotSupported),
}

/// QMP struct `BlockdevCreateOptionsFile`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
pub struct BlockdevCreateOptionsFile {
    #[qapi(name = "extent-size-hint")]
    #[qapi(added = "5.1.0")]
    pub extent_size_hint: Option<u64>,
    pub filename: String,
    pub nocow: Option<bool>,
    pub preallocation: Option<PreallocMode>,
    pub size: u64,
}

/// QMP struct `BlockdevCreateOptionsGluster`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
#[qapi(removed = "11.1.0")]
pub struct BlockdevCreateOptionsGluster {
    pub location: BlockdevOptionsGluster,
    pub preallocation: Option<PreallocMode>,
    pub size: u64,
}

/// QMP struct `BlockdevCreateOptionsLUKS`; see the QEMU QMP Reference Manual.
#[qapi(name = "BlockdevCreateOptionsLUKS")]
#[qapi(added = "2.12.0")]
pub struct BlockdevCreateOptionsLuks {
    #[qapi(name = "cipher-alg")]
    #[qapi(added = "9.2.0")]
    pub cipher_alg: Option<QCryptoCipherAlgo>,
    #[qapi(name = "cipher-mode")]
    pub cipher_mode: Option<QCryptoCipherMode>,
    #[qapi(name = "cipher-alg")]
    #[qapi(removed = "9.2.0")]
    pub cipher_alg_legacy: Option<QCryptoCipherAlgorithm>,
    #[qapi(name = "detached-header")]
    #[qapi(added = "9.0.0")]
    #[qapi(removed = "9.1.0")]
    pub detached_header: Option<bool>,
    #[qapi(added = "9.0.0")]
    pub file: Option<BlockdevRef>,
    #[qapi(name = "file")]
    #[qapi(removed = "9.0.0")]
    #[qapi(required)]
    pub file_legacy: Option<BlockdevRef>,
    #[qapi(name = "hash-alg")]
    #[qapi(added = "9.2.0")]
    pub hash_alg: Option<QCryptoHashAlgo>,
    #[qapi(name = "hash-alg")]
    #[qapi(removed = "9.2.0")]
    pub hash_alg_legacy: Option<QCryptoHashAlgorithm>,
    #[qapi(added = "9.0.0")]
    pub header: Option<BlockdevRef>,
    #[qapi(name = "iter-time")]
    pub iter_time: Option<i64>,
    #[qapi(name = "ivgen-alg")]
    #[qapi(added = "9.2.0")]
    pub ivgen_alg: Option<QCryptoIvGenAlgo>,
    #[qapi(name = "ivgen-hash-alg")]
    #[qapi(added = "9.2.0")]
    pub ivgen_hash_alg: Option<QCryptoHashAlgo>,
    #[qapi(name = "ivgen-alg")]
    #[qapi(removed = "9.2.0")]
    pub ivgen_alg_legacy: Option<QCryptoIvGenAlgorithm>,
    #[qapi(name = "ivgen-hash-alg")]
    #[qapi(removed = "9.2.0")]
    pub ivgen_hash_alg_legacy: Option<QCryptoHashAlgorithm>,
    #[qapi(name = "key-secret")]
    pub key_secret: Option<String>,
    #[qapi(added = "4.2.0")]
    pub preallocation: Option<PreallocMode>,
    pub size: u64,
}

/// QMP struct `BlockdevCreateOptionsNfs`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
pub struct BlockdevCreateOptionsNfs {
    pub location: BlockdevOptionsNfs,
    pub size: u64,
}

/// QMP struct `BlockdevCreateOptionsParallels`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
pub struct BlockdevCreateOptionsParallels {
    #[qapi(name = "cluster-size")]
    pub cluster_size: Option<u64>,
    pub file: BlockdevRef,
    pub size: u64,
}

/// QMP struct `BlockdevCreateOptionsQcow`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
pub struct BlockdevCreateOptionsQcow {
    #[qapi(name = "backing-file")]
    pub backing_file: Option<String>,
    pub encrypt: Option<QCryptoBlockCreateOptions>,
    pub file: BlockdevRef,
    pub size: u64,
}

/// QMP struct `BlockdevCreateOptionsQcow2`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
pub struct BlockdevCreateOptionsQcow2 {
    #[qapi(name = "backing-file")]
    pub backing_file: Option<String>,
    #[qapi(name = "backing-fmt")]
    pub backing_fmt: Option<BlockdevDriver>,
    #[qapi(name = "cluster-size")]
    pub cluster_size: Option<u64>,
    #[qapi(name = "compression-type")]
    #[qapi(added = "5.1.0")]
    pub compression_type: Option<Qcow2CompressionType>,
    #[qapi(name = "data-file")]
    #[qapi(added = "4.0.0")]
    pub data_file: Option<BlockdevRef>,
    #[qapi(name = "data-file-raw")]
    #[qapi(added = "4.0.0")]
    pub data_file_raw: Option<bool>,
    pub encrypt: Option<QCryptoBlockCreateOptions>,
    #[qapi(name = "extended-l2")]
    #[qapi(added = "5.2.0")]
    pub extended_l2: Option<bool>,
    pub file: BlockdevRef,
    #[qapi(name = "lazy-refcounts")]
    pub lazy_refcounts: Option<bool>,
    pub preallocation: Option<PreallocMode>,
    #[qapi(name = "refcount-bits")]
    pub refcount_bits: Option<i64>,
    pub size: u64,
    pub version: Option<BlockdevQcow2Version>,
}

/// QMP struct `BlockdevCreateOptionsQed`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
pub struct BlockdevCreateOptionsQed {
    #[qapi(name = "backing-file")]
    pub backing_file: Option<String>,
    #[qapi(name = "backing-fmt")]
    pub backing_fmt: Option<BlockdevDriver>,
    #[qapi(name = "cluster-size")]
    pub cluster_size: Option<u64>,
    pub file: BlockdevRef,
    pub size: u64,
    #[qapi(name = "table-size")]
    pub table_size: Option<i64>,
}

/// QMP struct `BlockdevCreateOptionsRbd`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
pub struct BlockdevCreateOptionsRbd {
    #[qapi(name = "cluster-size")]
    pub cluster_size: Option<u64>,
    #[qapi(added = "6.1.0")]
    pub encrypt: Option<RbdEncryptionCreateOptions>,
    pub location: BlockdevOptionsRbd,
    pub size: u64,
}

/// QMP struct `BlockdevCreateOptionsSheepdog`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
#[qapi(removed = "6.1.0")]
pub struct BlockdevCreateOptionsSheepdog {
    #[qapi(name = "backing-file")]
    pub backing_file: Option<String>,
    pub location: BlockdevOptionsSheepdog,
    #[qapi(name = "object-size")]
    pub object_size: Option<u64>,
    pub preallocation: Option<PreallocMode>,
    pub redundancy: Option<SheepdogRedundancy>,
    pub size: u64,
}

/// QMP struct `BlockdevCreateOptionsSsh`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
pub struct BlockdevCreateOptionsSsh {
    pub location: BlockdevOptionsSsh,
    pub size: u64,
}

/// QMP struct `BlockdevCreateOptionsVdi`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
pub struct BlockdevCreateOptionsVdi {
    pub file: BlockdevRef,
    pub preallocation: Option<PreallocMode>,
    pub size: u64,
}

/// QMP struct `BlockdevCreateOptionsVhdx`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
pub struct BlockdevCreateOptionsVhdx {
    #[qapi(name = "block-size")]
    pub block_size: Option<u64>,
    #[qapi(name = "block-state-zero")]
    pub block_state_zero: Option<bool>,
    pub file: BlockdevRef,
    #[qapi(name = "log-size")]
    pub log_size: Option<u64>,
    pub size: u64,
    pub subformat: Option<BlockdevVhdxSubformat>,
}

/// QMP struct `BlockdevCreateOptionsVmdk`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.0.0")]
pub struct BlockdevCreateOptionsVmdk {
    #[qapi(name = "adapter-type")]
    pub adapter_type: Option<BlockdevVmdkAdapterType>,
    #[qapi(name = "backing-file")]
    pub backing_file: Option<String>,
    pub extents: Option<Vec<BlockdevRef>>,
    pub file: BlockdevRef,
    pub hwversion: Option<String>,
    pub size: u64,
    pub subformat: Option<BlockdevVmdkSubformat>,
    #[qapi(added = "6.2.0")]
    pub toolsversion: Option<String>,
    #[qapi(name = "zeroed-grain")]
    pub zeroed_grain: Option<bool>,
}

/// QMP struct `BlockdevCreateOptionsVpc`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
pub struct BlockdevCreateOptionsVpc {
    pub file: BlockdevRef,
    #[qapi(name = "force-size")]
    pub force_size: Option<bool>,
    pub size: u64,
    pub subformat: Option<BlockdevVpcSubformat>,
}

/// QMP enum `BlockdevDetectZeroesOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.1.0")]
pub enum BlockdevDetectZeroesOptions {
    #[qapi(name = "off")]
    Off,
    #[qapi(name = "on")]
    On,
    #[qapi(name = "unmap")]
    Unmap,
}

/// QMP enum `BlockdevDiscardOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.7.0")]
pub enum BlockdevDiscardOptions {
    #[qapi(name = "ignore")]
    Ignore,
    #[qapi(name = "unmap")]
    Unmap,
}

/// QMP enum `BlockdevDriver`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.0.0")]
pub enum BlockdevDriver {
    #[qapi(name = "archipelago")]
    #[qapi(added = "2.2.0")]
    #[qapi(removed = "2.9.0")]
    Archipelago,
    #[qapi(name = "blkdebug")]
    Blkdebug,
    #[qapi(name = "blklogwrites")]
    #[qapi(added = "3.0.0")]
    Blklogwrites,
    #[qapi(name = "blkreplay")]
    #[qapi(added = "4.2.0")]
    Blkreplay,
    #[qapi(name = "blkverify")]
    Blkverify,
    #[qapi(name = "bochs")]
    Bochs,
    #[qapi(name = "cloop")]
    Cloop,
    #[qapi(name = "compress")]
    #[qapi(added = "5.0.0")]
    Compress,
    #[qapi(name = "copy-before-write")]
    #[qapi(added = "6.2.0")]
    CopyBeforeWrite,
    #[qapi(name = "copy-on-read")]
    #[qapi(added = "3.0.0")]
    CopyOnRead,
    #[qapi(name = "cow")]
    #[qapi(removed = "2.2.0")]
    Cow,
    #[qapi(name = "dmg")]
    Dmg,
    #[qapi(name = "file")]
    File,
    #[qapi(name = "ftp")]
    Ftp,
    #[qapi(name = "ftps")]
    Ftps,
    #[qapi(name = "gluster")]
    #[qapi(added = "2.7.0")]
    #[qapi(deprecated(added = "9.2.0"))]
    #[qapi(removed = "11.1.0")]
    Gluster,
    #[qapi(name = "host_cdrom")]
    #[qapi(condition = "HAVE_HOST_BLOCK_DEVICE")]
    #[qapi(added = "2.1.0")]
    HostCdrom,
    #[qapi(name = "host_device")]
    #[qapi(condition = "HAVE_HOST_BLOCK_DEVICE")]
    #[qapi(added = "2.1.0")]
    HostDevice,
    #[qapi(name = "host_floppy")]
    #[qapi(added = "2.1.0")]
    #[qapi(removed = "2.5.0")]
    HostFloppy,
    #[qapi(name = "http")]
    Http,
    #[qapi(name = "https")]
    Https,
    #[qapi(name = "io_uring")]
    #[qapi(condition = "CONFIG_BLKIO")]
    #[qapi(added = "7.2.0")]
    IoUring,
    #[qapi(name = "iscsi")]
    #[qapi(added = "2.9.0")]
    Iscsi,
    #[qapi(name = "luks")]
    #[qapi(added = "2.6.0")]
    Luks,
    #[qapi(name = "nbd")]
    #[qapi(added = "2.8.0")]
    Nbd,
    #[qapi(name = "nfs")]
    #[qapi(added = "2.8.0")]
    Nfs,
    #[qapi(name = "null-aio")]
    #[qapi(added = "2.2.0")]
    NullAio,
    #[qapi(name = "null-co")]
    #[qapi(added = "2.2.0")]
    NullCo,
    #[qapi(name = "nvme")]
    #[qapi(added = "2.12.0")]
    Nvme,
    #[qapi(name = "nvme-io_uring")]
    #[qapi(condition = "CONFIG_BLKIO")]
    #[qapi(added = "7.2.0")]
    NvmeIoUring,
    #[qapi(name = "parallels")]
    Parallels,
    #[qapi(name = "preallocate")]
    #[qapi(added = "6.0.0")]
    Preallocate,
    #[qapi(name = "qcow")]
    Qcow,
    #[qapi(name = "qcow2")]
    Qcow2,
    #[qapi(name = "qed")]
    Qed,
    #[qapi(name = "quorum")]
    Quorum,
    #[qapi(name = "raw")]
    Raw,
    #[qapi(name = "rbd")]
    #[qapi(added = "2.9.0")]
    Rbd,
    #[qapi(name = "replication")]
    #[qapi(condition = "CONFIG_REPLICATION")]
    #[qapi(added = "2.8.0")]
    Replication,
    #[qapi(name = "sheepdog")]
    #[qapi(added = "2.9.0")]
    #[qapi(removed = "6.1.0")]
    Sheepdog,
    #[qapi(name = "snapshot-access")]
    #[qapi(added = "7.0.0")]
    SnapshotAccess,
    #[qapi(name = "ssh")]
    #[qapi(added = "2.8.0")]
    Ssh,
    #[qapi(name = "tftp")]
    #[qapi(removed = "2.8.0")]
    Tftp,
    #[qapi(name = "throttle")]
    #[qapi(added = "2.11.0")]
    Throttle,
    #[qapi(name = "vdi")]
    Vdi,
    #[qapi(name = "vhdx")]
    Vhdx,
    #[qapi(name = "virtio-blk-vfio-pci")]
    #[qapi(condition = "CONFIG_BLKIO")]
    #[qapi(added = "7.2.0")]
    VirtioBlkVfioPci,
    #[qapi(name = "virtio-blk-vhost-user")]
    #[qapi(condition = "CONFIG_BLKIO")]
    #[qapi(added = "7.2.0")]
    VirtioBlkVhostUser,
    #[qapi(name = "virtio-blk-vhost-vdpa")]
    #[qapi(condition = "CONFIG_BLKIO")]
    #[qapi(added = "7.2.0")]
    VirtioBlkVhostVdpa,
    #[qapi(name = "vmdk")]
    Vmdk,
    #[qapi(name = "vpc")]
    Vpc,
    #[qapi(name = "vvfat")]
    Vvfat,
    #[qapi(name = "vxhs")]
    #[qapi(added = "2.10.0")]
    #[qapi(removed = "5.1.0")]
    Vxhs,
}

/// QMP struct `BlockDeviceInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.0.0")]
pub struct BlockDeviceInfo {
    #[qapi(added = "10.0.0")]
    #[qapi(required)]
    pub active: Option<bool>,
    pub backing_file: Option<String>,
    #[qapi(added = "1.2.0")]
    #[qapi(required)]
    pub backing_file_depth: Option<i64>,
    #[qapi(added = "1.1.0")]
    #[qapi(required)]
    pub bps: Option<i64>,
    #[qapi(added = "1.7.0")]
    pub bps_max: Option<i64>,
    #[qapi(added = "2.6.0")]
    pub bps_max_length: Option<i64>,
    #[qapi(added = "1.1.0")]
    #[qapi(required)]
    pub bps_rd: Option<i64>,
    #[qapi(added = "1.7.0")]
    pub bps_rd_max: Option<i64>,
    #[qapi(added = "2.6.0")]
    pub bps_rd_max_length: Option<i64>,
    #[qapi(added = "1.1.0")]
    #[qapi(required)]
    pub bps_wr: Option<i64>,
    #[qapi(added = "1.7.0")]
    pub bps_wr_max: Option<i64>,
    #[qapi(added = "2.6.0")]
    pub bps_wr_max_length: Option<i64>,
    #[qapi(added = "2.3.0")]
    #[qapi(required)]
    pub cache: Option<BlockdevCacheInfo>,
    #[qapi(added = "10.1.0")]
    #[qapi(required)]
    pub children: Option<Vec<BlockdevChild>>,
    #[qapi(added = "2.1.0")]
    #[qapi(required)]
    pub detect_zeroes: Option<BlockdevDetectZeroesOptions>,
    #[qapi(name = "dirty-bitmaps")]
    #[qapi(added = "4.2.0")]
    pub dirty_bitmaps: Option<Vec<BlockDirtyInfo>>,
    pub drv: String,
    pub encrypted: bool,
    #[qapi(added = "1.2.0")]
    #[qapi(deprecated(added = "5.0.0"))]
    #[qapi(removed = "6.0.0")]
    #[qapi(required)]
    pub encryption_key_missing: Option<bool>,
    pub file: String,
    #[qapi(added = "2.4.0")]
    pub group: Option<String>,
    #[qapi(added = "1.6.0")]
    #[qapi(required)]
    pub image: Option<ImageInfo>,
    #[qapi(added = "1.1.0")]
    #[qapi(required)]
    pub iops: Option<i64>,
    #[qapi(added = "1.7.0")]
    pub iops_max: Option<i64>,
    #[qapi(added = "2.6.0")]
    pub iops_max_length: Option<i64>,
    #[qapi(added = "1.1.0")]
    #[qapi(required)]
    pub iops_rd: Option<i64>,
    #[qapi(added = "1.7.0")]
    pub iops_rd_max: Option<i64>,
    #[qapi(added = "2.6.0")]
    pub iops_rd_max_length: Option<i64>,
    #[qapi(added = "1.7.0")]
    pub iops_size: Option<i64>,
    #[qapi(added = "1.1.0")]
    #[qapi(required)]
    pub iops_wr: Option<i64>,
    #[qapi(added = "1.7.0")]
    pub iops_wr_max: Option<i64>,
    #[qapi(added = "2.6.0")]
    pub iops_wr_max_length: Option<i64>,
    #[qapi(name = "node-name")]
    #[qapi(added = "10.1.0")]
    #[qapi(required)]
    pub node_name: Option<String>,
    #[qapi(name = "node-name")]
    #[qapi(added = "2.0.0")]
    #[qapi(removed = "10.1.0")]
    pub node_name_legacy: Option<String>,
    pub ro: bool,
    #[qapi(added = "2.3.0")]
    #[qapi(required)]
    pub write_threshold: Option<i64>,
}

/// QMP enum `BlockDeviceIoStatus`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.0.0")]
pub enum BlockDeviceIoStatus {
    #[qapi(name = "failed")]
    Failed,
    #[qapi(name = "nospace")]
    Nospace,
    #[qapi(name = "ok")]
    Ok,
}

/// QMP struct `BlockDeviceMapEntry`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.7.0")]
#[qapi(removed = "5.2.0")]
pub struct BlockDeviceMapEntry {
    pub data: bool,
    pub depth: i64,
    pub length: i64,
    pub offset: Option<i64>,
    pub start: i64,
    pub zero: bool,
}

/// QMP struct `BlockDeviceStats`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.0.0")]
pub struct BlockDeviceStats {
    #[qapi(added = "2.5.0")]
    #[qapi(required)]
    pub account_failed: Option<bool>,
    #[qapi(added = "2.5.0")]
    #[qapi(required)]
    pub account_invalid: Option<bool>,
    #[qapi(added = "2.5.0")]
    #[qapi(required)]
    pub failed_flush_operations: Option<i64>,
    #[qapi(added = "2.5.0")]
    #[qapi(required)]
    pub failed_rd_operations: Option<i64>,
    #[qapi(added = "4.2.0")]
    #[qapi(required)]
    pub failed_unmap_operations: Option<i64>,
    #[qapi(added = "2.5.0")]
    #[qapi(required)]
    pub failed_wr_operations: Option<i64>,
    #[qapi(added = "8.1.0")]
    #[qapi(required)]
    pub failed_zone_append_operations: Option<i64>,
    #[qapi(added = "4.0.0")]
    pub flush_latency_histogram: Option<BlockLatencyHistogramInfo>,
    pub flush_operations: i64,
    pub flush_total_time_ns: i64,
    #[qapi(added = "2.5.0")]
    pub idle_time_ns: Option<i64>,
    #[qapi(added = "2.5.0")]
    #[qapi(required)]
    pub invalid_flush_operations: Option<i64>,
    #[qapi(added = "2.5.0")]
    #[qapi(required)]
    pub invalid_rd_operations: Option<i64>,
    #[qapi(added = "4.2.0")]
    #[qapi(required)]
    pub invalid_unmap_operations: Option<i64>,
    #[qapi(added = "2.5.0")]
    #[qapi(required)]
    pub invalid_wr_operations: Option<i64>,
    #[qapi(added = "8.1.0")]
    #[qapi(required)]
    pub invalid_zone_append_operations: Option<i64>,
    pub rd_bytes: i64,
    #[qapi(added = "4.0.0")]
    pub rd_latency_histogram: Option<BlockLatencyHistogramInfo>,
    #[qapi(added = "2.3.0")]
    #[qapi(required)]
    pub rd_merged: Option<i64>,
    pub rd_operations: i64,
    pub rd_total_time_ns: i64,
    #[qapi(added = "2.5.0")]
    #[qapi(required)]
    pub timed_stats: Option<Vec<BlockDeviceTimedStats>>,
    #[qapi(added = "4.2.0")]
    #[qapi(required)]
    pub unmap_bytes: Option<i64>,
    #[qapi(added = "4.2.0")]
    #[qapi(required)]
    pub unmap_merged: Option<i64>,
    #[qapi(added = "4.2.0")]
    #[qapi(required)]
    pub unmap_operations: Option<i64>,
    #[qapi(added = "4.2.0")]
    #[qapi(required)]
    pub unmap_total_time_ns: Option<i64>,
    pub wr_bytes: i64,
    pub wr_highest_offset: i64,
    #[qapi(added = "4.0.0")]
    pub wr_latency_histogram: Option<BlockLatencyHistogramInfo>,
    #[qapi(added = "2.3.0")]
    #[qapi(required)]
    pub wr_merged: Option<i64>,
    pub wr_operations: i64,
    pub wr_total_time_ns: i64,
    #[qapi(added = "2.12.0")]
    #[qapi(removed = "4.0.0")]
    pub x_flush_latency_histogram: Option<BlockLatencyHistogramInfo>,
    #[qapi(added = "2.12.0")]
    #[qapi(removed = "4.0.0")]
    pub x_rd_latency_histogram: Option<BlockLatencyHistogramInfo>,
    #[qapi(added = "2.12.0")]
    #[qapi(removed = "4.0.0")]
    pub x_wr_latency_histogram: Option<BlockLatencyHistogramInfo>,
    #[qapi(added = "8.1.0")]
    #[qapi(required)]
    pub zone_append_bytes: Option<i64>,
    #[qapi(added = "8.1.0")]
    pub zone_append_latency_histogram: Option<BlockLatencyHistogramInfo>,
    #[qapi(added = "8.1.0")]
    #[qapi(required)]
    pub zone_append_merged: Option<i64>,
    #[qapi(added = "8.1.0")]
    #[qapi(required)]
    pub zone_append_operations: Option<i64>,
    #[qapi(added = "8.1.0")]
    #[qapi(required)]
    pub zone_append_total_time_ns: Option<i64>,
}

/// QMP struct `BlockDeviceTimedStats`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.5.0")]
pub struct BlockDeviceTimedStats {
    pub avg_flush_latency_ns: i64,
    pub avg_rd_latency_ns: i64,
    pub avg_rd_queue_depth: f64,
    pub avg_wr_latency_ns: i64,
    pub avg_wr_queue_depth: f64,
    #[qapi(added = "8.1.0")]
    #[qapi(required)]
    pub avg_zone_append_latency_ns: Option<i64>,
    #[qapi(added = "8.1.0")]
    #[qapi(required)]
    pub avg_zone_append_queue_depth: Option<f64>,
    pub interval_length: i64,
    pub max_flush_latency_ns: i64,
    pub max_rd_latency_ns: i64,
    pub max_wr_latency_ns: i64,
    #[qapi(added = "8.1.0")]
    #[qapi(required)]
    pub max_zone_append_latency_ns: Option<i64>,
    pub min_flush_latency_ns: i64,
    pub min_rd_latency_ns: i64,
    pub min_wr_latency_ns: i64,
    #[qapi(added = "8.1.0")]
    #[qapi(required)]
    pub min_zone_append_latency_ns: Option<i64>,
}

/// QMP enum `BlockdevOnError`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.3.0")]
pub enum BlockdevOnError {
    #[qapi(name = "auto")]
    #[qapi(added = "2.7.0")]
    Auto,
    #[qapi(name = "enospc")]
    Enospc,
    #[qapi(name = "ignore")]
    Ignore,
    #[qapi(name = "report")]
    Report,
    #[qapi(name = "stop")]
    Stop,
}

/// QMP union `BlockdevOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.7.0")]
pub struct BlockdevOptions {
    #[qapi(added = "10.0.0")]
    pub active: Option<bool>,
    #[qapi(removed = "2.8.0")]
    pub aio: Option<BlockdevAioOptions>,
    #[qapi(name = "auto-read-only")]
    #[qapi(added = "3.1.0")]
    pub auto_read_only: Option<bool>,
    pub cache: Option<BlockdevCacheOptions>,
    #[qapi(name = "detect-zeroes")]
    #[qapi(added = "2.1.0")]
    pub detect_zeroes: Option<BlockdevDetectZeroesOptions>,
    pub discard: Option<BlockdevDiscardOptions>,
    #[qapi(discriminator)]
    #[qapi(added = "2.0.0")]
    #[qapi(required)]
    pub driver: Option<BlockdevDriver>,
    #[qapi(discriminator)]
    #[qapi(name = "driver")]
    #[qapi(removed = "2.0.0")]
    #[qapi(required)]
    pub driver_legacy: Option<String>,
    #[qapi(name = "force-share")]
    #[qapi(added = "2.10.0")]
    pub force_share: Option<bool>,
    #[qapi(removed = "2.8.0")]
    pub id: Option<String>,
    #[qapi(name = "node-name")]
    #[qapi(added = "2.0.0")]
    pub node_name: Option<String>,
    #[qapi(name = "read-only")]
    pub read_only: Option<bool>,
    #[qapi(removed = "2.7.0")]
    pub rerror: Option<BlockdevOnError>,
    #[qapi(name = "stats-account-failed")]
    #[qapi(added = "2.5.0")]
    #[qapi(removed = "2.7.0")]
    pub stats_account_failed: Option<bool>,
    #[qapi(name = "stats-account-invalid")]
    #[qapi(added = "2.5.0")]
    #[qapi(removed = "2.7.0")]
    pub stats_account_invalid: Option<bool>,
    #[qapi(name = "stats-intervals")]
    #[qapi(added = "2.5.0")]
    #[qapi(removed = "2.7.0")]
    pub stats_intervals: Option<Vec<i64>>,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<Box<BlockdevOptionsBranch>>,
    #[qapi(removed = "2.7.0")]
    pub werror: Option<BlockdevOnError>,
}

/// QMP struct `BlockdevOptionsArchipelago`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.2.0")]
#[qapi(removed = "2.9.0")]
pub struct BlockdevOptionsArchipelago {
    pub mport: Option<i64>,
    pub segment: Option<String>,
    pub volume: String,
    pub vport: Option<i64>,
}

/// QMP struct `BlockdevOptionsBase`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.7.0")]
#[qapi(removed = "2.6.0")]
pub struct BlockdevOptionsBase {
    pub aio: Option<BlockdevAioOptions>,
    pub cache: Option<BlockdevCacheOptions>,
    #[qapi(name = "detect-zeroes")]
    #[qapi(added = "2.1.0")]
    pub detect_zeroes: Option<BlockdevDetectZeroesOptions>,
    pub discard: Option<BlockdevDiscardOptions>,
    #[qapi(added = "2.0.0")]
    #[qapi(required)]
    pub driver: Option<BlockdevDriver>,
    #[qapi(name = "driver")]
    #[qapi(removed = "2.0.0")]
    #[qapi(required)]
    pub driver_legacy: Option<String>,
    pub id: Option<String>,
    #[qapi(name = "node-name")]
    #[qapi(added = "2.0.0")]
    pub node_name: Option<String>,
    #[qapi(name = "read-only")]
    pub read_only: Option<bool>,
    pub rerror: Option<BlockdevOnError>,
    #[qapi(name = "stats-account-failed")]
    #[qapi(added = "2.5.0")]
    pub stats_account_failed: Option<bool>,
    #[qapi(name = "stats-account-invalid")]
    #[qapi(added = "2.5.0")]
    pub stats_account_invalid: Option<bool>,
    #[qapi(name = "stats-intervals")]
    #[qapi(added = "2.5.0")]
    pub stats_intervals: Option<Vec<i64>>,
    pub werror: Option<BlockdevOnError>,
}

/// QMP struct `BlockdevOptionsBlkdebug`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.0.0")]
pub struct BlockdevOptionsBlkdebug {
    pub align: Option<i64>,
    pub config: Option<String>,
    pub image: Box<BlockdevRef>,
    #[qapi(name = "inject-error")]
    pub inject_error: Option<Vec<BlkdebugInjectErrorOptions>>,
    #[qapi(name = "max-discard")]
    #[qapi(added = "2.9.1")]
    pub max_discard: Option<i32>,
    #[qapi(name = "max-transfer")]
    #[qapi(added = "2.9.1")]
    pub max_transfer: Option<i32>,
    #[qapi(name = "max-write-zero")]
    #[qapi(added = "2.9.1")]
    pub max_write_zero: Option<i32>,
    #[qapi(name = "opt-discard")]
    #[qapi(added = "2.9.1")]
    pub opt_discard: Option<i32>,
    #[qapi(name = "opt-write-zero")]
    #[qapi(added = "2.9.1")]
    pub opt_write_zero: Option<i32>,
    #[qapi(name = "set-state")]
    pub set_state: Option<Vec<BlkdebugSetStateOptions>>,
    #[qapi(name = "take-child-perms")]
    #[qapi(added = "4.2.1")]
    pub take_child_perms: Option<Vec<BlockPermission>>,
    #[qapi(name = "unshare-child-perms")]
    #[qapi(added = "4.2.1")]
    pub unshare_child_perms: Option<Vec<BlockPermission>>,
}

/// QMP struct `BlockdevOptionsBlklogwrites`; see the QEMU QMP Reference Manual.
#[qapi(added = "3.0.0")]
pub struct BlockdevOptionsBlklogwrites {
    pub file: Box<BlockdevRef>,
    pub log: Box<BlockdevRef>,
    #[qapi(name = "log-append")]
    pub log_append: Option<bool>,
    #[qapi(name = "log-sector-size")]
    pub log_sector_size: Option<u32>,
    #[qapi(name = "log-super-update-interval")]
    pub log_super_update_interval: Option<u64>,
}

/// QMP struct `BlockdevOptionsBlkreplay`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.2.0")]
pub struct BlockdevOptionsBlkreplay {
    pub image: Box<BlockdevRef>,
}

/// QMP struct `BlockdevOptionsBlkverify`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.0.0")]
pub struct BlockdevOptionsBlkverify {
    pub raw: Box<BlockdevRef>,
    pub test: Box<BlockdevRef>,
}

#[qapi(union_branch)]
#[qapi(added = "1.7.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum BlockdevOptionsBranch {
    #[qapi(name = "archipelago")]
    #[qapi(added = "2.2.0")]
    #[qapi(removed = "2.9.0")]
    Archipelago(BlockdevOptionsArchipelago),
    #[qapi(name = "blkdebug")]
    #[qapi(added = "2.0.0")]
    Blkdebug(BlockdevOptionsBlkdebug),
    #[qapi(name = "blklogwrites")]
    #[qapi(added = "3.0.0")]
    Blklogwrites(BlockdevOptionsBlklogwrites),
    #[qapi(name = "blkreplay")]
    #[qapi(added = "4.2.0")]
    Blkreplay(BlockdevOptionsBlkreplay),
    #[qapi(name = "blkverify")]
    #[qapi(added = "2.0.0")]
    Blkverify(BlockdevOptionsBlkverify),
    #[qapi(name = "bochs")]
    Bochs(BlockdevOptionsGenericFormat),
    #[qapi(name = "cloop")]
    Cloop(BlockdevOptionsGenericFormat),
    #[qapi(name = "compress")]
    #[qapi(added = "5.0.0")]
    Compress(BlockdevOptionsGenericFormat),
    #[qapi(name = "copy-before-write")]
    #[qapi(added = "6.2.0")]
    CopyBeforeWrite(BlockdevOptionsCbw),
    #[qapi(name = "copy-on-read")]
    #[qapi(added = "6.0.0")]
    CopyOnRead(BlockdevOptionsCor),
    #[qapi(name = "copy-on-read")]
    #[qapi(added = "3.0.0")]
    #[qapi(removed = "6.0.0")]
    CopyOnReadLegacy(BlockdevOptionsGenericFormat),
    #[qapi(name = "cow")]
    #[qapi(removed = "2.2.0")]
    Cow(BlockdevOptionsGenericCowFormat),
    #[qapi(name = "dmg")]
    Dmg(BlockdevOptionsGenericFormat),
    #[qapi(name = "file")]
    File(BlockdevOptionsFile),
    #[qapi(name = "ftp")]
    #[qapi(added = "2.9.0")]
    Ftp(BlockdevOptionsCurlFtp),
    #[qapi(name = "ftp")]
    #[qapi(removed = "2.8.0")]
    FtpLegacy(BlockdevOptionsFile),
    #[qapi(name = "ftp")]
    #[qapi(added = "2.8.0")]
    #[qapi(removed = "2.9.0")]
    FtpLegacy2(BlockdevOptionsCurl),
    #[qapi(name = "ftps")]
    #[qapi(added = "2.9.0")]
    Ftps(BlockdevOptionsCurlFtps),
    #[qapi(name = "ftps")]
    #[qapi(removed = "2.8.0")]
    FtpsLegacy(BlockdevOptionsFile),
    #[qapi(name = "ftps")]
    #[qapi(added = "2.8.0")]
    #[qapi(removed = "2.9.0")]
    FtpsLegacy2(BlockdevOptionsCurl),
    #[qapi(name = "gluster")]
    #[qapi(added = "2.7.0")]
    #[qapi(removed = "11.1.0")]
    Gluster(BlockdevOptionsGluster),
    #[qapi(name = "host_cdrom")]
    #[qapi(condition = "HAVE_HOST_BLOCK_DEVICE")]
    #[qapi(added = "2.1.0")]
    HostCdrom(BlockdevOptionsFile),
    #[qapi(name = "host_device")]
    #[qapi(condition = "HAVE_HOST_BLOCK_DEVICE")]
    #[qapi(added = "2.1.0")]
    HostDevice(BlockdevOptionsFile),
    #[qapi(name = "host_floppy")]
    #[qapi(added = "2.1.0")]
    #[qapi(removed = "2.5.0")]
    HostFloppy(BlockdevOptionsFile),
    #[qapi(name = "http")]
    #[qapi(added = "2.9.0")]
    Http(BlockdevOptionsCurlHttp),
    #[qapi(name = "http")]
    #[qapi(removed = "2.8.0")]
    HttpLegacy(BlockdevOptionsFile),
    #[qapi(name = "http")]
    #[qapi(added = "2.8.0")]
    #[qapi(removed = "2.9.0")]
    HttpLegacy2(BlockdevOptionsCurl),
    #[qapi(name = "https")]
    #[qapi(added = "2.9.0")]
    Https(BlockdevOptionsCurlHttps),
    #[qapi(name = "https")]
    #[qapi(removed = "2.8.0")]
    HttpsLegacy(BlockdevOptionsFile),
    #[qapi(name = "https")]
    #[qapi(added = "2.8.0")]
    #[qapi(removed = "2.9.0")]
    HttpsLegacy2(BlockdevOptionsCurl),
    #[qapi(name = "io_uring")]
    #[qapi(condition = "CONFIG_BLKIO")]
    #[qapi(added = "7.2.0")]
    IoUring(BlockdevOptionsIoUring),
    #[qapi(name = "iscsi")]
    #[qapi(added = "2.9.0")]
    Iscsi(BlockdevOptionsIscsi),
    #[qapi(name = "luks")]
    #[qapi(added = "2.6.0")]
    Luks(BlockdevOptionsLuks),
    #[qapi(name = "nbd")]
    #[qapi(added = "2.8.0")]
    Nbd(BlockdevOptionsNbd),
    #[qapi(name = "nfs")]
    #[qapi(added = "2.8.0")]
    Nfs(BlockdevOptionsNfs),
    #[qapi(name = "null-aio")]
    #[qapi(added = "2.2.0")]
    NullAio(BlockdevOptionsNull),
    #[qapi(name = "null-co")]
    #[qapi(added = "2.2.0")]
    NullCo(BlockdevOptionsNull),
    #[qapi(name = "nvme")]
    #[qapi(added = "2.12.0")]
    Nvme(BlockdevOptionsNvMe),
    #[qapi(name = "nvme-io_uring")]
    #[qapi(condition = "CONFIG_BLKIO")]
    #[qapi(added = "7.2.0")]
    NvmeIoUring(BlockdevOptionsNvmeIoUring),
    #[qapi(name = "parallels")]
    Parallels(BlockdevOptionsGenericFormat),
    #[qapi(name = "preallocate")]
    #[qapi(added = "6.0.0")]
    Preallocate(BlockdevOptionsPreallocate),
    #[qapi(name = "qcow")]
    #[qapi(added = "2.10.0")]
    Qcow(BlockdevOptionsQcow),
    #[qapi(name = "qcow2")]
    Qcow2(BlockdevOptionsQcow2),
    #[qapi(name = "qcow")]
    #[qapi(removed = "2.10.0")]
    QcowLegacy(BlockdevOptionsGenericCowFormat),
    #[qapi(name = "qed")]
    Qed(BlockdevOptionsGenericCowFormat),
    #[qapi(name = "quorum")]
    #[qapi(added = "2.0.0")]
    Quorum(BlockdevOptionsQuorum),
    #[qapi(name = "raw")]
    #[qapi(added = "2.8.0")]
    Raw(BlockdevOptionsRaw),
    #[qapi(name = "raw")]
    #[qapi(removed = "2.8.0")]
    RawLegacy(BlockdevOptionsGenericFormat),
    #[qapi(name = "rbd")]
    #[qapi(added = "2.9.0")]
    Rbd(BlockdevOptionsRbd),
    #[qapi(name = "replication")]
    #[qapi(condition = "CONFIG_REPLICATION")]
    #[qapi(added = "2.8.0")]
    Replication(BlockdevOptionsReplication),
    #[qapi(name = "sheepdog")]
    #[qapi(added = "2.9.0")]
    #[qapi(removed = "6.1.0")]
    Sheepdog(BlockdevOptionsSheepdog),
    #[qapi(name = "snapshot-access")]
    #[qapi(added = "7.0.0")]
    SnapshotAccess(BlockdevOptionsGenericFormat),
    #[qapi(name = "ssh")]
    #[qapi(added = "2.8.0")]
    Ssh(BlockdevOptionsSsh),
    #[qapi(name = "tftp")]
    #[qapi(removed = "2.8.0")]
    Tftp(BlockdevOptionsFile),
    #[qapi(name = "throttle")]
    #[qapi(added = "2.11.0")]
    Throttle(BlockdevOptionsThrottle),
    #[qapi(name = "vdi")]
    Vdi(BlockdevOptionsGenericFormat),
    #[qapi(name = "vhdx")]
    Vhdx(BlockdevOptionsGenericFormat),
    #[qapi(name = "virtio-blk-vfio-pci")]
    #[qapi(condition = "CONFIG_BLKIO")]
    #[qapi(added = "7.2.0")]
    VirtioBlkVfioPci(BlockdevOptionsVirtioBlkVfioPci),
    #[qapi(name = "virtio-blk-vhost-user")]
    #[qapi(condition = "CONFIG_BLKIO")]
    #[qapi(added = "7.2.0")]
    VirtioBlkVhostUser(BlockdevOptionsVirtioBlkVhostUser),
    #[qapi(name = "virtio-blk-vhost-vdpa")]
    #[qapi(condition = "CONFIG_BLKIO")]
    #[qapi(added = "7.2.0")]
    VirtioBlkVhostVdpa(BlockdevOptionsVirtioBlkVhostVdpa),
    #[qapi(name = "vmdk")]
    Vmdk(BlockdevOptionsGenericCowFormat),
    #[qapi(name = "vpc")]
    Vpc(BlockdevOptionsGenericFormat),
    #[qapi(name = "vvfat")]
    Vvfat(BlockdevOptionsVvfat),
    #[qapi(name = "vxhs")]
    #[qapi(added = "2.10.0")]
    #[qapi(removed = "5.1.0")]
    Vxhs(BlockdevOptionsVxHs),
}

/// QMP struct `BlockdevOptionsCbw`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct BlockdevOptionsCbw {
    #[qapi(added = "7.0.0")]
    pub bitmap: Option<BlockDirtyBitmap>,
    #[qapi(name = "cbw-timeout")]
    #[qapi(added = "7.1.0")]
    pub cbw_timeout: Option<u32>,
    pub file: Box<BlockdevRef>,
    #[qapi(name = "min-cluster-size")]
    #[qapi(added = "9.2.0")]
    pub min_cluster_size: Option<u64>,
    #[qapi(name = "on-cbw-error")]
    #[qapi(added = "7.1.0")]
    pub on_cbw_error: Option<OnCbwError>,
    pub target: Box<BlockdevRef>,
}

/// QMP struct `BlockdevOptionsCor`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct BlockdevOptionsCor {
    pub bottom: Option<String>,
    pub file: Box<BlockdevRef>,
}

/// QMP struct `BlockdevOptionsCurl`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.8.0")]
#[qapi(removed = "2.9.0")]
pub struct BlockdevOptionsCurl {
    pub filename: String,
}

/// QMP struct `BlockdevOptionsCurlBase`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.9.0")]
pub struct BlockdevOptionsCurlBase {
    #[qapi(name = "password-secret")]
    pub password_secret: Option<String>,
    #[qapi(name = "proxy-password-secret")]
    pub proxy_password_secret: Option<String>,
    #[qapi(name = "proxy-username")]
    pub proxy_username: Option<String>,
    pub readahead: Option<i64>,
    pub timeout: Option<i64>,
    pub url: String,
    pub username: Option<String>,
}

/// QMP struct `BlockdevOptionsCurlFtp`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.9.0")]
pub struct BlockdevOptionsCurlFtp {
    #[qapi(name = "password-secret")]
    pub password_secret: Option<String>,
    #[qapi(name = "proxy-password-secret")]
    pub proxy_password_secret: Option<String>,
    #[qapi(name = "proxy-username")]
    pub proxy_username: Option<String>,
    pub readahead: Option<i64>,
    pub timeout: Option<i64>,
    pub url: String,
    pub username: Option<String>,
}

/// QMP struct `BlockdevOptionsCurlFtps`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.9.0")]
pub struct BlockdevOptionsCurlFtps {
    #[qapi(name = "password-secret")]
    pub password_secret: Option<String>,
    #[qapi(name = "proxy-password-secret")]
    pub proxy_password_secret: Option<String>,
    #[qapi(name = "proxy-username")]
    pub proxy_username: Option<String>,
    pub readahead: Option<i64>,
    pub sslverify: Option<bool>,
    pub timeout: Option<i64>,
    pub url: String,
    pub username: Option<String>,
}

/// QMP struct `BlockdevOptionsCurlHttp`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.9.0")]
pub struct BlockdevOptionsCurlHttp {
    pub cookie: Option<String>,
    #[qapi(name = "cookie-secret")]
    #[qapi(added = "2.10.0")]
    pub cookie_secret: Option<String>,
    #[qapi(name = "force-range")]
    #[qapi(added = "11.0.0")]
    pub force_range: Option<bool>,
    #[qapi(name = "password-secret")]
    pub password_secret: Option<String>,
    #[qapi(name = "proxy-password-secret")]
    pub proxy_password_secret: Option<String>,
    #[qapi(name = "proxy-username")]
    pub proxy_username: Option<String>,
    pub readahead: Option<i64>,
    pub timeout: Option<i64>,
    pub url: String,
    pub username: Option<String>,
}

/// QMP struct `BlockdevOptionsCurlHttps`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.9.0")]
pub struct BlockdevOptionsCurlHttps {
    pub cookie: Option<String>,
    #[qapi(name = "cookie-secret")]
    #[qapi(added = "2.10.0")]
    pub cookie_secret: Option<String>,
    #[qapi(name = "force-range")]
    #[qapi(added = "11.0.0")]
    pub force_range: Option<bool>,
    #[qapi(name = "password-secret")]
    pub password_secret: Option<String>,
    #[qapi(name = "proxy-password-secret")]
    pub proxy_password_secret: Option<String>,
    #[qapi(name = "proxy-username")]
    pub proxy_username: Option<String>,
    pub readahead: Option<i64>,
    pub sslverify: Option<bool>,
    pub timeout: Option<i64>,
    pub url: String,
    pub username: Option<String>,
}

/// QMP struct `BlockdevOptionsFile`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.7.0")]
pub struct BlockdevOptionsFile {
    #[qapi(added = "2.8.0")]
    pub aio: Option<BlockdevAioOptions>,
    #[qapi(name = "aio-max-batch")]
    #[qapi(added = "6.2.0")]
    pub aio_max_batch: Option<i64>,
    #[qapi(name = "drop-cache")]
    #[qapi(condition = "CONFIG_LINUX")]
    #[qapi(added = "4.0.0")]
    pub drop_cache: Option<bool>,
    pub filename: String,
    #[qapi(added = "2.10.0")]
    pub locking: Option<OnOffAuto>,
    #[qapi(name = "pr-manager")]
    #[qapi(added = "2.11.0")]
    pub pr_manager: Option<String>,
    #[qapi(name = "x-check-cache-dropped")]
    #[qapi(feature = "unstable")]
    #[qapi(added = "3.0.0")]
    pub x_check_cache_dropped: Option<bool>,
}

/// QMP struct `BlockdevOptionsGenericCOWFormat`; see the QEMU QMP Reference Manual.
#[qapi(name = "BlockdevOptionsGenericCOWFormat")]
#[qapi(added = "1.7.0")]
pub struct BlockdevOptionsGenericCowFormat {
    #[qapi(added = "2.10.0")]
    pub backing: Option<Box<BlockdevRefOrNull>>,
    #[qapi(name = "backing")]
    #[qapi(removed = "2.10.0")]
    pub backing_legacy: Option<Box<BlockdevRef>>,
    pub file: Box<BlockdevRef>,
}

/// QMP struct `BlockdevOptionsGenericFormat`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.7.0")]
pub struct BlockdevOptionsGenericFormat {
    pub file: Box<BlockdevRef>,
}

/// QMP struct `BlockdevOptionsGluster`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.7.0")]
#[qapi(removed = "11.1.0")]
pub struct BlockdevOptionsGluster {
    #[qapi(added = "2.8.0")]
    pub debug: Option<i64>,
    #[qapi(name = "debug-level")]
    #[qapi(removed = "2.8.0")]
    pub debug_level: Option<i64>,
    #[qapi(added = "2.8.0")]
    pub logfile: Option<String>,
    pub path: String,
    #[qapi(added = "2.10.0")]
    #[qapi(required)]
    pub server: Option<Vec<SocketAddress>>,
    #[qapi(name = "server")]
    #[qapi(removed = "2.9.0")]
    #[qapi(required)]
    pub server_legacy: Option<Vec<GlusterServer>>,
    #[qapi(name = "server")]
    #[qapi(added = "2.9.0")]
    #[qapi(removed = "2.10.0")]
    #[qapi(required)]
    pub server_legacy2: Option<Vec<SocketAddressFlat>>,
    pub volume: String,
}

/// QMP struct `BlockdevOptionsIoUring`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_BLKIO")]
#[qapi(added = "7.2.0")]
pub struct BlockdevOptionsIoUring {
    pub filename: String,
}

/// QMP struct `BlockdevOptionsIscsi`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.9.0")]
pub struct BlockdevOptionsIscsi {
    #[qapi(name = "header-digest")]
    pub header_digest: Option<IscsiHeaderDigest>,
    #[qapi(name = "initiator-name")]
    pub initiator_name: Option<String>,
    pub lun: Option<i64>,
    #[qapi(name = "password-secret")]
    pub password_secret: Option<String>,
    pub portal: String,
    pub target: String,
    pub timeout: Option<i64>,
    pub transport: IscsiTransport,
    pub user: Option<String>,
}

/// QMP struct `BlockdevOptionsLUKS`; see the QEMU QMP Reference Manual.
#[qapi(name = "BlockdevOptionsLUKS")]
#[qapi(added = "2.6.0")]
pub struct BlockdevOptionsLuks {
    pub file: Box<BlockdevRef>,
    #[qapi(added = "9.0.0")]
    pub header: Option<Box<BlockdevRef>>,
    #[qapi(name = "key-secret")]
    pub key_secret: Option<String>,
}

/// QMP struct `BlockdevOptionsNbd`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.8.0")]
pub struct BlockdevOptionsNbd {
    pub export: Option<String>,
    #[qapi(name = "open-timeout")]
    #[qapi(added = "7.0.0")]
    pub open_timeout: Option<u32>,
    #[qapi(name = "reconnect-delay")]
    #[qapi(added = "4.2.0")]
    pub reconnect_delay: Option<u32>,
    #[qapi(added = "2.10.0")]
    #[qapi(required)]
    pub server: Option<SocketAddress>,
    #[qapi(name = "server")]
    #[qapi(removed = "2.9.0")]
    #[qapi(required)]
    pub server_legacy: Option<SocketAddress>,
    #[qapi(name = "server")]
    #[qapi(added = "2.9.0")]
    #[qapi(removed = "2.10.0")]
    #[qapi(required)]
    pub server_legacy2: Option<SocketAddressFlat>,
    #[qapi(name = "tls-creds")]
    pub tls_creds: Option<String>,
    #[qapi(name = "tls-hostname")]
    #[qapi(added = "7.0.0")]
    pub tls_hostname: Option<String>,
    #[qapi(name = "x-dirty-bitmap")]
    #[qapi(feature = "unstable")]
    #[qapi(added = "3.0.0")]
    pub x_dirty_bitmap: Option<String>,
}

/// QMP struct `BlockdevOptionsNfs`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.8.0")]
pub struct BlockdevOptionsNfs {
    pub debug: Option<i64>,
    pub group: Option<i64>,
    #[qapi(name = "page-cache-size")]
    pub page_cache_size: Option<i64>,
    pub path: String,
    #[qapi(name = "readahead-size")]
    pub readahead_size: Option<i64>,
    pub server: NfsServer,
    #[qapi(name = "tcp-syn-count")]
    pub tcp_syn_count: Option<i64>,
    pub user: Option<i64>,
}

/// QMP struct `BlockdevOptionsNull`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.2.0")]
pub struct BlockdevOptionsNull {
    #[qapi(name = "latency-ns")]
    #[qapi(added = "2.4.0")]
    pub latency_ns: Option<u64>,
    #[qapi(name = "read-zeroes")]
    #[qapi(added = "4.1.0")]
    pub read_zeroes: Option<bool>,
    pub size: Option<i64>,
}

/// QMP struct `BlockdevOptionsNVMe`; see the QEMU QMP Reference Manual.
#[qapi(name = "BlockdevOptionsNVMe")]
#[qapi(added = "2.12.0")]
pub struct BlockdevOptionsNvMe {
    pub device: String,
    pub namespace: i64,
}

/// QMP struct `BlockdevOptionsNvmeIoUring`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_BLKIO")]
#[qapi(added = "7.2.0")]
pub struct BlockdevOptionsNvmeIoUring {
    pub path: String,
}

/// QMP struct `BlockdevOptionsPreallocate`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct BlockdevOptionsPreallocate {
    pub file: Box<BlockdevRef>,
    #[qapi(name = "prealloc-align")]
    pub prealloc_align: Option<i64>,
    #[qapi(name = "prealloc-size")]
    pub prealloc_size: Option<i64>,
}

/// QMP struct `BlockdevOptionsQcow`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.10.0")]
pub struct BlockdevOptionsQcow {
    pub backing: Option<Box<BlockdevRefOrNull>>,
    pub encrypt: Option<BlockdevQcowEncryption>,
    pub file: Box<BlockdevRef>,
}

/// QMP struct `BlockdevOptionsQcow2`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.7.0")]
pub struct BlockdevOptionsQcow2 {
    #[qapi(added = "2.10.0")]
    pub backing: Option<Box<BlockdevRefOrNull>>,
    #[qapi(name = "backing")]
    #[qapi(removed = "2.10.0")]
    pub backing_legacy: Option<Box<BlockdevRef>>,
    #[qapi(name = "cache-clean-interval")]
    #[qapi(added = "2.5.0")]
    pub cache_clean_interval: Option<i64>,
    #[qapi(name = "cache-size")]
    #[qapi(added = "2.2.0")]
    pub cache_size: Option<i64>,
    #[qapi(name = "data-file")]
    #[qapi(added = "4.0.0")]
    pub data_file: Option<Box<BlockdevRef>>,
    #[qapi(name = "discard-no-unref")]
    #[qapi(added = "8.1.0")]
    pub discard_no_unref: Option<bool>,
    #[qapi(added = "2.10.0")]
    pub encrypt: Option<BlockdevQcow2Encryption>,
    pub file: Box<BlockdevRef>,
    #[qapi(name = "l2-cache-entry-size")]
    #[qapi(added = "2.12.0")]
    pub l2_cache_entry_size: Option<i64>,
    #[qapi(name = "l2-cache-size")]
    #[qapi(added = "2.2.0")]
    pub l2_cache_size: Option<i64>,
    #[qapi(name = "lazy-refcounts")]
    pub lazy_refcounts: Option<bool>,
    #[qapi(name = "overlap-check")]
    #[qapi(added = "2.2.0")]
    pub overlap_check: Option<Qcow2OverlapChecks>,
    #[qapi(name = "pass-discard-other")]
    pub pass_discard_other: Option<bool>,
    #[qapi(name = "pass-discard-request")]
    pub pass_discard_request: Option<bool>,
    #[qapi(name = "pass-discard-snapshot")]
    pub pass_discard_snapshot: Option<bool>,
    #[qapi(name = "refcount-cache-size")]
    #[qapi(added = "2.2.0")]
    pub refcount_cache_size: Option<i64>,
}

/// QMP struct `BlockdevOptionsQuorum`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.0.0")]
pub struct BlockdevOptionsQuorum {
    pub blkverify: Option<bool>,
    pub children: Vec<BlockdevRef>,
    #[qapi(name = "read-pattern")]
    #[qapi(added = "2.2.0")]
    pub read_pattern: Option<QuorumReadPattern>,
    #[qapi(name = "rewrite-corrupted")]
    #[qapi(added = "2.1.0")]
    pub rewrite_corrupted: Option<bool>,
    #[qapi(name = "vote-threshold")]
    pub vote_threshold: i64,
}

/// QMP struct `BlockdevOptionsRaw`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.8.0")]
pub struct BlockdevOptionsRaw {
    pub file: Box<BlockdevRef>,
    pub offset: Option<i64>,
    pub size: Option<i64>,
}

/// QMP struct `BlockdevOptionsRbd`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.9.0")]
pub struct BlockdevOptionsRbd {
    #[qapi(name = "auth-client-required")]
    #[qapi(added = "3.0.0")]
    pub auth_client_required: Option<Vec<RbdAuthMode>>,
    pub conf: Option<String>,
    #[qapi(added = "6.1.0")]
    pub encrypt: Option<RbdEncryptionOptions>,
    pub image: String,
    #[qapi(name = "key-secret")]
    #[qapi(added = "3.0.0")]
    pub key_secret: Option<String>,
    #[qapi(added = "5.0.0")]
    pub namespace: Option<String>,
    pub pool: String,
    pub server: Option<Vec<InetSocketAddressBase>>,
    pub snapshot: Option<String>,
    pub user: Option<String>,
}

/// QMP struct `BlockdevOptionsReplication`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_REPLICATION")]
#[qapi(added = "2.8.0")]
pub struct BlockdevOptionsReplication {
    pub file: Box<BlockdevRef>,
    pub mode: ReplicationMode,
    #[qapi(name = "top-id")]
    pub top_id: Option<String>,
}

/// QMP struct `BlockdevOptionsSheepdog`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.9.0")]
#[qapi(removed = "6.1.0")]
pub struct BlockdevOptionsSheepdog {
    #[qapi(added = "2.10.0")]
    #[qapi(required)]
    pub server: Option<SocketAddress>,
    #[qapi(name = "server")]
    #[qapi(removed = "2.10.0")]
    #[qapi(required)]
    pub server_legacy: Option<SocketAddressFlat>,
    #[qapi(name = "snap-id")]
    pub snap_id: Option<u32>,
    pub tag: Option<String>,
    pub vdi: String,
}

/// QMP struct `BlockdevOptionsSsh`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.8.0")]
pub struct BlockdevOptionsSsh {
    #[qapi(name = "host-key-check")]
    #[qapi(added = "2.12.0")]
    pub host_key_check: Option<SshHostKeyCheck>,
    pub path: String,
    pub server: InetSocketAddress,
    pub user: Option<String>,
}

/// QMP struct `BlockdevOptionsThrottle`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.11.0")]
pub struct BlockdevOptionsThrottle {
    pub file: Box<BlockdevRef>,
    #[qapi(name = "throttle-group")]
    pub throttle_group: String,
}

/// QMP struct `BlockdevOptionsVirtioBlkVfioPci`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_BLKIO")]
#[qapi(added = "7.2.0")]
pub struct BlockdevOptionsVirtioBlkVfioPci {
    pub path: String,
}

/// QMP struct `BlockdevOptionsVirtioBlkVhostUser`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_BLKIO")]
#[qapi(added = "7.2.0")]
pub struct BlockdevOptionsVirtioBlkVhostUser {
    pub path: String,
}

/// QMP struct `BlockdevOptionsVirtioBlkVhostVdpa`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_BLKIO")]
#[qapi(added = "7.2.0")]
pub struct BlockdevOptionsVirtioBlkVhostVdpa {
    pub path: String,
}

/// QMP struct `BlockdevOptionsVVFAT`; see the QEMU QMP Reference Manual.
#[qapi(name = "BlockdevOptionsVVFAT")]
#[qapi(added = "1.7.0")]
pub struct BlockdevOptionsVvfat {
    pub dir: String,
    #[qapi(name = "fat-type")]
    pub fat_type: Option<i64>,
    pub floppy: Option<bool>,
    #[qapi(added = "2.4.0")]
    pub label: Option<String>,
    pub rw: Option<bool>,
}

/// QMP struct `BlockdevOptionsVxHS`; see the QEMU QMP Reference Manual.
#[qapi(name = "BlockdevOptionsVxHS")]
#[qapi(added = "2.10.0")]
#[qapi(removed = "5.1.0")]
pub struct BlockdevOptionsVxHs {
    pub server: InetSocketAddressBase,
    #[qapi(name = "tls-creds")]
    pub tls_creds: Option<String>,
    #[qapi(name = "vdisk-id")]
    pub vdisk_id: String,
}

/// QMP union `BlockdevQcow2Encryption`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.10.0")]
pub struct BlockdevQcow2Encryption {
    #[qapi(discriminator)]
    pub format: BlockdevQcow2EncryptionFormat,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<BlockdevQcow2EncryptionBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "2.10.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum BlockdevQcow2EncryptionBranch {
    #[qapi(name = "aes")]
    Aes(QCryptoBlockOptionsQCow),
    #[qapi(name = "luks")]
    Luks(QCryptoBlockOptionsLuks),
}

/// QMP enum `BlockdevQcow2EncryptionFormat`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.10.0")]
pub enum BlockdevQcow2EncryptionFormat {
    #[qapi(name = "aes")]
    Aes,
    #[qapi(name = "luks")]
    Luks,
}

/// QMP enum `BlockdevQcow2Version`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
pub enum BlockdevQcow2Version {
    #[qapi(name = "v2")]
    V2,
    #[qapi(name = "v3")]
    V3,
}

/// QMP union `BlockdevQcowEncryption`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.10.0")]
pub struct BlockdevQcowEncryption {
    #[qapi(discriminator)]
    pub format: BlockdevQcowEncryptionFormat,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<BlockdevQcowEncryptionBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "2.10.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum BlockdevQcowEncryptionBranch {
    #[qapi(name = "aes")]
    Aes(QCryptoBlockOptionsQCow),
}

/// QMP enum `BlockdevQcowEncryptionFormat`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.10.0")]
pub enum BlockdevQcowEncryptionFormat {
    #[qapi(name = "aes")]
    Aes,
}

/// QMP alternate `BlockdevRef`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.7.0")]
pub enum BlockdevRef {
    #[qapi(name = "definition")]
    Definition(BlockdevOptions),
    #[qapi(name = "reference")]
    Reference(String),
}

/// QMP alternate `BlockdevRefOrNull`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.10.0")]
pub enum BlockdevRefOrNull {
    #[qapi(name = "definition")]
    Definition(BlockdevOptions),
    #[qapi(name = "null")]
    Null(()),
    #[qapi(name = "reference")]
    Reference(String),
}

/// QMP struct `BlockdevSnapshot`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.1.0")]
pub struct BlockdevSnapshot {
    #[qapi(added = "2.0.0")]
    #[qapi(removed = "2.5.0")]
    pub device: Option<String>,
    #[qapi(name = "device")]
    #[qapi(removed = "2.0.0")]
    #[qapi(required)]
    pub device_legacy: Option<String>,
    #[qapi(removed = "2.5.0")]
    pub format: Option<String>,
    #[qapi(removed = "2.5.0")]
    pub mode: Option<NewImageMode>,
    #[qapi(added = "2.5.0")]
    #[qapi(required)]
    pub node: Option<String>,
    #[qapi(name = "node-name")]
    #[qapi(added = "2.0.0")]
    #[qapi(removed = "2.5.0")]
    pub node_name: Option<String>,
    #[qapi(added = "2.5.0")]
    #[qapi(required)]
    pub overlay: Option<String>,
    #[qapi(name = "snapshot-file")]
    #[qapi(removed = "2.5.0")]
    #[qapi(required)]
    pub snapshot_file: Option<String>,
    #[qapi(name = "snapshot-node-name")]
    #[qapi(added = "2.0.0")]
    #[qapi(removed = "2.5.0")]
    pub snapshot_node_name: Option<String>,
}

/// QMP struct `BlockdevSnapshotInternal`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.7.0")]
pub struct BlockdevSnapshotInternal {
    pub device: String,
    pub name: String,
}

/// QMP struct `BlockdevSnapshotInternalWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct BlockdevSnapshotInternalWrapper {
    pub data: BlockdevSnapshotInternal,
}

/// QMP struct `BlockdevSnapshotSyncWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct BlockdevSnapshotSyncWrapper {
    pub data: BlockdevSnapshotSync,
}

/// QMP struct `BlockdevSnapshotWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct BlockdevSnapshotWrapper {
    pub data: BlockdevSnapshot,
}

/// QMP enum `BlockdevVhdxSubformat`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
pub enum BlockdevVhdxSubformat {
    #[qapi(name = "dynamic")]
    Dynamic,
    #[qapi(name = "fixed")]
    Fixed,
}

/// QMP enum `BlockdevVmdkAdapterType`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.0.0")]
pub enum BlockdevVmdkAdapterType {
    #[qapi(name = "buslogic")]
    Buslogic,
    #[qapi(name = "ide")]
    Ide,
    #[qapi(name = "legacyESX")]
    LegacyEsx,
    #[qapi(name = "lsilogic")]
    Lsilogic,
}

/// QMP enum `BlockdevVmdkSubformat`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.0.0")]
pub enum BlockdevVmdkSubformat {
    #[qapi(name = "monolithicFlat")]
    MonolithicFlat,
    #[qapi(name = "monolithicSparse")]
    MonolithicSparse,
    #[qapi(name = "streamOptimized")]
    StreamOptimized,
    #[qapi(name = "twoGbMaxExtentFlat")]
    TwoGbMaxExtentFlat,
    #[qapi(name = "twoGbMaxExtentSparse")]
    TwoGbMaxExtentSparse,
}

/// QMP enum `BlockdevVpcSubformat`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
pub enum BlockdevVpcSubformat {
    #[qapi(name = "dynamic")]
    Dynamic,
    #[qapi(name = "fixed")]
    Fixed,
}

/// QMP struct `BlockDirtyBitmap`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.4.0")]
pub struct BlockDirtyBitmap {
    pub name: String,
    pub node: String,
}

/// QMP struct `BlockDirtyBitmapAddWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct BlockDirtyBitmapAddWrapper {
    pub data: BlockDirtyBitmapAdd,
}

/// QMP struct `BlockDirtyBitmapMerge`; see the QEMU QMP Reference Manual.
#[qapi(added = "3.0.0")]
pub struct BlockDirtyBitmapMerge {
    #[qapi(added = "7.1.0")]
    #[qapi(required)]
    pub bitmaps: Option<Vec<BlockDirtyBitmapOrStr>>,
    #[qapi(name = "bitmaps")]
    #[qapi(added = "4.0.0")]
    #[qapi(removed = "4.1.0")]
    #[qapi(required)]
    pub bitmaps_legacy: Option<Vec<String>>,
    #[qapi(name = "bitmaps")]
    #[qapi(added = "4.1.0")]
    #[qapi(removed = "7.1.0")]
    #[qapi(required)]
    pub bitmaps_legacy2: Option<Vec<BlockDirtyBitmapMergeSource>>,
    #[qapi(removed = "4.0.0")]
    #[qapi(required)]
    pub dst_name: Option<String>,
    pub node: String,
    #[qapi(removed = "4.0.0")]
    #[qapi(required)]
    pub src_name: Option<String>,
    #[qapi(added = "4.0.0")]
    #[qapi(required)]
    pub target: Option<String>,
}

/// QMP alternate `BlockDirtyBitmapMergeSource`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.1.0")]
#[qapi(removed = "7.1.0")]
pub enum BlockDirtyBitmapMergeSource {
    #[qapi(name = "external")]
    External(BlockDirtyBitmap),
    #[qapi(name = "local")]
    Local(String),
}

/// QMP struct `BlockDirtyBitmapMergeWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct BlockDirtyBitmapMergeWrapper {
    pub data: BlockDirtyBitmapMerge,
}

/// QMP alternate `BlockDirtyBitmapOrStr`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.1.0")]
pub enum BlockDirtyBitmapOrStr {
    #[qapi(name = "external")]
    External(BlockDirtyBitmap),
    #[qapi(name = "local")]
    Local(String),
}

/// QMP struct `BlockDirtyBitmapSha256`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.10.0")]
pub struct BlockDirtyBitmapSha256 {
    pub sha256: String,
}

/// QMP struct `BlockDirtyBitmapWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct BlockDirtyBitmapWrapper {
    pub data: BlockDirtyBitmap,
}

/// QMP struct `BlockDirtyInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.3.0")]
pub struct BlockDirtyInfo {
    #[qapi(added = "4.0.0")]
    #[qapi(required)]
    pub busy: Option<bool>,
    pub count: i64,
    #[qapi(added = "2.4.0")]
    #[qapi(required)]
    pub granularity: Option<u32>,
    #[qapi(name = "granularity")]
    #[qapi(added = "1.4.0")]
    #[qapi(removed = "2.4.0")]
    #[qapi(required)]
    pub granularity_legacy: Option<i64>,
    #[qapi(added = "4.0.0")]
    pub inconsistent: Option<bool>,
    #[qapi(added = "2.4.0")]
    pub name: Option<String>,
    #[qapi(added = "4.0.0")]
    #[qapi(required)]
    pub persistent: Option<bool>,
    #[qapi(added = "4.0.0")]
    #[qapi(required)]
    pub recording: Option<bool>,
    #[qapi(added = "2.4.0")]
    #[qapi(deprecated(added = "5.0.0"))]
    #[qapi(removed = "6.0.0")]
    #[qapi(required)]
    pub status: Option<DirtyBitmapStatus>,
}

/// QMP enum `BlockErrorAction`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.1.0")]
pub enum BlockErrorAction {
    #[qapi(name = "ignore")]
    Ignore,
    #[qapi(name = "report")]
    Report,
    #[qapi(name = "stop")]
    Stop,
}

/// QMP union `BlockExport`; see the QEMU QMP Reference Manual.
#[qapi(added = "5.0.0")]
#[qapi(removed = "5.2.0")]
pub struct BlockExport {
    #[qapi(discriminator)]
    pub r#type: BlockExportType,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<BlockExportBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "5.0.0")]
#[qapi(removed = "5.2.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum BlockExportBranch {
    #[qapi(name = "nbd")]
    Nbd(BlockExportNbd),
}

/// QMP struct `BlockExportInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "5.2.0")]
pub struct BlockExportInfo {
    pub id: String,
    #[qapi(name = "node-name")]
    pub node_name: String,
    #[qapi(name = "shutting-down")]
    pub shutting_down: bool,
    pub r#type: BlockExportType,
}

/// QMP alternate `BlockExportIothreads`; see the QEMU QMP Reference Manual.
#[qapi(added = "11.0.0")]
pub enum BlockExportIothreads {
    #[qapi(name = "multi")]
    Multi(Vec<String>),
    #[qapi(name = "single")]
    Single(String),
}

/// QMP struct `BlockExportNbd`; see the QEMU QMP Reference Manual.
#[qapi(added = "5.0.0")]
#[qapi(removed = "5.2.0")]
pub struct BlockExportNbd {
    pub bitmap: Option<String>,
    pub description: Option<String>,
    pub device: String,
    pub name: Option<String>,
    pub writable: Option<bool>,
}

/// QMP union `BlockExportOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "5.2.0")]
pub struct BlockExportOptions {
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

#[qapi(union_branch)]
#[qapi(added = "5.2.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum BlockExportOptionsBranch {
    #[qapi(name = "fuse")]
    #[qapi(condition = "CONFIG_FUSE")]
    #[qapi(added = "6.0.0")]
    Fuse(BlockExportOptionsFuse),
    #[qapi(name = "nbd")]
    Nbd(BlockExportOptionsNbd),
    #[qapi(name = "vduse-blk")]
    #[qapi(condition = "CONFIG_VDUSE_BLK_EXPORT")]
    #[qapi(added = "7.1.0")]
    VduseBlk(BlockExportOptionsVduseBlk),
    #[qapi(name = "vhost-user-blk")]
    #[qapi(condition = "CONFIG_VHOST_USER_BLK_SERVER")]
    VhostUserBlk(BlockExportOptionsVhostUserBlk),
}

/// QMP struct `BlockExportOptionsFuse`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_FUSE")]
#[qapi(added = "6.0.0")]
pub struct BlockExportOptionsFuse {
    #[qapi(name = "allow-other")]
    #[qapi(added = "6.1.0")]
    pub allow_other: Option<FuseExportAllowOther>,
    pub growable: Option<bool>,
    pub mountpoint: String,
}

/// QMP struct `BlockExportOptionsNbd`; see the QEMU QMP Reference Manual.
#[qapi(added = "5.2.0")]
pub struct BlockExportOptionsNbd {
    #[qapi(name = "allocation-depth")]
    pub allocation_depth: Option<bool>,
    #[qapi(added = "7.1.0")]
    pub bitmaps: Option<Vec<BlockDirtyBitmapOrStr>>,
    #[qapi(name = "bitmaps")]
    #[qapi(removed = "7.1.0")]
    pub bitmaps_legacy: Option<Vec<String>>,
    pub description: Option<String>,
    pub name: Option<String>,
}

/// QMP struct `BlockExportOptionsNbdBase`; see the QEMU QMP Reference Manual.
#[qapi(added = "5.2.0")]
pub struct BlockExportOptionsNbdBase {
    pub description: Option<String>,
    pub name: Option<String>,
}

/// QMP struct `BlockExportOptionsVduseBlk`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.1.0")]
pub struct BlockExportOptionsVduseBlk {
    #[qapi(name = "logical-block-size")]
    pub logical_block_size: Option<u64>,
    pub name: String,
    #[qapi(name = "num-queues")]
    pub num_queues: Option<u16>,
    #[qapi(name = "queue-size")]
    pub queue_size: Option<u16>,
    pub serial: Option<String>,
}

/// QMP struct `BlockExportOptionsVhostUserBlk`; see the QEMU QMP Reference Manual.
#[qapi(added = "5.2.0")]
pub struct BlockExportOptionsVhostUserBlk {
    pub addr: SocketAddress,
    #[qapi(name = "logical-block-size")]
    pub logical_block_size: Option<u64>,
    #[qapi(name = "num-queues")]
    pub num_queues: Option<u16>,
}

/// QMP enum `BlockExportRemoveMode`; see the QEMU QMP Reference Manual.
#[qapi(added = "5.2.0")]
pub enum BlockExportRemoveMode {
    #[qapi(name = "hard")]
    Hard,
    #[qapi(name = "safe")]
    Safe,
}

/// QMP enum `BlockExportType`; see the QEMU QMP Reference Manual.
#[qapi(added = "5.0.0")]
pub enum BlockExportType {
    #[qapi(name = "fuse")]
    #[qapi(condition = "CONFIG_FUSE")]
    #[qapi(added = "6.0.0")]
    Fuse,
    #[qapi(name = "nbd")]
    Nbd,
    #[qapi(name = "vduse-blk")]
    #[qapi(condition = "CONFIG_VDUSE_BLK_EXPORT")]
    #[qapi(added = "7.1.0")]
    VduseBlk,
    #[qapi(name = "vhost-user-blk")]
    #[qapi(condition = "CONFIG_VHOST_USER_BLK_SERVER")]
    #[qapi(added = "5.2.0")]
    VhostUserBlk,
}

/// QMP struct `BlockGraphInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.0.0")]
pub struct BlockGraphInfo {
    #[qapi(name = "actual-size")]
    pub actual_size: Option<i64>,
    #[qapi(name = "backing-filename")]
    pub backing_filename: Option<String>,
    #[qapi(name = "backing-filename-format")]
    pub backing_filename_format: Option<String>,
    pub children: Vec<BlockChildInfo>,
    #[qapi(name = "cluster-size")]
    pub cluster_size: Option<i64>,
    pub compressed: Option<bool>,
    #[qapi(name = "dirty-flag")]
    pub dirty_flag: Option<bool>,
    pub encrypted: Option<bool>,
    pub filename: String,
    pub format: String,
    #[qapi(name = "format-specific")]
    pub format_specific: Option<ImageInfoSpecific>,
    #[qapi(name = "full-backing-filename")]
    pub full_backing_filename: Option<String>,
    #[qapi(added = "10.2.0")]
    pub limits: Option<BlockLimitsInfo>,
    pub snapshots: Option<Vec<SnapshotInfo>>,
    #[qapi(name = "virtual-size")]
    pub virtual_size: i64,
}

/// QMP struct `BlockInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.0.0")]
pub struct BlockInfo {
    pub device: String,
    #[qapi(added = "1.3.0")]
    #[qapi(removed = "2.0.0")]
    pub dirty: Option<BlockDirtyInfo>,
    #[qapi(name = "dirty-bitmaps")]
    #[qapi(added = "2.0.0")]
    #[qapi(deprecated(added = "5.0.0"))]
    #[qapi(removed = "6.0.0")]
    pub dirty_bitmaps: Option<Vec<BlockDirtyInfo>>,
    pub inserted: Option<BlockDeviceInfo>,
    #[qapi(name = "io-status")]
    pub io_status: Option<BlockDeviceIoStatus>,
    pub locked: bool,
    #[qapi(added = "2.10.0")]
    pub qdev: Option<String>,
    pub removable: bool,
    pub tray_open: Option<bool>,
    pub r#type: String,
}

/// QMP struct `BlockIOThrottle`; see the QEMU QMP Reference Manual.
#[qapi(name = "BlockIOThrottle")]
#[qapi(added = "2.7.0")]
pub struct BlockIoThrottle {
    pub bps: i64,
    pub bps_max: Option<i64>,
    pub bps_max_length: Option<i64>,
    pub bps_rd: i64,
    pub bps_rd_max: Option<i64>,
    pub bps_rd_max_length: Option<i64>,
    pub bps_wr: i64,
    pub bps_wr_max: Option<i64>,
    pub bps_wr_max_length: Option<i64>,
    #[qapi(added = "2.8.0")]
    #[qapi(deprecated(added = "5.0.0"))]
    pub device: Option<String>,
    #[qapi(name = "device")]
    #[qapi(removed = "2.8.0")]
    #[qapi(required)]
    pub device_legacy: Option<String>,
    pub group: Option<String>,
    #[qapi(added = "2.8.0")]
    pub id: Option<String>,
    pub iops: i64,
    pub iops_max: Option<i64>,
    pub iops_max_length: Option<i64>,
    pub iops_rd: i64,
    pub iops_rd_max: Option<i64>,
    pub iops_rd_max_length: Option<i64>,
    pub iops_size: Option<i64>,
    pub iops_wr: i64,
    pub iops_wr_max: Option<i64>,
    pub iops_wr_max_length: Option<i64>,
}

/// QMP union `BlockJobChangeOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.2.0")]
pub struct BlockJobChangeOptions {
    pub id: String,
    #[qapi(discriminator)]
    pub r#type: JobType,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<BlockJobChangeOptionsBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "8.2.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum BlockJobChangeOptionsBranch {
    #[qapi(name = "amend")]
    Amend,
    #[qapi(name = "backup")]
    Backup,
    #[qapi(name = "commit")]
    Commit,
    #[qapi(name = "create")]
    Create,
    #[qapi(name = "mirror")]
    Mirror(BlockJobChangeOptionsMirror),
    #[qapi(name = "snapshot-delete")]
    SnapshotDelete,
    #[qapi(name = "snapshot-load")]
    SnapshotLoad,
    #[qapi(name = "snapshot-save")]
    SnapshotSave,
    #[qapi(name = "stream")]
    Stream,
}

/// QMP struct `BlockJobChangeOptionsMirror`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.2.0")]
pub struct BlockJobChangeOptionsMirror {
    #[qapi(name = "copy-mode")]
    pub copy_mode: MirrorCopyMode,
}

/// QMP union `BlockJobInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.1.0")]
pub struct BlockJobInfo {
    #[qapi(name = "auto-dismiss")]
    #[qapi(added = "2.12.0")]
    #[qapi(required)]
    pub auto_dismiss: Option<bool>,
    #[qapi(name = "auto-finalize")]
    #[qapi(added = "2.12.0")]
    #[qapi(required)]
    pub auto_finalize: Option<bool>,
    #[qapi(added = "1.3.0")]
    #[qapi(required)]
    pub busy: Option<bool>,
    pub device: String,
    #[qapi(added = "2.12.1")]
    pub error: Option<String>,
    #[qapi(name = "io-status")]
    #[qapi(added = "1.3.0")]
    #[qapi(required)]
    pub io_status: Option<BlockDeviceIoStatus>,
    pub len: i64,
    pub offset: i64,
    #[qapi(added = "1.3.0")]
    #[qapi(required)]
    pub paused: Option<bool>,
    #[qapi(added = "2.2.0")]
    #[qapi(required)]
    pub ready: Option<bool>,
    pub speed: i64,
    #[qapi(added = "3.0.0")]
    #[qapi(required)]
    pub status: Option<JobStatus>,
    #[qapi(name = "status")]
    #[qapi(added = "2.12.0")]
    #[qapi(removed = "3.0.0")]
    #[qapi(required)]
    pub status_legacy: Option<BlockJobStatus>,
    #[qapi(discriminator)]
    #[qapi(added = "8.2.0")]
    #[qapi(required)]
    pub r#type: Option<JobType>,
    #[qapi(name = "type")]
    #[qapi(removed = "8.2.0")]
    #[qapi(required)]
    pub type_legacy: Option<String>,
    #[qapi(union)]
    #[qapi(flatten)]
    #[qapi(added = "8.2.0")]
    pub u: Option<BlockJobInfoBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "8.2.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum BlockJobInfoBranch {
    #[qapi(name = "amend")]
    Amend,
    #[qapi(name = "backup")]
    Backup,
    #[qapi(name = "commit")]
    Commit,
    #[qapi(name = "create")]
    Create,
    #[qapi(name = "mirror")]
    Mirror(BlockJobInfoMirror),
    #[qapi(name = "snapshot-delete")]
    SnapshotDelete,
    #[qapi(name = "snapshot-load")]
    SnapshotLoad,
    #[qapi(name = "snapshot-save")]
    SnapshotSave,
    #[qapi(name = "stream")]
    Stream,
}

/// QMP struct `BlockJobInfoMirror`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.2.0")]
pub struct BlockJobInfoMirror {
    #[qapi(name = "actively-synced")]
    pub actively_synced: bool,
}

/// QMP enum `BlockJobStatus`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
#[qapi(removed = "3.0.0")]
pub enum BlockJobStatus {
    #[qapi(name = "aborting")]
    Aborting,
    #[qapi(name = "concluded")]
    Concluded,
    #[qapi(name = "created")]
    Created,
    #[qapi(name = "null")]
    Null,
    #[qapi(name = "paused")]
    Paused,
    #[qapi(name = "pending")]
    Pending,
    #[qapi(name = "ready")]
    Ready,
    #[qapi(name = "running")]
    Running,
    #[qapi(name = "standby")]
    Standby,
    #[qapi(name = "undefined")]
    Undefined,
    #[qapi(name = "waiting")]
    Waiting,
}

/// QMP enum `BlockJobType`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.7.0")]
#[qapi(removed = "3.0.0")]
pub enum BlockJobType {
    #[qapi(name = "backup")]
    Backup,
    #[qapi(name = "commit")]
    Commit,
    #[qapi(name = "mirror")]
    Mirror,
    #[qapi(name = "stream")]
    Stream,
}

/// QMP enum `BlockJobVerb`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
#[qapi(removed = "3.0.0")]
pub enum BlockJobVerb {
    #[qapi(name = "cancel")]
    Cancel,
    #[qapi(name = "complete")]
    Complete,
    #[qapi(name = "dismiss")]
    Dismiss,
    #[qapi(name = "finalize")]
    Finalize,
    #[qapi(name = "pause")]
    Pause,
    #[qapi(name = "resume")]
    Resume,
    #[qapi(name = "set-speed")]
    SetSpeed,
}

/// QMP struct `BlockLatencyHistogramInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
pub struct BlockLatencyHistogramInfo {
    pub bins: Vec<u64>,
    pub boundaries: Vec<u64>,
}

/// QMP struct `BlockLimitsInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "10.2.0")]
pub struct BlockLimitsInfo {
    #[qapi(name = "discard-alignment")]
    pub discard_alignment: Option<u32>,
    #[qapi(name = "max-discard")]
    pub max_discard: Option<u64>,
    #[qapi(name = "max-hw-iov")]
    pub max_hw_iov: Option<i64>,
    #[qapi(name = "max-hw-transfer")]
    pub max_hw_transfer: Option<u32>,
    #[qapi(name = "max-iov")]
    pub max_iov: i64,
    #[qapi(name = "max-transfer")]
    pub max_transfer: Option<u32>,
    #[qapi(name = "max-write-zeroes")]
    pub max_write_zeroes: Option<u64>,
    #[qapi(name = "min-mem-alignment")]
    pub min_mem_alignment: u64,
    #[qapi(name = "opt-mem-alignment")]
    pub opt_mem_alignment: u64,
    #[qapi(name = "opt-transfer")]
    pub opt_transfer: Option<u32>,
    #[qapi(name = "request-alignment")]
    pub request_alignment: u32,
    #[qapi(name = "write-zeroes-alignment")]
    pub write_zeroes_alignment: Option<u32>,
}

/// QMP struct `BlockMeasureInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.10.0")]
pub struct BlockMeasureInfo {
    #[qapi(added = "5.1.0")]
    pub bitmaps: Option<i64>,
    #[qapi(name = "fully-allocated")]
    pub fully_allocated: i64,
    pub required: i64,
}

/// QMP struct `BlockNodeInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.0.0")]
pub struct BlockNodeInfo {
    #[qapi(name = "actual-size")]
    pub actual_size: Option<i64>,
    #[qapi(name = "backing-filename")]
    pub backing_filename: Option<String>,
    #[qapi(name = "backing-filename-format")]
    pub backing_filename_format: Option<String>,
    #[qapi(name = "cluster-size")]
    pub cluster_size: Option<i64>,
    pub compressed: Option<bool>,
    #[qapi(name = "dirty-flag")]
    pub dirty_flag: Option<bool>,
    pub encrypted: Option<bool>,
    pub filename: String,
    pub format: String,
    #[qapi(name = "format-specific")]
    pub format_specific: Option<ImageInfoSpecific>,
    #[qapi(name = "full-backing-filename")]
    pub full_backing_filename: Option<String>,
    #[qapi(added = "10.2.0")]
    pub limits: Option<BlockLimitsInfo>,
    pub snapshots: Option<Vec<SnapshotInfo>>,
    #[qapi(name = "virtual-size")]
    pub virtual_size: i64,
}

/// QMP enum `BlockPermission`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.0.0")]
pub enum BlockPermission {
    #[qapi(name = "consistent-read")]
    ConsistentRead,
    #[qapi(name = "graph-mod")]
    #[qapi(removed = "7.0.0")]
    GraphMod,
    #[qapi(name = "resize")]
    Resize,
    #[qapi(name = "write")]
    Write,
    #[qapi(name = "write-unchanged")]
    WriteUnchanged,
}

/// QMP struct `BlockStats`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.0.0")]
pub struct BlockStats {
    #[qapi(added = "2.0.0")]
    pub backing: Option<Box<BlockStats>>,
    pub device: Option<String>,
    #[qapi(name = "driver-specific")]
    #[qapi(added = "4.2.0")]
    pub driver_specific: Option<BlockStatsSpecific>,
    #[qapi(name = "node-name")]
    #[qapi(added = "2.3.0")]
    pub node_name: Option<String>,
    pub parent: Option<Box<BlockStats>>,
    #[qapi(added = "3.0.0")]
    pub qdev: Option<String>,
    pub stats: BlockDeviceStats,
}

/// QMP union `BlockStatsSpecific`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.2.0")]
pub struct BlockStatsSpecific {
    #[qapi(discriminator)]
    pub driver: BlockdevDriver,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<BlockStatsSpecificBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "4.2.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum BlockStatsSpecificBranch {
    #[qapi(name = "blkdebug")]
    Blkdebug,
    #[qapi(name = "blklogwrites")]
    Blklogwrites,
    #[qapi(name = "blkreplay")]
    Blkreplay,
    #[qapi(name = "blkverify")]
    Blkverify,
    #[qapi(name = "bochs")]
    Bochs,
    #[qapi(name = "cloop")]
    Cloop,
    #[qapi(name = "compress")]
    #[qapi(added = "5.0.0")]
    Compress,
    #[qapi(name = "copy-before-write")]
    #[qapi(added = "6.2.0")]
    CopyBeforeWrite,
    #[qapi(name = "copy-on-read")]
    CopyOnRead,
    #[qapi(name = "dmg")]
    Dmg,
    #[qapi(name = "file")]
    File(BlockStatsSpecificFile),
    #[qapi(name = "ftp")]
    Ftp,
    #[qapi(name = "ftps")]
    Ftps,
    #[qapi(name = "gluster")]
    #[qapi(deprecated(added = "9.2.0"))]
    #[qapi(removed = "11.1.0")]
    Gluster,
    #[qapi(name = "host_cdrom")]
    #[qapi(condition = "HAVE_HOST_BLOCK_DEVICE")]
    HostCdrom,
    #[qapi(name = "host_device")]
    #[qapi(condition = "HAVE_HOST_BLOCK_DEVICE")]
    HostDevice(BlockStatsSpecificFile),
    #[qapi(name = "http")]
    Http,
    #[qapi(name = "https")]
    Https,
    #[qapi(name = "io_uring")]
    #[qapi(condition = "CONFIG_BLKIO")]
    #[qapi(added = "7.2.0")]
    IoUring,
    #[qapi(name = "iscsi")]
    Iscsi,
    #[qapi(name = "luks")]
    Luks,
    #[qapi(name = "nbd")]
    Nbd,
    #[qapi(name = "nfs")]
    Nfs,
    #[qapi(name = "null-aio")]
    NullAio,
    #[qapi(name = "null-co")]
    NullCo,
    #[qapi(name = "nvme")]
    #[qapi(added = "5.2.0")]
    Nvme(BlockStatsSpecificNvme),
    #[qapi(name = "nvme-io_uring")]
    #[qapi(condition = "CONFIG_BLKIO")]
    #[qapi(added = "7.2.0")]
    NvmeIoUring,
    #[qapi(name = "nvme")]
    #[qapi(removed = "5.2.0")]
    NvmeLegacy,
    #[qapi(name = "parallels")]
    Parallels,
    #[qapi(name = "preallocate")]
    #[qapi(added = "6.0.0")]
    Preallocate,
    #[qapi(name = "qcow")]
    Qcow,
    #[qapi(name = "qcow2")]
    Qcow2,
    #[qapi(name = "qed")]
    Qed,
    #[qapi(name = "quorum")]
    Quorum,
    #[qapi(name = "raw")]
    Raw,
    #[qapi(name = "rbd")]
    Rbd,
    #[qapi(name = "replication")]
    #[qapi(condition = "CONFIG_REPLICATION")]
    Replication,
    #[qapi(name = "sheepdog")]
    #[qapi(removed = "6.1.0")]
    Sheepdog,
    #[qapi(name = "snapshot-access")]
    #[qapi(added = "7.0.0")]
    SnapshotAccess,
    #[qapi(name = "ssh")]
    Ssh,
    #[qapi(name = "throttle")]
    Throttle,
    #[qapi(name = "vdi")]
    Vdi,
    #[qapi(name = "vhdx")]
    Vhdx,
    #[qapi(name = "virtio-blk-vfio-pci")]
    #[qapi(condition = "CONFIG_BLKIO")]
    #[qapi(added = "7.2.0")]
    VirtioBlkVfioPci,
    #[qapi(name = "virtio-blk-vhost-user")]
    #[qapi(condition = "CONFIG_BLKIO")]
    #[qapi(added = "7.2.0")]
    VirtioBlkVhostUser,
    #[qapi(name = "virtio-blk-vhost-vdpa")]
    #[qapi(condition = "CONFIG_BLKIO")]
    #[qapi(added = "7.2.0")]
    VirtioBlkVhostVdpa,
    #[qapi(name = "vmdk")]
    Vmdk,
    #[qapi(name = "vpc")]
    Vpc,
    #[qapi(name = "vvfat")]
    Vvfat,
    #[qapi(name = "vxhs")]
    #[qapi(removed = "5.1.0")]
    Vxhs,
}

/// QMP struct `BlockStatsSpecificFile`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.2.0")]
pub struct BlockStatsSpecificFile {
    #[qapi(name = "discard-bytes-ok")]
    pub discard_bytes_ok: u64,
    #[qapi(name = "discard-nb-failed")]
    pub discard_nb_failed: u64,
    #[qapi(name = "discard-nb-ok")]
    pub discard_nb_ok: u64,
}

/// QMP struct `BlockStatsSpecificNvme`; see the QEMU QMP Reference Manual.
#[qapi(added = "5.2.0")]
pub struct BlockStatsSpecificNvme {
    #[qapi(name = "aligned-accesses")]
    pub aligned_accesses: u64,
    #[qapi(name = "completion-errors")]
    pub completion_errors: u64,
    #[qapi(name = "unaligned-accesses")]
    pub unaligned_accesses: u64,
}

/// QMP struct `BootConfiguration`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.1.0")]
pub struct BootConfiguration {
    pub menu: Option<bool>,
    pub once: Option<String>,
    pub order: Option<String>,
    #[qapi(name = "reboot-timeout")]
    pub reboot_timeout: Option<i64>,
    pub splash: Option<String>,
    #[qapi(name = "splash-time")]
    pub splash_time: Option<i64>,
    pub strict: Option<bool>,
}

/// QMP enum `CacheLevelAndType`; see the QEMU QMP Reference Manual.
#[qapi(added = "9.2.0")]
pub enum CacheLevelAndType {
    #[qapi(name = "l1d")]
    L1d,
    #[qapi(name = "l1i")]
    L1i,
    #[qapi(name = "l2")]
    L2,
    #[qapi(name = "l3")]
    L3,
}

/// QMP struct `CanHostSocketcanProperties`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_LINUX")]
#[qapi(added = "6.0.0")]
pub struct CanHostSocketcanProperties {
    pub canbus: String,
    pub r#if: String,
}

/// QMP union `ChardevBackend`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.4.0")]
#[qapi(simple_union(removed = "6.2.0"))]
pub struct ChardevBackend {
    #[qapi(discriminator)]
    #[qapi(name = "type")]
    #[qapi(removed = "6.2.0")]
    #[qapi(required)]
    pub r_type: Option<ChardevBackendKind>,
    #[qapi(discriminator)]
    #[qapi(added = "6.2.0")]
    #[qapi(required)]
    pub r#type: Option<ChardevBackendKind>,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<ChardevBackendBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "1.4.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum ChardevBackendBranch {
    #[qapi(name = "braille")]
    #[qapi(condition = "CONFIG_BRLAPI")]
    #[qapi(added = "6.2.0")]
    Braille(ChardevCommonWrapper),
    #[qapi(name = "braille")]
    #[qapi(added = "1.5.0")]
    #[qapi(removed = "2.6.0")]
    BrailleLegacy(ChardevDummy),
    #[qapi(name = "braille")]
    #[qapi(added = "2.6.0")]
    #[qapi(removed = "6.2.0")]
    BrailleLegacy2(ChardevCommon),
    #[qapi(name = "console")]
    #[qapi(condition = "CONFIG_WIN32")]
    #[qapi(added = "6.2.0")]
    Console(ChardevCommonWrapper),
    #[qapi(name = "console")]
    #[qapi(added = "1.5.0")]
    #[qapi(removed = "2.6.0")]
    ConsoleLegacy(ChardevDummy),
    #[qapi(name = "console")]
    #[qapi(added = "2.6.0")]
    #[qapi(removed = "6.2.0")]
    ConsoleLegacy2(ChardevCommon),
    #[qapi(name = "dbus")]
    #[qapi(condition = "CONFIG_DBUS_DISPLAY")]
    #[qapi(added = "7.0.0")]
    Dbus(ChardevDBusWrapper),
    #[qapi(name = "file")]
    #[qapi(added = "6.2.0")]
    File(ChardevFileWrapper),
    #[qapi(name = "file")]
    #[qapi(removed = "6.2.0")]
    FileLegacy(ChardevFile),
    #[qapi(name = "hub")]
    #[qapi(added = "10.0.0")]
    Hub(ChardevHubWrapper),
    #[qapi(name = "memory")]
    #[qapi(added = "6.2.0")]
    Memory(ChardevRingbufWrapper),
    #[qapi(name = "memory")]
    #[qapi(added = "1.5.0")]
    #[qapi(removed = "6.2.0")]
    MemoryLegacy(ChardevRingbuf),
    #[qapi(name = "msmouse")]
    #[qapi(added = "6.2.0")]
    Msmouse(ChardevCommonWrapper),
    #[qapi(name = "msmouse")]
    #[qapi(added = "1.5.0")]
    #[qapi(removed = "2.6.0")]
    MsmouseLegacy(ChardevDummy),
    #[qapi(name = "msmouse")]
    #[qapi(added = "2.6.0")]
    #[qapi(removed = "6.2.0")]
    MsmouseLegacy2(ChardevCommon),
    #[qapi(name = "mux")]
    #[qapi(added = "6.2.0")]
    Mux(ChardevMuxWrapper),
    #[qapi(name = "mux")]
    #[qapi(added = "1.5.0")]
    #[qapi(removed = "6.2.0")]
    MuxLegacy(ChardevMux),
    #[qapi(name = "null")]
    #[qapi(added = "6.2.0")]
    Null(ChardevCommonWrapper),
    #[qapi(name = "null")]
    #[qapi(removed = "2.6.0")]
    NullLegacy(ChardevDummy),
    #[qapi(name = "null")]
    #[qapi(added = "2.6.0")]
    #[qapi(removed = "6.2.0")]
    NullLegacy2(ChardevCommon),
    #[qapi(name = "parallel")]
    #[qapi(condition = "HAVE_CHARDEV_PARALLEL")]
    #[qapi(added = "6.2.0")]
    Parallel(ChardevHostdevWrapper),
    #[qapi(name = "parallel")]
    #[qapi(removed = "6.2.0")]
    ParallelLegacy(ChardevHostdev),
    #[qapi(name = "pipe")]
    #[qapi(added = "6.2.0")]
    Pipe(ChardevHostdevWrapper),
    #[qapi(name = "pipe")]
    #[qapi(added = "1.5.0")]
    #[qapi(removed = "6.2.0")]
    PipeLegacy(ChardevHostdev),
    #[qapi(name = "pty")]
    #[qapi(added = "9.2.0")]
    Pty(ChardevPtyWrapper),
    #[qapi(name = "pty")]
    #[qapi(removed = "2.6.0")]
    PtyLegacy(ChardevDummy),
    #[qapi(name = "pty")]
    #[qapi(added = "2.6.0")]
    #[qapi(removed = "6.2.0")]
    PtyLegacy2(ChardevCommon),
    #[qapi(name = "pty")]
    #[qapi(added = "6.2.0")]
    #[qapi(removed = "9.2.0")]
    PtyLegacy3(ChardevCommonWrapper),
    #[qapi(name = "qemu-vdagent")]
    #[qapi(condition = "CONFIG_SPICE_PROTOCOL")]
    #[qapi(added = "6.2.0")]
    QemuVdagent(ChardevQemuVdAgentWrapper),
    #[qapi(condition = "defined(CONFIG_SPICE_PROTOCOL)")]
    #[qapi(name = "qemu-vdagent")]
    #[qapi(added = "6.1.0")]
    #[qapi(removed = "6.2.0")]
    QemuVdagentLegacy(ChardevQemuVdAgent),
    #[qapi(name = "ringbuf")]
    #[qapi(added = "6.2.0")]
    Ringbuf(ChardevRingbufWrapper),
    #[qapi(name = "ringbuf")]
    #[qapi(added = "1.5.3")]
    #[qapi(removed = "6.2.0")]
    RingbufLegacy(ChardevRingbuf),
    #[qapi(name = "serial")]
    #[qapi(condition = "HAVE_CHARDEV_SERIAL")]
    #[qapi(added = "6.2.0")]
    Serial(ChardevHostdevWrapper),
    #[qapi(name = "serial")]
    #[qapi(removed = "6.2.0")]
    SerialLegacy(ChardevHostdev),
    #[qapi(name = "socket")]
    #[qapi(added = "6.2.0")]
    Socket(ChardevSocketWrapper),
    #[qapi(name = "socket")]
    #[qapi(removed = "6.2.0")]
    SocketLegacy(ChardevSocket),
    #[qapi(name = "spiceport")]
    #[qapi(condition = "CONFIG_SPICE")]
    #[qapi(added = "6.2.0")]
    Spiceport(ChardevSpicePortWrapper),
    #[qapi(condition = "defined(CONFIG_SPICE)")]
    #[qapi(name = "spiceport")]
    #[qapi(added = "1.5.0")]
    #[qapi(removed = "6.2.0")]
    SpiceportLegacy(ChardevSpicePort),
    #[qapi(name = "spicevmc")]
    #[qapi(condition = "CONFIG_SPICE")]
    #[qapi(added = "6.2.0")]
    Spicevmc(ChardevSpiceChannelWrapper),
    #[qapi(condition = "defined(CONFIG_SPICE)")]
    #[qapi(name = "spicevmc")]
    #[qapi(added = "1.5.0")]
    #[qapi(removed = "6.2.0")]
    SpicevmcLegacy(ChardevSpiceChannel),
    #[qapi(name = "stdio")]
    #[qapi(added = "6.2.0")]
    Stdio(ChardevStdioWrapper),
    #[qapi(name = "stdio")]
    #[qapi(added = "1.5.0")]
    #[qapi(removed = "6.2.0")]
    StdioLegacy(ChardevStdio),
    #[qapi(name = "testdev")]
    #[qapi(added = "6.2.0")]
    Testdev(ChardevCommonWrapper),
    #[qapi(name = "testdev")]
    #[qapi(added = "2.2.0")]
    #[qapi(removed = "2.6.0")]
    TestdevLegacy(ChardevDummy),
    #[qapi(name = "testdev")]
    #[qapi(added = "2.6.0")]
    #[qapi(removed = "6.2.0")]
    TestdevLegacy2(ChardevCommon),
    #[qapi(name = "udp")]
    #[qapi(added = "6.2.0")]
    Udp(ChardevUdpWrapper),
    #[qapi(name = "udp")]
    #[qapi(added = "1.5.0")]
    #[qapi(removed = "6.2.0")]
    UdpLegacy(ChardevUdp),
    #[qapi(name = "vc")]
    #[qapi(added = "6.2.0")]
    Vc(ChardevVcWrapper),
    #[qapi(name = "vc")]
    #[qapi(added = "1.5.0")]
    #[qapi(removed = "6.2.0")]
    VcLegacy(ChardevVc),
    #[qapi(name = "wctablet")]
    #[qapi(added = "6.2.0")]
    Wctablet(ChardevCommonWrapper),
    #[qapi(name = "wctablet")]
    #[qapi(added = "2.9.0")]
    #[qapi(removed = "6.2.0")]
    WctabletLegacy(ChardevCommon),
}

/// QMP struct `ChardevBackendInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.0.0")]
pub struct ChardevBackendInfo {
    pub name: String,
}

/// QMP enum `ChardevBackendKind`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.4.0")]
pub enum ChardevBackendKind {
    #[qapi(name = "braille")]
    #[qapi(condition = "CONFIG_BRLAPI")]
    #[qapi(added = "1.5.0")]
    Braille,
    #[qapi(name = "console")]
    #[qapi(condition = "CONFIG_WIN32")]
    #[qapi(added = "1.5.0")]
    Console,
    #[qapi(name = "dbus")]
    #[qapi(condition = "CONFIG_DBUS_DISPLAY")]
    #[qapi(added = "7.0.0")]
    Dbus,
    #[qapi(name = "file")]
    File,
    #[qapi(name = "hub")]
    #[qapi(added = "10.0.0")]
    Hub,
    #[qapi(name = "memory")]
    #[qapi(added = "1.5.0")]
    #[qapi(deprecated(added = "9.0.0"))]
    Memory,
    #[qapi(name = "msmouse")]
    #[qapi(added = "1.5.0")]
    Msmouse,
    #[qapi(name = "mux")]
    #[qapi(added = "1.5.0")]
    Mux,
    #[qapi(name = "null")]
    Null,
    #[qapi(name = "parallel")]
    #[qapi(condition = "HAVE_CHARDEV_PARALLEL")]
    Parallel,
    #[qapi(name = "pipe")]
    #[qapi(added = "1.5.0")]
    Pipe,
    #[qapi(name = "pty")]
    Pty,
    #[qapi(name = "qemu-vdagent")]
    #[qapi(condition = "CONFIG_SPICE_PROTOCOL")]
    #[qapi(added = "6.1.0")]
    QemuVdagent,
    #[qapi(name = "ringbuf")]
    #[qapi(added = "1.5.3")]
    Ringbuf,
    #[qapi(name = "serial")]
    #[qapi(condition = "HAVE_CHARDEV_SERIAL")]
    Serial,
    #[qapi(name = "socket")]
    Socket,
    #[qapi(name = "spiceport")]
    #[qapi(condition = "CONFIG_SPICE")]
    #[qapi(added = "1.5.0")]
    Spiceport,
    #[qapi(name = "spicevmc")]
    #[qapi(condition = "CONFIG_SPICE")]
    #[qapi(added = "1.5.0")]
    Spicevmc,
    #[qapi(name = "stdio")]
    #[qapi(added = "1.5.0")]
    Stdio,
    #[qapi(name = "testdev")]
    #[qapi(added = "2.2.0")]
    Testdev,
    #[qapi(name = "udp")]
    #[qapi(added = "1.5.0")]
    Udp,
    #[qapi(name = "vc")]
    #[qapi(added = "1.5.0")]
    Vc,
    #[qapi(name = "wctablet")]
    #[qapi(added = "2.9.0")]
    Wctablet,
}

/// QMP struct `ChardevCommon`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.6.0")]
pub struct ChardevCommon {
    pub logappend: Option<bool>,
    pub logfile: Option<String>,
    #[qapi(added = "11.0.0")]
    pub logtimestamp: Option<bool>,
}

/// QMP struct `ChardevCommonWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct ChardevCommonWrapper {
    pub data: ChardevCommon,
}

/// QMP struct `ChardevDBus`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_DBUS_DISPLAY")]
#[qapi(added = "7.0.0")]
pub struct ChardevDBus {
    #[qapi(added = "11.1.0")]
    pub encoding: Option<ChardevVcEncoding>,
    pub logappend: Option<bool>,
    pub logfile: Option<String>,
    #[qapi(added = "11.0.0")]
    pub logtimestamp: Option<bool>,
    pub name: String,
}

/// QMP struct `ChardevDBusWrapper`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_DBUS_DISPLAY")]
#[qapi(added = "7.0.0")]
pub struct ChardevDBusWrapper {
    pub data: ChardevDBus,
}

/// QMP struct `ChardevDummy`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.4.0")]
#[qapi(removed = "2.6.0")]
pub struct ChardevDummy {}

/// QMP struct `ChardevFile`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.4.0")]
pub struct ChardevFile {
    #[qapi(added = "2.6.0")]
    pub append: Option<bool>,
    pub r#in: Option<String>,
    #[qapi(added = "2.6.0")]
    pub logappend: Option<bool>,
    #[qapi(added = "2.6.0")]
    pub logfile: Option<String>,
    #[qapi(added = "11.0.0")]
    pub logtimestamp: Option<bool>,
    pub out: String,
}

/// QMP struct `ChardevFileWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct ChardevFileWrapper {
    pub data: ChardevFile,
}

/// QMP struct `ChardevHostdev`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.4.0")]
pub struct ChardevHostdev {
    pub device: String,
    #[qapi(added = "2.6.0")]
    pub logappend: Option<bool>,
    #[qapi(added = "2.6.0")]
    pub logfile: Option<String>,
    #[qapi(added = "11.0.0")]
    pub logtimestamp: Option<bool>,
}

/// QMP struct `ChardevHostdevWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct ChardevHostdevWrapper {
    pub data: ChardevHostdev,
}

/// QMP struct `ChardevHub`; see the QEMU QMP Reference Manual.
#[qapi(added = "10.0.0")]
pub struct ChardevHub {
    pub chardevs: Vec<String>,
    pub logappend: Option<bool>,
    pub logfile: Option<String>,
    #[qapi(added = "11.0.0")]
    pub logtimestamp: Option<bool>,
}

/// QMP struct `ChardevHubWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "10.0.0")]
pub struct ChardevHubWrapper {
    pub data: ChardevHub,
}

/// QMP struct `ChardevInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.0.0")]
pub struct ChardevInfo {
    pub filename: String,
    #[qapi(name = "frontend-open")]
    #[qapi(added = "2.1.0")]
    #[qapi(required)]
    pub frontend_open: Option<bool>,
    pub label: String,
}

/// QMP struct `ChardevMux`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.5.0")]
pub struct ChardevMux {
    pub chardev: String,
    #[qapi(added = "2.6.0")]
    pub logappend: Option<bool>,
    #[qapi(added = "2.6.0")]
    pub logfile: Option<String>,
    #[qapi(added = "11.0.0")]
    pub logtimestamp: Option<bool>,
}

/// QMP struct `ChardevMuxWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct ChardevMuxWrapper {
    pub data: ChardevMux,
}

/// QMP struct `ChardevPty`; see the QEMU QMP Reference Manual.
#[qapi(added = "9.2.0")]
pub struct ChardevPty {
    pub logappend: Option<bool>,
    pub logfile: Option<String>,
    #[qapi(added = "11.0.0")]
    pub logtimestamp: Option<bool>,
    pub path: Option<String>,
}

/// QMP struct `ChardevPtyWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "9.2.0")]
pub struct ChardevPtyWrapper {
    pub data: ChardevPty,
}

/// QMP struct `ChardevQemuVDAgent`; see the QEMU QMP Reference Manual.
#[qapi(name = "ChardevQemuVDAgent")]
#[qapi(condition = "CONFIG_SPICE_PROTOCOL")]
#[qapi(added = "6.1.0")]
pub struct ChardevQemuVdAgent {
    pub clipboard: Option<bool>,
    pub logappend: Option<bool>,
    pub logfile: Option<String>,
    #[qapi(added = "11.0.0")]
    pub logtimestamp: Option<bool>,
    pub mouse: Option<bool>,
}

/// QMP struct `ChardevQemuVDAgentWrapper`; see the QEMU QMP Reference Manual.
#[qapi(name = "ChardevQemuVDAgentWrapper")]
#[qapi(condition = "CONFIG_SPICE_PROTOCOL")]
#[qapi(added = "6.2.0")]
pub struct ChardevQemuVdAgentWrapper {
    pub data: ChardevQemuVdAgent,
}

/// QMP struct `ChardevReturn`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.4.0")]
pub struct ChardevReturn {
    pub pty: Option<String>,
}

/// QMP struct `ChardevRingbuf`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.5.0")]
pub struct ChardevRingbuf {
    #[qapi(added = "2.6.0")]
    pub logappend: Option<bool>,
    #[qapi(added = "2.6.0")]
    pub logfile: Option<String>,
    #[qapi(added = "11.0.0")]
    pub logtimestamp: Option<bool>,
    pub size: Option<i64>,
}

/// QMP struct `ChardevRingbufWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct ChardevRingbufWrapper {
    pub data: ChardevRingbuf,
}

/// QMP struct `ChardevSocket`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.4.0")]
pub struct ChardevSocket {
    #[qapi(added = "2.10.0")]
    #[qapi(required)]
    pub addr: Option<SocketAddressLegacy>,
    #[qapi(name = "addr")]
    #[qapi(removed = "2.10.0")]
    #[qapi(required)]
    pub addr_legacy: Option<SocketAddress>,
    #[qapi(added = "2.6.0")]
    pub logappend: Option<bool>,
    #[qapi(added = "2.6.0")]
    pub logfile: Option<String>,
    #[qapi(added = "11.0.0")]
    pub logtimestamp: Option<bool>,
    pub nodelay: Option<bool>,
    #[qapi(added = "2.2.0")]
    #[qapi(deprecated(added = "9.2.0"))]
    #[qapi(removed = "10.2.0")]
    pub reconnect: Option<i64>,
    #[qapi(name = "reconnect-ms")]
    #[qapi(added = "9.2.0")]
    pub reconnect_ms: Option<i64>,
    pub server: Option<bool>,
    pub telnet: Option<bool>,
    #[qapi(name = "tls-authz")]
    #[qapi(added = "4.0.0")]
    pub tls_authz: Option<String>,
    #[qapi(name = "tls-creds")]
    #[qapi(added = "2.6.0")]
    pub tls_creds: Option<String>,
    #[qapi(added = "2.10.0")]
    pub tn3270: Option<bool>,
    pub wait: Option<bool>,
    #[qapi(added = "3.1.0")]
    pub websocket: Option<bool>,
}

/// QMP struct `ChardevSocketWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct ChardevSocketWrapper {
    pub data: ChardevSocket,
}

/// QMP struct `ChardevSpiceChannel`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_SPICE")]
#[qapi(added = "1.5.0")]
pub struct ChardevSpiceChannel {
    #[qapi(added = "2.6.0")]
    pub logappend: Option<bool>,
    #[qapi(added = "2.6.0")]
    pub logfile: Option<String>,
    #[qapi(added = "11.0.0")]
    pub logtimestamp: Option<bool>,
    pub r#type: String,
}

/// QMP struct `ChardevSpiceChannelWrapper`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_SPICE")]
#[qapi(added = "6.2.0")]
pub struct ChardevSpiceChannelWrapper {
    pub data: ChardevSpiceChannel,
}

/// QMP struct `ChardevSpicePort`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_SPICE")]
#[qapi(added = "1.5.0")]
pub struct ChardevSpicePort {
    pub fqdn: String,
    #[qapi(added = "2.6.0")]
    pub logappend: Option<bool>,
    #[qapi(added = "2.6.0")]
    pub logfile: Option<String>,
    #[qapi(added = "11.0.0")]
    pub logtimestamp: Option<bool>,
}

/// QMP struct `ChardevSpicePortWrapper`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_SPICE")]
#[qapi(added = "6.2.0")]
pub struct ChardevSpicePortWrapper {
    pub data: ChardevSpicePort,
}

/// QMP struct `ChardevStdio`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.5.0")]
pub struct ChardevStdio {
    #[qapi(added = "2.6.0")]
    pub logappend: Option<bool>,
    #[qapi(added = "2.6.0")]
    pub logfile: Option<String>,
    #[qapi(added = "11.0.0")]
    pub logtimestamp: Option<bool>,
    pub signal: Option<bool>,
}

/// QMP struct `ChardevStdioWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct ChardevStdioWrapper {
    pub data: ChardevStdio,
}

/// QMP struct `ChardevUdp`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.5.0")]
pub struct ChardevUdp {
    #[qapi(added = "2.10.0")]
    pub local: Option<SocketAddressLegacy>,
    #[qapi(name = "local")]
    #[qapi(removed = "2.10.0")]
    pub local_legacy: Option<SocketAddress>,
    #[qapi(added = "2.6.0")]
    pub logappend: Option<bool>,
    #[qapi(added = "2.6.0")]
    pub logfile: Option<String>,
    #[qapi(added = "11.0.0")]
    pub logtimestamp: Option<bool>,
    #[qapi(added = "2.10.0")]
    #[qapi(required)]
    pub remote: Option<SocketAddressLegacy>,
    #[qapi(name = "remote")]
    #[qapi(removed = "2.10.0")]
    #[qapi(required)]
    pub remote_legacy: Option<SocketAddress>,
}

/// QMP struct `ChardevUdpWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct ChardevUdpWrapper {
    pub data: ChardevUdp,
}

/// QMP struct `ChardevVC`; see the QEMU QMP Reference Manual.
#[qapi(name = "ChardevVC")]
#[qapi(added = "1.5.0")]
pub struct ChardevVc {
    pub cols: Option<i64>,
    #[qapi(added = "11.1.0")]
    pub encoding: Option<ChardevVcEncoding>,
    pub height: Option<i64>,
    #[qapi(added = "2.6.0")]
    pub logappend: Option<bool>,
    #[qapi(added = "2.6.0")]
    pub logfile: Option<String>,
    #[qapi(added = "11.0.0")]
    pub logtimestamp: Option<bool>,
    pub rows: Option<i64>,
    pub width: Option<i64>,
}

/// QMP enum `ChardevVCEncoding`; see the QEMU QMP Reference Manual.
#[qapi(name = "ChardevVCEncoding")]
#[qapi(added = "11.1.0")]
pub enum ChardevVcEncoding {
    #[qapi(name = "cp437")]
    Cp437,
    #[qapi(name = "utf8")]
    Utf8,
}

/// QMP struct `ChardevVCWrapper`; see the QEMU QMP Reference Manual.
#[qapi(name = "ChardevVCWrapper")]
#[qapi(added = "6.2.0")]
pub struct ChardevVcWrapper {
    pub data: ChardevVc,
}

/// QMP struct `ColoCompareProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct ColoCompareProperties {
    pub compare_timeout: Option<u64>,
    pub expired_scan_cycle: Option<u32>,
    pub iothread: String,
    pub max_queue_size: Option<u32>,
    pub notify_dev: Option<String>,
    pub outdev: String,
    pub primary_in: String,
    pub secondary_in: String,
    pub vnet_hdr_support: Option<bool>,
}

/// QMP enum `COLOExitReason`; see the QEMU QMP Reference Manual.
#[qapi(name = "COLOExitReason")]
#[qapi(added = "3.1.0")]
pub enum ColoExitReason {
    #[qapi(name = "error")]
    Error,
    #[qapi(name = "none")]
    None,
    #[qapi(name = "processing")]
    #[qapi(added = "4.0.0")]
    Processing,
    #[qapi(name = "request")]
    Request,
}

/// QMP enum `COLOMessage`; see the QEMU QMP Reference Manual.
#[qapi(name = "COLOMessage")]
#[qapi(added = "2.8.0")]
pub enum ColoMessage {
    #[qapi(name = "checkpoint-ready")]
    CheckpointReady,
    #[qapi(name = "checkpoint-reply")]
    CheckpointReply,
    #[qapi(name = "checkpoint-request")]
    CheckpointRequest,
    #[qapi(name = "vmstate-loaded")]
    VmstateLoaded,
    #[qapi(name = "vmstate-received")]
    VmstateReceived,
    #[qapi(name = "vmstate-send")]
    VmstateSend,
    #[qapi(name = "vmstate-size")]
    VmstateSize,
}

/// QMP enum `COLOMode`; see the QEMU QMP Reference Manual.
#[qapi(name = "COLOMode")]
#[qapi(added = "2.8.0")]
pub enum ColoMode {
    #[qapi(name = "none")]
    #[qapi(added = "3.1.0")]
    None,
    #[qapi(name = "primary")]
    Primary,
    #[qapi(name = "secondary")]
    Secondary,
    #[qapi(name = "unknown")]
    #[qapi(removed = "3.1.0")]
    Unknown,
}

/// QMP struct `COLOStatus`; see the QEMU QMP Reference Manual.
#[qapi(name = "COLOStatus")]
#[qapi(condition = "CONFIG_REPLICATION")]
#[qapi(added = "3.1.0")]
pub struct ColoStatus {
    #[qapi(name = "last-mode")]
    #[qapi(added = "4.0.0")]
    #[qapi(required)]
    pub last_mode: Option<ColoMode>,
    pub mode: ColoMode,
    pub reason: ColoExitReason,
}

/// QMP enum `CommandDropReason`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
#[qapi(removed = "4.0.0")]
pub enum CommandDropReason {
    #[qapi(name = "queue-full")]
    QueueFull,
}

/// QMP struct `CommandInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.0.0")]
pub struct CommandInfo {
    pub name: String,
}

/// QMP struct `CommandLineOptionInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.5.0")]
pub struct CommandLineOptionInfo {
    pub option: String,
    pub parameters: Vec<CommandLineParameterInfo>,
}

/// QMP struct `CommandLineParameterInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.5.0")]
pub struct CommandLineParameterInfo {
    #[qapi(added = "2.1.0")]
    pub default: Option<String>,
    pub help: Option<String>,
    pub name: String,
    pub r#type: CommandLineParameterType,
}

/// QMP enum `CommandLineParameterType`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.5.0")]
pub enum CommandLineParameterType {
    #[qapi(name = "boolean")]
    Boolean,
    #[qapi(name = "number")]
    Number,
    #[qapi(name = "size")]
    Size,
    #[qapi(name = "string")]
    String,
}

/// QMP struct `CompatPolicy`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct CompatPolicy {
    #[qapi(name = "deprecated-input")]
    pub deprecated_input: Option<CompatPolicyInput>,
    #[qapi(name = "deprecated-output")]
    pub deprecated_output: Option<CompatPolicyOutput>,
    #[qapi(name = "unstable-input")]
    #[qapi(added = "6.2.0")]
    pub unstable_input: Option<CompatPolicyInput>,
    #[qapi(name = "unstable-output")]
    #[qapi(added = "6.2.0")]
    pub unstable_output: Option<CompatPolicyOutput>,
}

/// QMP enum `CompatPolicyInput`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub enum CompatPolicyInput {
    #[qapi(name = "accept")]
    Accept,
    #[qapi(name = "crash")]
    Crash,
    #[qapi(name = "reject")]
    Reject,
}

/// QMP enum `CompatPolicyOutput`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub enum CompatPolicyOutput {
    #[qapi(name = "accept")]
    Accept,
    #[qapi(name = "hide")]
    Hide,
}

/// QMP struct `CompatProperty`; see the QEMU QMP Reference Manual.
#[qapi(added = "9.1.0")]
pub struct CompatProperty {
    pub property: String,
    #[qapi(name = "qom-type")]
    pub qom_type: String,
    pub value: String,
}

/// QMP struct `CompressionStats`; see the QEMU QMP Reference Manual.
#[qapi(added = "3.1.0")]
pub struct CompressionStats {
    pub busy: i64,
    #[qapi(name = "busy-rate")]
    pub busy_rate: f64,
    #[qapi(name = "compressed-size")]
    pub compressed_size: i64,
    #[qapi(name = "compression-rate")]
    pub compression_rate: f64,
    pub pages: i64,
}

/// QMP struct `CpuDefinitionInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.2.0")]
pub struct CpuDefinitionInfo {
    #[qapi(name = "alias-of")]
    #[qapi(added = "4.1.0")]
    pub alias_of: Option<String>,
    #[qapi(added = "5.2.0")]
    #[qapi(required)]
    pub deprecated: Option<bool>,
    #[qapi(name = "migration-safe")]
    #[qapi(added = "2.8.0")]
    pub migration_safe: Option<bool>,
    pub name: String,
    #[qapi(added = "2.8.0")]
    #[qapi(required)]
    pub r#static: Option<bool>,
    #[qapi(added = "2.9.0")]
    #[qapi(required)]
    pub typename: Option<String>,
    #[qapi(name = "unavailable-features")]
    #[qapi(added = "2.8.0")]
    pub unavailable_features: Option<Vec<String>>,
}

/// QMP union `CpuInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.0.0")]
#[qapi(removed = "6.0.0")]
pub struct CpuInfo {
    #[qapi(discriminator)]
    #[qapi(added = "2.6.0")]
    #[qapi(required)]
    pub arch: Option<CpuInfoArch>,
    #[qapi(name = "CPU")]
    pub cpu: i64,
    pub current: bool,
    pub halted: bool,
    #[qapi(removed = "2.6.0")]
    pub nip: Option<i64>,
    #[qapi(removed = "2.6.0")]
    pub npc: Option<i64>,
    #[qapi(removed = "2.6.0")]
    pub pc: Option<i64>,
    #[qapi(name = "PC")]
    #[qapi(removed = "2.6.0")]
    pub pc_alt: Option<i64>,
    #[qapi(added = "2.10.0")]
    pub props: Option<CpuInstanceProperties>,
    #[qapi(added = "2.4.0")]
    #[qapi(required)]
    pub qom_path: Option<String>,
    pub thread_id: i64,
    #[qapi(union)]
    #[qapi(flatten)]
    #[qapi(added = "2.6.0")]
    pub u: Option<CpuInfoBranch>,
}

/// QMP enum `CpuInfoArch`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.6.0")]
#[qapi(removed = "6.0.0")]
pub enum CpuInfoArch {
    #[qapi(name = "mips")]
    Mips,
    #[qapi(name = "other")]
    Other,
    #[qapi(name = "ppc")]
    Ppc,
    #[qapi(name = "riscv")]
    #[qapi(added = "2.12.0")]
    Riscv,
    #[qapi(name = "s390")]
    #[qapi(added = "2.12.0")]
    S390,
    #[qapi(name = "sparc")]
    Sparc,
    #[qapi(name = "tricore")]
    Tricore,
    #[qapi(name = "x86")]
    X86,
}

#[qapi(union_branch)]
#[qapi(added = "2.6.0")]
#[qapi(removed = "6.0.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum CpuInfoBranch {
    #[qapi(name = "mips")]
    Mips(CpuInfoMips),
    #[qapi(name = "other")]
    #[qapi(added = "3.0.0")]
    Other,
    #[qapi(name = "other")]
    #[qapi(removed = "3.0.0")]
    OtherLegacy(CpuInfoOther),
    #[qapi(name = "ppc")]
    Ppc(CpuInfoPpc),
    #[qapi(name = "riscv")]
    #[qapi(added = "2.12.0")]
    Riscv(CpuInfoRiscv),
    #[qapi(name = "s390")]
    #[qapi(added = "2.12.0")]
    S390(CpuInfoS390),
    #[qapi(name = "sparc")]
    Sparc(CpuInfoSparc),
    #[qapi(name = "tricore")]
    Tricore(CpuInfoTricore),
    #[qapi(name = "x86")]
    X86(CpuInfoX86),
}

/// QMP union `CpuInfoFast`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
pub struct CpuInfoFast {
    #[qapi(discriminator(removed = "3.0.0"))]
    #[qapi(deprecated(added = "5.0.0"))]
    #[qapi(removed = "6.0.0")]
    #[qapi(required)]
    pub arch: Option<CpuInfoArch>,
    #[qapi(name = "cpu-index")]
    pub cpu_index: i64,
    pub props: Option<CpuInstanceProperties>,
    #[qapi(name = "qom-path")]
    pub qom_path: String,
    #[qapi(name = "qom-type")]
    #[qapi(added = "10.1.0")]
    #[qapi(required)]
    pub qom_type: Option<String>,
    #[qapi(discriminator)]
    #[qapi(added = "3.0.0")]
    #[qapi(required)]
    pub target: Option<SysEmuTarget>,
    #[qapi(name = "thread-id")]
    pub thread_id: i64,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<CpuInfoFastBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "2.12.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum CpuInfoFastBranch {
    #[qapi(name = "aarch64")]
    #[qapi(added = "3.0.0")]
    Aarch64,
    #[qapi(name = "alpha")]
    #[qapi(added = "3.0.0")]
    Alpha,
    #[qapi(name = "arm")]
    #[qapi(added = "3.0.0")]
    Arm,
    #[qapi(name = "avr")]
    #[qapi(added = "5.1.0")]
    Avr,
    #[qapi(name = "cris")]
    #[qapi(added = "3.0.0")]
    #[qapi(removed = "9.2.0")]
    Cris,
    #[qapi(name = "hexagon")]
    #[qapi(added = "11.0.0")]
    Hexagon,
    #[qapi(name = "hppa")]
    #[qapi(added = "3.0.0")]
    Hppa,
    #[qapi(name = "i386")]
    #[qapi(added = "3.0.0")]
    I386,
    #[qapi(name = "lm32")]
    #[qapi(added = "3.0.0")]
    #[qapi(removed = "6.1.0")]
    Lm32,
    #[qapi(name = "loongarch64")]
    #[qapi(added = "7.1.0")]
    Loongarch64,
    #[qapi(name = "m68k")]
    #[qapi(added = "3.0.0")]
    M68k,
    #[qapi(name = "microblaze")]
    #[qapi(added = "3.0.0")]
    Microblaze,
    #[qapi(name = "microblazeel")]
    #[qapi(added = "3.0.0")]
    #[qapi(removed = "11.0.0")]
    Microblazeel,
    #[qapi(name = "mips")]
    #[qapi(added = "3.0.0")]
    Mips,
    #[qapi(name = "mips64")]
    #[qapi(added = "3.0.0")]
    Mips64,
    #[qapi(name = "mips64el")]
    #[qapi(added = "3.0.0")]
    Mips64el,
    #[qapi(name = "mipsel")]
    #[qapi(added = "3.0.0")]
    Mipsel,
    #[qapi(name = "mips")]
    #[qapi(removed = "3.0.0")]
    MipsLegacy(CpuInfoOther),
    #[qapi(name = "moxie")]
    #[qapi(added = "3.0.0")]
    #[qapi(removed = "6.1.0")]
    Moxie,
    #[qapi(name = "nios2")]
    #[qapi(added = "3.0.0")]
    #[qapi(removed = "9.1.0")]
    Nios2,
    #[qapi(name = "or1k")]
    #[qapi(added = "3.0.0")]
    Or1k,
    #[qapi(name = "other")]
    #[qapi(removed = "3.0.0")]
    Other(CpuInfoOther),
    #[qapi(name = "ppc")]
    #[qapi(added = "3.0.0")]
    Ppc,
    #[qapi(name = "ppc64")]
    #[qapi(added = "3.0.0")]
    Ppc64,
    #[qapi(name = "ppcemb")]
    #[qapi(added = "3.0.0")]
    #[qapi(removed = "3.1.0")]
    Ppcemb,
    #[qapi(name = "ppc")]
    #[qapi(removed = "3.0.0")]
    PpcLegacy(CpuInfoOther),
    #[qapi(name = "riscv")]
    #[qapi(added = "2.12.1")]
    #[qapi(removed = "3.0.0")]
    Riscv(CpuInfoOther),
    #[qapi(name = "riscv32")]
    #[qapi(added = "3.0.0")]
    Riscv32,
    #[qapi(name = "riscv64")]
    #[qapi(added = "3.0.0")]
    Riscv64,
    #[qapi(name = "riscv")]
    #[qapi(removed = "2.12.1")]
    RiscvLegacy(CpuInfoRiscv),
    #[qapi(name = "rx")]
    #[qapi(added = "5.0.0")]
    Rx,
    #[qapi(name = "s390")]
    #[qapi(removed = "3.0.0")]
    S390(CpuInfoS390),
    #[qapi(name = "s390x")]
    #[qapi(added = "3.0.0")]
    S390x(CpuInfoS390),
    #[qapi(name = "sh4")]
    #[qapi(added = "3.0.0")]
    Sh4,
    #[qapi(name = "sh4eb")]
    #[qapi(added = "3.0.0")]
    Sh4eb,
    #[qapi(name = "sparc")]
    #[qapi(added = "3.0.0")]
    Sparc,
    #[qapi(name = "sparc64")]
    #[qapi(added = "3.0.0")]
    Sparc64,
    #[qapi(name = "sparc")]
    #[qapi(removed = "3.0.0")]
    SparcLegacy(CpuInfoOther),
    #[qapi(name = "tricore")]
    #[qapi(added = "3.0.0")]
    Tricore,
    #[qapi(name = "tricore")]
    #[qapi(removed = "3.0.0")]
    TricoreLegacy(CpuInfoOther),
    #[qapi(name = "unicore32")]
    #[qapi(added = "3.0.0")]
    #[qapi(removed = "6.1.0")]
    Unicore32,
    #[qapi(name = "x86")]
    #[qapi(removed = "3.0.0")]
    X86(CpuInfoOther),
    #[qapi(name = "x86_64")]
    #[qapi(added = "3.0.0")]
    X8664,
    #[qapi(name = "xtensa")]
    #[qapi(added = "3.0.0")]
    Xtensa,
    #[qapi(name = "xtensaeb")]
    #[qapi(added = "3.0.0")]
    Xtensaeb,
}

/// QMP struct `CpuInfoMIPS`; see the QEMU QMP Reference Manual.
#[qapi(name = "CpuInfoMIPS")]
#[qapi(added = "2.6.0")]
#[qapi(removed = "6.0.0")]
pub struct CpuInfoMips {
    #[qapi(name = "PC")]
    pub pc: i64,
}

/// QMP struct `CpuInfoOther`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.6.0")]
#[qapi(removed = "3.0.0")]
pub struct CpuInfoOther {}

/// QMP struct `CpuInfoPPC`; see the QEMU QMP Reference Manual.
#[qapi(name = "CpuInfoPPC")]
#[qapi(added = "2.6.0")]
#[qapi(removed = "6.0.0")]
pub struct CpuInfoPpc {
    pub nip: i64,
}

/// QMP struct `CpuInfoRISCV`; see the QEMU QMP Reference Manual.
#[qapi(name = "CpuInfoRISCV")]
#[qapi(added = "2.12.0")]
#[qapi(removed = "6.0.0")]
pub struct CpuInfoRiscv {
    pub pc: i64,
}

/// QMP struct `CpuInfoS390`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
pub struct CpuInfoS390 {
    #[qapi(name = "cpu-state")]
    #[qapi(added = "9.2.0")]
    #[qapi(required)]
    pub cpu_state: Option<S390CpuState>,
    #[qapi(name = "cpu-state")]
    #[qapi(removed = "9.2.0")]
    #[qapi(required)]
    pub cpu_state_legacy: Option<CpuS390State>,
    #[qapi(added = "8.2.0")]
    pub dedicated: Option<bool>,
    #[qapi(added = "9.2.0")]
    pub entitlement: Option<S390CpuEntitlement>,
    #[qapi(name = "entitlement")]
    #[qapi(added = "8.2.0")]
    #[qapi(removed = "9.2.0")]
    pub entitlement_legacy: Option<CpuS390Entitlement>,
}

/// QMP struct `CpuInfoSPARC`; see the QEMU QMP Reference Manual.
#[qapi(name = "CpuInfoSPARC")]
#[qapi(added = "2.6.0")]
#[qapi(removed = "6.0.0")]
pub struct CpuInfoSparc {
    pub npc: i64,
    pub pc: i64,
}

/// QMP struct `CpuInfoTricore`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.6.0")]
#[qapi(removed = "6.0.0")]
pub struct CpuInfoTricore {
    #[qapi(name = "PC")]
    pub pc: i64,
}

/// QMP struct `CpuInfoX86`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.6.0")]
#[qapi(removed = "6.0.0")]
pub struct CpuInfoX86 {
    pub pc: i64,
}

/// QMP struct `CpuInstanceProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.7.0")]
pub struct CpuInstanceProperties {
    #[qapi(name = "book-id")]
    #[qapi(added = "8.2.0")]
    pub book_id: Option<i64>,
    #[qapi(name = "cluster-id")]
    #[qapi(added = "7.1.0")]
    pub cluster_id: Option<i64>,
    #[qapi(name = "core-id")]
    pub core_id: Option<i64>,
    #[qapi(name = "die-id")]
    #[qapi(added = "4.1.0")]
    pub die_id: Option<i64>,
    #[qapi(name = "drawer-id")]
    #[qapi(added = "8.2.0")]
    pub drawer_id: Option<i64>,
    #[qapi(name = "module-id")]
    #[qapi(added = "9.1.0")]
    pub module_id: Option<i64>,
    #[qapi(name = "node-id")]
    pub node_id: Option<i64>,
    #[qapi(name = "socket-id")]
    pub socket_id: Option<i64>,
    #[qapi(name = "thread-id")]
    pub thread_id: Option<i64>,
}

/// QMP struct `CpuModelBaselineInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.8.0")]
pub struct CpuModelBaselineInfo {
    pub model: CpuModelInfo,
}

/// QMP struct `CpuModelCompareInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.8.0")]
pub struct CpuModelCompareInfo {
    #[qapi(name = "responsible-properties")]
    pub responsible_properties: Vec<String>,
    pub result: CpuModelCompareResult,
}

/// QMP enum `CpuModelCompareResult`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.8.0")]
pub enum CpuModelCompareResult {
    #[qapi(name = "identical")]
    Identical,
    #[qapi(name = "incompatible")]
    Incompatible,
    #[qapi(name = "subset")]
    Subset,
    #[qapi(name = "superset")]
    Superset,
}

/// QMP struct `CpuModelExpansionInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.8.0")]
pub struct CpuModelExpansionInfo {
    #[qapi(name = "deprecated-props")]
    #[qapi(added = "10.1.0")]
    pub deprecated_props: Option<Vec<String>>,
    #[qapi(condition = "TARGET_S390X")]
    #[qapi(name = "deprecated-props")]
    #[qapi(added = "9.1.0")]
    #[qapi(removed = "10.1.0")]
    pub deprecated_props_legacy: Option<Vec<String>>,
    pub model: CpuModelInfo,
}

/// QMP enum `CpuModelExpansionType`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.8.0")]
pub enum CpuModelExpansionType {
    #[qapi(name = "full")]
    Full,
    #[qapi(name = "static")]
    Static,
}

/// QMP struct `CpuModelInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.8.0")]
pub struct CpuModelInfo {
    pub name: String,
    pub props: Option<serde_json::Value>,
}

/// QMP struct `CpuPolarizationInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.2.0")]
pub struct CpuPolarizationInfo {
    #[qapi(added = "9.2.0")]
    #[qapi(required)]
    pub polarization: Option<S390CpuPolarization>,
    #[qapi(name = "polarization")]
    #[qapi(removed = "9.2.0")]
    #[qapi(required)]
    pub polarization_legacy: Option<CpuS390Polarization>,
}

/// QMP enum `CpuS390Entitlement`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.2.0")]
#[qapi(removed = "9.2.0")]
pub enum CpuS390Entitlement {
    #[qapi(name = "auto")]
    Auto,
    #[qapi(name = "high")]
    High,
    #[qapi(name = "low")]
    Low,
    #[qapi(name = "medium")]
    Medium,
}

/// QMP enum `CpuS390Polarization`; see the QEMU QMP Reference Manual.
#[qapi(condition = "TARGET_S390X")]
#[qapi(added = "8.2.0")]
#[qapi(removed = "9.2.0")]
pub enum CpuS390Polarization {
    #[qapi(name = "horizontal")]
    Horizontal,
    #[qapi(name = "vertical")]
    Vertical,
}

/// QMP enum `CpuS390State`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
#[qapi(removed = "9.2.0")]
pub enum CpuS390State {
    #[qapi(name = "check-stop")]
    CheckStop,
    #[qapi(name = "load")]
    Load,
    #[qapi(name = "operating")]
    Operating,
    #[qapi(name = "stopped")]
    Stopped,
    #[qapi(name = "uninitialized")]
    Uninitialized,
}

/// QMP enum `CpuTopologyLevel`; see the QEMU QMP Reference Manual.
#[qapi(added = "9.2.0")]
pub enum CpuTopologyLevel {
    #[qapi(name = "book")]
    Book,
    #[qapi(name = "cluster")]
    Cluster,
    #[qapi(name = "core")]
    Core,
    #[qapi(name = "default")]
    Default,
    #[qapi(name = "die")]
    Die,
    #[qapi(name = "drawer")]
    Drawer,
    #[qapi(name = "module")]
    Module,
    #[qapi(name = "socket")]
    Socket,
    #[qapi(name = "thread")]
    Thread,
}

/// QMP struct `CryptodevBackendProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct CryptodevBackendProperties {
    pub queues: Option<u32>,
    #[qapi(name = "throttle-bps")]
    #[qapi(added = "8.0.0")]
    pub throttle_bps: Option<u64>,
    #[qapi(name = "throttle-ops")]
    #[qapi(added = "8.0.0")]
    pub throttle_ops: Option<u64>,
}

/// QMP struct `CryptodevVhostUserProperties`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_VHOST_CRYPTO")]
#[qapi(added = "6.0.0")]
pub struct CryptodevVhostUserProperties {
    pub chardev: String,
    pub queues: Option<u32>,
    #[qapi(name = "throttle-bps")]
    #[qapi(added = "8.0.0")]
    pub throttle_bps: Option<u64>,
    #[qapi(name = "throttle-ops")]
    #[qapi(added = "8.0.0")]
    pub throttle_ops: Option<u64>,
}

/// QMP struct `CurrentMachineParams`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.0.0")]
pub struct CurrentMachineParams {
    #[qapi(name = "wakeup-suspend-support")]
    pub wakeup_suspend_support: bool,
}

/// QMP struct `CXLCommonEventBase`; see the QEMU QMP Reference Manual.
#[qapi(name = "CXLCommonEventBase")]
#[qapi(added = "11.0.0")]
pub struct CxlCommonEventBase {
    pub flags: u32,
    #[qapi(name = "head-id")]
    pub head_id: Option<u8>,
    #[qapi(name = "ld-id")]
    pub ld_id: Option<u16>,
    pub log: CxlEventLog,
    #[qapi(name = "maint-op-class")]
    pub maint_op_class: Option<u8>,
    #[qapi(name = "maint-op-subclass")]
    pub maint_op_subclass: Option<u8>,
    pub path: String,
}

/// QMP enum `CxlCorErrorType`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.0.0")]
pub enum CxlCorErrorType {
    #[qapi(name = "cache-data-ecc")]
    CacheDataEcc,
    #[qapi(name = "cache-poison-received")]
    CachePoisonReceived,
    #[qapi(name = "crc-threshold")]
    CrcThreshold,
    #[qapi(name = "mem-data-ecc")]
    MemDataEcc,
    #[qapi(name = "mem-poison-received")]
    MemPoisonReceived,
    #[qapi(name = "physical")]
    Physical,
    #[qapi(name = "retry-threshold")]
    RetryThreshold,
}

/// QMP struct `CXLDRAMEvent`; see the QEMU QMP Reference Manual.
#[qapi(name = "CXLDRAMEvent")]
#[qapi(added = "11.0.0")]
pub struct CxldramEvent {
    pub bank: Option<u8>,
    #[qapi(name = "bank-group")]
    pub bank_group: Option<u8>,
    pub channel: Option<u8>,
    #[qapi(name = "cme-ev-flags")]
    pub cme_ev_flags: Option<u8>,
    pub column: Option<u16>,
    #[qapi(name = "component-id")]
    pub component_id: Option<String>,
    #[qapi(name = "correction-mask")]
    pub correction_mask: Option<Vec<u64>>,
    #[qapi(name = "cvme-count")]
    pub cvme_count: Option<u32>,
    pub descriptor: u8,
    pub dpa: u64,
    pub flags: u32,
    #[qapi(name = "head-id")]
    pub head_id: Option<u8>,
    #[qapi(name = "is-comp-id-pldm")]
    pub is_comp_id_pldm: Option<bool>,
    #[qapi(name = "ld-id")]
    pub ld_id: Option<u16>,
    pub log: CxlEventLog,
    #[qapi(name = "maint-op-class")]
    pub maint_op_class: Option<u8>,
    #[qapi(name = "maint-op-subclass")]
    pub maint_op_subclass: Option<u8>,
    #[qapi(name = "nibble-mask")]
    pub nibble_mask: Option<u32>,
    pub path: String,
    pub rank: Option<u8>,
    pub row: Option<u32>,
    #[qapi(name = "sub-channel")]
    pub sub_channel: Option<u8>,
    #[qapi(name = "sub-type")]
    pub sub_type: u8,
    #[qapi(name = "transaction-type")]
    pub transaction_type: u8,
    pub r#type: u8,
}

/// QMP struct `CxlDynamicCapacityExtent`; see the QEMU QMP Reference Manual.
#[qapi(added = "9.1.0")]
pub struct CxlDynamicCapacityExtent {
    pub len: u64,
    pub offset: u64,
}

/// QMP enum `CxlEventLog`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.1.0")]
pub enum CxlEventLog {
    #[qapi(name = "failure")]
    Failure,
    #[qapi(name = "fatal")]
    Fatal,
    #[qapi(name = "informational")]
    Informational,
    #[qapi(name = "warning")]
    Warning,
}

/// QMP enum `CxlExtentRemovalPolicy`; see the QEMU QMP Reference Manual.
#[qapi(added = "9.1.0")]
pub enum CxlExtentRemovalPolicy {
    #[qapi(name = "prescriptive")]
    Prescriptive,
    #[qapi(name = "tag-based")]
    TagBased,
}

/// QMP enum `CxlExtentSelectionPolicy`; see the QEMU QMP Reference Manual.
#[qapi(added = "9.1.0")]
pub enum CxlExtentSelectionPolicy {
    #[qapi(name = "contiguous")]
    Contiguous,
    #[qapi(name = "enable-shared-access")]
    EnableSharedAccess,
    #[qapi(name = "free")]
    Free,
    #[qapi(name = "prescriptive")]
    Prescriptive,
}

/// QMP struct `CXLFixedMemoryWindowOptions`; see the QEMU QMP Reference Manual.
#[qapi(name = "CXLFixedMemoryWindowOptions")]
#[qapi(added = "7.1.0")]
pub struct CxlFixedMemoryWindowOptions {
    #[qapi(name = "interleave-granularity")]
    pub interleave_granularity: Option<u64>,
    pub size: u64,
    pub targets: Vec<String>,
}

/// QMP struct `CXLFMWProperties`; see the QEMU QMP Reference Manual.
#[qapi(name = "CXLFMWProperties")]
#[qapi(added = "7.1.0")]
pub struct CxlfmwProperties {
    #[qapi(name = "cxl-fmw")]
    pub cxl_fmw: Vec<CxlFixedMemoryWindowOptions>,
}

/// QMP struct `CXLGeneralMediaEvent`; see the QEMU QMP Reference Manual.
#[qapi(name = "CXLGeneralMediaEvent")]
#[qapi(added = "11.0.0")]
pub struct CxlGeneralMediaEvent {
    pub channel: Option<u8>,
    #[qapi(name = "cme-count")]
    pub cme_count: Option<u32>,
    #[qapi(name = "cme-ev-flags")]
    pub cme_ev_flags: Option<u8>,
    #[qapi(name = "component-id")]
    pub component_id: Option<String>,
    pub descriptor: u8,
    pub device: Option<u32>,
    pub dpa: u64,
    pub flags: u32,
    #[qapi(name = "head-id")]
    pub head_id: Option<u8>,
    #[qapi(name = "is-comp-id-pldm")]
    pub is_comp_id_pldm: Option<bool>,
    #[qapi(name = "ld-id")]
    pub ld_id: Option<u16>,
    pub log: CxlEventLog,
    #[qapi(name = "maint-op-class")]
    pub maint_op_class: Option<u8>,
    #[qapi(name = "maint-op-subclass")]
    pub maint_op_subclass: Option<u8>,
    pub path: String,
    pub rank: Option<u8>,
    #[qapi(name = "sub-type")]
    pub sub_type: u8,
    #[qapi(name = "transaction-type")]
    pub transaction_type: u8,
    pub r#type: u8,
}

/// QMP struct `CXLMemModuleEvent`; see the QEMU QMP Reference Manual.
#[qapi(name = "CXLMemModuleEvent")]
#[qapi(added = "11.0.0")]
pub struct CxlMemModuleEvent {
    #[qapi(name = "additional-status")]
    pub additional_status: u8,
    #[qapi(name = "component-id")]
    pub component_id: Option<String>,
    #[qapi(name = "corrected-persistent-error-count")]
    pub corrected_persistent_error_count: u32,
    #[qapi(name = "corrected-volatile-error-count")]
    pub corrected_volatile_error_count: u32,
    #[qapi(name = "dirty-shutdown-count")]
    pub dirty_shutdown_count: u32,
    pub flags: u32,
    #[qapi(name = "head-id")]
    pub head_id: Option<u8>,
    #[qapi(name = "health-status")]
    pub health_status: u8,
    #[qapi(name = "is-comp-id-pldm")]
    pub is_comp_id_pldm: Option<bool>,
    #[qapi(name = "ld-id")]
    pub ld_id: Option<u16>,
    #[qapi(name = "life-used")]
    pub life_used: u8,
    pub log: CxlEventLog,
    #[qapi(name = "maint-op-class")]
    pub maint_op_class: Option<u8>,
    #[qapi(name = "maint-op-subclass")]
    pub maint_op_subclass: Option<u8>,
    #[qapi(name = "media-status")]
    pub media_status: u8,
    pub path: String,
    #[qapi(name = "sub-type")]
    pub sub_type: u8,
    pub temperature: i16,
    pub r#type: u8,
}

/// QMP struct `CXLUncorErrorRecord`; see the QEMU QMP Reference Manual.
#[qapi(name = "CXLUncorErrorRecord")]
#[qapi(added = "8.0.0")]
pub struct CxlUncorErrorRecord {
    pub header: Vec<u32>,
    pub r#type: CxlUncorErrorType,
}

/// QMP enum `CxlUncorErrorType`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.0.0")]
pub enum CxlUncorErrorType {
    #[qapi(name = "cache-address-parity")]
    CacheAddressParity,
    #[qapi(name = "cache-be-parity")]
    CacheBeParity,
    #[qapi(name = "cache-data-ecc")]
    CacheDataEcc,
    #[qapi(name = "cache-data-parity")]
    CacheDataParity,
    #[qapi(name = "cxl-ide-rx")]
    CxlIdeRx,
    #[qapi(name = "cxl-ide-tx")]
    CxlIdeTx,
    #[qapi(name = "internal")]
    Internal,
    #[qapi(name = "mem-address-parity")]
    MemAddressParity,
    #[qapi(name = "mem-be-parity")]
    MemBeParity,
    #[qapi(name = "mem-data-ecc")]
    MemDataEcc,
    #[qapi(name = "mem-data-parity")]
    MemDataParity,
    #[qapi(name = "poison-received")]
    PoisonReceived,
    #[qapi(name = "receiver-overflow")]
    ReceiverOverflow,
    #[qapi(name = "reinit-threshold")]
    ReinitThreshold,
    #[qapi(name = "rsvd-encoding")]
    RsvdEncoding,
}

/// QMP enum `DataFormat`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.4.0")]
pub enum DataFormat {
    #[qapi(name = "base64")]
    Base64,
    #[qapi(name = "utf8")]
    Utf8,
}

/// QMP struct `DBusVMStateProperties`; see the QEMU QMP Reference Manual.
#[qapi(name = "DBusVMStateProperties")]
#[qapi(added = "6.0.0")]
pub struct DBusVmStateProperties {
    pub addr: String,
    #[qapi(name = "id-list")]
    pub id_list: Option<String>,
}

/// QMP struct `DevicePropertyInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.2.0")]
#[qapi(removed = "2.12.0")]
pub struct DevicePropertyInfo {
    #[qapi(added = "2.2.0")]
    pub description: Option<String>,
    pub name: String,
    pub r#type: String,
}

/// QMP enum `DirtyBitmapStatus`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.4.0")]
#[qapi(removed = "6.0.0")]
pub enum DirtyBitmapStatus {
    #[qapi(name = "active")]
    Active,
    #[qapi(name = "disabled")]
    Disabled,
    #[qapi(name = "frozen")]
    Frozen,
    #[qapi(name = "inconsistent")]
    #[qapi(added = "4.0.0")]
    Inconsistent,
    #[qapi(name = "locked")]
    #[qapi(added = "2.12.0")]
    Locked,
}

/// QMP struct `DirtyLimitInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.1.0")]
pub struct DirtyLimitInfo {
    #[qapi(name = "cpu-index")]
    pub cpu_index: i64,
    #[qapi(name = "current-rate")]
    pub current_rate: u64,
    #[qapi(name = "limit-rate")]
    pub limit_rate: u64,
}

/// QMP struct `DirtyRateInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "5.2.0")]
pub struct DirtyRateInfo {
    #[qapi(name = "calc-time")]
    pub calc_time: i64,
    #[qapi(name = "calc-time-unit")]
    #[qapi(added = "8.2.0")]
    #[qapi(required)]
    pub calc_time_unit: Option<TimeUnit>,
    #[qapi(name = "dirty-rate")]
    pub dirty_rate: Option<i64>,
    #[qapi(added = "6.2.0")]
    #[qapi(required)]
    pub mode: Option<DirtyRateMeasureMode>,
    #[qapi(name = "sample-pages")]
    #[qapi(added = "6.1.0")]
    #[qapi(required)]
    pub sample_pages: Option<u64>,
    #[qapi(name = "start-time")]
    pub start_time: i64,
    pub status: DirtyRateStatus,
    #[qapi(name = "vcpu-dirty-rate")]
    #[qapi(added = "6.2.0")]
    pub vcpu_dirty_rate: Option<Vec<DirtyRateVcpu>>,
}

/// QMP enum `DirtyRateMeasureMode`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub enum DirtyRateMeasureMode {
    #[qapi(name = "dirty-bitmap")]
    DirtyBitmap,
    #[qapi(name = "dirty-ring")]
    DirtyRing,
    #[qapi(name = "page-sampling")]
    PageSampling,
}

/// QMP enum `DirtyRateStatus`; see the QEMU QMP Reference Manual.
#[qapi(added = "5.2.0")]
pub enum DirtyRateStatus {
    #[qapi(name = "measured")]
    Measured,
    #[qapi(name = "measuring")]
    Measuring,
    #[qapi(name = "unstarted")]
    Unstarted,
}

/// QMP struct `DirtyRateVcpu`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct DirtyRateVcpu {
    #[qapi(name = "dirty-rate")]
    pub dirty_rate: i64,
    pub id: i64,
}

/// QMP struct `DisplayCocoa`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.0.0")]
pub struct DisplayCocoa {
    #[qapi(name = "full-grab")]
    pub full_grab: Option<bool>,
    #[qapi(name = "left-command-key")]
    pub left_command_key: Option<bool>,
    #[qapi(name = "swap-opt-cmd")]
    pub swap_opt_cmd: Option<bool>,
    #[qapi(name = "zoom-interpolation")]
    #[qapi(added = "9.0.0")]
    pub zoom_interpolation: Option<bool>,
    #[qapi(name = "zoom-to-fit")]
    #[qapi(added = "8.2.0")]
    pub zoom_to_fit: Option<bool>,
}

/// QMP struct `DisplayCurses`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.0.0")]
pub struct DisplayCurses {
    pub charset: Option<String>,
}

/// QMP struct `DisplayDBus`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.0.0")]
pub struct DisplayDBus {
    pub addr: Option<String>,
    pub audiodev: Option<String>,
    pub p2p: Option<bool>,
    pub rendernode: Option<String>,
}

/// QMP struct `DisplayEGLHeadless`; see the QEMU QMP Reference Manual.
#[qapi(name = "DisplayEGLHeadless")]
#[qapi(added = "3.1.0")]
pub struct DisplayEglHeadless {
    pub rendernode: Option<String>,
}

/// QMP enum `DisplayGLMode`; see the QEMU QMP Reference Manual.
#[qapi(name = "DisplayGLMode")]
#[qapi(added = "3.0.0")]
pub enum DisplayGlMode {
    #[qapi(name = "core")]
    Core,
    #[qapi(name = "es")]
    Es,
    #[qapi(name = "off")]
    Off,
    #[qapi(name = "on")]
    On,
}

/// QMP struct `DisplayGTK`; see the QEMU QMP Reference Manual.
#[qapi(name = "DisplayGTK")]
#[qapi(added = "2.12.0")]
pub struct DisplayGtk {
    #[qapi(added = "11.1.0")]
    pub clipboard: Option<bool>,
    #[qapi(name = "grab-on-hover")]
    pub grab_on_hover: Option<bool>,
    #[qapi(name = "keep-aspect-ratio")]
    #[qapi(added = "10.1.0")]
    pub keep_aspect_ratio: Option<bool>,
    #[qapi(added = "10.1.0")]
    pub scale: Option<f64>,
    #[qapi(name = "show-menubar")]
    #[qapi(added = "7.2.0")]
    pub show_menubar: Option<bool>,
    #[qapi(name = "show-tabs")]
    #[qapi(added = "7.1.0")]
    pub show_tabs: Option<bool>,
    #[qapi(name = "zoom-to-fit")]
    #[qapi(added = "3.1.0")]
    pub zoom_to_fit: Option<bool>,
}

/// QMP struct `DisplayNoOpts`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
#[qapi(removed = "3.0.0")]
pub struct DisplayNoOpts {}

/// QMP union `DisplayOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
pub struct DisplayOptions {
    #[qapi(name = "full-screen")]
    pub full_screen: Option<bool>,
    #[qapi(added = "3.0.0")]
    pub gl: Option<DisplayGlMode>,
    #[qapi(name = "gl")]
    #[qapi(removed = "3.0.0")]
    pub gl_legacy: Option<bool>,
    #[qapi(name = "show-cursor")]
    #[qapi(added = "5.0.0")]
    pub show_cursor: Option<bool>,
    #[qapi(discriminator)]
    pub r#type: DisplayType,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<DisplayOptionsBranch>,
    #[qapi(name = "window-close")]
    pub window_close: Option<bool>,
}

#[qapi(union_branch)]
#[qapi(added = "2.12.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum DisplayOptionsBranch {
    #[qapi(name = "cocoa")]
    #[qapi(condition = "CONFIG_COCOA")]
    #[qapi(added = "7.0.0")]
    Cocoa(DisplayCocoa),
    #[qapi(name = "cocoa")]
    #[qapi(removed = "3.0.0")]
    CocoaLegacy(DisplayNoOpts),
    #[qapi(condition = "CONFIG_COCOA")]
    #[qapi(name = "cocoa")]
    #[qapi(added = "3.0.0")]
    #[qapi(removed = "7.0.0")]
    CocoaLegacy2,
    #[qapi(name = "curses")]
    #[qapi(condition = "CONFIG_CURSES")]
    #[qapi(added = "4.0.0")]
    Curses(DisplayCurses),
    #[qapi(name = "curses")]
    #[qapi(removed = "3.0.0")]
    CursesLegacy(DisplayNoOpts),
    #[qapi(name = "curses")]
    #[qapi(added = "3.0.0")]
    #[qapi(removed = "4.0.0")]
    CursesLegacy2,
    #[qapi(name = "dbus")]
    #[qapi(condition = "CONFIG_DBUS_DISPLAY")]
    #[qapi(added = "7.0.0")]
    Dbus(DisplayDBus),
    #[qapi(name = "default")]
    #[qapi(added = "3.0.0")]
    Default,
    #[qapi(name = "default")]
    #[qapi(removed = "3.0.0")]
    DefaultLegacy(DisplayNoOpts),
    #[qapi(name = "egl-headless")]
    #[qapi(condition = "CONFIG_OPENGL")]
    #[qapi(added = "3.1.0")]
    EglHeadless(DisplayEglHeadless),
    #[qapi(name = "egl-headless")]
    #[qapi(removed = "3.0.0")]
    EglHeadlessLegacy(DisplayNoOpts),
    #[qapi(name = "egl-headless")]
    #[qapi(added = "3.0.0")]
    #[qapi(removed = "3.1.0")]
    EglHeadlessLegacy2,
    #[qapi(name = "gtk")]
    #[qapi(condition = "CONFIG_GTK")]
    Gtk(DisplayGtk),
    #[qapi(name = "none")]
    #[qapi(added = "3.0.0")]
    None,
    #[qapi(name = "none")]
    #[qapi(removed = "3.0.0")]
    NoneLegacy(DisplayNoOpts),
    #[qapi(name = "sdl")]
    #[qapi(condition = "CONFIG_SDL")]
    #[qapi(added = "7.1.0")]
    Sdl(DisplaySdl),
    #[qapi(name = "sdl")]
    #[qapi(removed = "3.0.0")]
    SdlLegacy(DisplayNoOpts),
    #[qapi(condition = "CONFIG_SDL")]
    #[qapi(name = "sdl")]
    #[qapi(added = "3.0.0")]
    #[qapi(removed = "7.1.0")]
    SdlLegacy2,
    #[qapi(name = "spice-app")]
    #[qapi(condition = "CONFIG_SPICE")]
    #[qapi(added = "4.0.0")]
    SpiceApp,
}

/// QMP enum `DisplayProtocol`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.0.0")]
pub enum DisplayProtocol {
    #[qapi(name = "spice")]
    Spice,
    #[qapi(name = "vnc")]
    Vnc,
}

/// QMP union `DisplayReloadOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct DisplayReloadOptions {
    #[qapi(discriminator)]
    pub r#type: DisplayReloadType,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<DisplayReloadOptionsBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "6.0.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum DisplayReloadOptionsBranch {
    #[qapi(name = "vnc")]
    Vnc(DisplayReloadOptionsVnc),
}

/// QMP struct `DisplayReloadOptionsVNC`; see the QEMU QMP Reference Manual.
#[qapi(name = "DisplayReloadOptionsVNC")]
#[qapi(added = "6.0.0")]
pub struct DisplayReloadOptionsVnc {
    #[qapi(name = "tls-certs")]
    pub tls_certs: Option<bool>,
}

/// QMP enum `DisplayReloadType`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub enum DisplayReloadType {
    #[qapi(name = "vnc")]
    Vnc,
}

/// QMP struct `DisplaySDL`; see the QEMU QMP Reference Manual.
#[qapi(name = "DisplaySDL")]
#[qapi(added = "7.1.0")]
pub struct DisplaySdl {
    #[qapi(name = "grab-mod")]
    pub grab_mod: Option<HotKeyMod>,
}

/// QMP enum `DisplayType`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
pub enum DisplayType {
    #[qapi(name = "cocoa")]
    #[qapi(condition = "CONFIG_COCOA")]
    Cocoa,
    #[qapi(name = "curses")]
    #[qapi(condition = "CONFIG_CURSES")]
    Curses,
    #[qapi(name = "dbus")]
    #[qapi(condition = "CONFIG_DBUS_DISPLAY")]
    #[qapi(added = "7.0.0")]
    Dbus,
    #[qapi(name = "default")]
    Default,
    #[qapi(name = "egl-headless")]
    #[qapi(condition = "CONFIG_OPENGL")]
    EglHeadless,
    #[qapi(name = "gtk")]
    #[qapi(condition = "CONFIG_GTK")]
    Gtk,
    #[qapi(name = "none")]
    None,
    #[qapi(name = "sdl")]
    #[qapi(condition = "CONFIG_SDL")]
    Sdl,
    #[qapi(name = "spice-app")]
    #[qapi(condition = "CONFIG_SPICE")]
    #[qapi(added = "4.0.0")]
    SpiceApp,
}

/// QMP union `DisplayUpdateOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.1.0")]
pub struct DisplayUpdateOptions {
    #[qapi(discriminator)]
    pub r#type: DisplayUpdateType,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<DisplayUpdateOptionsBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "7.1.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum DisplayUpdateOptionsBranch {
    #[qapi(name = "vnc")]
    Vnc(DisplayUpdateOptionsVnc),
}

/// QMP struct `DisplayUpdateOptionsVNC`; see the QEMU QMP Reference Manual.
#[qapi(name = "DisplayUpdateOptionsVNC")]
#[qapi(added = "7.1.0")]
pub struct DisplayUpdateOptionsVnc {
    pub addresses: Option<Vec<SocketAddress>>,
}

/// QMP enum `DisplayUpdateType`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.1.0")]
pub enum DisplayUpdateType {
    #[qapi(name = "vnc")]
    Vnc,
}

/// QMP struct `DriveBackupWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct DriveBackupWrapper {
    pub data: DriveBackup,
}

/// QMP struct `DummyBlockCoreForceArrays`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.0.0")]
pub struct DummyBlockCoreForceArrays {
    #[qapi(name = "unused-block-graph-info")]
    pub unused_block_graph_info: Vec<BlockGraphInfo>,
}

/// QMP struct `DummyForceArrays`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.5.0")]
pub struct DummyForceArrays {
    pub unused: Vec<X86cpuFeatureWordInfo>,
}

/// QMP struct `DummyVirtioForceArrays`; see the QEMU QMP Reference Manual.
#[qapi(added = "9.0.0")]
pub struct DummyVirtioForceArrays {
    #[qapi(name = "unused-iothread-vq-mapping")]
    pub unused_iothread_vq_mapping: Vec<IoThreadVirtQueueMapping>,
    #[qapi(name = "unused-virtio-gpu-output")]
    #[qapi(added = "10.1.0")]
    #[qapi(required)]
    pub unused_virtio_gpu_output: Option<Vec<VirtIogpuOutput>>,
}

/// QMP struct `DumpGuestMemoryCapability`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.0.0")]
pub struct DumpGuestMemoryCapability {
    pub formats: Vec<DumpGuestMemoryFormat>,
}

/// QMP enum `DumpGuestMemoryFormat`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.0.0")]
pub enum DumpGuestMemoryFormat {
    #[qapi(name = "elf")]
    Elf,
    #[qapi(name = "kdump-lzo")]
    KdumpLzo,
    #[qapi(name = "kdump-raw-lzo")]
    #[qapi(added = "8.2.0")]
    KdumpRawLzo,
    #[qapi(name = "kdump-raw-snappy")]
    #[qapi(added = "8.2.0")]
    KdumpRawSnappy,
    #[qapi(name = "kdump-raw-zlib")]
    #[qapi(added = "8.2.0")]
    KdumpRawZlib,
    #[qapi(name = "kdump-snappy")]
    KdumpSnappy,
    #[qapi(name = "kdump-zlib")]
    KdumpZlib,
    #[qapi(name = "win-dmp")]
    #[qapi(added = "3.0.0")]
    WinDmp,
}

/// QMP struct `DumpQueryResult`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.6.0")]
pub struct DumpQueryResult {
    pub completed: i64,
    pub status: DumpStatus,
    pub total: i64,
}

/// QMP enum `DumpStatus`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.6.0")]
pub enum DumpStatus {
    #[qapi(name = "active")]
    Active,
    #[qapi(name = "completed")]
    Completed,
    #[qapi(name = "failed")]
    Failed,
    #[qapi(name = "none")]
    None,
}

/// QMP struct `EbpfObject`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_EBPF")]
#[qapi(added = "9.0.0")]
pub struct EbpfObject {
    pub object: String,
}

/// QMP enum `EbpfProgramID`; see the QEMU QMP Reference Manual.
#[qapi(name = "EbpfProgramID")]
#[qapi(condition = "CONFIG_EBPF")]
#[qapi(added = "9.0.0")]
pub enum EbpfProgramId {
    #[qapi(name = "rss")]
    Rss,
}

/// QMP enum `EndianMode`; see the QEMU QMP Reference Manual.
#[qapi(added = "10.0.0")]
pub enum EndianMode {
    #[qapi(name = "big")]
    Big,
    #[qapi(name = "little")]
    Little,
    #[qapi(name = "unspecified")]
    Unspecified,
}

/// QMP enum `ErrorClass`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.2.0")]
#[qapi(removed = "2.6.0")]
pub enum ErrorClass {
    CommandNotFound,
    DeviceEncrypted,
    DeviceNotActive,
    DeviceNotFound,
    GenericError,
    #[qapi(name = "KVMMissingCap")]
    KvmMissingCap,
    #[qapi(removed = "1.3.0")]
    MigrationExpected,
}

/// QMP struct `EventInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.2.0")]
#[qapi(removed = "6.0.0")]
pub struct EventInfo {
    pub name: String,
}

/// QMP struct `EventLoopBaseProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.1.0")]
pub struct EventLoopBaseProperties {
    #[qapi(name = "aio-max-batch")]
    pub aio_max_batch: Option<i64>,
    #[qapi(name = "thread-pool-max")]
    pub thread_pool_max: Option<i64>,
    #[qapi(name = "thread-pool-min")]
    pub thread_pool_min: Option<i64>,
}

/// QMP struct `EvtchnInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.0.0")]
pub struct EvtchnInfo {
    pub masked: bool,
    pub pending: bool,
    pub port: u16,
    #[qapi(name = "remote-domain")]
    pub remote_domain: String,
    pub target: u16,
    pub r#type: EvtchnPortType,
    pub vcpu: u32,
}

/// QMP enum `EvtchnPortType`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.0.0")]
pub enum EvtchnPortType {
    #[qapi(name = "closed")]
    Closed,
    #[qapi(name = "interdomain")]
    Interdomain,
    #[qapi(name = "ipi")]
    Ipi,
    #[qapi(name = "pirq")]
    Pirq,
    #[qapi(name = "unbound")]
    Unbound,
    #[qapi(name = "virq")]
    Virq,
}

/// QMP union `ExpirePasswordOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.0.0")]
pub struct ExpirePasswordOptions {
    #[qapi(discriminator)]
    pub protocol: DisplayProtocol,
    pub time: String,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<ExpirePasswordOptionsBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "7.0.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum ExpirePasswordOptionsBranch {
    #[qapi(name = "spice")]
    Spice,
    #[qapi(name = "vnc")]
    Vnc(ExpirePasswordOptionsVnc),
}

/// QMP struct `ExpirePasswordOptionsVnc`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.0.0")]
pub struct ExpirePasswordOptionsVnc {
    pub display: Option<String>,
}

/// QMP enum `FailoverStatus`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.8.0")]
pub enum FailoverStatus {
    #[qapi(name = "active")]
    Active,
    #[qapi(name = "completed")]
    Completed,
    #[qapi(name = "none")]
    None,
    #[qapi(name = "relaunch")]
    #[qapi(added = "2.9.0")]
    Relaunch,
    #[qapi(name = "require")]
    Require,
}

/// QMP struct `FdsetFdInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.2.0")]
pub struct FdsetFdInfo {
    pub fd: i64,
    pub opaque: Option<String>,
}

/// QMP struct `FdsetInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.2.0")]
pub struct FdsetInfo {
    pub fds: Vec<FdsetFdInfo>,
    #[qapi(name = "fdset-id")]
    pub fdset_id: i64,
}

/// QMP struct `FdSocketAddress`; see the QEMU QMP Reference Manual.
#[qapi(added = "9.0.0")]
pub struct FdSocketAddress {
    pub str: String,
}

/// QMP struct `FdSocketAddressWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "9.0.0")]
pub struct FdSocketAddressWrapper {
    pub data: FdSocketAddress,
}

/// QMP struct `FileMigrationArgs`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.2.0")]
pub struct FileMigrationArgs {
    pub filename: String,
    pub offset: u64,
}

/// QMP struct `FilterBufferProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct FilterBufferProperties {
    pub insert: Option<NetfilterInsert>,
    pub interval: u32,
    pub netdev: String,
    pub position: Option<String>,
    pub queue: Option<NetFilterDirection>,
    pub status: Option<String>,
}

/// QMP struct `FilterDumpProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct FilterDumpProperties {
    pub file: String,
    pub insert: Option<NetfilterInsert>,
    pub maxlen: Option<u32>,
    pub netdev: String,
    pub position: Option<String>,
    pub queue: Option<NetFilterDirection>,
    pub status: Option<String>,
}

/// QMP struct `FilterMirrorProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct FilterMirrorProperties {
    pub insert: Option<NetfilterInsert>,
    pub netdev: String,
    pub outdev: String,
    pub position: Option<String>,
    pub queue: Option<NetFilterDirection>,
    pub status: Option<String>,
    pub vnet_hdr_support: Option<bool>,
}

/// QMP struct `FilterRedirectorProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct FilterRedirectorProperties {
    pub indev: Option<String>,
    pub insert: Option<NetfilterInsert>,
    pub netdev: String,
    pub outdev: Option<String>,
    pub position: Option<String>,
    pub queue: Option<NetFilterDirection>,
    pub status: Option<String>,
    pub vnet_hdr_support: Option<bool>,
}

/// QMP struct `FilterRewriterProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct FilterRewriterProperties {
    pub insert: Option<NetfilterInsert>,
    pub netdev: String,
    pub position: Option<String>,
    pub queue: Option<NetFilterDirection>,
    pub status: Option<String>,
    pub vnet_hdr_support: Option<bool>,
}

/// QMP struct `FirmwareLog`; see the QEMU QMP Reference Manual.
#[qapi(added = "10.2.0")]
pub struct FirmwareLog {
    pub log: String,
    pub version: Option<String>,
}

/// QMP enum `FloppyDriveType`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.6.0")]
pub enum FloppyDriveType {
    #[qapi(name = "120")]
    _120,
    #[qapi(name = "144")]
    _144,
    #[qapi(name = "288")]
    _288,
    #[qapi(name = "auto")]
    Auto,
    #[qapi(name = "none")]
    None,
}

/// QMP enum `FuseExportAllowOther`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.1.0")]
pub enum FuseExportAllowOther {
    #[qapi(name = "auto")]
    Auto,
    #[qapi(name = "off")]
    Off,
    #[qapi(name = "on")]
    On,
}

/// QMP struct `GICCapability`; see the QEMU QMP Reference Manual.
#[qapi(name = "GICCapability")]
#[qapi(added = "2.6.0")]
pub struct GicCapability {
    pub emulated: bool,
    pub kernel: bool,
    pub version: i64,
}

/// QMP union `GlusterServer`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.7.0")]
#[qapi(removed = "2.9.0")]
pub struct GlusterServer {
    #[qapi(discriminator)]
    pub r#type: GlusterTransport,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<GlusterServerBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "2.7.0")]
#[qapi(removed = "2.9.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum GlusterServerBranch {
    #[qapi(name = "tcp")]
    Tcp(InetSocketAddress),
    #[qapi(name = "unix")]
    Unix(UnixSocketAddress),
}

/// QMP enum `GlusterTransport`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.7.0")]
#[qapi(removed = "2.9.0")]
pub enum GlusterTransport {
    #[qapi(name = "tcp")]
    Tcp,
    #[qapi(name = "unix")]
    Unix,
}

/// QMP enum `GrabToggleKeys`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.0.0")]
pub enum GrabToggleKeys {
    #[qapi(name = "alt-alt")]
    AltAlt,
    #[qapi(name = "ctrl-ctrl")]
    CtrlCtrl,
    #[qapi(name = "ctrl-scrolllock")]
    CtrlScrolllock,
    #[qapi(name = "meta-meta")]
    MetaMeta,
    #[qapi(name = "scrolllock")]
    Scrolllock,
    #[qapi(name = "shift-shift")]
    #[qapi(added = "4.2.0")]
    ShiftShift,
}

/// QMP enum `GranuleMode`; see the QEMU QMP Reference Manual.
#[qapi(added = "9.0.0")]
pub enum GranuleMode {
    #[qapi(name = "16k")]
    _16k,
    #[qapi(name = "4k")]
    _4k,
    #[qapi(name = "64k")]
    _64k,
    #[qapi(name = "8k")]
    _8k,
    #[qapi(name = "host")]
    Host,
}

/// QMP enum `GuestPanicAction`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.1.0")]
pub enum GuestPanicAction {
    #[qapi(name = "pause")]
    Pause,
    #[qapi(name = "poweroff")]
    #[qapi(added = "2.8.0")]
    Poweroff,
    #[qapi(name = "run")]
    #[qapi(added = "5.0.0")]
    Run,
}

/// QMP union `GuestPanicInformation`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.9.0")]
pub struct GuestPanicInformation {
    #[qapi(discriminator)]
    pub r#type: GuestPanicInformationType,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<GuestPanicInformationBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "2.9.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum GuestPanicInformationBranch {
    #[qapi(name = "hyper-v")]
    HyperV(GuestPanicInformationHyperV),
    #[qapi(name = "s390")]
    #[qapi(added = "2.12.0")]
    S390(GuestPanicInformationS390),
    #[qapi(name = "sev")]
    #[qapi(added = "11.0.0")]
    Sev(GuestPanicInformationSev),
    #[qapi(name = "tdx")]
    #[qapi(added = "10.1.0")]
    Tdx(GuestPanicInformationTdx),
}

/// QMP struct `GuestPanicInformationHyperV`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.9.0")]
pub struct GuestPanicInformationHyperV {
    pub arg1: u64,
    pub arg2: u64,
    pub arg3: u64,
    pub arg4: u64,
    pub arg5: u64,
}

/// QMP struct `GuestPanicInformationS390`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
pub struct GuestPanicInformationS390 {
    pub core: u32,
    #[qapi(name = "psw-addr")]
    pub psw_addr: u64,
    #[qapi(name = "psw-mask")]
    pub psw_mask: u64,
    pub reason: S390CrashReason,
}

/// QMP struct `GuestPanicInformationSev`; see the QEMU QMP Reference Manual.
#[qapi(added = "11.0.0")]
pub struct GuestPanicInformationSev {
    pub code: u32,
    pub set: u32,
}

/// QMP struct `GuestPanicInformationTdx`; see the QEMU QMP Reference Manual.
#[qapi(added = "10.1.0")]
pub struct GuestPanicInformationTdx {
    #[qapi(name = "error-code")]
    pub error_code: u32,
    pub gpa: Option<u64>,
    pub message: String,
}

/// QMP enum `GuestPanicInformationType`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.9.0")]
pub enum GuestPanicInformationType {
    #[qapi(name = "hyper-v")]
    HyperV,
    #[qapi(name = "s390")]
    #[qapi(added = "2.12.0")]
    S390,
    #[qapi(name = "sev")]
    #[qapi(added = "11.0.0")]
    Sev,
    #[qapi(name = "tdx")]
    #[qapi(added = "10.1.0")]
    Tdx,
}

/// QMP struct `GuidInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.9.0")]
pub struct GuidInfo {
    pub guid: String,
}

/// QMP enum `HmatCacheAssociativity`; see the QEMU QMP Reference Manual.
#[qapi(added = "5.0.0")]
pub enum HmatCacheAssociativity {
    #[qapi(name = "complex")]
    Complex,
    #[qapi(name = "direct")]
    Direct,
    #[qapi(name = "none")]
    None,
}

/// QMP enum `HmatCacheWritePolicy`; see the QEMU QMP Reference Manual.
#[qapi(added = "5.0.0")]
pub enum HmatCacheWritePolicy {
    #[qapi(name = "none")]
    None,
    #[qapi(name = "write-back")]
    WriteBack,
    #[qapi(name = "write-through")]
    WriteThrough,
}

/// QMP enum `HmatLBDataType`; see the QEMU QMP Reference Manual.
#[qapi(name = "HmatLBDataType")]
#[qapi(added = "5.0.0")]
pub enum HmatLbDataType {
    #[qapi(name = "access-bandwidth")]
    AccessBandwidth,
    #[qapi(name = "access-latency")]
    AccessLatency,
    #[qapi(name = "read-bandwidth")]
    ReadBandwidth,
    #[qapi(name = "read-latency")]
    ReadLatency,
    #[qapi(name = "write-bandwidth")]
    WriteBandwidth,
    #[qapi(name = "write-latency")]
    WriteLatency,
}

/// QMP enum `HmatLBMemoryHierarchy`; see the QEMU QMP Reference Manual.
#[qapi(name = "HmatLBMemoryHierarchy")]
#[qapi(added = "5.0.0")]
pub enum HmatLbMemoryHierarchy {
    #[qapi(name = "first-level")]
    FirstLevel,
    #[qapi(name = "memory")]
    Memory,
    #[qapi(name = "second-level")]
    SecondLevel,
    #[qapi(name = "third-level")]
    ThirdLevel,
}

/// QMP enum `HostMemPolicy`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.1.0")]
pub enum HostMemPolicy {
    #[qapi(name = "bind")]
    Bind,
    #[qapi(name = "default")]
    Default,
    #[qapi(name = "interleave")]
    Interleave,
    #[qapi(name = "preferred")]
    Preferred,
}

/// QMP enum `HotKeyMod`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.1.0")]
pub enum HotKeyMod {
    #[qapi(name = "lctrl-lalt")]
    LctrlLalt,
    #[qapi(name = "lshift-lctrl-lalt")]
    LshiftLctrlLalt,
    #[qapi(name = "rctrl")]
    Rctrl,
}

/// QMP struct `HotpluggableCPU`; see the QEMU QMP Reference Manual.
#[qapi(name = "HotpluggableCPU")]
#[qapi(added = "2.7.0")]
pub struct HotpluggableCpu {
    pub props: CpuInstanceProperties,
    #[qapi(name = "qom-path")]
    pub qom_path: Option<String>,
    pub r#type: String,
    #[qapi(name = "vcpus-count")]
    pub vcpus_count: i64,
}

/// QMP struct `HumanReadableText`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct HumanReadableText {
    #[qapi(name = "human-readable-text")]
    pub human_readable_text: String,
}

/// QMP struct `HvBalloonDeviceInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.2.0")]
pub struct HvBalloonDeviceInfo {
    pub id: Option<String>,
    #[qapi(name = "max-size")]
    pub max_size: u64,
    pub memaddr: Option<u64>,
    pub memdev: Option<String>,
}

/// QMP struct `HvBalloonDeviceInfoWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.2.0")]
pub struct HvBalloonDeviceInfoWrapper {
    pub data: HvBalloonDeviceInfo,
}

/// QMP struct `HvBalloonInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.2.0")]
pub struct HvBalloonInfo {
    pub available: u64,
    pub committed: u64,
}

/// QMP struct `IgvmCfgProperties`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_IGVM")]
#[qapi(added = "10.1.0")]
pub struct IgvmCfgProperties {
    pub file: String,
}

/// QMP struct `ImageCheck`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.5.0")]
pub struct ImageCheck {
    #[qapi(name = "allocated-clusters")]
    pub allocated_clusters: Option<i64>,
    #[qapi(name = "check-errors")]
    pub check_errors: i64,
    #[qapi(name = "compressed-clusters")]
    pub compressed_clusters: Option<i64>,
    pub corruptions: Option<i64>,
    #[qapi(name = "corruptions-fixed")]
    pub corruptions_fixed: Option<i64>,
    pub filename: String,
    pub format: String,
    #[qapi(name = "fragmented-clusters")]
    pub fragmented_clusters: Option<i64>,
    #[qapi(name = "image-end-offset")]
    pub image_end_offset: Option<i64>,
    pub leaks: Option<i64>,
    #[qapi(name = "leaks-fixed")]
    pub leaks_fixed: Option<i64>,
    #[qapi(name = "total-clusters")]
    pub total_clusters: Option<i64>,
}

/// QMP enum `ImageFormat`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.1.0")]
pub enum ImageFormat {
    #[qapi(name = "png")]
    Png,
    #[qapi(name = "ppm")]
    Ppm,
}

/// QMP struct `ImageInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.3.0")]
pub struct ImageInfo {
    #[qapi(name = "actual-size")]
    pub actual_size: Option<i64>,
    #[qapi(name = "backing-filename")]
    pub backing_filename: Option<String>,
    #[qapi(name = "backing-filename-format")]
    pub backing_filename_format: Option<String>,
    #[qapi(name = "backing-image")]
    #[qapi(added = "1.6.0")]
    pub backing_image: Option<Box<ImageInfo>>,
    #[qapi(name = "cluster-size")]
    pub cluster_size: Option<i64>,
    #[qapi(added = "1.7.0")]
    pub compressed: Option<bool>,
    #[qapi(name = "dirty-flag")]
    pub dirty_flag: Option<bool>,
    pub encrypted: Option<bool>,
    pub filename: String,
    pub format: String,
    #[qapi(name = "format-specific")]
    #[qapi(added = "1.7.0")]
    pub format_specific: Option<ImageInfoSpecific>,
    #[qapi(name = "full-backing-filename")]
    pub full_backing_filename: Option<String>,
    #[qapi(added = "10.2.0")]
    pub limits: Option<BlockLimitsInfo>,
    pub snapshots: Option<Vec<SnapshotInfo>>,
    #[qapi(name = "virtual-size")]
    pub virtual_size: i64,
}

/// QMP union `ImageInfoSpecific`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.7.0")]
#[qapi(simple_union(removed = "6.2.0"))]
pub struct ImageInfoSpecific {
    #[qapi(discriminator)]
    #[qapi(name = "type")]
    #[qapi(removed = "6.2.0")]
    #[qapi(required)]
    pub r_type: Option<ImageInfoSpecificKind>,
    #[qapi(discriminator)]
    #[qapi(added = "6.2.0")]
    #[qapi(required)]
    pub r#type: Option<ImageInfoSpecificKind>,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<ImageInfoSpecificBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "1.7.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum ImageInfoSpecificBranch {
    #[qapi(name = "file")]
    #[qapi(added = "8.0.0")]
    File(ImageInfoSpecificFileWrapper),
    #[qapi(name = "luks")]
    #[qapi(added = "6.2.0")]
    Luks(ImageInfoSpecificLuksWrapper),
    #[qapi(name = "luks")]
    #[qapi(added = "2.7.0")]
    #[qapi(removed = "6.2.0")]
    LuksLegacy(QCryptoBlockInfoLuks),
    #[qapi(name = "qcow2")]
    #[qapi(added = "6.2.0")]
    Qcow2(ImageInfoSpecificQCow2Wrapper),
    #[qapi(name = "qcow2")]
    #[qapi(removed = "6.2.0")]
    Qcow2Legacy(ImageInfoSpecificQCow2),
    #[qapi(name = "rbd")]
    #[qapi(added = "6.2.0")]
    Rbd(ImageInfoSpecificRbdWrapper),
    #[qapi(name = "rbd")]
    #[qapi(added = "6.1.0")]
    #[qapi(removed = "6.2.0")]
    RbdLegacy(ImageInfoSpecificRbd),
    #[qapi(name = "vmdk")]
    #[qapi(added = "6.2.0")]
    Vmdk(ImageInfoSpecificVmdkWrapper),
    #[qapi(name = "vmdk")]
    #[qapi(removed = "6.2.0")]
    VmdkLegacy(ImageInfoSpecificVmdk),
}

/// QMP struct `ImageInfoSpecificFile`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.0.0")]
pub struct ImageInfoSpecificFile {
    #[qapi(name = "extent-size-hint")]
    pub extent_size_hint: Option<u64>,
}

/// QMP struct `ImageInfoSpecificFileWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.0.0")]
pub struct ImageInfoSpecificFileWrapper {
    pub data: ImageInfoSpecificFile,
}

/// QMP enum `ImageInfoSpecificKind`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.7.0")]
pub enum ImageInfoSpecificKind {
    #[qapi(name = "file")]
    #[qapi(added = "8.0.0")]
    File,
    #[qapi(name = "luks")]
    #[qapi(added = "2.7.0")]
    Luks,
    #[qapi(name = "qcow2")]
    Qcow2,
    #[qapi(name = "rbd")]
    #[qapi(added = "6.1.0")]
    Rbd,
    #[qapi(name = "vmdk")]
    Vmdk,
}

/// QMP struct `ImageInfoSpecificLUKSWrapper`; see the QEMU QMP Reference Manual.
#[qapi(name = "ImageInfoSpecificLUKSWrapper")]
#[qapi(added = "6.2.0")]
pub struct ImageInfoSpecificLuksWrapper {
    pub data: QCryptoBlockInfoLuks,
}

/// QMP struct `ImageInfoSpecificQCow2`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.7.0")]
pub struct ImageInfoSpecificQCow2 {
    #[qapi(added = "4.0.0")]
    pub bitmaps: Option<Vec<Qcow2BitmapInfo>>,
    pub compat: String,
    #[qapi(name = "compression-type")]
    #[qapi(added = "5.1.0")]
    #[qapi(required)]
    pub compression_type: Option<Qcow2CompressionType>,
    #[qapi(added = "2.2.0")]
    pub corrupt: Option<bool>,
    #[qapi(name = "data-file")]
    #[qapi(added = "4.0.0")]
    pub data_file: Option<String>,
    #[qapi(name = "data-file-raw")]
    #[qapi(added = "4.0.0")]
    pub data_file_raw: Option<bool>,
    #[qapi(added = "2.10.0")]
    pub encrypt: Option<ImageInfoSpecificQCow2Encryption>,
    #[qapi(name = "extended-l2")]
    #[qapi(added = "5.2.0")]
    pub extended_l2: Option<bool>,
    #[qapi(name = "lazy-refcounts")]
    pub lazy_refcounts: Option<bool>,
    #[qapi(name = "refcount-bits")]
    #[qapi(added = "2.3.0")]
    #[qapi(required)]
    pub refcount_bits: Option<i64>,
}

/// QMP union `ImageInfoSpecificQCow2Encryption`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.10.0")]
pub struct ImageInfoSpecificQCow2Encryption {
    #[qapi(discriminator)]
    pub format: BlockdevQcow2EncryptionFormat,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<ImageInfoSpecificQCow2EncryptionBranch>,
}

/// QMP struct `ImageInfoSpecificQCow2EncryptionBase`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.10.0")]
pub struct ImageInfoSpecificQCow2EncryptionBase {
    pub format: BlockdevQcow2EncryptionFormat,
}

#[qapi(union_branch)]
#[qapi(added = "2.10.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum ImageInfoSpecificQCow2EncryptionBranch {
    #[qapi(name = "aes")]
    #[qapi(added = "3.0.0")]
    Aes,
    #[qapi(name = "aes")]
    #[qapi(removed = "3.0.0")]
    AesLegacy(QCryptoBlockInfoQCow),
    #[qapi(name = "luks")]
    Luks(QCryptoBlockInfoLuks),
}

/// QMP struct `ImageInfoSpecificQCow2Wrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct ImageInfoSpecificQCow2Wrapper {
    pub data: ImageInfoSpecificQCow2,
}

/// QMP struct `ImageInfoSpecificRbd`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.1.0")]
pub struct ImageInfoSpecificRbd {
    #[qapi(name = "encryption-format")]
    pub encryption_format: Option<RbdImageEncryptionFormat>,
}

/// QMP struct `ImageInfoSpecificRbdWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct ImageInfoSpecificRbdWrapper {
    pub data: ImageInfoSpecificRbd,
}

/// QMP struct `ImageInfoSpecificVmdk`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.7.0")]
pub struct ImageInfoSpecificVmdk {
    pub cid: i64,
    #[qapi(name = "create-type")]
    pub create_type: String,
    #[qapi(added = "8.0.0")]
    #[qapi(required)]
    pub extents: Option<Vec<VmdkExtentInfo>>,
    #[qapi(name = "extents")]
    #[qapi(removed = "8.0.0")]
    #[qapi(required)]
    pub extents_legacy: Option<Vec<ImageInfo>>,
    #[qapi(name = "parent-cid")]
    pub parent_cid: i64,
}

/// QMP struct `ImageInfoSpecificVmdkWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct ImageInfoSpecificVmdkWrapper {
    pub data: ImageInfoSpecificVmdk,
}

/// QMP struct `InetSocketAddress`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.3.0")]
pub struct InetSocketAddress {
    pub host: String,
    pub ipv4: Option<bool>,
    pub ipv6: Option<bool>,
    #[qapi(name = "keep-alive")]
    #[qapi(added = "4.2.0")]
    pub keep_alive: Option<bool>,
    #[qapi(name = "keep-alive-count")]
    #[qapi(condition = "HAVE_TCP_KEEPCNT")]
    #[qapi(added = "10.1.0")]
    pub keep_alive_count: Option<u32>,
    #[qapi(name = "keep-alive-idle")]
    #[qapi(condition = "HAVE_TCP_KEEPIDLE")]
    #[qapi(added = "10.1.0")]
    pub keep_alive_idle: Option<u32>,
    #[qapi(name = "keep-alive-interval")]
    #[qapi(condition = "HAVE_TCP_KEEPINTVL")]
    #[qapi(added = "10.1.0")]
    pub keep_alive_interval: Option<u32>,
    #[qapi(condition = "HAVE_IPPROTO_MPTCP")]
    #[qapi(added = "6.1.0")]
    pub mptcp: Option<bool>,
    #[qapi(added = "2.9.0")]
    pub numeric: Option<bool>,
    pub port: String,
    pub to: Option<u16>,
}

/// QMP struct `InetSocketAddressBase`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.9.0")]
pub struct InetSocketAddressBase {
    pub host: String,
    pub port: String,
}

/// QMP struct `InetSocketAddressWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct InetSocketAddressWrapper {
    pub data: InetSocketAddress,
}

/// QMP enum `InputAxis`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.0.0")]
pub enum InputAxis {
    #[qapi(name = "x")]
    #[qapi(added = "2.6.0")]
    X,
    #[qapi(name = "X")]
    #[qapi(removed = "2.6.0")]
    XLegacy,
    #[qapi(name = "y")]
    #[qapi(added = "2.6.0")]
    Y,
    #[qapi(name = "Y")]
    #[qapi(removed = "2.6.0")]
    YLegacy,
}

/// QMP struct `InputBarrierProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct InputBarrierProperties {
    pub height: Option<String>,
    pub name: String,
    pub port: Option<String>,
    pub server: Option<String>,
    pub width: Option<String>,
    #[qapi(name = "x-origin")]
    pub x_origin: Option<String>,
    #[qapi(name = "y-origin")]
    pub y_origin: Option<String>,
}

/// QMP struct `InputBtnEvent`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.0.0")]
pub struct InputBtnEvent {
    pub button: InputButton,
    pub down: bool,
}

/// QMP struct `InputBtnEventWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct InputBtnEventWrapper {
    pub data: InputBtnEvent,
}

/// QMP enum `InputButton`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.0.0")]
pub enum InputButton {
    #[qapi(name = "extra")]
    #[qapi(added = "2.9.0")]
    Extra,
    #[qapi(name = "left")]
    #[qapi(added = "2.6.0")]
    Left,
    #[qapi(name = "Left")]
    #[qapi(removed = "2.6.0")]
    LeftLegacy,
    #[qapi(name = "middle")]
    #[qapi(added = "2.6.0")]
    Middle,
    #[qapi(name = "Middle")]
    #[qapi(removed = "2.6.0")]
    MiddleLegacy,
    #[qapi(name = "right")]
    #[qapi(added = "2.6.0")]
    Right,
    #[qapi(name = "Right")]
    #[qapi(removed = "2.6.0")]
    RightLegacy,
    #[qapi(name = "side")]
    #[qapi(added = "2.9.0")]
    Side,
    #[qapi(name = "touch")]
    #[qapi(added = "8.1.0")]
    Touch,
    #[qapi(name = "wheel-down")]
    #[qapi(added = "2.6.0")]
    WheelDown,
    #[qapi(name = "wheel-left")]
    #[qapi(added = "7.0.0")]
    WheelLeft,
    #[qapi(name = "wheel-right")]
    #[qapi(added = "7.0.0")]
    WheelRight,
    #[qapi(name = "wheel-up")]
    #[qapi(added = "2.6.0")]
    WheelUp,
    #[qapi(name = "WheelDown")]
    #[qapi(removed = "2.6.0")]
    WheelDownLegacy,
    #[qapi(name = "WheelUp")]
    #[qapi(removed = "2.6.0")]
    WheelUpLegacy,
}

/// QMP union `InputEvent`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.0.0")]
#[qapi(simple_union(removed = "6.2.0"))]
pub struct InputEvent {
    #[qapi(discriminator)]
    #[qapi(name = "type")]
    #[qapi(removed = "6.2.0")]
    #[qapi(required)]
    pub r_type: Option<InputEventKind>,
    #[qapi(discriminator)]
    #[qapi(added = "6.2.0")]
    #[qapi(required)]
    pub r#type: Option<InputEventKind>,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<InputEventBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "2.0.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum InputEventBranch {
    #[qapi(name = "abs")]
    #[qapi(added = "6.2.0")]
    Abs(InputMoveEventWrapper),
    #[qapi(name = "abs")]
    #[qapi(removed = "6.2.0")]
    AbsLegacy(InputMoveEvent),
    #[qapi(name = "btn")]
    #[qapi(added = "6.2.0")]
    Btn(InputBtnEventWrapper),
    #[qapi(name = "btn")]
    #[qapi(removed = "6.2.0")]
    BtnLegacy(InputBtnEvent),
    #[qapi(name = "key")]
    #[qapi(added = "6.2.0")]
    Key(InputKeyEventWrapper),
    #[qapi(name = "key")]
    #[qapi(removed = "6.2.0")]
    KeyLegacy(InputKeyEvent),
    #[qapi(name = "mtt")]
    #[qapi(added = "8.1.0")]
    Mtt(InputMultiTouchEventWrapper),
    #[qapi(name = "rel")]
    #[qapi(added = "6.2.0")]
    Rel(InputMoveEventWrapper),
    #[qapi(name = "rel")]
    #[qapi(removed = "6.2.0")]
    RelLegacy(InputMoveEvent),
}

/// QMP enum `InputEventKind`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.0.0")]
pub enum InputEventKind {
    #[qapi(name = "abs")]
    Abs,
    #[qapi(name = "btn")]
    Btn,
    #[qapi(name = "key")]
    Key,
    #[qapi(name = "mtt")]
    #[qapi(added = "8.1.0")]
    Mtt,
    #[qapi(name = "rel")]
    Rel,
}

/// QMP struct `InputKeyEvent`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.0.0")]
pub struct InputKeyEvent {
    pub down: bool,
    pub key: KeyValue,
}

/// QMP struct `InputKeyEventWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct InputKeyEventWrapper {
    pub data: InputKeyEvent,
}

/// QMP struct `InputLinuxProperties`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_LINUX")]
#[qapi(added = "6.0.0")]
pub struct InputLinuxProperties {
    pub evdev: String,
    #[qapi(name = "grab-toggle")]
    pub grab_toggle: Option<GrabToggleKeys>,
    pub grab_all: Option<bool>,
    pub repeat: Option<bool>,
}

/// QMP struct `InputMoveEvent`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.0.0")]
pub struct InputMoveEvent {
    pub axis: InputAxis,
    pub value: i64,
}

/// QMP struct `InputMoveEventWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct InputMoveEventWrapper {
    pub data: InputMoveEvent,
}

/// QMP struct `InputMultiTouchEvent`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.1.0")]
pub struct InputMultiTouchEvent {
    pub axis: InputAxis,
    pub slot: i64,
    #[qapi(name = "tracking-id")]
    pub tracking_id: i64,
    pub r#type: InputMultiTouchType,
    pub value: i64,
}

/// QMP struct `InputMultiTouchEventWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.1.0")]
pub struct InputMultiTouchEventWrapper {
    pub data: InputMultiTouchEvent,
}

/// QMP enum `InputMultiTouchType`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.1.0")]
pub enum InputMultiTouchType {
    #[qapi(name = "begin")]
    Begin,
    #[qapi(name = "cancel")]
    Cancel,
    #[qapi(name = "data")]
    Data,
    #[qapi(name = "end")]
    End,
    #[qapi(name = "update")]
    Update,
}

/// QMP struct `IntWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct IntWrapper {
    pub data: i64,
}

/// QMP struct `IOMMUFDProperties`; see the QEMU QMP Reference Manual.
#[qapi(name = "IOMMUFDProperties")]
#[qapi(added = "9.0.0")]
pub struct IommufdProperties {
    pub fd: Option<String>,
}

/// QMP enum `IoOperationType`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.1.0")]
pub enum IoOperationType {
    #[qapi(name = "read")]
    Read,
    #[qapi(name = "write")]
    Write,
}

/// QMP struct `IOThreadInfo`; see the QEMU QMP Reference Manual.
#[qapi(name = "IOThreadInfo")]
#[qapi(added = "2.0.0")]
pub struct IoThreadInfo {
    #[qapi(name = "aio-max-batch")]
    #[qapi(added = "6.1.0")]
    #[qapi(required)]
    pub aio_max_batch: Option<i64>,
    pub id: String,
    #[qapi(name = "poll-grow")]
    #[qapi(added = "2.9.0")]
    #[qapi(required)]
    pub poll_grow: Option<i64>,
    #[qapi(name = "poll-max-ns")]
    #[qapi(added = "2.9.0")]
    #[qapi(required)]
    pub poll_max_ns: Option<i64>,
    #[qapi(name = "poll-shrink")]
    #[qapi(added = "2.9.0")]
    #[qapi(required)]
    pub poll_shrink: Option<i64>,
    #[qapi(name = "poll-weight")]
    #[qapi(added = "11.1.0")]
    #[qapi(required)]
    pub poll_weight: Option<i64>,
    #[qapi(name = "thread-id")]
    pub thread_id: i64,
}

/// QMP struct `IothreadProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct IothreadProperties {
    #[qapi(name = "aio-max-batch")]
    #[qapi(added = "6.1.0")]
    pub aio_max_batch: Option<i64>,
    #[qapi(name = "poll-grow")]
    pub poll_grow: Option<i64>,
    #[qapi(name = "poll-max-ns")]
    pub poll_max_ns: Option<i64>,
    #[qapi(name = "poll-shrink")]
    pub poll_shrink: Option<i64>,
    #[qapi(name = "poll-weight")]
    #[qapi(added = "11.1.0")]
    pub poll_weight: Option<i64>,
    #[qapi(name = "thread-pool-max")]
    #[qapi(added = "7.1.0")]
    pub thread_pool_max: Option<i64>,
    #[qapi(name = "thread-pool-min")]
    #[qapi(added = "7.1.0")]
    pub thread_pool_min: Option<i64>,
}

/// QMP struct `IOThreadVirtQueueMapping`; see the QEMU QMP Reference Manual.
#[qapi(name = "IOThreadVirtQueueMapping")]
#[qapi(added = "9.0.0")]
pub struct IoThreadVirtQueueMapping {
    pub iothread: String,
    pub vqs: Option<Vec<u16>>,
}

/// QMP enum `IscsiHeaderDigest`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.9.0")]
pub enum IscsiHeaderDigest {
    #[qapi(name = "crc32c")]
    Crc32c,
    #[qapi(name = "crc32c-none")]
    Crc32cNone,
    #[qapi(name = "none")]
    None,
    #[qapi(name = "none-crc32c")]
    NoneCrc32c,
}

/// QMP enum `IscsiTransport`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.9.0")]
pub enum IscsiTransport {
    #[qapi(name = "iser")]
    Iser,
    #[qapi(name = "tcp")]
    Tcp,
}

/// QMP struct `JobInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "3.0.0")]
pub struct JobInfo {
    #[qapi(name = "current-progress")]
    pub current_progress: i64,
    pub error: Option<String>,
    pub id: String,
    pub status: JobStatus,
    #[qapi(name = "total-progress")]
    pub total_progress: i64,
    pub r#type: JobType,
}

/// QMP enum `JobStatus`; see the QEMU QMP Reference Manual.
#[qapi(added = "3.0.0")]
pub enum JobStatus {
    #[qapi(name = "aborting")]
    Aborting,
    #[qapi(name = "concluded")]
    Concluded,
    #[qapi(name = "created")]
    Created,
    #[qapi(name = "null")]
    Null,
    #[qapi(name = "paused")]
    Paused,
    #[qapi(name = "pending")]
    Pending,
    #[qapi(name = "ready")]
    Ready,
    #[qapi(name = "running")]
    Running,
    #[qapi(name = "standby")]
    Standby,
    #[qapi(name = "undefined")]
    Undefined,
    #[qapi(name = "waiting")]
    Waiting,
}

/// QMP enum `JobType`; see the QEMU QMP Reference Manual.
#[qapi(added = "3.0.0")]
pub enum JobType {
    #[qapi(name = "amend")]
    #[qapi(added = "5.1.0")]
    Amend,
    #[qapi(name = "backup")]
    Backup,
    #[qapi(name = "commit")]
    Commit,
    #[qapi(name = "create")]
    Create,
    #[qapi(name = "mirror")]
    Mirror,
    #[qapi(name = "snapshot-delete")]
    #[qapi(added = "6.0.0")]
    SnapshotDelete,
    #[qapi(name = "snapshot-load")]
    #[qapi(added = "6.0.0")]
    SnapshotLoad,
    #[qapi(name = "snapshot-save")]
    #[qapi(added = "6.0.0")]
    SnapshotSave,
    #[qapi(name = "stream")]
    Stream,
}

/// QMP enum `JobVerb`; see the QEMU QMP Reference Manual.
#[qapi(added = "3.0.0")]
pub enum JobVerb {
    #[qapi(name = "cancel")]
    Cancel,
    #[qapi(name = "change")]
    #[qapi(added = "8.2.0")]
    Change,
    #[qapi(name = "complete")]
    Complete,
    #[qapi(name = "dismiss")]
    Dismiss,
    #[qapi(name = "finalize")]
    Finalize,
    #[qapi(name = "pause")]
    Pause,
    #[qapi(name = "resume")]
    Resume,
    #[qapi(name = "set-speed")]
    SetSpeed,
}

/// QMP enum `JSONType`; see the QEMU QMP Reference Manual.
#[qapi(name = "JSONType")]
#[qapi(added = "2.5.0")]
pub enum JsonType {
    #[qapi(name = "array")]
    Array,
    #[qapi(name = "boolean")]
    Boolean,
    #[qapi(name = "int")]
    Int,
    #[qapi(name = "null")]
    Null,
    #[qapi(name = "number")]
    Number,
    #[qapi(name = "object")]
    Object,
    #[qapi(name = "string")]
    String,
    #[qapi(name = "value")]
    Value,
}

/// QMP union `KeyValue`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.3.0")]
#[qapi(simple_union(removed = "6.2.0"))]
pub struct KeyValue {
    #[qapi(discriminator)]
    #[qapi(name = "type")]
    #[qapi(removed = "6.2.0")]
    #[qapi(required)]
    pub r_type: Option<KeyValueKind>,
    #[qapi(discriminator)]
    #[qapi(added = "6.2.0")]
    #[qapi(required)]
    pub r#type: Option<KeyValueKind>,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<KeyValueBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "1.3.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum KeyValueBranch {
    #[qapi(name = "number")]
    #[qapi(added = "6.2.0")]
    Number(IntWrapper),
    #[qapi(name = "number")]
    #[qapi(removed = "6.2.0")]
    NumberLegacy(i64),
    #[qapi(name = "qcode")]
    #[qapi(added = "6.2.0")]
    Qcode(QKeyCodeWrapper),
    #[qapi(name = "qcode")]
    #[qapi(removed = "6.2.0")]
    QcodeLegacy(QKeyCode),
}

/// QMP enum `KeyValueKind`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.3.0")]
pub enum KeyValueKind {
    #[qapi(name = "number")]
    Number,
    #[qapi(name = "qcode")]
    Qcode,
}

/// QMP struct `KvmInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.0.0")]
pub struct KvmInfo {
    pub enabled: bool,
    pub present: bool,
}

/// QMP enum `LostTickPolicy`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.0.0")]
pub enum LostTickPolicy {
    #[qapi(name = "delay")]
    Delay,
    #[qapi(name = "discard")]
    Discard,
    #[qapi(name = "merge")]
    #[qapi(removed = "4.1.0")]
    Merge,
    #[qapi(name = "slew")]
    Slew,
}

/// QMP struct `MachineInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.2.0")]
pub struct MachineInfo {
    #[qapi(added = "8.0.0")]
    #[qapi(required)]
    pub acpi: Option<bool>,
    pub alias: Option<String>,
    #[qapi(name = "compat-props")]
    #[qapi(feature = "unstable")]
    #[qapi(added = "9.1.0")]
    pub compat_props: Option<Vec<CompatProperty>>,
    #[qapi(name = "cpu-max")]
    #[qapi(added = "1.5.0")]
    #[qapi(required)]
    pub cpu_max: Option<i64>,
    #[qapi(name = "default-cpu-type")]
    #[qapi(added = "4.2.0")]
    pub default_cpu_type: Option<String>,
    #[qapi(name = "default-ram-id")]
    #[qapi(added = "5.2.0")]
    pub default_ram_id: Option<String>,
    #[qapi(added = "4.1.0")]
    #[qapi(required)]
    pub deprecated: Option<bool>,
    #[qapi(name = "hotpluggable-cpus")]
    #[qapi(added = "2.7.0")]
    #[qapi(required)]
    pub hotpluggable_cpus: Option<bool>,
    #[qapi(name = "is-default")]
    pub is_default: Option<bool>,
    pub name: String,
    #[qapi(name = "numa-mem-supported")]
    #[qapi(added = "4.1.0")]
    #[qapi(required)]
    pub numa_mem_supported: Option<bool>,
}

/// QMP struct `MainLoopProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.1.0")]
pub struct MainLoopProperties {
    #[qapi(name = "aio-max-batch")]
    pub aio_max_batch: Option<i64>,
    #[qapi(name = "thread-pool-max")]
    pub thread_pool_max: Option<i64>,
    #[qapi(name = "thread-pool-min")]
    pub thread_pool_min: Option<i64>,
}

/// QMP struct `MapEntry`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.6.0")]
pub struct MapEntry {
    #[qapi(added = "8.2.0")]
    #[qapi(required)]
    pub compressed: Option<bool>,
    pub data: bool,
    pub depth: i64,
    pub filename: Option<String>,
    pub length: i64,
    pub offset: Option<i64>,
    #[qapi(added = "6.1.0")]
    #[qapi(required)]
    pub present: Option<bool>,
    pub start: i64,
    pub zero: bool,
}

/// QMP struct `Memdev`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.1.0")]
pub struct Memdev {
    pub dump: bool,
    #[qapi(name = "host-nodes")]
    pub host_nodes: Vec<u16>,
    #[qapi(added = "2.9.0")]
    pub id: Option<String>,
    pub merge: bool,
    pub policy: HostMemPolicy,
    pub prealloc: bool,
    #[qapi(added = "6.1.0")]
    pub reserve: Option<bool>,
    #[qapi(added = "6.1.0")]
    #[qapi(required)]
    pub share: Option<bool>,
    pub size: u64,
}

/// QMP struct `MemoryBackendEpcProperties`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_LINUX")]
#[qapi(added = "6.2.0")]
pub struct MemoryBackendEpcProperties {
    pub dump: Option<bool>,
    #[qapi(name = "host-nodes")]
    pub host_nodes: Option<Vec<u16>>,
    pub merge: Option<bool>,
    pub policy: Option<HostMemPolicy>,
    pub prealloc: Option<bool>,
    #[qapi(name = "prealloc-context")]
    #[qapi(added = "7.2.0")]
    pub prealloc_context: Option<String>,
    #[qapi(name = "prealloc-threads")]
    pub prealloc_threads: Option<u32>,
    pub reserve: Option<bool>,
    pub share: Option<bool>,
    pub size: u64,
    #[qapi(name = "x-use-canonical-path-for-ramblock-id")]
    pub x_use_canonical_path_for_ramblock_id: Option<bool>,
}

/// QMP struct `MemoryBackendFileProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct MemoryBackendFileProperties {
    pub align: Option<u64>,
    #[qapi(name = "discard-data")]
    pub discard_data: Option<bool>,
    pub dump: Option<bool>,
    #[qapi(name = "host-nodes")]
    pub host_nodes: Option<Vec<u16>>,
    #[qapi(name = "mem-path")]
    pub mem_path: String,
    pub merge: Option<bool>,
    #[qapi(added = "8.1.0")]
    pub offset: Option<u64>,
    #[qapi(condition = "CONFIG_LIBPMEM")]
    pub pmem: Option<bool>,
    pub policy: Option<HostMemPolicy>,
    pub prealloc: Option<bool>,
    #[qapi(name = "prealloc-context")]
    #[qapi(added = "7.2.0")]
    pub prealloc_context: Option<String>,
    #[qapi(name = "prealloc-threads")]
    pub prealloc_threads: Option<u32>,
    pub readonly: Option<bool>,
    #[qapi(added = "6.1.0")]
    pub reserve: Option<bool>,
    #[qapi(added = "8.2.0")]
    pub rom: Option<OnOffAuto>,
    pub share: Option<bool>,
    pub size: u64,
    #[qapi(name = "x-use-canonical-path-for-ramblock-id")]
    pub x_use_canonical_path_for_ramblock_id: Option<bool>,
}

/// QMP struct `MemoryBackendMemfdProperties`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_LINUX")]
#[qapi(added = "6.0.0")]
pub struct MemoryBackendMemfdProperties {
    pub dump: Option<bool>,
    #[qapi(name = "host-nodes")]
    pub host_nodes: Option<Vec<u16>>,
    pub hugetlb: Option<bool>,
    pub hugetlbsize: Option<u64>,
    pub merge: Option<bool>,
    pub policy: Option<HostMemPolicy>,
    pub prealloc: Option<bool>,
    #[qapi(name = "prealloc-context")]
    #[qapi(added = "7.2.0")]
    pub prealloc_context: Option<String>,
    #[qapi(name = "prealloc-threads")]
    pub prealloc_threads: Option<u32>,
    #[qapi(added = "6.1.0")]
    pub reserve: Option<bool>,
    pub seal: Option<bool>,
    pub share: Option<bool>,
    pub size: u64,
    #[qapi(name = "x-use-canonical-path-for-ramblock-id")]
    pub x_use_canonical_path_for_ramblock_id: Option<bool>,
}

/// QMP struct `MemoryBackendProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct MemoryBackendProperties {
    pub dump: Option<bool>,
    #[qapi(name = "host-nodes")]
    pub host_nodes: Option<Vec<u16>>,
    pub merge: Option<bool>,
    pub policy: Option<HostMemPolicy>,
    pub prealloc: Option<bool>,
    #[qapi(name = "prealloc-context")]
    #[qapi(added = "7.2.0")]
    pub prealloc_context: Option<String>,
    #[qapi(name = "prealloc-threads")]
    pub prealloc_threads: Option<u32>,
    #[qapi(added = "6.1.0")]
    pub reserve: Option<bool>,
    pub share: Option<bool>,
    pub size: u64,
    #[qapi(name = "x-use-canonical-path-for-ramblock-id")]
    pub x_use_canonical_path_for_ramblock_id: Option<bool>,
}

/// QMP struct `MemoryBackendShmProperties`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_POSIX")]
#[qapi(added = "9.1.0")]
pub struct MemoryBackendShmProperties {
    pub dump: Option<bool>,
    #[qapi(name = "host-nodes")]
    pub host_nodes: Option<Vec<u16>>,
    pub merge: Option<bool>,
    pub policy: Option<HostMemPolicy>,
    pub prealloc: Option<bool>,
    #[qapi(name = "prealloc-context")]
    pub prealloc_context: Option<String>,
    #[qapi(name = "prealloc-threads")]
    pub prealloc_threads: Option<u32>,
    pub reserve: Option<bool>,
    pub share: Option<bool>,
    pub size: u64,
    #[qapi(name = "x-use-canonical-path-for-ramblock-id")]
    pub x_use_canonical_path_for_ramblock_id: Option<bool>,
}

/// QMP union `MemoryDeviceInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.1.0")]
#[qapi(simple_union(removed = "6.2.0"))]
pub struct MemoryDeviceInfo {
    #[qapi(discriminator)]
    #[qapi(name = "type")]
    #[qapi(removed = "6.2.0")]
    #[qapi(required)]
    pub r_type: Option<MemoryDeviceInfoKind>,
    #[qapi(discriminator)]
    #[qapi(added = "6.2.0")]
    #[qapi(required)]
    pub r#type: Option<MemoryDeviceInfoKind>,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<MemoryDeviceInfoBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "2.1.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum MemoryDeviceInfoBranch {
    #[qapi(name = "dimm")]
    #[qapi(added = "6.2.0")]
    Dimm(PcdimmDeviceInfoWrapper),
    #[qapi(name = "dimm")]
    #[qapi(removed = "6.2.0")]
    DimmLegacy(PcdimmDeviceInfo),
    #[qapi(name = "hv-balloon")]
    #[qapi(added = "8.2.0")]
    HvBalloon(HvBalloonDeviceInfoWrapper),
    #[qapi(name = "nvdimm")]
    #[qapi(added = "6.2.0")]
    Nvdimm(PcdimmDeviceInfoWrapper),
    #[qapi(name = "nvdimm")]
    #[qapi(added = "2.12.0")]
    #[qapi(removed = "6.2.0")]
    NvdimmLegacy(PcdimmDeviceInfo),
    #[qapi(name = "sgx-epc")]
    #[qapi(added = "6.2.0")]
    SgxEpc(SgxEpcDeviceInfoWrapper),
    #[qapi(name = "sp-mem")]
    #[qapi(added = "11.1.0")]
    SpMem(SpMemDeviceInfoWrapper),
    #[qapi(name = "virtio-mem")]
    #[qapi(added = "6.2.0")]
    VirtioMem(VirtioMemDeviceInfoWrapper),
    #[qapi(name = "virtio-pmem")]
    #[qapi(added = "6.2.0")]
    VirtioPmem(VirtioPmemDeviceInfoWrapper),
    #[qapi(name = "virtio-mem")]
    #[qapi(added = "5.1.0")]
    #[qapi(removed = "6.2.0")]
    VirtioMemLegacy(VirtioMemDeviceInfo),
    #[qapi(name = "virtio-pmem")]
    #[qapi(added = "4.1.0")]
    #[qapi(removed = "6.2.0")]
    VirtioPmemLegacy(VirtioPmemDeviceInfo),
}

/// QMP enum `MemoryDeviceInfoKind`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.1.0")]
pub enum MemoryDeviceInfoKind {
    #[qapi(name = "dimm")]
    Dimm,
    #[qapi(name = "hv-balloon")]
    #[qapi(added = "8.2.0")]
    HvBalloon,
    #[qapi(name = "nvdimm")]
    #[qapi(added = "2.12.0")]
    Nvdimm,
    #[qapi(name = "sgx-epc")]
    #[qapi(added = "6.2.0")]
    SgxEpc,
    #[qapi(name = "sp-mem")]
    #[qapi(added = "11.1.0")]
    SpMem,
    #[qapi(name = "virtio-mem")]
    #[qapi(added = "5.1.0")]
    VirtioMem,
    #[qapi(name = "virtio-pmem")]
    #[qapi(added = "4.1.0")]
    VirtioPmem,
}

/// QMP enum `MemoryFailureAction`; see the QEMU QMP Reference Manual.
#[qapi(added = "5.2.0")]
pub enum MemoryFailureAction {
    #[qapi(name = "fatal")]
    Fatal,
    #[qapi(name = "ignore")]
    Ignore,
    #[qapi(name = "inject")]
    Inject,
    #[qapi(name = "reset")]
    Reset,
}

/// QMP struct `MemoryFailureFlags`; see the QEMU QMP Reference Manual.
#[qapi(added = "5.2.0")]
pub struct MemoryFailureFlags {
    #[qapi(name = "action-required")]
    pub action_required: bool,
    pub recursive: bool,
}

/// QMP enum `MemoryFailureRecipient`; see the QEMU QMP Reference Manual.
#[qapi(added = "5.2.0")]
pub enum MemoryFailureRecipient {
    #[qapi(name = "guest")]
    Guest,
    #[qapi(name = "hypervisor")]
    Hypervisor,
}

/// QMP struct `MemoryInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.11.0")]
pub struct MemoryInfo {
    #[qapi(name = "base-memory")]
    pub base_memory: u64,
    #[qapi(name = "plugged-memory")]
    pub plugged_memory: Option<u64>,
}

/// QMP struct `MemorySizeConfiguration`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.1.0")]
pub struct MemorySizeConfiguration {
    #[qapi(name = "max-size")]
    pub max_size: Option<u64>,
    pub size: Option<u64>,
    pub slots: Option<u64>,
}

/// QMP enum `MigMode`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.2.0")]
pub enum MigMode {
    #[qapi(name = "cpr-exec")]
    #[qapi(added = "10.2.0")]
    CprExec,
    #[qapi(name = "cpr-reboot")]
    CprReboot,
    #[qapi(name = "cpr-transfer")]
    #[qapi(added = "10.0.0")]
    CprTransfer,
    #[qapi(name = "normal")]
    Normal,
}

/// QMP union `MigrationAddress`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.2.0")]
pub struct MigrationAddress {
    #[qapi(discriminator)]
    pub transport: MigrationAddressType,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<MigrationAddressBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "8.2.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum MigrationAddressBranch {
    #[qapi(name = "exec")]
    Exec(MigrationExecCommand),
    #[qapi(name = "file")]
    File(FileMigrationArgs),
    #[qapi(name = "rdma")]
    Rdma(InetSocketAddress),
    #[qapi(name = "socket")]
    Socket(SocketAddress),
}

/// QMP enum `MigrationAddressType`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.2.0")]
pub enum MigrationAddressType {
    #[qapi(name = "exec")]
    Exec,
    #[qapi(name = "file")]
    File,
    #[qapi(name = "rdma")]
    Rdma,
    #[qapi(name = "socket")]
    Socket,
}

/// QMP enum `MigrationCapability`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.2.0")]
pub enum MigrationCapability {
    #[qapi(name = "auto-converge")]
    #[qapi(added = "1.6.0")]
    AutoConverge,
    #[qapi(name = "background-snapshot")]
    #[qapi(added = "6.0.0")]
    BackgroundSnapshot,
    #[qapi(name = "block")]
    #[qapi(added = "2.10.0")]
    #[qapi(deprecated(added = "8.2.0"))]
    #[qapi(removed = "9.1.0")]
    Block,
    #[qapi(name = "compress")]
    #[qapi(added = "2.4.0")]
    #[qapi(deprecated(added = "8.2.0"))]
    #[qapi(removed = "9.1.0")]
    Compress,
    #[qapi(name = "dirty-bitmaps")]
    #[qapi(added = "2.12.0")]
    DirtyBitmaps,
    #[qapi(name = "dirty-limit")]
    #[qapi(added = "8.1.0")]
    DirtyLimit,
    #[qapi(name = "events")]
    #[qapi(added = "2.4.0")]
    Events,
    #[qapi(name = "late-block-activate")]
    #[qapi(added = "3.0.0")]
    LateBlockActivate,
    #[qapi(name = "mapped-ram")]
    #[qapi(added = "9.0.0")]
    MappedRam,
    #[qapi(name = "multifd")]
    #[qapi(added = "4.0.0")]
    Multifd,
    #[qapi(name = "pause-before-switchover")]
    #[qapi(added = "2.11.0")]
    PauseBeforeSwitchover,
    #[qapi(name = "postcopy-blocktime")]
    #[qapi(added = "3.0.0")]
    PostcopyBlocktime,
    #[qapi(name = "postcopy-preempt")]
    #[qapi(added = "7.1.0")]
    PostcopyPreempt,
    #[qapi(name = "postcopy-ram")]
    #[qapi(added = "2.6.0")]
    PostcopyRam,
    #[qapi(name = "rdma-pin-all")]
    #[qapi(added = "2.0.0")]
    RdmaPinAll,
    #[qapi(name = "release-ram")]
    #[qapi(added = "2.9.0")]
    ReleaseRam,
    #[qapi(name = "return-path")]
    #[qapi(added = "2.10.0")]
    ReturnPath,
    #[qapi(name = "switchover-ack")]
    #[qapi(added = "8.1.0")]
    SwitchoverAck,
    #[qapi(name = "validate-uuid")]
    #[qapi(added = "4.2.0")]
    ValidateUuid,
    #[qapi(name = "x-colo")]
    #[qapi(feature = "unstable")]
    #[qapi(added = "2.8.0")]
    XColo,
    #[qapi(name = "x-ignore-shared")]
    #[qapi(feature = "unstable")]
    #[qapi(added = "4.0.0")]
    XIgnoreShared,
    #[qapi(name = "x-multifd")]
    #[qapi(added = "2.11.0")]
    #[qapi(removed = "4.0.0")]
    XMultifd,
    #[qapi(name = "x-postcopy-ram")]
    #[qapi(added = "2.5.0")]
    #[qapi(removed = "2.6.0")]
    XPostcopyRam,
    #[qapi(name = "x-rdma-pin-all")]
    #[qapi(added = "1.6.0")]
    #[qapi(removed = "2.0.0")]
    XRdmaPinAll,
    #[qapi(name = "xbzrle")]
    Xbzrle,
    #[qapi(name = "zero-blocks")]
    #[qapi(added = "1.6.0")]
    #[qapi(deprecated(added = "9.2.0"))]
    #[qapi(removed = "11.0.0")]
    ZeroBlocks,
    #[qapi(name = "zero-copy-send")]
    #[qapi(added = "7.1.0")]
    ZeroCopySend,
}

/// QMP struct `MigrationCapabilityStatus`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.2.0")]
pub struct MigrationCapabilityStatus {
    pub capability: MigrationCapability,
    pub state: bool,
}

/// QMP struct `MigrationChannel`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.2.0")]
pub struct MigrationChannel {
    pub addr: MigrationAddress,
    #[qapi(name = "channel-type")]
    pub channel_type: MigrationChannelType,
}

/// QMP enum `MigrationChannelType`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.2.0")]
pub enum MigrationChannelType {
    #[qapi(name = "cpr")]
    #[qapi(added = "10.0.0")]
    Cpr,
    #[qapi(name = "main")]
    Main,
}

/// QMP struct `MigrationExecCommand`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.2.0")]
pub struct MigrationExecCommand {
    pub args: Vec<String>,
}

/// QMP struct `MigrationInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.0.0")]
pub struct MigrationInfo {
    #[qapi(added = "6.0.0")]
    #[qapi(deprecated(added = "6.0.0"))]
    #[qapi(removed = "6.1.0")]
    #[qapi(required)]
    pub blocked: Option<bool>,
    #[qapi(name = "blocked-reasons")]
    #[qapi(added = "6.0.0")]
    pub blocked_reasons: Option<Vec<String>>,
    #[qapi(added = "3.1.0")]
    #[qapi(deprecated(added = "8.2.0"))]
    #[qapi(removed = "9.1.0")]
    pub compression: Option<CompressionStats>,
    #[qapi(name = "cpu-throttle-percentage")]
    #[qapi(added = "2.7.0")]
    pub cpu_throttle_percentage: Option<i64>,
    #[qapi(name = "dirty-limit-ring-full-time")]
    #[qapi(added = "8.1.0")]
    pub dirty_limit_ring_full_time: Option<u64>,
    #[qapi(name = "dirty-limit-throttle-time-per-round")]
    #[qapi(added = "8.1.0")]
    pub dirty_limit_throttle_time_per_round: Option<u64>,
    #[qapi(deprecated(added = "8.2.0"))]
    #[qapi(removed = "9.1.0")]
    pub disk: Option<MigrationStats>,
    #[qapi(added = "1.3.0")]
    pub downtime: Option<i64>,
    #[qapi(name = "error-desc")]
    #[qapi(added = "2.7.0")]
    pub error_desc: Option<String>,
    #[qapi(name = "expected-downtime")]
    #[qapi(added = "1.3.0")]
    pub expected_downtime: Option<i64>,
    #[qapi(name = "postcopy-blocktime")]
    #[qapi(added = "3.0.0")]
    pub postcopy_blocktime: Option<u32>,
    #[qapi(name = "postcopy-latency")]
    #[qapi(feature = "unstable")]
    #[qapi(added = "10.1.0")]
    pub postcopy_latency: Option<u64>,
    #[qapi(name = "postcopy-latency-dist")]
    #[qapi(feature = "unstable")]
    #[qapi(added = "10.1.0")]
    pub postcopy_latency_dist: Option<Vec<u64>>,
    #[qapi(name = "postcopy-non-vcpu-latency")]
    #[qapi(feature = "unstable")]
    #[qapi(added = "10.1.0")]
    pub postcopy_non_vcpu_latency: Option<u64>,
    #[qapi(name = "postcopy-vcpu-blocktime")]
    #[qapi(added = "3.0.0")]
    pub postcopy_vcpu_blocktime: Option<Vec<u32>>,
    #[qapi(name = "postcopy-vcpu-latency")]
    #[qapi(feature = "unstable")]
    #[qapi(added = "10.1.0")]
    pub postcopy_vcpu_latency: Option<Vec<u64>>,
    #[qapi(added = "11.1.0")]
    pub ram: Option<MigrationRamStats>,
    #[qapi(name = "ram")]
    #[qapi(removed = "11.1.0")]
    pub ram_legacy: Option<MigrationStats>,
    #[qapi(added = "11.1.0")]
    pub remaining: Option<u64>,
    #[qapi(name = "setup-time")]
    #[qapi(added = "1.6.0")]
    pub setup_time: Option<i64>,
    #[qapi(name = "socket-address")]
    #[qapi(added = "4.0.0")]
    pub socket_address: Option<Vec<SocketAddress>>,
    #[qapi(added = "2.3.0")]
    pub status: Option<MigrationStatus>,
    #[qapi(name = "status")]
    #[qapi(removed = "2.3.0")]
    pub status_legacy: Option<String>,
    #[qapi(name = "total-time")]
    #[qapi(added = "1.2.0")]
    pub total_time: Option<i64>,
    #[qapi(added = "5.2.0")]
    pub vfio: Option<VfioStats>,
    #[qapi(name = "x-cpu-throttle-percentage")]
    #[qapi(added = "2.5.0")]
    #[qapi(removed = "2.7.0")]
    pub x_cpu_throttle_percentage: Option<i64>,
    #[qapi(name = "xbzrle-cache")]
    #[qapi(added = "1.2.0")]
    pub xbzrle_cache: Option<XbzrleCacheStats>,
}

/// QMP enum `MigrationParameter`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.4.0")]
pub enum MigrationParameter {
    #[qapi(name = "announce-initial")]
    #[qapi(added = "4.0.0")]
    AnnounceInitial,
    #[qapi(name = "announce-max")]
    #[qapi(added = "4.0.0")]
    AnnounceMax,
    #[qapi(name = "announce-rounds")]
    #[qapi(added = "4.0.0")]
    AnnounceRounds,
    #[qapi(name = "announce-step")]
    #[qapi(added = "4.0.0")]
    AnnounceStep,
    #[qapi(name = "avail-switchover-bandwidth")]
    #[qapi(added = "8.2.0")]
    AvailSwitchoverBandwidth,
    #[qapi(name = "block-bitmap-mapping")]
    #[qapi(added = "5.2.0")]
    BlockBitmapMapping,
    #[qapi(name = "block-incremental")]
    #[qapi(added = "2.10.0")]
    #[qapi(deprecated(added = "8.2.0"))]
    #[qapi(removed = "9.1.0")]
    BlockIncremental,
    #[qapi(name = "compress-level")]
    #[qapi(deprecated(added = "8.2.0"))]
    #[qapi(removed = "9.1.0")]
    CompressLevel,
    #[qapi(name = "compress-threads")]
    #[qapi(deprecated(added = "8.2.0"))]
    #[qapi(removed = "9.1.0")]
    CompressThreads,
    #[qapi(name = "compress-wait-thread")]
    #[qapi(added = "3.1.0")]
    #[qapi(deprecated(added = "8.2.0"))]
    #[qapi(removed = "9.1.0")]
    CompressWaitThread,
    #[qapi(name = "cpr-exec-command")]
    #[qapi(added = "10.2.0")]
    CprExecCommand,
    #[qapi(name = "cpu-throttle-increment")]
    #[qapi(added = "2.7.0")]
    CpuThrottleIncrement,
    #[qapi(name = "cpu-throttle-initial")]
    #[qapi(added = "2.7.0")]
    CpuThrottleInitial,
    #[qapi(name = "cpu-throttle-tailslow")]
    #[qapi(added = "5.1.0")]
    CpuThrottleTailslow,
    #[qapi(name = "decompress-threads")]
    #[qapi(deprecated(added = "8.2.0"))]
    #[qapi(removed = "9.1.0")]
    DecompressThreads,
    #[qapi(name = "direct-io")]
    #[qapi(added = "9.1.0")]
    DirectIo,
    #[qapi(name = "downtime-limit")]
    #[qapi(added = "2.8.0")]
    DowntimeLimit,
    #[qapi(name = "max-bandwidth")]
    #[qapi(added = "2.8.0")]
    MaxBandwidth,
    #[qapi(name = "max-cpu-throttle")]
    #[qapi(added = "3.1.0")]
    MaxCpuThrottle,
    #[qapi(name = "max-postcopy-bandwidth")]
    #[qapi(added = "3.0.0")]
    MaxPostcopyBandwidth,
    #[qapi(name = "mode")]
    #[qapi(added = "8.2.0")]
    Mode,
    #[qapi(name = "multifd-channels")]
    #[qapi(added = "4.0.0")]
    MultifdChannels,
    #[qapi(name = "multifd-compression")]
    #[qapi(added = "5.0.0")]
    MultifdCompression,
    #[qapi(name = "multifd-qatzip-level")]
    #[qapi(added = "9.2.0")]
    MultifdQatzipLevel,
    #[qapi(name = "multifd-zlib-level")]
    #[qapi(added = "5.0.0")]
    MultifdZlibLevel,
    #[qapi(name = "multifd-zstd-level")]
    #[qapi(added = "5.0.0")]
    MultifdZstdLevel,
    #[qapi(name = "throttle-trigger-threshold")]
    #[qapi(added = "5.0.0")]
    ThrottleTriggerThreshold,
    #[qapi(name = "tls-authz")]
    #[qapi(added = "4.0.0")]
    TlsAuthz,
    #[qapi(name = "tls-creds")]
    #[qapi(added = "2.7.0")]
    TlsCreds,
    #[qapi(name = "tls-hostname")]
    #[qapi(added = "2.7.0")]
    TlsHostname,
    #[qapi(name = "vcpu-dirty-limit")]
    #[qapi(added = "8.1.0")]
    VcpuDirtyLimit,
    #[qapi(name = "x-checkpoint-delay")]
    #[qapi(feature = "unstable")]
    #[qapi(added = "2.8.0")]
    XCheckpointDelay,
    #[qapi(name = "x-cpu-throttle-increment")]
    #[qapi(added = "2.5.0")]
    #[qapi(removed = "2.7.0")]
    XCpuThrottleIncrement,
    #[qapi(name = "x-cpu-throttle-initial")]
    #[qapi(added = "2.5.0")]
    #[qapi(removed = "2.7.0")]
    XCpuThrottleInitial,
    #[qapi(name = "x-multifd-channels")]
    #[qapi(added = "2.11.0")]
    #[qapi(removed = "4.0.0")]
    XMultifdChannels,
    #[qapi(name = "x-multifd-page-count")]
    #[qapi(added = "2.11.0")]
    #[qapi(removed = "4.0.0")]
    XMultifdPageCount,
    #[qapi(name = "x-rdma-chunk-size")]
    #[qapi(feature = "unstable")]
    #[qapi(added = "11.1.0")]
    XRdmaChunkSize,
    #[qapi(name = "x-vcpu-dirty-limit-period")]
    #[qapi(feature = "unstable")]
    #[qapi(added = "8.1.0")]
    XVcpuDirtyLimitPeriod,
    #[qapi(name = "xbzrle-cache-size")]
    #[qapi(added = "2.11.0")]
    XbzrleCacheSize,
    #[qapi(name = "zero-page-detection")]
    #[qapi(added = "9.0.0")]
    ZeroPageDetection,
}

/// QMP struct `MigrationParameters`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.4.0")]
pub struct MigrationParameters {
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
    #[qapi(added = "2.12.0")]
    #[qapi(deprecated(added = "8.2.0"))]
    #[qapi(removed = "9.1.0")]
    pub compress_level: Option<u8>,
    #[qapi(name = "compress-threads")]
    #[qapi(added = "2.12.0")]
    #[qapi(deprecated(added = "8.2.0"))]
    #[qapi(removed = "9.1.0")]
    pub compress_threads: Option<u8>,
    #[qapi(name = "compress-wait-thread")]
    #[qapi(added = "3.1.0")]
    #[qapi(deprecated(added = "8.2.0"))]
    #[qapi(removed = "9.1.0")]
    pub compress_wait_thread: Option<bool>,
    #[qapi(name = "compress-level")]
    #[qapi(removed = "2.8.0")]
    #[qapi(required)]
    pub compress_level_legacy: Option<i64>,
    #[qapi(name = "compress-level")]
    #[qapi(added = "2.8.0")]
    #[qapi(removed = "2.12.0")]
    pub compress_level_legacy2: Option<i64>,
    #[qapi(name = "compress-threads")]
    #[qapi(removed = "2.8.0")]
    #[qapi(required)]
    pub compress_threads_legacy: Option<i64>,
    #[qapi(name = "compress-threads")]
    #[qapi(added = "2.8.0")]
    #[qapi(removed = "2.12.0")]
    pub compress_threads_legacy2: Option<i64>,
    #[qapi(name = "cpr-exec-command")]
    #[qapi(added = "10.2.0")]
    pub cpr_exec_command: Option<Vec<String>>,
    #[qapi(name = "cpu-throttle-increment")]
    #[qapi(added = "2.12.0")]
    pub cpu_throttle_increment: Option<u8>,
    #[qapi(name = "cpu-throttle-initial")]
    #[qapi(added = "2.12.0")]
    pub cpu_throttle_initial: Option<u8>,
    #[qapi(name = "cpu-throttle-tailslow")]
    #[qapi(added = "5.1.0")]
    pub cpu_throttle_tailslow: Option<bool>,
    #[qapi(name = "cpu-throttle-increment")]
    #[qapi(added = "2.7.0")]
    #[qapi(removed = "2.8.0")]
    #[qapi(required)]
    pub cpu_throttle_increment_legacy: Option<i64>,
    #[qapi(name = "cpu-throttle-increment")]
    #[qapi(added = "2.8.0")]
    #[qapi(removed = "2.12.0")]
    pub cpu_throttle_increment_legacy2: Option<i64>,
    #[qapi(name = "cpu-throttle-initial")]
    #[qapi(added = "2.7.0")]
    #[qapi(removed = "2.8.0")]
    #[qapi(required)]
    pub cpu_throttle_initial_legacy: Option<i64>,
    #[qapi(name = "cpu-throttle-initial")]
    #[qapi(added = "2.8.0")]
    #[qapi(removed = "2.12.0")]
    pub cpu_throttle_initial_legacy2: Option<i64>,
    #[qapi(name = "decompress-threads")]
    #[qapi(added = "2.12.0")]
    #[qapi(deprecated(added = "8.2.0"))]
    #[qapi(removed = "9.1.0")]
    pub decompress_threads: Option<u8>,
    #[qapi(name = "decompress-threads")]
    #[qapi(removed = "2.8.0")]
    #[qapi(required)]
    pub decompress_threads_legacy: Option<i64>,
    #[qapi(name = "decompress-threads")]
    #[qapi(added = "2.8.0")]
    #[qapi(removed = "2.12.0")]
    pub decompress_threads_legacy2: Option<i64>,
    #[qapi(name = "direct-io")]
    #[qapi(added = "9.1.0")]
    pub direct_io: Option<bool>,
    #[qapi(name = "downtime-limit")]
    #[qapi(added = "2.12.0")]
    pub downtime_limit: Option<u64>,
    #[qapi(name = "downtime-limit")]
    #[qapi(added = "2.8.0")]
    #[qapi(removed = "2.12.0")]
    pub downtime_limit_legacy: Option<i64>,
    #[qapi(name = "max-bandwidth")]
    #[qapi(added = "2.12.0")]
    pub max_bandwidth: Option<u64>,
    #[qapi(name = "max-cpu-throttle")]
    #[qapi(added = "3.1.0")]
    pub max_cpu_throttle: Option<u8>,
    #[qapi(name = "max-postcopy-bandwidth")]
    #[qapi(added = "3.0.0")]
    pub max_postcopy_bandwidth: Option<u64>,
    #[qapi(name = "max-bandwidth")]
    #[qapi(added = "2.8.0")]
    #[qapi(removed = "2.12.0")]
    pub max_bandwidth_legacy: Option<i64>,
    #[qapi(added = "8.2.0")]
    pub mode: Option<MigMode>,
    #[qapi(name = "multifd-channels")]
    #[qapi(added = "4.0.0")]
    pub multifd_channels: Option<u8>,
    #[qapi(name = "multifd-compression")]
    #[qapi(added = "5.0.0")]
    pub multifd_compression: Option<MultiFdCompression>,
    #[qapi(name = "multifd-qatzip-level")]
    #[qapi(added = "9.2.0")]
    pub multifd_qatzip_level: Option<u8>,
    #[qapi(name = "multifd-zlib-level")]
    #[qapi(added = "5.0.0")]
    pub multifd_zlib_level: Option<u8>,
    #[qapi(name = "multifd-zstd-level")]
    #[qapi(added = "5.0.0")]
    pub multifd_zstd_level: Option<u8>,
    #[qapi(name = "throttle-trigger-threshold")]
    #[qapi(added = "5.0.0")]
    pub throttle_trigger_threshold: Option<u8>,
    #[qapi(name = "tls-authz")]
    #[qapi(added = "11.0.0")]
    pub tls_authz: Option<StrOrNull>,
    #[qapi(name = "tls-creds")]
    #[qapi(added = "11.0.0")]
    pub tls_creds: Option<StrOrNull>,
    #[qapi(name = "tls-hostname")]
    #[qapi(added = "11.0.0")]
    pub tls_hostname: Option<StrOrNull>,
    #[qapi(name = "tls-authz")]
    #[qapi(added = "4.0.0")]
    #[qapi(removed = "11.0.0")]
    pub tls_authz_legacy: Option<String>,
    #[qapi(name = "tls-creds")]
    #[qapi(added = "2.7.0")]
    #[qapi(removed = "2.8.0")]
    #[qapi(required)]
    pub tls_creds_legacy: Option<String>,
    #[qapi(name = "tls-creds")]
    #[qapi(added = "2.8.0")]
    #[qapi(removed = "11.0.0")]
    pub tls_creds_legacy2: Option<String>,
    #[qapi(name = "tls-hostname")]
    #[qapi(added = "2.7.0")]
    #[qapi(removed = "2.8.0")]
    #[qapi(required)]
    pub tls_hostname_legacy: Option<String>,
    #[qapi(name = "tls-hostname")]
    #[qapi(added = "2.8.0")]
    #[qapi(removed = "11.0.0")]
    pub tls_hostname_legacy2: Option<String>,
    #[qapi(name = "vcpu-dirty-limit")]
    #[qapi(added = "8.1.0")]
    pub vcpu_dirty_limit: Option<u64>,
    #[qapi(name = "x-checkpoint-delay")]
    #[qapi(feature = "unstable")]
    #[qapi(added = "2.12.0")]
    pub x_checkpoint_delay: Option<u32>,
    #[qapi(name = "x-cpu-throttle-increment")]
    #[qapi(added = "2.5.0")]
    #[qapi(removed = "2.7.0")]
    #[qapi(required)]
    pub x_cpu_throttle_increment: Option<i64>,
    #[qapi(name = "x-cpu-throttle-initial")]
    #[qapi(added = "2.5.0")]
    #[qapi(removed = "2.7.0")]
    #[qapi(required)]
    pub x_cpu_throttle_initial: Option<i64>,
    #[qapi(name = "x-multifd-channels")]
    #[qapi(added = "2.12.0")]
    #[qapi(removed = "4.0.0")]
    pub x_multifd_channels: Option<u8>,
    #[qapi(name = "x-multifd-page-count")]
    #[qapi(added = "2.12.0")]
    #[qapi(removed = "4.0.0")]
    pub x_multifd_page_count: Option<u32>,
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
    #[qapi(removed = "2.12.0")]
    pub x_checkpoint_delay_legacy: Option<i64>,
    #[qapi(name = "x-multifd-channels")]
    #[qapi(added = "2.11.0")]
    #[qapi(removed = "2.12.0")]
    pub x_multifd_channels_legacy: Option<i64>,
    #[qapi(name = "x-multifd-page-count")]
    #[qapi(added = "2.11.0")]
    #[qapi(removed = "2.12.0")]
    pub x_multifd_page_count_legacy: Option<i64>,
    #[qapi(name = "xbzrle-cache-size")]
    #[qapi(added = "2.11.0")]
    pub xbzrle_cache_size: Option<u64>,
    #[qapi(name = "zero-page-detection")]
    #[qapi(added = "9.0.0")]
    pub zero_page_detection: Option<ZeroPageDetection>,
}

/// QMP struct `MigrationRAMStats`; see the QEMU QMP Reference Manual.
#[qapi(name = "MigrationRAMStats")]
#[qapi(added = "11.1.0")]
pub struct MigrationRamStats {
    #[qapi(name = "dirty-pages-rate")]
    pub dirty_pages_rate: i64,
    #[qapi(name = "dirty-sync-count")]
    pub dirty_sync_count: i64,
    #[qapi(name = "dirty-sync-missed-zero-copy")]
    pub dirty_sync_missed_zero_copy: u64,
    #[qapi(name = "downtime-bytes")]
    pub downtime_bytes: u64,
    pub duplicate: i64,
    pub mbps: f64,
    #[qapi(name = "multifd-bytes")]
    pub multifd_bytes: u64,
    pub normal: i64,
    #[qapi(name = "normal-bytes")]
    pub normal_bytes: i64,
    #[qapi(name = "page-size")]
    pub page_size: i64,
    #[qapi(name = "pages-per-second")]
    pub pages_per_second: u64,
    #[qapi(name = "postcopy-bytes")]
    pub postcopy_bytes: u64,
    #[qapi(name = "postcopy-requests")]
    pub postcopy_requests: i64,
    #[qapi(name = "precopy-bytes")]
    pub precopy_bytes: u64,
    pub remaining: i64,
    pub total: i64,
    pub transferred: i64,
}

/// QMP struct `MigrationStats`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.0.0")]
#[qapi(removed = "11.1.0")]
pub struct MigrationStats {
    #[qapi(name = "dirty-pages-rate")]
    #[qapi(added = "1.3.0")]
    #[qapi(required)]
    pub dirty_pages_rate: Option<i64>,
    #[qapi(name = "dirty-sync-count")]
    #[qapi(added = "2.1.0")]
    #[qapi(required)]
    pub dirty_sync_count: Option<i64>,
    #[qapi(name = "dirty-sync-missed-zero-copy")]
    #[qapi(added = "7.1.0")]
    #[qapi(required)]
    pub dirty_sync_missed_zero_copy: Option<u64>,
    #[qapi(name = "downtime-bytes")]
    #[qapi(added = "7.0.0")]
    #[qapi(required)]
    pub downtime_bytes: Option<u64>,
    #[qapi(added = "1.2.0")]
    #[qapi(required)]
    pub duplicate: Option<i64>,
    #[qapi(added = "1.6.0")]
    #[qapi(required)]
    pub mbps: Option<f64>,
    #[qapi(name = "multifd-bytes")]
    #[qapi(added = "3.0.0")]
    #[qapi(required)]
    pub multifd_bytes: Option<u64>,
    #[qapi(added = "1.2.0")]
    #[qapi(required)]
    pub normal: Option<i64>,
    #[qapi(name = "normal-bytes")]
    #[qapi(added = "1.2.0")]
    #[qapi(required)]
    pub normal_bytes: Option<i64>,
    #[qapi(name = "page-size")]
    #[qapi(added = "2.10.0")]
    #[qapi(required)]
    pub page_size: Option<i64>,
    #[qapi(name = "pages-per-second")]
    #[qapi(added = "4.0.0")]
    #[qapi(required)]
    pub pages_per_second: Option<u64>,
    #[qapi(name = "postcopy-bytes")]
    #[qapi(added = "7.0.0")]
    #[qapi(required)]
    pub postcopy_bytes: Option<u64>,
    #[qapi(name = "postcopy-requests")]
    #[qapi(added = "2.7.0")]
    #[qapi(required)]
    pub postcopy_requests: Option<i64>,
    #[qapi(name = "precopy-bytes")]
    #[qapi(added = "7.0.0")]
    #[qapi(required)]
    pub precopy_bytes: Option<u64>,
    pub remaining: i64,
    #[qapi(added = "1.5.0")]
    #[qapi(deprecated(added = "8.1.0"))]
    #[qapi(removed = "9.1.0")]
    #[qapi(required)]
    pub skipped: Option<i64>,
    pub total: i64,
    pub transferred: i64,
}

/// QMP enum `MigrationStatus`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.3.0")]
pub enum MigrationStatus {
    #[qapi(name = "active")]
    Active,
    #[qapi(name = "cancelled")]
    Cancelled,
    #[qapi(name = "cancelling")]
    Cancelling,
    #[qapi(name = "colo")]
    #[qapi(added = "2.8.0")]
    Colo,
    #[qapi(name = "completed")]
    Completed,
    #[qapi(name = "device")]
    #[qapi(added = "2.11.0")]
    Device,
    #[qapi(name = "failed")]
    Failed,
    #[qapi(name = "failing")]
    #[qapi(added = "11.0.0")]
    Failing,
    #[qapi(name = "none")]
    None,
    #[qapi(name = "postcopy-active")]
    #[qapi(added = "2.5.0")]
    PostcopyActive,
    #[qapi(name = "postcopy-device")]
    #[qapi(added = "10.2.0")]
    PostcopyDevice,
    #[qapi(name = "postcopy-paused")]
    #[qapi(added = "3.0.0")]
    PostcopyPaused,
    #[qapi(name = "postcopy-recover")]
    #[qapi(added = "3.0.0")]
    PostcopyRecover,
    #[qapi(name = "postcopy-recover-setup")]
    #[qapi(added = "9.1.0")]
    PostcopyRecoverSetup,
    #[qapi(name = "pre-switchover")]
    #[qapi(added = "2.11.0")]
    PreSwitchover,
    #[qapi(name = "setup")]
    Setup,
    #[qapi(name = "wait-unplug")]
    #[qapi(added = "4.2.0")]
    WaitUnplug,
}

/// QMP struct `MigrationThreadInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.0.0")]
#[qapi(removed = "11.0.0")]
pub struct MigrationThreadInfo {
    pub name: String,
    #[qapi(name = "thread-id")]
    pub thread_id: i64,
}

/// QMP enum `MirrorCopyMode`; see the QEMU QMP Reference Manual.
#[qapi(added = "3.0.0")]
pub enum MirrorCopyMode {
    #[qapi(name = "background")]
    Background,
    #[qapi(name = "write-blocking")]
    WriteBlocking,
}

/// QMP enum `MirrorSyncMode`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.3.0")]
pub enum MirrorSyncMode {
    #[qapi(name = "bitmap")]
    #[qapi(added = "4.2.0")]
    Bitmap,
    #[qapi(name = "full")]
    Full,
    #[qapi(name = "incremental")]
    #[qapi(added = "2.4.0")]
    Incremental,
    #[qapi(name = "none")]
    None,
    #[qapi(name = "top")]
    Top,
}

/// QMP struct `MonitorHMPProperties`; see the QEMU QMP Reference Manual.
#[qapi(name = "MonitorHMPProperties")]
#[qapi(added = "11.1.0")]
pub struct MonitorHmpProperties {
    pub chardev: String,
    pub readline: Option<bool>,
}

/// QMP enum `MonitorMode`; see the QEMU QMP Reference Manual.
#[qapi(added = "5.0.0")]
pub enum MonitorMode {
    #[qapi(name = "control")]
    Control,
    #[qapi(name = "readline")]
    Readline,
}

/// QMP struct `MonitorOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "5.0.0")]
pub struct MonitorOptions {
    pub chardev: String,
    pub id: Option<String>,
    pub mode: Option<MonitorMode>,
    pub pretty: Option<bool>,
}

/// QMP struct `MonitorProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "11.1.0")]
pub struct MonitorProperties {
    pub chardev: String,
}

/// QMP enum `MonitorQMPCloseAction`; see the QEMU QMP Reference Manual.
#[qapi(name = "MonitorQMPCloseAction")]
#[qapi(added = "11.1.0")]
pub enum MonitorQmpCloseAction {
    #[qapi(name = "delete")]
    Delete,
    #[qapi(name = "none")]
    None,
}

/// QMP struct `MonitorQMPProperties`; see the QEMU QMP Reference Manual.
#[qapi(name = "MonitorQMPProperties")]
#[qapi(added = "11.1.0")]
pub struct MonitorQmpProperties {
    pub chardev: String,
    #[qapi(name = "close-action")]
    pub close_action: Option<MonitorQmpCloseAction>,
    pub pretty: Option<bool>,
}

/// QMP struct `MouseInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.0.0")]
pub struct MouseInfo {
    pub absolute: bool,
    pub current: bool,
    pub index: i64,
    pub name: String,
}

/// QMP enum `MultiFDCompression`; see the QEMU QMP Reference Manual.
#[qapi(name = "MultiFDCompression")]
#[qapi(added = "5.0.0")]
pub enum MultiFdCompression {
    #[qapi(name = "none")]
    None,
    #[qapi(name = "qatzip")]
    #[qapi(condition = "CONFIG_QATZIP")]
    #[qapi(added = "9.2.0")]
    Qatzip,
    #[qapi(name = "qpl")]
    #[qapi(condition = "CONFIG_QPL")]
    #[qapi(added = "9.1.0")]
    Qpl,
    #[qapi(name = "uadk")]
    #[qapi(condition = "CONFIG_UADK")]
    #[qapi(added = "9.1.0")]
    Uadk,
    #[qapi(name = "zlib")]
    Zlib,
    #[qapi(name = "zstd")]
    #[qapi(condition = "CONFIG_ZSTD")]
    Zstd,
}

/// QMP struct `NameInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.0.0")]
pub struct NameInfo {
    pub name: Option<String>,
}

/// QMP struct `NbdServerAddOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "5.2.0")]
pub struct NbdServerAddOptions {
    pub bitmap: Option<String>,
    pub description: Option<String>,
    pub device: String,
    pub name: Option<String>,
    pub writable: Option<bool>,
}

/// QMP struct `NbdServerOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "5.0.0")]
pub struct NbdServerOptions {
    pub addr: SocketAddress,
    #[qapi(name = "handshake-max-seconds")]
    #[qapi(added = "10.0.0")]
    pub handshake_max_seconds: Option<u32>,
    #[qapi(name = "max-connections")]
    #[qapi(added = "5.2.0")]
    pub max_connections: Option<u32>,
    #[qapi(name = "tls-authz")]
    pub tls_authz: Option<String>,
    #[qapi(name = "tls-creds")]
    pub tls_creds: Option<String>,
}

/// QMP struct `NbdServerOptionsBase`; see the QEMU QMP Reference Manual.
#[qapi(added = "10.0.0")]
pub struct NbdServerOptionsBase {
    #[qapi(name = "handshake-max-seconds")]
    pub handshake_max_seconds: Option<u32>,
    #[qapi(name = "max-connections")]
    pub max_connections: Option<u32>,
    #[qapi(name = "tls-authz")]
    pub tls_authz: Option<String>,
    #[qapi(name = "tls-creds")]
    pub tls_creds: Option<String>,
}

/// QMP struct `NbdServerOptionsLegacy`; see the QEMU QMP Reference Manual.
#[qapi(added = "10.0.0")]
pub struct NbdServerOptionsLegacy {
    pub addr: SocketAddressLegacy,
    #[qapi(name = "handshake-max-seconds")]
    pub handshake_max_seconds: Option<u32>,
    #[qapi(name = "max-connections")]
    pub max_connections: Option<u32>,
    #[qapi(name = "tls-authz")]
    pub tls_authz: Option<String>,
    #[qapi(name = "tls-creds")]
    pub tls_creds: Option<String>,
}

/// QMP enum `NbdServerRemoveMode`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
#[qapi(removed = "5.2.0")]
pub enum NbdServerRemoveMode {
    #[qapi(name = "hard")]
    Hard,
    #[qapi(name = "safe")]
    Safe,
}

/// QMP enum `NetClientDriver`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.7.0")]
pub enum NetClientDriver {
    #[qapi(name = "af-xdp")]
    #[qapi(condition = "CONFIG_AF_XDP")]
    #[qapi(added = "8.2.0")]
    AfXdp,
    #[qapi(name = "bridge")]
    Bridge,
    #[qapi(name = "dgram")]
    #[qapi(added = "7.2.0")]
    Dgram,
    #[qapi(name = "dump")]
    #[qapi(removed = "2.12.0")]
    Dump,
    #[qapi(name = "hubport")]
    Hubport,
    #[qapi(name = "l2tpv3")]
    L2tpv3,
    #[qapi(name = "netmap")]
    Netmap,
    #[qapi(name = "nic")]
    Nic,
    #[qapi(name = "none")]
    None,
    #[qapi(name = "passt")]
    #[qapi(condition = "CONFIG_PASST")]
    #[qapi(added = "10.1.0")]
    Passt,
    #[qapi(name = "socket")]
    Socket,
    #[qapi(name = "stream")]
    #[qapi(added = "7.2.0")]
    Stream,
    #[qapi(name = "tap")]
    Tap,
    #[qapi(name = "user")]
    User,
    #[qapi(name = "vde")]
    Vde,
    #[qapi(name = "vhost-user")]
    VhostUser,
    #[qapi(name = "vhost-vdpa")]
    #[qapi(added = "5.1.0")]
    VhostVdpa,
    #[qapi(name = "vmnet-bridged")]
    #[qapi(condition = "CONFIG_VMNET")]
    #[qapi(added = "7.1.0")]
    VmnetBridged,
    #[qapi(name = "vmnet-host")]
    #[qapi(condition = "CONFIG_VMNET")]
    #[qapi(added = "7.1.0")]
    VmnetHost,
    #[qapi(name = "vmnet-shared")]
    #[qapi(condition = "CONFIG_VMNET")]
    #[qapi(added = "7.1.0")]
    VmnetShared,
}

/// QMP union `NetClientOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.2.0")]
#[qapi(removed = "2.7.0")]
#[qapi(simple_union)]
pub struct NetClientOptions {
    #[qapi(discriminator)]
    #[qapi(name = "type")]
    pub r_type: NetClientOptionsKind,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<NetClientOptionsBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "1.2.0")]
#[qapi(removed = "2.7.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum NetClientOptionsBranch {
    #[qapi(name = "bridge")]
    Bridge(NetdevBridgeOptions),
    #[qapi(name = "dump")]
    Dump(NetdevDumpOptions),
    #[qapi(name = "hubport")]
    Hubport(NetdevHubPortOptions),
    #[qapi(name = "l2tpv3")]
    #[qapi(added = "2.1.0")]
    L2tpv3(NetdevL2tPv3Options),
    #[qapi(name = "netmap")]
    #[qapi(added = "2.0.0")]
    Netmap(NetdevNetmapOptions),
    #[qapi(name = "nic")]
    Nic(NetLegacyNicOptions),
    #[qapi(name = "none")]
    None(NetdevNoneOptions),
    #[qapi(name = "socket")]
    Socket(NetdevSocketOptions),
    #[qapi(name = "tap")]
    Tap(NetdevTapOptions),
    #[qapi(name = "user")]
    User(NetdevUserOptions),
    #[qapi(name = "vde")]
    Vde(NetdevVdeOptions),
    #[qapi(name = "vhost-user")]
    #[qapi(added = "2.1.0")]
    VhostUser(NetdevVhostUserOptions),
}

#[qapi(added = "1.2.0")]
#[qapi(removed = "2.7.0")]
pub enum NetClientOptionsKind {
    #[qapi(name = "bridge")]
    Bridge,
    #[qapi(name = "dump")]
    Dump,
    #[qapi(name = "hubport")]
    Hubport,
    #[qapi(name = "l2tpv3")]
    #[qapi(added = "2.1.0")]
    L2tpv3,
    #[qapi(name = "netmap")]
    #[qapi(added = "2.0.0")]
    Netmap,
    #[qapi(name = "nic")]
    Nic,
    #[qapi(name = "none")]
    None,
    #[qapi(name = "socket")]
    Socket,
    #[qapi(name = "tap")]
    Tap,
    #[qapi(name = "user")]
    User,
    #[qapi(name = "vde")]
    Vde,
    #[qapi(name = "vhost-user")]
    #[qapi(added = "2.1.0")]
    VhostUser,
}

/// QMP union `Netdev`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.2.0")]
pub struct Netdev {
    pub id: String,
    #[qapi(removed = "2.7.0")]
    #[qapi(required)]
    pub opts: Option<NetClientOptions>,
    #[qapi(discriminator)]
    #[qapi(added = "2.7.0")]
    #[qapi(required)]
    pub r#type: Option<NetClientDriver>,
    #[qapi(union)]
    #[qapi(flatten)]
    #[qapi(added = "2.7.0")]
    pub u: Option<NetdevBranch>,
}

/// QMP struct `NetdevAFXDPOptions`; see the QEMU QMP Reference Manual.
#[qapi(name = "NetdevAFXDPOptions")]
#[qapi(condition = "CONFIG_AF_XDP")]
#[qapi(added = "8.2.0")]
pub struct NetdevAfxdpOptions {
    #[qapi(name = "force-copy")]
    pub force_copy: Option<bool>,
    pub ifname: String,
    pub inhibit: Option<bool>,
    #[qapi(name = "map-path")]
    #[qapi(added = "10.1.0")]
    pub map_path: Option<String>,
    #[qapi(name = "map-start-index")]
    #[qapi(added = "10.1.0")]
    pub map_start_index: Option<i32>,
    pub mode: Option<AfxdpMode>,
    pub queues: Option<i64>,
    #[qapi(name = "sock-fds")]
    pub sock_fds: Option<String>,
    #[qapi(name = "start-queue")]
    pub start_queue: Option<i64>,
}

#[qapi(union_branch)]
#[qapi(added = "2.7.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum NetdevBranch {
    #[qapi(name = "af-xdp")]
    #[qapi(condition = "CONFIG_AF_XDP")]
    #[qapi(added = "8.2.0")]
    AfXdp(NetdevAfxdpOptions),
    #[qapi(name = "bridge")]
    Bridge(NetdevBridgeOptions),
    #[qapi(name = "dgram")]
    #[qapi(added = "7.2.0")]
    Dgram(NetdevDgramOptions),
    #[qapi(name = "dump")]
    #[qapi(removed = "2.12.0")]
    Dump(NetdevDumpOptions),
    #[qapi(name = "hubport")]
    Hubport(NetdevHubPortOptions),
    #[qapi(name = "l2tpv3")]
    L2tpv3(NetdevL2tPv3Options),
    #[qapi(name = "netmap")]
    Netmap(NetdevNetmapOptions),
    #[qapi(name = "nic")]
    Nic(NetLegacyNicOptions),
    #[qapi(name = "none")]
    #[qapi(added = "3.0.0")]
    None,
    #[qapi(name = "none")]
    #[qapi(removed = "3.0.0")]
    NoneLegacy(NetdevNoneOptions),
    #[qapi(name = "passt")]
    #[qapi(condition = "CONFIG_PASST")]
    #[qapi(added = "11.1.0")]
    Passt(NetdevPasstOptions),
    #[qapi(condition = "CONFIG_PASST")]
    #[qapi(name = "passt")]
    #[qapi(added = "10.1.0")]
    #[qapi(removed = "11.1.0")]
    PasstLegacy(NetDevPasstOptions),
    #[qapi(name = "socket")]
    Socket(NetdevSocketOptions),
    #[qapi(name = "stream")]
    #[qapi(added = "7.2.0")]
    Stream(NetdevStreamOptions),
    #[qapi(name = "tap")]
    Tap(NetdevTapOptions),
    #[qapi(name = "user")]
    User(NetdevUserOptions),
    #[qapi(name = "vde")]
    Vde(NetdevVdeOptions),
    #[qapi(name = "vhost-user")]
    VhostUser(NetdevVhostUserOptions),
    #[qapi(name = "vhost-vdpa")]
    #[qapi(added = "5.1.0")]
    VhostVdpa(NetdevVhostVdpaOptions),
    #[qapi(name = "vmnet-bridged")]
    #[qapi(condition = "CONFIG_VMNET")]
    #[qapi(added = "7.1.0")]
    VmnetBridged(NetdevVmnetBridgedOptions),
    #[qapi(name = "vmnet-host")]
    #[qapi(condition = "CONFIG_VMNET")]
    #[qapi(added = "7.1.0")]
    VmnetHost(NetdevVmnetHostOptions),
    #[qapi(name = "vmnet-shared")]
    #[qapi(condition = "CONFIG_VMNET")]
    #[qapi(added = "7.1.0")]
    VmnetShared(NetdevVmnetSharedOptions),
}

/// QMP struct `NetdevBridgeOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.2.0")]
pub struct NetdevBridgeOptions {
    pub br: Option<String>,
    pub helper: Option<String>,
}

/// QMP struct `NetdevDgramOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.2.0")]
pub struct NetdevDgramOptions {
    pub local: Option<SocketAddress>,
    pub remote: Option<SocketAddress>,
}

/// QMP struct `NetdevDumpOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.2.0")]
#[qapi(removed = "2.12.0")]
pub struct NetdevDumpOptions {
    pub file: Option<String>,
    pub len: Option<u64>,
}

/// QMP struct `NetdevHubPortOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.2.0")]
pub struct NetdevHubPortOptions {
    pub hubid: i32,
    #[qapi(added = "2.12.0")]
    pub netdev: Option<String>,
}

/// QMP struct `NetdevL2TPv3Options`; see the QEMU QMP Reference Manual.
#[qapi(name = "NetdevL2TPv3Options")]
#[qapi(added = "2.1.0")]
pub struct NetdevL2tPv3Options {
    pub cookie64: Option<bool>,
    pub counter: Option<bool>,
    pub dst: String,
    pub dstport: Option<String>,
    pub ipv6: Option<bool>,
    pub offset: Option<u32>,
    pub pincounter: Option<bool>,
    pub rxcookie: Option<u64>,
    pub rxsession: Option<u32>,
    pub src: String,
    pub srcport: Option<String>,
    pub txcookie: Option<u64>,
    pub txsession: u32,
    pub udp: Option<bool>,
}

/// QMP struct `NetdevNetmapOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.0.0")]
pub struct NetdevNetmapOptions {
    pub devname: Option<String>,
    pub ifname: String,
}

/// QMP struct `NetdevNoneOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.2.0")]
#[qapi(removed = "3.0.0")]
pub struct NetdevNoneOptions {}

/// QMP struct `NetDevPasstOptions`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_PASST")]
#[qapi(added = "10.1.0")]
#[qapi(removed = "11.1.0")]
pub struct NetDevPasstOptions {
    pub address: Option<String>,
    pub dhcp: Option<bool>,
    #[qapi(name = "dhcp-dns")]
    pub dhcp_dns: Option<bool>,
    #[qapi(name = "dhcp-search")]
    pub dhcp_search: Option<bool>,
    pub dhcpv6: Option<bool>,
    pub dns: Option<String>,
    #[qapi(name = "dns-forward")]
    pub dns_forward: Option<String>,
    #[qapi(name = "dns-host")]
    pub dns_host: Option<String>,
    pub fqdn: Option<String>,
    pub freebind: Option<bool>,
    pub gateway: Option<String>,
    pub icmp: Option<bool>,
    pub interface: Option<String>,
    pub ipv4: Option<bool>,
    pub ipv6: Option<bool>,
    pub mac: Option<String>,
    #[qapi(name = "map-guest-addr")]
    pub map_guest_addr: Option<String>,
    #[qapi(name = "map-host-loopback")]
    pub map_host_loopback: Option<String>,
    pub mtu: Option<i64>,
    pub ndp: Option<bool>,
    pub netmask: Option<String>,
    pub outbound: Option<String>,
    #[qapi(name = "outbound-if4")]
    pub outbound_if4: Option<String>,
    #[qapi(name = "outbound-if6")]
    pub outbound_if6: Option<String>,
    pub param: Option<Vec<QapiString>>,
    pub path: Option<String>,
    pub quiet: Option<bool>,
    pub ra: Option<bool>,
    pub search: Option<Vec<QapiString>>,
    pub tcp: Option<bool>,
    #[qapi(name = "tcp-ports")]
    pub tcp_ports: Option<Vec<QapiString>>,
    pub udp: Option<bool>,
    #[qapi(name = "udp-ports")]
    pub udp_ports: Option<Vec<QapiString>>,
    #[qapi(name = "vhost-user")]
    pub vhost_user: Option<bool>,
}

/// QMP struct `NetdevPasstOptions`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_PASST")]
#[qapi(added = "11.1.0")]
pub struct NetdevPasstOptions {
    pub address: Option<String>,
    pub dhcp: Option<bool>,
    #[qapi(name = "dhcp-dns")]
    pub dhcp_dns: Option<bool>,
    #[qapi(name = "dhcp-search")]
    pub dhcp_search: Option<bool>,
    pub dhcpv6: Option<bool>,
    pub dns: Option<String>,
    #[qapi(name = "dns-forward")]
    pub dns_forward: Option<String>,
    #[qapi(name = "dns-host")]
    pub dns_host: Option<String>,
    pub fqdn: Option<String>,
    pub freebind: Option<bool>,
    pub gateway: Option<String>,
    pub icmp: Option<bool>,
    pub interface: Option<String>,
    pub ipv4: Option<bool>,
    pub ipv6: Option<bool>,
    pub mac: Option<String>,
    #[qapi(name = "map-guest-addr")]
    pub map_guest_addr: Option<String>,
    #[qapi(name = "map-host-loopback")]
    pub map_host_loopback: Option<String>,
    pub mtu: Option<i64>,
    pub ndp: Option<bool>,
    pub netmask: Option<String>,
    pub outbound: Option<String>,
    #[qapi(name = "outbound-if4")]
    pub outbound_if4: Option<String>,
    #[qapi(name = "outbound-if6")]
    pub outbound_if6: Option<String>,
    pub param: Option<Vec<PasstParameter>>,
    pub path: Option<String>,
    pub quiet: Option<bool>,
    pub ra: Option<bool>,
    pub search: Option<Vec<PasstSearch>>,
    pub tcp: Option<bool>,
    #[qapi(name = "tcp-ports")]
    pub tcp_ports: Option<Vec<PasstPortForward>>,
    pub udp: Option<bool>,
    #[qapi(name = "udp-ports")]
    pub udp_ports: Option<Vec<PasstPortForward>>,
    #[qapi(name = "vhost-user")]
    pub vhost_user: Option<bool>,
}

/// QMP struct `NetdevSocketOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.2.0")]
pub struct NetdevSocketOptions {
    pub connect: Option<String>,
    pub fd: Option<String>,
    pub listen: Option<String>,
    pub localaddr: Option<String>,
    pub mcast: Option<String>,
    pub udp: Option<String>,
}

/// QMP struct `NetdevStreamOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.2.0")]
pub struct NetdevStreamOptions {
    pub addr: SocketAddress,
    #[qapi(added = "9.2.0")]
    #[qapi(deprecated(added = "9.2.0"))]
    #[qapi(removed = "10.2.0")]
    pub reconnect: Option<i64>,
    #[qapi(name = "reconnect-ms")]
    #[qapi(added = "9.2.0")]
    pub reconnect_ms: Option<i64>,
    #[qapi(name = "reconnect")]
    #[qapi(added = "8.0.0")]
    #[qapi(removed = "9.2.0")]
    pub reconnect_legacy: Option<u32>,
    pub server: Option<bool>,
}

/// QMP struct `NetdevTapOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.2.0")]
pub struct NetdevTapOptions {
    #[qapi(added = "2.8.0")]
    pub br: Option<String>,
    pub downscript: Option<String>,
    pub fd: Option<String>,
    #[qapi(added = "1.4.0")]
    pub fds: Option<String>,
    pub helper: Option<String>,
    pub ifname: Option<String>,
    #[qapi(name = "poll-us")]
    #[qapi(added = "2.7.0")]
    pub poll_us: Option<u32>,
    #[qapi(added = "1.4.0")]
    pub queues: Option<u32>,
    pub script: Option<String>,
    pub sndbuf: Option<u64>,
    pub vhost: Option<bool>,
    pub vhostfd: Option<String>,
    #[qapi(added = "1.4.0")]
    pub vhostfds: Option<String>,
    pub vhostforce: Option<bool>,
    pub vnet_hdr: Option<bool>,
}

/// QMP struct `NetdevUserDomainSuffix`; see the QEMU QMP Reference Manual.
#[qapi(added = "11.1.0")]
pub struct NetdevUserDomainSuffix {
    pub str: String,
}

/// QMP struct `NetdevUserGuestForward`; see the QEMU QMP Reference Manual.
#[qapi(added = "11.1.0")]
pub struct NetdevUserGuestForward {
    pub str: String,
}

/// QMP struct `NetdevUserHostForward`; see the QEMU QMP Reference Manual.
#[qapi(added = "11.1.0")]
pub struct NetdevUserHostForward {
    pub str: String,
}

/// QMP struct `NetdevUserOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.2.0")]
pub struct NetdevUserOptions {
    pub bootfile: Option<String>,
    pub dhcpstart: Option<String>,
    pub dns: Option<String>,
    #[qapi(added = "11.1.0")]
    pub dnssearch: Option<Vec<NetdevUserDomainSuffix>>,
    #[qapi(name = "dnssearch")]
    #[qapi(added = "1.3.0")]
    #[qapi(removed = "11.1.0")]
    pub dnssearch_legacy: Option<Vec<QapiString>>,
    #[qapi(added = "3.0.0")]
    pub domainname: Option<String>,
    #[qapi(added = "11.1.0")]
    pub guestfwd: Option<Vec<NetdevUserGuestForward>>,
    #[qapi(name = "guestfwd")]
    #[qapi(removed = "11.1.0")]
    pub guestfwd_legacy: Option<Vec<QapiString>>,
    pub host: Option<String>,
    #[qapi(added = "11.1.0")]
    pub hostfwd: Option<Vec<NetdevUserHostForward>>,
    #[qapi(name = "hostfwd")]
    #[qapi(removed = "11.1.0")]
    pub hostfwd_legacy: Option<Vec<QapiString>>,
    pub hostname: Option<String>,
    pub ip: Option<String>,
    #[qapi(added = "2.6.0")]
    pub ipv4: Option<bool>,
    #[qapi(added = "2.6.0")]
    pub ipv6: Option<bool>,
    #[qapi(name = "ipv6-dns")]
    #[qapi(added = "2.6.0")]
    pub ipv6_dns: Option<String>,
    #[qapi(name = "ipv6-host")]
    #[qapi(added = "2.6.0")]
    pub ipv6_host: Option<String>,
    #[qapi(name = "ipv6-prefix")]
    #[qapi(added = "2.6.0")]
    pub ipv6_prefix: Option<String>,
    #[qapi(name = "ipv6-prefixlen")]
    #[qapi(added = "2.6.0")]
    pub ipv6_prefixlen: Option<i64>,
    pub net: Option<String>,
    pub restrict: Option<bool>,
    pub smb: Option<String>,
    pub smbserver: Option<String>,
    pub tftp: Option<String>,
    #[qapi(name = "tftp-server-name")]
    #[qapi(added = "3.1.0")]
    pub tftp_server_name: Option<String>,
}

/// QMP struct `NetdevVdeOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.2.0")]
pub struct NetdevVdeOptions {
    pub group: Option<String>,
    pub mode: Option<u16>,
    pub port: Option<u16>,
    pub sock: Option<String>,
}

/// QMP struct `NetdevVhostUserOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.1.0")]
pub struct NetdevVhostUserOptions {
    pub chardev: String,
    #[qapi(added = "2.5.0")]
    pub queues: Option<i64>,
    pub vhostforce: Option<bool>,
}

/// QMP struct `NetdevVhostVDPAOptions`; see the QEMU QMP Reference Manual.
#[qapi(name = "NetdevVhostVDPAOptions")]
#[qapi(added = "5.1.0")]
pub struct NetdevVhostVdpaOptions {
    pub queues: Option<i64>,
    pub vhostdev: Option<String>,
    #[qapi(added = "7.2.0")]
    pub vhostfd: Option<String>,
    #[qapi(name = "x-svq")]
    #[qapi(feature = "unstable")]
    #[qapi(added = "7.1.0")]
    pub x_svq: Option<bool>,
}

/// QMP struct `NetdevVmnetBridgedOptions`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_VMNET")]
#[qapi(added = "7.1.0")]
pub struct NetdevVmnetBridgedOptions {
    pub ifname: String,
    pub isolated: Option<bool>,
}

/// QMP struct `NetdevVmnetHostOptions`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_VMNET")]
#[qapi(added = "7.1.0")]
pub struct NetdevVmnetHostOptions {
    #[qapi(name = "end-address")]
    pub end_address: Option<String>,
    pub isolated: Option<bool>,
    #[qapi(name = "net-uuid")]
    pub net_uuid: Option<String>,
    #[qapi(name = "start-address")]
    pub start_address: Option<String>,
    #[qapi(name = "subnet-mask")]
    pub subnet_mask: Option<String>,
}

/// QMP struct `NetdevVmnetSharedOptions`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_VMNET")]
#[qapi(added = "7.1.0")]
pub struct NetdevVmnetSharedOptions {
    #[qapi(name = "end-address")]
    pub end_address: Option<String>,
    pub isolated: Option<bool>,
    #[qapi(name = "nat66-prefix")]
    pub nat66_prefix: Option<String>,
    #[qapi(name = "start-address")]
    pub start_address: Option<String>,
    #[qapi(name = "subnet-mask")]
    pub subnet_mask: Option<String>,
}

/// QMP enum `NetFilterDirection`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.5.0")]
pub enum NetFilterDirection {
    #[qapi(name = "all")]
    All,
    #[qapi(name = "rx")]
    Rx,
    #[qapi(name = "tx")]
    Tx,
}

/// QMP enum `NetfilterInsert`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub enum NetfilterInsert {
    #[qapi(name = "before")]
    Before,
    #[qapi(name = "behind")]
    Behind,
}

/// QMP struct `NetfilterProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct NetfilterProperties {
    pub insert: Option<NetfilterInsert>,
    pub netdev: String,
    pub position: Option<String>,
    pub queue: Option<NetFilterDirection>,
    pub status: Option<String>,
}

/// QMP struct `NetLegacy`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.2.0")]
#[qapi(removed = "5.1.0")]
pub struct NetLegacy {
    pub id: Option<String>,
    pub name: Option<String>,
    #[qapi(added = "2.7.0")]
    #[qapi(required)]
    pub opts: Option<NetLegacyOptions>,
    #[qapi(name = "opts")]
    #[qapi(removed = "2.7.0")]
    #[qapi(required)]
    pub opts_legacy: Option<NetClientOptions>,
    #[qapi(removed = "3.0.0")]
    pub vlan: Option<i32>,
}

/// QMP struct `NetLegacyNicOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.2.0")]
pub struct NetLegacyNicOptions {
    pub addr: Option<String>,
    pub macaddr: Option<String>,
    pub model: Option<String>,
    pub netdev: Option<String>,
    pub vectors: Option<u32>,
}

/// QMP union `NetLegacyOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.7.0")]
#[qapi(removed = "5.1.0")]
#[qapi(simple_union(removed = "2.9.0"))]
pub struct NetLegacyOptions {
    #[qapi(discriminator)]
    #[qapi(name = "type")]
    #[qapi(removed = "2.9.0")]
    #[qapi(required)]
    pub r_type: Option<NetLegacyOptionsKind>,
    #[qapi(discriminator)]
    #[qapi(added = "2.9.0")]
    #[qapi(required)]
    pub r#type: Option<NetLegacyOptionsType>,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<NetLegacyOptionsBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "2.7.0")]
#[qapi(removed = "5.1.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum NetLegacyOptionsBranch {
    #[qapi(name = "bridge")]
    Bridge(NetdevBridgeOptions),
    #[qapi(name = "dump")]
    #[qapi(removed = "2.12.0")]
    Dump(NetdevDumpOptions),
    #[qapi(name = "l2tpv3")]
    L2tpv3(NetdevL2tPv3Options),
    #[qapi(name = "netmap")]
    Netmap(NetdevNetmapOptions),
    #[qapi(name = "nic")]
    Nic(NetLegacyNicOptions),
    #[qapi(name = "none")]
    #[qapi(added = "3.0.0")]
    None,
    #[qapi(name = "none")]
    #[qapi(removed = "3.0.0")]
    NoneLegacy(NetdevNoneOptions),
    #[qapi(name = "socket")]
    Socket(NetdevSocketOptions),
    #[qapi(name = "tap")]
    Tap(NetdevTapOptions),
    #[qapi(name = "user")]
    User(NetdevUserOptions),
    #[qapi(name = "vde")]
    Vde(NetdevVdeOptions),
    #[qapi(name = "vhost-user")]
    VhostUser(NetdevVhostUserOptions),
}

#[qapi(added = "2.7.0")]
#[qapi(removed = "2.9.0")]
pub enum NetLegacyOptionsKind {
    #[qapi(name = "bridge")]
    Bridge,
    #[qapi(name = "dump")]
    Dump,
    #[qapi(name = "l2tpv3")]
    L2tpv3,
    #[qapi(name = "netmap")]
    Netmap,
    #[qapi(name = "nic")]
    Nic,
    #[qapi(name = "none")]
    None,
    #[qapi(name = "socket")]
    Socket,
    #[qapi(name = "tap")]
    Tap,
    #[qapi(name = "user")]
    User,
    #[qapi(name = "vde")]
    Vde,
    #[qapi(name = "vhost-user")]
    VhostUser,
}

/// QMP enum `NetLegacyOptionsType`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.9.0")]
#[qapi(removed = "5.1.0")]
pub enum NetLegacyOptionsType {
    #[qapi(name = "bridge")]
    Bridge,
    #[qapi(name = "dump")]
    #[qapi(removed = "2.12.0")]
    Dump,
    #[qapi(name = "l2tpv3")]
    L2tpv3,
    #[qapi(name = "netmap")]
    Netmap,
    #[qapi(name = "nic")]
    Nic,
    #[qapi(name = "none")]
    None,
    #[qapi(name = "socket")]
    Socket,
    #[qapi(name = "tap")]
    Tap,
    #[qapi(name = "user")]
    User,
    #[qapi(name = "vde")]
    Vde,
    #[qapi(name = "vhost-user")]
    VhostUser,
}

/// QMP enum `NetworkAddressFamily`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.1.0")]
pub enum NetworkAddressFamily {
    #[qapi(name = "ipv4")]
    Ipv4,
    #[qapi(name = "ipv6")]
    Ipv6,
    #[qapi(name = "unix")]
    Unix,
    #[qapi(name = "unknown")]
    Unknown,
    #[qapi(name = "vsock")]
    #[qapi(added = "2.8.0")]
    Vsock,
}

/// QMP enum `NewImageMode`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.1.0")]
pub enum NewImageMode {
    #[qapi(name = "absolute-paths")]
    AbsolutePaths,
    #[qapi(name = "existing")]
    Existing,
}

/// QMP struct `NFSServer`; see the QEMU QMP Reference Manual.
#[qapi(name = "NFSServer")]
#[qapi(added = "2.8.0")]
pub struct NfsServer {
    pub host: String,
    pub r#type: NfsTransport,
}

/// QMP enum `NFSTransport`; see the QEMU QMP Reference Manual.
#[qapi(name = "NFSTransport")]
#[qapi(added = "2.8.0")]
pub enum NfsTransport {
    #[qapi(name = "inet")]
    Inet,
}

/// QMP enum `NotifyVmexitOption`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.2.0")]
pub enum NotifyVmexitOption {
    #[qapi(name = "disable")]
    Disable,
    #[qapi(name = "internal-error")]
    InternalError,
    #[qapi(name = "run")]
    Run,
}

/// QMP struct `NumaCpuOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.10.0")]
pub struct NumaCpuOptions {
    #[qapi(name = "book-id")]
    #[qapi(added = "8.2.0")]
    pub book_id: Option<i64>,
    #[qapi(name = "cluster-id")]
    #[qapi(added = "7.1.0")]
    pub cluster_id: Option<i64>,
    #[qapi(name = "core-id")]
    pub core_id: Option<i64>,
    #[qapi(name = "die-id")]
    #[qapi(added = "4.1.0")]
    pub die_id: Option<i64>,
    #[qapi(name = "drawer-id")]
    #[qapi(added = "8.2.0")]
    pub drawer_id: Option<i64>,
    #[qapi(name = "module-id")]
    #[qapi(added = "9.1.0")]
    pub module_id: Option<i64>,
    #[qapi(name = "node-id")]
    pub node_id: Option<i64>,
    #[qapi(name = "socket-id")]
    pub socket_id: Option<i64>,
    #[qapi(name = "thread-id")]
    pub thread_id: Option<i64>,
}

/// QMP struct `NumaDistOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.10.0")]
pub struct NumaDistOptions {
    pub dst: u16,
    pub src: u16,
    pub val: u8,
}

/// QMP struct `NumaHmatCacheOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "5.0.0")]
pub struct NumaHmatCacheOptions {
    pub associativity: HmatCacheAssociativity,
    pub level: u8,
    pub line: u16,
    #[qapi(name = "node-id")]
    pub node_id: u32,
    pub policy: HmatCacheWritePolicy,
    pub size: u64,
}

/// QMP struct `NumaHmatLBOptions`; see the QEMU QMP Reference Manual.
#[qapi(name = "NumaHmatLBOptions")]
#[qapi(added = "5.0.0")]
pub struct NumaHmatLbOptions {
    pub bandwidth: Option<u64>,
    #[qapi(name = "data-type")]
    pub data_type: HmatLbDataType,
    pub hierarchy: HmatLbMemoryHierarchy,
    pub initiator: u16,
    pub latency: Option<u64>,
    pub target: u16,
}

/// QMP struct `NumaNodeOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.1.0")]
pub struct NumaNodeOptions {
    pub cpus: Option<Vec<u16>>,
    #[qapi(added = "5.0.0")]
    pub initiator: Option<u16>,
    pub mem: Option<u64>,
    pub memdev: Option<String>,
    pub nodeid: Option<u16>,
}

/// QMP union `NumaOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.1.0")]
#[qapi(simple_union(removed = "2.9.0"))]
pub struct NumaOptions {
    #[qapi(discriminator)]
    #[qapi(name = "type")]
    #[qapi(removed = "2.9.0")]
    #[qapi(required)]
    pub r_type: Option<NumaOptionsKind>,
    #[qapi(discriminator)]
    #[qapi(added = "2.9.0")]
    #[qapi(required)]
    pub r#type: Option<NumaOptionsType>,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<NumaOptionsBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "2.1.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum NumaOptionsBranch {
    #[qapi(name = "cpu")]
    #[qapi(added = "2.10.0")]
    Cpu(NumaCpuOptions),
    #[qapi(name = "dist")]
    #[qapi(added = "2.10.0")]
    Dist(NumaDistOptions),
    #[qapi(name = "hmat-cache")]
    #[qapi(added = "5.0.0")]
    HmatCache(NumaHmatCacheOptions),
    #[qapi(name = "hmat-lb")]
    #[qapi(added = "5.0.0")]
    HmatLb(NumaHmatLbOptions),
    #[qapi(name = "node")]
    Node(NumaNodeOptions),
}

#[qapi(added = "2.1.0")]
#[qapi(removed = "2.9.0")]
pub enum NumaOptionsKind {
    #[qapi(name = "node")]
    Node,
}

/// QMP enum `NumaOptionsType`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.9.0")]
pub enum NumaOptionsType {
    #[qapi(name = "cpu")]
    #[qapi(added = "2.10.0")]
    Cpu,
    #[qapi(name = "dist")]
    #[qapi(added = "2.10.0")]
    Dist,
    #[qapi(name = "hmat-cache")]
    #[qapi(added = "5.0.0")]
    HmatCache,
    #[qapi(name = "hmat-lb")]
    #[qapi(added = "5.0.0")]
    HmatLb,
    #[qapi(name = "node")]
    Node,
}

/// QMP enum `OasMode`; see the QEMU QMP Reference Manual.
#[qapi(added = "11.0.0")]
pub enum OasMode {
    #[qapi(name = "32")]
    _32,
    #[qapi(name = "36")]
    _36,
    #[qapi(name = "40")]
    _40,
    #[qapi(name = "42")]
    _42,
    #[qapi(name = "44")]
    _44,
    #[qapi(name = "48")]
    _48,
    #[qapi(name = "52")]
    _52,
    #[qapi(name = "56")]
    _56,
    #[qapi(name = "auto")]
    Auto,
}

/// QMP union `ObjectOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct ObjectOptions {
    pub id: String,
    #[qapi(name = "qom-type")]
    #[qapi(discriminator)]
    pub qom_type: ObjectType,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<ObjectOptionsBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "6.0.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum ObjectOptionsBranch {
    #[qapi(name = "acpi-generic-initiator")]
    #[qapi(added = "9.0.0")]
    AcpiGenericInitiator(AcpiGenericInitiatorProperties),
    #[qapi(name = "acpi-generic-port")]
    #[qapi(added = "9.2.0")]
    AcpiGenericPort(AcpiGenericPortProperties),
    #[qapi(name = "authz-list")]
    AuthzList(AuthZListProperties),
    #[qapi(name = "authz-listfile")]
    AuthzListfile(AuthZListFileProperties),
    #[qapi(name = "authz-pam")]
    AuthzPam(AuthZpamProperties),
    #[qapi(name = "authz-simple")]
    AuthzSimple(AuthZSimpleProperties),
    #[qapi(name = "can-bus")]
    CanBus,
    #[qapi(name = "can-host-socketcan")]
    #[qapi(condition = "CONFIG_LINUX")]
    CanHostSocketcan(CanHostSocketcanProperties),
    #[qapi(name = "colo-compare")]
    ColoCompare(ColoCompareProperties),
    #[qapi(name = "cryptodev-backend")]
    CryptodevBackend(CryptodevBackendProperties),
    #[qapi(name = "cryptodev-backend-builtin")]
    CryptodevBackendBuiltin(CryptodevBackendProperties),
    #[qapi(name = "cryptodev-backend-lkcf")]
    #[qapi(added = "7.2.0")]
    CryptodevBackendLkcf(CryptodevBackendProperties),
    #[qapi(name = "cryptodev-vhost-user")]
    #[qapi(condition = "CONFIG_VHOST_CRYPTO")]
    CryptodevVhostUser(CryptodevVhostUserProperties),
    #[qapi(name = "dbus-vmstate")]
    DbusVmstate(DBusVmStateProperties),
    #[qapi(name = "filter-buffer")]
    FilterBuffer(FilterBufferProperties),
    #[qapi(name = "filter-dump")]
    FilterDump(FilterDumpProperties),
    #[qapi(name = "filter-mirror")]
    FilterMirror(FilterMirrorProperties),
    #[qapi(name = "filter-redirector")]
    FilterRedirector(FilterRedirectorProperties),
    #[qapi(name = "filter-replay")]
    FilterReplay(NetfilterProperties),
    #[qapi(name = "filter-rewriter")]
    FilterRewriter(FilterRewriterProperties),
    #[qapi(name = "igvm-cfg")]
    #[qapi(condition = "CONFIG_IGVM")]
    #[qapi(added = "10.1.0")]
    IgvmCfg(IgvmCfgProperties),
    #[qapi(name = "input-barrier")]
    InputBarrier(InputBarrierProperties),
    #[qapi(name = "input-linux")]
    #[qapi(condition = "CONFIG_LINUX")]
    InputLinux(InputLinuxProperties),
    #[qapi(name = "iommufd")]
    #[qapi(added = "9.0.0")]
    Iommufd(IommufdProperties),
    #[qapi(name = "iothread")]
    Iothread(IothreadProperties),
    #[qapi(name = "main-loop")]
    #[qapi(added = "7.1.0")]
    MainLoop(MainLoopProperties),
    #[qapi(name = "memory-backend-epc")]
    #[qapi(condition = "CONFIG_LINUX")]
    #[qapi(added = "6.2.0")]
    MemoryBackendEpc(MemoryBackendEpcProperties),
    #[qapi(name = "memory-backend-file")]
    MemoryBackendFile(MemoryBackendFileProperties),
    #[qapi(name = "memory-backend-memfd")]
    #[qapi(condition = "CONFIG_LINUX")]
    MemoryBackendMemfd(MemoryBackendMemfdProperties),
    #[qapi(name = "memory-backend-ram")]
    MemoryBackendRam(MemoryBackendProperties),
    #[qapi(name = "memory-backend-shm")]
    #[qapi(condition = "CONFIG_POSIX")]
    #[qapi(added = "9.1.0")]
    MemoryBackendShm(MemoryBackendShmProperties),
    #[qapi(name = "monitor-hmp")]
    #[qapi(added = "11.1.0")]
    MonitorHmp(MonitorHmpProperties),
    #[qapi(name = "monitor-qmp")]
    #[qapi(added = "11.1.0")]
    MonitorQmp(MonitorQmpProperties),
    #[qapi(name = "pef-guest")]
    PefGuest,
    #[qapi(name = "pr-manager-helper")]
    #[qapi(condition = "CONFIG_LINUX")]
    PrManagerHelper(PrManagerHelperProperties),
    #[qapi(name = "qtest")]
    #[qapi(added = "6.1.0")]
    Qtest(QtestProperties),
    #[qapi(name = "rng-builtin")]
    RngBuiltin(RngProperties),
    #[qapi(name = "rng-egd")]
    RngEgd(RngEgdProperties),
    #[qapi(name = "rng-random")]
    #[qapi(condition = "CONFIG_POSIX")]
    RngRandom(RngRandomProperties),
    #[qapi(name = "s390-pv-guest")]
    S390PvGuest,
    #[qapi(name = "secret")]
    Secret(SecretProperties),
    #[qapi(name = "secret_keyring")]
    #[qapi(condition = "CONFIG_SECRET_KEYRING")]
    SecretKeyring(SecretKeyringProperties),
    #[qapi(name = "sev-guest")]
    SevGuest(SevGuestProperties),
    #[qapi(name = "sev-snp-guest")]
    #[qapi(added = "9.1.0")]
    SevSnpGuest(SevSnpGuestProperties),
    #[qapi(name = "tdx-guest")]
    #[qapi(added = "10.1.0")]
    TdxGuest(TdxGuestProperties),
    #[qapi(name = "thread-context")]
    #[qapi(added = "7.2.0")]
    ThreadContext(ThreadContextProperties),
    #[qapi(name = "throttle-group")]
    ThrottleGroup(ThrottleGroupProperties),
    #[qapi(name = "tls-cipher-suites")]
    TlsCipherSuites(TlsCredsProperties),
    #[qapi(name = "tls-creds-anon")]
    TlsCredsAnon(TlsCredsAnonProperties),
    #[qapi(name = "tls-creds-psk")]
    TlsCredsPsk(TlsCredsPskProperties),
    #[qapi(name = "tls-creds-x509")]
    TlsCredsX509(TlsCredsX509Properties),
    #[qapi(name = "x-remote-object")]
    XRemoteObject(RemoteObjectProperties),
    #[qapi(name = "x-vfio-user-server")]
    #[qapi(added = "7.1.0")]
    XVfioUserServer(VfioUserServerProperties),
}

/// QMP struct `ObjectPropertiesValues`; see the QEMU QMP Reference Manual.
#[qapi(added = "10.1.0")]
pub struct ObjectPropertiesValues {
    pub properties: Vec<ObjectPropertyValue>,
}

/// QMP struct `ObjectPropertyInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.1.0")]
pub struct ObjectPropertyInfo {
    #[qapi(name = "default-value")]
    #[qapi(added = "5.0.0")]
    pub default_value: Option<serde_json::Value>,
    #[qapi(added = "2.12.0")]
    pub description: Option<String>,
    pub name: String,
    pub r#type: String,
}

/// QMP struct `ObjectPropertyValue`; see the QEMU QMP Reference Manual.
#[qapi(added = "10.1.0")]
pub struct ObjectPropertyValue {
    pub name: String,
    pub r#type: String,
    pub value: Option<serde_json::Value>,
}

/// QMP enum `ObjectType`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub enum ObjectType {
    #[qapi(name = "acpi-generic-initiator")]
    #[qapi(added = "9.0.0")]
    AcpiGenericInitiator,
    #[qapi(name = "acpi-generic-port")]
    #[qapi(added = "9.2.0")]
    AcpiGenericPort,
    #[qapi(name = "authz-list")]
    AuthzList,
    #[qapi(name = "authz-listfile")]
    AuthzListfile,
    #[qapi(name = "authz-pam")]
    AuthzPam,
    #[qapi(name = "authz-simple")]
    AuthzSimple,
    #[qapi(name = "can-bus")]
    CanBus,
    #[qapi(name = "can-host-socketcan")]
    #[qapi(condition = "CONFIG_LINUX")]
    CanHostSocketcan,
    #[qapi(name = "colo-compare")]
    ColoCompare,
    #[qapi(name = "cryptodev-backend")]
    CryptodevBackend,
    #[qapi(name = "cryptodev-backend-builtin")]
    CryptodevBackendBuiltin,
    #[qapi(name = "cryptodev-backend-lkcf")]
    #[qapi(added = "7.2.0")]
    CryptodevBackendLkcf,
    #[qapi(name = "cryptodev-vhost-user")]
    #[qapi(condition = "CONFIG_VHOST_CRYPTO")]
    CryptodevVhostUser,
    #[qapi(name = "dbus-vmstate")]
    DbusVmstate,
    #[qapi(name = "filter-buffer")]
    FilterBuffer,
    #[qapi(name = "filter-dump")]
    FilterDump,
    #[qapi(name = "filter-mirror")]
    FilterMirror,
    #[qapi(name = "filter-redirector")]
    FilterRedirector,
    #[qapi(name = "filter-replay")]
    FilterReplay,
    #[qapi(name = "filter-rewriter")]
    FilterRewriter,
    #[qapi(name = "igvm-cfg")]
    #[qapi(condition = "CONFIG_IGVM")]
    #[qapi(added = "10.1.0")]
    IgvmCfg,
    #[qapi(name = "input-barrier")]
    InputBarrier,
    #[qapi(name = "input-linux")]
    #[qapi(condition = "CONFIG_LINUX")]
    InputLinux,
    #[qapi(name = "iommufd")]
    #[qapi(added = "9.0.0")]
    Iommufd,
    #[qapi(name = "iothread")]
    Iothread,
    #[qapi(name = "main-loop")]
    #[qapi(added = "7.1.0")]
    MainLoop,
    #[qapi(name = "memory-backend-epc")]
    #[qapi(condition = "CONFIG_LINUX")]
    #[qapi(added = "6.2.0")]
    MemoryBackendEpc,
    #[qapi(name = "memory-backend-file")]
    MemoryBackendFile,
    #[qapi(name = "memory-backend-memfd")]
    #[qapi(condition = "CONFIG_LINUX")]
    MemoryBackendMemfd,
    #[qapi(name = "memory-backend-ram")]
    MemoryBackendRam,
    #[qapi(name = "memory-backend-shm")]
    #[qapi(condition = "CONFIG_POSIX")]
    #[qapi(added = "9.1.0")]
    MemoryBackendShm,
    #[qapi(name = "monitor-hmp")]
    #[qapi(added = "11.1.0")]
    MonitorHmp,
    #[qapi(name = "monitor-qmp")]
    #[qapi(added = "11.1.0")]
    MonitorQmp,
    #[qapi(name = "pef-guest")]
    PefGuest,
    #[qapi(name = "pr-manager-helper")]
    #[qapi(condition = "CONFIG_LINUX")]
    PrManagerHelper,
    #[qapi(name = "qtest")]
    #[qapi(added = "6.1.0")]
    Qtest,
    #[qapi(name = "rng-builtin")]
    RngBuiltin,
    #[qapi(name = "rng-egd")]
    RngEgd,
    #[qapi(name = "rng-random")]
    #[qapi(condition = "CONFIG_POSIX")]
    RngRandom,
    #[qapi(name = "s390-pv-guest")]
    S390PvGuest,
    #[qapi(name = "secret")]
    Secret,
    #[qapi(name = "secret_keyring")]
    #[qapi(condition = "CONFIG_SECRET_KEYRING")]
    SecretKeyring,
    #[qapi(name = "sev-guest")]
    SevGuest,
    #[qapi(name = "sev-snp-guest")]
    #[qapi(added = "9.1.0")]
    SevSnpGuest,
    #[qapi(name = "tdx-guest")]
    #[qapi(added = "10.1.0")]
    TdxGuest,
    #[qapi(name = "thread-context")]
    #[qapi(added = "7.2.0")]
    ThreadContext,
    #[qapi(name = "throttle-group")]
    ThrottleGroup,
    #[qapi(name = "tls-cipher-suites")]
    TlsCipherSuites,
    #[qapi(name = "tls-creds-anon")]
    TlsCredsAnon,
    #[qapi(name = "tls-creds-psk")]
    TlsCredsPsk,
    #[qapi(name = "tls-creds-x509")]
    TlsCredsX509,
    #[qapi(name = "x-remote-object")]
    #[qapi(feature = "unstable")]
    XRemoteObject,
    #[qapi(name = "x-vfio-user-server")]
    #[qapi(feature = "unstable")]
    #[qapi(added = "7.1.0")]
    XVfioUserServer,
}

/// QMP struct `ObjectTypeInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.1.0")]
pub struct ObjectTypeInfo {
    #[qapi(added = "2.10.0")]
    pub r#abstract: Option<bool>,
    pub name: String,
    #[qapi(added = "2.10.0")]
    pub parent: Option<String>,
}

/// QMP enum `OffAutoPCIBAR`; see the QEMU QMP Reference Manual.
#[qapi(name = "OffAutoPCIBAR")]
#[qapi(added = "2.12.0")]
pub enum OffAutoPcibar {
    #[qapi(name = "auto")]
    Auto,
    #[qapi(name = "bar0")]
    Bar0,
    #[qapi(name = "bar1")]
    Bar1,
    #[qapi(name = "bar2")]
    Bar2,
    #[qapi(name = "bar3")]
    Bar3,
    #[qapi(name = "bar4")]
    Bar4,
    #[qapi(name = "bar5")]
    Bar5,
    #[qapi(name = "off")]
    Off,
}

/// QMP enum `OnCbwError`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.1.0")]
pub enum OnCbwError {
    #[qapi(name = "break-guest-write")]
    BreakGuestWrite,
    #[qapi(name = "break-snapshot")]
    BreakSnapshot,
}

/// QMP enum `OnOffAuto`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.2.0")]
pub enum OnOffAuto {
    #[qapi(name = "auto")]
    Auto,
    #[qapi(name = "off")]
    Off,
    #[qapi(name = "on")]
    On,
}

/// QMP enum `OnOffSplit`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.6.0")]
pub enum OnOffSplit {
    #[qapi(name = "off")]
    Off,
    #[qapi(name = "on")]
    On,
    #[qapi(name = "split")]
    Split,
}

/// QMP enum `PanicAction`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub enum PanicAction {
    #[qapi(name = "exit-failure")]
    #[qapi(added = "7.1.0")]
    ExitFailure,
    #[qapi(name = "none")]
    None,
    #[qapi(name = "pause")]
    Pause,
    #[qapi(name = "shutdown")]
    Shutdown,
}

/// QMP struct `PasstParameter`; see the QEMU QMP Reference Manual.
#[qapi(added = "11.1.0")]
pub struct PasstParameter {
    pub str: String,
}

/// QMP struct `PasstPortForward`; see the QEMU QMP Reference Manual.
#[qapi(added = "11.1.0")]
pub struct PasstPortForward {
    pub str: String,
}

/// QMP struct `PasstSearch`; see the QEMU QMP Reference Manual.
#[qapi(added = "11.1.0")]
pub struct PasstSearch {
    pub str: String,
}

/// QMP struct `PCDIMMDeviceInfo`; see the QEMU QMP Reference Manual.
#[qapi(name = "PCDIMMDeviceInfo")]
#[qapi(added = "2.1.0")]
pub struct PcdimmDeviceInfo {
    pub addr: i64,
    pub hotpluggable: bool,
    pub hotplugged: bool,
    pub id: Option<String>,
    pub memdev: String,
    pub node: i64,
    pub size: i64,
    pub slot: i64,
}

/// QMP struct `PCDIMMDeviceInfoWrapper`; see the QEMU QMP Reference Manual.
#[qapi(name = "PCDIMMDeviceInfoWrapper")]
#[qapi(added = "6.2.0")]
pub struct PcdimmDeviceInfoWrapper {
    pub data: PcdimmDeviceInfo,
}

/// QMP struct `PciBridgeInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.0.0")]
pub struct PciBridgeInfo {
    #[qapi(added = "2.4.0")]
    #[qapi(required)]
    pub bus: Option<PciBusInfo>,
    #[qapi(name = "bus")]
    #[qapi(removed = "2.4.0")]
    #[qapi(required)]
    pub bus_legacy: Option<PciBridgeInfoBus>,
    pub devices: Option<Vec<PciDeviceInfo>>,
}

#[qapi(added = "1.0.0")]
#[qapi(removed = "2.4.0")]
pub struct PciBridgeInfoBus {
    pub io_range: PciMemoryRange,
    pub memory_range: PciMemoryRange,
    pub number: i64,
    pub prefetchable_range: PciMemoryRange,
    pub secondary: i64,
    pub subordinate: i64,
}

/// QMP struct `PciBusInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.4.0")]
pub struct PciBusInfo {
    pub io_range: PciMemoryRange,
    pub memory_range: PciMemoryRange,
    pub number: i64,
    pub prefetchable_range: PciMemoryRange,
    pub secondary: i64,
    pub subordinate: i64,
}

/// QMP struct `PciDeviceClass`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.4.0")]
pub struct PciDeviceClass {
    pub class: i64,
    pub desc: Option<String>,
}

/// QMP struct `PciDeviceId`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.4.0")]
pub struct PciDeviceId {
    pub device: i64,
    #[qapi(added = "3.1.0")]
    pub subsystem: Option<i64>,
    #[qapi(name = "subsystem-vendor")]
    #[qapi(added = "3.1.0")]
    pub subsystem_vendor: Option<i64>,
    pub vendor: i64,
}

/// QMP struct `PciDeviceInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.0.0")]
pub struct PciDeviceInfo {
    pub bus: i64,
    #[qapi(added = "2.4.0")]
    #[qapi(required)]
    pub class_info: Option<PciDeviceClass>,
    #[qapi(name = "class_info")]
    #[qapi(removed = "2.4.0")]
    #[qapi(required)]
    pub class_info_legacy: Option<PciDeviceInfoClassInfo>,
    pub function: i64,
    #[qapi(added = "2.4.0")]
    #[qapi(required)]
    pub id: Option<PciDeviceId>,
    #[qapi(name = "id")]
    #[qapi(removed = "2.4.0")]
    #[qapi(required)]
    pub id_legacy: Option<PciDeviceInfoId>,
    pub irq: Option<i64>,
    #[qapi(added = "5.1.0")]
    #[qapi(required)]
    pub irq_pin: Option<i64>,
    pub pci_bridge: Option<PciBridgeInfo>,
    pub qdev_id: String,
    pub regions: Vec<PciMemoryRegion>,
    pub slot: i64,
}

#[qapi(added = "1.0.0")]
#[qapi(removed = "2.4.0")]
pub struct PciDeviceInfoClassInfo {
    pub class: i64,
    pub desc: Option<String>,
}

#[qapi(added = "1.0.0")]
#[qapi(removed = "2.4.0")]
pub struct PciDeviceInfoId {
    pub device: i64,
    pub vendor: i64,
}

/// QMP enum `PCIELinkSpeed`; see the QEMU QMP Reference Manual.
#[qapi(name = "PCIELinkSpeed")]
#[qapi(added = "4.0.0")]
pub enum PcieLinkSpeed {
    #[qapi(name = "16")]
    _16,
    #[qapi(name = "2_5")]
    _25,
    #[qapi(name = "32")]
    #[qapi(added = "9.0.0")]
    _32,
    #[qapi(name = "5")]
    _5,
    #[qapi(name = "64")]
    #[qapi(added = "9.0.0")]
    _64,
    #[qapi(name = "8")]
    _8,
}

/// QMP enum `PCIELinkWidth`; see the QEMU QMP Reference Manual.
#[qapi(name = "PCIELinkWidth")]
#[qapi(added = "4.0.0")]
pub enum PcieLinkWidth {
    #[qapi(name = "1")]
    _1,
    #[qapi(name = "12")]
    _12,
    #[qapi(name = "16")]
    _16,
    #[qapi(name = "2")]
    _2,
    #[qapi(name = "32")]
    _32,
    #[qapi(name = "4")]
    _4,
    #[qapi(name = "8")]
    _8,
}

/// QMP struct `PciInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.0.0")]
pub struct PciInfo {
    pub bus: i64,
    pub devices: Vec<PciDeviceInfo>,
}

/// QMP struct `PciMemoryRange`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.0.0")]
pub struct PciMemoryRange {
    pub base: i64,
    pub limit: i64,
}

/// QMP struct `PciMemoryRegion`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.0.0")]
pub struct PciMemoryRegion {
    pub address: i64,
    pub bar: i64,
    pub mem_type_64: Option<bool>,
    pub prefetch: Option<bool>,
    pub size: i64,
    pub r#type: String,
}

/// QMP enum `PreallocMode`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.2.0")]
pub enum PreallocMode {
    #[qapi(name = "falloc")]
    Falloc,
    #[qapi(name = "full")]
    Full,
    #[qapi(name = "metadata")]
    Metadata,
    #[qapi(name = "off")]
    Off,
}

/// QMP struct `PrManagerHelperProperties`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_LINUX")]
#[qapi(added = "6.0.0")]
pub struct PrManagerHelperProperties {
    pub path: String,
}

/// QMP struct `PRManagerInfo`; see the QEMU QMP Reference Manual.
#[qapi(name = "PRManagerInfo")]
#[qapi(added = "3.0.0")]
pub struct PrManagerInfo {
    pub connected: bool,
    pub id: String,
}

/// QMP enum `QapiErrorClass`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.6.0")]
pub enum QapiErrorClass {
    CommandNotFound,
    #[qapi(removed = "2.10.0")]
    DeviceEncrypted,
    DeviceNotActive,
    DeviceNotFound,
    GenericError,
    #[qapi(name = "KVMMissingCap")]
    KvmMissingCap,
}

/// QMP struct `String`; see the QEMU QMP Reference Manual.
#[qapi(name = "String")]
#[qapi(added = "1.2.0")]
#[qapi(removed = "11.1.0")]
pub struct QapiString {
    pub str: String,
}

/// QMP enum `QapiVfioMigrationState`; see the QEMU QMP Reference Manual.
#[qapi(added = "9.2.0")]
pub enum QapiVfioMigrationState {
    #[qapi(name = "pre-copy")]
    PreCopy,
    #[qapi(name = "pre-copy-p2p")]
    PreCopyP2p,
    #[qapi(name = "pre-copy-p2p-prepare")]
    #[qapi(added = "11.0.0")]
    PreCopyP2pPrepare,
    #[qapi(name = "resuming")]
    Resuming,
    #[qapi(name = "running")]
    Running,
    #[qapi(name = "running-p2p")]
    RunningP2p,
    #[qapi(name = "stop")]
    Stop,
    #[qapi(name = "stop-copy")]
    StopCopy,
}

/// QMP enum `QAuthZListFormat`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.0.0")]
pub enum QAuthZListFormat {
    #[qapi(name = "exact")]
    Exact,
    #[qapi(name = "glob")]
    Glob,
}

/// QMP enum `QAuthZListPolicy`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.0.0")]
pub enum QAuthZListPolicy {
    #[qapi(name = "allow")]
    Allow,
    #[qapi(name = "deny")]
    Deny,
}

/// QMP struct `QAuthZListRule`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.0.0")]
pub struct QAuthZListRule {
    pub format: Option<QAuthZListFormat>,
    pub r#match: String,
    pub policy: QAuthZListPolicy,
}

/// QMP struct `QAuthZListRuleListHack`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.0.0")]
#[qapi(removed = "6.0.0")]
pub struct QAuthZListRuleListHack {
    pub unused: Vec<QAuthZListRule>,
}

/// QMP struct `Qcow2BitmapInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.0.0")]
pub struct Qcow2BitmapInfo {
    pub flags: Vec<Qcow2BitmapInfoFlags>,
    pub granularity: u32,
    pub name: String,
}

/// QMP enum `Qcow2BitmapInfoFlags`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.0.0")]
pub enum Qcow2BitmapInfoFlags {
    #[qapi(name = "auto")]
    Auto,
    #[qapi(name = "in-use")]
    InUse,
}

/// QMP enum `Qcow2CompressionType`; see the QEMU QMP Reference Manual.
#[qapi(added = "5.1.0")]
pub enum Qcow2CompressionType {
    #[qapi(name = "zlib")]
    Zlib,
    #[qapi(name = "zstd")]
    #[qapi(condition = "CONFIG_ZSTD")]
    Zstd,
}

/// QMP struct `Qcow2OverlapCheckFlags`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.2.0")]
pub struct Qcow2OverlapCheckFlags {
    #[qapi(name = "active-l1")]
    pub active_l1: Option<bool>,
    #[qapi(name = "active-l2")]
    pub active_l2: Option<bool>,
    #[qapi(name = "bitmap-directory")]
    #[qapi(added = "3.0.0")]
    pub bitmap_directory: Option<bool>,
    #[qapi(name = "inactive-l1")]
    pub inactive_l1: Option<bool>,
    #[qapi(name = "inactive-l2")]
    pub inactive_l2: Option<bool>,
    #[qapi(name = "main-header")]
    pub main_header: Option<bool>,
    #[qapi(name = "refcount-block")]
    pub refcount_block: Option<bool>,
    #[qapi(name = "refcount-table")]
    pub refcount_table: Option<bool>,
    #[qapi(name = "snapshot-table")]
    pub snapshot_table: Option<bool>,
    pub template: Option<Qcow2OverlapCheckMode>,
}

/// QMP enum `Qcow2OverlapCheckMode`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.2.0")]
pub enum Qcow2OverlapCheckMode {
    #[qapi(name = "all")]
    All,
    #[qapi(name = "cached")]
    Cached,
    #[qapi(name = "constant")]
    Constant,
    #[qapi(name = "none")]
    None,
}

/// QMP alternate `Qcow2OverlapChecks`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.2.0")]
pub enum Qcow2OverlapChecks {
    #[qapi(name = "flags")]
    Flags(Qcow2OverlapCheckFlags),
    #[qapi(name = "mode")]
    Mode(Qcow2OverlapCheckMode),
}

/// QMP enum `QCryptoAkCipherAlgo`; see the QEMU QMP Reference Manual.
#[qapi(added = "9.2.0")]
pub enum QCryptoAkCipherAlgo {
    #[qapi(name = "rsa")]
    Rsa,
}

/// QMP enum `QCryptoAkCipherAlgorithm`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.1.0")]
#[qapi(removed = "9.2.0")]
pub enum QCryptoAkCipherAlgorithm {
    #[qapi(name = "rsa")]
    Rsa,
}

/// QMP enum `QCryptoAkCipherKeyType`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.1.0")]
pub enum QCryptoAkCipherKeyType {
    #[qapi(name = "private")]
    Private,
    #[qapi(name = "public")]
    Public,
}

/// QMP union `QCryptoAkCipherOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.1.0")]
pub struct QCryptoAkCipherOptions {
    #[qapi(discriminator)]
    #[qapi(added = "9.2.0")]
    #[qapi(required)]
    pub alg: Option<QCryptoAkCipherAlgo>,
    #[qapi(discriminator)]
    #[qapi(name = "alg")]
    #[qapi(removed = "9.2.0")]
    #[qapi(required)]
    pub alg_legacy: Option<QCryptoAkCipherAlgorithm>,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<QCryptoAkCipherOptionsBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "7.1.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum QCryptoAkCipherOptionsBranch {
    #[qapi(name = "rsa")]
    Rsa(QCryptoAkCipherOptionsRsa),
}

/// QMP struct `QCryptoAkCipherOptionsRSA`; see the QEMU QMP Reference Manual.
#[qapi(name = "QCryptoAkCipherOptionsRSA")]
#[qapi(added = "7.1.0")]
pub struct QCryptoAkCipherOptionsRsa {
    #[qapi(name = "hash-alg")]
    #[qapi(added = "9.2.0")]
    #[qapi(required)]
    pub hash_alg: Option<QCryptoHashAlgo>,
    #[qapi(name = "hash-alg")]
    #[qapi(removed = "9.2.0")]
    #[qapi(required)]
    pub hash_alg_legacy: Option<QCryptoHashAlgorithm>,
    #[qapi(name = "padding-alg")]
    #[qapi(added = "9.2.0")]
    #[qapi(required)]
    pub padding_alg: Option<QCryptoRsaPaddingAlgo>,
    #[qapi(name = "padding-alg")]
    #[qapi(removed = "9.2.0")]
    #[qapi(required)]
    pub padding_alg_legacy: Option<QCryptoRsaPaddingAlgorithm>,
}

/// QMP union `QCryptoBlockAmendOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "5.1.0")]
pub struct QCryptoBlockAmendOptions {
    #[qapi(discriminator)]
    pub format: QCryptoBlockFormat,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<QCryptoBlockAmendOptionsBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "5.1.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum QCryptoBlockAmendOptionsBranch {
    #[qapi(name = "luks")]
    Luks(QCryptoBlockAmendOptionsLuks),
    #[qapi(name = "qcow")]
    Qcow,
}

/// QMP struct `QCryptoBlockAmendOptionsLUKS`; see the QEMU QMP Reference Manual.
#[qapi(name = "QCryptoBlockAmendOptionsLUKS")]
#[qapi(added = "5.1.0")]
pub struct QCryptoBlockAmendOptionsLuks {
    #[qapi(name = "iter-time")]
    pub iter_time: Option<i64>,
    pub keyslot: Option<i64>,
    #[qapi(name = "new-secret")]
    pub new_secret: Option<String>,
    #[qapi(name = "old-secret")]
    pub old_secret: Option<String>,
    pub secret: Option<String>,
    pub state: QCryptoBlockLuksKeyslotState,
}

/// QMP union `QCryptoBlockCreateOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.6.0")]
pub struct QCryptoBlockCreateOptions {
    #[qapi(discriminator)]
    pub format: QCryptoBlockFormat,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<QCryptoBlockCreateOptionsBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "2.6.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum QCryptoBlockCreateOptionsBranch {
    #[qapi(name = "luks")]
    Luks(QCryptoBlockCreateOptionsLuks),
    #[qapi(name = "qcow")]
    Qcow(QCryptoBlockOptionsQCow),
}

/// QMP struct `QCryptoBlockCreateOptionsLUKS`; see the QEMU QMP Reference Manual.
#[qapi(name = "QCryptoBlockCreateOptionsLUKS")]
#[qapi(added = "2.6.0")]
pub struct QCryptoBlockCreateOptionsLuks {
    #[qapi(name = "cipher-alg")]
    #[qapi(added = "9.2.0")]
    pub cipher_alg: Option<QCryptoCipherAlgo>,
    #[qapi(name = "cipher-mode")]
    pub cipher_mode: Option<QCryptoCipherMode>,
    #[qapi(name = "cipher-alg")]
    #[qapi(removed = "9.2.0")]
    pub cipher_alg_legacy: Option<QCryptoCipherAlgorithm>,
    #[qapi(name = "detached-header")]
    #[qapi(added = "9.0.0")]
    #[qapi(removed = "9.1.0")]
    pub detached_header: Option<bool>,
    #[qapi(name = "hash-alg")]
    #[qapi(added = "9.2.0")]
    pub hash_alg: Option<QCryptoHashAlgo>,
    #[qapi(name = "hash-alg")]
    #[qapi(removed = "9.2.0")]
    pub hash_alg_legacy: Option<QCryptoHashAlgorithm>,
    #[qapi(name = "iter-time")]
    #[qapi(added = "2.8.0")]
    pub iter_time: Option<i64>,
    #[qapi(name = "ivgen-alg")]
    #[qapi(added = "9.2.0")]
    pub ivgen_alg: Option<QCryptoIvGenAlgo>,
    #[qapi(name = "ivgen-hash-alg")]
    #[qapi(added = "9.2.0")]
    pub ivgen_hash_alg: Option<QCryptoHashAlgo>,
    #[qapi(name = "ivgen-alg")]
    #[qapi(removed = "9.2.0")]
    pub ivgen_alg_legacy: Option<QCryptoIvGenAlgorithm>,
    #[qapi(name = "ivgen-hash-alg")]
    #[qapi(removed = "9.2.0")]
    pub ivgen_hash_alg_legacy: Option<QCryptoHashAlgorithm>,
    #[qapi(name = "key-secret")]
    pub key_secret: Option<String>,
}

/// QMP enum `QCryptoBlockFormat`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.6.0")]
pub enum QCryptoBlockFormat {
    #[qapi(name = "luks")]
    Luks,
    #[qapi(name = "qcow")]
    Qcow,
}

/// QMP union `QCryptoBlockInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.7.0")]
pub struct QCryptoBlockInfo {
    #[qapi(discriminator)]
    pub format: QCryptoBlockFormat,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<QCryptoBlockInfoBranch>,
}

/// QMP struct `QCryptoBlockInfoBase`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.7.0")]
pub struct QCryptoBlockInfoBase {
    pub format: QCryptoBlockFormat,
}

#[qapi(union_branch)]
#[qapi(added = "2.7.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum QCryptoBlockInfoBranch {
    #[qapi(name = "luks")]
    Luks(QCryptoBlockInfoLuks),
    #[qapi(name = "qcow")]
    #[qapi(added = "3.0.0")]
    Qcow,
    #[qapi(name = "qcow")]
    #[qapi(removed = "3.0.0")]
    QcowLegacy(QCryptoBlockInfoQCow),
}

/// QMP struct `QCryptoBlockInfoLUKS`; see the QEMU QMP Reference Manual.
#[qapi(name = "QCryptoBlockInfoLUKS")]
#[qapi(added = "2.7.0")]
pub struct QCryptoBlockInfoLuks {
    #[qapi(name = "cipher-alg")]
    #[qapi(added = "9.2.0")]
    #[qapi(required)]
    pub cipher_alg: Option<QCryptoCipherAlgo>,
    #[qapi(name = "cipher-mode")]
    pub cipher_mode: QCryptoCipherMode,
    #[qapi(name = "cipher-alg")]
    #[qapi(removed = "9.2.0")]
    #[qapi(required)]
    pub cipher_alg_legacy: Option<QCryptoCipherAlgorithm>,
    #[qapi(name = "detached-header")]
    #[qapi(added = "9.0.0")]
    #[qapi(required)]
    pub detached_header: Option<bool>,
    #[qapi(name = "hash-alg")]
    #[qapi(added = "9.2.0")]
    #[qapi(required)]
    pub hash_alg: Option<QCryptoHashAlgo>,
    #[qapi(name = "hash-alg")]
    #[qapi(removed = "9.2.0")]
    #[qapi(required)]
    pub hash_alg_legacy: Option<QCryptoHashAlgorithm>,
    #[qapi(name = "ivgen-alg")]
    #[qapi(added = "9.2.0")]
    #[qapi(required)]
    pub ivgen_alg: Option<QCryptoIvGenAlgo>,
    #[qapi(name = "ivgen-hash-alg")]
    #[qapi(added = "9.2.0")]
    pub ivgen_hash_alg: Option<QCryptoHashAlgo>,
    #[qapi(name = "ivgen-alg")]
    #[qapi(removed = "9.2.0")]
    #[qapi(required)]
    pub ivgen_alg_legacy: Option<QCryptoIvGenAlgorithm>,
    #[qapi(name = "ivgen-hash-alg")]
    #[qapi(removed = "9.2.0")]
    pub ivgen_hash_alg_legacy: Option<QCryptoHashAlgorithm>,
    #[qapi(name = "master-key-iters")]
    pub master_key_iters: i64,
    #[qapi(name = "payload-offset")]
    pub payload_offset: i64,
    pub slots: Vec<QCryptoBlockInfoLuksSlot>,
    pub uuid: String,
}

/// QMP struct `QCryptoBlockInfoLUKSSlot`; see the QEMU QMP Reference Manual.
#[qapi(name = "QCryptoBlockInfoLUKSSlot")]
#[qapi(added = "2.7.0")]
pub struct QCryptoBlockInfoLuksSlot {
    pub active: bool,
    pub iters: Option<i64>,
    #[qapi(name = "key-offset")]
    pub key_offset: i64,
    pub stripes: Option<i64>,
}

/// QMP struct `QCryptoBlockInfoQCow`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.7.0")]
#[qapi(removed = "3.0.0")]
pub struct QCryptoBlockInfoQCow {}

/// QMP enum `QCryptoBlockLUKSKeyslotState`; see the QEMU QMP Reference Manual.
#[qapi(name = "QCryptoBlockLUKSKeyslotState")]
#[qapi(added = "5.1.0")]
pub enum QCryptoBlockLuksKeyslotState {
    #[qapi(name = "active")]
    Active,
    #[qapi(name = "inactive")]
    Inactive,
}

/// QMP union `QCryptoBlockOpenOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.6.0")]
pub struct QCryptoBlockOpenOptions {
    #[qapi(discriminator)]
    pub format: QCryptoBlockFormat,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<QCryptoBlockOpenOptionsBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "2.6.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum QCryptoBlockOpenOptionsBranch {
    #[qapi(name = "luks")]
    Luks(QCryptoBlockOptionsLuks),
    #[qapi(name = "qcow")]
    Qcow(QCryptoBlockOptionsQCow),
}

/// QMP struct `QCryptoBlockOptionsBase`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.6.0")]
pub struct QCryptoBlockOptionsBase {
    pub format: QCryptoBlockFormat,
}

/// QMP struct `QCryptoBlockOptionsLUKS`; see the QEMU QMP Reference Manual.
#[qapi(name = "QCryptoBlockOptionsLUKS")]
#[qapi(added = "2.6.0")]
pub struct QCryptoBlockOptionsLuks {
    #[qapi(name = "key-secret")]
    pub key_secret: Option<String>,
}

/// QMP struct `QCryptoBlockOptionsQCow`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.6.0")]
pub struct QCryptoBlockOptionsQCow {
    #[qapi(name = "key-secret")]
    pub key_secret: Option<String>,
}

/// QMP enum `QCryptoCipherAlgo`; see the QEMU QMP Reference Manual.
#[qapi(added = "9.2.0")]
pub enum QCryptoCipherAlgo {
    #[qapi(name = "3des")]
    _3des,
    #[qapi(name = "aes-128")]
    Aes128,
    #[qapi(name = "aes-192")]
    Aes192,
    #[qapi(name = "aes-256")]
    Aes256,
    #[qapi(name = "cast5-128")]
    Cast5128,
    #[qapi(name = "des")]
    Des,
    #[qapi(name = "serpent-128")]
    Serpent128,
    #[qapi(name = "serpent-192")]
    Serpent192,
    #[qapi(name = "serpent-256")]
    Serpent256,
    #[qapi(name = "sm4")]
    Sm4,
    #[qapi(name = "twofish-128")]
    Twofish128,
    #[qapi(name = "twofish-192")]
    Twofish192,
    #[qapi(name = "twofish-256")]
    Twofish256,
}

/// QMP enum `QCryptoCipherAlgorithm`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.6.0")]
#[qapi(removed = "9.2.0")]
pub enum QCryptoCipherAlgorithm {
    #[qapi(name = "3des")]
    #[qapi(added = "2.9.0")]
    _3des,
    #[qapi(name = "aes-128")]
    Aes128,
    #[qapi(name = "aes-192")]
    Aes192,
    #[qapi(name = "aes-256")]
    Aes256,
    #[qapi(name = "cast5-128")]
    Cast5128,
    #[qapi(name = "des")]
    #[qapi(added = "6.1.0")]
    Des,
    #[qapi(name = "des-rfb")]
    #[qapi(removed = "6.1.0")]
    DesRfb,
    #[qapi(name = "serpent-128")]
    Serpent128,
    #[qapi(name = "serpent-192")]
    Serpent192,
    #[qapi(name = "serpent-256")]
    Serpent256,
    #[qapi(name = "sm4")]
    #[qapi(added = "9.0.0")]
    Sm4,
    #[qapi(name = "twofish-128")]
    Twofish128,
    #[qapi(name = "twofish-192")]
    Twofish192,
    #[qapi(name = "twofish-256")]
    Twofish256,
}

/// QMP enum `QCryptoCipherMode`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.6.0")]
pub enum QCryptoCipherMode {
    #[qapi(name = "cbc")]
    Cbc,
    #[qapi(name = "ctr")]
    #[qapi(added = "2.8.0")]
    Ctr,
    #[qapi(name = "ecb")]
    Ecb,
    #[qapi(name = "xts")]
    Xts,
}

/// QMP enum `QCryptodevBackendAlgoType`; see the QEMU QMP Reference Manual.
#[qapi(added = "9.2.0")]
pub enum QCryptodevBackendAlgoType {
    #[qapi(name = "asym")]
    Asym,
    #[qapi(name = "sym")]
    Sym,
}

/// QMP enum `QCryptodevBackendAlgType`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.0.0")]
#[qapi(removed = "9.2.0")]
pub enum QCryptodevBackendAlgType {
    #[qapi(name = "asym")]
    Asym,
    #[qapi(name = "sym")]
    Sym,
}

/// QMP struct `QCryptodevBackendClient`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.0.0")]
pub struct QCryptodevBackendClient {
    pub queue: u32,
    pub r#type: QCryptodevBackendType,
}

/// QMP enum `QCryptodevBackendServiceType`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.0.0")]
pub enum QCryptodevBackendServiceType {
    #[qapi(name = "aead")]
    Aead,
    #[qapi(name = "akcipher")]
    Akcipher,
    #[qapi(name = "cipher")]
    Cipher,
    #[qapi(name = "hash")]
    Hash,
    #[qapi(name = "mac")]
    Mac,
}

/// QMP enum `QCryptodevBackendType`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.0.0")]
pub enum QCryptodevBackendType {
    #[qapi(name = "builtin")]
    Builtin,
    #[qapi(name = "lkcf")]
    Lkcf,
    #[qapi(name = "vhost-user")]
    VhostUser,
}

/// QMP struct `QCryptodevInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.0.0")]
pub struct QCryptodevInfo {
    pub client: Vec<QCryptodevBackendClient>,
    pub id: String,
    pub service: Vec<QCryptodevBackendServiceType>,
}

/// QMP enum `QCryptoHashAlgo`; see the QEMU QMP Reference Manual.
#[qapi(added = "9.2.0")]
pub enum QCryptoHashAlgo {
    #[qapi(name = "md5")]
    Md5,
    #[qapi(name = "ripemd160")]
    Ripemd160,
    #[qapi(name = "sha1")]
    Sha1,
    #[qapi(name = "sha224")]
    Sha224,
    #[qapi(name = "sha256")]
    Sha256,
    #[qapi(name = "sha384")]
    Sha384,
    #[qapi(name = "sha512")]
    Sha512,
    #[qapi(name = "sm3")]
    Sm3,
}

/// QMP enum `QCryptoHashAlgorithm`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.6.0")]
#[qapi(removed = "9.2.0")]
pub enum QCryptoHashAlgorithm {
    #[qapi(name = "md5")]
    Md5,
    #[qapi(name = "ripemd160")]
    #[qapi(added = "2.7.0")]
    Ripemd160,
    #[qapi(name = "sha1")]
    Sha1,
    #[qapi(name = "sha224")]
    #[qapi(added = "2.7.0")]
    Sha224,
    #[qapi(name = "sha256")]
    Sha256,
    #[qapi(name = "sha384")]
    #[qapi(added = "2.7.0")]
    Sha384,
    #[qapi(name = "sha512")]
    #[qapi(added = "2.7.0")]
    Sha512,
}

/// QMP enum `QCryptoIVGenAlgo`; see the QEMU QMP Reference Manual.
#[qapi(name = "QCryptoIVGenAlgo")]
#[qapi(added = "9.2.0")]
pub enum QCryptoIvGenAlgo {
    #[qapi(name = "essiv")]
    Essiv,
    #[qapi(name = "plain")]
    Plain,
    #[qapi(name = "plain64")]
    Plain64,
}

/// QMP enum `QCryptoIVGenAlgorithm`; see the QEMU QMP Reference Manual.
#[qapi(name = "QCryptoIVGenAlgorithm")]
#[qapi(added = "2.6.0")]
#[qapi(removed = "9.2.0")]
pub enum QCryptoIvGenAlgorithm {
    #[qapi(name = "essiv")]
    Essiv,
    #[qapi(name = "plain")]
    Plain,
    #[qapi(name = "plain64")]
    Plain64,
}

/// QMP enum `QCryptoRSAPaddingAlgo`; see the QEMU QMP Reference Manual.
#[qapi(name = "QCryptoRSAPaddingAlgo")]
#[qapi(added = "9.2.0")]
pub enum QCryptoRsaPaddingAlgo {
    #[qapi(name = "pkcs1")]
    Pkcs1,
    #[qapi(name = "raw")]
    Raw,
}

/// QMP enum `QCryptoRSAPaddingAlgorithm`; see the QEMU QMP Reference Manual.
#[qapi(name = "QCryptoRSAPaddingAlgorithm")]
#[qapi(added = "7.1.0")]
#[qapi(removed = "9.2.0")]
pub enum QCryptoRsaPaddingAlgorithm {
    #[qapi(name = "pkcs1")]
    Pkcs1,
    #[qapi(name = "raw")]
    Raw,
}

/// QMP enum `QCryptoSecretFormat`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.6.0")]
pub enum QCryptoSecretFormat {
    #[qapi(name = "base64")]
    Base64,
    #[qapi(name = "raw")]
    Raw,
}

/// QMP enum `QCryptoTLSCredsEndpoint`; see the QEMU QMP Reference Manual.
#[qapi(name = "QCryptoTLSCredsEndpoint")]
#[qapi(added = "2.5.0")]
pub enum QCryptoTlsCredsEndpoint {
    #[qapi(name = "client")]
    Client,
    #[qapi(name = "server")]
    Server,
}

/// QMP struct `QemuTargetInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "3.0.0")]
pub struct QemuTargetInfo {
    pub arch: SysEmuTarget,
}

/// QMP enum `QKeyCode`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.3.0")]
pub enum QKeyCode {
    #[qapi(name = "0")]
    _0,
    #[qapi(name = "1")]
    _1,
    #[qapi(name = "2")]
    _2,
    #[qapi(name = "3")]
    _3,
    #[qapi(name = "4")]
    _4,
    #[qapi(name = "5")]
    _5,
    #[qapi(name = "6")]
    _6,
    #[qapi(name = "7")]
    _7,
    #[qapi(name = "8")]
    _8,
    #[qapi(name = "9")]
    _9,
    #[qapi(name = "a")]
    A,
    #[qapi(name = "ac_back")]
    #[qapi(added = "2.10.0")]
    AcBack,
    #[qapi(name = "ac_bookmarks")]
    #[qapi(added = "2.10.0")]
    AcBookmarks,
    #[qapi(name = "ac_forward")]
    #[qapi(added = "2.10.0")]
    AcForward,
    #[qapi(name = "ac_home")]
    #[qapi(added = "2.10.0")]
    AcHome,
    #[qapi(name = "ac_refresh")]
    #[qapi(added = "2.10.0")]
    AcRefresh,
    #[qapi(name = "again")]
    Again,
    #[qapi(name = "alt")]
    Alt,
    #[qapi(name = "alt_r")]
    AltR,
    #[qapi(name = "altgr")]
    #[qapi(removed = "2.10.0")]
    Altgr,
    #[qapi(name = "altgr_r")]
    #[qapi(removed = "2.10.0")]
    AltgrR,
    #[qapi(name = "apostrophe")]
    Apostrophe,
    #[qapi(name = "asterisk")]
    Asterisk,
    #[qapi(name = "audiomute")]
    #[qapi(added = "2.10.0")]
    Audiomute,
    #[qapi(name = "audionext")]
    #[qapi(added = "2.10.0")]
    Audionext,
    #[qapi(name = "audioplay")]
    #[qapi(added = "2.10.0")]
    Audioplay,
    #[qapi(name = "audioprev")]
    #[qapi(added = "2.10.0")]
    Audioprev,
    #[qapi(name = "audiostop")]
    #[qapi(added = "2.10.0")]
    Audiostop,
    #[qapi(name = "b")]
    B,
    #[qapi(name = "backslash")]
    Backslash,
    #[qapi(name = "backspace")]
    Backspace,
    #[qapi(name = "bracket_left")]
    BracketLeft,
    #[qapi(name = "bracket_right")]
    BracketRight,
    #[qapi(name = "c")]
    C,
    #[qapi(name = "calculator")]
    #[qapi(added = "2.10.0")]
    Calculator,
    #[qapi(name = "caps_lock")]
    CapsLock,
    #[qapi(name = "comma")]
    Comma,
    #[qapi(name = "compose")]
    Compose,
    #[qapi(name = "computer")]
    #[qapi(added = "2.10.0")]
    Computer,
    #[qapi(name = "copy")]
    Copy,
    #[qapi(name = "ctrl")]
    Ctrl,
    #[qapi(name = "ctrl_r")]
    CtrlR,
    #[qapi(name = "cut")]
    Cut,
    #[qapi(name = "d")]
    D,
    #[qapi(name = "delete")]
    Delete,
    #[qapi(name = "dot")]
    Dot,
    #[qapi(name = "down")]
    Down,
    #[qapi(name = "e")]
    E,
    #[qapi(name = "end")]
    End,
    #[qapi(name = "equal")]
    Equal,
    #[qapi(name = "esc")]
    Esc,
    #[qapi(name = "f")]
    F,
    #[qapi(name = "f1")]
    F1,
    #[qapi(name = "f10")]
    F10,
    #[qapi(name = "f11")]
    F11,
    #[qapi(name = "f12")]
    F12,
    #[qapi(name = "f13")]
    #[qapi(added = "8.0.0")]
    F13,
    #[qapi(name = "f14")]
    #[qapi(added = "8.0.0")]
    F14,
    #[qapi(name = "f15")]
    #[qapi(added = "8.0.0")]
    F15,
    #[qapi(name = "f16")]
    #[qapi(added = "8.0.0")]
    F16,
    #[qapi(name = "f17")]
    #[qapi(added = "8.0.0")]
    F17,
    #[qapi(name = "f18")]
    #[qapi(added = "8.0.0")]
    F18,
    #[qapi(name = "f19")]
    #[qapi(added = "8.0.0")]
    F19,
    #[qapi(name = "f2")]
    F2,
    #[qapi(name = "f20")]
    #[qapi(added = "8.0.0")]
    F20,
    #[qapi(name = "f21")]
    #[qapi(added = "8.0.0")]
    F21,
    #[qapi(name = "f22")]
    #[qapi(added = "8.0.0")]
    F22,
    #[qapi(name = "f23")]
    #[qapi(added = "8.0.0")]
    F23,
    #[qapi(name = "f24")]
    #[qapi(added = "8.0.0")]
    F24,
    #[qapi(name = "f3")]
    F3,
    #[qapi(name = "f4")]
    F4,
    #[qapi(name = "f5")]
    F5,
    #[qapi(name = "f6")]
    F6,
    #[qapi(name = "f7")]
    F7,
    #[qapi(name = "f8")]
    F8,
    #[qapi(name = "f9")]
    F9,
    #[qapi(name = "find")]
    Find,
    #[qapi(name = "front")]
    Front,
    #[qapi(name = "g")]
    G,
    #[qapi(name = "grave_accent")]
    GraveAccent,
    #[qapi(name = "h")]
    H,
    #[qapi(name = "help")]
    Help,
    #[qapi(name = "henkan")]
    #[qapi(added = "2.9.0")]
    Henkan,
    #[qapi(name = "hiragana")]
    #[qapi(added = "2.9.0")]
    Hiragana,
    #[qapi(name = "home")]
    Home,
    #[qapi(name = "i")]
    I,
    #[qapi(name = "insert")]
    Insert,
    #[qapi(name = "j")]
    J,
    #[qapi(name = "k")]
    K,
    #[qapi(name = "katakanahiragana")]
    #[qapi(added = "2.12.0")]
    Katakanahiragana,
    #[qapi(name = "kp_0")]
    Kp0,
    #[qapi(name = "kp_1")]
    Kp1,
    #[qapi(name = "kp_2")]
    Kp2,
    #[qapi(name = "kp_3")]
    Kp3,
    #[qapi(name = "kp_4")]
    Kp4,
    #[qapi(name = "kp_5")]
    Kp5,
    #[qapi(name = "kp_6")]
    Kp6,
    #[qapi(name = "kp_7")]
    Kp7,
    #[qapi(name = "kp_8")]
    Kp8,
    #[qapi(name = "kp_9")]
    Kp9,
    #[qapi(name = "kp_add")]
    KpAdd,
    #[qapi(name = "kp_comma")]
    #[qapi(added = "2.3.1")]
    KpComma,
    #[qapi(name = "kp_decimal")]
    KpDecimal,
    #[qapi(name = "kp_divide")]
    KpDivide,
    #[qapi(name = "kp_enter")]
    KpEnter,
    #[qapi(name = "kp_equals")]
    #[qapi(added = "2.6.0")]
    KpEquals,
    #[qapi(name = "kp_multiply")]
    KpMultiply,
    #[qapi(name = "kp_subtract")]
    KpSubtract,
    #[qapi(name = "l")]
    L,
    #[qapi(name = "lang1")]
    #[qapi(added = "6.1.0")]
    Lang1,
    #[qapi(name = "lang2")]
    #[qapi(added = "6.1.0")]
    Lang2,
    #[qapi(name = "left")]
    Left,
    #[qapi(name = "less")]
    Less,
    #[qapi(name = "lf")]
    Lf,
    #[qapi(name = "m")]
    M,
    #[qapi(name = "mail")]
    #[qapi(added = "2.10.0")]
    Mail,
    #[qapi(name = "mediaselect")]
    #[qapi(added = "2.10.0")]
    Mediaselect,
    #[qapi(name = "menu")]
    Menu,
    #[qapi(name = "meta_l")]
    MetaL,
    #[qapi(name = "meta_r")]
    MetaR,
    #[qapi(name = "minus")]
    Minus,
    #[qapi(name = "muhenkan")]
    #[qapi(added = "2.12.0")]
    Muhenkan,
    #[qapi(name = "n")]
    N,
    #[qapi(name = "num_lock")]
    NumLock,
    #[qapi(name = "o")]
    O,
    #[qapi(name = "open")]
    Open,
    #[qapi(name = "p")]
    P,
    #[qapi(name = "paste")]
    Paste,
    #[qapi(name = "pause")]
    #[qapi(added = "2.0.0")]
    Pause,
    #[qapi(name = "pgdn")]
    Pgdn,
    #[qapi(name = "pgup")]
    Pgup,
    #[qapi(name = "power")]
    #[qapi(added = "2.6.0")]
    Power,
    #[qapi(name = "print")]
    Print,
    #[qapi(name = "props")]
    Props,
    #[qapi(name = "q")]
    Q,
    #[qapi(name = "r")]
    R,
    #[qapi(name = "ret")]
    Ret,
    #[qapi(name = "right")]
    Right,
    #[qapi(name = "ro")]
    #[qapi(added = "2.3.1")]
    Ro,
    #[qapi(name = "s")]
    S,
    #[qapi(name = "scroll_lock")]
    ScrollLock,
    #[qapi(name = "semicolon")]
    Semicolon,
    #[qapi(name = "shift")]
    Shift,
    #[qapi(name = "shift_r")]
    ShiftR,
    #[qapi(name = "slash")]
    Slash,
    #[qapi(name = "sleep")]
    #[qapi(added = "2.10.0")]
    Sleep,
    #[qapi(name = "spc")]
    Spc,
    #[qapi(name = "stop")]
    Stop,
    #[qapi(name = "sysrq")]
    Sysrq,
    #[qapi(name = "t")]
    T,
    #[qapi(name = "tab")]
    Tab,
    #[qapi(name = "u")]
    U,
    #[qapi(name = "undo")]
    Undo,
    #[qapi(name = "unmapped")]
    #[qapi(added = "2.0.0")]
    Unmapped,
    #[qapi(name = "up")]
    Up,
    #[qapi(name = "v")]
    V,
    #[qapi(name = "volumedown")]
    #[qapi(added = "2.10.0")]
    Volumedown,
    #[qapi(name = "volumeup")]
    #[qapi(added = "2.10.0")]
    Volumeup,
    #[qapi(name = "w")]
    W,
    #[qapi(name = "wake")]
    #[qapi(added = "2.10.0")]
    Wake,
    #[qapi(name = "x")]
    X,
    #[qapi(name = "y")]
    Y,
    #[qapi(name = "yen")]
    #[qapi(added = "2.9.0")]
    Yen,
    #[qapi(name = "z")]
    Z,
}

/// QMP struct `QKeyCodeWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct QKeyCodeWrapper {
    pub data: QKeyCode,
}

/// QMP enum `QMPCapability`; see the QEMU QMP Reference Manual.
#[qapi(name = "QMPCapability")]
#[qapi(added = "2.12.0")]
pub enum QmpCapability {
    #[qapi(name = "oob")]
    Oob,
}

/// QMP struct `QtestProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.1.0")]
pub struct QtestProperties {
    pub chardev: String,
    pub log: Option<String>,
}

/// QMP enum `QuorumOpType`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.6.0")]
pub enum QuorumOpType {
    #[qapi(name = "flush")]
    Flush,
    #[qapi(name = "read")]
    Read,
    #[qapi(name = "write")]
    Write,
}

/// QMP enum `QuorumReadPattern`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.2.0")]
pub enum QuorumReadPattern {
    #[qapi(name = "fifo")]
    Fifo,
    #[qapi(name = "quorum")]
    Quorum,
}

/// QMP enum `RbdAuthMode`; see the QEMU QMP Reference Manual.
#[qapi(added = "3.0.0")]
pub enum RbdAuthMode {
    #[qapi(name = "cephx")]
    Cephx,
    #[qapi(name = "none")]
    None,
}

/// QMP union `RbdEncryptionCreateOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.1.0")]
pub struct RbdEncryptionCreateOptions {
    #[qapi(discriminator)]
    pub format: RbdImageEncryptionFormat,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<RbdEncryptionCreateOptionsBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "6.1.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum RbdEncryptionCreateOptionsBranch {
    #[qapi(name = "luks")]
    Luks(RbdEncryptionCreateOptionsLuks),
    #[qapi(name = "luks-any")]
    #[qapi(added = "8.0.0")]
    LuksAny,
    #[qapi(name = "luks2")]
    Luks2(RbdEncryptionCreateOptionsLuks2),
}

/// QMP struct `RbdEncryptionCreateOptionsLUKS`; see the QEMU QMP Reference Manual.
#[qapi(name = "RbdEncryptionCreateOptionsLUKS")]
#[qapi(added = "6.1.0")]
pub struct RbdEncryptionCreateOptionsLuks {
    #[qapi(name = "cipher-alg")]
    #[qapi(added = "9.2.0")]
    pub cipher_alg: Option<QCryptoCipherAlgo>,
    #[qapi(name = "cipher-alg")]
    #[qapi(removed = "9.2.0")]
    pub cipher_alg_legacy: Option<QCryptoCipherAlgorithm>,
    #[qapi(name = "key-secret")]
    pub key_secret: String,
}

/// QMP struct `RbdEncryptionCreateOptionsLUKS2`; see the QEMU QMP Reference Manual.
#[qapi(name = "RbdEncryptionCreateOptionsLUKS2")]
#[qapi(added = "6.1.0")]
pub struct RbdEncryptionCreateOptionsLuks2 {
    #[qapi(name = "cipher-alg")]
    #[qapi(added = "9.2.0")]
    pub cipher_alg: Option<QCryptoCipherAlgo>,
    #[qapi(name = "cipher-alg")]
    #[qapi(removed = "9.2.0")]
    pub cipher_alg_legacy: Option<QCryptoCipherAlgorithm>,
    #[qapi(name = "key-secret")]
    pub key_secret: String,
}

/// QMP struct `RbdEncryptionCreateOptionsLUKSBase`; see the QEMU QMP Reference Manual.
#[qapi(name = "RbdEncryptionCreateOptionsLUKSBase")]
#[qapi(added = "6.1.0")]
pub struct RbdEncryptionCreateOptionsLuksBase {
    #[qapi(name = "cipher-alg")]
    #[qapi(added = "9.2.0")]
    pub cipher_alg: Option<QCryptoCipherAlgo>,
    #[qapi(name = "cipher-alg")]
    #[qapi(removed = "9.2.0")]
    pub cipher_alg_legacy: Option<QCryptoCipherAlgorithm>,
    #[qapi(name = "key-secret")]
    pub key_secret: String,
}

/// QMP union `RbdEncryptionOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.1.0")]
pub struct RbdEncryptionOptions {
    #[qapi(discriminator)]
    pub format: RbdImageEncryptionFormat,
    #[qapi(added = "8.0.0")]
    pub parent: Option<Box<RbdEncryptionOptions>>,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<RbdEncryptionOptionsBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "6.1.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum RbdEncryptionOptionsBranch {
    #[qapi(name = "luks")]
    Luks(RbdEncryptionOptionsLuks),
    #[qapi(name = "luks-any")]
    #[qapi(added = "8.0.0")]
    LuksAny(RbdEncryptionOptionsLuksAny),
    #[qapi(name = "luks2")]
    Luks2(RbdEncryptionOptionsLuks2),
}

/// QMP struct `RbdEncryptionOptionsLUKS`; see the QEMU QMP Reference Manual.
#[qapi(name = "RbdEncryptionOptionsLUKS")]
#[qapi(added = "6.1.0")]
pub struct RbdEncryptionOptionsLuks {
    #[qapi(name = "key-secret")]
    pub key_secret: String,
}

/// QMP struct `RbdEncryptionOptionsLUKS2`; see the QEMU QMP Reference Manual.
#[qapi(name = "RbdEncryptionOptionsLUKS2")]
#[qapi(added = "6.1.0")]
pub struct RbdEncryptionOptionsLuks2 {
    #[qapi(name = "key-secret")]
    pub key_secret: String,
}

/// QMP struct `RbdEncryptionOptionsLUKSAny`; see the QEMU QMP Reference Manual.
#[qapi(name = "RbdEncryptionOptionsLUKSAny")]
#[qapi(added = "8.0.0")]
pub struct RbdEncryptionOptionsLuksAny {
    #[qapi(name = "key-secret")]
    pub key_secret: String,
}

/// QMP struct `RbdEncryptionOptionsLUKSBase`; see the QEMU QMP Reference Manual.
#[qapi(name = "RbdEncryptionOptionsLUKSBase")]
#[qapi(added = "6.1.0")]
pub struct RbdEncryptionOptionsLuksBase {
    #[qapi(name = "key-secret")]
    pub key_secret: String,
}

/// QMP enum `RbdImageEncryptionFormat`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.1.0")]
pub enum RbdImageEncryptionFormat {
    #[qapi(name = "luks")]
    Luks,
    #[qapi(name = "luks-any")]
    #[qapi(added = "8.0.0")]
    LuksAny,
    #[qapi(name = "luks2")]
    Luks2,
}

/// QMP enum `RebootAction`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub enum RebootAction {
    #[qapi(name = "reset")]
    Reset,
    #[qapi(name = "shutdown")]
    Shutdown,
}

/// QMP struct `RemoteObjectProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct RemoteObjectProperties {
    pub devid: String,
    pub fd: String,
}

/// QMP struct `ReplayInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "5.2.0")]
pub struct ReplayInfo {
    pub filename: Option<String>,
    pub icount: i64,
    pub mode: ReplayMode,
}

/// QMP enum `ReplayMode`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.5.0")]
pub enum ReplayMode {
    #[qapi(name = "none")]
    None,
    #[qapi(name = "play")]
    Play,
    #[qapi(name = "record")]
    Record,
}

/// QMP enum `ReplicationMode`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_REPLICATION")]
#[qapi(added = "2.8.0")]
pub enum ReplicationMode {
    #[qapi(name = "primary")]
    Primary,
    #[qapi(name = "secondary")]
    Secondary,
}

/// QMP struct `ReplicationStatus`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_REPLICATION")]
#[qapi(added = "2.9.0")]
pub struct ReplicationStatus {
    pub desc: Option<String>,
    pub error: bool,
}

/// QMP struct `RngEgdProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct RngEgdProperties {
    pub chardev: String,
    #[qapi(deprecated(added = "6.0.0"))]
    pub opened: Option<bool>,
}

/// QMP struct `RngProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct RngProperties {
    #[qapi(deprecated(added = "6.0.0"))]
    pub opened: Option<bool>,
}

/// QMP struct `RngRandomProperties`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_POSIX")]
#[qapi(added = "6.0.0")]
pub struct RngRandomProperties {
    pub filename: Option<String>,
    #[qapi(deprecated(added = "6.0.0"))]
    pub opened: Option<bool>,
}

/// QMP struct `RockerOfDpaFlow`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.4.0")]
pub struct RockerOfDpaFlow {
    pub action: RockerOfDpaFlowAction,
    pub cookie: u64,
    pub hits: u64,
    pub key: RockerOfDpaFlowKey,
    pub mask: RockerOfDpaFlowMask,
}

/// QMP struct `RockerOfDpaFlowAction`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.4.0")]
pub struct RockerOfDpaFlowAction {
    #[qapi(name = "goto-tbl")]
    pub goto_tbl: Option<u32>,
    #[qapi(name = "group-id")]
    pub group_id: Option<u32>,
    #[qapi(name = "new-vlan-id")]
    pub new_vlan_id: Option<u16>,
    #[qapi(name = "out-pport")]
    pub out_pport: Option<u32>,
    #[qapi(name = "tunnel-lport")]
    pub tunnel_lport: Option<u32>,
    #[qapi(name = "vlan-id")]
    pub vlan_id: Option<u16>,
}

/// QMP struct `RockerOfDpaFlowKey`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.4.0")]
pub struct RockerOfDpaFlowKey {
    #[qapi(name = "eth-dst")]
    pub eth_dst: Option<String>,
    #[qapi(name = "eth-src")]
    pub eth_src: Option<String>,
    #[qapi(name = "eth-type")]
    pub eth_type: Option<u16>,
    #[qapi(name = "in-pport")]
    pub in_pport: Option<u32>,
    #[qapi(name = "ip-dst")]
    pub ip_dst: Option<String>,
    #[qapi(name = "ip-proto")]
    pub ip_proto: Option<u8>,
    #[qapi(name = "ip-tos")]
    pub ip_tos: Option<u8>,
    pub priority: u32,
    #[qapi(name = "tbl-id")]
    pub tbl_id: u32,
    #[qapi(name = "tunnel-id")]
    pub tunnel_id: Option<u32>,
    #[qapi(name = "vlan-id")]
    pub vlan_id: Option<u16>,
}

/// QMP struct `RockerOfDpaFlowMask`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.4.0")]
pub struct RockerOfDpaFlowMask {
    #[qapi(name = "eth-dst")]
    pub eth_dst: Option<String>,
    #[qapi(name = "eth-src")]
    pub eth_src: Option<String>,
    #[qapi(name = "in-pport")]
    pub in_pport: Option<u32>,
    #[qapi(name = "ip-proto")]
    pub ip_proto: Option<u8>,
    #[qapi(name = "ip-tos")]
    pub ip_tos: Option<u8>,
    #[qapi(name = "tunnel-id")]
    pub tunnel_id: Option<u32>,
    #[qapi(name = "vlan-id")]
    pub vlan_id: Option<u16>,
}

/// QMP struct `RockerOfDpaGroup`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.4.0")]
pub struct RockerOfDpaGroup {
    #[qapi(name = "group-id")]
    pub group_id: Option<u32>,
    #[qapi(name = "group-ids")]
    pub group_ids: Option<Vec<u32>>,
    pub id: u32,
    pub index: Option<u32>,
    #[qapi(name = "out-pport")]
    pub out_pport: Option<u32>,
    #[qapi(name = "pop-vlan")]
    pub pop_vlan: Option<u8>,
    pub pport: Option<u32>,
    #[qapi(name = "set-eth-dst")]
    pub set_eth_dst: Option<String>,
    #[qapi(name = "set-eth-src")]
    pub set_eth_src: Option<String>,
    #[qapi(name = "set-vlan-id")]
    pub set_vlan_id: Option<u16>,
    #[qapi(name = "ttl-check")]
    pub ttl_check: Option<u8>,
    pub r#type: u8,
    #[qapi(name = "vlan-id")]
    pub vlan_id: Option<u16>,
}

/// QMP struct `RockerPort`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.4.0")]
pub struct RockerPort {
    pub autoneg: RockerPortAutoneg,
    pub duplex: RockerPortDuplex,
    pub enabled: bool,
    #[qapi(name = "link-up")]
    pub link_up: bool,
    pub name: String,
    pub speed: u32,
}

/// QMP enum `RockerPortAutoneg`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.4.0")]
pub enum RockerPortAutoneg {
    #[qapi(name = "off")]
    Off,
    #[qapi(name = "on")]
    On,
}

/// QMP enum `RockerPortDuplex`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.4.0")]
pub enum RockerPortDuplex {
    #[qapi(name = "full")]
    Full,
    #[qapi(name = "half")]
    Half,
}

/// QMP struct `RockerSwitch`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.4.0")]
pub struct RockerSwitch {
    pub id: u64,
    pub name: String,
    pub ports: u32,
}

/// QMP enum `RunState`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.0.0")]
pub enum RunState {
    #[qapi(name = "colo")]
    #[qapi(added = "2.8.0")]
    Colo,
    #[qapi(name = "debug")]
    Debug,
    #[qapi(name = "finish-migrate")]
    FinishMigrate,
    #[qapi(name = "guest-panicked")]
    #[qapi(added = "1.5.0")]
    GuestPanicked,
    #[qapi(name = "inmigrate")]
    Inmigrate,
    #[qapi(name = "internal-error")]
    InternalError,
    #[qapi(name = "io-error")]
    IoError,
    #[qapi(name = "paused")]
    Paused,
    #[qapi(name = "postmigrate")]
    Postmigrate,
    #[qapi(name = "preconfig")]
    #[qapi(added = "3.0.0")]
    #[qapi(removed = "6.0.0")]
    Preconfig,
    #[qapi(name = "prelaunch")]
    Prelaunch,
    #[qapi(name = "restore-vm")]
    RestoreVm,
    #[qapi(name = "running")]
    Running,
    #[qapi(name = "save-vm")]
    SaveVm,
    #[qapi(name = "shutdown")]
    Shutdown,
    #[qapi(name = "suspended")]
    #[qapi(added = "1.1.0")]
    Suspended,
    #[qapi(name = "watchdog")]
    Watchdog,
}

/// QMP struct `RxFilterInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.6.0")]
pub struct RxFilterInfo {
    #[qapi(name = "broadcast-allowed")]
    pub broadcast_allowed: bool,
    #[qapi(name = "main-mac")]
    pub main_mac: String,
    pub multicast: RxState,
    #[qapi(name = "multicast-overflow")]
    pub multicast_overflow: bool,
    #[qapi(name = "multicast-table")]
    pub multicast_table: Vec<String>,
    pub name: String,
    pub promiscuous: bool,
    pub unicast: RxState,
    #[qapi(name = "unicast-overflow")]
    pub unicast_overflow: bool,
    #[qapi(name = "unicast-table")]
    pub unicast_table: Vec<String>,
    #[qapi(added = "2.0.0")]
    #[qapi(required)]
    pub vlan: Option<RxState>,
    #[qapi(name = "vlan-table")]
    pub vlan_table: Vec<i64>,
}

/// QMP enum `RxState`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.6.0")]
pub enum RxState {
    #[qapi(name = "all")]
    All,
    #[qapi(name = "none")]
    None,
    #[qapi(name = "normal")]
    Normal,
}

/// QMP enum `S390CpuEntitlement`; see the QEMU QMP Reference Manual.
#[qapi(added = "9.2.0")]
pub enum S390CpuEntitlement {
    #[qapi(name = "auto")]
    Auto,
    #[qapi(name = "high")]
    High,
    #[qapi(name = "low")]
    Low,
    #[qapi(name = "medium")]
    Medium,
}

/// QMP enum `S390CpuPolarization`; see the QEMU QMP Reference Manual.
#[qapi(added = "9.2.0")]
pub enum S390CpuPolarization {
    #[qapi(name = "horizontal")]
    Horizontal,
    #[qapi(name = "vertical")]
    Vertical,
}

/// QMP enum `S390CpuState`; see the QEMU QMP Reference Manual.
#[qapi(added = "9.2.0")]
pub enum S390CpuState {
    #[qapi(name = "check-stop")]
    CheckStop,
    #[qapi(name = "load")]
    Load,
    #[qapi(name = "operating")]
    Operating,
    #[qapi(name = "stopped")]
    Stopped,
    #[qapi(name = "uninitialized")]
    Uninitialized,
}

/// QMP enum `S390CrashReason`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
pub enum S390CrashReason {
    #[qapi(name = "disabled-wait")]
    DisabledWait,
    #[qapi(name = "extint-loop")]
    ExtintLoop,
    #[qapi(name = "opint-loop")]
    OpintLoop,
    #[qapi(name = "pgmint-loop")]
    PgmintLoop,
    #[qapi(name = "unknown")]
    Unknown,
}

/// QMP union `SchemaInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.5.0")]
pub struct SchemaInfo {
    #[qapi(added = "5.0.0")]
    pub features: Option<Vec<String>>,
    #[qapi(name = "meta-type")]
    #[qapi(discriminator)]
    pub meta_type: SchemaMetaType,
    pub name: String,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<SchemaInfoBranch>,
}

/// QMP struct `SchemaInfoAlternate`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.5.0")]
pub struct SchemaInfoAlternate {
    pub members: Vec<SchemaInfoAlternateMember>,
}

/// QMP struct `SchemaInfoAlternateMember`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.5.0")]
pub struct SchemaInfoAlternateMember {
    pub r#type: String,
}

/// QMP struct `SchemaInfoArray`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.5.0")]
pub struct SchemaInfoArray {
    #[qapi(name = "element-type")]
    pub element_type: String,
}

/// QMP struct `SchemaInfoBase`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.5.0")]
#[qapi(removed = "2.6.0")]
pub struct SchemaInfoBase {
    #[qapi(name = "meta-type")]
    pub meta_type: SchemaMetaType,
    pub name: String,
}

#[qapi(union_branch)]
#[qapi(added = "2.5.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum SchemaInfoBranch {
    #[qapi(name = "alternate")]
    Alternate(SchemaInfoAlternate),
    #[qapi(name = "array")]
    Array(SchemaInfoArray),
    #[qapi(name = "builtin")]
    Builtin(SchemaInfoBuiltin),
    #[qapi(name = "command")]
    Command(SchemaInfoCommand),
    #[qapi(name = "enum")]
    Enum(SchemaInfoEnum),
    #[qapi(name = "event")]
    Event(SchemaInfoEvent),
    #[qapi(name = "object")]
    Object(SchemaInfoObject),
}

/// QMP struct `SchemaInfoBuiltin`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.5.0")]
pub struct SchemaInfoBuiltin {
    #[qapi(name = "json-type")]
    pub json_type: JsonType,
}

/// QMP struct `SchemaInfoCommand`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.5.0")]
pub struct SchemaInfoCommand {
    #[qapi(name = "allow-oob")]
    #[qapi(added = "3.0.0")]
    pub allow_oob: Option<bool>,
    #[qapi(name = "allow-oob")]
    #[qapi(added = "2.12.0")]
    #[qapi(removed = "3.0.0")]
    #[qapi(required)]
    pub allow_oob_legacy: Option<bool>,
    #[qapi(name = "arg-type")]
    pub arg_type: String,
    #[qapi(added = "4.2.0")]
    #[qapi(removed = "5.0.0")]
    pub features: Option<Vec<String>>,
    #[qapi(name = "ret-type")]
    pub ret_type: String,
}

/// QMP struct `SchemaInfoEnum`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.5.0")]
pub struct SchemaInfoEnum {
    #[qapi(added = "6.2.0")]
    #[qapi(required)]
    pub members: Option<Vec<SchemaInfoEnumMember>>,
    #[qapi(deprecated(added = "6.2.0"))]
    pub values: Vec<String>,
}

/// QMP struct `SchemaInfoEnumMember`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct SchemaInfoEnumMember {
    pub features: Option<Vec<String>>,
    pub name: String,
}

/// QMP struct `SchemaInfoEvent`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.5.0")]
pub struct SchemaInfoEvent {
    #[qapi(name = "arg-type")]
    pub arg_type: String,
}

/// QMP struct `SchemaInfoObject`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.5.0")]
pub struct SchemaInfoObject {
    #[qapi(added = "4.1.0")]
    #[qapi(removed = "5.0.0")]
    pub features: Option<Vec<String>>,
    pub members: Vec<SchemaInfoObjectMember>,
    pub tag: Option<String>,
    pub variants: Option<Vec<SchemaInfoObjectVariant>>,
}

/// QMP struct `SchemaInfoObjectMember`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.5.0")]
pub struct SchemaInfoObjectMember {
    pub default: Option<serde_json::Value>,
    #[qapi(added = "5.0.0")]
    pub features: Option<Vec<String>>,
    pub name: String,
    pub r#type: String,
}

/// QMP struct `SchemaInfoObjectVariant`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.5.0")]
pub struct SchemaInfoObjectVariant {
    pub case: String,
    pub r#type: String,
}

/// QMP enum `SchemaMetaType`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.5.0")]
pub enum SchemaMetaType {
    #[qapi(name = "alternate")]
    Alternate,
    #[qapi(name = "array")]
    Array,
    #[qapi(name = "builtin")]
    Builtin,
    #[qapi(name = "command")]
    Command,
    #[qapi(name = "enum")]
    Enum,
    #[qapi(name = "event")]
    Event,
    #[qapi(name = "object")]
    Object,
}

/// QMP struct `SecretCommonProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct SecretCommonProperties {
    pub format: Option<QCryptoSecretFormat>,
    pub iv: Option<String>,
    pub keyid: Option<String>,
    #[qapi(deprecated(added = "6.0.0"))]
    #[qapi(removed = "9.2.0")]
    pub loaded: Option<bool>,
}

/// QMP struct `SecretKeyringProperties`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_SECRET_KEYRING")]
#[qapi(added = "6.0.0")]
pub struct SecretKeyringProperties {
    pub format: Option<QCryptoSecretFormat>,
    pub iv: Option<String>,
    pub keyid: Option<String>,
    #[qapi(deprecated(added = "6.0.0"))]
    #[qapi(removed = "9.2.0")]
    pub loaded: Option<bool>,
    pub serial: i32,
}

/// QMP struct `SecretProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct SecretProperties {
    pub data: Option<String>,
    pub file: Option<String>,
    pub format: Option<QCryptoSecretFormat>,
    pub iv: Option<String>,
    pub keyid: Option<String>,
    #[qapi(deprecated(added = "6.0.0"))]
    #[qapi(removed = "9.2.0")]
    pub loaded: Option<bool>,
}

/// QMP enum `SetPasswordAction`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.0.0")]
pub enum SetPasswordAction {
    #[qapi(name = "disconnect")]
    Disconnect,
    #[qapi(name = "fail")]
    Fail,
    #[qapi(name = "keep")]
    Keep,
}

/// QMP union `SetPasswordOptions`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.0.0")]
pub struct SetPasswordOptions {
    pub connected: Option<SetPasswordAction>,
    pub password: String,
    #[qapi(discriminator)]
    pub protocol: DisplayProtocol,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<SetPasswordOptionsBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "7.0.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum SetPasswordOptionsBranch {
    #[qapi(name = "spice")]
    Spice,
    #[qapi(name = "vnc")]
    Vnc(SetPasswordOptionsVnc),
}

/// QMP struct `SetPasswordOptionsVnc`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.0.0")]
pub struct SetPasswordOptionsVnc {
    pub display: Option<String>,
}

/// QMP struct `SevAttestationReport`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.1.0")]
pub struct SevAttestationReport {
    pub data: String,
}

/// QMP struct `SevCapability`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
pub struct SevCapability {
    pub cbitpos: i64,
    #[qapi(name = "cert-chain")]
    pub cert_chain: String,
    #[qapi(name = "cpu0-id")]
    #[qapi(added = "7.1.0")]
    #[qapi(required)]
    pub cpu0_id: Option<String>,
    pub pdh: String,
    #[qapi(name = "reduced-phys-bits")]
    pub reduced_phys_bits: i64,
}

/// QMP struct `SevCommonProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "9.1.0")]
pub struct SevCommonProperties {
    pub cbitpos: Option<u32>,
    #[qapi(name = "kernel-hashes")]
    pub kernel_hashes: Option<bool>,
    #[qapi(name = "reduced-phys-bits")]
    pub reduced_phys_bits: u32,
    #[qapi(name = "sev-device")]
    pub sev_device: Option<String>,
}

/// QMP struct `SevGuestInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "9.1.0")]
pub struct SevGuestInfo {
    pub handle: u32,
    pub policy: u32,
}

/// QMP struct `SevGuestProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct SevGuestProperties {
    pub cbitpos: Option<u32>,
    #[qapi(name = "dh-cert-file")]
    pub dh_cert_file: Option<String>,
    pub handle: Option<u32>,
    #[qapi(name = "kernel-hashes")]
    #[qapi(added = "6.2.0")]
    pub kernel_hashes: Option<bool>,
    #[qapi(name = "legacy-vm-type")]
    #[qapi(added = "9.1.0")]
    pub legacy_vm_type: Option<OnOffAuto>,
    pub policy: Option<u32>,
    #[qapi(name = "reduced-phys-bits")]
    pub reduced_phys_bits: u32,
    #[qapi(name = "session-file")]
    pub session_file: Option<String>,
    #[qapi(name = "sev-device")]
    pub sev_device: Option<String>,
}

/// QMP enum `SevGuestType`; see the QEMU QMP Reference Manual.
#[qapi(added = "9.1.0")]
pub enum SevGuestType {
    #[qapi(name = "sev")]
    Sev,
    #[qapi(name = "sev-snp")]
    SevSnp,
}

/// QMP union `SevInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
pub struct SevInfo {
    #[qapi(name = "api-major")]
    pub api_major: u8,
    #[qapi(name = "api-minor")]
    pub api_minor: u8,
    #[qapi(name = "build-id")]
    pub build_id: u8,
    pub enabled: bool,
    #[qapi(removed = "9.1.0")]
    #[qapi(required)]
    pub handle: Option<u32>,
    #[qapi(removed = "9.1.0")]
    #[qapi(required)]
    pub policy: Option<u32>,
    #[qapi(name = "sev-type")]
    #[qapi(discriminator)]
    #[qapi(added = "9.1.0")]
    #[qapi(required)]
    pub sev_type: Option<SevGuestType>,
    pub state: SevState,
    #[qapi(union)]
    #[qapi(flatten)]
    #[qapi(added = "9.1.0")]
    pub u: Option<SevInfoBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "9.1.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum SevInfoBranch {
    #[qapi(name = "sev")]
    Sev(SevGuestInfo),
    #[qapi(name = "sev-snp")]
    SevSnp(SevSnpGuestInfo),
}

/// QMP struct `SevLaunchMeasureInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
pub struct SevLaunchMeasureInfo {
    pub data: String,
}

/// QMP struct `SevSnpGuestInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "9.1.0")]
pub struct SevSnpGuestInfo {
    #[qapi(name = "snp-policy")]
    pub snp_policy: u64,
}

/// QMP struct `SevSnpGuestProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "9.1.0")]
pub struct SevSnpGuestProperties {
    #[qapi(name = "author-key-enabled")]
    pub author_key_enabled: Option<bool>,
    pub cbitpos: Option<u32>,
    #[qapi(name = "guest-visible-workarounds")]
    pub guest_visible_workarounds: Option<String>,
    #[qapi(name = "host-data")]
    pub host_data: Option<String>,
    #[qapi(name = "id-auth")]
    pub id_auth: Option<String>,
    #[qapi(name = "id-block")]
    pub id_block: Option<String>,
    #[qapi(name = "kernel-hashes")]
    pub kernel_hashes: Option<bool>,
    pub policy: Option<u64>,
    #[qapi(name = "reduced-phys-bits")]
    pub reduced_phys_bits: u32,
    #[qapi(name = "sev-device")]
    pub sev_device: Option<String>,
    #[qapi(name = "vcek-disabled")]
    pub vcek_disabled: Option<bool>,
}

/// QMP enum `SevState`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
pub enum SevState {
    #[qapi(name = "launch-secret")]
    LaunchSecret,
    #[qapi(name = "launch-update")]
    LaunchUpdate,
    #[qapi(name = "receive-update")]
    ReceiveUpdate,
    #[qapi(name = "running")]
    Running,
    #[qapi(name = "send-update")]
    SendUpdate,
    #[qapi(name = "uninit")]
    Uninit,
}

/// QMP struct `SgxEPC`; see the QEMU QMP Reference Manual.
#[qapi(name = "SgxEPC")]
#[qapi(added = "6.2.0")]
pub struct SgxEpc {
    pub memdev: String,
    #[qapi(added = "7.0.0")]
    #[qapi(required)]
    pub node: Option<i64>,
}

/// QMP struct `SgxEPCDeviceInfo`; see the QEMU QMP Reference Manual.
#[qapi(name = "SgxEPCDeviceInfo")]
#[qapi(added = "6.2.0")]
pub struct SgxEpcDeviceInfo {
    pub id: Option<String>,
    pub memaddr: u64,
    pub memdev: String,
    #[qapi(added = "7.0.0")]
    #[qapi(required)]
    pub node: Option<i64>,
    pub size: u64,
}

/// QMP struct `SgxEPCDeviceInfoWrapper`; see the QEMU QMP Reference Manual.
#[qapi(name = "SgxEPCDeviceInfoWrapper")]
#[qapi(added = "6.2.0")]
pub struct SgxEpcDeviceInfoWrapper {
    pub data: SgxEpcDeviceInfo,
}

/// QMP struct `SgxEPCProperties`; see the QEMU QMP Reference Manual.
#[qapi(name = "SgxEPCProperties")]
#[qapi(added = "6.2.0")]
pub struct SgxEpcProperties {
    #[qapi(name = "sgx-epc")]
    pub sgx_epc: Vec<SgxEpc>,
}

/// QMP struct `SgxEpcSection`; see the QEMU QMP Reference Manual.
#[qapi(added = "10.1.0")]
pub struct SgxEpcSection {
    pub node: i64,
    pub size: u64,
}

/// QMP struct `SGXEPCSection`; see the QEMU QMP Reference Manual.
#[qapi(name = "SGXEPCSection")]
#[qapi(added = "7.0.0")]
#[qapi(removed = "10.1.0")]
pub struct SgxepcSection {
    pub node: i64,
    pub size: u64,
}

/// QMP struct `SgxInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct SgxInfo {
    pub flc: bool,
    #[qapi(name = "section-size")]
    #[qapi(deprecated(added = "7.0.0"))]
    #[qapi(removed = "8.0.0")]
    #[qapi(required)]
    pub section_size: Option<u64>,
    #[qapi(added = "10.1.0")]
    #[qapi(required)]
    pub sections: Option<Vec<SgxEpcSection>>,
    #[qapi(name = "sections")]
    #[qapi(added = "7.0.0")]
    #[qapi(removed = "10.1.0")]
    #[qapi(required)]
    pub sections_legacy: Option<Vec<SgxepcSection>>,
    pub sgx: bool,
    pub sgx1: bool,
    pub sgx2: bool,
}

/// QMP union `SheepdogRedundancy`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
#[qapi(removed = "6.1.0")]
pub struct SheepdogRedundancy {
    #[qapi(discriminator)]
    pub r#type: SheepdogRedundancyType,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<SheepdogRedundancyBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "2.12.0")]
#[qapi(removed = "6.1.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum SheepdogRedundancyBranch {
    #[qapi(name = "erasure-coded")]
    ErasureCoded(SheepdogRedundancyErasureCoded),
    #[qapi(name = "full")]
    Full(SheepdogRedundancyFull),
}

/// QMP struct `SheepdogRedundancyErasureCoded`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
#[qapi(removed = "6.1.0")]
pub struct SheepdogRedundancyErasureCoded {
    #[qapi(name = "data-strips")]
    pub data_strips: i64,
    #[qapi(name = "parity-strips")]
    pub parity_strips: i64,
}

/// QMP struct `SheepdogRedundancyFull`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
#[qapi(removed = "6.1.0")]
pub struct SheepdogRedundancyFull {
    pub copies: i64,
}

/// QMP enum `SheepdogRedundancyType`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
#[qapi(removed = "6.1.0")]
pub enum SheepdogRedundancyType {
    #[qapi(name = "erasure-coded")]
    ErasureCoded,
    #[qapi(name = "full")]
    Full,
}

/// QMP enum `ShutdownAction`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub enum ShutdownAction {
    #[qapi(name = "pause")]
    Pause,
    #[qapi(name = "poweroff")]
    Poweroff,
}

/// QMP enum `ShutdownCause`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.0.0")]
pub enum ShutdownCause {
    #[qapi(name = "guest-panic")]
    GuestPanic,
    #[qapi(name = "guest-reset")]
    GuestReset,
    #[qapi(name = "guest-shutdown")]
    GuestShutdown,
    #[qapi(name = "host-error")]
    HostError,
    #[qapi(name = "host-qmp-quit")]
    HostQmpQuit,
    #[qapi(name = "host-qmp-system-reset")]
    HostQmpSystemReset,
    #[qapi(name = "host-signal")]
    HostSignal,
    #[qapi(name = "host-ui")]
    HostUi,
    #[qapi(name = "none")]
    None,
    #[qapi(name = "snapshot-load")]
    #[qapi(added = "7.2.0")]
    SnapshotLoad,
    #[qapi(name = "subsystem-reset")]
    SubsystemReset,
}

/// QMP enum `SmbiosEntryPointType`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.0.0")]
pub enum SmbiosEntryPointType {
    #[qapi(name = "32")]
    _32,
    #[qapi(name = "64")]
    _64,
    #[qapi(name = "auto")]
    #[qapi(added = "9.0.0")]
    Auto,
}

/// QMP struct `SmpCacheProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "9.2.0")]
pub struct SmpCacheProperties {
    pub cache: CacheLevelAndType,
    pub topology: CpuTopologyLevel,
}

/// QMP struct `SmpCachePropertiesWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "9.2.0")]
pub struct SmpCachePropertiesWrapper {
    pub caches: Vec<SmpCacheProperties>,
}

/// QMP struct `SMPConfiguration`; see the QEMU QMP Reference Manual.
#[qapi(name = "SMPConfiguration")]
#[qapi(added = "6.1.0")]
pub struct SmpConfiguration {
    #[qapi(added = "8.2.0")]
    pub books: Option<i64>,
    #[qapi(added = "7.0.0")]
    pub clusters: Option<i64>,
    pub cores: Option<i64>,
    pub cpus: Option<i64>,
    pub dies: Option<i64>,
    #[qapi(added = "8.2.0")]
    pub drawers: Option<i64>,
    pub maxcpus: Option<i64>,
    #[qapi(added = "9.1.0")]
    pub modules: Option<i64>,
    pub sockets: Option<i64>,
    pub threads: Option<i64>,
}

/// QMP struct `SnapshotInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.3.0")]
pub struct SnapshotInfo {
    #[qapi(name = "date-nsec")]
    pub date_nsec: i64,
    #[qapi(name = "date-sec")]
    pub date_sec: i64,
    #[qapi(added = "5.2.0")]
    pub icount: Option<i64>,
    pub id: String,
    pub name: String,
    #[qapi(name = "vm-clock-nsec")]
    pub vm_clock_nsec: i64,
    #[qapi(name = "vm-clock-sec")]
    pub vm_clock_sec: i64,
    #[qapi(name = "vm-state-size")]
    pub vm_state_size: i64,
}

/// QMP union `SocketAddress`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.3.0")]
#[qapi(simple_union(removed = "2.10.0"))]
pub struct SocketAddress {
    #[qapi(discriminator)]
    #[qapi(name = "type")]
    #[qapi(removed = "2.10.0")]
    #[qapi(required)]
    pub r_type: Option<SocketAddressKind>,
    #[qapi(discriminator)]
    #[qapi(added = "2.10.0")]
    #[qapi(required)]
    pub r#type: Option<SocketAddressType>,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<SocketAddressBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "1.3.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum SocketAddressBranch {
    #[qapi(name = "fd")]
    #[qapi(added = "9.0.0")]
    Fd(FdSocketAddress),
    #[qapi(name = "fd")]
    #[qapi(removed = "9.0.0")]
    FdLegacy(QapiString),
    #[qapi(name = "inet")]
    Inet(InetSocketAddress),
    #[qapi(name = "unix")]
    Unix(UnixSocketAddress),
    #[qapi(name = "vsock")]
    #[qapi(added = "2.8.0")]
    Vsock(VsockSocketAddress),
}

/// QMP union `SocketAddressFlat`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.9.0")]
#[qapi(removed = "2.10.0")]
pub struct SocketAddressFlat {
    #[qapi(discriminator)]
    pub r#type: SocketAddressFlatType,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<SocketAddressFlatBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "2.9.0")]
#[qapi(removed = "2.10.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum SocketAddressFlatBranch {
    #[qapi(name = "fd")]
    Fd(QapiString),
    #[qapi(name = "inet")]
    Inet(InetSocketAddress),
    #[qapi(name = "unix")]
    Unix(UnixSocketAddress),
    #[qapi(name = "vsock")]
    Vsock(VsockSocketAddress),
}

/// QMP enum `SocketAddressFlatType`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.9.0")]
#[qapi(removed = "2.10.0")]
pub enum SocketAddressFlatType {
    #[qapi(name = "fd")]
    Fd,
    #[qapi(name = "inet")]
    Inet,
    #[qapi(name = "unix")]
    Unix,
    #[qapi(name = "vsock")]
    Vsock,
}

#[qapi(added = "1.3.0")]
#[qapi(removed = "2.10.0")]
pub enum SocketAddressKind {
    #[qapi(name = "fd")]
    Fd,
    #[qapi(name = "inet")]
    Inet,
    #[qapi(name = "unix")]
    Unix,
    #[qapi(name = "vsock")]
    #[qapi(added = "2.8.0")]
    Vsock,
}

/// QMP union `SocketAddressLegacy`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.10.0")]
#[qapi(simple_union(removed = "6.2.0"))]
pub struct SocketAddressLegacy {
    #[qapi(discriminator)]
    #[qapi(name = "type")]
    #[qapi(removed = "6.2.0")]
    #[qapi(required)]
    pub r_type: Option<SocketAddressLegacyKind>,
    #[qapi(discriminator)]
    #[qapi(added = "6.2.0")]
    #[qapi(required)]
    pub r#type: Option<SocketAddressType>,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<SocketAddressLegacyBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "2.10.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum SocketAddressLegacyBranch {
    #[qapi(name = "fd")]
    #[qapi(added = "9.0.0")]
    Fd(FdSocketAddressWrapper),
    #[qapi(name = "fd")]
    #[qapi(removed = "6.2.0")]
    FdLegacy(QapiString),
    #[qapi(name = "fd")]
    #[qapi(added = "6.2.0")]
    #[qapi(removed = "9.0.0")]
    FdLegacy2(StringWrapper),
    #[qapi(name = "inet")]
    #[qapi(added = "6.2.0")]
    Inet(InetSocketAddressWrapper),
    #[qapi(name = "inet")]
    #[qapi(removed = "6.2.0")]
    InetLegacy(InetSocketAddress),
    #[qapi(name = "unix")]
    #[qapi(added = "6.2.0")]
    Unix(UnixSocketAddressWrapper),
    #[qapi(name = "unix")]
    #[qapi(removed = "6.2.0")]
    UnixLegacy(UnixSocketAddress),
    #[qapi(name = "vsock")]
    #[qapi(added = "6.2.0")]
    Vsock(VsockSocketAddressWrapper),
    #[qapi(name = "vsock")]
    #[qapi(removed = "6.2.0")]
    VsockLegacy(VsockSocketAddress),
}

#[qapi(added = "2.10.0")]
#[qapi(removed = "6.2.0")]
pub enum SocketAddressLegacyKind {
    #[qapi(name = "fd")]
    Fd,
    #[qapi(name = "inet")]
    Inet,
    #[qapi(name = "unix")]
    Unix,
    #[qapi(name = "vsock")]
    Vsock,
}

/// QMP enum `SocketAddressType`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.10.0")]
pub enum SocketAddressType {
    #[qapi(name = "fd")]
    Fd,
    #[qapi(name = "inet")]
    Inet,
    #[qapi(name = "unix")]
    Unix,
    #[qapi(name = "vsock")]
    Vsock,
}

/// QMP struct `SpiceBasicInfo`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_SPICE")]
#[qapi(added = "2.1.0")]
pub struct SpiceBasicInfo {
    pub family: NetworkAddressFamily,
    pub host: String,
    pub port: String,
}

/// QMP struct `SpiceChannel`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_SPICE")]
#[qapi(added = "1.0.0")]
pub struct SpiceChannel {
    #[qapi(name = "channel-id")]
    pub channel_id: i64,
    #[qapi(name = "channel-type")]
    pub channel_type: i64,
    #[qapi(name = "connection-id")]
    pub connection_id: i64,
    #[qapi(added = "2.1.0")]
    #[qapi(required)]
    pub family: Option<NetworkAddressFamily>,
    #[qapi(name = "family")]
    #[qapi(removed = "2.1.0")]
    #[qapi(required)]
    pub family_legacy: Option<String>,
    pub host: String,
    pub port: String,
    pub tls: bool,
}

/// QMP struct `SpiceInfo`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_SPICE")]
#[qapi(added = "1.0.0")]
pub struct SpiceInfo {
    pub auth: Option<String>,
    pub channels: Option<Vec<SpiceChannel>>,
    #[qapi(name = "compiled-version")]
    pub compiled_version: Option<String>,
    pub enabled: bool,
    pub host: Option<String>,
    #[qapi(added = "1.2.1")]
    #[qapi(required)]
    pub migrated: Option<bool>,
    #[qapi(name = "mouse-mode")]
    #[qapi(added = "1.1.0")]
    #[qapi(required)]
    pub mouse_mode: Option<SpiceQueryMouseMode>,
    pub port: Option<i64>,
    #[qapi(name = "tls-port")]
    pub tls_port: Option<i64>,
}

/// QMP enum `SpiceQueryMouseMode`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_SPICE")]
#[qapi(added = "1.1.0")]
pub enum SpiceQueryMouseMode {
    #[qapi(name = "client")]
    Client,
    #[qapi(name = "server")]
    Server,
    #[qapi(name = "unknown")]
    Unknown,
}

/// QMP struct `SpiceServerInfo`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_SPICE")]
#[qapi(added = "2.1.0")]
pub struct SpiceServerInfo {
    pub auth: Option<String>,
    pub family: NetworkAddressFamily,
    pub host: String,
    pub port: String,
}

/// QMP struct `SpMemDeviceInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "11.1.0")]
pub struct SpMemDeviceInfo {
    pub addr: u64,
    pub id: Option<String>,
    pub memdev: String,
    pub node: i64,
    pub size: u64,
}

/// QMP struct `SpMemDeviceInfoWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "11.1.0")]
pub struct SpMemDeviceInfoWrapper {
    pub data: SpMemDeviceInfo,
}

/// QMP union `SshHostKeyCheck`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
pub struct SshHostKeyCheck {
    #[qapi(discriminator)]
    pub mode: SshHostKeyCheckMode,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<SshHostKeyCheckBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "2.12.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum SshHostKeyCheckBranch {
    #[qapi(name = "hash")]
    Hash(SshHostKeyHash),
    #[qapi(name = "known_hosts")]
    #[qapi(added = "3.0.0")]
    KnownHosts,
    #[qapi(name = "known_hosts")]
    #[qapi(removed = "3.0.0")]
    KnownHostsLegacy(SshHostKeyDummy),
    #[qapi(name = "none")]
    #[qapi(added = "3.0.0")]
    None,
    #[qapi(name = "none")]
    #[qapi(removed = "3.0.0")]
    NoneLegacy(SshHostKeyDummy),
}

/// QMP enum `SshHostKeyCheckHashType`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
pub enum SshHostKeyCheckHashType {
    #[qapi(name = "md5")]
    Md5,
    #[qapi(name = "sha1")]
    Sha1,
    #[qapi(name = "sha256")]
    #[qapi(added = "6.1.0")]
    Sha256,
}

/// QMP enum `SshHostKeyCheckMode`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
pub enum SshHostKeyCheckMode {
    #[qapi(name = "hash")]
    Hash,
    #[qapi(name = "known_hosts")]
    KnownHosts,
    #[qapi(name = "none")]
    None,
}

/// QMP struct `SshHostKeyDummy`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
#[qapi(removed = "3.0.0")]
pub struct SshHostKeyDummy {}

/// QMP struct `SshHostKeyHash`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.12.0")]
pub struct SshHostKeyHash {
    pub hash: String,
    pub r#type: SshHostKeyCheckHashType,
}

/// QMP enum `SsidSizeMode`; see the QEMU QMP Reference Manual.
#[qapi(added = "11.0.0")]
pub enum SsidSizeMode {
    #[qapi(name = "0")]
    _0,
    #[qapi(name = "1")]
    _1,
    #[qapi(name = "10")]
    _10,
    #[qapi(name = "11")]
    _11,
    #[qapi(name = "12")]
    _12,
    #[qapi(name = "13")]
    _13,
    #[qapi(name = "14")]
    _14,
    #[qapi(name = "15")]
    _15,
    #[qapi(name = "16")]
    _16,
    #[qapi(name = "17")]
    _17,
    #[qapi(name = "18")]
    _18,
    #[qapi(name = "19")]
    _19,
    #[qapi(name = "2")]
    _2,
    #[qapi(name = "20")]
    _20,
    #[qapi(name = "3")]
    _3,
    #[qapi(name = "4")]
    _4,
    #[qapi(name = "5")]
    _5,
    #[qapi(name = "6")]
    _6,
    #[qapi(name = "7")]
    _7,
    #[qapi(name = "8")]
    _8,
    #[qapi(name = "9")]
    _9,
    #[qapi(name = "auto")]
    Auto,
}

/// QMP struct `Stats`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.1.0")]
pub struct Stats {
    pub name: String,
    pub value: StatsValue,
}

/// QMP union `StatsFilter`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.1.0")]
pub struct StatsFilter {
    pub providers: Option<Vec<StatsRequest>>,
    #[qapi(discriminator)]
    pub target: StatsTarget,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<StatsFilterBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "7.1.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum StatsFilterBranch {
    #[qapi(name = "cryptodev")]
    #[qapi(added = "8.0.0")]
    Cryptodev,
    #[qapi(name = "vcpu")]
    Vcpu(StatsVcpuFilter),
    #[qapi(name = "vm")]
    Vm,
}

/// QMP enum `StatsProvider`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.1.0")]
pub enum StatsProvider {
    #[qapi(name = "cryptodev")]
    #[qapi(added = "8.0.0")]
    Cryptodev,
    #[qapi(name = "kvm")]
    Kvm,
}

/// QMP struct `StatsRequest`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.1.0")]
pub struct StatsRequest {
    pub names: Option<Vec<String>>,
    pub provider: StatsProvider,
}

/// QMP struct `StatsResult`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.1.0")]
pub struct StatsResult {
    pub provider: StatsProvider,
    #[qapi(name = "qom-path")]
    pub qom_path: Option<String>,
    pub stats: Vec<Stats>,
}

/// QMP struct `StatsSchema`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.1.0")]
pub struct StatsSchema {
    pub provider: StatsProvider,
    pub stats: Vec<StatsSchemaValue>,
    pub target: StatsTarget,
}

/// QMP struct `StatsSchemaValue`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.1.0")]
pub struct StatsSchemaValue {
    pub base: Option<i8>,
    #[qapi(name = "bucket-size")]
    pub bucket_size: Option<u32>,
    pub exponent: i16,
    pub name: String,
    pub r#type: StatsType,
    pub unit: Option<StatsUnit>,
}

/// QMP enum `StatsTarget`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.1.0")]
pub enum StatsTarget {
    #[qapi(name = "cryptodev")]
    #[qapi(added = "8.0.0")]
    Cryptodev,
    #[qapi(name = "vcpu")]
    Vcpu,
    #[qapi(name = "vm")]
    Vm,
}

/// QMP enum `StatsType`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.1.0")]
pub enum StatsType {
    #[qapi(name = "cumulative")]
    Cumulative,
    #[qapi(name = "instant")]
    Instant,
    #[qapi(name = "linear-histogram")]
    LinearHistogram,
    #[qapi(name = "log2-histogram")]
    Log2Histogram,
    #[qapi(name = "peak")]
    Peak,
}

/// QMP enum `StatsUnit`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.1.0")]
pub enum StatsUnit {
    #[qapi(name = "boolean")]
    Boolean,
    #[qapi(name = "bytes")]
    Bytes,
    #[qapi(name = "cycles")]
    Cycles,
    #[qapi(name = "seconds")]
    Seconds,
}

/// QMP alternate `StatsValue`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.1.0")]
pub enum StatsValue {
    #[qapi(name = "boolean")]
    Boolean(bool),
    #[qapi(name = "list")]
    List(Vec<u64>),
    #[qapi(name = "scalar")]
    Scalar(u64),
}

/// QMP struct `StatsVCPUFilter`; see the QEMU QMP Reference Manual.
#[qapi(name = "StatsVCPUFilter")]
#[qapi(added = "7.1.0")]
pub struct StatsVcpuFilter {
    pub vcpus: Option<Vec<String>>,
}

/// QMP struct `StatusInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.0.0")]
pub struct StatusInfo {
    pub running: bool,
    #[qapi(deprecated(added = "8.1.0"))]
    #[qapi(removed = "9.0.0")]
    #[qapi(required)]
    pub singlestep: Option<bool>,
    pub status: RunState,
}

/// QMP struct `StringWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
#[qapi(removed = "9.0.0")]
pub struct StringWrapper {
    pub data: QapiString,
}

/// QMP alternate `StrOrNull`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.10.0")]
pub enum StrOrNull {
    #[qapi(name = "n")]
    N(()),
    #[qapi(name = "s")]
    S(String),
}

/// QMP enum `SysEmuTarget`; see the QEMU QMP Reference Manual.
#[qapi(added = "3.0.0")]
pub enum SysEmuTarget {
    #[qapi(name = "aarch64")]
    Aarch64,
    #[qapi(name = "alpha")]
    Alpha,
    #[qapi(name = "arm")]
    Arm,
    #[qapi(name = "avr")]
    #[qapi(added = "5.1.0")]
    Avr,
    #[qapi(name = "cris")]
    #[qapi(removed = "9.2.0")]
    Cris,
    #[qapi(name = "hexagon")]
    #[qapi(added = "11.0.0")]
    Hexagon,
    #[qapi(name = "hppa")]
    Hppa,
    #[qapi(name = "i386")]
    I386,
    #[qapi(name = "lm32")]
    #[qapi(removed = "6.1.0")]
    Lm32,
    #[qapi(name = "loongarch64")]
    #[qapi(added = "7.1.0")]
    Loongarch64,
    #[qapi(name = "m68k")]
    M68k,
    #[qapi(name = "microblaze")]
    Microblaze,
    #[qapi(name = "microblazeel")]
    #[qapi(removed = "11.0.0")]
    Microblazeel,
    #[qapi(name = "mips")]
    Mips,
    #[qapi(name = "mips64")]
    Mips64,
    #[qapi(name = "mips64el")]
    Mips64el,
    #[qapi(name = "mipsel")]
    Mipsel,
    #[qapi(name = "moxie")]
    #[qapi(removed = "6.1.0")]
    Moxie,
    #[qapi(name = "nios2")]
    #[qapi(removed = "9.1.0")]
    Nios2,
    #[qapi(name = "or1k")]
    Or1k,
    #[qapi(name = "ppc")]
    Ppc,
    #[qapi(name = "ppc64")]
    Ppc64,
    #[qapi(name = "ppcemb")]
    #[qapi(removed = "3.1.0")]
    Ppcemb,
    #[qapi(name = "riscv32")]
    Riscv32,
    #[qapi(name = "riscv64")]
    Riscv64,
    #[qapi(name = "rx")]
    #[qapi(added = "5.0.0")]
    Rx,
    #[qapi(name = "s390x")]
    S390x,
    #[qapi(name = "sh4")]
    Sh4,
    #[qapi(name = "sh4eb")]
    Sh4eb,
    #[qapi(name = "sparc")]
    Sparc,
    #[qapi(name = "sparc64")]
    Sparc64,
    #[qapi(name = "tricore")]
    Tricore,
    #[qapi(name = "unicore32")]
    #[qapi(removed = "6.1.0")]
    Unicore32,
    #[qapi(name = "x86_64")]
    X8664,
    #[qapi(name = "xtensa")]
    Xtensa,
    #[qapi(name = "xtensaeb")]
    Xtensaeb,
}

/// QMP struct `TargetInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.2.0")]
#[qapi(removed = "10.1.0")]
#[qapi(alias(added = "3.0.0", value = "QemuTargetInfo"))]
pub struct TargetInfo {
    #[qapi(added = "3.0.0")]
    #[qapi(required)]
    pub arch: Option<SysEmuTarget>,
    #[qapi(name = "arch")]
    #[qapi(removed = "1.6.0")]
    #[qapi(required)]
    pub arch_legacy: Option<TargetType>,
    #[qapi(name = "arch")]
    #[qapi(added = "1.6.0")]
    #[qapi(removed = "3.0.0")]
    #[qapi(required)]
    pub arch_legacy2: Option<String>,
}

/// QMP enum `TargetType`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.2.0")]
#[qapi(removed = "1.6.0")]
pub enum TargetType {
    #[qapi(name = "alpha")]
    Alpha,
    #[qapi(name = "arm")]
    Arm,
    #[qapi(name = "cris")]
    Cris,
    #[qapi(name = "i386")]
    I386,
    #[qapi(name = "lm32")]
    Lm32,
    #[qapi(name = "m68k")]
    M68k,
    #[qapi(name = "microblaze")]
    Microblaze,
    #[qapi(name = "microblazeel")]
    Microblazeel,
    #[qapi(name = "mips")]
    Mips,
    #[qapi(name = "mips64")]
    Mips64,
    #[qapi(name = "mips64el")]
    Mips64el,
    #[qapi(name = "mipsel")]
    Mipsel,
    #[qapi(name = "moxie")]
    #[qapi(added = "1.5.0")]
    Moxie,
    #[qapi(name = "or32")]
    Or32,
    #[qapi(name = "ppc")]
    Ppc,
    #[qapi(name = "ppc64")]
    Ppc64,
    #[qapi(name = "ppcemb")]
    Ppcemb,
    #[qapi(name = "s390x")]
    S390x,
    #[qapi(name = "sh4")]
    Sh4,
    #[qapi(name = "sh4eb")]
    Sh4eb,
    #[qapi(name = "sparc")]
    Sparc,
    #[qapi(name = "sparc64")]
    Sparc64,
    #[qapi(name = "unicore32")]
    Unicore32,
    #[qapi(name = "x86_64")]
    X8664,
    #[qapi(name = "xtensa")]
    Xtensa,
    #[qapi(name = "xtensaeb")]
    Xtensaeb,
}

/// QMP struct `TdxGuestProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "10.1.0")]
pub struct TdxGuestProperties {
    pub attributes: Option<u64>,
    pub mrconfigid: Option<String>,
    pub mrowner: Option<String>,
    pub mrownerconfig: Option<String>,
    #[qapi(name = "quote-generation-socket")]
    pub quote_generation_socket: Option<SocketAddress>,
    #[qapi(name = "sept-ve-disable")]
    pub sept_ve_disable: Option<bool>,
}

/// QMP struct `ThreadContextProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.2.0")]
pub struct ThreadContextProperties {
    #[qapi(name = "cpu-affinity")]
    pub cpu_affinity: Option<Vec<u16>>,
    #[qapi(name = "node-affinity")]
    pub node_affinity: Option<Vec<u16>>,
}

/// QMP struct `ThrottleGroupProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct ThrottleGroupProperties {
    pub limits: Option<ThrottleLimits>,
    #[qapi(name = "x-bps-read")]
    #[qapi(feature = "unstable")]
    pub x_bps_read: Option<i64>,
    #[qapi(name = "x-bps-read-max")]
    #[qapi(feature = "unstable")]
    pub x_bps_read_max: Option<i64>,
    #[qapi(name = "x-bps-read-max-length")]
    #[qapi(feature = "unstable")]
    pub x_bps_read_max_length: Option<i64>,
    #[qapi(name = "x-bps-total")]
    #[qapi(feature = "unstable")]
    pub x_bps_total: Option<i64>,
    #[qapi(name = "x-bps-total-max")]
    #[qapi(feature = "unstable")]
    pub x_bps_total_max: Option<i64>,
    #[qapi(name = "x-bps-total-max-length")]
    #[qapi(feature = "unstable")]
    pub x_bps_total_max_length: Option<i64>,
    #[qapi(name = "x-bps-write")]
    #[qapi(feature = "unstable")]
    pub x_bps_write: Option<i64>,
    #[qapi(name = "x-bps-write-max")]
    #[qapi(feature = "unstable")]
    pub x_bps_write_max: Option<i64>,
    #[qapi(name = "x-bps-write-max-length")]
    #[qapi(feature = "unstable")]
    pub x_bps_write_max_length: Option<i64>,
    #[qapi(name = "x-iops-read")]
    #[qapi(feature = "unstable")]
    pub x_iops_read: Option<i64>,
    #[qapi(name = "x-iops-read-max")]
    #[qapi(feature = "unstable")]
    pub x_iops_read_max: Option<i64>,
    #[qapi(name = "x-iops-read-max-length")]
    #[qapi(feature = "unstable")]
    pub x_iops_read_max_length: Option<i64>,
    #[qapi(name = "x-iops-size")]
    #[qapi(feature = "unstable")]
    pub x_iops_size: Option<i64>,
    #[qapi(name = "x-iops-total")]
    #[qapi(feature = "unstable")]
    pub x_iops_total: Option<i64>,
    #[qapi(name = "x-iops-total-max")]
    #[qapi(feature = "unstable")]
    pub x_iops_total_max: Option<i64>,
    #[qapi(name = "x-iops-total-max-length")]
    #[qapi(feature = "unstable")]
    pub x_iops_total_max_length: Option<i64>,
    #[qapi(name = "x-iops-write")]
    #[qapi(feature = "unstable")]
    pub x_iops_write: Option<i64>,
    #[qapi(name = "x-iops-write-max")]
    #[qapi(feature = "unstable")]
    pub x_iops_write_max: Option<i64>,
    #[qapi(name = "x-iops-write-max-length")]
    #[qapi(feature = "unstable")]
    pub x_iops_write_max_length: Option<i64>,
}

/// QMP struct `ThrottleLimits`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.11.0")]
pub struct ThrottleLimits {
    #[qapi(name = "bps-read")]
    pub bps_read: Option<i64>,
    #[qapi(name = "bps-read-max")]
    pub bps_read_max: Option<i64>,
    #[qapi(name = "bps-read-max-length")]
    pub bps_read_max_length: Option<i64>,
    #[qapi(name = "bps-total")]
    pub bps_total: Option<i64>,
    #[qapi(name = "bps-total-max")]
    pub bps_total_max: Option<i64>,
    #[qapi(name = "bps-total-max-length")]
    pub bps_total_max_length: Option<i64>,
    #[qapi(name = "bps-write")]
    pub bps_write: Option<i64>,
    #[qapi(name = "bps-write-max")]
    pub bps_write_max: Option<i64>,
    #[qapi(name = "bps-write-max-length")]
    pub bps_write_max_length: Option<i64>,
    #[qapi(name = "iops-read")]
    pub iops_read: Option<i64>,
    #[qapi(name = "iops-read-max")]
    pub iops_read_max: Option<i64>,
    #[qapi(name = "iops-read-max-length")]
    pub iops_read_max_length: Option<i64>,
    #[qapi(name = "iops-size")]
    pub iops_size: Option<i64>,
    #[qapi(name = "iops-total")]
    pub iops_total: Option<i64>,
    #[qapi(name = "iops-total-max")]
    pub iops_total_max: Option<i64>,
    #[qapi(name = "iops-total-max-length")]
    pub iops_total_max_length: Option<i64>,
    #[qapi(name = "iops-write")]
    pub iops_write: Option<i64>,
    #[qapi(name = "iops-write-max")]
    pub iops_write_max: Option<i64>,
    #[qapi(name = "iops-write-max-length")]
    pub iops_write_max_length: Option<i64>,
}

/// QMP enum `TimeUnit`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.2.0")]
pub enum TimeUnit {
    #[qapi(name = "millisecond")]
    Millisecond,
    #[qapi(name = "second")]
    Second,
}

/// QMP struct `TlsCredsAnonProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct TlsCredsAnonProperties {
    pub dir: Option<String>,
    pub endpoint: Option<QCryptoTlsCredsEndpoint>,
    #[qapi(deprecated(added = "6.0.0"))]
    #[qapi(removed = "9.2.0")]
    pub loaded: Option<bool>,
    pub priority: Option<String>,
    #[qapi(name = "verify-peer")]
    pub verify_peer: Option<bool>,
}

/// QMP struct `TlsCredsProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct TlsCredsProperties {
    pub dir: Option<String>,
    pub endpoint: Option<QCryptoTlsCredsEndpoint>,
    pub priority: Option<String>,
    #[qapi(name = "verify-peer")]
    pub verify_peer: Option<bool>,
}

/// QMP struct `TlsCredsPskProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct TlsCredsPskProperties {
    pub dir: Option<String>,
    pub endpoint: Option<QCryptoTlsCredsEndpoint>,
    #[qapi(deprecated(added = "6.0.0"))]
    #[qapi(removed = "9.2.0")]
    pub loaded: Option<bool>,
    pub priority: Option<String>,
    pub username: Option<String>,
    #[qapi(name = "verify-peer")]
    pub verify_peer: Option<bool>,
}

/// QMP struct `TlsCredsX509Properties`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct TlsCredsX509Properties {
    pub dir: Option<String>,
    pub endpoint: Option<QCryptoTlsCredsEndpoint>,
    #[qapi(deprecated(added = "6.0.0"))]
    #[qapi(removed = "9.2.0")]
    pub loaded: Option<bool>,
    pub passwordid: Option<String>,
    pub priority: Option<String>,
    #[qapi(name = "sanity-check")]
    pub sanity_check: Option<bool>,
    #[qapi(name = "verify-peer")]
    pub verify_peer: Option<bool>,
}

/// QMP struct `TPMEmulatorOptions`; see the QEMU QMP Reference Manual.
#[qapi(name = "TPMEmulatorOptions")]
#[qapi(condition = "CONFIG_TPM")]
#[qapi(added = "2.11.0")]
pub struct TpmEmulatorOptions {
    pub chardev: String,
}

/// QMP struct `TPMEmulatorOptionsWrapper`; see the QEMU QMP Reference Manual.
#[qapi(name = "TPMEmulatorOptionsWrapper")]
#[qapi(condition = "CONFIG_TPM")]
#[qapi(added = "6.2.0")]
pub struct TpmEmulatorOptionsWrapper {
    pub data: TpmEmulatorOptions,
}

/// QMP struct `TPMInfo`; see the QEMU QMP Reference Manual.
#[qapi(name = "TPMInfo")]
#[qapi(condition = "CONFIG_TPM")]
#[qapi(added = "1.5.0")]
pub struct TpmInfo {
    pub id: String,
    pub model: TpmModel,
    pub options: TpmTypeOptions,
}

/// QMP enum `TpmModel`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_TPM")]
#[qapi(added = "1.5.0")]
pub enum TpmModel {
    #[qapi(name = "tpm-crb")]
    #[qapi(added = "2.12.0")]
    TpmCrb,
    #[qapi(name = "tpm-spapr")]
    #[qapi(added = "5.0.0")]
    TpmSpapr,
    #[qapi(name = "tpm-tis")]
    TpmTis,
}

/// QMP struct `TPMPassthroughOptions`; see the QEMU QMP Reference Manual.
#[qapi(name = "TPMPassthroughOptions")]
#[qapi(condition = "CONFIG_TPM")]
#[qapi(added = "1.5.0")]
pub struct TpmPassthroughOptions {
    #[qapi(name = "cancel-path")]
    pub cancel_path: Option<String>,
    pub path: Option<String>,
}

/// QMP struct `TPMPassthroughOptionsWrapper`; see the QEMU QMP Reference Manual.
#[qapi(name = "TPMPassthroughOptionsWrapper")]
#[qapi(condition = "CONFIG_TPM")]
#[qapi(added = "6.2.0")]
pub struct TpmPassthroughOptionsWrapper {
    pub data: TpmPassthroughOptions,
}

/// QMP enum `TpmType`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_TPM")]
#[qapi(added = "1.5.0")]
pub enum TpmType {
    #[qapi(name = "emulator")]
    #[qapi(added = "2.11.0")]
    Emulator,
    #[qapi(name = "passthrough")]
    Passthrough,
}

/// QMP union `TpmTypeOptions`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_TPM")]
#[qapi(added = "1.5.0")]
#[qapi(simple_union(removed = "6.2.0"))]
pub struct TpmTypeOptions {
    #[qapi(discriminator)]
    #[qapi(name = "type")]
    #[qapi(removed = "6.2.0")]
    #[qapi(required)]
    pub r_type: Option<TpmTypeOptionsKind>,
    #[qapi(discriminator)]
    #[qapi(added = "6.2.0")]
    #[qapi(required)]
    pub r#type: Option<TpmType>,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<TpmTypeOptionsBranch>,
}

#[qapi(union_branch)]
#[qapi(condition = "CONFIG_TPM")]
#[qapi(added = "1.5.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum TpmTypeOptionsBranch {
    #[qapi(name = "emulator")]
    #[qapi(added = "6.2.0")]
    Emulator(TpmEmulatorOptionsWrapper),
    #[qapi(name = "emulator")]
    #[qapi(added = "2.11.0")]
    #[qapi(removed = "6.2.0")]
    EmulatorLegacy(TpmEmulatorOptions),
    #[qapi(name = "passthrough")]
    #[qapi(added = "6.2.0")]
    Passthrough(TpmPassthroughOptionsWrapper),
    #[qapi(name = "passthrough")]
    #[qapi(removed = "6.2.0")]
    PassthroughLegacy(TpmPassthroughOptions),
}

#[qapi(condition = "defined(CONFIG_TPM)")]
#[qapi(added = "1.5.0")]
#[qapi(removed = "6.2.0")]
pub enum TpmTypeOptionsKind {
    #[qapi(name = "emulator")]
    #[qapi(added = "2.11.0")]
    Emulator,
    #[qapi(name = "passthrough")]
    Passthrough,
}

/// QMP struct `TraceEventInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.2.0")]
pub struct TraceEventInfo {
    pub name: String,
    pub state: TraceEventState,
    #[qapi(added = "2.7.0")]
    #[qapi(deprecated(added = "8.1.0"))]
    #[qapi(removed = "9.1.0")]
    #[qapi(required)]
    pub vcpu: Option<bool>,
}

/// QMP enum `TraceEventState`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.2.0")]
pub enum TraceEventState {
    #[qapi(name = "disabled")]
    Disabled,
    #[qapi(name = "enabled")]
    Enabled,
    #[qapi(name = "unavailable")]
    Unavailable,
}

/// QMP union `TransactionAction`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.6.0")]
#[qapi(simple_union(removed = "6.2.0"))]
pub struct TransactionAction {
    #[qapi(discriminator)]
    #[qapi(name = "type")]
    #[qapi(removed = "6.2.0")]
    #[qapi(required)]
    pub r_type: Option<TransactionActionKind>,
    #[qapi(discriminator)]
    #[qapi(added = "6.2.0")]
    #[qapi(required)]
    pub r#type: Option<TransactionActionKind>,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<TransactionActionBranch>,
}

#[qapi(union_branch)]
#[qapi(added = "1.6.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum TransactionActionBranch {
    #[qapi(name = "abort")]
    #[qapi(added = "6.2.0")]
    Abort(AbortWrapper),
    #[qapi(name = "abort")]
    #[qapi(removed = "6.2.0")]
    AbortLegacy(Abort),
    #[qapi(name = "block-dirty-bitmap-add")]
    #[qapi(added = "6.2.0")]
    BlockDirtyBitmapAdd(BlockDirtyBitmapAddWrapper),
    #[qapi(name = "block-dirty-bitmap-clear")]
    #[qapi(added = "6.2.0")]
    BlockDirtyBitmapClear(BlockDirtyBitmapWrapper),
    #[qapi(name = "block-dirty-bitmap-disable")]
    #[qapi(added = "6.2.0")]
    BlockDirtyBitmapDisable(BlockDirtyBitmapWrapper),
    #[qapi(name = "block-dirty-bitmap-enable")]
    #[qapi(added = "6.2.0")]
    BlockDirtyBitmapEnable(BlockDirtyBitmapWrapper),
    #[qapi(name = "block-dirty-bitmap-merge")]
    #[qapi(added = "6.2.0")]
    BlockDirtyBitmapMerge(BlockDirtyBitmapMergeWrapper),
    #[qapi(name = "block-dirty-bitmap-remove")]
    #[qapi(added = "6.2.0")]
    BlockDirtyBitmapRemove(BlockDirtyBitmapWrapper),
    #[qapi(name = "blockdev-backup")]
    #[qapi(added = "6.2.0")]
    BlockdevBackup(BlockdevBackupWrapper),
    #[qapi(name = "blockdev-snapshot")]
    #[qapi(added = "6.2.0")]
    BlockdevSnapshot(BlockdevSnapshotWrapper),
    #[qapi(name = "blockdev-snapshot-internal-sync")]
    #[qapi(added = "6.2.0")]
    BlockdevSnapshotInternalSync(BlockdevSnapshotInternalWrapper),
    #[qapi(name = "blockdev-snapshot-sync")]
    #[qapi(added = "6.2.0")]
    BlockdevSnapshotSync(BlockdevSnapshotSyncWrapper),
    #[qapi(name = "blockdev-backup")]
    #[qapi(added = "2.3.0")]
    #[qapi(removed = "6.2.0")]
    BlockdevBackupLegacy(BlockdevBackup),
    #[qapi(name = "blockdev-snapshot-internal-sync")]
    #[qapi(added = "1.7.0")]
    #[qapi(removed = "6.2.0")]
    BlockdevSnapshotInternalSyncLegacy(BlockdevSnapshotInternal),
    #[qapi(name = "blockdev-snapshot")]
    #[qapi(added = "2.5.0")]
    #[qapi(removed = "6.2.0")]
    BlockdevSnapshotLegacy(BlockdevSnapshot),
    #[qapi(name = "blockdev-snapshot-sync")]
    #[qapi(removed = "2.5.0")]
    BlockdevSnapshotSyncLegacy(BlockdevSnapshot),
    #[qapi(name = "blockdev-snapshot-sync")]
    #[qapi(added = "2.5.0")]
    #[qapi(removed = "6.2.0")]
    BlockdevSnapshotSyncLegacy2(BlockdevSnapshotSync),
    #[qapi(name = "block-dirty-bitmap-add")]
    #[qapi(added = "2.5.0")]
    #[qapi(removed = "6.2.0")]
    BlockDirtyBitmapAddLegacy(BlockDirtyBitmapAdd),
    #[qapi(name = "block-dirty-bitmap-clear")]
    #[qapi(added = "2.5.0")]
    #[qapi(removed = "6.2.0")]
    BlockDirtyBitmapClearLegacy(BlockDirtyBitmap),
    #[qapi(name = "block-dirty-bitmap-disable")]
    #[qapi(added = "4.0.0")]
    #[qapi(removed = "6.2.0")]
    BlockDirtyBitmapDisableLegacy(BlockDirtyBitmap),
    #[qapi(name = "block-dirty-bitmap-enable")]
    #[qapi(added = "4.0.0")]
    #[qapi(removed = "6.2.0")]
    BlockDirtyBitmapEnableLegacy(BlockDirtyBitmap),
    #[qapi(name = "block-dirty-bitmap-merge")]
    #[qapi(added = "4.0.0")]
    #[qapi(removed = "6.2.0")]
    BlockDirtyBitmapMergeLegacy(BlockDirtyBitmapMerge),
    #[qapi(name = "block-dirty-bitmap-remove")]
    #[qapi(added = "4.2.0")]
    #[qapi(removed = "6.2.0")]
    BlockDirtyBitmapRemoveLegacy(BlockDirtyBitmap),
    #[qapi(name = "drive-backup")]
    #[qapi(added = "6.2.0")]
    DriveBackup(DriveBackupWrapper),
    #[qapi(name = "drive-backup")]
    #[qapi(removed = "6.2.0")]
    DriveBackupLegacy(DriveBackup),
    #[qapi(name = "x-block-dirty-bitmap-disable")]
    #[qapi(added = "3.0.0")]
    #[qapi(removed = "4.0.0")]
    XBlockDirtyBitmapDisable(BlockDirtyBitmap),
    #[qapi(name = "x-block-dirty-bitmap-enable")]
    #[qapi(added = "3.0.0")]
    #[qapi(removed = "4.0.0")]
    XBlockDirtyBitmapEnable(BlockDirtyBitmap),
    #[qapi(name = "x-block-dirty-bitmap-merge")]
    #[qapi(added = "3.1.0")]
    #[qapi(removed = "4.0.0")]
    XBlockDirtyBitmapMerge(BlockDirtyBitmapMerge),
}

/// QMP enum `TransactionActionKind`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.6.0")]
pub enum TransactionActionKind {
    #[qapi(name = "abort")]
    Abort,
    #[qapi(name = "block-dirty-bitmap-add")]
    #[qapi(added = "2.5.0")]
    BlockDirtyBitmapAdd,
    #[qapi(name = "block-dirty-bitmap-clear")]
    #[qapi(added = "2.5.0")]
    BlockDirtyBitmapClear,
    #[qapi(name = "block-dirty-bitmap-disable")]
    #[qapi(added = "4.0.0")]
    BlockDirtyBitmapDisable,
    #[qapi(name = "block-dirty-bitmap-enable")]
    #[qapi(added = "4.0.0")]
    BlockDirtyBitmapEnable,
    #[qapi(name = "block-dirty-bitmap-merge")]
    #[qapi(added = "4.0.0")]
    BlockDirtyBitmapMerge,
    #[qapi(name = "block-dirty-bitmap-remove")]
    #[qapi(added = "4.2.0")]
    BlockDirtyBitmapRemove,
    #[qapi(name = "blockdev-backup")]
    #[qapi(added = "2.3.0")]
    BlockdevBackup,
    #[qapi(name = "blockdev-snapshot")]
    #[qapi(added = "2.5.0")]
    BlockdevSnapshot,
    #[qapi(name = "blockdev-snapshot-internal-sync")]
    #[qapi(added = "1.7.0")]
    BlockdevSnapshotInternalSync,
    #[qapi(name = "blockdev-snapshot-sync")]
    BlockdevSnapshotSync,
    #[qapi(name = "drive-backup")]
    #[qapi(deprecated(added = "6.2.0"))]
    DriveBackup,
    #[qapi(name = "x-block-dirty-bitmap-disable")]
    #[qapi(added = "3.0.0")]
    #[qapi(removed = "4.0.0")]
    XBlockDirtyBitmapDisable,
    #[qapi(name = "x-block-dirty-bitmap-enable")]
    #[qapi(added = "3.0.0")]
    #[qapi(removed = "4.0.0")]
    XBlockDirtyBitmapEnable,
    #[qapi(name = "x-block-dirty-bitmap-merge")]
    #[qapi(added = "3.1.0")]
    #[qapi(removed = "4.0.0")]
    XBlockDirtyBitmapMerge,
}

/// QMP struct `TransactionProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.5.0")]
pub struct TransactionProperties {
    #[qapi(name = "completion-mode")]
    pub completion_mode: Option<ActionCompletionMode>,
}

/// QMP struct `UefiVariable`; see the QEMU QMP Reference Manual.
#[qapi(added = "10.0.0")]
pub struct UefiVariable {
    pub attr: i64,
    pub data: String,
    pub digest: Option<String>,
    pub guid: String,
    pub name: String,
    pub time: Option<String>,
}

/// QMP struct `UefiVarStore`; see the QEMU QMP Reference Manual.
#[qapi(added = "10.0.0")]
pub struct UefiVarStore {
    pub variables: Vec<UefiVariable>,
    pub version: i64,
}

/// QMP struct `UnixSocketAddress`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.3.0")]
pub struct UnixSocketAddress {
    #[qapi(condition = "CONFIG_LINUX")]
    #[qapi(added = "5.1.0")]
    pub r#abstract: Option<bool>,
    pub path: String,
    #[qapi(condition = "CONFIG_LINUX")]
    #[qapi(added = "5.1.0")]
    pub tight: Option<bool>,
}

/// QMP struct `UnixSocketAddressWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct UnixSocketAddressWrapper {
    pub data: UnixSocketAddress,
}

/// QMP struct `UuidInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.0.0")]
pub struct UuidInfo {
    #[qapi(name = "UUID")]
    pub uuid: String,
}

/// QMP struct `VersionInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "1.0.0")]
pub struct VersionInfo {
    pub package: String,
    #[qapi(added = "2.4.0")]
    #[qapi(required)]
    pub qemu: Option<VersionTriple>,
    #[qapi(name = "qemu")]
    #[qapi(removed = "2.4.0")]
    #[qapi(required)]
    pub qemu_legacy: Option<VersionInfoQemu>,
}

#[qapi(added = "1.0.0")]
#[qapi(removed = "2.4.0")]
pub struct VersionInfoQemu {
    pub major: i64,
    pub micro: i64,
    pub minor: i64,
}

/// QMP struct `VersionTriple`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.4.0")]
pub struct VersionTriple {
    pub major: i64,
    pub micro: i64,
    pub minor: i64,
}

/// QMP enum `VfioMigrationState`; see the QEMU QMP Reference Manual.
#[qapi(added = "9.1.0")]
#[qapi(removed = "9.2.0")]
pub enum VfioMigrationState {
    #[qapi(name = "pre-copy")]
    PreCopy,
    #[qapi(name = "pre-copy-p2p")]
    PreCopyP2p,
    #[qapi(name = "resuming")]
    Resuming,
    #[qapi(name = "running")]
    Running,
    #[qapi(name = "running-p2p")]
    RunningP2p,
    #[qapi(name = "stop")]
    Stop,
    #[qapi(name = "stop-copy")]
    StopCopy,
}

/// QMP struct `VfioStats`; see the QEMU QMP Reference Manual.
#[qapi(added = "5.2.0")]
pub struct VfioStats {
    pub transferred: i64,
}

/// QMP struct `VfioUserServerProperties`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.1.0")]
pub struct VfioUserServerProperties {
    pub device: String,
    pub socket: SocketAddress,
}

/// QMP struct `VhostDeviceProtocols`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.2.0")]
pub struct VhostDeviceProtocols {
    pub protocols: Vec<String>,
    #[qapi(name = "unknown-protocols")]
    pub unknown_protocols: Option<u64>,
}

/// QMP struct `VhostStatus`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.2.0")]
pub struct VhostStatus {
    #[qapi(name = "acked-features")]
    pub acked_features: VirtioDeviceFeatures,
    #[qapi(name = "backend-cap")]
    pub backend_cap: u64,
    #[qapi(name = "backend-features")]
    #[qapi(removed = "11.1.0")]
    #[qapi(required)]
    pub backend_features: Option<VirtioDeviceFeatures>,
    pub features: VirtioDeviceFeatures,
    #[qapi(name = "log-enabled")]
    pub log_enabled: bool,
    #[qapi(name = "log-size")]
    pub log_size: u64,
    #[qapi(name = "max-queues")]
    pub max_queues: u64,
    #[qapi(name = "n-mem-sections")]
    pub n_mem_sections: i64,
    #[qapi(name = "n-tmp-sections")]
    pub n_tmp_sections: i64,
    pub nvqs: u32,
    #[qapi(name = "protocol-features")]
    pub protocol_features: VhostDeviceProtocols,
    #[qapi(name = "vq-index")]
    pub vq_index: i64,
}

/// QMP struct `VirtioDeviceFeatures`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.2.0")]
pub struct VirtioDeviceFeatures {
    #[qapi(name = "dev-features")]
    pub dev_features: Option<Vec<String>>,
    pub transports: Vec<String>,
    #[qapi(name = "unknown-dev-features")]
    pub unknown_dev_features: Option<u64>,
    #[qapi(name = "unknown-dev-features2")]
    #[qapi(added = "10.2.0")]
    pub unknown_dev_features2: Option<u64>,
}

/// QMP struct `VirtioDeviceStatus`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.2.0")]
pub struct VirtioDeviceStatus {
    pub statuses: Vec<String>,
    #[qapi(name = "unknown-statuses")]
    pub unknown_statuses: Option<u8>,
}

/// QMP struct `VirtIOGPUOutput`; see the QEMU QMP Reference Manual.
#[qapi(name = "VirtIOGPUOutput")]
#[qapi(added = "10.1.0")]
pub struct VirtIogpuOutput {
    pub name: String,
    #[qapi(added = "11.0.0")]
    pub xres: Option<u16>,
    #[qapi(added = "11.0.0")]
    pub yres: Option<u16>,
}

/// QMP struct `VirtioInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.2.0")]
pub struct VirtioInfo {
    pub name: String,
    pub path: String,
}

/// QMP struct `VirtioMEMDeviceInfo`; see the QEMU QMP Reference Manual.
#[qapi(name = "VirtioMEMDeviceInfo")]
#[qapi(added = "5.1.0")]
pub struct VirtioMemDeviceInfo {
    #[qapi(name = "block-size")]
    pub block_size: u64,
    pub id: Option<String>,
    #[qapi(name = "max-size")]
    pub max_size: u64,
    pub memaddr: u64,
    pub memdev: String,
    pub node: i64,
    #[qapi(name = "requested-size")]
    pub requested_size: u64,
    pub size: u64,
}

/// QMP struct `VirtioMEMDeviceInfoWrapper`; see the QEMU QMP Reference Manual.
#[qapi(name = "VirtioMEMDeviceInfoWrapper")]
#[qapi(added = "6.2.0")]
pub struct VirtioMemDeviceInfoWrapper {
    pub data: VirtioMemDeviceInfo,
}

/// QMP struct `VirtioPMEMDeviceInfo`; see the QEMU QMP Reference Manual.
#[qapi(name = "VirtioPMEMDeviceInfo")]
#[qapi(added = "4.1.0")]
pub struct VirtioPmemDeviceInfo {
    pub id: Option<String>,
    pub memaddr: u64,
    pub memdev: String,
    pub size: u64,
}

/// QMP struct `VirtioPMEMDeviceInfoWrapper`; see the QEMU QMP Reference Manual.
#[qapi(name = "VirtioPMEMDeviceInfoWrapper")]
#[qapi(added = "6.2.0")]
pub struct VirtioPmemDeviceInfoWrapper {
    pub data: VirtioPmemDeviceInfo,
}

/// QMP struct `VirtioQueueElement`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.2.0")]
pub struct VirtioQueueElement {
    pub avail: VirtioRingAvail,
    pub descs: Vec<VirtioRingDesc>,
    pub index: u32,
    pub name: String,
    pub used: VirtioRingUsed,
}

/// QMP struct `VirtioRingAvail`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.2.0")]
pub struct VirtioRingAvail {
    pub flags: u16,
    pub idx: u16,
    pub ring: u16,
}

/// QMP struct `VirtioRingDesc`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.2.0")]
pub struct VirtioRingDesc {
    pub addr: u64,
    pub flags: Vec<String>,
    pub len: u32,
}

/// QMP struct `VirtioRingUsed`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.2.0")]
pub struct VirtioRingUsed {
    pub flags: u16,
    pub idx: u16,
}

/// QMP struct `VirtioStatus`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.2.0")]
pub struct VirtioStatus {
    #[qapi(name = "backend-features")]
    pub backend_features: VirtioDeviceFeatures,
    pub broken: bool,
    #[qapi(name = "bus-name")]
    pub bus_name: String,
    #[qapi(name = "device-endian")]
    pub device_endian: String,
    #[qapi(name = "device-id")]
    pub device_id: u16,
    #[qapi(name = "disable-legacy-check")]
    pub disable_legacy_check: bool,
    pub disabled: bool,
    #[qapi(name = "guest-features")]
    pub guest_features: VirtioDeviceFeatures,
    #[qapi(name = "host-features")]
    pub host_features: VirtioDeviceFeatures,
    pub isr: u8,
    pub name: String,
    #[qapi(name = "num-vqs")]
    pub num_vqs: i64,
    #[qapi(name = "queue-sel")]
    pub queue_sel: u16,
    #[qapi(name = "start-on-kick")]
    pub start_on_kick: bool,
    pub started: bool,
    pub status: VirtioDeviceStatus,
    #[qapi(name = "use-guest-notifier-mask")]
    pub use_guest_notifier_mask: bool,
    #[qapi(name = "use-started")]
    pub use_started: bool,
    #[qapi(name = "vhost-dev")]
    pub vhost_dev: Option<VhostStatus>,
    #[qapi(name = "vhost-started")]
    pub vhost_started: bool,
    #[qapi(name = "vm-running")]
    pub vm_running: bool,
}

/// QMP struct `VirtQueueStatus`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.2.0")]
pub struct VirtQueueStatus {
    pub inuse: u32,
    #[qapi(name = "last-avail-idx")]
    pub last_avail_idx: Option<u16>,
    pub name: String,
    #[qapi(name = "queue-index")]
    pub queue_index: u16,
    #[qapi(name = "shadow-avail-idx")]
    pub shadow_avail_idx: Option<u16>,
    #[qapi(name = "signalled-used")]
    pub signalled_used: u16,
    #[qapi(name = "signalled-used-valid")]
    pub signalled_used_valid: bool,
    #[qapi(name = "used-idx")]
    pub used_idx: u16,
    #[qapi(name = "vring-align")]
    pub vring_align: u32,
    #[qapi(name = "vring-avail")]
    pub vring_avail: u64,
    #[qapi(name = "vring-desc")]
    pub vring_desc: u64,
    #[qapi(name = "vring-num")]
    pub vring_num: u32,
    #[qapi(name = "vring-num-default")]
    pub vring_num_default: u32,
    #[qapi(name = "vring-used")]
    pub vring_used: u64,
}

/// QMP struct `VirtVhostQueueStatus`; see the QEMU QMP Reference Manual.
#[qapi(added = "7.2.0")]
pub struct VirtVhostQueueStatus {
    #[qapi(removed = "11.1.0")]
    #[qapi(required)]
    pub avail: Option<u64>,
    #[qapi(name = "avail-phys")]
    pub avail_phys: u64,
    #[qapi(name = "avail-size")]
    pub avail_size: u32,
    pub call: i64,
    #[qapi(removed = "11.1.0")]
    #[qapi(required)]
    pub desc: Option<u64>,
    #[qapi(name = "desc-phys")]
    pub desc_phys: u64,
    #[qapi(name = "desc-size")]
    pub desc_size: u32,
    pub kick: i64,
    pub name: String,
    pub num: i64,
    #[qapi(removed = "11.1.0")]
    #[qapi(required)]
    pub used: Option<u64>,
    #[qapi(name = "used-phys")]
    pub used_phys: u64,
    #[qapi(name = "used-size")]
    pub used_size: u32,
}

/// QMP enum `VMAppleVirtioBlkVariant`; see the QEMU QMP Reference Manual.
#[qapi(name = "VMAppleVirtioBlkVariant")]
#[qapi(added = "10.0.0")]
pub enum VmAppleVirtioBlkVariant {
    #[qapi(name = "aux")]
    Aux,
    #[qapi(name = "root")]
    Root,
    #[qapi(name = "unspecified")]
    Unspecified,
}

/// QMP struct `VmdkExtentInfo`; see the QEMU QMP Reference Manual.
#[qapi(added = "8.0.0")]
pub struct VmdkExtentInfo {
    #[qapi(name = "cluster-size")]
    pub cluster_size: Option<i64>,
    pub compressed: Option<bool>,
    pub filename: String,
    pub format: String,
    #[qapi(name = "virtual-size")]
    pub virtual_size: i64,
}

/// QMP struct `VncBasicInfo`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_VNC")]
#[qapi(added = "2.1.0")]
pub struct VncBasicInfo {
    pub family: NetworkAddressFamily,
    pub host: String,
    pub service: String,
    #[qapi(added = "2.3.0")]
    #[qapi(required)]
    pub websocket: Option<bool>,
}

/// QMP struct `VncClientInfo`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_VNC")]
#[qapi(added = "1.0.0")]
pub struct VncClientInfo {
    #[qapi(added = "2.1.0")]
    #[qapi(required)]
    pub family: Option<NetworkAddressFamily>,
    #[qapi(name = "family")]
    #[qapi(removed = "2.1.0")]
    #[qapi(required)]
    pub family_legacy: Option<String>,
    pub host: String,
    pub sasl_username: Option<String>,
    pub service: String,
    #[qapi(added = "2.3.0")]
    #[qapi(required)]
    pub websocket: Option<bool>,
    pub x509_dname: Option<String>,
}

/// QMP struct `VncInfo`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_VNC")]
#[qapi(added = "1.0.0")]
pub struct VncInfo {
    pub auth: Option<String>,
    pub clients: Option<Vec<VncClientInfo>>,
    pub enabled: bool,
    #[qapi(added = "2.1.0")]
    pub family: Option<NetworkAddressFamily>,
    #[qapi(name = "family")]
    #[qapi(removed = "2.1.0")]
    pub family_legacy: Option<String>,
    pub host: Option<String>,
    pub service: Option<String>,
}

/// QMP struct `VncInfo2`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_VNC")]
#[qapi(added = "2.3.0")]
pub struct VncInfo2 {
    pub auth: VncPrimaryAuth,
    pub clients: Vec<VncClientInfo>,
    pub display: Option<String>,
    pub id: String,
    #[qapi(added = "2.9.0")]
    #[qapi(required)]
    pub server: Option<Vec<VncServerInfo2>>,
    #[qapi(name = "server")]
    #[qapi(removed = "2.9.0")]
    #[qapi(required)]
    pub server_legacy: Option<Vec<VncBasicInfo>>,
    pub vencrypt: Option<VncVencryptSubAuth>,
}

/// QMP enum `VncPrimaryAuth`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_VNC")]
#[qapi(added = "2.3.0")]
pub enum VncPrimaryAuth {
    #[qapi(name = "none")]
    None,
    #[qapi(name = "ra2")]
    Ra2,
    #[qapi(name = "ra2ne")]
    Ra2ne,
    #[qapi(name = "sasl")]
    Sasl,
    #[qapi(name = "tight")]
    Tight,
    #[qapi(name = "tls")]
    Tls,
    #[qapi(name = "ultra")]
    Ultra,
    #[qapi(name = "vencrypt")]
    Vencrypt,
    #[qapi(name = "vnc")]
    Vnc,
}

/// QMP struct `VncServerInfo`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_VNC")]
#[qapi(added = "2.1.0")]
pub struct VncServerInfo {
    pub auth: Option<String>,
    pub family: NetworkAddressFamily,
    pub host: String,
    pub service: String,
    #[qapi(added = "2.3.0")]
    #[qapi(required)]
    pub websocket: Option<bool>,
}

/// QMP struct `VncServerInfo2`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_VNC")]
#[qapi(added = "2.9.0")]
pub struct VncServerInfo2 {
    pub auth: VncPrimaryAuth,
    pub family: NetworkAddressFamily,
    pub host: String,
    pub service: String,
    pub vencrypt: Option<VncVencryptSubAuth>,
    pub websocket: bool,
}

/// QMP enum `VncVencryptSubAuth`; see the QEMU QMP Reference Manual.
#[qapi(condition = "CONFIG_VNC")]
#[qapi(added = "2.3.0")]
pub enum VncVencryptSubAuth {
    #[qapi(name = "plain")]
    Plain,
    #[qapi(name = "tls-none")]
    TlsNone,
    #[qapi(name = "tls-plain")]
    TlsPlain,
    #[qapi(name = "tls-sasl")]
    TlsSasl,
    #[qapi(name = "tls-vnc")]
    TlsVnc,
    #[qapi(name = "x509-none")]
    X509None,
    #[qapi(name = "x509-plain")]
    X509Plain,
    #[qapi(name = "x509-sasl")]
    X509Sasl,
    #[qapi(name = "x509-vnc")]
    X509Vnc,
}

/// QMP struct `VsockSocketAddress`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.8.0")]
pub struct VsockSocketAddress {
    pub cid: String,
    pub port: String,
}

/// QMP struct `VsockSocketAddressWrapper`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.2.0")]
pub struct VsockSocketAddressWrapper {
    pub data: VsockSocketAddress,
}

/// QMP enum `WatchdogAction`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.11.0")]
pub enum WatchdogAction {
    #[qapi(name = "debug")]
    Debug,
    #[qapi(name = "inject-nmi")]
    InjectNmi,
    #[qapi(name = "none")]
    None,
    #[qapi(name = "pause")]
    Pause,
    #[qapi(name = "poweroff")]
    Poweroff,
    #[qapi(name = "reset")]
    Reset,
    #[qapi(name = "shutdown")]
    Shutdown,
}

/// QMP enum `WatchdogExpirationAction`; see the QEMU QMP Reference Manual.
#[qapi(added = "2.1.0")]
#[qapi(removed = "2.11.0")]
pub enum WatchdogExpirationAction {
    #[qapi(name = "debug")]
    Debug,
    #[qapi(name = "inject-nmi")]
    #[qapi(added = "2.4.0")]
    InjectNmi,
    #[qapi(name = "none")]
    None,
    #[qapi(name = "pause")]
    Pause,
    #[qapi(name = "poweroff")]
    Poweroff,
    #[qapi(name = "reset")]
    Reset,
    #[qapi(name = "shutdown")]
    Shutdown,
}

/// QMP struct `X86CPUFeatureWordInfo`; see the QEMU QMP Reference Manual.
#[qapi(name = "X86CPUFeatureWordInfo")]
#[qapi(added = "1.5.0")]
pub struct X86cpuFeatureWordInfo {
    #[qapi(name = "cpuid-input-eax")]
    pub cpuid_input_eax: i64,
    #[qapi(name = "cpuid-input-ecx")]
    pub cpuid_input_ecx: Option<i64>,
    #[qapi(name = "cpuid-register")]
    pub cpuid_register: X86cpuRegister32,
    pub features: i64,
}

/// QMP enum `X86CPURegister32`; see the QEMU QMP Reference Manual.
#[qapi(name = "X86CPURegister32")]
#[qapi(added = "1.5.0")]
pub enum X86cpuRegister32 {
    #[qapi(name = "EAX")]
    Eax,
    #[qapi(name = "EBP")]
    Ebp,
    #[qapi(name = "EBX")]
    Ebx,
    #[qapi(name = "ECX")]
    Ecx,
    #[qapi(name = "EDI")]
    Edi,
    #[qapi(name = "EDX")]
    Edx,
    #[qapi(name = "ESI")]
    Esi,
    #[qapi(name = "ESP")]
    Esp,
}

/// QMP struct `XBZRLECacheStats`; see the QEMU QMP Reference Manual.
#[qapi(name = "XBZRLECacheStats")]
#[qapi(added = "1.2.0")]
pub struct XbzrleCacheStats {
    pub bytes: i64,
    #[qapi(name = "cache-miss")]
    pub cache_miss: i64,
    #[qapi(name = "cache-miss-rate")]
    #[qapi(added = "2.1.0")]
    #[qapi(required)]
    pub cache_miss_rate: Option<f64>,
    #[qapi(name = "cache-size")]
    #[qapi(added = "6.0.0")]
    #[qapi(required)]
    pub cache_size: Option<u64>,
    #[qapi(name = "cache-size")]
    #[qapi(removed = "6.0.0")]
    #[qapi(required)]
    pub cache_size_legacy: Option<i64>,
    #[qapi(name = "encoding-rate")]
    #[qapi(added = "5.1.0")]
    #[qapi(required)]
    pub encoding_rate: Option<f64>,
    pub overflow: i64,
    pub pages: i64,
}

/// QMP struct `XDbgBlockGraph`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.0.0")]
pub struct XDbgBlockGraph {
    pub edges: Vec<XDbgBlockGraphEdge>,
    pub nodes: Vec<XDbgBlockGraphNode>,
}

/// QMP struct `XDbgBlockGraphEdge`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.0.0")]
pub struct XDbgBlockGraphEdge {
    pub child: u64,
    pub name: String,
    pub parent: u64,
    pub perm: Vec<BlockPermission>,
    #[qapi(name = "shared-perm")]
    pub shared_perm: Vec<BlockPermission>,
}

/// QMP struct `XDbgBlockGraphNode`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.0.0")]
pub struct XDbgBlockGraphNode {
    pub id: u64,
    pub name: String,
    pub r#type: XDbgBlockGraphNodeType,
}

/// QMP enum `XDbgBlockGraphNodeType`; see the QEMU QMP Reference Manual.
#[qapi(added = "4.0.0")]
pub enum XDbgBlockGraphNodeType {
    #[qapi(name = "block-backend")]
    BlockBackend,
    #[qapi(name = "block-driver")]
    BlockDriver,
    #[qapi(name = "block-job")]
    BlockJob,
}

/// QMP union `YankInstance`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct YankInstance {
    #[qapi(discriminator)]
    pub r#type: YankInstanceType,
    #[qapi(union)]
    #[qapi(flatten)]
    pub u: Option<YankInstanceBranch>,
}

/// QMP struct `YankInstanceBlockNode`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct YankInstanceBlockNode {
    #[qapi(name = "node-name")]
    pub node_name: String,
}

#[qapi(union_branch)]
#[qapi(added = "6.0.0")]
#[allow(
    clippy::large_enum_variant,
    reason = "QAPI union branches hold their payload structs by value"
)]
pub enum YankInstanceBranch {
    #[qapi(name = "block-node")]
    BlockNode(YankInstanceBlockNode),
    #[qapi(name = "chardev")]
    Chardev(YankInstanceChardev),
    #[qapi(name = "migration")]
    Migration,
}

/// QMP struct `YankInstanceChardev`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub struct YankInstanceChardev {
    pub id: String,
}

/// QMP enum `YankInstanceType`; see the QEMU QMP Reference Manual.
#[qapi(added = "6.0.0")]
pub enum YankInstanceType {
    #[qapi(name = "block-node")]
    BlockNode,
    #[qapi(name = "chardev")]
    Chardev,
    #[qapi(name = "migration")]
    Migration,
}

/// QMP enum `ZeroPageDetection`; see the QEMU QMP Reference Manual.
#[qapi(added = "9.0.0")]
pub enum ZeroPageDetection {
    #[qapi(name = "legacy")]
    Legacy,
    #[qapi(name = "multifd")]
    Multifd,
    #[qapi(name = "none")]
    None,
}

pub mod commands;
pub mod events;
pub mod metadata;
