// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Modules can be visited more than once when resolving bindings through
    "star" exports as long as the exportName is different each time.
esid: sec-moduledeclarationinstantiation
info: |
    [...]
    12. For each ImportEntry Record in in module.[[ImportEntries]], do
        a. Let importedModule be ? HostResolveImportedModule(module,
           in.[[ModuleRequest]]).
        b. If in.[[ImportName]] is "*", then
           [...]
        c. Else,
           i. Let resolution be ?
              importedModule.ResolveExport(in.[[ImportName]], « », « »).
           [...]
           iii. Call envRec.CreateImportBinding(in.[[LocalName]],
                resolution.[[Module]], resolution.[[BindingName]]).
    [...]

    15.2.1.16.3 ResolveExport( exportName, resolveSet )

    [...]
    3. Append the Record {[[Module]]: module, [[ExportName]]: exportName} to resolveSet.
    4. For each ExportEntry Record e in module.[[LocalExportEntries]], do
       a. If SameValue(exportName, e.[[ExportName]]) is true, then
          i.  Assert: module provides the direct binding for this export.
          ii. Return Record{[[Module]]: module, [[BindingName]]: e.[[LocalName]]}.
    5. For each ExportEntry Record e in module.[[IndirectExportEntries]], do
       a. If SameValue(exportName, e.[[ExportName]]) is true, then
          i.   Assert: module imports a specific binding for this export.
          ii.  Let importedModule be ? HostResolveImportedModule(module, e.[[ModuleRequest]]).
          iii. Return ? importedModule.ResolveExport(e.[[ImportName]], resolveSet).
    [...]
    8. For each ExportEntry Record e in module.[[StarExportEntries]], do
       a. Let importedModule be ? HostResolveImportedModule(module, e.[[ModuleRequest]]).
       b. Let resolution be ? importedModule.ResolveExport(exportName, resolveSet).
       [...]
       d. If resolution is not null, then
          i.  If starResolution is null, let starResolution be resolution.
          [...]
    9. Return starResolution.
flags: [module]
---*/

import { x } from './instn-named-star-cycle-2_FIXTURE.js';

assert.sameValue(x, 45);
