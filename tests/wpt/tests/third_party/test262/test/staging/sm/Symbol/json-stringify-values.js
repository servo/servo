/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
// To JSON.stringify, symbols are the same as undefined.

var symbols = [
    Symbol(),
    Symbol.for("ponies"),
    Symbol.iterator
];

for (var sym of symbols) {
    assert.sameValue(JSON.stringify(sym), undefined);
    assert.sameValue(JSON.stringify([sym]), "[null]");

    // JSON.stringify skips symbol-valued properties!
    assert.sameValue(JSON.stringify({x: sym}), '{}');

    // However such properties are passed to the replacerFunction if any.
    var replacer = function (key, val) {
        assert.sameValue(typeof this, "object");
        if (typeof val === "symbol") {
            assert.sameValue(val, sym);
            return "ding";
        }
        return val;
    };
    assert.sameValue(JSON.stringify(sym, replacer), '"ding"');
    assert.sameValue(JSON.stringify({x: sym}, replacer), '{"x":"ding"}');
}

