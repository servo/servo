// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Accepts undefined for the `assert` property of the second argument
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
              1. Perform ! Call(promiseCapability.[[Reject]], undefined, « a
                 newly created TypeError object »).
              2. Return promiseCapability.[[Promise]].
    [...]
features: [dynamic-import, import-attributes, Symbol, BigInt]
flags: [async]
---*/

Promise.all([
    import('./2nd-param_FIXTURE.js', {}),
    import('./2nd-param_FIXTURE.js', {with:undefined}),
  ])
  .then(function(values) {
    assert.sameValue(values[0].default, 262);
    assert.sameValue(values[1].default, 262);
  })
  .then($DONE, $DONE);
