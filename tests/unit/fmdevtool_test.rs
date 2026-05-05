use fm_size::fmdevtool::FMDeveloperTool;

#[test]
fn test_version_to_number() {
    // Test various version formats
    assert_eq!(FMDeveloperTool::version_to_number("21.1.1"), 21_000_100_010_000);
    assert_eq!(FMDeveloperTool::version_to_number("21.1.1.4"), 21_000_100_010_004);
    assert_eq!(FMDeveloperTool::version_to_number("26.0.1"), 26_000_000_010_000);
    assert_eq!(FMDeveloperTool::version_to_number("26.0.1.10"), 26_000_000_010_010);
    assert_eq!(FMDeveloperTool::version_to_number("21"), 21_000_000_000_000);
    assert_eq!(FMDeveloperTool::version_to_number("21.1"), 21_000_100_000_000);
}

#[test]
fn test_version_to_number_invalid() {
    // Test with invalid formats - should default to 0 for missing parts
    assert_eq!(FMDeveloperTool::version_to_number(""), 0);
    assert_eq!(FMDeveloperTool::version_to_number("invalid"), 0);
    assert_eq!(FMDeveloperTool::version_to_number("21.invalid"), 21_000_000_000_000);
}

#[test]
fn test_check_version_supported() {
    // Test supported versions
    assert!(FMDeveloperTool::check_version_supported("21.1.1"));
    assert!(FMDeveloperTool::check_version_supported("21.1.1.4"));
    assert!(FMDeveloperTool::check_version_supported("22.0.0"));
    assert!(FMDeveloperTool::check_version_supported("25.0.0"));
    assert!(FMDeveloperTool::check_version_supported("26.0.1"));
    assert!(FMDeveloperTool::check_version_supported("26.0.1.100")); // Any build number should work
    
    // Test unsupported versions (too old)
    assert!(!FMDeveloperTool::check_version_supported("21.1.0"));
    assert!(!FMDeveloperTool::check_version_supported("20.0.0"));
    assert!(!FMDeveloperTool::check_version_supported("1.0.0"));
    
    // Test unsupported versions (too new)
    assert!(!FMDeveloperTool::check_version_supported("26.0.2"));
    assert!(!FMDeveloperTool::check_version_supported("27.0.0"));
}

#[test]
fn test_get_supported_versions_string() {
    let supported = FMDeveloperTool::get_supported_versions_string();
    assert!(supported.contains("21.1.1"));
    assert!(supported.contains("26.0.1"));
}

#[test]
fn test_parse_table_sizes_csv() {
    let csv = r#"
"TableName","Size"
"Table1","1000"
"Table2","2000"
"#;
    
    let result = FMDeveloperTool::parse_table_sizes_csv(csv).unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].name, "Table1");
    assert_eq!(result[0].size_bytes, 1000);
    assert_eq!(result[1].name, "Table2");
    assert_eq!(result[1].size_bytes, 2000);
}

#[test]
fn test_parse_table_sizes_csv_with_noise() {
    let csv = r#"
Some noise before
"TableName","Size"
"Table1","1000"
Table not found: __checking_if_file_can_be_opened__
"Table2","2000"
More noise
"#;
    
    let result = FMDeveloperTool::parse_table_sizes_csv(csv).unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].name, "Table1");
    assert_eq!(result[1].name, "Table2");
}

#[test]
fn test_parse_table_sizes_csv_empty() {
    let csv = "";
    let result = FMDeveloperTool::parse_table_sizes_csv(csv).unwrap();
    assert_eq!(result.len(), 0);
}

#[test]
fn test_parse_table_sizes_csv_invalid_size() {
    let csv = r#"
"TableName","Size"
"Table1","invalid"
"#;
    
    let result = FMDeveloperTool::parse_table_sizes_csv(csv).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "Table1");
    assert_eq!(result[0].size_bytes, 0); // Invalid size defaults to 0
}

#[test]
fn test_parse_field_sizes_csv() {
    let csv = r#"
"FieldName","Size"
"Field1","500"
"Field2","750"
"#;
    
    let result = FMDeveloperTool::parse_field_sizes_csv(csv).unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].name, "Field1");
    assert_eq!(result[0].field_size_bytes, 500);
    assert_eq!(result[0].value_index_size_bytes, 0);
    assert_eq!(result[0].word_index_size_bytes, 0);
    assert_eq!(result[1].name, "Field2");
    assert_eq!(result[1].field_size_bytes, 750);
}

#[test]
fn test_parse_index_sizes_csv() {
    let csv = r#"
"FieldName","ValueIndexSize","Something","WordIndexSize"
"Field1","100","","200"
"Field2","150","","250"
"#;
    
    let result = FMDeveloperTool::parse_index_sizes_csv(csv).unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].name, "Field1");
    assert_eq!(result[0].value_index_size_bytes, 100);
    assert_eq!(result[0].word_index_size_bytes, 200);
    assert_eq!(result[0].field_size_bytes, 0); // Index sizes don't have field size
    assert_eq!(result[1].name, "Field2");
    assert_eq!(result[1].value_index_size_bytes, 150);
    assert_eq!(result[1].word_index_size_bytes, 250);
}

#[test]
fn test_parse_index_sizes_csv_missing_columns() {
    let csv = r#"
"FieldName","ValueIndexSize"
"Field1","100"
"#;
    
    let result = FMDeveloperTool::parse_index_sizes_csv(csv).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "Field1");
    assert_eq!(result[0].value_index_size_bytes, 100);
    assert_eq!(result[0].word_index_size_bytes, 0); // Missing column defaults to 0
}

#[test]
fn test_filter_csv_lines() {
    let csv = r#"
"TableName","Size"
"Table1","1000"
Some error message
Table not found: __checking_if_file_can_be_opened__
"Table2","2000"
Another line that doesn't start with quote
"#;
    
    let filtered = FMDeveloperTool::filter_csv_lines(csv);
    // Should keep lines that start with quotes or are empty
    assert!(filtered.contains("\"TableName\""));
    assert!(filtered.contains("\"Table1\""));
    assert!(filtered.contains("\"Table2\""));
    // Should filter out error messages and noise
    assert!(!filtered.contains("Some error message"));
    assert!(!filtered.contains("Another line"));
}

#[test]
fn test_filter_csv_lines_with_table_not_found() {
    let csv = r#"
"TableName","Size"
Table not found: __checking_if_file_can_be_opened__
"Table1","1000"
"#;
    
    let filtered = FMDeveloperTool::filter_csv_lines(csv);
    // Should filter out the "Table not found" line with our check pattern
    assert!(!filtered.contains("Table not found"));
    assert!(filtered.contains("\"Table1\""));
}

