// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-uint8array.prototype.setfrombase64
description: Handling of final chunks in target.setFromBase64
includes: [compareArray.js]
features: [uint8array-base64, TypedArray]
---*/

// padding
var target = new Uint8Array([255, 255, 255, 255, 255, 255]);
var result = target.setFromBase64('ZXhhZg==');
assert.sameValue(result.read, 8);
assert.sameValue(result.written, 4);
assert.compareArray(target, [101, 120, 97, 102, 255, 255]);

var target = new Uint8Array([255, 255, 255, 255, 255, 255]);
var result = target.setFromBase64('ZXhhZg==', { lastChunkHandling: 'loose' });
assert.sameValue(result.read, 8);
assert.sameValue(result.written, 4);
assert.compareArray(target, [101, 120, 97, 102, 255, 255]);

var target = new Uint8Array([255, 255, 255, 255, 255, 255]);
var result = target.setFromBase64('ZXhhZg==', { lastChunkHandling: 'stop-before-partial' });
assert.sameValue(result.read, 8);
assert.sameValue(result.written, 4);
assert.compareArray(target, [101, 120, 97, 102, 255, 255]);

var target = new Uint8Array([255, 255, 255, 255, 255, 255]);
var result = target.setFromBase64('ZXhhZg==', { lastChunkHandling: 'strict' });
assert.sameValue(result.read, 8);
assert.sameValue(result.written, 4);
assert.compareArray(target, [101, 120, 97, 102, 255, 255]);


// no padding
var target = new Uint8Array([255, 255, 255, 255, 255, 255]);
var result = target.setFromBase64('ZXhhZg');
assert.sameValue(result.read, 6);
assert.sameValue(result.written, 4);
assert.compareArray(target, [101, 120, 97, 102, 255, 255]);

var target = new Uint8Array([255, 255, 255, 255, 255, 255]);
var result = target.setFromBase64('ZXhhZg', { lastChunkHandling: 'loose' });
assert.sameValue(result.read, 6);
assert.sameValue(result.written, 4);
assert.compareArray(target, [101, 120, 97, 102, 255, 255]);

var target = new Uint8Array([255, 255, 255, 255, 255, 255]);
var result = target.setFromBase64('ZXhhZg', { lastChunkHandling: 'stop-before-partial' });
assert.sameValue(result.read, 4);
assert.sameValue(result.written, 3);
assert.compareArray(target, [101, 120, 97, 255, 255, 255]);

assert.throws(SyntaxError, function() {
  var target = new Uint8Array([255, 255, 255, 255, 255, 255]);
  target.setFromBase64('ZXhhZg', { lastChunkHandling: 'strict' });
});


// non-zero padding bits
var target = new Uint8Array([255, 255, 255, 255, 255, 255]);
var result = target.setFromBase64('ZXhhZh==');
assert.sameValue(result.read, 8);
assert.sameValue(result.written, 4);
assert.compareArray(target, [101, 120, 97, 102, 255, 255]);

var target = new Uint8Array([255, 255, 255, 255, 255, 255]);
var result = target.setFromBase64('ZXhhZh==', { lastChunkHandling: 'loose' });
assert.sameValue(result.read, 8);
assert.sameValue(result.written, 4);
assert.compareArray(target, [101, 120, 97, 102, 255, 255]);

var target = new Uint8Array([255, 255, 255, 255, 255, 255]);
var result = target.setFromBase64('ZXhhZh==', { lastChunkHandling: 'stop-before-partial' });
assert.sameValue(result.read, 8);
assert.sameValue(result.written, 4);
assert.compareArray(target, [101, 120, 97, 102, 255, 255]);

assert.throws(SyntaxError, function() {
  var target = new Uint8Array([255, 255, 255, 255, 255, 255]);
  target.setFromBase64('ZXhhZh==', { lastChunkHandling: 'strict' });
});


// non-zero padding bits, no padding
var target = new Uint8Array([255, 255, 255, 255, 255, 255]);
var result = target.setFromBase64('ZXhhZh');
assert.sameValue(result.read, 6);
assert.sameValue(result.written, 4);
assert.compareArray(target, [101, 120, 97, 102, 255, 255]);

var target = new Uint8Array([255, 255, 255, 255, 255, 255]);
var result = target.setFromBase64('ZXhhZh', { lastChunkHandling: 'loose' });
assert.sameValue(result.read, 6);
assert.sameValue(result.written, 4);
assert.compareArray(target, [101, 120, 97, 102, 255, 255]);

var target = new Uint8Array([255, 255, 255, 255, 255, 255]);
var result = target.setFromBase64('ZXhhZh', { lastChunkHandling: 'stop-before-partial' });
assert.sameValue(result.read, 4);
assert.sameValue(result.written, 3);
assert.compareArray(target, [101, 120, 97, 255, 255, 255]);

assert.throws(SyntaxError, function() {
  var target = new Uint8Array([255, 255, 255, 255, 255, 255]);
  target.setFromBase64('ZXhhZh', { lastChunkHandling: 'strict' });
});


// partial padding
assert.throws(SyntaxError, function() {
  var target = new Uint8Array([255, 255, 255, 255, 255, 255]);
  target.setFromBase64('ZXhhZg=');
});

assert.throws(SyntaxError, function() {
  var target = new Uint8Array([255, 255, 255, 255, 255, 255]);
  target.setFromBase64('ZXhhZg=', { lastChunkHandling: 'loose' });
});

var target = new Uint8Array([255, 255, 255, 255, 255, 255]);
var result = target.setFromBase64('ZXhhZg=', { lastChunkHandling: 'stop-before-partial' });
assert.sameValue(result.read, 4);
assert.sameValue(result.written, 3);
assert.compareArray(target, [101, 120, 97, 255, 255, 255]);

assert.throws(SyntaxError, function() {
  var target = new Uint8Array([255, 255, 255, 255, 255, 255]);
  target.setFromBase64('ZXhhZg=', { lastChunkHandling: 'strict' });
});


// excess padding
assert.throws(SyntaxError, function() {
  var target = new Uint8Array([255, 255, 255, 255, 255, 255]);
  target.setFromBase64('ZXhhZg===');
});

assert.throws(SyntaxError, function() {
  var target = new Uint8Array([255, 255, 255, 255, 255, 255]);
  target.setFromBase64('ZXhhZg===', { lastChunkHandling: 'loose' });
});

assert.throws(SyntaxError, function() {
  var target = new Uint8Array([255, 255, 255, 255, 255, 255]);
  target.setFromBase64('ZXhhZg===', { lastChunkHandling: 'stop-before-partial' });
});

assert.throws(SyntaxError, function() {
  var target = new Uint8Array([255, 255, 255, 255, 255, 255]);
  target.setFromBase64('ZXhhZg===', { lastChunkHandling: 'strict' });
});
