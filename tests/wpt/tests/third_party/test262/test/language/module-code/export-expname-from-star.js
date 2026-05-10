// Copyright (C) 2020 Bradley Farias. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
  ExportFromClause : `*`
  esid: prod-ExportFromClause
info: |
  ExportFromClause :
    `*`

flags: [module]
features: [arbitrary-module-namespace-names]
---*/
import * as Scouts from "./export-expname-from-star.js";
export * from "./export-expname_FIXTURE.js";

assert.sameValue(Scouts["☿"], globalThis.Mercury);
