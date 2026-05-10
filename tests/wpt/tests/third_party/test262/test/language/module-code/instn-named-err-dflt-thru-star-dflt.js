// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Named import binding - default not found (excluding *)
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

import x from './instn-named-err-dflt-thru-star-int_FIXTURE.js';
