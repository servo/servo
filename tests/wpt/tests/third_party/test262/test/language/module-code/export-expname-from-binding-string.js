// Copyright (C) 2020 Bradley Farias. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
  ExportSpecifier : ModuleExportName
  esid: prod-ExportSpecifier
info: |
  ExportFromClause :
    NamedExports[+From]

  ExportSpecifier[From] :
    IdentifierName `as` ModuleExportName

  ModuleExportName : StringLiteral

flags: [module]
features: [arbitrary-module-namespace-names]
---*/
import * as Scouts from "./export-expname-from-binding-string.js";
export { Mercury as "☿" } from "./export-expname_FIXTURE.js";

assert.sameValue(Scouts.Mercury, undefined);
assert.sameValue(Scouts["☿"], globalThis.Mercury);
