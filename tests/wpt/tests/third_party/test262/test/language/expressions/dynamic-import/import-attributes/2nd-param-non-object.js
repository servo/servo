// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
  Rejects promise when the second argument is neither undefined nor an object
esid: sec-import-call-runtime-semantics-evaluation
info: |
  2.1.1.1 EvaluateImportCall ( specifierExpression [ , optionsExpression ] )
    [...]
    6. Let promiseCapability be ! NewPromiseCapability(%Promise%).
    7. Let specifierString be ToString(specifier).
    8. IfAbruptRejectPromise(specifierString, promiseCapability).
    9. Let assertions be a new empty List.
    10. If options is not undefined, then
        a. If Type(options) is not Object,
           i. Perform ! Call(promiseCapability.[[Reject]], undefined, « a newly created TypeError object »).
           ii. Return promiseCapability.[[Promise]].
    [...]
features: [dynamic-import, import-attributes, Symbol, BigInt]
flags: [async]
---*/

function test(promise, valueType) {
  return promise.then(function() {
      throw new Test262Error('Promise for ' + valueType + ' was not rejected.');
    }, function(error) {
      assert.sameValue(error.constructor, TypeError, valueType);
    });
}

Promise.all([
    test(import('./2nd-param_FIXTURE.js', null), 'null'),
    test(import('./2nd-param_FIXTURE.js', false), 'boolean'),
    test(import('./2nd-param_FIXTURE.js', 23), 'number'),
    test(import('./2nd-param_FIXTURE.js', ''), 'string'),
    test(import('./2nd-param_FIXTURE.js', Symbol('')), 'symbol'),
    test(import('./2nd-param_FIXTURE.js', 23n), 'bigint')
  ])
  .then(function() {})
  .then($DONE, $DONE);
