// Copyright (C) 2026 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncdisposablestack.prototype.defer
description: Throws a ReferenceError if this is disposed.
info: |
  AsyncDisposableStack.prototype.defer ( onDisposeAsync )

  1. Let asyncDisposableStack be the this value.
  2. Perform ? RequireInternalSlot(asyncDisposableStack, [[AsyncDisposableState]]).
  3. If asyncDisposableStack.[[AsyncDisposableState]] is disposed, throw a ReferenceError exception.
  ...

features: [explicit-resource-management]
---*/

var stack = new AsyncDisposableStack();
stack.disposeAsync();

assert.throws(ReferenceError, function() {
  stack.defer(async _ => {});
});
