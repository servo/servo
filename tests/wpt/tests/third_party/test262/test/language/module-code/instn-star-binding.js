// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Immutable binding is created for module namespace object
esid: sec-moduledeclarationinstantiation
info: |
    [...]
    12. For each ImportEntry Record in in module.[[ImportEntries]], do
        a. Let importedModule be ? HostResolveImportedModule(module,
           in.[[ModuleRequest]]).
        b. If in.[[ImportName]] is "*", then
           i. Let namespace be ? GetModuleNamespace(importedModule).
           ii. Perform ! envRec.CreateImmutableBinding(in.[[LocalName]], true).
           iii. Call envRec.InitializeBinding(in.[[LocalName]], namespace).
    [...]
flags: [module]
---*/

assert.sameValue(
  typeof ns, 'object', 'binding is initialized prior to module evaluation'
);

var original = ns;

assert.throws(TypeError, function() {
  ns = null;
}, 'binding rejects assignment');

assert.sameValue(ns, original, 'binding value is immutable');

import * as ns from './instn-star-binding.js';
