# Testing: https://w3c.github.io/core-aam/#ariaDescribedBy

TEST_HTML = "<div role='group' id='test' aria-describedby='description'>content</div> <div id='description'>hello world</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Property: Description: <value>
    # Relation: RELATION_DESCRIBED_BY: points to accessible nodes matching IDREFs, if the referenced objects are in the accessibility tree
    # Reverse Relation: RELATION_DESCRIPTION_FOR: points to element

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_description(node) == "hello world"
    relations = atspi.get_relations_dictionary_helper(node)
    assert 'RELATION_DESCRIBED_BY' in relations
    assert 'description' in relations['RELATION_DESCRIBED_BY']
    reverse_node = atspi.find_node('description', session.url)
    reverse_relations = atspi.get_relations_dictionary_helper(reverse_node)
    assert 'RELATION_DESCRIPTION_FOR' in reverse_relations
    assert 'test' in reverse_relations['RELATION_DESCRIPTION_FOR']

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # <span class="api">In the accessibilityCustomContent API, expose as an <code>AXCustomContent</code> object with <code>{ label: "description" }</code> and `<code>value</code>` set to the description string.</span><br> - <span class="seealso">See also: <a href="#mapping_additional_nd">Name Computation</a></span>

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: accDescription: <value>
#     # Relation: IA2_RELATION_DESCRIBED_BY: points to accessible nodes matching IDREFs, if the referenced objects are in the accessibility tree
#     # Reverse Relation: IA2_RELATION_DESCRIPTION_FOR: points to element
#     # See also: Name Computation and Mapping Additional Relations

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: FullDescription: <value>
#     # See also: Name Computation
