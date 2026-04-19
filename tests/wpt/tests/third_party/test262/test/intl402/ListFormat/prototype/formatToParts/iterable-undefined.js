// Copyright 2019 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.ListFormat.prototype.format
description: >
    Checks the behavior of Abstract Operation StringListFromIterable
    called by Intl.ListFormat.prototype.formatToParts(undefined).
info: |
    StringListFromIterable
    1. If iterable is undefined, then
      a. Return a new empty List.
features: [Intl.ListFormat]
includes: [compareArray.js]
---*/

let lf = new Intl.ListFormat();

assert.compareArray([], lf.formatToParts(undefined));
