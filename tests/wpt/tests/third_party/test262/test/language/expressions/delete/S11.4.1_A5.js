// Copyright 2011 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    A strict delete should either succeed, returning true, or it
    should fail by throwing a TypeError. Under no circumstances
    should a strict delete return false.
esid: sec-delete-operator-runtime-semantics-evaluation
description: >
    See if a strict delete returns false when deleting a  non-standard
    property.
flags: [onlyStrict]
---*/

var reNames = Object.getOwnPropertyNames(RegExp);
for (var i = 0, len = reNames.length; i < len; i++) {
  var reName = reNames[i];
  if (reName !== 'prototype') {
    var deleted = 'unassigned';
    try {
      deleted = delete RegExp[reName];
    } catch (err) {
      if (!(err instanceof TypeError)) {
        throw new Test262Error('#1: strict delete threw a non-TypeError: ' + err);
      }
      // fall through
    }
    if (deleted === false) {
      throw new Test262Error('#2: Strict delete returned false');
    }
  }
}
