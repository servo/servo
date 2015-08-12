// |expected| should be an object indicating the expected type of node.
function assert_node(actual, expected)
{
    assert_true(actual instanceof expected.type,
                'Node type mismatch: actual = ' + actual.nodeType + ', expected = ' + expected.nodeType);
    if (typeof(expected.id) !== 'undefined')
        assert_equals(actual.id, expected.id);
    if (typeof(expected.nodeValue) !== 'undefined')
        assert_equals(actual.nodeValue, expected.nodeValue);
}
