// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncdisposablestack.prototype.adopt
description: Allows any 'value'
info: |
  AsyncDisposableStack.prototype.adopt ( value, onDisposeAsync )

  1. Let asyncDisposableStack be the this value.
  2. Perform ? RequireInternalSlot(asyncDisposableStack, [[AsyncDisposableState]]).
  3. If asyncDisposableStack.[[AsyncDisposableState]] is disposed, throw a ReferenceError exception.
  4. If IsCallable(onDisposeAsync) is false, throw a TypeError exception.
  5. Let closure be a new Abstract Closure with no parameters that captures value and onDisposeAsync and performs the following steps when called:
    a. Perform ? Call(onDisposeAsync, undefined, « value »).
  6. Let F be CreateBuiltinFunction(closure, 0, "", « »).
  7. Perform ? AddDisposableResource(asyncDisposableStack.[[DisposeCapability]], undefined, async-dispose, F).
  ...

features: [explicit-resource-management]
---*/

var stack = new AsyncDisposableStack();
stack.adopt(null, async _ => {});
stack.adopt(undefined, async _ => {});
stack.adopt({}, async _ => {});
stack.adopt({ async [Symbol.asyncDispose]() {} }, async _ => {});
stack.adopt(() => {}, async _ => {});
stack.adopt(true, async _ => {});
stack.adopt(false, async _ => {});
stack.adopt(1, async _ => {});
stack.adopt('object', async _ => {});
stack.adopt(Symbol(), async _ => {});
