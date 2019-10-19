'use strict';
const { expectSpecifierMap } = require('./helpers/parsing.js');
const { BUILT_IN_MODULE_SCHEME } = require('../lib/utils.js');

describe('Relative URL-like addresses', () => {
  it('should accept strings prefixed with ./, ../, or /', () => {
    expectSpecifierMap(
      `{
        "dotSlash": "./foo",
        "dotDotSlash": "../foo",
        "slash": "/foo"
      }`,
      'https://base.example/path1/path2/path3',
      {
        dotSlash: [expect.toMatchURL('https://base.example/path1/path2/foo')],
        dotDotSlash: [expect.toMatchURL('https://base.example/path1/foo')],
        slash: [expect.toMatchURL('https://base.example/foo')]
      }
    );
  });

  it('should not accept strings prefixed with ./, ../, or / for data: base URLs', () => {
    expectSpecifierMap(
      `{
        "dotSlash": "./foo",
        "dotDotSlash": "../foo",
        "slash": "/foo"
      }`,
      'data:text/html,test',
      {
        dotSlash: [],
        dotDotSlash: [],
        slash: []
      },
      [
        `Invalid address "./foo" for the specifier key "dotSlash".`,
        `Invalid address "../foo" for the specifier key "dotDotSlash".`,
        `Invalid address "/foo" for the specifier key "slash".`
      ]
    );
  });

  it('should accept the literal strings ./, ../, or / with no suffix', () => {
    expectSpecifierMap(
      `{
        "dotSlash": "./",
        "dotDotSlash": "../",
        "slash": "/"
      }`,
      'https://base.example/path1/path2/path3',
      {
        dotSlash: [expect.toMatchURL('https://base.example/path1/path2/')],
        dotDotSlash: [expect.toMatchURL('https://base.example/path1/')],
        slash: [expect.toMatchURL('https://base.example/')]
      }
    );
  });

  it('should ignore percent-encoded variants of ./, ../, or /', () => {
    expectSpecifierMap(
      `{
        "dotSlash1": "%2E/",
        "dotDotSlash1": "%2E%2E/",
        "dotSlash2": ".%2F",
        "dotDotSlash2": "..%2F",
        "slash2": "%2F",
        "dotSlash3": "%2E%2F",
        "dotDotSlash3": "%2E%2E%2F"
      }`,
      'https://base.example/path1/path2/path3',
      {
        dotSlash1: [],
        dotDotSlash1: [],
        dotSlash2: [],
        dotDotSlash2: [],
        slash2: [],
        dotSlash3: [],
        dotDotSlash3: []
      },
      [
        `Invalid address "%2E/" for the specifier key "dotSlash1".`,
        `Invalid address "%2E%2E/" for the specifier key "dotDotSlash1".`,
        `Invalid address ".%2F" for the specifier key "dotSlash2".`,
        `Invalid address "..%2F" for the specifier key "dotDotSlash2".`,
        `Invalid address "%2F" for the specifier key "slash2".`,
        `Invalid address "%2E%2F" for the specifier key "dotSlash3".`,
        `Invalid address "%2E%2E%2F" for the specifier key "dotDotSlash3".`
      ]
    );
  });
});

describe('Built-in module addresses', () => {
  it('should accept URLs using the built-in module scheme', () => {
    expectSpecifierMap(
      `{
        "foo": "${BUILT_IN_MODULE_SCHEME}:foo"
      }`,
      'https://base.example/path1/path2/path3',
      {
        foo: [expect.toMatchURL(`${BUILT_IN_MODULE_SCHEME}:foo`)]
      }
    );
  });

  it('should ignore percent-encoded variants of the built-in module scheme', () => {
    expectSpecifierMap(
      `{
        "foo": "${encodeURIComponent(BUILT_IN_MODULE_SCHEME + ':')}foo"
      }`,
      'https://base.example/path1/path2/path3',
      {
        foo: []
      },
      [`Invalid address "${encodeURIComponent(BUILT_IN_MODULE_SCHEME + ':')}foo" for the specifier key "foo".`]
    );
  });

  it('should allow built-in module URLs that contain "/" or "\\"', () => {
    expectSpecifierMap(
      `{
        "slashEnd": "${BUILT_IN_MODULE_SCHEME}:foo/",
        "slashMiddle": "${BUILT_IN_MODULE_SCHEME}:foo/bar",
        "backslash": "${BUILT_IN_MODULE_SCHEME}:foo\\\\baz"
      }`,
      'https://base.example/path1/path2/path3',
      {
        slashEnd: [expect.toMatchURL(`${BUILT_IN_MODULE_SCHEME}:foo/`)],
        slashMiddle: [expect.toMatchURL(`${BUILT_IN_MODULE_SCHEME}:foo/bar`)],
        backslash: [expect.toMatchURL(`${BUILT_IN_MODULE_SCHEME}:foo\\baz`)]
      }
    );
  });
});

describe('Absolute URL addresses', () => {
  it('should only accept absolute URL addresses with fetch schemes', () => {
    expectSpecifierMap(
      `{
        "about": "about:good",
        "blob": "blob:good",
        "data": "data:good",
        "file": "file:///good",
        "filesystem": "filesystem:good",
        "http": "http://good/",
        "https": "https://good/",
        "ftp": "ftp://good/",
        "import": "import:bad",
        "mailto": "mailto:bad",
        "javascript": "javascript:bad",
        "wss": "wss:bad"
      }`,
      'https://base.example/path1/path2/path3',
      {
        about: [expect.toMatchURL('about:good')],
        blob: [expect.toMatchURL('blob:good')],
        data: [expect.toMatchURL('data:good')],
        file: [expect.toMatchURL('file:///good')],
        filesystem: [expect.toMatchURL('filesystem:good')],
        http: [expect.toMatchURL('http://good/')],
        https: [expect.toMatchURL('https://good/')],
        ftp: [expect.toMatchURL('ftp://good/')],
        import: [],
        mailto: [],
        javascript: [],
        wss: []
      },
      [
        `Invalid address "import:bad" for the specifier key "import".`,
        `Invalid address "mailto:bad" for the specifier key "mailto".`,
        `Invalid address "javascript:bad" for the specifier key "javascript".`,
        `Invalid address "wss:bad" for the specifier key "wss".`
      ]
    );
  });

  it('should only accept absolute URL addresses with fetch schemes inside arrays', () => {
    expectSpecifierMap(
      `{
        "about": ["about:good"],
        "blob": ["blob:good"],
        "data": ["data:good"],
        "file": ["file:///good"],
        "filesystem": ["filesystem:good"],
        "http": ["http://good/"],
        "https": ["https://good/"],
        "ftp": ["ftp://good/"],
        "import": ["import:bad"],
        "mailto": ["mailto:bad"],
        "javascript": ["javascript:bad"],
        "wss": ["wss:bad"]
      }`,
      'https://base.example/path1/path2/path3',
      {
        about: [expect.toMatchURL('about:good')],
        blob: [expect.toMatchURL('blob:good')],
        data: [expect.toMatchURL('data:good')],
        file: [expect.toMatchURL('file:///good')],
        filesystem: [expect.toMatchURL('filesystem:good')],
        http: [expect.toMatchURL('http://good/')],
        https: [expect.toMatchURL('https://good/')],
        ftp: [expect.toMatchURL('ftp://good/')],
        import: [],
        mailto: [],
        javascript: [],
        wss: []
      },
      [
        `Invalid address "import:bad" for the specifier key "import".`,
        `Invalid address "mailto:bad" for the specifier key "mailto".`,
        `Invalid address "javascript:bad" for the specifier key "javascript".`,
        `Invalid address "wss:bad" for the specifier key "wss".`
      ]
    );
  });

  it('should parse absolute URLs, ignoring unparseable ones', () => {
    expectSpecifierMap(
      `{
        "unparseable1": "https://ex ample.org/",
        "unparseable2": "https://example.com:demo",
        "unparseable3": "http://[www.example.com]/",
        "invalidButParseable1": "https:example.org",
        "invalidButParseable2": "https://///example.com///",
        "prettyNormal": "https://example.net",
        "percentDecoding": "https://ex%41mple.com/",
        "noPercentDecoding": "https://example.com/%41"
      }`,
      'https://base.example/path1/path2/path3',
      {
        unparseable1: [],
        unparseable2: [],
        unparseable3: [],
        invalidButParseable1: [expect.toMatchURL('https://example.org/')],
        invalidButParseable2: [expect.toMatchURL('https://example.com///')],
        prettyNormal: [expect.toMatchURL('https://example.net/')],
        percentDecoding: [expect.toMatchURL('https://example.com/')],
        noPercentDecoding: [expect.toMatchURL('https://example.com/%41')]
      },
      [
        `Invalid address "https://ex ample.org/" for the specifier key "unparseable1".`,
        `Invalid address "https://example.com:demo" for the specifier key "unparseable2".`,
        `Invalid address "http://[www.example.com]/" for the specifier key "unparseable3".`
      ]
    );
  });

  it('should parse absolute URLs, ignoring unparseable ones inside arrays', () => {
    expectSpecifierMap(
      `{
        "unparseable1": ["https://ex ample.org/"],
        "unparseable2": ["https://example.com:demo"],
        "unparseable3": ["http://[www.example.com]/"],
        "invalidButParseable1": ["https:example.org"],
        "invalidButParseable2": ["https://///example.com///"],
        "prettyNormal": ["https://example.net"],
        "percentDecoding": ["https://ex%41mple.com/"],
        "noPercentDecoding": ["https://example.com/%41"]
      }`,
      'https://base.example/path1/path2/path3',
      {
        unparseable1: [],
        unparseable2: [],
        unparseable3: [],
        invalidButParseable1: [expect.toMatchURL('https://example.org/')],
        invalidButParseable2: [expect.toMatchURL('https://example.com///')],
        prettyNormal: [expect.toMatchURL('https://example.net/')],
        percentDecoding: [expect.toMatchURL('https://example.com/')],
        noPercentDecoding: [expect.toMatchURL('https://example.com/%41')]
      },
      [
        `Invalid address "https://ex ample.org/" for the specifier key "unparseable1".`,
        `Invalid address "https://example.com:demo" for the specifier key "unparseable2".`,
        `Invalid address "http://[www.example.com]/" for the specifier key "unparseable3".`
      ]
    );
  });
});

describe('Failing addresses: mismatched trailing slashes', () => {
  it('should warn for the simple case', () => {
    expectSpecifierMap(
      `{
        "trailer/": "/notrailer",
        "${BUILT_IN_MODULE_SCHEME}:trailer/": "/bim-notrailer"
      }`,
      'https://base.example/path1/path2/path3',
      {
        'trailer/': [],
        [`${BUILT_IN_MODULE_SCHEME}:trailer/`]: []
      },
      [
        `Invalid address "https://base.example/notrailer" for package specifier key "trailer/". Package addresses must end with "/".`,
        `Invalid address "https://base.example/bim-notrailer" for package specifier key "${BUILT_IN_MODULE_SCHEME}:trailer/". Package addresses must end with "/".`
      ]
    );
  });

  it('should warn for a mismatch alone in an array', () => {
    expectSpecifierMap(
      `{
        "trailer/": ["/notrailer"],
        "${BUILT_IN_MODULE_SCHEME}:trailer/": ["/bim-notrailer"]
      }`,
      'https://base.example/path1/path2/path3',
      {
        'trailer/': [],
        [`${BUILT_IN_MODULE_SCHEME}:trailer/`]: []
      },
      [
        `Invalid address "https://base.example/notrailer" for package specifier key "trailer/". Package addresses must end with "/".`,
        `Invalid address "https://base.example/bim-notrailer" for package specifier key "${BUILT_IN_MODULE_SCHEME}:trailer/". Package addresses must end with "/".`
      ]
    );
  });

  it('should warn for a mismatch alongside non-mismatches in an array', () => {
    expectSpecifierMap(
      `{
        "trailer/": ["/atrailer/", "/notrailer"],
        "${BUILT_IN_MODULE_SCHEME}:trailer/": ["/bim-atrailer/", "/bim-notrailer"]
      }`,
      'https://base.example/path1/path2/path3',
      {
        'trailer/': [expect.toMatchURL('https://base.example/atrailer/')],
        [`${BUILT_IN_MODULE_SCHEME}:trailer/`]: [expect.toMatchURL('https://base.example/bim-atrailer/')]
      },
      [
        `Invalid address "https://base.example/notrailer" for package specifier key "trailer/". Package addresses must end with "/".`,
        `Invalid address "https://base.example/bim-notrailer" for package specifier key "${BUILT_IN_MODULE_SCHEME}:trailer/". Package addresses must end with "/".`
      ]
    );
  });
});

describe('Other invalid addresses', () => {
  it('should ignore unprefixed strings that are not absolute URLs', () => {
    for (const bad of ['bar', '\\bar', '~bar', '#bar', '?bar']) {
      expectSpecifierMap(
        `{
          "foo": ${JSON.stringify(bad)}
        }`,
        'https://base.example/path1/path2/path3',
        {
          foo: []
        },
        [`Invalid address "${bad}" for the specifier key "foo".`]
      );
    }
  });
});
