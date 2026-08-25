// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Indirect default exports are included in the module namespace object
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
    6. For each ExportEntry Record e in module.[[IndirectExportEntries]], do
       a. Assert: module imports a specific binding for this export.
       b. Append e.[[ExportName]] to exportedNames.
    [...]
flags: [module]
---*/

import * as ns from './instn-star-props-dflt-keep-indirect-reexport_FIXTURE.js';

assert.sameValue('default' in ns, true);
