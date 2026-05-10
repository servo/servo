/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [sm/non262-TypedArray-shell.js]
description: |
  Implement %TypedArray%.prototype.{findLast, findLastIndex}
info: bugzilla.mozilla.org/show_bug.cgi?id=1704385
esid: pending
features: [Symbol]
---*/

const methods = ["findLast", "findLastIndex"];

anyTypedArrayConstructors.forEach(constructor => {
    methods.forEach(method => {
        var arr = new constructor([0, 1, 2, 3, 4, 5]);
        // test that this.length is never called
        Object.defineProperty(arr, "length", {
            get() {
                throw new Error("length accessor called");
            }
        });
        assert.sameValue(arr[method].length, 1);
        assert.sameValue(arr[method](v => v === 3), 3);
        assert.sameValue(arr[method](v => v === 6), method === "findLast" ? undefined : -1);

        var thisValues = [undefined, null, true, 1, "foo", [], {}, Symbol()];

        thisValues.forEach(thisArg =>
            assert.throws(TypeError, () => arr[method].call(thisArg, () => true))
        );

        assert.throws(TypeError, () => arr[method]());
        assert.throws(TypeError, () => arr[method](1));
    });
});

anyTypedArrayConstructors.filter(isFloatConstructor).forEach(constructor => {
    var arr = new constructor([-0, 0, 1, 5, NaN, 6]);
    assert.sameValue(arr.findLast(v => Number.isNaN(v)), NaN);
    assert.sameValue(arr.findLastIndex(v => Number.isNaN(v)), 4);

    assert.sameValue(arr.findLast(v => Object.is(v, 0)), 0);
    assert.sameValue(arr.findLastIndex(v => Object.is(v, 0)), 1);

    assert.sameValue(arr.findLast(v => Object.is(v, -0)), -0);
    assert.sameValue(arr.findLastIndex(v => Object.is(v, -0)), 0);
})
