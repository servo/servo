// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

// Tests taken from:
// http://mathias.html5.org/tests/javascript/string/

/*---
description: >
    String.prototype.fontcolor returns a string of HTML describing a single
    HTML font element. The element's content is the `this` value of the
    function invocation, coerced to a string. If specified, the first argument
    will be coerced to a string, escaped, and set as the element's `color`
    attribute.
es6id: B.2.3.7
---*/

assert.sameValue('_'.fontcolor('b'), '<font color="b">_</font>');
assert.sameValue('<'.fontcolor('<'), '<font color="<"><</font>');
assert.sameValue('_'.fontcolor(0x2A), '<font color="42">_</font>');
assert.sameValue('_'.fontcolor('\x22'), '<font color="&quot;">_</font>');
assert.sameValue(String.prototype.fontcolor.call(0x2A, 0x2A),
  '<font color="42">42</font>');
assert.throws(TypeError, function() {
  String.prototype.fontcolor.call(undefined);
});
assert.throws(TypeError, function() {
  String.prototype.fontcolor.call(null);
});
