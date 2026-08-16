// Copyright (C) 2026 Guy Bedford. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
  Re-exporting the same source-phase import via `import source mod from
  '<module source>'; export { mod }` from two different modules produces an
  unambiguous binding.
esid: sec-resolveexport
info: |
  When ParseModule encounters `export { mod }` where `mod` is a local binding
  introduced by `import source mod from "..."`, the ExportEntry is
  reclassified as an indirect ExportEntry whose [[ImportName]] is ~source~.

  ResolveExport ( exportName, resolveSet )

    [...]
    For each ExportEntry Record e of module.[[IndirectExportEntries]], do
      a. If e.[[ExportName]] is exportName, then
         [...]
         iii. If e.[[ImportName]] is namespace, then [...]
         iv. Else if e.[[ImportName]] is source, then
            1. Assert: module does not provide the direct binding for this export.
            2. Return ResolvedBinding Record { [[Module]]: importedModule,
               [[BindingName]]: source }.
         [...]

    [...]
    For each ExportEntry Record e of module.[[StarExportEntries]], do
      a. Let importedModule be GetImportedModule(module, e.[[ModuleRequest]]).
      b. Let resolution be ? importedModule.ResolveExport(exportName, resolveSet).
      c. If resolution is ambiguous, return ambiguous.
      d. If resolution is not null, then
         i. If starResolution is null, set starResolution to resolution.
         ii. Else,
            1. Assert: there is more than one * import that includes the requested name.
            2. If resolution.[[Module]] and starResolution.[[Module]] are not the
               same Module Record, return ambiguous.
            3. If resolution.[[BindingName]] is not starResolution.[[BindingName]],
               return ambiguous.

  Both fixtures contain `import source mod from '<module source>'; export { mod };`.
  The host resolves '<module source>' to the same Module Record in both, so
  the two ResolveExport results carry the same [[Module]] and the same
  [[BindingName]] (~source~). The star-export ambiguity check therefore
  succeeds, the named import of `mod` from the re-export resolves to a
  ResolvedBinding Record with [[BindingName]] ~source~, and InitializeEnvironment
  initializes `mod` to the underlying [[ModuleSource]].
features: [source-phase-imports, source-phase-imports-module-source]
flags: [module]
---*/

import { mod } from './namespace-import-source-and-export-reexport_FIXTURE.js';

assert.sameValue(typeof mod, 'object',
  'Re-exported source-phase binding should resolve to a ModuleSource object');
assert(mod instanceof $262.AbstractModuleSource,
  '`mod` should be an instance of %AbstractModuleSource%');
