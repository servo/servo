// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: IndirectExportEntries validation - circular imported bindings
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
    2. For each Record {[[Module]], [[ExportName]]} r in resolveSet, do:
       a. If module and r.[[Module]] are the same Module Record and
          SameValue(exportName, r.[[ExportName]]) is true, then
          i. Assert: this is a circular import request.
          ii. Return null.
negative:
  phase: resolution
  type: SyntaxError
flags: [module]
---*/

$DONOTEVALUATE();

export { x } from './instn-iee-err-circular_FIXTURE.js';
