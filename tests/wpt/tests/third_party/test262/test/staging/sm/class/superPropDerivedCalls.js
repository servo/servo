// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
let derivedInstance;

class base {
    constructor() { }
    method(a, b, c) {
        assert.sameValue(this, derivedInstance);
        this.methodCalled = true;
        assert.sameValue(a, 1);
        assert.sameValue(b, 2);
        assert.sameValue(c, 3);
    }

    get prop() {
        assert.sameValue(this, derivedInstance);
        this.getterCalled = true;
        return this._prop;
    }

    set prop(val) {
        assert.sameValue(this, derivedInstance);
        this.setterCalled = true;
        this._prop = val;
    }
}

class derived extends base {
    constructor() { super(); }

    // |super| actually checks the chain, not |this|
    method() { throw "FAIL"; }
    get prop() { throw "FAIL"; }
    set prop(v) { throw "FAIL"; }

    test() {
        this.reset();
        // While we're here. Let's check on super spread calls...
        let spread = [1,2,3];
        super.method(...spread);
        super.prop++;
        this.asserts();
    }

    testInEval() {
        this.reset();
        eval("super.method(1,2,3); super.prop++");
        this.asserts();
    }

    testInArrow() {
        this.reset();
        (() => (super.method(1,2,3), super.prop++))();
        this.asserts();
    }

    reset() {
        this._prop = 0;
        this.methodCalled = false;
        this.setterCalled = false;
        this.getterCalled = false;
    }

    asserts() {
        assert.sameValue(this.methodCalled, true);
        assert.sameValue(this.getterCalled, true);
        assert.sameValue(this.setterCalled, true);
        assert.sameValue(this._prop, 1);
    }

}

derivedInstance = new derived();
derivedInstance.test();
derivedInstance.testInEval();
derivedInstance.testInArrow();

