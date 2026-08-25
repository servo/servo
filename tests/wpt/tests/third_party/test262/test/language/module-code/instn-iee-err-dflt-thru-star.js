// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: IndirectExportEntries validation - default not found (excluding *)
esid: sec-moduledeclarationinstantiation
info: |
    [...]
    9. For each ExportEntry Record e in module.[[IndirectExportEntries]], do
       a. Let resolution be ? module.ResolveExport(e.[[ExportName]], « », « »).
       b. If resolution is null or resolution is "ambiguous", throw a
          SyntaxError exception.
    [...]

    15.2.1.16.3 ResolveExport

    [...]
    6. If SameValue(exportName, "default") is true, then
       a. Assert: A default export was not explicitly defined by this module.
       b. Throw a SyntaxError exception.
       c. NOTE A default export cannot be provided by an export *.
negative:
  phase: resolution
  type: SyntaxError
flags: [module]
---*/

$DONOTEVALUATE();

export { default } from './instn-iee-err-dflt-thru-star-int_FIXTURE.js';
