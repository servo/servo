// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: Behavior with a custom RegExp exec
info: |
  %RegExpStringIteratorPrototype%.next ( )
    [...]
    9. Let match be ? RegExpExec(R, S).

  Runtime Semantics: RegExpExec ( R, S )
    1. Assert: Type(R) is Object.
    2. Assert: Type(S) is String.
    3. Let exec be ? Get(R, "exec").
    4. If IsCallable(exec) is true, then
      a. Let result be ? Call(exec, R, « S »).
      b. If Type(result) is neither Object or Null, throw a TypeError exception.
      c. Return result.
features: [Symbol.matchAll]
---*/

var regexp = /./g;
var str = 'abc';
var iter = regexp[Symbol.matchAll](str);

var callArgs, callCount;
function callNextWithExecReturnValue(returnValue) {
  callArgs = undefined;
  callCount = 0;

  RegExp.prototype.exec = function() {
    callArgs = arguments;
    callCount++;
    return returnValue;
  }

  return iter.next();
}

var firstExecReturnValue = ['ab'];
var result = callNextWithExecReturnValue(firstExecReturnValue);
assert.sameValue(result.value, firstExecReturnValue);
assert(!result.done);

assert.sameValue(callArgs.length, 1);
assert.sameValue(callArgs[0], str);
assert.sameValue(callCount, 1);

result = callNextWithExecReturnValue(null);
assert.sameValue(result.value, undefined);
assert(result.done);

assert.sameValue(callArgs.length, 1);
assert.sameValue(callArgs[0], str);
assert.sameValue(callCount, 1);
