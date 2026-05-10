// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-TypedArray-shell.js, deepEqual.js]
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
for (var constructor of anyTypedArrayConstructors) {
    // The third argument to %TypedArray%.from is passed as the 'this' value to the
    // mapping function.
    var hits = 0, obj = {};
    function f(x) {
        assert.sameValue(this, obj);
        hits++;
    }
    constructor.from(["a", "b", "c"], f, obj);
    assert.sameValue(hits, 3);

    // Without an argument, undefined is passed...
    hits = 0;
    function gs(x) {
        "use strict";
        assert.sameValue(this, undefined);
        hits++;
    }
    constructor.from("def", gs);
    assert.sameValue(hits, 3);

    // ...and if the mapping function is non-strict, that means the global is
    // passed.
    var global = this;
    hits = 0;
    function g(x) {
        assert.sameValue(this, global);
        hits++;
    }
    constructor.from("ghi", g);
    assert.sameValue(hits, 3);

    // A primitive value can be passed.
    for (var v of [0, "str", undefined]) {
        hits = 0;
        var mapfn = function h(x) {
            "use strict";
            assert.sameValue(this, v);
            hits++;
        };
        constructor.from("pq", mapfn, v);
        assert.sameValue(hits, 2);
    }

    // ...and if the mapping function is non-strict, primitive values will
    // be wrapped to objects.
    for (var v of [0, "str", true]) {
        hits = 0;
        var mapfn = function h(x) {
            assert.deepEqual(this, Object(v));
            hits++;
        };
        constructor.from("pq", mapfn, v);
        assert.sameValue(hits, 2);
    }
}

