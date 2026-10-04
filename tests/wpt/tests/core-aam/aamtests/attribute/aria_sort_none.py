# Testing: https://w3c.github.io/core-aam/#ariaSortNone

TEST_HTML = "<div role='grid'> <div role='row'> <div role='columnheader' id='test' aria-sort='none'>content</div> </div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Object Attribute: sort:none: , if the value is not unspecified

    node = atspi.find_node("test", session.url)
    assert "sort:none" in atspi.Accessible.get_attributes_as_array(node)

# Intentionally no AX API test. AX API does not surface this node or attribute.

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Object Attribute: sort:none: , if the value is not unspecified

# Intentionally no UIA test. UIA does not surface this node or attribute.
