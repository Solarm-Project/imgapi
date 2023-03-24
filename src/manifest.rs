use chrono::{DateTime, Utc};
use derive_builder::{Builder, UninitializedFieldError};
use indexmap::IndexMap;
use miette::Diagnostic;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fmt::Display;
use strum::{Display as StrumDisplay, EnumString};
use thiserror::Error;
use url::Url;
use uuid::Uuid;

#[doc = "Error type for All zfs related builders"]
#[derive(Debug, Error, Diagnostic)]
#[non_exhaustive]
pub enum ManifestBuilderError {
    // where `LoremBuilder` is the name of the builder struct
    /// Uninitialized field
    UninitializedField(&'static str),
    /// Custom validation error
    ValidationError(String),
}

impl From<String> for ManifestBuilderError {
    fn from(s: String) -> Self {
        Self::ValidationError(s)
    }
}
impl From<UninitializedFieldError> for ManifestBuilderError {
    fn from(value: UninitializedFieldError) -> Self {
        Self::UninitializedField(value.field_name())
    }
}
impl Display for ManifestBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ManifestBuilderError::UninitializedField(value) => {
                write!(f, "field {} must be initialized", value)
            }
            ManifestBuilderError::ValidationError(s) => write!(f, "validation error: {}", s),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "ManifestBuilderError"))]
pub struct Manifest {
    //Version of the manifest format/spec. The current value is 2.
    #[builder(setter(skip), default = "2")]
    pub v: i32,

    //The unique identifier for a UUID. This is set by the IMGAPI server. See details below.
    #[builder(setter(skip), default = "uuid::Builder::nil().into_uuid()")]
    pub uuid: Uuid,

    //The UUID of the owner of this image (the account that created it).
    #[builder(setter(skip), default = "uuid::Builder::nil().into_uuid()")]
    pub owner: Uuid,

    //A short name for this image. Max 512 characters (though practical usage should be much shorter). No uniqueness guarantee.
    #[builder(setter(into))]
    pub name: String,

    //A version string for this image. Max 128 characters. No uniqueness guarantee.
    #[builder(setter(into))]
    pub version: String,

    //A short description of the image.
    #[builder(setter(into, strip_option), default)]
    pub description: Option<String>,

    //Homepage URL where users can find more information about the image.
    #[builder(setter(into, strip_option), default)]
    pub homepage: Option<Url>,

    //URL of the End User License Agreement (EULA) for the image.
    #[builder(setter(into, strip_option), default)]
    pub eula: Option<Url>,

    //Indicates if the image has an icon file. If not present, then no icon is present.
    #[builder(setter(into, strip_option), default)]
    pub icon: Option<bool>,

    //The current state of the image. One of 'active', 'unactivated', 'disabled', 'creating', 'failed'.
    #[builder(default)]
    pub state: ImageState,

    //An object with details on image creation failure. It only exists when state=='failed'.
    #[builder(setter(into, strip_option), default)]
    pub error: Option<Map<String, Value>>,

    //Indicates if this image is available for provisioning.
    #[builder(default = "false")]
    pub disabled: bool,

    //Indicates if this image is publicly available.
    #[builder(default = "false")]
    pub public: bool,

    //The date at which the image is activated. Set by the IMGAPI server.
    #[builder(setter(into, strip_option), default)]
    pub published_at: Option<DateTime<Utc>>,

    //The image type. One of "zone-dataset" for a ZFS dataset used to create a new SmartOS zone, "lx-dataset" for a Lx-brand image, "lxd" for a LXD image, "zvol" for a virtual machine image or "other" for image types that serve any other specific purpose.
    #[serde(rename = "type")]
    #[builder(setter(into), default)]
    pub image_type: ImageType,

    //The OS family this image provides. One of "smartos", "windows", "linux", "bsd", "illumos" or "other".
    #[builder(setter(into), default)]
    pub os: ImageOs,

    //The origin image UUID if this is an incremental image.
    #[builder(setter(into, strip_option), default)]
    pub origin: Option<Uuid>,

    //An array of objects describing the image files.
    #[builder(default)]
    pub files: Vec<Map<String, Value>>,

    //Access Control List. An array of account UUIDs given access to a private image. The field is only relevant to private images.
    #[builder(setter(into, strip_option), default)]
    pub acl: Option<Vec<Uuid>>,

    //A set of named requirements for provisioning a VM with this image
    #[builder(setter(into, strip_option), default)]
    pub requirements: Option<ImageRequirements>,

    //A list of users for which passwords should be generated for provisioning. This may only make sense for some images. Example: [{"name": "root"}, {"name": "admin"}]
    #[builder(setter(into, strip_option), default)]
    pub users: Option<Vec<ImageUsers>>,

    //A list of tags that can be used by operators for additional billing processing.
    #[builder(setter(into, strip_option), default)]
    pub billing_tags: Option<Vec<String>>,

    //An object that defines a collection of properties that is used by other APIs to evaluate where should customer VMs be placed.
    #[builder(setter(into, strip_option), default)]
    pub traits: Option<Vec<String>>,

    //An object of key/value pairs that allows clients to categorize images by any given criteria.
    #[builder(setter(into, strip_option), default)]
    pub tags: Option<IndexMap<String, String>>,

    //A boolean indicating whether to generate passwords for the users in the "users" field. If not present, the default value is true.
    #[builder(setter(into, strip_option), default)]
    pub generate_password: Option<bool>,

    //A list of inherited directories (other than the defaults for the brand).
    #[builder(setter(into, strip_option), default)]
    pub inherited_directories: Option<Vec<String>>,

    //Array of channel names to which this image belongs.
    #[builder(setter(into, strip_option), default)]
    pub channels: Option<Vec<String>>,

    #[serde(flatten)]
    #[builder(setter(into, strip_option), default)]
    pub vm_image_properties: Option<ImageVMProperties>,
}

#[derive(Default, Deserialize, Serialize, Debug, Clone, StrumDisplay, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImageState {
    Active,
    Unactivated,
    Disabled,
    #[default]
    Creating,
    Failed,
}

#[derive(Default, Deserialize, Serialize, Debug, Clone, StrumDisplay, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ImageType {
    #[strum(serialize = "zone-dataset")]
    #[default]
    ZoneDataset,
    #[strum(serialize = "lx-dataset")]
    LxDataset,
    #[strum(serialize = "lxd")]
    Lxd,
    #[strum(serialize = "zvol")]
    Zvol,
    #[strum(serialize = "other")]
    Other,
}

#[derive(Default, Deserialize, Serialize, Debug, Clone, StrumDisplay, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ImageOs {
    #[default]
    Smartos,
    Windows,
    Linux,
    Bsd,
    Illumos,
    Other,
}

#[derive(Deserialize, Serialize, Debug, Clone, Builder)]
pub struct ImageRequirements {
    //Defines the minimum number of network interfaces required by this image.
    #[builder(setter(into, strip_option), default)]
    pub networks: Option<Vec<RequirementNetworks>>,

    //Defines the brand that is required to provision with this image.
    #[builder(setter(into, strip_option), default)]
    pub brand: Option<String>,

    //Indicates that provisioning with this image requires that an SSH public key be provided.
    #[builder(setter(into, strip_option), default)]
    pub ssh_key: Option<bool>,

    //Minimum RAM (in MiB) required to provision this image.
    #[builder(setter(into, strip_option), default)]
    pub min_ram: Option<i64>,

    //Maximum RAM (in MiB) this image may be provisioned with.
    #[builder(setter(into, strip_option), default)]
    pub max_ram: Option<i64>,

    //Minimum platform requirement for provisioning with this image.
    #[builder(setter(into, strip_option), default)]
    pub min_platform: Option<IndexMap<String, String>>,

    //Maximum platform requirement for provisioning with this image.
    #[builder(setter(into, strip_option), default)]
    pub max_platform: Option<IndexMap<String, String>>,

    //Bootrom image to use with this image.
    #[builder(setter(into, strip_option), default)]
    pub bootrom: Option<ImageRequirementBootRom>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Builder)]
pub struct RequirementNetworks {
    name: String,
    description: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, StrumDisplay)]
#[serde(rename_all = "kebab-case")]
pub enum ImageRequirementBootRom {
    Bios,
    Uefi,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ImageUsers {
    name: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, Builder)]
#[builder(build_fn(error = "ManifestBuilderError"))]
pub struct ImageVMProperties {
    //NIC driver used by this VM image.
    #[builder(setter(into))]
    pub nic_driver: NetDrivers,

    //Disk driver used by this VM image.
    #[builder(setter(into))]
    pub disk_driver: DiskDrivers,

    //The QEMU CPU model to use for this VM image.
    #[builder(setter(into))]
    pub cpu_type: String,

    //The size (in MiB) of this VM image's disk.
    pub image_size: u64,
}

#[derive(Deserialize, Serialize, Debug, Clone, EnumString, StrumDisplay)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum NetDrivers {
    Virtio,
    E1000g0,
}

#[derive(Deserialize, Serialize, Debug, Clone, EnumString, StrumDisplay)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum DiskDrivers {
    Virtio,
    Sata,
}

#[derive(Deserialize, Serialize, Debug, Clone, Builder)]
pub struct ImageFile {
    //SHA-1 hex digest of the file content. Used for upload/download corruption checking.
    pub sha1: String,

    //Number of bytes. Maximum 20GiB. This maximum is meant to be a "you'll never hit it" cap, the purpose is to inform cache handling in IMGAPI servers.
    pub size: i64,

    //The type of file compression used by the file. One of 'bzip2', 'gzip', 'none'.
    pub compression: ImageFileCompression,

    //Optional. The ZFS internal unique identifier for this dataset's snapshot (available via zfs get guid SNAPSHOT, e.g. zfs get guid zones/f669428c-a939-11e2-a485-b790efc0f0c1@final). If available, this is used to ensure a common base snapshot for incremental images (via imgadm create -i) and VM migrations (via vmadm send/receive).
    #[builder(setter(into, strip_option), default)]
    pub dataset_guid: Option<String>,

    //Only included if ?inclAdminFields=true is passed to GetImage/ListImages. The IMGAPI storage type used to store this file.
    #[builder(setter(into, strip_option), default)]
    pub stor: Option<String>,

    //Optional. Docker digest of the file contents. Only used when manifest.type is 'docker'. This field gets set automatically by the AdminImportDockerImage call.
    #[builder(setter(into, strip_option), default)]
    pub digest: Option<String>,

    //Optional. Docker digest of the uncompressed file contents. Only used when manifest.type is 'docker'. This field gets set automatically by the AdminImportDockerImage call. Note that this field will be removed in a future version of IMGAPI.
    #[serde(rename = "uncompressedDigest")]
    #[builder(setter(into, strip_option), default)]
    pub uncompressed_digest: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, StrumDisplay)]
#[serde(rename_all = "kebab-case")]
pub enum ImageFileCompression {
    Bzip2,
    Gzip,
    None,
}
