// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    There are no restrictions on the number of cycles during module traversal
    during indirect export resolution, given unique export names.
esid: sec-moduledeclarationinstantiation
info: |
    [...]
    9. For each ExportEntry Record e in module.[[IndirectExportEntries]], do
       a. Let resolution be ? module.ResolveExport(e.[[ExportName]], « », « »).
       b. If resolution is null or resolution is "ambiguous", throw a
          SyntaxError exception.

    15.2.1.16.3 ResolveExport( exportName, resolveSet, exportStarSet )

    2. For each Record {[[Module]], [[ExportName]]} r in resolveSet, do:
       a. If module and r.[[Module]] are the same Module Record and
          SameValue(exportName, r.[[ExportName]]) is true, then
          i. Assert: this is a circular import request.
          ii. Return null.
    3. Append the Record {[[Module]]: module, [[ExportName]]: exportName} to resolveSet.
    [...]
    5. For each ExportEntry Record e in module.[[IndirectExportEntries]], do
       a. If SameValue(exportName, e.[[ExportName]]) is true, then
          i. Assert: module imports a specific binding for this export.
          ii. Let importedModule be ? HostResolveImportedModule(module,
              e.[[ModuleRequest]]).
          iii. Let indirectResolution be ?
               importedModule.ResolveExport(e.[[ImportName]], resolveSet,
               exportStarSet).
          iv. If indirectResolution is not null, return indirectResolution.
flags: [module]
---*/

export { a } from './instn-iee-iee-cycle-2_FIXTURE.js';
export { c as b } from './instn-iee-iee-cycle-2_FIXTURE.js';
export { e as d } from './instn-iee-iee-cycle-2_FIXTURE.js';
export { g as f } from './instn-iee-iee-cycle-2_FIXTURE.js';
export { i as h } from './instn-iee-iee-cycle-2_FIXTURE.js';
export { k as j } from './instn-iee-iee-cycle-2_FIXTURE.js';
export { m as l } from './instn-iee-iee-cycle-2_FIXTURE.js';
export { o as n } from './instn-iee-iee-cycle-2_FIXTURE.js';
export { q as p } from './instn-iee-iee-cycle-2_FIXTURE.js';
export { s as r } from './instn-iee-iee-cycle-2_FIXTURE.js';
export { u as t } from './instn-iee-iee-cycle-2_FIXTURE.js';
export { w as v } from './instn-iee-iee-cycle-2_FIXTURE.js';
export { y as x } from './instn-iee-iee-cycle-2_FIXTURE.js';
export var z;
