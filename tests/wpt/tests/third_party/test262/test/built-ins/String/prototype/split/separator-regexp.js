// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.split
description: Separator is a regexp
info: |
  String.prototype.split ( separator, limit )

  If separator is neither undefined nor null, then
    Let splitter be ? GetMethod(separator, @@split).
    If splitter is not undefined, then
      Return ? Call(splitter, separator, « O, limit »).

  RegExp.prototype [ @@split ] ( string, limit )

  Let C be ? SpeciesConstructor(rx, %RegExp%).

includes: [compareArray.js]
---*/

assert.compareArray("x".split(/^/), ["x"], '"x".split(/^/) must return ["x"]');
assert.compareArray("x".split(/$/), ["x"], '"x".split(/$/) must return ["x"]');
assert.compareArray("x".split(/.?/), ["", ""], '"x".split(/.?/) must return ["", ""]');
assert.compareArray("x".split(/.*/), ["", ""], '"x".split(/.*/) must return ["", ""]');
assert.compareArray("x".split(/.+/), ["", ""], '"x".split(/.+/) must return ["", ""]');
assert.compareArray("x".split(/.*?/), ["x"], '"x".split(/.*?/) must return ["x"]');
assert.compareArray("x".split(/.{1}/), ["", ""], '"x".split(/.{1}/) must return ["", ""]');
assert.compareArray("x".split(/.{1,}/), ["", ""], '"x".split(/.{1,}/) must return ["", ""]');
assert.compareArray("x".split(/.{1,2}/), ["", ""], '"x".split(/.{1,2}/) must return ["", ""]');
assert.compareArray("x".split(/()/), ["x"], '"x".split(/()/) must return ["x"]');
assert.compareArray("x".split(/./), ["",""], '"x".split(/./) must return ["",""]');
assert.compareArray("x".split(/(?:)/), ["x"], '"x".split(/(?:)/) must return ["x"]');
assert.compareArray("x".split(/(...)/), ["x"], '"x".split(/(...)/) must return ["x"]');
assert.compareArray("x".split(/(|)/), ["x"], '"x".split(/(|)/) must return ["x"]');
assert.compareArray("x".split(/[]/), ["x"], '"x".split(/[]/) must return ["x"]');
assert.compareArray("x".split(/[^]/), ["", ""], '"x".split(/[^]/) must return ["", ""]');
assert.compareArray("x".split(/[.-.]/), ["x"], '"x".split(/[.-.]/) must return ["x"]');
assert.compareArray("x".split(/\0/), ["x"], '"x".split(/\\0/) must return ["x"]');
assert.compareArray("x".split(/\b/), ["x"], '"x".split(/\\b/) must return ["x"]');
assert.compareArray("x".split(/\B/), ["x"], '"x".split(/\\B/) must return ["x"]');
assert.compareArray("x".split(/\d/), ["x"], '"x".split(/\\d/) must return ["x"]');
assert.compareArray("x".split(/\D/), ["", ""], '"x".split(/\\D/) must return ["", ""]');
assert.compareArray("x".split(/\n/), ["x"], '"x".split(/\\n/) must return ["x"]');
assert.compareArray("x".split(/\r/), ["x"], '"x".split(/\\r/) must return ["x"]');
assert.compareArray("x".split(/\s/), ["x"], '"x".split(/\\s/) must return ["x"]');
assert.compareArray("x".split(/\S/), ["", ""], '"x".split(/\\S/) must return ["", ""]');
assert.compareArray("x".split(/\v/), ["x"], '"x".split(/\\v/) must return ["x"]');
assert.compareArray("x".split(/\w/), ["", ""], '"x".split(/\\w/) must return ["", ""]');
assert.compareArray("x".split(/\W/), ["x"], '"x".split(/\\W/) must return ["x"]');
assert.compareArray("x".split(/\k<x>/), ["x"], '"x".split(/\\k<x>/) must return ["x"]');
assert.compareArray("x".split(/\xA0/), ["x"], '"x".split(/\\xA0/) must return ["x"]');
assert.compareArray("x".split(/\XA0/), ["x"], '"x".split(/\\XA0/) must return ["x"]');
assert.compareArray("x".split(/\ddd/), ["x"], '"x".split(/\\ddd/) must return ["x"]');
assert.compareArray("x".split(/\cY/), ["x"], '"x".split(/\\cY/) must return ["x"]');
assert.compareArray("x".split(/[\b]/), ["x"], '"x".split(/[\\b]/) must return ["x"]');
assert.compareArray("x".split(/\x/), ["", ""], '"x".split(/\\x/) must return ["", ""]');
assert.compareArray("x".split(/\X/), ["x"], '"x".split(/\\X/) must return ["x"]');
