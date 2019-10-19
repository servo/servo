'use strict';
const { expectScopes } = require('./helpers/parsing.js');

describe('Relative URL scope keys', () => {
  it('should work with no prefix', () => {
    expectScopes(
      ['foo'],
      'https://base.example/path1/path2/path3',
      ['https://base.example/path1/path2/foo']
    );
  });

  it('should work with ./, ../, and / prefixes', () => {
    expectScopes(
      ['./foo', '../foo', '/foo'],
      'https://base.example/path1/path2/path3',
      [
        'https://base.example/path1/path2/foo',
        'https://base.example/path1/foo',
        'https://base.example/foo'
      ]
    );
  });

  it('should work with /s, ?s, and #s', () => {
    expectScopes(
      ['foo/bar?baz#qux'],
      'https://base.example/path1/path2/path3',
      ['https://base.example/path1/path2/foo/bar?baz#qux']
    );
  });

  it('should work with an empty string scope key', () => {
    expectScopes(
      [''],
      'https://base.example/path1/path2/path3',
      ['https://base.example/path1/path2/path3']
    );
  });

  it('should work with / suffixes', () => {
    expectScopes(
      ['foo/', './foo/', '../foo/', '/foo/', '/foo//'],
      'https://base.example/path1/path2/path3',
      [
        'https://base.example/path1/path2/foo/',
        'https://base.example/path1/path2/foo/',
        'https://base.example/path1/foo/',
        'https://base.example/foo/',
        'https://base.example/foo//'
      ]
    );
  });

  it('should deduplicate based on URL parsing rules', () => {
    expectScopes(
      ['foo/\\', 'foo//', 'foo\\\\'],
      'https://base.example/path1/path2/path3',
      ['https://base.example/path1/path2/foo//']
    );
  });
});

describe('Absolute URL scope keys', () => {
  it('should accept all absolute URL scope keys, with or without fetch schemes', () => {
    expectScopes(
      [
        'about:good',
        'blob:good',
        'data:good',
        'file:///good',
        'filesystem:http://example.com/good/',
        'http://good/',
        'https://good/',
        'ftp://good/',
        'import:bad',
        'mailto:bad',
        'javascript:bad',
        'wss:ba'
      ],
      'https://base.example/path1/path2/path3',
      [
        'about:good',
        'blob:good',
        'data:good',
        'file:///good',
        'filesystem:http://example.com/good/',
        'http://good/',
        'https://good/',
        'ftp://good/',
        'import:bad',
        'mailto:bad',
        'javascript:bad',
        'wss://ba/'
      ],
      []
    );
  });

  it('should parse absolute URL scope keys, ignoring unparseable ones', () => {
    expectScopes(
      [
        'https://ex ample.org/',
        'https://example.com:demo',
        'http://[www.example.com]/',
        'https:example.org',
        'https://///example.com///',
        'https://example.net',
        'https://ex%41mple.com/foo/',
        'https://example.com/%41'
      ],
      'https://base.example/path1/path2/path3',
      [
        'https://base.example/path1/path2/example.org', // tricky case! remember we have a base URL
        'https://example.com///',
        'https://example.net/',
        'https://example.com/foo/',
        'https://example.com/%41'
      ],
      [
        'Invalid scope "https://ex ample.org/" (parsed against base URL "https://base.example/path1/path2/path3").',
        'Invalid scope "https://example.com:demo" (parsed against base URL "https://base.example/path1/path2/path3").',
        'Invalid scope "http://[www.example.com]/" (parsed against base URL "https://base.example/path1/path2/path3").'
      ]
    );
  });

  it('should ignore relative URL scope keys when the base URL is a data: URL', () => {
    expectScopes(
      [
        './foo',
        '../foo',
        '/foo'
      ],
      'data:text/html,test',
      [],
      [
        'Invalid scope "./foo" (parsed against base URL "data:text/html,test").',
        'Invalid scope "../foo" (parsed against base URL "data:text/html,test").',
        'Invalid scope "/foo" (parsed against base URL "data:text/html,test").'
      ]
    );
  });
});
