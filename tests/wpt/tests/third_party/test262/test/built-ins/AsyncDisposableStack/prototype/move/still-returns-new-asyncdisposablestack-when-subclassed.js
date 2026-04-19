// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncdisposablestack.prototype.move
description: Returns a new %AsyncDisposableStack%, even when subclassed
info: |
  AsyncDisposableStack.prototype.move ( )

  1. Let asyncDisposableStack be the this value.
  2. Perform ? RequireInternalSlot(asyncDisposableStack, [[AsyncDisposableState]]).
  3. If asyncDisposableStack.[[AsyncDisposableState]] is disposed, throw a ReferenceError exception.
  4. Let newAsyncDisposableStack be ? OrdinaryCreateFromConstructor(%AsyncDisposableStack%, "%AsyncDisposableStack.prototype%", « [[AsyncDisposableState]], [[DisposeCapability]] »).
  5. Set newAsyncDisposableStack.[[AsyncDisposableState]] to pending.
  6. Set newAsyncDisposableStack.[[DisposeCapability]] to asyncDisposableStack.[[DisposeCapability]].
  7. Set asyncDisposableStack.[[DisposeCapability]] to NewDisposeCapability().
  8. Set asyncDisposableStack.[[AsyncDisposableState]] to disposed.
  9. Return newAsyncDisposableStack.

features: [explicit-resource-management]
---*/

class MyAsyncDisposableStack extends AsyncDisposableStack {}

var stack1 = new MyAsyncDisposableStack();
var stack2 = stack1.move();
assert(stack2 instanceof AsyncDisposableStack, 'Expected stack2 to be an instance of AsyncDisposableStack');
assert(!(stack2 instanceof MyAsyncDisposableStack), 'Expected stack2 to not be an instance of MyAsyncDisposableStack');
