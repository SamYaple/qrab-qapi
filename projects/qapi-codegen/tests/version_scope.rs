use qapi_codegen::{
    ReadError, parse_schema, parse_schema_with_version, parser_version, read_schema_with_version,
    with_parser_version,
};
use std::path::Path;

#[test]
fn explicit_file_grammar_restores_the_caller_after_success_and_failure() {
    let original = parser_version();
    with_parser_version((11, 1, 3), || {
        let legacy = "{'enum':'Kind','data':['one',]}";
        let schema = parse_schema_with_version(legacy, (2, 3, 0)).unwrap();
        assert_eq!(schema.len(), 1);
        assert_eq!(parser_version(), (11, 1, 3));
        assert!(parse_schema(legacy).is_err());
        assert!(parse_schema_with_version("invalid", (2, 3, 0)).is_err());
        assert_eq!(parser_version(), (11, 1, 3));
    });
    assert_eq!(parser_version(), original);
}

#[test]
fn reading_schema_files_restores_the_caller_grammar() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("version_scope_read");
    std::fs::create_dir_all(&dir).unwrap();
    let root = dir.join("root.json");
    std::fs::write(&root, "{ 'include': 'legacy.json' }\n").unwrap();
    std::fs::write(dir.join("legacy.json"), "{'enum':'Kind','data':['one',]}\n").unwrap();
    with_parser_version((11, 1, 3), || {
        assert_eq!(
            read_schema_with_version(&root, Some((2, 3, 0)))
                .unwrap()
                .len(),
            2
        );
        assert_eq!(parser_version(), (11, 1, 3));
        let error = read_schema_with_version(&root, None).unwrap_err();
        assert!(
            matches!(
                error,
                ReadError::Parse {
                    version: (11, 1, 3),
                    ..
                }
            ),
            "{error}"
        );
        assert_eq!(parser_version(), (11, 1, 3));
        let error = read_schema_with_version(&dir.join("missing.json"), Some((2, 3, 0)));
        assert!(matches!(error, Err(ReadError::Resolve { .. })));
        assert_eq!(parser_version(), (11, 1, 3));
    });
}

#[test]
fn grammar_scopes_are_nested_thread_local_and_unwind_safe() {
    let original = parser_version();
    let thread_default = std::thread::spawn(parser_version).join().unwrap();
    with_parser_version((2, 3, 0), || {
        let result = std::panic::catch_unwind(|| {
            with_parser_version((4, 2, 0), || {
                assert_eq!(parser_version(), (4, 2, 0));
                assert_eq!(
                    std::thread::spawn(parser_version).join().unwrap(),
                    thread_default
                );
                panic!("scope unwind");
            });
        });
        assert!(result.is_err());
        assert_eq!(parser_version(), (2, 3, 0));
    });
    assert_eq!(parser_version(), original);
}
