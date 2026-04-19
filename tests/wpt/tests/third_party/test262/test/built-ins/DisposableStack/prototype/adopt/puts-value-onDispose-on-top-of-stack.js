// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-disposablestack.prototype.adopt
description: Adds a disposable resource to the stack
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

  AddDisposableResource ( disposeCapability, V, hint [, method ] )

  1. If method is not present then,
    ...
  2. Else,
    a. Assert: V is undefined.
    b. Let resource be ? CreateDisposableResource(undefined, hint, method).
  3. Append resource to disposeCapability.[[DisposableResourceStack]].
  4. Return unused.

features: [explicit-resource-management]
---*/

var stack = new DisposableStack();
var disposed = [];
var resource1 = {};
function dispose1(res) { disposed.push([res, dispose1]); }
var resource2 = {};
function dispose2(res) { disposed.push([res, dispose2]); }
stack.adopt(resource1, dispose1);
stack.adopt(resource2, dispose2);
stack.dispose();
assert.sameValue(2, disposed.length);
assert.sameValue(disposed[0][0], resource2, 'Expected resource2 to be the first disposed resource');
assert.sameValue(disposed[0][1], dispose2, 'Expected dispose2 to be the first onDispose invoked');
assert.sameValue(disposed[1][0], resource1, 'Expected resource1 to be the second disposed resource');
assert.sameValue(disposed[1][1], dispose1, 'Expected dispose1 to be the second onDispose invoked');
