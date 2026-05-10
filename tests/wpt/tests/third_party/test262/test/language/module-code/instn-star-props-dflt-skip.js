// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Default exports are not included in the module namespace object
esid: sec-moduledeclarationinstantiation
info: |
    [...]
    12. For each ImportEntry Record in in module.[[ImportEntries]], do
        a. Let importedModule be ? HostResolveImportedModule(module,
           in.[[ModuleRequest]]).
        b. If in.[[ImportName]] is "*", then
           i. Let namespace be ? GetModuleNamespace(importedModule).
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
---*/

import * as named from './instn-star-props-dflt-skip-star-named_FIXTURE.js';
import * as production from './instn-star-props-dflt-skip-star-prod_FIXTURE.js';

assert('namedOther' in named);
assert.sameValue(
  'default' in named, false, 'default specified via identifier'
);

assert('productionOther' in production);
assert.sameValue(
  'default' in production, false, 'default specified via dedicated production'
);
