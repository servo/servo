// META: global=worker

'use strict';

test(t => {
  assert_throws(new TypeError(), () => new DecompressionStream('a'), 'constructor should throw');
}, '"a" should cause the constructor to throw');

test(t => {
  assert_throws(new TypeError(), () => new DecompressionStream(), 'constructor should throw');
}, 'no input should cause the constructor to throw');

test(t => {
  assert_throws(new Error(), () => new DecompressionStream({ toString() { throw Error(); } }), 'constructor should throw');
}, 'non-string input should cause the constructor to throw');
