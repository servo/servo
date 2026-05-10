// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-moduleevaluation
description: >
  Dynamic imported module with async exports
info: |
  InnerModuleEvaluation( module, stack, index )

  ...
  14. If module.[[PendingAsyncDependencies]] is > 0, set module.[[AsyncEvaluating]] to true.
  15. Otherwise, if module.[[Async]] is true, perform ! ExecuteAsyncModule(module).
  16. Otherwise, perform ? module.ExecuteModule().

  ExecuteAsyncModule ( module )

  1. Assert: module.[[Status]] is "evaluating" or "evaluated".
  2. Assert: module.[[Async]] is true.
  3. Set module.[[AsyncEvaluating]] to true.
  4. Let capability be ! NewPromiseCapability(%Promise%).
  5. Let stepsFulfilled be the steps of a CallAsyncModuleFulfilled function as specified below.
  ...
  8. Let stepsRejected be the steps of a CallAsyncModuleRejected function as specified below.
  ...
  11. Perform ! PerformPromiseThen(capability.[[Promise]], onFulfilled, onRejected).
  12. Perform ! module.ExecuteModule(capability).
  13. Return.

  ExecuteModule ( [ capability ] )

  ...
  11. If module.[[Async]] is false, then
    a. Assert: capability was not provided.
    b. Push moduleCxt on to the execution context stack; moduleCxt is now the running execution context.
    c. Let result be the result of evaluating module.[[ECMAScriptCode]].
    d. Suspend moduleCxt and remove it from the execution context stack.
    e. Resume the context that is now on the top of the execution context stack as the running execution context.
    f. Return Completion(result).
  12. Otherwise,
    a. Assert: capability is a PromiseCapability Record.
    b. Perform ! AsyncBlockStart(capability, module.[[ECMAScriptCode]], moduleCxt).
    c. Return.
flags: [module, async]
features: [top-level-await, dynamic-import]
---*/

import('./module-import-rejection_FIXTURE.js').then(() => {
  throw new Test262Error('Should not be fulfilled');
}, (err) => {
  assert(err instanceof TypeError);
  assert.sameValue(err.constructor, TypeError);
  assert.sameValue(err.message, 'error in the default export line');
}).then($DONE, $DONE);
