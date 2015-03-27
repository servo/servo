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

// XXX Servo doesn't have these constants in NodeFilter yet
var FILTER_ACCEPT = 1;
var FILTER_REJECT = 2;
var FILTER_SKIP = 3;
var SHOW_ALL = 0xFFFFFFFF;
var SHOW_ELEMENT = 0x1;
var SHOW_ATTRIBUTE = 0x2;
var SHOW_TEXT = 0x4;
var SHOW_CDATA_SECTION = 0x8;
var SHOW_ENTITY_REFERENCE = 0x10;
var SHOW_ENTITY = 0x20;
var SHOW_PROCESSING_INSTRUCTION = 0x40;
var SHOW_COMMENT = 0x80;
var SHOW_DOCUMENT = 0x100;
var SHOW_DOCUMENT_TYPE = 0x200;
var SHOW_DOCUMENT_FRAGMENT = 0x400;
var SHOW_NOTATION = 0x800;
