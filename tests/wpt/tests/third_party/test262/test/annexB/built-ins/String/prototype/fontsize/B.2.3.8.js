// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

// Tests taken from:
// http://mathias.html5.org/tests/javascript/string/

/*---
description: >
    String.prototype.fontsize returns a string of HTML describing a single
    HTML font element. The element's content is the `this` value of the
    function invocation, coerced to a string. If specified, the first argument
    will be coerced to a string, escaped, and set as the element's `size`
    attribute.
es6id: B.2.3.8
---*/

assert.sameValue('_'.fontsize('b'), '<font size="b">_</font>');
assert.sameValue('<'.fontsize('<'), '<font size="<"><</font>');
assert.sameValue('_'.fontsize(0x2A), '<font size="42">_</font>');
assert.sameValue('_'.fontsize('\x22'), '<font size="&quot;">_</font>');
assert.sameValue(String.prototype.fontsize.call(0x2A, 0x2A),
  '<font size="42">42</font>');
assert.throws(TypeError, function() {
  String.prototype.fontsize.call(undefined);
});
assert.throws(TypeError, function() {
  String.prototype.fontsize.call(null);
});
