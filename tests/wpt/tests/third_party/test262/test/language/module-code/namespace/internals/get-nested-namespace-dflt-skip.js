// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
  Default exports are included in an imported module namespace object when a namespace object is created.
esid: sec-module-namespace-exotic-objects-get-p-receiver
info: |
  [...]
  6. Let binding be ! m.ResolveExport(P, « »).
  7. Assert: binding is a ResolvedBinding Record.
  8. Let targetModule be binding.[[Module]].
  9. Assert: targetModule is not undefined.
  10. If binding.[[BindingName]] is "*namespace*", then
  11. Return ? GetModuleNamespace(targetModule).

  Runtime Semantics: GetModuleNamespace
  [...]
    3. If namespace is undefined, then
       a. Let exportedNames be ? module.GetExportedNames(« »).
       b. Let unambiguousNames be a new empty List.
       c. For each name that is an element of exportedNames,
          i. Let resolution be ? module.ResolveExport(name, « », « »).
          ii. If resolution is null, throw a SyntaxError exception.
          iii. If resolution is not "ambiguous", append name to
               unambiguousNames.
       d. Let namespace be ModuleNamespaceCreate(module, unambiguousNames).
  [...]
flags: [module]
features: [export-star-as-namespace-from-module]
---*/

import * as namedns1 from './get-nested-namespace-dflt-skip-named_FIXTURE.js';
import * as productionns1 from './get-nested-namespace-dflt-skip-prod_FIXTURE.js';

assert('namedOther' in namedns1.namedns2);
assert.sameValue(
  'default' in namedns1.namedns2, true, 'default specified via identifier'
);

assert('productionOther' in productionns1.productionns2);
assert.sameValue(
  'default' in productionns1.productionns2, true, 'default specified via dedicated production'
);
