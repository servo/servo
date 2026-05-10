// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-disposablestack.prototype.adopt
description: Allows any 'value'
info: |
  DisposableStack.prototype.adopt ( value, onDispose )

  1. Let disposableStack be the this value.
  2. Perform ? RequireInternalSlot(disposableStack, [[DisposableState]]).
  3. If disposableStack.[[DisposableState]] is disposed, throw a ReferenceError exception.
  4. If IsCallable(onDispose) is false, throw a TypeError exception.
  5. Let closure be a new Abstract Closure with no parameters that captures value and onDispose and performs the following steps when called:
    a. Perform ? Call(onDispose, undefined, « value »).
  6. Let F be CreateBuiltinFunction(closure, 0, "", « »).
  7. Perform ? AddDisposableResource(disposableStack.[[DisposeCapability]], undefined, sync-dispose, F).
  ...

features: [explicit-resource-management]
---*/

var stack = new DisposableStack();
stack.adopt(null, _ => {});
stack.adopt(undefined, _ => {});
stack.adopt({}, _ => {});
stack.adopt({ [Symbol.dispose]() {} }, _ => {});
stack.adopt(() => {}, _ => {});
stack.adopt(true, _ => {});
stack.adopt(false, _ => {});
stack.adopt(1, _ => {});
stack.adopt('object', _ => {});
stack.adopt(Symbol(), _ => {});
