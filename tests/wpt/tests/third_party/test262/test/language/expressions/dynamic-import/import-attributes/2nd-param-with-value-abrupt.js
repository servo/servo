// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
  Rejects promise when retrieving a value of the `assert` object produces an
  abrupt completion
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
    [...]
features: [dynamic-import, import-attributes]
flags: [async]
---*/

var thrown = new Test262Error();

import('./2nd-param_FIXTURE.js', {with:{get ''() { throw thrown; }}})
  .then(function() {
    throw new Test262Error('Expected promise to be rejected, but it was fulfilled');
  }, function(error) {
    assert.sameValue(error, thrown);
  })
  .then($DONE, $DONE);
