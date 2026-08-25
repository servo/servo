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

import * as ns from './instn-star-props-nrml-1_FIXTURE.js';

// Export entries defined by a directly-imported module
assert('localVarDecl' in ns, 'localVarDecl');
assert('localLetDecl' in ns, 'localLetDecl');
assert('localConstDecl' in ns, 'localConstDecl');
assert('localFuncDecl' in ns, 'localFuncDecl');
assert('localGenDecl' in ns, 'localGenDecl');
assert('localClassDecl' in ns, 'localClassDecl');
assert('localBindingId' in ns, 'localBindingId');
assert('localIdName' in ns, 'localIdName');
assert('indirectIdName' in ns, 'indirectIdName');
assert('indirectIdName2' in ns, 'indirectIdName2');
assert('namespaceBinding' in ns, 'namespaceBinding');

// Export entries defined by a re-exported module
assert('starVarDecl' in ns, 'starVarDecl');
assert('starLetDecl' in ns, 'starLetDecl');
assert('starConstDecl' in ns, 'starConstDecl');
assert('starFuncDecl' in ns, 'starFuncDecl');
assert('starGenDecl' in ns, 'starGenDecl');
assert('starClassDecl' in ns, 'starClassDecl');
assert('starBindingId' in ns, 'starBindingId');
assert('starIdName' in ns, 'starIdName');
assert('starIndirectIdName' in ns, 'starIndirectIdName');
assert('starIndirectIdName2' in ns, 'starIndirectIdName2');
assert('starIndirectNamespaceBinding' in ns, 'starIndirectNamespaceBinding');

// Bindings that were not exported from any module
assert.sameValue('nonExportedVar1' in ns, false, 'nonExportedVar1');
assert.sameValue('nonExportedVar2' in ns, false, 'nonExportedVar2');
assert.sameValue('nonExportedLet1' in ns, false, 'nonExportedLet1');
assert.sameValue('nonExportedLet2' in ns, false, 'nonExportedLet2');
assert.sameValue('nonExportedConst1' in ns, false, 'nonExportedConst1');
assert.sameValue('nonExportedConst2' in ns, false, 'nonExportedConst2');
assert.sameValue('nonExportedFunc1' in ns, false, 'nonExportedFunc1');
assert.sameValue('nonExportedFunc2' in ns, false, 'nonExportedFunc2');
assert.sameValue('nonExportedGen1' in ns, false, 'nonExportedGen1');
assert.sameValue('nonExportedGen2' in ns, false, 'nonExportedGen2');
assert.sameValue('nonExportedClass1' in ns, false, 'nonExportedClass1');
assert.sameValue('nonExportedClass2' in ns, false, 'nonExportedClass2');
