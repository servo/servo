'use strict';
const { URL } = require('url');
const { parseFromString } = require('../lib/parser.js');
const { resolve } = require('../lib/resolver.js');
const { BUILT_IN_MODULE_SCHEME } = require('../lib/utils.js');

const mapBaseURL = new URL('https://example.com/app/index.html');
const scriptURL = new URL('https://example.com/js/app.mjs');

const BLANK = `${BUILT_IN_MODULE_SCHEME}:blank`;

function makeResolveUnderTest(mapString) {
  const map = parseFromString(mapString, mapBaseURL);
  return specifier => resolve(specifier, map, scriptURL);
}

describe('Fallbacks that are not [built-in, fetch scheme]', () => {
  const resolveUnderTest = makeResolveUnderTest(`{
    "imports": {
      "bad1": [
        "${BLANK}",
        "${BLANK}"
      ],
      "bad2": [
        "${BLANK}",
        "/bad2-1.mjs",
        "/bad2-2.mjs"
      ],
      "bad3": [
        "/bad3-1.mjs",
        "/bad3-2.mjs"
      ]
    }
  }`);

  it('should fail for [built-in, built-in]', () => {
    expect(() => resolveUnderTest('bad1')).toThrow(/not yet implemented/);
  });

  it('should fail for [built-in, fetch scheme, fetch scheme]', () => {
    expect(() => resolveUnderTest('bad2')).toThrow(/not yet implemented/);
  });

  it('should fail for [fetch scheme, fetch scheme]', () => {
    expect(() => resolveUnderTest('bad3')).toThrow(/not yet implemented/);
  });
});
