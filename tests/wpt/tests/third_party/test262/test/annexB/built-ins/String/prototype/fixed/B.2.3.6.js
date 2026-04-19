// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

// Tests taken from:
// http://mathias.html5.org/tests/javascript/string/

/*---
description: >
    String.prototype.fixed returns a string of HTML describing a single HTML
    teletype text element. The element's content is the `this` value of the
    function invocation, coerced to a string.
es6id: B.2.3.6
---*/

assert.sameValue('_'.fixed(), '<tt>_</tt>');
assert.sameValue('<'.fixed(), '<tt><</tt>');
assert.sameValue(String.prototype.fixed.call(0x2A), '<tt>42</tt>');
assert.throws(TypeError, function() {
  String.prototype.fixed.call(undefined);
});
assert.throws(TypeError, function() {
  String.prototype.fixed.call(null);
});
