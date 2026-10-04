# Testing: https://w3c.github.io/core-aam/#ariaSortAscending

TEST_HTML = "<div role='grid'> <div role='row'> <div role='columnheader' id='test' aria-sort='ascending'>content</div> </div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Object Attribute: sort:ascending

    node = atspi.find_node("test", session.url)
    assert "sort:ascending" in atspi.Accessible.get_attributes_as_array(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXSortDirection: AXAscendingSortDirection

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Object Attribute: sort:ascending

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AriaProperties.sort: ascending
#     # Property: ItemStatus: ascending if the element maps to HeaderItem Control Type
