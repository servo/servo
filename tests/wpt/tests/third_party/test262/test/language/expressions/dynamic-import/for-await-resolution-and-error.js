// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Resolve multiple imports through a for await loop
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
flags: [async]
features: [dynamic-import]
includes: [compareArray.js]
---*/

let r = [];
async function aiter() {
  for await (let imported of [
      import('./for-await-resolution-and-error-a_FIXTURE.js'),
      import('./for-await-resolution-and-error-b_FIXTURE.js'),
      import('./for-await-resolution-and-error-poisoned_FIXTURE.js'),
      import('./for-await-resolution-and-error-a_FIXTURE.js'), // this should be ignored
    ]) {
    r.push(imported.x);
  }
}

aiter().then(() => { throw 'The async function should not resolve' }, error => {
  assert.compareArray(r, [42, 39]);
  assert.sameValue(error, 'foo');
}).then($DONE, $DONE);
