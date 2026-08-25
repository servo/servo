// Copyright 2019 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: The "indices" property is an Array.
esid: sec-regexpbuiltinexec
features: [regexp-match-indices]
info: |
  RegExpBuiltinExec ( R, S )
    
    8. If _flags_ contains `"d"`, let _hasIndices_ be *true*, else let _hasIndices_ be *false*.
    36. If _hasIndices_ is *true*, then
      b. Perform ! CreateDataProperty(_A_, `"indices"`, _indicesArray_).
---*/

let match = /a/.exec("a");

// `indices` is not defined by default
assert(!match.hasOwnProperty("indices"));
