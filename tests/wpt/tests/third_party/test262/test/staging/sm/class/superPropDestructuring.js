// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  pending
esid: pending
---*/
class base {
    constructor() { }
}

let seenValues;
Object.defineProperty(base.prototype, "minutes",
                      {
                        set(x) {
                            assert.sameValue(x, 525600);
                            seenValues.push(x);
                        }
                      });
Object.defineProperty(base.prototype, "intendent",
                      {
                        set(x) {
                            assert.sameValue(x, "Fred");
                            seenValues.push(x)
                        }
                      });

const testArr = [525600, "Fred"];
class derived extends base {
    constructor() { super(); }
    prepForTest() { seenValues = []; }
    testAsserts() { assert.compareArray(seenValues, testArr); }
    testProps() {
        this.prepForTest();
        [super.minutes, super.intendent] = testArr;
        this.testAsserts();
    }
    testElems() {
        this.prepForTest();
        [super["minutes"], super["intendent"]] = testArr;
        this.testAsserts();
    }
}

let d = new derived();
d.testProps();
d.testElems();

