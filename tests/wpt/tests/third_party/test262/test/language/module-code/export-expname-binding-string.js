// Copyright (C) 2020 Bradley Farias. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Module Namespace Objects can have non-Identifier properties
esid: prod-ExportSpecifier
info: |
  ExportSpecifier[From] :
    IdentifierName `as` ModuleExportName

  ModuleExportName : StringLiteral

flags: [module]
features: [arbitrary-module-namespace-names]
---*/
import * as Scouts from "./export-expname_FIXTURE.js";

assert.sameValue(Scouts.Mercury, globalThis.Mercury);
assert.sameValue(Scouts["☿"], globalThis.Mercury);
