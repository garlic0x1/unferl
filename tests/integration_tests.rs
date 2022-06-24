use unferl::ParsedUrl;

#[test]
fn test_format() {
    let parsed = ParsedUrl::new("https://garlic0x1.com:9999".to_string()).unwrap();
    assert_eq!(&parsed.format("%s://%S"), "https://");
    assert_eq!(&parsed.format("%@%%%P"), "%9999");

}
