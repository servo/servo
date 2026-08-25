// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-disposablestack.prototype.use
description: Returns the argument provided.
info: |
  DisposableStack.prototype.use ( value )

  ...
  5. Return value.

features: [explicit-resource-management]
---*/

var stack = new DisposableStack();
var resource = { [Symbol.dispose]() { } };
assert.sameValue(stack.use(resource), resource);
assert.sameValue(stack.use(null), null);
assert.sameValue(stack.use(undefined), undefined);
