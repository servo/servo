// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Imported binding reflects state of exported default binding (expressions)
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
           iii. Call envRec.CreateImportBinding(in.[[LocalName]],
                resolution.[[Module]], resolution.[[BindingName]]).
    [...]

    8.1.1.5.5 CreateImportBinding

    [...]
    5. Create an immutable indirect binding in envRec for N that references M
       and N2 as its target binding and record that the binding is initialized.
    6. Return NormalCompletion(empty).
flags: [module]
---*/

assert.throws(ReferenceError, function() {
  typeof dflt;
}, 'binding is created but not initialized');

import dflt from './instn-named-bndng-dflt-expr.js';
export default (function() {});
