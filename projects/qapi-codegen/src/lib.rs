#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

pub mod parsers;

pub use parsers::{
    parser_version,
    qapi_cond::{QapiCond, take_cond},
    qapi_feature::{QapiFeature, take_feature},
    qapi_features::{QapiFeatures, take_features},
    qapi_include::{QapiInclude, take_include},
    qapi_pragma::{QapiPragma, take_pragma},
    set_parser_version, with_parser_version,
};

pub use parsers::common::QapiString;
