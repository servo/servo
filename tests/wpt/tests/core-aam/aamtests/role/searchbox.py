# Testing: https://w3c.github.io/core-aam/#role-map-searchbox

TEST_HTML = "<div role='searchbox' contenteditable id='test'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ROLE_ENTRY
    # Object Attribute: xml-roles:searchbox
    # Object Attribute: text-input-type:search
    # Interface: EditableText: if aria-readonly is not "true"

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.ENTRY
    assert "xml-roles:searchbox" in atspi.Accessible.get_attributes_as_array(node)
    assert "text-input-type:search" in atspi.Accessible.get_attributes_as_array(node)
    assert atspi.Accessible.get_editable_text_iface(node) is not None

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # AXRole: AXTextField
#     # AXSubrole: AXSearchField

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Role: ROLE_SYSTEM_TEXT
#     # Object Attribute: text-input-type:search

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Control Type: Edit
#     # Localized Control Type: search box
