# Testing: https://w3c.github.io/core-aam/#ariaColSpan

TEST_HTML = "<div role='grid'> <div role='row'> <div role='cell' id='test' aria-colspan='3'>content</div> </div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Object Attribute: colspan: should contain the author-provided value.
    # Method: atk_table_cell_get_row_column_span(): should return the actual column span.

    node = atspi.find_node("test", session.url)
    assert "colspan:3" in atspi.Accessible.get_attributes_as_array(node)
    assert atspi.TableCell.get_row_column_span(node).column_span == 1

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXColumnIndexRange.length: <value>

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Object Attribute: colspan:<value>
#     # Method: IAccessibleTableCell::columnExtent(): <value>

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: GridItem.ColumnSpan: <value>
