// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    ImportCall returns a new instance of Promise
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
---*/

const p1 = import('./dynamic-import-module_FIXTURE.js');
const p2 = import('./dynamic-import-module_FIXTURE.js');

assert.notSameValue(p1, p2, 'the returned promises are not the same, regardless the reference and specifier pair');

assert.sameValue(p1.constructor, Promise, 'p1 constructor is %Promise%');
assert.sameValue(Object.getPrototypeOf(p1), Promise.prototype, 'p1 prototype is %PromisePrototype%');

assert.sameValue(p2.constructor, Promise, 'p2 constructor is %Promise%');
assert.sameValue(Object.getPrototypeOf(p2), Promise.prototype, 'p2 prototype is %PromisePrototype%');
