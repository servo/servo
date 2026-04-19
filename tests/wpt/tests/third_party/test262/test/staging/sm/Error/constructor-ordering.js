// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
var order = 0;
function assertOrdering(ordering) {
    assert.sameValue(order, ordering);
    order++;
}

// Spec mandates that the prototype is looked up /before/ we toString the
// argument.
var handler = { get() { assertOrdering(0); return Error.prototype } };
var errorProxy = new Proxy(Error, handler);

var toStringable = { toString() { assertOrdering(1); return "Argument"; } };

new errorProxy(toStringable);

