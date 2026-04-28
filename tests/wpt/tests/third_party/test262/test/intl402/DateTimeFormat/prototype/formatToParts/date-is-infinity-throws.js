// Copyright 2016 Leonardo Balter. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: >
  Throws a RangeError if date arg is cast to an Infinity value
info: |
  Intl.DateTimeFormat.prototype.formatToParts ([ date ])

  4. If _date_ is not provided or is *undefined*, then
    a. Let _x_ be *%Date_now%*().
  5. Else,
    a. Let _x_ be ? ToNumber(_date_).
  6. Return ? FormatDateTimeToParts(_dtf_, _x_).

  FormatDateTimeToParts(dateTimeFormat, x)

  1. Let _parts_ be ? PartitionDateTimePattern(_dateTimeFormat_, _x_).

  PartitionDateTimePattern (dateTimeFormat, x)

  1. If _x_ is not a finite Number, throw a *RangeError* exception.
---*/

var dtf = new Intl.DateTimeFormat(["pt-BR"]);

assert.throws(RangeError, function() {
  dtf.formatToParts(Infinity);
}, "+Infinity");

assert.throws(RangeError, function() {
  dtf.formatToParts(-Infinity);
}, "-Infinity");
