// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: The `export` declaration may not appear within a ScriptBody
esid: sec-scripts
negative:
  phase: parse
  type: SyntaxError
info: |
     A.5 Scripts and Modules

     Script:
         ScriptBodyopt

     ScriptBody:
         StatementList
---*/

$DONOTEVALUATE();

export default null;
