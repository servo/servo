// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

// Tests taken from:
// http://mathias.html5.org/tests/javascript/string/

/*---
description: >
    String.prototype.blink returns a string of HTML describing a single HTML
    blink element. The element's content is the `this` value of the function
    invocation, coerced to a string.
es6id: B.2.3.4
---*/

assert.sameValue('_'.blink(), '<blink>_</blink>');
assert.sameValue('<'.blink(), '<blink><</blink>');
assert.sameValue(String.prototype.blink.call(0x2A), '<blink>42</blink>');
assert.throws(TypeError, function() {
  String.prototype.blink.call(undefined);
});
assert.throws(TypeError, function() {
  String.prototype.blink.call(null);
});
