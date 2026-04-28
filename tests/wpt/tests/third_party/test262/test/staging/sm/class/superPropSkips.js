// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Ensure that super lookups and sets skip over properties on the |this| object.
// That is, super lookups start with the superclass, not the current class.

// The whole point: an empty superclass
class base {
    constructor() { }
}

class derived extends base {
    constructor() { super(); this.prop = "flamingo"; }

    toString() { throw "No!"; }

    testSkipGet() {
        assert.sameValue(super.prop, undefined);
    }

    testSkipDerivedOverrides() {
        assert.sameValue(super["toString"](), Object.prototype.toString.call(this));
    }

    testSkipSet() {
        // since there's no prop on the chain, we should set the data property
        // on the receiver, |this|
        super.prop = "rat";
        assert.sameValue(this.prop, "rat");

        // Since the receiver is the instance, we can overwrite inherited
        // properties of the instance, even non-writable ones, as they could be
        // skipped in the super lookup.
        assert.sameValue(this.nonWritableProp, "pony");
        super.nonWritableProp = "bear";
        assert.sameValue(this.nonWritableProp, "bear");
    }
}

Object.defineProperty(derived.prototype, "nonWritableProp", { writable: false, value: "pony" });

let instance = new derived();
instance.testSkipGet();
instance.testSkipDerivedOverrides();
instance.testSkipSet();

