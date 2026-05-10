// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Make sure we get the proper side effects.
// |delete super[expr]| applies ToPropertyKey on |expr| before throwing.

class base {
    constructor() { }
}

class derived extends base {
    constructor() { super(); }
    testDeleteElem() {
        let sideEffect = 0;
        let key = {
            toString() {
                sideEffect++;
                return "";
            }
        };
        assert.throws(ReferenceError, () => delete super[key]);
        assert.sameValue(sideEffect, 0);
    }
}

class derivedTestDeleteElem extends base {
    constructor() {
        let sideEffect = 0;
        let key = {
            toString() {
                sideEffect++;
                return "";
            }
        };

        assert.throws(ReferenceError, () => delete super[key]);
        assert.sameValue(sideEffect, 0);

        super();

        assert.throws(ReferenceError, () => delete super[key]);
        assert.sameValue(sideEffect, 0);

        Object.setPrototypeOf(derivedTestDeleteElem.prototype, null);

        assert.throws(ReferenceError, () => delete super[key]);
        assert.sameValue(sideEffect, 0);

        return {};
    }
}

var d = new derived();
d.testDeleteElem();

new derivedTestDeleteElem();

