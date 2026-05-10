// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-disposablestack.prototype.move
description: Throws a ReferenceError if this is disposed.
info: |
  DisposableStack.prototype.move ( value )

  1. Let disposableStack be the this value.
  2. Perform ? RequireInternalSlot(disposableStack, [[DisposableState]]).
  3. If disposableStack.[[DisposableState]] is disposed, throw a ReferenceError exception.
  ...

features: [explicit-resource-management]
---*/

var stack = new DisposableStack();
stack.dispose();

assert.throws(ReferenceError, function() {
  stack.move();
}, 'undefined');
