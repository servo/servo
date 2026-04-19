// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Dynamic Import should await for evaluation
esid: sec-finishdynamicimport
info: |
    Runtime Semantics: FinishDynamicImport ( referencingScriptOrModule, specifier, promiseCapability, completion )
    
    2. Otherwise,
        a. Assert: completion is a normal completion and completion.[[Value]] is undefined.
        b. Let moduleRecord be ! HostResolveImportedModule(referencingScriptOrModule, specifier).
        c. Assert: Evaluate has already been invoked on moduleRecord and successfully completed.
flags: [async]
features: [dynamic-import]
---*/

var startTime = Date.now();

import('./await-import-evaluation_FIXTURE.js').then(imported => {
  var endTime = Date.now() - startTime;
  assert(imported.time > 100, `${String(imported.time)} > 100`);
  assert(imported.time <= endTime, `${String(imported.time)} > ${String(endTime)}`);
}).then($DONE, $DONE);
