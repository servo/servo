// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

// Tests taken from:
// http://mathias.html5.org/tests/javascript/string/

/*---
description: >
    String.prototype.sub returns a string of HTML describing a single HTML
    subscript element. The element's content is the `this` value of the
    function invocation, coerced to a string.
es6id: B.2.3.13
---*/

assert.sameValue('_'.sub(), '<sub>_</sub>');
assert.sameValue('<'.sub(), '<sub><</sub>');
assert.sameValue(String.prototype.sub.call(0x2A), '<sub>42</sub>');
assert.throws(TypeError, function() {
  String.prototype.sub.call(undefined);
});
assert.throws(TypeError, function() {
  String.prototype.sub.call(null);
});
