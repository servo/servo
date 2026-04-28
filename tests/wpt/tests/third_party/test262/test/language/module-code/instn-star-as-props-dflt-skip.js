// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
  Default exports are included in an imported module namespace object when module exported with `* as namespace`
esid: sec-moduledeclarationinstantiation
info: |
  [...]
  4. Let result be InnerModuleInstantiation(module, stack, 0).
  [...]

  InnerModuleInstantiation( module, stack, index )
  [...]
  10. Perform ? ModuleDeclarationEnvironmentSetup(module).
  [...]

  ModuleDeclarationEnvironmentSetup( module )
  [...]
    c. If in.[[ImportName]] is "*", then
    [...]
    d. Else,
      i. Let resolution be ? importedModule.ResolveExport(in.[[ImportName]], « »).
      ii. If resolution is null or "ambiguous", throw a SyntaxError exception.
      iii. If resolution.[[BindingName]] is "*namespace*", then
        1. Let namespace be ? GetModuleNamespace(resolution.[[Module]]).
    [...]

  15.2.1.18 Runtime Semantics: GetModuleNamespace

  [...]
  3. If namespace is undefined, then
  a. Let exportedNames be ? module.GetExportedNames(« »).
  [...]

  15.2.1.16.2 GetExportedNames

  [...]
  7. For each ExportEntry Record e in module.[[StarExportEntries]], do
  [...]
  c. For each element n of starNames, do
  i. If SameValue(n, "default") is false, then
  [...]
flags: [module]
features: [export-star-as-namespace-from-module]
---*/

import {named} from './instn-star-props-dflt-skip-star-as-named_FIXTURE.js';
import {production} from './instn-star-props-dflt-skip-star-as-prod_FIXTURE.js';

assert('namedOther' in named);
assert.sameValue(
  'default' in named, true, 'default specified via identifier'
);

assert('productionOther' in production);
assert.sameValue(
  'default' in production, true, 'default specified via dedicated production'
);
