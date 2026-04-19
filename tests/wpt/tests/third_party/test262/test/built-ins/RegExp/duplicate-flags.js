// Copyright 2017 the V8 project authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    RegExpInitialize ( obj, pattern, flags )
      5. If F contains any code unit other than "g", "i", "m", "s", "u", or "y" or if it contains the same code unit more than once, throw a SyntaxError exception.
esid: sec-regexpinitialize
description: Check that duplicate RegExp flags are disallowed
features: [regexp-dotall, regexp-match-indices]
---*/

new RegExp("", "mig"); // single g will not throw SyntaxError
assert.throws(SyntaxError, () => new RegExp("", "migg"), "duplicate g");

new RegExp("", "i"); // single i will not throw SyntaxError
assert.throws(SyntaxError, () => new RegExp("", "ii"), "duplicate i");

new RegExp("", "m"); // single m will not throw SyntaxError
assert.throws(SyntaxError, () => new RegExp("", "mm"), "duplicate m");

new RegExp("", "s"); // single s will not throw SyntaxError
assert.throws(SyntaxError, () => new RegExp("", "ss"), "duplicate s");

new RegExp("", "u"); // single u will not throw SyntaxError
assert.throws(SyntaxError, () => new RegExp("", "uu"), "duplicate u");

new RegExp("", "y"); // single y will not throw SyntaxError
assert.throws(SyntaxError, () => new RegExp("", "yy"), "duplicate y");

new RegExp("", "d"); // single d will not throw SyntaxError
assert.throws(SyntaxError, () => new RegExp("", "dd"), "duplicate d");
