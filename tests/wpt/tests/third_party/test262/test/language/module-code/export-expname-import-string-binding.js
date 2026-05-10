// Copyright (C) 2020 Bradley Farias. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
  ImportSpecifier :
    ModuleExportName `as` IdentifierName
  esid: prod-ImportSpecifier
info: |
  ImportSpecifier :
    ModuleExportName `as` IdentifierName

flags: [module]
features: [arbitrary-module-namespace-names]
---*/
import { "☿" as Ami } from "./export-expname_FIXTURE.js";

assert.sameValue(Ami, globalThis.Mercury);
