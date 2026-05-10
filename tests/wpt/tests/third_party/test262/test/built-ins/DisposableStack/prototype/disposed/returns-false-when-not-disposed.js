// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-disposablestack.prototype.disposed
description: >
  Returns `false` when the DisposableStack has not yet been disposed.
info: |
  get DisposableStack.prototype.disposed

  ...
  3. If disposableStack.[[DisposableState]] is disposed, return true.
  4. Otherwise, return false.

features: [explicit-resource-management]
---*/

var stack = new DisposableStack();

assert.sameValue(stack.disposed, false);
