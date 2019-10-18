'use strict';
const { parseFromString } = require('../lib/parser.js');
const { expectBad, expectWarnings, expectSpecifierMap } = require('./helpers/parsing.js');

const nonObjectStrings = ['null', 'true', '1', '"foo"', '[]'];

test('Invalid JSON', () => {
  expect(() => parseFromString('{ imports: {} }', 'https://base.example/')).toThrow(SyntaxError);
});

describe('Mismatching the top-level schema', () => {
  it('should throw for top-level non-objects', () => {
    for (const nonObject of nonObjectStrings) {
      expectBad(nonObject, 'https://base.example/');
    }
  });

  it('should throw if imports is a non-object', () => {
    for (const nonObject of nonObjectStrings) {
      expectBad(`{ "imports": ${nonObject} }`, 'https://base.example/');
    }
  });

  it('should throw if scopes is a non-object', () => {
    for (const nonObject of nonObjectStrings) {
      expectBad(`{ "scopes": ${nonObject} }`, 'https://base.example/');
    }
  });

  it('should ignore unspecified top-level entries', () => {
    expectWarnings(
      `{
        "imports": {},
        "new-feature": {},
        "scops": {}
      }`,
      'https://base.example/',
      { imports: {}, scopes: {} },
      [
        `Invalid top-level key "new-feature". Only "imports" and "scopes" can be present.`,
        `Invalid top-level key "scops". Only "imports" and "scopes" can be present.`
      ]
    );
  });
});

describe('Mismatching the specifier map schema', () => {
  const invalidAddressStrings = ['null', 'true', '1', '{}', '[]', '["https://example.com/"]'];

  it('should ignore entries where the address is not a string', () => {
    for (const invalid of invalidAddressStrings) {
      expectSpecifierMap(
        `{
          "foo": ${invalid},
          "bar": "https://example.com/"
        }`,
        'https://base.example/',
        {
          bar: expect.toMatchURL('https://example.com/')
        },
        [`Invalid address ${invalid} for the specifier key "foo". Addresses must be strings.`]
      );
    }
  });

  it('should ignore entries where the specifier key is an empty string', () => {
    expectSpecifierMap(
      `{
        "": "https://example.com/"
      }`,
      'https://base.example/',
      {},
      [`Invalid empty string specifier key.`]
    );
  });

  it('should throw if a scope\'s value is not an object', () => {
    for (const invalid of nonObjectStrings) {
      expectBad(`{ "scopes": { "https://scope.example/": ${invalid} } }`, 'https://base.example/');
    }
  });
});

describe('Normalization', () => {
  it('should normalize empty import maps to have imports and scopes keys', () => {
    expect(parseFromString(`{}`, 'https://base.example/'))
      .toEqual({ imports: {}, scopes: {} });
  });

  it('should normalize an import map without imports to have imports', () => {
    expect(parseFromString(`{ "scopes": {} }`, 'https://base.example/'))
      .toEqual({ imports: {}, scopes: {} });
  });

  it('should normalize an import map without scopes to have scopes', () => {
    expect(parseFromString(`{ "imports": {} }`, 'https://base.example/'))
      .toEqual({ imports: {}, scopes: {} });
  });
});
