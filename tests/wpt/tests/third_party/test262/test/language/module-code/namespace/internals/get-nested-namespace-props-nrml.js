// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Module namespace object reports properties for all ExportEntries of all
    dependencies.
esid: sec-moduledeclarationinstantiation
info: |
  [...]
  12. For each ImportEntry Record in in module.[[ImportEntries]], do
      a. Let importedModule be ? HostResolveImportedModule(module,
         in.[[ModuleRequest]]).
      b. If in.[[ImportName]] is "*", then
         i. Let namespace be ? GetModuleNamespace(importedModule).
  [...]

  Runtime Semantics: GetModuleNamespace
    3. If namespace is undefined, then
       a. Let exportedNames be ? module.GetExportedNames(« »).
       b. Let unambiguousNames be a new empty List.
       c. For each name that is an element of exportedNames,
          i. Let resolution be ? module.ResolveExport(name, « », « »).
          ii. If resolution is null, throw a SyntaxError exception.
          iii. If resolution is not "ambiguous", append name to
               unambiguousNames.
       d. Let namespace be ModuleNamespaceCreate(module, unambiguousNames).
flags: [module]
features: [export-star-as-namespace-from-module]
---*/

import * as ns from './get-nested-namespace-props-nrml-1_FIXTURE.js';

// Export entries defined by a re-exported as exportns module
assert('starAsVarDecl' in ns.exportns, 'starssVarDecl');
assert('starAsLetDecl' in ns.exportns, 'starSsLetDecl');
assert('starAsConstDecl' in ns.exportns, 'starSsConstDecl');
assert('starAsFuncDecl' in ns.exportns, 'starAsFuncDecl');
assert('starAsGenDecl' in ns.exportns, 'starAsGenDecl');
assert('starAsClassDecl' in ns.exportns, 'starAsClassDecl');
assert('starAsBindingId' in ns.exportns, 'starAsBindingId');
assert('starIdName' in ns.exportns, 'starIdName');
assert('starAsIndirectIdName' in ns.exportns, 'starAsIndirectIdName');
assert('starAsIndirectIdName2' in ns.exportns, 'starAsIndirectIdName2');
assert('namespaceBinding' in ns.exportns, 'namespaceBinding');

// Bindings that were not exported from any module
assert.sameValue('nonExportedVar' in ns.exportns, false, 'nonExportedVar');
assert.sameValue('nonExportedLet' in ns.exportns, false, 'nonExportedLet');
assert.sameValue('nonExportedConst' in ns.exportns, false, 'nonExportedConst');
assert.sameValue('nonExportedFunc' in ns.exportns, false, 'nonExportedFunc');
assert.sameValue('nonExportedGen' in ns.exportns, false, 'nonExportedGen');
assert.sameValue('nonExportedClass' in ns.exportns, false, 'nonExportedClass');
