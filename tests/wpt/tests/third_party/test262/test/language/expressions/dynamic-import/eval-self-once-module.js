// Copyright (C) 2018 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Module is evaluated exactly once
esid: sec-moduleevaluation
info: |
    [...]
    4. If module.[[Evaluated]] is true, return undefined.
    5. Set module.[[Evaluated]] to true.
    6. For each String required that is an element of module.[[RequestedModules]] do,
       a. Let requiredModule be ? HostResolveImportedModule(module, required).
       b. Perform ? requiredModule.ModuleEvaluation().
    [...]

    This test is meant to be flagged as module code, it should not initially
    run as script code or the result will not be the same.
includes: [fnGlobalObject.js]
flags: [async, module]
features: [dynamic-import]
---*/

var global = fnGlobalObject();

if (typeof global.evaluated === 'undefined') {
  global.evaluated = 0;
}

global.evaluated++;

Promise.all([
  import('./eval-self-once-module.js'),
  import('./eval-self-once-module.js'),
]).then(async () => {
  // Use await to serialize imports
  await import('./eval-self-once-module.js');
  await import('./eval-self-once-module.js');

  assert.sameValue(global.evaluated, 1, 'global property was defined and incremented only once');
}).then($DONE, $DONE);
