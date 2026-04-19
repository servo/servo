// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
class base {
    constructor() { }
    static found() {
        this.foundCalled = true;
    }
    static get accessor() {
        assert.sameValue(this, derived);
        return 45;
    }
    notFound() { }
}

class derived extends base {
    constructor() { }

    static found() { throw "NO!"; }
    static get accessor() { throw "NO!"; }

    static test() {
        assert.sameValue(super["notFound"], undefined);
        super.found();

        // foundCalled is set on |derived| specifically.
        let calledDesc = Object.getOwnPropertyDescriptor(derived, "foundCalled");
        assert.sameValue(calledDesc.value, true);

        assert.sameValue(super.accessor, 45);
    }
}

derived.test();

