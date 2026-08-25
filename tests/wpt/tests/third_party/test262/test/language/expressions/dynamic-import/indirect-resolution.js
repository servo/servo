// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Dynamic Import should resolve another import call
esid: sec-import-call-runtime-semantics-evaluation
info: |
    Runtime Semantics: Evaluation

    ImportCall : import ( AssignmentExpression )

    1. Let referencingScriptOrModule be ! GetActiveScriptOrModule().
    2. Let argRef be the result of evaluating AssignmentExpression.
    3. Let specifier be ? GetValue(argRef).
    4. Let promiseCapability be ! NewPromiseCapability(%Promise%).
    5. Let specifierString be ToString(specifier).
    6. IfAbruptRejectPromise(specifierString, promiseCapability).
    7. Perform ! HostImportModuleDynamically(referencingScriptOrModule, specifierString, promiseCapability).
    8. Return promiseCapability.[[Promise]].
flags: [async]
features: [dynamic-import]
---*/

import('./indirect-resolution-1_FIXTURE.js').then(async imported => {
  assert.sameValue(Promise.resolve(imported.default), imported.default, 'default is Promise instance');
  assert.sameValue(Object.getPrototypeOf(imported.default), Promise.prototype, 'default proto is Promise.prototype');
  assert.sameValue(imported.default.constructor, Promise, 'default.constructor is Promise');

  var indirect = await imported.default;
  assert.sameValue(indirect.default, 42);
}).then($DONE, $DONE);
