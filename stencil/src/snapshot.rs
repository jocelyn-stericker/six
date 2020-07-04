pub fn snapshot(path: &str, contents: &str) {
    if std::env::vars().any(|(key, _val)| key == "SIX_SNAPSHOT") {
        std::fs::write(path, contents).unwrap();
    } else {
        assert_eq!(std::fs::read_to_string(path).unwrap(), contents);
    }
}
