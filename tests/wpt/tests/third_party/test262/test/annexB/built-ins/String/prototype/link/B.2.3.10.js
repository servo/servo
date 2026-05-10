// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

// Tests taken from:
// http://mathias.html5.org/tests/javascript/string/

/*---
description: >
    String.prototype.link returns a string of HTML describing a single HTML
    link element. The element's content is the `this` value of the function
    invocation, coerced to a string. If specified, the first argument will be
    coerced to a string, escaped, and set as the element's `href` attribute.
es6id: B.2.3.10
---*/

assert.sameValue('_'.link('b'), '<a href="b">_</a>');
assert.sameValue('<'.link('<'), '<a href="<"><</a>');
assert.sameValue('_'.link(0x2A), '<a href="42">_</a>');
assert.sameValue('_'.link('\x22'), '<a href="&quot;">_</a>');
assert.sameValue(String.prototype.link.call(0x2A, 0x2A), '<a href="42">42</a>');
assert.throws(TypeError, function() {
  String.prototype.link.call(undefined);
});
assert.throws(TypeError, function() {
  String.prototype.link.call(null);
});
