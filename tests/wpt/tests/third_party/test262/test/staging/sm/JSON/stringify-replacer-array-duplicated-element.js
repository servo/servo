/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Better/more correct handling for replacer arrays with getter array index properties
info: bugzilla.mozilla.org/show_bug.cgi?id=648471
esid: pending
---*/

var bigOdd = Math.pow(2, 50) + 1;

function two()
{
  return Math.random() < 0.5 ? 2 : "2";
}

assert.sameValue(JSON.stringify({ 1: 1 }, [1, 1]), '{"1":1}');

assert.sameValue(JSON.stringify({ 1: 1 }, [1, "1"]), '{"1":1}');

assert.sameValue(JSON.stringify({ 1: 1 }, [1, bigOdd % two()]), '{"1":1}');

assert.sameValue(JSON.stringify({ 1: 1 }, ["1", 1]), '{"1":1}');

assert.sameValue(JSON.stringify({ 1: 1 }, ["1", "1"]), '{"1":1}');

assert.sameValue(JSON.stringify({ 1: 1 }, ["1", bigOdd % two()]), '{"1":1}');

assert.sameValue(JSON.stringify({ 1: 1 }, [bigOdd % two(), 1]), '{"1":1}');

assert.sameValue(JSON.stringify({ 1: 1 }, [bigOdd % two(), "1"]), '{"1":1}');

assert.sameValue(JSON.stringify({ 1: 1 }, [bigOdd % two(), bigOdd % two()]), '{"1":1}');


assert.sameValue(JSON.stringify({ 1: 1 }, [1, new String(1)]), '{"1":1}');

assert.sameValue(JSON.stringify({ 1: 1 }, [1, new Number(1)]), '{"1":1}');

assert.sameValue(JSON.stringify({ 1: 1 }, ["1", new Number(1)]), '{"1":1}');

assert.sameValue(JSON.stringify({ 1: 1 }, ["1", new String(1)]), '{"1":1}');

assert.sameValue(JSON.stringify({ 1: 1 }, [bigOdd % two(), new Number(1)]), '{"1":1}');

assert.sameValue(JSON.stringify({ 1: 1 }, [bigOdd % two(), new String(1)]), '{"1":1}');


assert.sameValue(JSON.stringify({ 1: 1 }, [new String(1), new String(1)]), '{"1":1}');

assert.sameValue(JSON.stringify({ 1: 1 }, [new String(1), new Number(1)]), '{"1":1}');

assert.sameValue(JSON.stringify({ 1: 1 }, [new Number(1), new String(1)]), '{"1":1}');

assert.sameValue(JSON.stringify({ 1: 1 }, [new Number(1), new Number(1)]), '{"1":1}');
