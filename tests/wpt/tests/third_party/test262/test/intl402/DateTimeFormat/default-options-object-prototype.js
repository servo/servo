// Copyright (C) 2017 Daniel Ehrenberg. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-todatetimeoptions
description: >
  Monkey-patching Object.prototype does not change the default
  options for DateTimeFormat as a null prototype is used.
info: |
  ToDateTimeOptions ( options, required, defaults )

  1. If options is undefined, let options be null; otherwise let options be ? ToObject(options).
  1. Let options be ObjectCreate(options).
---*/

let defaultYear = new Intl.DateTimeFormat("en").resolvedOptions().year;

Object.prototype.year = "2-digit";
let formatter = new Intl.DateTimeFormat("en");
assert.sameValue(formatter.resolvedOptions().year, defaultYear);
