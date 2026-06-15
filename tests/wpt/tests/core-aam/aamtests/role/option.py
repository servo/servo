# Testing: https://w3c.github.io/core-aam/#role-map-option

TEST_HTML = "<div role='listbox'> <div role='option' id='test'>content</div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ROLE_LIST_ITEM

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.LIST_ITEM

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # AXRole: AXStaticText
#     # AXSubrole: <nil>
#     # See also: aria-checked in the State and Property Mapping Tables

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Role: ROLE_SYSTEM_LISTITEM
#     # See also: aria-checked in the State and Property Mapping Tables

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Control Type: ListItem
#     # Control Pattern: Invoke
#     # See also: aria-checked in the State and Property Mapping Tables
