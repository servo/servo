# Testing: https://w3c.github.io/core-aam/#role-map-form

TEST_HTML = "<div role='form' id='test' aria-labelled='the name of the form'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Role: ROLE_LANDMARK
    # Object Attribute: xml-roles:form

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.LANDMARK
    assert "xml-roles:form" in atspi.Accessible.get_attributes_as_array(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # AXRole: AXGroup
#     # AXSubrole: AXLandmarkForm

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Role: IA2_ROLE_FORM
#     # Object Attribute: xml-roles:form

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Control Type: Group
#     # Localized Control Type: form
#     # Landmark Type: Form
