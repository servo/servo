'use strict';
const { URL } = require('url');
const { parseFromString } = require('../lib/parser.js');
const { resolve } = require('../lib/resolver.js');

const mapBaseURL = new URL('https://example.com/app/index.html');

function makeResolveUnderTest(mapString) {
  const map = parseFromString(mapString, mapBaseURL);
  return (specifier, baseURL) => resolve(specifier, map, baseURL);
}

describe('Mapped using scope instead of "imports"', () => {
  const jsNonDirURL = new URL('https://example.com/js');
  const jsPrefixedURL = new URL('https://example.com/jsiscool');
  const inJSDirURL = new URL('https://example.com/js/app.mjs');
  const topLevelURL = new URL('https://example.com/app.mjs');

  it('should fail when the mapping is to an empty array', () => {
    const resolveUnderTest = makeResolveUnderTest(`{
      "scopes": {
        "/js/": {
          "moment": null,
          "lodash": []
        }
      }
    }`);

    expect(() => resolveUnderTest('moment', inJSDirURL)).toThrow(TypeError);
    expect(() => resolveUnderTest('lodash', inJSDirURL)).toThrow(TypeError);
  });

  describe('Exact vs. prefix based matching', () => {
    it('should match correctly when both are in the map', () => {
      const resolveUnderTest = makeResolveUnderTest(`{
        "scopes": {
          "/js": {
            "moment": "/only-triggered-by-exact/moment",
            "moment/": "/only-triggered-by-exact/moment/"
          },
          "/js/": {
            "moment": "/triggered-by-any-subpath/moment",
            "moment/": "/triggered-by-any-subpath/moment/"
          }
        }
      }`);

      expect(resolveUnderTest('moment', jsNonDirURL)).toMatchURL('https://example.com/only-triggered-by-exact/moment');
      expect(resolveUnderTest('moment/foo', jsNonDirURL)).toMatchURL('https://example.com/only-triggered-by-exact/moment/foo');

      expect(resolveUnderTest('moment', inJSDirURL)).toMatchURL('https://example.com/triggered-by-any-subpath/moment');
      expect(resolveUnderTest('moment/foo', inJSDirURL)).toMatchURL('https://example.com/triggered-by-any-subpath/moment/foo');

      expect(() => resolveUnderTest('moment', jsPrefixedURL)).toThrow(TypeError);
      expect(() => resolveUnderTest('moment/foo', jsPrefixedURL)).toThrow(TypeError);
    });

    it('should match correctly when only an exact match is in the map', () => {
      const resolveUnderTest = makeResolveUnderTest(`{
        "scopes": {
          "/js": {
            "moment": "/only-triggered-by-exact/moment",
            "moment/": "/only-triggered-by-exact/moment/"
          }
        }
      }`);

      expect(resolveUnderTest('moment', jsNonDirURL)).toMatchURL('https://example.com/only-triggered-by-exact/moment');
      expect(resolveUnderTest('moment/foo', jsNonDirURL)).toMatchURL('https://example.com/only-triggered-by-exact/moment/foo');

      expect(() => resolveUnderTest('moment', inJSDirURL)).toThrow(TypeError);
      expect(() => resolveUnderTest('moment/foo', inJSDirURL)).toThrow(TypeError);

      expect(() => resolveUnderTest('moment', jsPrefixedURL)).toThrow(TypeError);
      expect(() => resolveUnderTest('moment/foo', jsPrefixedURL)).toThrow(TypeError);
    });

    it('should match correctly when only a prefix match is in the map', () => {
      const resolveUnderTest = makeResolveUnderTest(`{
        "scopes": {
          "/js/": {
            "moment": "/triggered-by-any-subpath/moment",
            "moment/": "/triggered-by-any-subpath/moment/"
          }
        }
      }`);

      expect(() => resolveUnderTest('moment', jsNonDirURL)).toThrow(TypeError);
      expect(() => resolveUnderTest('moment/foo', jsNonDirURL)).toThrow(TypeError);

      expect(resolveUnderTest('moment', inJSDirURL)).toMatchURL('https://example.com/triggered-by-any-subpath/moment');
      expect(resolveUnderTest('moment/foo', inJSDirURL)).toMatchURL('https://example.com/triggered-by-any-subpath/moment/foo');

      expect(() => resolveUnderTest('moment', jsPrefixedURL)).toThrow(TypeError);
      expect(() => resolveUnderTest('moment/foo', jsPrefixedURL)).toThrow(TypeError);
    });
  });

  describe('Package-like scenarios', () => {
    const resolveUnderTest = makeResolveUnderTest(`{
      "imports": {
        "moment": "/node_modules/moment/src/moment.js",
        "moment/": "/node_modules/moment/src/",
        "lodash-dot": "./node_modules/lodash-es/lodash.js",
        "lodash-dot/": "./node_modules/lodash-es/",
        "lodash-dotdot": "../node_modules/lodash-es/lodash.js",
        "lodash-dotdot/": "../node_modules/lodash-es/"
      },
      "scopes": {
        "/": {
          "moment": "/node_modules_3/moment/src/moment.js",
          "vue": "/node_modules_3/vue/dist/vue.runtime.esm.js"
        },
        "/js/": {
          "lodash-dot": "./node_modules_2/lodash-es/lodash.js",
          "lodash-dot/": "./node_modules_2/lodash-es/",
          "lodash-dotdot": "../node_modules_2/lodash-es/lodash.js",
          "lodash-dotdot/": "../node_modules_2/lodash-es/"
        }
      }
    }`);

    it('should resolve scoped', () => {
      expect(resolveUnderTest('lodash-dot', inJSDirURL)).toMatchURL('https://example.com/app/node_modules_2/lodash-es/lodash.js');
      expect(resolveUnderTest('lodash-dotdot', inJSDirURL)).toMatchURL('https://example.com/node_modules_2/lodash-es/lodash.js');
      expect(resolveUnderTest('lodash-dot/foo', inJSDirURL)).toMatchURL('https://example.com/app/node_modules_2/lodash-es/foo');
      expect(resolveUnderTest('lodash-dotdot/foo', inJSDirURL)).toMatchURL('https://example.com/node_modules_2/lodash-es/foo');
    });

    it('should apply best scope match', () => {
      expect(resolveUnderTest('moment', topLevelURL)).toMatchURL('https://example.com/node_modules_3/moment/src/moment.js');
      expect(resolveUnderTest('moment', inJSDirURL)).toMatchURL('https://example.com/node_modules_3/moment/src/moment.js');
      expect(resolveUnderTest('vue', inJSDirURL)).toMatchURL('https://example.com/node_modules_3/vue/dist/vue.runtime.esm.js');
    });

    it('should fallback to "imports"', () => {
      expect(resolveUnderTest('moment/foo', topLevelURL)).toMatchURL('https://example.com/node_modules/moment/src/foo');
      expect(resolveUnderTest('moment/foo', inJSDirURL)).toMatchURL('https://example.com/node_modules/moment/src/foo');
      expect(resolveUnderTest('lodash-dot', topLevelURL)).toMatchURL('https://example.com/app/node_modules/lodash-es/lodash.js');
      expect(resolveUnderTest('lodash-dotdot', topLevelURL)).toMatchURL('https://example.com/node_modules/lodash-es/lodash.js');
      expect(resolveUnderTest('lodash-dot/foo', topLevelURL)).toMatchURL('https://example.com/app/node_modules/lodash-es/foo');
      expect(resolveUnderTest('lodash-dotdot/foo', topLevelURL)).toMatchURL('https://example.com/node_modules/lodash-es/foo');
    });

    it('should still fail for package-like specifiers that are not declared', () => {
      expect(() => resolveUnderTest('underscore/', inJSDirURL)).toThrow(TypeError);
      expect(() => resolveUnderTest('underscore/foo', inJSDirURL)).toThrow(TypeError);
    });
  });

  describe('The scope inheritance example from the README', () => {
    const resolveUnderTest = makeResolveUnderTest(`{
      "imports": {
        "a": "/a-1.mjs",
        "b": "/b-1.mjs",
        "c": "/c-1.mjs"
      },
      "scopes": {
        "/scope2/": {
          "a": "/a-2.mjs"
        },
        "/scope2/scope3/": {
          "b": "/b-3.mjs"
        }
      }
    }`);

    const scope1URL = new URL('https://example.com/scope1/foo.mjs');
    const scope2URL = new URL('https://example.com/scope2/foo.mjs');
    const scope3URL = new URL('https://example.com/scope2/scope3/foo.mjs');

    it('should fall back to "imports" when none match', () => {
      expect(resolveUnderTest('a', scope1URL)).toMatchURL('https://example.com/a-1.mjs');
      expect(resolveUnderTest('b', scope1URL)).toMatchURL('https://example.com/b-1.mjs');
      expect(resolveUnderTest('c', scope1URL)).toMatchURL('https://example.com/c-1.mjs');
    });

    it('should use a direct scope override', () => {
      expect(resolveUnderTest('a', scope2URL)).toMatchURL('https://example.com/a-2.mjs');
      expect(resolveUnderTest('b', scope2URL)).toMatchURL('https://example.com/b-1.mjs');
      expect(resolveUnderTest('c', scope2URL)).toMatchURL('https://example.com/c-1.mjs');
    });

    it('should use an indirect scope override', () => {
      expect(resolveUnderTest('a', scope3URL)).toMatchURL('https://example.com/a-2.mjs');
      expect(resolveUnderTest('b', scope3URL)).toMatchURL('https://example.com/b-3.mjs');
      expect(resolveUnderTest('c', scope3URL)).toMatchURL('https://example.com/c-1.mjs');
    });
  });

  describe('Relative URL scope keys', () => {
    const resolveUnderTest = makeResolveUnderTest(`{
      "imports": {
        "a": "/a-1.mjs",
        "b": "/b-1.mjs",
        "c": "/c-1.mjs"
      },
      "scopes": {
        "": {
          "a": "/a-empty-string.mjs"
        },
        "./": {
          "b": "/b-dot-slash.mjs"
        },
        "../": {
          "c": "/c-dot-dot-slash.mjs"
        }
      }
    }`);
    const inSameDirAsMap = new URL('./foo.mjs', mapBaseURL);
    const inDirAboveMap = new URL('../foo.mjs', mapBaseURL);

    it('should resolve an empty string scope using the import map URL', () => {
      expect(resolveUnderTest('a', mapBaseURL)).toMatchURL('https://example.com/a-empty-string.mjs');
      expect(resolveUnderTest('a', inSameDirAsMap)).toMatchURL('https://example.com/a-1.mjs');
    });

    it('should resolve a ./ scope using the import map URL\'s directory', () => {
      expect(resolveUnderTest('b', mapBaseURL)).toMatchURL('https://example.com/b-dot-slash.mjs');
      expect(resolveUnderTest('b', inSameDirAsMap)).toMatchURL('https://example.com/b-dot-slash.mjs');
    });

    it('should resolve a ../ scope using the import map URL\'s directory', () => {
      expect(resolveUnderTest('c', mapBaseURL)).toMatchURL('https://example.com/c-dot-dot-slash.mjs');
      expect(resolveUnderTest('c', inSameDirAsMap)).toMatchURL('https://example.com/c-dot-dot-slash.mjs');
      expect(resolveUnderTest('c', inDirAboveMap)).toMatchURL('https://example.com/c-dot-dot-slash.mjs');
    });
  });
});

