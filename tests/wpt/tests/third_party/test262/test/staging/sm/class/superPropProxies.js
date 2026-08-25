// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
class base {
    constructor() { }
}

let midStaticHandler = { };

// We shouldn't use the |this.called| strategy here, since we have proxies
// snooping property accesses.
let getterCalled, setterCalled;

class mid extends new Proxy(base, midStaticHandler) {
    constructor() { super(); }
    testSuperInProxy() {
        super.prop = "looking";
        assert.sameValue(setterCalled, true);
        assert.sameValue(super.prop, "found");
        assert.sameValue(getterCalled, true);
    }
}

class child extends mid {
    constructor() { super(); }
    static testStaticLookups() {
        // This funtion is called more than once.
        this.called = false;
        super.prop;
        assert.sameValue(this.called, true);
    }
}

let midInstance = new mid();

// Make sure proxies are searched properly on the prototype chain
let baseHandler = {
    get(target, p, receiver) {
        assert.sameValue(receiver, midInstance);
        getterCalled = true;
        return "found";
    },

    set(t,p,val,receiver) {
        assert.sameValue(receiver, midInstance);
        assert.sameValue(val, "looking");
        setterCalled = true;
        return true;
    }
}
Object.setPrototypeOf(base.prototype, new Proxy(Object.prototype, baseHandler));

// make sure subclasses are not searched on static or super lookups.
let childHandler = {
    get() { throw "NO!"; },
    set() { throw "NO!"; }
}
Object.setPrototypeOf(child.prototype, new Proxy(mid.prototype, childHandler));

midInstance.testSuperInProxy();

// Don't do this earlier to avoid the lookup of .prototype during class creation
function midGet(target, p, receiver) {
    assert.sameValue(receiver, child);
    receiver.called = true;
}
midStaticHandler.get = midGet;

child.testStaticLookups();

// Hey does super work in a proxy?
assert.sameValue(new Proxy(({ method() { return super.hasOwnProperty("method"); } }), {}).method(), true);

// What about a CCW?
var g = $262.createRealm().global;
var wrappedSuper = g.eval("({ method() { return super.hasOwnProperty('method'); } })");
assert.sameValue(wrappedSuper.method(), true);

// With a CCW on the proto chain?
var wrappedBase = g.eval("({ method() { return this.__secretProp__; } })");
var unwrappedDerived = { __secretProp__: 42, method() { return super.method(); } }
Object.setPrototypeOf(unwrappedDerived, wrappedBase);
assert.sameValue(unwrappedDerived.method(), 42);

