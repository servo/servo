// Copyright (C) 2019 Adrian Heine. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: A default export cannot be provided by an export * or export * from "mod" declaration
esid: sec-static-semantics-exportentriesformodule
info: |
    15.2..3.6 Static Semantics: ExportEntriesForModule

    [...]

    ExportFromClause : * as IdentifierName

    1. Let exportName be the StringValue of IdentifierName.
    2. Let entry be the ExportEntry Record { [[ModuleRequest]]: module, [[ImportName]]: "*", [[LocalName]]: null, [[ExportName]]: exportName }.
    3. Return a new List containing entry.

flags: [module]
features: [export-star-as-namespace-from-module]
---*/

export * as default from './export-star-as-dflt_FIXTURE.js';
import Self from './export-star-as-dflt.js';
import { default as named } from './export-star-as-dflt.js';
import * as ns from './export-star-as-dflt.js';

assert.sameValue(Self.x, 1, 'Module was re-exported under the name `default`');
assert.sameValue(named.x, 1, 'named binding was re-exported under the name `default`');
assert.sameValue(ns.default.x, 1, 'namespace was re-exported under the name `default`');
