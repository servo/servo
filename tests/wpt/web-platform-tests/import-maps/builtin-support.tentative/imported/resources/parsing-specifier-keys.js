'use strict';
const { expectSpecifierMap } = require('./helpers/parsing.js');
const { BUILT_IN_MODULE_SCHEME } = require('../lib/utils.js');

const BLANK = `${BUILT_IN_MODULE_SCHEME}:blank`;

describe('Relative URL-like specifier keys', () => {
  it('should absolutize strings prefixed with ./, ../, or / into the corresponding URLs', () => {
    expectSpecifierMap(
      `{
        "./foo": "/dotslash",
        "../foo": "/dotdotslash",
        "/foo": "/slash"
      }`,
      'https://base.example/path1/path2/path3',
      {
        'https://base.example/path1/path2/foo': [expect.toMatchURL('https://base.example/dotslash')],
        'https://base.example/path1/foo': [expect.toMatchURL('https://base.example/dotdotslash')],
        'https://base.example/foo': [expect.toMatchURL('https://base.example/slash')]
      }
    );
  });

  it('should not absolutize strings prefixed with ./, ../, or / with a data: URL base', () => {
    expectSpecifierMap(
      `{
        "./foo": "https://example.com/dotslash",
        "../foo": "https://example.com/dotdotslash",
        "/foo": "https://example.com/slash"
      }`,
      'data:text/html,test',
      {
        './foo': [expect.toMatchURL('https://example.com/dotslash')],
        '../foo': [expect.toMatchURL('https://example.com/dotdotslash')],
        '/foo': [expect.toMatchURL('https://example.com/slash')]
      }
    );
  });

  it('should absolutize the literal strings ./, ../, or / with no suffix', () => {
    expectSpecifierMap(
      `{
        "./": "/dotslash/",
        "../": "/dotdotslash/",
        "/": "/slash/"
      }`,
      'https://base.example/path1/path2/path3',
      {
        'https://base.example/path1/path2/': [expect.toMatchURL('https://base.example/dotslash/')],
        'https://base.example/path1/': [expect.toMatchURL('https://base.example/dotdotslash/')],
        'https://base.example/': [expect.toMatchURL('https://base.example/slash/')]
      }
    );
  });

  it('should treat percent-encoded variants of ./, ../, or / as bare specifiers', () => {
    expectSpecifierMap(
      `{
        "%2E/": "/dotSlash1/",
        "%2E%2E/": "/dotDotSlash1/",
        ".%2F": "/dotSlash2",
        "..%2F": "/dotDotSlash2",
        "%2F": "/slash2",
        "%2E%2F": "/dotSlash3",
        "%2E%2E%2F": "/dotDotSlash3"
      }`,
      'https://base.example/path1/path2/path3',
      {
        '%2E/': [expect.toMatchURL('https://base.example/dotSlash1/')],
        '%2E%2E/': [expect.toMatchURL('https://base.example/dotDotSlash1/')],
        '.%2F': [expect.toMatchURL('https://base.example/dotSlash2')],
        '..%2F': [expect.toMatchURL('https://base.example/dotDotSlash2')],
        '%2F': [expect.toMatchURL('https://base.example/slash2')],
        '%2E%2F': [expect.toMatchURL('https://base.example/dotSlash3')],
        '%2E%2E%2F': [expect.toMatchURL('https://base.example/dotDotSlash3')]
      }
    );
  });
});

describe('Absolute URL specifier keys', () => {
  it('should only accept absolute URL specifier keys with fetch schemes, treating others as bare specifiers', () => {
    expectSpecifierMap(
      `{
        "about:good": "/about",
        "blob:good": "/blob",
        "data:good": "/data",
        "file:///good": "/file",
        "filesystem:good": "/filesystem",
        "http://good/": "/http/",
        "https://good/": "/https/",
        "ftp://good/": "/ftp/",
        "import:bad": "/import",
        "mailto:bad": "/mailto",
        "javascript:bad": "/javascript",
        "wss:bad": "/wss"
      }`,
      'https://base.example/path1/path2/path3',
      {
        'about:good': [expect.toMatchURL('https://base.example/about')],
        'blob:good': [expect.toMatchURL('https://base.example/blob')],
        'data:good': [expect.toMatchURL('https://base.example/data')],
        'file:///good': [expect.toMatchURL('https://base.example/file')],
        'filesystem:good': [expect.toMatchURL('https://base.example/filesystem')],
        'http://good/': [expect.toMatchURL('https://base.example/http/')],
        'https://good/': [expect.toMatchURL('https://base.example/https/')],
        'ftp://good/': [expect.toMatchURL('https://base.example/ftp/')],
        'import:bad': [expect.toMatchURL('https://base.example/import')],
        'mailto:bad': [expect.toMatchURL('https://base.example/mailto')],
        'javascript:bad': [expect.toMatchURL('https://base.example/javascript')],
        'wss:bad': [expect.toMatchURL('https://base.example/wss')]
      }
    );
  });

  it('should parse absolute URLs, treating unparseable ones as bare specifiers', () => {
    expectSpecifierMap(
      `{
        "https://ex ample.org/": "/unparseable1/",
        "https://example.com:demo": "/unparseable2",
        "http://[www.example.com]/": "/unparseable3/",
        "https:example.org": "/invalidButParseable1/",
        "https://///example.com///": "/invalidButParseable2/",
        "https://example.net": "/prettyNormal/",
        "https://ex%41mple.com/": "/percentDecoding/",
        "https://example.com/%41": "/noPercentDecoding"
      }`,
      'https://base.example/path1/path2/path3',
      {
        'https://ex ample.org/': [expect.toMatchURL('https://base.example/unparseable1/')],
        'https://example.com:demo': [expect.toMatchURL('https://base.example/unparseable2')],
        'http://[www.example.com]/': [expect.toMatchURL('https://base.example/unparseable3/')],
        'https://example.org/': [expect.toMatchURL('https://base.example/invalidButParseable1/')],
        'https://example.com///': [expect.toMatchURL('https://base.example/invalidButParseable2/')],
        'https://example.net/': [expect.toMatchURL('https://base.example/prettyNormal/')],
        'https://example.com/': [expect.toMatchURL('https://base.example/percentDecoding/')],
        'https://example.com/%41': [expect.toMatchURL('https://base.example/noPercentDecoding')]
      }
    );
  });

  it('should parse built-in module specifier keys, including with a "/"', () => {
    expectSpecifierMap(
      `{
        "${BLANK}": "/blank",
        "${BLANK}/": "/blank/",
        "${BLANK}/foo": "/blank/foo",
        "${BLANK}\\\\foo": "/blank/backslashfoo"
      }`,
      'https://base.example/path1/path2/path3',
      {
        [BLANK]: [expect.toMatchURL('https://base.example/blank')],
        [`${BLANK}/`]: [expect.toMatchURL('https://base.example/blank/')],
        [`${BLANK}/foo`]: [expect.toMatchURL('https://base.example/blank/foo')],
        [`${BLANK}\\foo`]: [expect.toMatchURL('https://base.example/blank/backslashfoo')]
      }
    );
  });
});
