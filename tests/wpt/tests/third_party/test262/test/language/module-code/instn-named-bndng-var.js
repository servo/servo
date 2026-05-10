// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Imported binding reflects state of exported `var` binding
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
    14. Let declaredVarNames be a new empty List.
    15. For each element d in varDeclarations do
        a. For each element dn of the BoundNames of d do
           i. If dn is not an element of declaredVarNames, then
              1. Perform ! envRec.CreateMutableBinding(dn, false).
              2. Call envRec.InitializeBinding(dn, undefined).
              3. Append dn to declaredVarNames.
    [...]

    8.1.1.5.5 CreateImportBinding

    [...]
    5. Create an immutable indirect binding in envRec for N that references M
       and N2 as its target binding and record that the binding is initialized.
    6. Return NormalCompletion(empty).
flags: [module]
---*/

assert.sameValue(
  y,
  undefined,
  'binding is initialized to `undefined` prior to module evaulation'
);

assert.throws(TypeError, function() {
  y = null;
}, 'binding rejects assignment');

assert.sameValue(y, undefined, 'binding value is immutable');

import { x as y } from './instn-named-bndng-var.js';
export var x = 23;
