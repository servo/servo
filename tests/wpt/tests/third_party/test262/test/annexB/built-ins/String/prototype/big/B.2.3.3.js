// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

// Tests taken from:
// http://mathias.html5.org/tests/javascript/string/

/*---
description: >
    String.prototype.big returns a string of HTML describing a single HTML
    big element. The element's content is the `this` value of the function
    invocation, coerced to a string.
es6id: B.2.3.3
---*/

assert.sameValue('_'.big(), '<big>_</big>');
assert.sameValue('<'.big(), '<big><</big>');
assert.sameValue(String.prototype.big.call(0x2A), '<big>42</big>');
assert.throws(TypeError, function() {
  String.prototype.big.call(undefined);
});
assert.throws(TypeError, function() {
  String.prototype.big.call(null);
});
