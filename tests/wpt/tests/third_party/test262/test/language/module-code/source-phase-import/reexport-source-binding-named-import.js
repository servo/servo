// Copyright (C) 2026 Guy Bedford. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
  A named import of a re-exported source-phase binding is initialized to the
  imported module's ModuleSource object.
esid: sec-source-text-module-record-initialize-environment
info: |
  When a module does `import source x from "..."; export { x };`, ParseModule
  reclassifies the local re-export to an indirect ExportEntry whose
  [[ImportName]] is ~source~. ResolveExport on the re-exporting module returns
  a ResolvedBinding Record with [[BindingName]] ~source~ and [[Module]] set to
  the source-phase target module.

  InitializeEnvironment ( )

    7. For each ImportEntry Record in of module.[[ImportEntries]], do
       [...]
       d. Else,
          i. Assert: in.[[ImportName]] is a String.
          ii. Let resolution be importedModule.ResolveExport(in.[[ImportName]]).
          iii. If resolution is either null or ambiguous, throw a SyntaxError.
          iv. If resolution.[[BindingName]] is namespace, then [...]
          v. Else if resolution.[[BindingName]] is source, then
             1. Let moduleSourceObject be resolution.[[Module]].[[ModuleSource]].
             2. If moduleSourceObject is empty, throw a SyntaxError exception.
             3. Perform ! env.CreateImmutableBinding(in.[[LocalName]], true).
             4. Perform ! env.InitializeBinding(in.[[LocalName]], moduleSourceObject).

  The named import `x` is therefore bound directly to the ModuleSource object
  of the underlying module, an instance of %AbstractModuleSource%.
features: [source-phase-imports, source-phase-imports-module-source]
flags: [module]
---*/

import { x } from './reexport-source-binding_FIXTURE.js';

assert.sameValue(typeof x, 'object',
  'Named import of a re-exported source binding should be bound to an object');
assert(x instanceof $262.AbstractModuleSource,
  'x should be a %AbstractModuleSource% instance (the underlying ModuleSource)');
