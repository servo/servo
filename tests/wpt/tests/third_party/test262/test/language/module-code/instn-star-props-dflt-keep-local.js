// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Local default exports are included in the module namespace object
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
    5. For each ExportEntry Record e in module.[[LocalExportEntries]], do
       a. Assert: module provides the direct binding for this export.
       b. Append e.[[ExportName]] to exportedNames.
    [...]
flags: [module]
---*/

import * as named from './instn-star-props-dflt-keep-local-named_FIXTURE.js';
import * as production from './instn-star-props-dflt-keep-local-prod_FIXTURE.js';

assert.sameValue('default' in named, true, 'default specified via identifier');

assert.sameValue(
  'default' in production, true, 'default specified via dedicated production'
);
