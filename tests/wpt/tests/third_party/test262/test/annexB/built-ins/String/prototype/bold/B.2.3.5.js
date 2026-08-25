// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

// Tests taken from:
// http://mathias.html5.org/tests/javascript/string/

/*---
description: >
    String.prototype.bold returns a string of HTML describing a single HTML
    bold element. The element's content is the `this` value of the function
    invocation, coerced to a string.
es6id: B.2.3.5
---*/

assert.sameValue('_'.bold(), '<b>_</b>');
assert.sameValue('<'.bold(), '<b><</b>');
assert.sameValue(String.prototype.bold.call(0x2A), '<b>42</b>');
assert.throws(TypeError, function() {
  String.prototype.bold.call(undefined);
});
assert.throws(TypeError, function() {
  String.prototype.bold.call(null);
});
