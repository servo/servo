// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
function makeProxyPrototype(target) {
    return Object.setPrototypeOf(target, new Proxy({}, new Proxy({
        getPrototypeOf() {
            return null;
        },
        ownKeys() {
            return [];
        },
        get(t, pk, r) {
            throw new Error("Unexpected [[Get]]: " + String(pk));
        }
    }, {
        get(t, pk, r) {
            if (pk in t)
                return Reflect.get(t, pk, r);
            throw new Error("Unexpected trap called: " + pk);
        }
    })));
}

function enumerateMappedArgs(x) {
    var a = makeProxyPrototype(arguments);

    // Delete all lazy properties and ensure no [[Has]] trap is called for them
    // on the prototype chain.
    delete a.length;
    delete a.callee;
    delete a[Symbol.iterator];
    delete a[0];

    for (var k in a);
}
enumerateMappedArgs(0);

function enumerateUnmappedArgs(x) {
    "use strict";
    var a = makeProxyPrototype(arguments);

    delete a.length;
    // delete a.callee; // .callee is non-configurable
    delete a[Symbol.iterator];
    delete a[0];

    for (var k in a);
}
enumerateUnmappedArgs(0);

function enumerateFunction() {
    var f = makeProxyPrototype(function named() {});

    // delete f.prototype; // .prototype is non-configurable
    delete f.length;
    delete f.name;

    for (var k in f);
}
enumerateFunction();


