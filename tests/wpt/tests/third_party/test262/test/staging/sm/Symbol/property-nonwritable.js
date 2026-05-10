/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
var sym = Symbol.for("moon");
function checkNotWritable(obj) {
    // In sloppy mode, assigning to a nonwritable property silently fails.
    obj[sym] = "portals";
    assert.sameValue(obj[sym], "cheese");

    // In strict mode code, it throws.
    assert.throws(TypeError, function () { "use strict"; obj[sym] = "robots"; });
    assert.sameValue(obj[sym], "cheese");
}

var x = {};
Object.defineProperty(x, sym, {
    configurable: true,
    enumerable: true,
    value: "cheese",
    writable: false
});

checkNotWritable(x);

// Assignment can't shadow inherited nonwritable properties either.
var y = Object.create(x);
checkNotWritable(y);
checkNotWritable(Object.create(y));

