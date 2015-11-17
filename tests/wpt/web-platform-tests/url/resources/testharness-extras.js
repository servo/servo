/*
Extending the W3C testharness.js with locally useful functionality.
*/

function assert_type_error(f, msg) {
    assert_throws(TypeError(), f, msg);
}

function assert_syntax_error(f, msg) {
    assert_throws(SyntaxError(), f, msg);
}
