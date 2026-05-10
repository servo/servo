// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Star export resolution skips multiple cyclic indirect named re-exports
    and resolves the binding from a non-cyclic path.
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

    Module "a" has three star exports:
    - "b" has `export { foo } from "a"` (cycles back, returns null).
    - "c" has `export let foo = 1` (provides the binding).
    - "d" has `export { foo } from "a"` (cycles back, returns null).
    The two cyclic paths return null (cycle detection), the star loop
    skips them, and the binding from "c" is used.
flags: [module]
---*/

import { foo } from './instn-star-iee-multi-cycle-same-name-a_FIXTURE.js';

assert.sameValue(foo, 1);
