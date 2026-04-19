// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    ImportCall can be used from eval code
esid: sec-import-call-runtime-semantics-evaluation
info: |
    Import Calls

    Runtime Semantics: Evaluation
    
    ImportCall : import(AssignmentExpression)
    
    1. Let referencingScriptOrModule be ! GetActiveScriptOrModule().
    2. Let argRef be the result of evaluating AssignmentExpression.
    3. Let specifier be ? GetValue(argRef).
    4. Let promiseCapability be ! NewPromiseCapability(%Promise%).
    5. Let specifierString be ToString(specifier).
    6. IfAbruptRejectPromise(specifierString, promiseCapability).
    7. Perform ! HostImportModuleDynamically(referencingScriptOrModule, specifierString, promiseCapability).
    8. Return promiseCapability.[[Promise]].
features: [dynamic-import]
flags: [async]
---*/

const p = eval("import('./module-code_FIXTURE.js');");

assert.sameValue(Promise.resolve(p), p, 'constructor is %Promise%');
assert.sameValue(Object.getPrototypeOf(p), Promise.prototype, 'prototype is %PromisePrototype%');

p.then(imported => {
    assert.sameValue(imported.default, 42);
    assert.sameValue(imported.local1, 'Test262');
    assert.sameValue(imported.renamed, 'TC39');
    assert.sameValue(imported.indirect, 'Test262');
}).then($DONE, $DONE);
