# Testing: https://w3c.github.io/core-aam/#ariaColIndex

TEST_HTML = "<div role='grid'> <div role='row'> <div role='cell' id='test' aria-colindex='3'>content</div> </div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Object Attribute: colindex: should contain the author-provided value.
    # Method: atk_table_cell_get_position(): should return the actual (zero-based) column index.

    node = atspi.find_node("test", session.url)
    assert "colindex:3" in atspi.Accessible.get_attributes_as_array(node)
    assert atspi.TableCell.get_position(node).column == 0

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXARIAColumnIndex: <value>

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Object Attribute: colindex:<value>
#     # Method: IAccessible2::groupPosition(): positionInGroup=<value> on cells and headers

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: GridItem.Column: <value> (zero-based)
