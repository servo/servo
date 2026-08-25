// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Star export resolution skips a single cyclic indirect named re-export
    and resolves the binding from the remaining non-cyclic path.
esid: sec-resolveexport
info: |
    ResolveExport ( exportName [ , resolveSet ] )

    [...]
    1. For each Record { [[Module]], [[ExportName]] } r of resolveSet, do
      a. If module and r.[[Module]] are the same Module Record and exportName is r.[[ExportName]], then
        i. Assert: This is a circular import request.
        i. Return null.
    1. Append the Record { [[Module]]: module, [[ExportName]]: exportName }
       to resolveSet.
    [...]
    1. For each ExportEntry Record e of module.[[IndirectExportEntries]], do
      a. If exportName is e.[[ExportName]], then
        [...]
        1. Return importedModule.ResolveExport(e.[[ImportName]],
           resolveSet).
    [...]
    1. Let starResolution be null.
    1. For each ExportEntry Record e of module.[[StarExportEntries]], do
      a. Let importedModule be GetImportedModule(module,
         e.[[ModuleRequest]]).
      a. Let resolution be importedModule.ResolveExport(exportName,
         resolveSet).
      a. If resolution is AMBIGUOUS, return AMBIGUOUS.
      a. If resolution is not null, then
         i. If starResolution is null, let starResolution be resolution.
         [...]
    1. Return starResolution.

    Module "a" has two star exports:
    - One to "b", which has `export { foo } from "a"` (cycles back).
    - One to "c", which has `export let foo = 42` (provides the binding).
    The cyclic path returns null (cycle detection), the star loop skips it,
    and the binding from "c" is used.
flags: [module]
---*/

import { foo } from './instn-star-iee-single-cycle-same-name-a_FIXTURE.js';

assert.sameValue(foo, 42);
