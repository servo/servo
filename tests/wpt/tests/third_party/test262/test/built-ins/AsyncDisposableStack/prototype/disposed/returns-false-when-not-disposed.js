// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-asyncdisposablestack.prototype.disposed
description: >
  Returns `false` when the AsyncDisposableStack has not yet been disposed.
info: |
  get AsyncDisposableStack.prototype.disposed

  ...
  3. If asyncDisposableStack.[[AsyncDisposableState]] is disposed, return true.
  4. Otherwise, return false.

features: [explicit-resource-management]
---*/

var stack = new AsyncDisposableStack();

assert.sameValue(stack.disposed, false);
