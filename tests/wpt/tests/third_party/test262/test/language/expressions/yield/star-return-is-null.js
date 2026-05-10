// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-generator-function-definitions-runtime-semantics-evaluation
description: >
  If iterator's "return" method is `null`,
  received completion is forwarded to the runtime.
info: |
  YieldExpression : yield * AssignmentExpression

  [...]
  7. Repeat,
    [...]
    c. Else,
      i. Assert: received.[[Type]] is return.
      ii. Let return be ? GetMethod(iterator, "return").
      iii. If return is undefined, then
        [...]
        2. Return Completion(received).

  GetMethod ( V, P )

  [...]
  2. Let func be ? GetV(V, P).
  3. If func is either undefined or null, return undefined.
features: [generators, Symbol.iterator]
---*/

var returnGets = 0;
var iterable = {
  next: function() {
    return {value: 1, done: false};
  },
  get return() {
    returnGets += 1;
    return null;
  },
};

iterable[Symbol.iterator] = function() {
  return iterable;
};

function* generator() {
  yield* iterable;
}

var iterator = generator();
iterator.next();

var result = iterator.return(2);
assert.sameValue(result.value, 2);
assert.sameValue(result.done, true);

assert.sameValue(returnGets, 1);
