// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// basic static method test
class X {
    static count() { return ++this.hits; }
    constructor() { }
}
X.hits = 0;
assert.sameValue(X.count(), 1);

// A static method is just a function.
assert.sameValue(X.count instanceof Function, true);
assert.sameValue(X.count.length, 0);
assert.sameValue(X.count.bind({hits: 77})(), 78);

