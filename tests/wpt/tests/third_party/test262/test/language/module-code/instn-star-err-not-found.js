// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Importing a namespace for a module which contains an unresolvable named
    export
esid: sec-moduledeclarationinstantiation
info: |
    [...]
    12. For each ImportEntry Record in in module.[[ImportEntries]], do
        a. Let importedModule be ? HostResolveImportedModule(module,
           in.[[ModuleRequest]]).
        b. If in.[[ImportName]] is "*", then
           i. Let namespace be ? GetModuleNamespace(importedModule).
    [...]

    15.2.1.18 Runtime Semantics: GetModuleNamespace

    [...]
    3. If namespace is undefined, then
       [...]
       c. For each name that is an element of exportedNames,
          i. Let resolution be ? module.ResolveExport(name, « », « »).
          ii. If resolution is null, throw a SyntaxError exception.
negative:
  phase: resolution
  type: SyntaxError
flags: [module]
---*/

$DONOTEVALUATE();

import * as ns from './instn-star-err-not-found-faulty_FIXTURE.js';
