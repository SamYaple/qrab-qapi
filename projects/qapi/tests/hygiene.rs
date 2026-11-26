//! Valid field names must not capture generated serde locals.

// The tests use items and members of older releases, which the newest
// release no longer has and which are therefore deprecated.
#![allow(deprecated)]

use serde_json::json;

#[qapi::qapi]
struct Tail {
    tail: String,
}

#[qapi::qapi]
struct Flattened {
    map: String,
    full_map: String,
    __qapi_map: String,
    #[qapi(flatten)]
    extra: Tail,
}

#[qapi::qapi]
struct Ordinary {
    __field_value: String,
    __qapi_field_value: String,
    value: String,
    value2: String,
}

#[test]
fn flattened_field_names_and_internal_prefixes_round_trip() {
    let wire = json!({"map":"map","full_map":"full","__qapi_map":"reserved","tail":"tail"});
    let value: Flattened = serde_json::from_value(wire.clone()).unwrap();
    assert_eq!(serde_json::to_value(value).unwrap(), wire);
}

#[test]
fn ordinary_field_names_cannot_capture_another_fields_storage() {
    let wire = json!({"__field_value":"internal-looking","__qapi_field_value":"second","value":"value","value2":"second value"});
    let value: Ordinary = serde_json::from_value(wire.clone()).unwrap();
    assert_eq!(serde_json::to_value(value).unwrap(), wire);
}

#[qapi::qapi]
struct D {
    text: String,
}
#[qapi::qapi]
enum S {
    Value(D),
}
#[qapi::qapi]
struct __Visitor {
    text: String,
}
#[qapi::qapi]
struct __QapiDeserializer {
    text: String,
}
#[qapi::qapi]
struct WithD {
    data: D,
}

#[test]
fn schema_type_names_do_not_capture_visitor_or_serde_generic_names() {
    let value: S = serde_json::from_value(json!({"text":"value"})).unwrap();
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({"text":"value"})
    );
    let value: __Visitor = serde_json::from_value(json!({"text":"visitor"})).unwrap();
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({"text":"visitor"})
    );
    let value: __QapiDeserializer = serde_json::from_value(json!({"text":"deserializer"})).unwrap();
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({"text":"deserializer"})
    );
    let value: WithD = serde_json::from_value(json!({"data":{"text":"nested"}})).unwrap();
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({"data":{"text":"nested"}})
    );
}

#[qapi::qapi]
struct SharedStorage {
    __qapi_raw_76616c7565: String,
    #[qapi(name = "value", added = "10.0.0")]
    current: Option<String>,
    #[qapi(name = "value", removed = "10.0.0")]
    historical: Option<i64>,
}
#[qapi::qapi]
enum Kind {
    #[qapi(name = "one")]
    One,
}
#[qapi::qapi(union_branch)]
enum Branch {
    #[qapi(name = "one")]
    One(Tail),
}
#[qapi::qapi]
struct UnionStorage {
    #[qapi(discriminator)]
    kind: Kind,
    __qapi_union_payload: String,
    #[qapi(flatten, union)]
    u: Option<Branch>,
}

#[test]
fn user_fields_cannot_capture_shared_wire_or_union_payload_storage() {
    qapi::with_version((11, 1, 3), || {
        let wire = json!({"__qapi_raw_76616c7565":"user","value":"current"});
        let value: SharedStorage = serde_json::from_value(wire.clone()).unwrap();
        assert_eq!(serde_json::to_value(value).unwrap(), wire);
        let wire = json!({"kind":"one","__qapi_union_payload":"user","tail":"payload"});
        let value: UnionStorage = serde_json::from_value(wire.clone()).unwrap();
        assert_eq!(serde_json::to_value(value).unwrap(), wire);
    });
}

#[qapi::qapi]
struct __QapiVisitor {
    text: String,
}
#[qapi::qapi]
struct ReferencedVisitor {
    value: __QapiVisitor,
}

#[test]
fn helper_type_names_cannot_shadow_referenced_schema_types() {
    let wire = json!({"value":{"text":"schema type"}});
    let value: ReferencedVisitor = serde_json::from_value(wire.clone()).unwrap();
    assert_eq!(serde_json::to_value(value).unwrap(), wire);
}

/// The expansion names what it uses by absolute path, qrab-qapi's items as
/// `::qapi`, so it compiles where the prelude's names, `std`, `serde`,
/// `serde_json` and `qapi` mean something else, as in a crate that depends
/// only on qrab-qapi.
mod shadowed {
    #![allow(dead_code, unused_macros)]

    pub type Result<T> = ::core::result::Result<T, ()>;
    pub struct String;
    pub struct Option;
    pub struct Vec;
    pub struct Some;
    pub struct None;
    pub struct Ok;
    pub struct Err;
    mod qapi {}
    mod serde {}
    mod serde_json {}
    mod std {}
    macro_rules! format {
        ($($t:tt)*) => {
            compile_error!("bare format!")
        };
    }
    macro_rules! concat {
        ($($t:tt)*) => {
            compile_error!("bare concat!")
        };
    }
    macro_rules! stringify {
        ($($t:tt)*) => {
            compile_error!("bare stringify!")
        };
    }
    macro_rules! write {
        ($($t:tt)*) => {
            compile_error!("bare write!")
        };
    }
    macro_rules! eprintln {
        ($($t:tt)*) => {
            compile_error!("bare eprintln!")
        };
    }

    #[::qapi::qapi]
    pub enum Kind {
        #[qapi(name = "one")]
        One,
        #[qapi(name = "two", deprecated(added = "1.0.0"))]
        Two,
    }

    #[::qapi::qapi]
    pub struct Payload {
        pub text: ::std::string::String,
    }

    #[::qapi::qapi(union_branch)]
    pub enum Branch {
        #[qapi(name = "one")]
        One(Payload),
        #[qapi(name = "two")]
        Two(Payload),
    }

    #[::qapi::qapi]
    pub enum Alternate {
        Number(i64),
        Text(::std::string::String),
        Object(Payload),
    }

    #[::qapi::qapi(simple_union(removed = "2.0.0"))]
    pub struct Union {
        #[qapi(name = "type", discriminator)]
        pub kind: Kind,
        #[qapi(flatten, union)]
        pub u: ::core::option::Option<Branch>,
    }

    #[::qapi::qapi(
        command,
        name = "do-it",
        returns = "Payload",
        deprecated(added = "1.0.0")
    )]
    pub struct DoIt {
        pub count: i64,
        #[qapi(removed = "2.0.0")]
        pub old: ::core::option::Option<i64>,
        #[qapi(added = "2.0.0", required)]
        pub new: ::core::option::Option<i64>,
        #[qapi(name = "value", added = "2.0.0")]
        pub value: ::core::option::Option<::std::string::String>,
        #[qapi(name = "value", removed = "2.0.0")]
        pub value_legacy: ::core::option::Option<i64>,
        #[qapi(deprecated(added = "1.0.0"))]
        pub list: ::std::vec::Vec<i64>,
    }

    #[::qapi::qapi(event, name = "DONE")]
    pub struct Done {
        pub payload: Payload,
    }
}

#[test]
fn expansion_does_not_depend_on_names_in_scope() {
    use qapi::{QapiCommand, QapiEvent};
    assert_eq!(<shadowed::DoIt as QapiCommand>::NAME, "do-it");
    assert_eq!(<shadowed::Done as QapiEvent>::NAME, "DONE");
    assert_eq!(shadowed::Kind::One.to_string(), "one");
    qapi::with_version((2, 0, 0), || {
        let command = shadowed::DoIt {
            count: 1,
            old: None,
            new: Some(2),
            value: Some("current".into()),
            value_legacy: None,
            list: vec![1],
        };
        let wire = json!({"count": 1, "new": 2, "value": "current", "list": [1]});
        assert_eq!(serde_json::to_value(&command).unwrap(), wire);
        let back: shadowed::DoIt = serde_json::from_value(wire).unwrap();
        assert_eq!(back.value.as_deref(), Some("current"));

        let wire = json!({"type": "two", "text": "flat"});
        let union: shadowed::Union = serde_json::from_value(wire.clone()).unwrap();
        assert_eq!(serde_json::to_value(&union).unwrap(), wire);

        for wire in [json!(3), json!("text"), json!({"text": "object"})] {
            let value: shadowed::Alternate = serde_json::from_value(wire.clone()).unwrap();
            assert_eq!(serde_json::to_value(&value).unwrap(), wire);
        }
        let event: shadowed::Done =
            serde_json::from_value(json!({"payload": {"text": "done"}})).unwrap();
        assert_eq!(
            event.to_string(),
            "DONE(payload=Payload { text: \"done\" })"
        );
    });
}
