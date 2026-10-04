# Testing: https://w3c.github.io/core-aam/#ariaRowIndex

TEST_HTML = "<div role='grid'> <div role='row' id='row'> <div role='cell' id='test' aria-rowindex='3'>content</div> </div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Object Attribute: rowindex: should contain the author-provided value.
    # Method: atk_table_cell_get_position(): should return the actual (zero-based) row index.

    node = atspi.find_node("test", session.url)
    assert "rowindex:3" in atspi.Accessible.get_attributes_as_array(node)
    assert atspi.TableCell.get_position(node).row == 0

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXARIARowIndex: <value>

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Object Attribute: rowindex:<value>
#     # Method: IAccessible2::groupPosition(): positionInGroup=<value> on rows

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: GridItem.Row: <value> (zero-based)
