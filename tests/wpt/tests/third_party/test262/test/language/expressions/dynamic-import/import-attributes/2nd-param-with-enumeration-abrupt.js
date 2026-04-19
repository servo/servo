// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Reports abrupt completions produced by attributes enumeration
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
           [...]
        b. Let assertionsObj be Get(options, "assert").
        c. IfAbruptRejectPromise(assertionsObj, promiseCapability).
        d. If assertionsObj is not undefined,
           i. If Type(assertionsObj) is not Object,
              [...]
           ii. Let keys be EnumerableOwnPropertyNames(assertionsObj, key).
           iii. IfAbruptRejectPromise(keys, promiseCapability).
    [...]
features: [dynamic-import, import-attributes, Proxy]
flags: [async]
---*/

var thrown = new Test262Error();
var options = {
  with: new Proxy({}, {
    ownKeys: function() {
      throw thrown;
    },
  })
};

import('./2nd-param_FIXTURE.js', options)
  .then(function() {
    throw new Test262Error('Expected promise to be rejected, but promise was fulfilled.');
  }, function(error) {
    assert.sameValue(error, thrown);
  })
  .then($DONE, $DONE);
