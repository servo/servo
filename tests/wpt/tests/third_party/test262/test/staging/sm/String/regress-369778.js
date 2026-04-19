// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  RegExpStatics::makeMatch should make an undefined value when the last match had an undefined capture.
info: bugzilla.mozilla.org/show_bug.cgi?id=369778
esid: pending
---*/

/* Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/publicdomain/zero/1.0/ */

var expected = undefined;
var actual;

'x'.replace(/x(.)?/g, function(m, group) { actual = group; })

assert.sameValue(expected, actual)
