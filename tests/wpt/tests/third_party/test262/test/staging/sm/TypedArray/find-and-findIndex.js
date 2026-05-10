/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [sm/non262-TypedArray-shell.js]
description: |
  Implement %TypedArray%.prototype.{find, findIndex}
info: bugzilla.mozilla.org/show_bug.cgi?id=1078975
esid: pending
features: [Symbol]
---*/

const methods = ["find", "findIndex"];

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
        assert.sameValue(arr[method](v => v === 6), method === "find" ? undefined : -1);

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
    assert.sameValue(arr.find(v => Number.isNaN(v)), NaN);
    assert.sameValue(arr.findIndex(v => Number.isNaN(v)), 4);

    assert.sameValue(arr.find(v => Object.is(v, 0)), 0);
    assert.sameValue(arr.findIndex(v => Object.is(v, 0)), 1);

    assert.sameValue(arr.find(v => Object.is(v, -0)), -0);
    assert.sameValue(arr.findIndex(v => Object.is(v, -0)), 0);
})
