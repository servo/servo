// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Exporting the same namespace object twice with `export * as foo` produces an unambiguous binding
esid: sec-source-text-module-record-initialize-environment
info: |
   [...]
   7. For each ImportEntry Record in of module.[[ImportEntries]], do
   a. Let importedModule be GetImportedModule(module, in.[[ModuleRequest]]).
   b. If in.[[ImportName]] is namespace-object, then
      i. Let namespace be GetModuleNamespace(importedModule).
      ii. Perform ! env.CreateImmutableBinding(in.[[LocalName]], true).
      iii. Perform ! env.InitializeBinding(in.[[LocalName]], namespace).
   c. Else,
      i. Let resolution be importedModule.ResolveExport(in.[[ImportName]]).
      ii. If resolution is either null or ambiguous, throw a SyntaxError exception.

   Table 59 (Informative): Export Forms Mappings to ExportEntry Records

   Export Statement Form 	      [[ExportName]] 	   [[ModuleRequest]] 	   [[ImportName]] 	   [[LocalName]]
   export {x}; 	               "x" 	               null 	                  null 	               "x"
   export * as ns from "mod"; 	"ns" 	               "mod"                   all                  null

   16.2.1.7.1 ParseModule ( sourceText, realm, hostDefined )
      [...]
      10. For each ExportEntry Record ee of exportEntries, do
         1. If ee.[[ModuleRequest]] is null, then
            i. If importedBoundNames does not contain ee.[[LocalName]], then
              1. Append ee to localExportEntries.
            ii. Else,
              1. Let ie be the element of importEntries whose [[LocalName]] is ee.[[LocalName]].
              2. If ie.[[ImportName]] is namespace-object, then
                a. NOTE: This is a re-export of an imported module namespace object.
                b. Append the ExportEntry Record { [[ModuleRequest]]: ie.[[ModuleRequest]], [[ImportName]]: ~all~, [[LocalName]]: *null*, [[ExportName]]: ee.[[ExportName]] } to indirectExportEntries.
              3. Else,
                a. NOTE: This is a re-export of a single name.
                b. Append the ExportEntry Record { [[ModuleRequest]]: ie.[[ModuleRequest]],
                [[ImportName]]: ie.[[ImportName]], [[LocalName]]: null, [[ExportName]]:
                ee.[[ExportName]] } to indirectExportEntries.
         2. Else if ee.[[ImportName]] is all-but-default, then
            [...]
         3. Else,
            a. Append ee to indirectExportEntries.

   15.2.1.16.3 ResolveExport

   [...]
   6. For each ExportEntry Record e of module.[[IndirectExportEntries]], do
   a. If e.[[ExportName]] is exportName, then
      i. Assert: e.[[ModuleRequest]] is not null.
      ii. Let importedModule be GetImportedModule(module, e.[[ModuleRequest]]).
      iii. If e.[[ImportName]] is all, then
         1. Assert: module does not provide the direct binding for this export.
         2. Return ResolvedBinding Record { [[Module]]: importedModule, [[BindingName]]: namespace }.
   [...]
   9. Let starResolution be null.
   10. For each ExportEntry Record e in module.[[StarExportEntries]], do
      a. Let importedModule be GetImportedModule(module,
         e.[[ModuleRequest]]).
      b. Let resolution be ? importedModule.ResolveExport(exportName,
         resolveSet, exportStarSet).
      c. If resolution is ~ambiguous~, return ~ambiguous~.
      d. If resolution is not null, then
         i. If starResolution is null, let starResolution be resolution.
         ii. Else,
            1. Assert: there is more than one * import that includes the
               requested name.
            2. If _resolution_.[[Module]] and _starResolution_.[[Module]] are
               not the same Module Record, return ~ambiguous~.
            3. If _resolution_.[[BindingName]] is not _starResolution_.[[BindingName]],
               return ~ambiguous~.
flags: [module]
---*/

export * from "./namespace-export-star-as-from-1_FIXTURE.js";
export * from "./namespace-export-star-as-from-2_FIXTURE.js";

import { foo } from './namespace-unambiguous-if-export-star-as-from.js';

assert.sameValue(typeof foo, 'object');
