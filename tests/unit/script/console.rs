use script::test::console::{ConsoleTable, TableRow};

/// Equivalent to `console.table(["A".repeat(70000)])`.
///
/// <https://github.com/servo/servo/issues/49522>.
#[test]
fn console_table_large_width() {
    let table = ConsoleTable {
        columns: Vec::new(),
        rows: vec![TableRow {
            index: "0".to_string(),
            cells: vec![],
            value: Some("A".repeat(70000)),
        }],
    };

    // should not panic here
    let _rendered = table.render();
}
