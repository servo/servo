// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Imported self bindings should update the references
esid: sec-finishdynamicimport
info: |
    Runtime Semantics: FinishDynamicImport ( referencingScriptOrModule, specifier, promiseCapability, completion )
    
    2. Otherwise,
        a. Assert: completion is a normal completion and completion.[[Value]] is undefined.
        b. Let moduleRecord be ! HostResolveImportedModule(referencingScriptOrModule, specifier).
        c. Assert: Evaluate has already been invoked on moduleRecord and successfully completed.
        d. Let namespace be GetModuleNamespace(moduleRecord).
        ...
        f. Otherwise, perform ! Call(promiseCapability.[[Resolve]], undefined, « namespace.[[Value]] »).
flags: [async, module]
features: [dynamic-import]
---*/

let x = 0;
export { x, x as y };
async function fn() {
  var imported = await import('./imported-self-update.js');
  assert.sameValue(imported.x, 0, 'original value, direct binding');
  assert.sameValue(imported.y, 0, 'original value, indirect binding');
  x = 1;
  assert.sameValue(imported.x, 1, 'updated value, direct binding');
  assert.sameValue(imported.y, 1, 'updated value, indirect binding');
}

// Do not use asyncTest: when self imported, $DONE is not defined, asyncTest will throw
fn().then($DONE, $DONE);
