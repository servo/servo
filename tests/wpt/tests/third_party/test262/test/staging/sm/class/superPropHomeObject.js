// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  pending
esid: pending
---*/
// This is super weird. A super property reference in the spec contains two
// things. The first is the object to do the lookup on, the super base. This
// should be unchanged, no matter what's going on: I can move the method to
// another object. I can pull it out as its own function. I can put it on my
// head and run around the front yard. No changes. The other half, the |this|
// for invoked calls, is the this at the time of referencing the property, which
// means it's gonna vary wildly as stuff gets moved around.

class base {
    constructor() { }
    test(expectedThis) { assert.sameValue(this, expectedThis); }
}

class derived extends base {
    constructor() { super(); }
    test(expected) { super.test(expected); }
    testArrow() { return (() => super.test(this)); }
    ["testCPN"](expected) { super.test(expected); }
}

let derivedInstance = new derived();
derivedInstance.test(derivedInstance);
derivedInstance.testCPN(derivedInstance);

let obj = { test: derivedInstance.test };
obj.test(obj);

// Classes are strict, so primitives are not boxed/turned into globals
let testSolo = derivedInstance.test;
testSolo(undefined);

let anotherObject = { };
derivedInstance.test.call(anotherObject, anotherObject);

let strThis = "this is not an object!";
derivedInstance.test.call(strThis, strThis);

// You can take the arrow function out of the super, ... or something like that
let arrowTest = derivedInstance.testArrow();
arrowTest();

// There's no magic "super script index" per code location.
class base1 {
    constructor() { }
    test() { return "llama"; }
}
class base2 {
    constructor() { }
    test() { return "alpaca"; }
}

let animals = [];
for (let exprBase of [base1, base2])
    new class extends exprBase {
        constructor() { super(); }
        test() { animals.push(super["test"]()); }
    }().test();
assert.compareArray(animals, ["llama", "alpaca"]);
