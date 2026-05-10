// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 15.2.1.1
description: >
    It is a Syntax Error if the ExportedNames of ModuleItemList contains any
    duplicate entries.
flags: [module]
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

export default var x = null;
export default var x = null;
