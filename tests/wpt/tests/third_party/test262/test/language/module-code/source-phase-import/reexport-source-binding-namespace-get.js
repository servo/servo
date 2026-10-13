// Copyright (C) 2026 Guy Bedford. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
  Namespace [[Get]] of a re-exported source-phase binding returns the
  imported module's ModuleSource object.
esid: sec-module-namespace-exotic-objects-get-p-receiver
info: |
  When a module does `import source x from "..."; export { x };`, ParseModule
  reclassifies the local re-export to an indirect ExportEntry whose
  [[ImportName]] is ~source~. ResolveExport on the re-exporting module returns
  a ResolvedBinding Record with [[BindingName]] ~source~ and [[Module]] set to
  the source-phase target module.

  Module Namespace Exotic Object [[Get]] ( P, Receiver )

    [...]
    7. If binding.[[BindingName]] is namespace, then
      a. Return GetModuleNamespace(targetModule).
    8. If binding.[[BindingName]] is source, then
      a. Let moduleSourceObject be targetModule.[[ModuleSource]].
      b. If moduleSourceObject is empty, throw a ReferenceError exception.
      c. Return moduleSourceObject.
    9. [...]

  Accessing the re-exported binding via `ns.x` therefore returns the
  ModuleSource object of the underlying module, an instance of
  %AbstractModuleSource%.
features: [source-phase-imports, source-phase-imports-module-source]
flags: [module]
---*/

import * as ns from './reexport-source-binding_FIXTURE.js';

assert.sameValue(typeof ns.x, 'object',
  'Namespace [[Get]] of a re-exported source binding should return an object');
assert(ns.x instanceof $262.AbstractModuleSource,
  'ns.x should be a %AbstractModuleSource% instance (the underlying ModuleSource)');
