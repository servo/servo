# Testing: https://w3c.github.io/core-aam/#ariaRoleDescriptionEmptyString

TEST_HTML = "<div role='group' id='test' aria-roledescription=''>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Not mapped

    node = atspi.find_node("test", session.url)
    assert 'roledescription' not in atspi.Accessible.get_attributes(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # AXRoleDescription: Defined as that specified for the role of the element: based on the explicit role if the role attribute is provided; otherwise, based on the implicit role for the host language.

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Not mapped

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Localized Control Type: Defined as that specified for the role of the element: based on the explicit role if the role attribute is provided; otherwise, based on the implicit role for the host language.
