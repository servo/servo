// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Boolean coercion of `unicode` property
es6id: 21.2.5.8
info: |
    21.2.5.6 RegExp.prototype [ @@replace ] ( string )

    [...]
    10. If global is true, then
        a. Let fullUnicode be ToBoolean(Get(rx, "unicode")).
    [...]
features: [Symbol.replace]
---*/

var r = /^|\udf06/g;
Object.defineProperty(r, 'unicode', { writable: true });

r.unicode = undefined;
assert.sameValue(r[Symbol.replace]('\ud834\udf06', 'XXX'), 'XXX\ud834XXX');

r.unicode = null;
assert.sameValue(r[Symbol.replace]('\ud834\udf06', 'XXX'), 'XXX\ud834XXX');

r.unicode = false;
assert.sameValue(r[Symbol.replace]('\ud834\udf06', 'XXX'), 'XXX\ud834XXX');

r.unicode = NaN;
assert.sameValue(r[Symbol.replace]('\ud834\udf06', 'XXX'), 'XXX\ud834XXX');

r.unicode = 0;
assert.sameValue(r[Symbol.replace]('\ud834\udf06', 'XXX'), 'XXX\ud834XXX');

r.unicode = '';
assert.sameValue(r[Symbol.replace]('\ud834\udf06', 'XXX'), 'XXX\ud834XXX');

r.unicode = true;
assert.sameValue(r[Symbol.replace]('\ud834\udf06', 'XXX'), 'XXX\ud834\udf06');

r.unicode = 86;
assert.sameValue(r[Symbol.replace]('\ud834\udf06', 'XXX'), 'XXX\ud834\udf06');

r.unicode = Symbol.replace;
assert.sameValue(r[Symbol.replace]('\ud834\udf06', 'XXX'), 'XXX\ud834\udf06');

r.unicode = {};
assert.sameValue(r[Symbol.replace]('\ud834\udf06', 'XXX'), 'XXX\ud834\udf06');
