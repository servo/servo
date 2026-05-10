// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
class base {
    constructor() { }
    method() { this.methodCalled++; }
}

class derived extends base {
    constructor() { super(); this.methodCalled = 0; }

    // Test orderings of various evaluations relative to the superbase

    // Unlike in regular element evaluation, the propVal is evaluated before
    // checking the starting object ([[HomeObject]].[[Prototype]])
    testElem() { super[ruin()]; }

    // The starting object for looking up super.method is determined before
    // ruin() is called.
    testProp() { super.method(ruin()); }

    // The entire super.method property lookup has concluded before the args
    // are evaluated
    testPropCallDeleted() { super.method(()=>delete base.prototype.method); }

    // The starting object for looking up super["prop"] is determined before
    // ruin() is called.
    testElemAssign() { super["prop"] = ruin(); }

    // Test the normal assignment gotchas
    testAssignElemPropValChange() {
        let x = "prop1";
        super[x] = (()=>(x = "prop2", 0))();
        assert.sameValue(this.prop1, 0);
        assert.sameValue(this.prop2, undefined);
    }

    testAssignProp() {
        Object.defineProperty(base.prototype, "piggy",
            {
              configurable: true,
              set() { throw "WEE WEE WEE WEE"; }
            });

        // The property lookup is noted, but not actually evaluated, until the
        // right hand side is. Please don't make the piggy cry.
        super.piggy = (() => delete base.prototype.piggy)();
    }
    testCompoundAssignProp() {
        let getterCalled = false;
        Object.defineProperty(base.prototype, "horse",
            {
              configurable: true,
              get() { getterCalled = true; return "Of course"; },
              set() { throw "NO!"; }
            });
        super.horse += (()=>(delete base.prototype.horse, ", of course!"))();
        assert.sameValue(getterCalled, true);

        // So, is a horse a horse?
        assert.sameValue(this.horse, "Of course, of course!");
    }
}

function ruin() {
    Object.setPrototypeOf(derived.prototype, null);
    return 5;
}

function reset() {
    Object.setPrototypeOf(derived.prototype, base.prototype);
}

let instance = new derived();
assert.throws(TypeError, () => instance.testElem());
reset();

instance.testProp();
assert.sameValue(instance.methodCalled, 1);
reset();

instance.testPropCallDeleted();
assert.sameValue(instance.methodCalled, 2);

instance.testElemAssign();
assert.sameValue(instance.prop, 5);
reset();

instance.testAssignElemPropValChange();

instance.testAssignProp();

instance.testCompoundAssignProp();

