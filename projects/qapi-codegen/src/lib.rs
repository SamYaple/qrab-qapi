#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

pub mod parsers;

pub use parsers::{
    parser_version,
    qapi_cond::{QapiCond, take_cond},
    qapi_documentation::{
        QapiDocumentation, QapiFreeformDoc, QapiSectionDocumentation, extract_since_from_comment,
    },
    qapi_enum::{QapiEnum, take_enum},
    qapi_enum_value::{QapiEnumValue, take_enum_value},
    qapi_feature::{QapiFeature, take_feature},
    qapi_features::{QapiFeatures, take_features},
    qapi_include::{QapiInclude, take_include},
    qapi_member::{QapiMember, take_member},
    qapi_members::{MembersOrRef, QapiMembers, take_members, take_members_or_ref},
    qapi_pragma::{QapiPragma, take_pragma},
    qapi_struct::{QapiStruct, take_struct},
    qapi_type_ref::{QapiTypeRef, take_type_ref},
    set_parser_version, with_parser_version,
};

pub use parsers::common::QapiString;
