// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Module is evaluated exactly once
esid: sec-moduleevaluation
info: |
  Evaluate( ) Concrete Method
    [...]
    4. Let result be InnerModuleEvaluation(module, stack, 0).
    [...]

  InnerModuleEvaluation( module, stack, index )
    [...]
    2. If module.[[Status]] is "evaluated", then
      a. If module.[[EvaluationError]] is undefined, return index.
      b. Otherwise return module.[[EvaluationError]].
    [...]
    6. For each String required that is an element of module.[[RequestedModules]] do,
       a. Let requiredModule be ? HostResolveImportedModule(module, required).
       [...]
       c. Set index to ? InnerModuleEvaluation(requiredModule, stack, index).
    [...]
includes: [fnGlobalObject.js]
flags: [module]
features: [export-star-as-namespace-from-module]
---*/

import {} from './eval-self-once.js';
import './eval-self-once.js';
import * as ns1 from './eval-self-once.js';
import dflt1 from './eval-self-once.js';
export {} from './eval-self-once.js';
import dflt2, {} from './eval-self-once.js';
export * from './eval-self-once.js';
export * as ns2 from './eval-self-once.js';
import dflt3, * as ns from './eval-self-once.js';
export default null;

var global = fnGlobalObject();

assert.sameValue(global.test262, undefined, 'global property initially unset');

global.test262 = 262;

assert.sameValue(global.test262, 262, 'global property was defined');
