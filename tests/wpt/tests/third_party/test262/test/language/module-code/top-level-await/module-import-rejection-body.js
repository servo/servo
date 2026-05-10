// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-moduleevaluation
description: >
  Evaluate imported rejected module
info: |
  Table 3: Additional Fields of Cyclic Module Records

  [[Async]]

  ...
  Having an asynchronous dependency does not make the module asynchronous. This field must not change after the module is parsed.

  Evaluate ( ) Concrete Method

  ...
  6. Let capability be ! NewPromiseCapability(%Promise%).
  7. Set module.[[TopLevelCapability]] to capability.
  8. Let result be InnerModuleEvaluation(module, stack, 0).
  9. If result is an abrupt completion, then
    ...
    d. Perform ! Call(capability.[[Reject]], undefined, «result.[[Value]]»).
  10. Otherwise,
    ...
    b. If module.[[AsyncEvaluating]] is false, then
      i. Perform ! Call(capability.[[Resolve]], undefined, «undefined»).
    ...
  11. Return undefinedcapability.[[Promise]].

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
flags: [module]
features: [top-level-await]
negative:
  phase: runtime
  type: TypeError
---*/

import foo from './module-import-rejection-body_FIXTURE.js';

throw new Test262Error('this should be unreachable');
