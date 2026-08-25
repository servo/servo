// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Named import binding - resolution failure (not found)
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
        [...]
    11. Return starResolution.
negative:
  phase: resolution
  type: SyntaxError
flags: [module]
---*/

$DONOTEVALUATE();

import { x as y } from './instn-named-err-not-found-empty_FIXTURE.js';
