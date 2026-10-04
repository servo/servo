# Testing: https://w3c.github.io/core-aam/#ariaRowCount

TEST_HTML = "<div role='table' id='test' aria-rowcount='3'> <div role='row' id='row'> <div role='cell'>content</div> </div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Object Attribute: rowcount: should contain the author-provided value.
    # Method: atk_table_get_n_rows(): should return the actual number of rows.

    node = atspi.find_node("test", session.url)
    assert "rowcount:3" in atspi.Accessible.get_attributes_as_array(node)
    assert atspi.Table.get_n_rows(node) == 1

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXARIARowCount: <value>

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Object Attribute: rowcount:<value>
#     # Method: IAccessible2::groupPosition(): similarItemsInGroup=<value> on rows

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: Grid.RowCount: <value>
