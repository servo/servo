// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: IndirectExportEntries validation - undefined imported bindings
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

export { x } from './instn-iee-err-not-found-empty_FIXTURE.js';
