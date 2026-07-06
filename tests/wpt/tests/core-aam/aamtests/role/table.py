# Testing: https://w3c.github.io/core-aam/#role-map-table

TEST_HTML = "<div role='table' id='test'> <div role='row' id='headerrow'> <div role='columnheader' id='colheader1'>content</div> <div role='columnheader' id='colheader2'>content</div> </div> <div role='row'> <div role='rowheader' id='rowheader1'>content</div> <div role='cell'>content</div> </div> <div role='row'> <div role='rowheader' id='rowheader2'>content</div> <div role='cell'>content</div> </div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ROLE_TABLE
    # Object Attribute: xml-roles:table
    # Interface: Table

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.TABLE
    assert "xml-roles:table" in atspi.Accessible.get_attributes_as_array(node)
    assert atspi.Accessible.get_table_iface(node) is not None

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # AXRole: AXTable
#     # AXSubrole: <nil>
#     # AXColumnHeaderUIElements: a list of pointers to the columnheader elements
#     # AXHeader: a pointer to the row or group containing those columnheader elements
#     # AXRowHeaderUIElements: a list of pointers to the rowheader elements

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Role: ROLE_SYSTEM_TABLE
#     # Object Attribute: xml-roles:table
#     # Interface: IAccessibleTable2

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Control Type: Table
#     # Control Pattern: Grid
#     # Control Pattern: Table
