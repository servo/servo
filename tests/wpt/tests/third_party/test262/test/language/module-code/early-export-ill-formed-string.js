// Copyright (C) 2020 Bradley Farias. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Ill formed unicode cannot be an exported name
esid: sec-module-semantics
info: |
    ModuleExportName : StringLiteral

    It is a Syntax Error if IsStringWellFormedUnicode of the StringValue of StringLiteral is *false*.
flags: [module]
negative:
  phase: parse
  type: SyntaxError
features: [arbitrary-module-namespace-names]
---*/

$DONOTEVALUATE();

// 🌙 is '\uD83C\uDF19'
export {Moon as "\uD83C",} from "./early-export-ill-formed-string.js";
export {"\uD83C"} from "./early-export-ill-formed-string.js";
import {'\uD83C' as Usagi} from "./early-export-ill-formed-string.js";

function Moon() {}
