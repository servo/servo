// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-generator-function-definitions-runtime-semantics-evaluation
description: >
  `resolve` is anonymous built-in function with length of 1.
info: |
  YieldExpression : yield AssignmentExpression 

  [...]
  4. If generatorKind is async, then return ? AsyncGeneratorYield(value).

  AsyncGeneratorYield ( value )

  [...]
  5. Set value to ? Await(value).
  [...]
  9. Return ! AsyncGeneratorResolve(generator, value, false).

  Await

  [...]
  2. Let promise be ? PromiseResolve(%Promise%, value).
  [...]

  PromiseResolve ( C, x )

  [...]
  3. Let promiseCapability be ? NewPromiseCapability(C).
  4. Perform ? Call(promiseCapability.[[Resolve]], undefined, « x »).
  5. Return promiseCapability.[[Promise]].

  NewPromiseCapability ( C )

  [...]
  7. Let promise be ? Construct(C, « executor »).
  [...]
  11. Return promiseCapability.

  Promise ( executor )

  [...]
  8. Let resolvingFunctions be CreateResolvingFunctions(promise).
  [...]
  11. Return promise.

  CreateResolvingFunctions ( promise )

  [...]
  3. Let resolve be ! CreateBuiltinFunction(stepsResolve, « [[Promise]], [[AlreadyResolved]] »).
  [...]
  10. Return the Record { [[Resolve]]: resolve, [[Reject]]: reject }.
flags: [async]
features: [async-iteration, Reflect.construct]
includes: [isConstructor.js]
---*/

var thenable = {
  then: function(resolve) {
    resolve(resolve);
  },
};

var iter = (async function*() {
  yield thenable;
}());

iter.next().then(function(result) {
  var resolve = result.value;
  assert(!isConstructor(resolve));
  assert.sameValue(resolve.length, 1);
  assert.sameValue(resolve.name, '');
}).then($DONE, $DONE);
