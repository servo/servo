// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-resolveexport
description: >
  Re-exported deferred namespace preserves deferred semantics
info: |
  ResolveExport ( _exportName_ )
    1. ...
    1. For each ExportEntry Record _e_ of _module_.[[IndirectExportEntries]], do
      1. If _e_.[[ExportName]] is _exportName_, then
        1. Assert: _e_.[[ModuleRequest]] is not *null*.
        1. Let _importedModule_ be GetImportedModule(_module_, _e_.[[ModuleRequest]]).
        1. If _e_.[[ImportName]] is ~all~, then
          1. Assert: _module_ does not provide the direct binding for this export.
          1. If _e_.[[ModuleRequest]].[[Phase]] is ~defer~, then
            1. Return ResolvedBinding Record { [[Module]]: _importedModule_,
               [[BindingName]]: ~deferred-namespace~ }.
          1. Else,
            1. Assert: _e_.[[Phase]] is ~evaluation~.
            1. Return ResolvedBinding Record { [[Module]]: _importedModule_,
               [[BindingName]]: ~namespace~ }.
flags: [module]
features: [import-defer]
includes: [compareArray.js]
---*/

import { ns } from "./deferred_ns_export_FIXTURE.js";

assert.compareArray(globalThis.evaluations, ["reexport"],
  "deferred module should not be evaluated");

assert.sameValue(ns[Symbol.toStringTag], "Deferred Module",
  "'ns' should be a deferred namespace object");

assert.sameValue(ns.foo, 42);
assert.compareArray(globalThis.evaluations, ["reexport", "dep"],
  "deferred module should be evaluated after property access");
