'use strict';
const { URL } = require('url');
const { parseFromString } = require('../lib/parser.js');
const { resolve } = require('../lib/resolver.js');
const { BUILT_IN_MODULE_SCHEME } = require('../lib/utils.js');

const mapBaseURL = new URL('https://example.com/app/index.html');
const scriptURL = new URL('https://example.com/js/app.mjs');

const BLANK = `${BUILT_IN_MODULE_SCHEME}:blank`;
const NONE = `${BUILT_IN_MODULE_SCHEME}:none`;

function makeResolveUnderTest(mapString) {
  const map = parseFromString(mapString, mapBaseURL);
  return specifier => resolve(specifier, map, scriptURL);
}

describe('Unmapped built-in module specifiers', () => {
  const resolveUnderTest = makeResolveUnderTest(`{}`);

  it(`should resolve "${BLANK}" to "${BLANK}"`, () => {
    expect(resolveUnderTest(BLANK)).toMatchURL(BLANK);
  });

  it(`should error resolving "${NONE}"`, () => {
    expect(() => resolveUnderTest(NONE)).toThrow(TypeError);
  });
});

describe('Remapping built-in module specifiers', () => {
  it('should remap built-in modules', () => {
    const resolveUnderTest = makeResolveUnderTest(`{
      "imports": {
        "${BLANK}": "./blank.mjs",
        "${NONE}": "./none.mjs"
      }
    }`);

    expect(resolveUnderTest(BLANK)).toMatchURL('https://example.com/app/blank.mjs');
    expect(resolveUnderTest(NONE)).toMatchURL('https://example.com/app/none.mjs');
  });

  it('should remap built-in modules with slashes', () => {
    const resolveUnderTest = makeResolveUnderTest(`{
      "imports": {
        "${BLANK}/": "./blank-slash/",
        "${BLANK}/foo": "./blank-foo.mjs",
        "${NONE}/": "./none-slash/",
        "${NONE}/foo": "./none-foo.mjs"
      }
    }`);

    expect(resolveUnderTest(`${BLANK}/`)).toMatchURL('https://example.com/app/blank-slash/');
    expect(resolveUnderTest(`${BLANK}/foo`)).toMatchURL('https://example.com/app/blank-foo.mjs');
    expect(resolveUnderTest(`${BLANK}/bar`)).toMatchURL('https://example.com/app/blank-slash/bar');
    expect(resolveUnderTest(`${NONE}/`)).toMatchURL('https://example.com/app/none-slash/');
    expect(resolveUnderTest(`${NONE}/foo`)).toMatchURL('https://example.com/app/none-foo.mjs');
    expect(resolveUnderTest(`${NONE}/bar`)).toMatchURL('https://example.com/app/none-slash/bar');
  });

  it('should remap built-in modules with fallbacks', () => {
    const resolveUnderTest = makeResolveUnderTest(`{
      "imports": {
        "${BLANK}": ["${BLANK}", "./blank.mjs"],
        "${NONE}": ["${NONE}", "./none.mjs"]
      }
    }`);

    expect(resolveUnderTest(BLANK)).toMatchURL(BLANK);
    expect(resolveUnderTest(NONE)).toMatchURL('https://example.com/app/none.mjs');
  });

  it('should remap built-in modules with slashes and fallbacks', () => {
    // NOTE: `${BLANK}/for-testing` is not per spec, just for these tests.
    // See resolver.js.
    const resolveUnderTest = makeResolveUnderTest(`{
      "imports": {
        "${BLANK}/": ["${BLANK}/", "./blank/"],
        "${BLANK}/for-testing": ["${BLANK}/for-testing", "./blank-for-testing-special"],
        "${NONE}/": ["${NONE}/", "./none/"],
        "${NONE}/foo": ["${NONE}/foo", "./none-foo-special"]
      }
    }`);

    // Built-in modules only resolve for exact matches, so this will trigger the fallback.
    expect(resolveUnderTest(`${BLANK}/`)).toMatchURL('https://example.com/app/blank/');
    expect(resolveUnderTest(`${BLANK}/foo`)).toMatchURL('https://example.com/app/blank/foo');

    // This would fall back in a real implementation; it's only because we've gone against
    // spec in the reference implementation (to make this testable) that this maps.
    expect(resolveUnderTest(`${BLANK}/for-testing`)).toMatchURL(`${BLANK}/for-testing`);

    expect(resolveUnderTest(`${NONE}/`)).toMatchURL('https://example.com/app/none/');
    expect(resolveUnderTest(`${NONE}/bar`)).toMatchURL('https://example.com/app/none/bar');
    expect(resolveUnderTest(`${NONE}/foo`)).toMatchURL('https://example.com/app/none-foo-special');
  });
});

describe('Remapping to built-in modules', () => {
  const resolveUnderTest = makeResolveUnderTest(`{
    "imports": {
      "blank": "${BLANK}",
      "/blank": "${BLANK}",
      "/blank/": "${BLANK}/",
      "/blank-for-testing": "${BLANK}/for-testing",
      "none": "${NONE}",
      "/none": "${NONE}"
    }
  }`);

  it(`should remap to "${BLANK}"`, () => {
    expect(resolveUnderTest('blank')).toMatchURL(BLANK);
    expect(resolveUnderTest('/blank')).toMatchURL(BLANK);
  });

  it(`should fail when remapping to "${BLANK}/"`, () => {
    expect(() => resolveUnderTest('/blank/')).toThrow(TypeError);
  });

  it(`should remap to "${BLANK}/for-testing"`, () => {
    expect(resolveUnderTest('/blank/for-testing')).toMatchURL(`${BLANK}/for-testing`);
    expect(resolveUnderTest('/blank-for-testing')).toMatchURL(`${BLANK}/for-testing`);
  });

  it(`should remap to "${BLANK}" for URL-like specifiers`, () => {
    expect(resolveUnderTest('/blank')).toMatchURL(BLANK);
    expect(resolveUnderTest('https://example.com/blank')).toMatchURL(BLANK);
    expect(resolveUnderTest('https://///example.com/blank')).toMatchURL(BLANK);
  });

  it(`should fail when remapping to "${NONE}"`, () => {
    expect(() => resolveUnderTest('none')).toThrow(TypeError);
    expect(() => resolveUnderTest('/none')).toThrow(TypeError);
  });
});

describe('Fallbacks with built-in module addresses', () => {
  const resolveUnderTest = makeResolveUnderTest(`{
    "imports": {
      "blank": [
        "${BLANK}",
        "./blank-fallback.mjs"
      ],
      "none": [
        "${NONE}",
        "./none-fallback.mjs"
      ]
    }
  }`);

  it(`should resolve to "${BLANK}"`, () => {
    expect(resolveUnderTest('blank')).toMatchURL(BLANK);
  });

  it(`should fall back past "${NONE}"`, () => {
    expect(resolveUnderTest('none')).toMatchURL('https://example.com/app/none-fallback.mjs');
  });
});
