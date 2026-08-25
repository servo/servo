/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
let count = 0;
let verifyProxy = new Proxy({}, {
    defineProperty(target, property, descriptor) {
        assert.sameValue(property, "x");

        assert.sameValue(descriptor.enumerable, true);
        assert.sameValue(descriptor.configurable, true);

        if ("set" in descriptor)
            assert.sameValue(descriptor.set, Object.prototype.__defineSetter__);
        else
            assert.sameValue(descriptor.get, Object.prototype.__defineGetter__);

        assert.sameValue(Object.keys(descriptor).length, 3);

        count++;
        return true;
    }
});

for (let define of [Object.prototype.__defineGetter__, Object.prototype.__defineSetter__]) {
    // null/undefined |this| value
    for (let thisv of [undefined, null])
        assert.throws(TypeError, () => define.call(thisv, "x", define));

    // non-callable getter/setter
    let nonCallable = [{}, [], new Proxy({}, {})];
    for (let value of nonCallable)
        assert.throws(TypeError, () => define.call(verifyProxy, "x", value));

    // ToPropertyKey
    let key = {
        [Symbol.toPrimitive](hint) {
            assert.sameValue(hint, "string");
            // Throws, because non-primitive is returned
            return {};
        },
        valueOf() { throw "wrongly invoked"; },
        toString() { throw "wrongly invoked"; }
    };
    assert.throws(TypeError, () => define.call(verifyProxy, key, define));

    key = {
        [Symbol.toPrimitive](hint) {
            assert.sameValue(hint, "string");
            return "x";
        },
        valueOf() { throw "wrongly invoked"; },
        toString() { throw "wrongly invoked"; }
    }
    define.call(verifyProxy, key, define);

    key = {
        [Symbol.toPrimitive]: undefined,

        valueOf() { throw "wrongly invoked"; },
        toString() { return "x"; }
    }
    define.call(verifyProxy, key, define);

    // Bog standard call
    define.call(verifyProxy, "x", define);

    let obj = {};
    define.call(obj, "x", define);
    let descriptor = Object.getOwnPropertyDescriptor(obj, "x");

    assert.sameValue(descriptor.enumerable, true);
    assert.sameValue(descriptor.configurable, true);

    if (define == Object.prototype.__defineSetter__) {
        assert.sameValue(descriptor.get, undefined);
        assert.sameValue(descriptor.set, define);
    } else {
        assert.sameValue(descriptor.get, define);
        assert.sameValue(descriptor.set, undefined);
    }

    assert.sameValue(Object.keys(descriptor).length, 4);


}

// Number of calls that should succeed
assert.sameValue(count, 6);

