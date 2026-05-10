// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncdisposablestack.prototype.use
description: Returns the argument provided.
info: |
  AsyncDisposableStack.prototype.use ( value )

  ...
  5. Return value.

features: [explicit-resource-management]
---*/

var stack = new AsyncDisposableStack();
var resource1 = { async [Symbol.asyncDispose]() { } };
var resource2 = { [Symbol.dispose]() { } };
assert.sameValue(stack.use(resource1), resource1);
assert.sameValue(stack.use(resource2), resource2);
assert.sameValue(stack.use(null), null);
assert.sameValue(stack.use(undefined), undefined);
