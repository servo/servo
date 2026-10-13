# Testing: https://w3c.github.io/core-aam/#ariaLabelledBy

TEST_HTML = "<div role='group' id='test' aria-labelledby='label'>content</div> <div id='label'>hello world</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Property: Name: <value>
    # Relation: RELATION_LABELLED_BY: points to accessible nodes matching IDREFs, if the referenced objects are in the accessibility tree
    # Reverse Relation: RELATION_LABEL_FOR: points to element

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_name(node) == "hello world"
    relations = atspi.get_relations_dictionary_helper(node)
    assert 'RELATION_LABELLED_BY' in relations
    assert 'label' in relations['RELATION_LABELLED_BY']
    reverse_node = atspi.find_node('label', session.url)
    reverse_relations = atspi.get_relations_dictionary_helper(reverse_node)
    assert 'RELATION_LABEL_FOR' in reverse_relations
    assert 'test' in reverse_relations['RELATION_LABEL_FOR']

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXTitle: <value>
#     # Property: AXTitleUIElement: points to accessible node matching IDREF, if there is a single referenced element that is in the accessibility tree
#     # See also: Name Computation

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: accName: <value>
#     # Relation: IA2_RELATION_LABELLED_BY: points to accessible nodes matching IDREFs, if the referenced objects are in the accessibility tree
#     # Reverse Relation: IA2_RELATION_LABEL_FOR: points to element
#     # See also: Name Computation and Mapping Additional Relations

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: Name: <value>
#     # Property: LabeledBy: points to accessible nodes matching IDREFs, if the referenced objects are in the accessibility tree
#     # See also: Name Computation
