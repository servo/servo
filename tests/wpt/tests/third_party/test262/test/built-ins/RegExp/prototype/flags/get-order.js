// Copyright (C) 2017 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-regexp.prototype.flags
description: Gets are performed in specified order
info: |
  get RegExp.prototype.flags

  [...]
  4. let hasIndices be ToBoolean(? Get(R, "hasIndices"))
  6. Let global be ToBoolean(? Get(R, "global")).
  8. Let ignoreCase be ToBoolean(? Get(R, "ignoreCase")).
  10. Let multiline be ToBoolean(? Get(R, "multiline")).
  12. Let dotAll be ToBoolean(? Get(R, "dotAll")).
  14. Let unicode be ToBoolean(? Get(R, "unicode")).
  18. Let sticky be ToBoolean(? Get(R, "sticky")).
features: [regexp-dotall, regexp-match-indices]
---*/

var calls = '';
var re = {
  get hasIndices() {
    calls += 'd';
  },
  get global() {
    calls += 'g';
  },
  get ignoreCase() {
    calls += 'i';
  },
  get multiline() {
    calls += 'm';
  },
  get dotAll() {
    calls += 's';
  },
  get unicode() {
    calls += 'u';
  },
  get sticky() {
    calls += 'y';
  },
};

var get = Object.getOwnPropertyDescriptor(RegExp.prototype, 'flags').get;

get.call(re);
assert.sameValue(calls, 'dgimsuy');
