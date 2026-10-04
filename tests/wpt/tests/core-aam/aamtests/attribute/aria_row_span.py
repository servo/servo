# Testing: https://w3c.github.io/core-aam/#ariaRowSpan

TEST_HTML = "<div role='grid'> <div role='row'> <div role='cell' id='test' aria-rowspan='2'>content</div> </div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Object Attribute: rowspan: should contain the author-provided value.
    # Method: atk_table_cell_get_row_column_span(): should return the actual row span.

    node = atspi.find_node("test", session.url)
    assert "rowspan:2" in atspi.Accessible.get_attributes_as_array(node)
    assert atspi.TableCell.get_row_column_span(node).row_span == 1

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXRowIndexRange.length: <value>

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Object Attribute: rowspan:<value>
#     # Method: IAccessibleTableCell::rowExtent(): column=<value>

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: GridItem.RowSpan: <value>
