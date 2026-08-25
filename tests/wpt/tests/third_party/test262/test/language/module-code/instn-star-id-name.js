// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Namespace object reports properties for any valid exported IdentifierName.
esid: sec-imports
info: |
    [...]
    5. For each ExportEntry Record e in module.[[LocalExportEntries]], do
       a. Assert: module provides the direct binding for this export.
       b. Append e.[[ExportName]] to exportedNames.
    [...]
flags: [module]
---*/

var _if = null;
var _import = null;
var _export = null;
var _await = null;
var _arguments = null;
var _eval = null;
var _default = null;
var as = null;

export {
    _if as if,
    _import as import,
    _export as export,
    _await as await,
    _arguments as arguments,
    _eval as eval,
    _default as default,
    as as as
  };

import * as ns from './instn-star-id-name.js';

assert('if' in ns, 'property name: if');
assert('import' in ns, 'property name: import');
assert('export' in ns, 'property name: export');
assert('await' in ns, 'property name: await');
assert('arguments' in ns, 'property name: arguments');
assert('eval' in ns, 'property name: eval');
assert('default' in ns, 'property name: default');
assert('as' in ns, 'property name: as');
