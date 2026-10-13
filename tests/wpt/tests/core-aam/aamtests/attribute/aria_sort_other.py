# Testing: https://w3c.github.io/core-aam/#ariaSortOther

TEST_HTML = "<div role='grid'> <div role='row'> <div role='columnheader' id='test' aria-sort='other'>content</div> </div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Object Attribute: sort:other

    node = atspi.find_node("test", session.url)
    assert "sort:other" in atspi.Accessible.get_attributes_as_array(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXSortDirection: AXUnknownSortDirection

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Object Attribute: sort:other

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AriaProperties.sort: other
#     # Property: ItemStatus: other if the element maps to HeaderItem Control Type
