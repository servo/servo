/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

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

    // This should not panic.
    let _rendered = table.render();
}
