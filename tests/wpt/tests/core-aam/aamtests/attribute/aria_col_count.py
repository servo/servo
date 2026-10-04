# Testing: https://w3c.github.io/core-aam/#ariaColCount

TEST_HTML = "<div role='table' id='test' aria-colcount='3'> <div role='row'> <div role='cell' id='cell'>content</div> </div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Object Attribute: colcount: should contain the author-provided value.
    # Method: atk_table_get_n_columns(): should return the actual number of columns.

    node = atspi.find_node("test", session.url)
    assert "colcount:3" in atspi.Accessible.get_attributes_as_array(node)
    assert atspi.Table.get_n_columns(node) == 1

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXARIAColumnCount: <value>

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Object Attribute: colcount:<value>
#     # Method: IAccessible2::groupPosition(): similarItemsInGroup=<value> on cells and headers

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: Grid.ColumnCount: <value>
