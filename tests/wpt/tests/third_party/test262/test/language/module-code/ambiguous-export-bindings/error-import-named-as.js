// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Named import binding - resolution failure (ambiguous name)
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
           ii. If resolution is null or resolution is "ambiguous", throw a
               SyntaxError exception.

    15.2.1.16.3 ResolveExport

    [...]
    9. Let starResolution be null.
    10. For each ExportEntry Record e in module.[[StarExportEntries]], do
        a. Let importedModule be ? HostResolveImportedModule(module,
           e.[[ModuleRequest]]).
        b. Let resolution be ? importedModule.ResolveExport(exportName,
           resolveSet, exportStarSet).
        c. If resolution is "ambiguous", return "ambiguous".
        d. If resolution is not null, then
           i. If starResolution is null, let starResolution be resolution.
           ii. Else,
               1. Assert: there is more than one * import that includes the
                  requested name.
               2. If resolution.[[Module]] and starResolution.[[Module]] are
                  not the same Module Record or
                  SameValue(resolution.[[BindingName]],
                  starResolution.[[BindingName]]) is false, return "ambiguous".
negative:
  phase: resolution
  type: SyntaxError
flags: [module]
---*/

$DONOTEVALUATE();

import { x as y } from './error-import-named_FIXTURE.js';
