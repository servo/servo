// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
  Rejects promise when any property of the `assert` object is not a string
esid: sec-import-call-runtime-semantics-evaluation
info: |
  2.1.1.1 EvaluateImportCall ( specifierExpression [ , optionsExpression ] )
    [...]
    10. If options is not undefined, then
           [...]
        d. If assertionsObj is not undefined,
           [...]
           ii. Let keys be EnumerableOwnPropertyNames(assertionsObj, key).
           iii. IfAbruptRejectPromise(keys, promiseCapability).
           iv. Let supportedAssertions be ! HostGetSupportedImportAssertions().
           v. For each String key of keys,
              1. Let value be Get(assertionsObj, key).
              2. IfAbruptRejectPromise(value, promiseCapability).
              3. If Type(value) is not String, then
                 a. Perform ! Call(promiseCapability.[[Reject]], undefined, « a
                    newly created TypeError object »).
                 b. Return promiseCapability.[[Promise]].
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
    test(import('./2nd-param_FIXTURE.js', {with:{'': undefined}}), 'undefined'),
    test(import('./2nd-param_FIXTURE.js', {with:{'': null}}), 'null'),
    test(import('./2nd-param_FIXTURE.js', {with:{'': false}}), 'boolean'),
    test(import('./2nd-param_FIXTURE.js', {with:{'': 23}}), 'number'),
    test(import('./2nd-param_FIXTURE.js', {with:{'': Symbol('')}}), 'symbol'),
    test(import('./2nd-param_FIXTURE.js', {with:{'': 23n}}), 'bigint'),
    test(import('./2nd-param_FIXTURE.js', {with:{'': {}}}), 'object')
  ])
  .then(function() {})
  .then($DONE, $DONE);
