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

assert.sameValue(JSON.stringify({ 3: 3, 4: 4 },
                        ["3", { toString: function() { return "4" } }]),
         '{"3":3}');

assert.sameValue(JSON.stringify({ 3: 3, true: 4 }, ["3", true]),
         '{"3":3}');

assert.sameValue(JSON.stringify({ 3: 3, true: 4 }, ["3", "true", true]),
         '{"3":3,"true":4}');

assert.sameValue(JSON.stringify({ 3: 3, true: 4 }, ["3", true, "true"]),
         '{"3":3,"true":4}');

assert.sameValue(JSON.stringify({ 3: 3, false: 4 }, ["3", false]),
         '{"3":3}');

assert.sameValue(JSON.stringify({ 3: 3, false: 4 }, ["3", "false", false]),
         '{"3":3,"false":4}');

assert.sameValue(JSON.stringify({ 3: 3, false: 4 }, ["3", false, "false"]),
         '{"3":3,"false":4}');

assert.sameValue(JSON.stringify({ 3: 3, undefined: 4 }, ["3", undefined]),
         '{"3":3}');

assert.sameValue(JSON.stringify({ 3: 3, undefined: 4 }, ["3", "undefined", undefined]),
         '{"3":3,"undefined":4}');

assert.sameValue(JSON.stringify({ 3: 3, undefined: 4 }, ["3", undefined, "undefined"]),
         '{"3":3,"undefined":4}');

assert.sameValue(JSON.stringify({ 3: 3, null: 4 }, ["3", null]),
         '{"3":3}');

assert.sameValue(JSON.stringify({ 3: 3, null: 4 }, ["3", "null", null]),
         '{"3":3,"null":4}');

assert.sameValue(JSON.stringify({ 3: 3, null: 4 }, ["3", null, "null"]),
         '{"3":3,"null":4}');
