// Copyright (C) 2018 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Script is evaluated exactly once after loaded by import
esid: sec-hostimportmoduledynamically
info: |
  Success path

  The completion value of any subsequent call to HostResolveImportedModule after
  FinishDynamicImport has completed, given the arguments referencingScriptOrModule
  and specifier, must be a module which has already been evaluated, i.e. whose
  Evaluate concrete method has already been called and returned a normal completion.

  This test is meant to __not__ be flagged as module code, it should not initially
  run as module code or the result will not be the same.
includes: [fnGlobalObject.js]
flags: [async]
features: [dynamic-import]
---*/

var global = fnGlobalObject();

var isFirstScript = typeof global.evaluated === 'undefined';
if (isFirstScript) {
  global.evaluated = 0;
}

global.evaluated++;

var p = Promise.all([
  import('./eval-self-once-script.js'),
  import('./eval-self-once-script.js'),
]).then(async () => {
  // Use await to serialize imports
  await import('./eval-self-once-script.js');
  await import('./eval-self-once-script.js');

  assert.sameValue(global.evaluated, 2, 'global property was defined once and incremented twice');
});

if (isFirstScript) {
  p.then($DONE, $DONE);
}
