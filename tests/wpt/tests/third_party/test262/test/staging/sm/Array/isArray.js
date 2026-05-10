/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
var global = this;
var otherGlobal = $262.createRealm().global;

var thisGlobal = () => global;
var alternateGlobals = (function(i) {
    return () => (i++ % 2) === 0 ? global : otherGlobal;
})(0);

function performTests(pickGlobal)
{
    // Base case.
    assert.sameValue(Array.isArray([]), true);

    // Simple case: proxy to an array.
    var proxy = new (pickGlobal()).Proxy([], {});
    assert.sameValue(Array.isArray(proxy), true);

    // Recursive proxy ultimately terminating in an array.
    for (var i = 0; i < 10; i++) {
        proxy = new (pickGlobal()).Proxy(proxy, {});
        assert.sameValue(Array.isArray(proxy), true);
    }

    // Revocable proxy to an array.
    var revocable = (pickGlobal()).Proxy.revocable([], {});
    proxy = revocable.proxy;
    assert.sameValue(Array.isArray(proxy), true);

    // Recursive proxy ultimately terminating in a revocable proxy to an array.
    for (var i = 0; i < 10; i++) {
        proxy = new (pickGlobal()).Proxy(proxy, {});
        assert.sameValue(Array.isArray(proxy), true);
    }

    // Revoked proxy to (formerly) an array.
    revocable.revoke();
    assert.throws(TypeError, () => Array.isArray(revocable.proxy));

    // Recursive proxy ultimately terminating in a revoked proxy to an array.
    assert.throws(TypeError, () => Array.isArray(proxy));

}

performTests(thisGlobal);
performTests(alternateGlobals);

function crossGlobalTest()
{
    var array = new otherGlobal.Array();

    // Array from another global.
    assert.sameValue(Array.isArray(array), true);

    // Proxy to an array from another global.
    assert.sameValue(Array.isArray(new Proxy(array, {})), true);

    // Other-global proxy to an array from that selfsame global.
    assert.sameValue(Array.isArray(new otherGlobal.Proxy(array, {})), true);
}

crossGlobalTest();

