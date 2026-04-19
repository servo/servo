// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    A NamedExport without an ExportsList contributes to the list of requested
    modules
esid: sec-moduledeclarationinstantiation
info: |
    [...]
    8. For each String required that is an element of
       module.[[RequestedModules]] do,
       a. NOTE: Before instantiating a module, all of the modules it requested
          must be available. An implementation may perform this test at any
          time prior to this point.
       b. Let requiredModule be ? HostResolveImportedModule(module, required).
       c. Perform ? requiredModule.ModuleDeclarationInstantiation().

    15.2.2.5 Static Semantics: ModuleRequests

    ImportDeclaration : import ImportClause FromClause;

        1. Return ModuleRequests of FromClause.

    15.2.3 Exports

    Syntax

      NamedExport:
        { }
        { ExportsList }
        { ExportsList , }
negative:
  phase: resolution
  type: SyntaxError
flags: [module]
---*/

$DONOTEVALUATE();

export {} from './instn-resolve-empty-export_FIXTURE.js';
