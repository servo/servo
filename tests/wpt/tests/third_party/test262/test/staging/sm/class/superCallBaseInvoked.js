// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
function testBase(base) {
    class instance extends base {
        constructor(inst, one) {
            super(inst, one);
        }
    }

    let inst = new instance(instance, 1);
    assert.sameValue(Object.getPrototypeOf(inst), instance.prototype);
    assert.sameValue(inst.calledBase, true);

    class defaultInstance extends base { }
    let defInst = new defaultInstance(defaultInstance, 1);
    assert.sameValue(Object.getPrototypeOf(defInst), defaultInstance.prototype);
    assert.sameValue(defInst.calledBase, true);
}

class base {
    // Base class must be [[Construct]]ed, as you cannot [[Call]] a class
    // constructor
    constructor(nt, one) {
        assert.sameValue(new.target, nt);

        // Check argument ordering
        assert.sameValue(one, 1);
        this.calledBase = true;
    }
}

testBase(base);
testBase(class extends base {
             constructor(nt, one) {
                 // Every step of the way, new.target and args should be right
                 assert.sameValue(new.target, nt);
                 assert.sameValue(one, 1);
                 super(nt, one);
             }
         });
function baseFunc(nt, one) {
    assert.sameValue(new.target, nt);
    assert.sameValue(one, 1);
    this.calledBase = true;
}

testBase(baseFunc);

let handler = {};
let p = new Proxy(baseFunc, handler);
testBase(p);

handler.construct = (target, args, nt) => Reflect.construct(target, args, nt);
testBase(p);

