/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Tests for proper table box tree construction.

mod tables {
    use euclid::Vector2D;
    use layout_2020::table::{Table, TableBuilder, TableSlot, TableSlotCell, TableSlotOffset};

    fn row_lengths(table: &Table) -> Vec<usize> {
        table.slots.iter().map(|row| row.len()).collect()
    }

    fn slot_is_cell_with_id(slot: &TableSlot, id: usize) -> bool {
        match slot {
            TableSlot::Cell(cell) if cell.node_id() == id => true,
            _ => false,
        }
    }

    fn slot_is_empty(slot: &TableSlot) -> bool {
        match slot {
            TableSlot::Empty => true,
            _ => false,
        }
    }

    fn slot_is_spanned_with_offsets(slot: &TableSlot, offsets: Vec<(usize, usize)>) -> bool {
        match slot {
            TableSlot::Spanned(slot_offsets) => {
                let offsets: Vec<TableSlotOffset> = offsets
                    .iter()
                    .map(|offset| Vector2D::new(offset.0, offset.1))
                    .collect();
                offsets == *slot_offsets
            },
            _ => false,
        }
    }

    #[test]
    fn test_empty_table() {
        let table_builder = TableBuilder::new_for_tests();
        let table = table_builder.finish();
        assert!(table.slots.is_empty())
    }

    #[test]
    fn test_simple_table() {
        let mut table_builder = TableBuilder::new_for_tests();

        table_builder.start_row();
        table_builder.add_cell(TableSlotCell::mock_for_testing(1, 1, 1));
        table_builder.add_cell(TableSlotCell::mock_for_testing(2, 1, 1));
        table_builder.end_row();

        table_builder.start_row();
        table_builder.add_cell(TableSlotCell::mock_for_testing(3, 1, 1));
        table_builder.add_cell(TableSlotCell::mock_for_testing(4, 1, 1));
        table_builder.end_row();

        let table = table_builder.finish();
        assert_eq!(row_lengths(&table), vec![2, 2]);

        assert!(slot_is_cell_with_id(&table.slots[0][0], 1));
        assert!(slot_is_cell_with_id(&table.slots[0][1], 2));
        assert!(slot_is_cell_with_id(&table.slots[1][0], 3));
        assert!(slot_is_cell_with_id(&table.slots[1][1], 4));
    }

    #[test]
    fn test_simple_rowspan() {
        let mut table_builder = TableBuilder::new_for_tests();

        table_builder.start_row();
        table_builder.add_cell(TableSlotCell::mock_for_testing(1, 1, 1));
        table_builder.add_cell(TableSlotCell::mock_for_testing(2, 1, 1));
        table_builder.add_cell(TableSlotCell::mock_for_testing(3, 1, 2));
        table_builder.end_row();

        table_builder.start_row();
        table_builder.add_cell(TableSlotCell::mock_for_testing(4, 1, 1));
        table_builder.end_row();

        let table = table_builder.finish();
        assert_eq!(row_lengths(&table), vec![3, 3]);

        assert!(slot_is_cell_with_id(&table.slots[0][0], 1));
        assert!(slot_is_cell_with_id(&table.slots[0][1], 2));
        assert!(slot_is_cell_with_id(&table.slots[0][2], 3));

        assert!(slot_is_cell_with_id(&table.slots[1][0], 4));
        assert!(slot_is_empty(&table.slots[1][1]));
        assert!(slot_is_spanned_with_offsets(
            &table.slots[1][2],
            vec![(0, 1)]
        ));
    }

    #[test]
    fn test_simple_colspan() {
        let mut table_builder = TableBuilder::new_for_tests();

        table_builder.start_row();
        table_builder.add_cell(TableSlotCell::mock_for_testing(1, 3, 1));
        table_builder.add_cell(TableSlotCell::mock_for_testing(2, 1, 1));
        table_builder.add_cell(TableSlotCell::mock_for_testing(3, 1, 1));
        table_builder.end_row();

        table_builder.start_row();
        table_builder.add_cell(TableSlotCell::mock_for_testing(4, 1, 1));
        table_builder.add_cell(TableSlotCell::mock_for_testing(5, 3, 1));
        table_builder.add_cell(TableSlotCell::mock_for_testing(6, 1, 1));
        table_builder.end_row();

        table_builder.start_row();
        table_builder.add_cell(TableSlotCell::mock_for_testing(7, 1, 1));
        table_builder.add_cell(TableSlotCell::mock_for_testing(8, 1, 1));
        table_builder.add_cell(TableSlotCell::mock_for_testing(9, 3, 1));
        table_builder.end_row();

        let table = table_builder.finish();
        assert_eq!(row_lengths(&table), vec![5, 5, 5]);

        assert!(slot_is_cell_with_id(&table.slots[0][0], 1));
        assert!(slot_is_spanned_with_offsets(
            &table.slots[0][1],
            vec![(1, 0)]
        ));
        assert!(slot_is_spanned_with_offsets(
            &table.slots[0][2],
            vec![(2, 0)]
        ));
        assert!(slot_is_cell_with_id(&table.slots[0][3], 2));
        assert!(slot_is_cell_with_id(&table.slots[0][4], 3));

        assert!(slot_is_cell_with_id(&table.slots[1][0], 4));
        assert!(slot_is_cell_with_id(&table.slots[1][1], 5));
        assert!(slot_is_spanned_with_offsets(
            &table.slots[1][2],
            vec![(1, 0)]
        ));
        assert!(slot_is_spanned_with_offsets(
            &table.slots[1][3],
            vec![(2, 0)]
        ));
        assert!(slot_is_cell_with_id(&table.slots[1][4], 6));

        assert!(slot_is_cell_with_id(&table.slots[2][0], 7));
        assert!(slot_is_cell_with_id(&table.slots[2][1], 8));
        assert!(slot_is_cell_with_id(&table.slots[2][2], 9));
        assert!(slot_is_spanned_with_offsets(
            &table.slots[2][3],
            vec![(1, 0)]
        ));
        assert!(slot_is_spanned_with_offsets(
            &table.slots[2][4],
            vec![(2, 0)]
        ));
    }

    #[test]
    fn test_simple_table_model_error() {
        let mut table_builder = TableBuilder::new_for_tests();

        table_builder.start_row();
        table_builder.add_cell(TableSlotCell::mock_for_testing(1, 1, 1));
        table_builder.add_cell(TableSlotCell::mock_for_testing(2, 1, 1));
        table_builder.add_cell(TableSlotCell::mock_for_testing(3, 1, 2));
        table_builder.end_row();

        table_builder.start_row();
        table_builder.add_cell(TableSlotCell::mock_for_testing(4, 3, 1));
        table_builder.end_row();

        let table = table_builder.finish();
        assert_eq!(row_lengths(&table), vec![3, 3]);

        assert!(slot_is_cell_with_id(&table.slots[0][0], 1));
        assert!(slot_is_cell_with_id(&table.slots[0][1], 2));
        assert!(slot_is_cell_with_id(&table.slots[0][2], 3));

        assert!(slot_is_cell_with_id(&table.slots[1][0], 4));
        assert!(slot_is_spanned_with_offsets(
            &table.slots[1][1],
            vec![(1, 0)]
        ));
        assert!(slot_is_spanned_with_offsets(
            &table.slots[1][2],
            vec![(2, 0), (0, 1)]
        ));
    }

    #[test]
    fn test_simple_rowspan_0() {
        let mut table_builder = TableBuilder::new_for_tests();

        table_builder.start_row();
        table_builder.add_cell(TableSlotCell::mock_for_testing(1, 1, 1));
        table_builder.add_cell(TableSlotCell::mock_for_testing(2, 1, 1));
        table_builder.add_cell(TableSlotCell::mock_for_testing(3, 1, 0));
        table_builder.end_row();

        table_builder.start_row();
        table_builder.end_row();

        table_builder.start_row();
        table_builder.end_row();

        table_builder.start_row();
        table_builder.end_row();

        let table = table_builder.finish();
        assert_eq!(row_lengths(&table), vec![3, 3, 3, 3]);

        assert!(slot_is_empty(&table.slots[1][0]));
        assert!(slot_is_empty(&table.slots[1][1]));
        assert!(slot_is_spanned_with_offsets(
            &table.slots[1][2],
            vec![(0, 1)]
        ));
        assert!(slot_is_spanned_with_offsets(
            &table.slots[2][2],
            vec![(0, 2)]
        ));
        assert!(slot_is_spanned_with_offsets(
            &table.slots[3][2],
            vec![(0, 3)]
        ));
    }

    #[test]
    fn test_incoming_rowspans() {
        let mut table_builder = TableBuilder::new_for_tests();

        table_builder.start_row();
        table_builder.add_cell(TableSlotCell::mock_for_testing(1, 1, 1));
        table_builder.add_cell(TableSlotCell::mock_for_testing(2, 1, 30));
        table_builder.end_row();

        table_builder.start_row();
        table_builder.add_cell(TableSlotCell::mock_for_testing(3, 2, 1));
        table_builder.end_row();

        assert_eq!(table_builder.incoming_rowspans, vec![0, 28]);

        let table = table_builder.finish();
        assert_eq!(row_lengths(&table), vec![2, 2]);
    }
}
