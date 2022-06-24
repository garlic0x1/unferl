use unferl::ParsedUrl;

#[test]
fn test_format() {
    let mut parsed =
        ParsedUrl::new("https://garlic0x1.com:9999/foo/bar.php?kei=val#octothorpe".to_string())
            .unwrap();
    // test scheme and null subdomain
    assert_eq!(&parsed.format("%s://%S"), "https://");
    // test port, null user info, and escape
    assert_eq!(&parsed.format("%@%%%P"), "%9999");
    // test file extension
    assert_eq!(&parsed.format("%e"), "php");
    // test auth macro
    assert_eq!(&parsed.format("%a"), "garlic0x1.com:9999");
    // reconstruct url
    assert_eq!(
        &parsed.format("%s://%d%:%P%p%?%q%#%f"),
        "https://garlic0x1.com:9999/foo/bar.php?kei=val#octothorpe"
    );

    parsed =
        ParsedUrl::new("https://user:pass@example.com:8080/?foo=bar#frag".to_string()).unwrap();
    // test auth macro
    assert_eq!(&parsed.format("%a"), "user:pass@example.com:8080");
    // test octothorpe
    assert_eq!(&parsed.format("%^%#%f"), "#frag");
}
