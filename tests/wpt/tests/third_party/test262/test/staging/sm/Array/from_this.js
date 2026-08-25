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
// The third argument to Array.from is passed as the 'this' value to the
// mapping function.
var hits = 0, obj = {};
function f(x) {
    assert.sameValue(this, obj);
    hits++;
}
Array.from(["a", "b", "c"], f, obj);
assert.sameValue(hits, 3);

// Without an argument, undefined is passed...
hits = 0;
function gs(x) {
    "use strict";
    assert.sameValue(this, undefined);
    hits++;
}
Array.from("def", gs);
assert.sameValue(hits, 3);

// ...and if the mapping function is non-strict, that means the global is
// passed.
var global = this;
hits = 0;
function g(x) {
    assert.sameValue(this, global);
    hits++;
}
Array.from("ghi", g);
assert.sameValue(hits, 3);

// A primitive value can be passed.
for (var v of [0, "str", undefined]) {
    hits = 0;
    var mapfn = function h(x) {
        "use strict";
        assert.sameValue(this, v);
        hits++;
    };
    Array.from("pq", mapfn, v);
    assert.sameValue(hits, 2);
}

