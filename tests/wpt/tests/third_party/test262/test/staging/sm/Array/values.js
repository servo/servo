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
if (Array.prototype.values) {
    assert.sameValue(Array.prototype.values, Array.prototype[Symbol.iterator]);
    assert.sameValue(Array.prototype.values.name, "values");
    assert.sameValue(Array.prototype.values.length, 0);

    function valuesUnscopeable() {
        var values = "foo";
        with ([1, 2, 3]) {
            assert.sameValue(indexOf, Array.prototype.indexOf);
            assert.sameValue(values, "foo");
        }
    }
    valuesUnscopeable();
}

