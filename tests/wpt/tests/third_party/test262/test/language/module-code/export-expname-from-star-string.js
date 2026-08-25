// Copyright (C) 2020 Bradley Farias. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
  ExportFromClause : `*` `as` ModuleExportName
  esid: prod-ExportFromClause
info: |
  ExportFromClause :
    `*` `as` ModuleExportName

  ModuleExportName : StringLiteral

flags: [module]
features: [arbitrary-module-namespace-names]
---*/
import * as Scouts from "./export-expname-from-star-string.js";
export * as "All" from "./export-expname_FIXTURE.js";

assert.sameValue(Scouts["☿"], undefined);
assert.sameValue(Scouts.All["☿"], globalThis.Mercury);
