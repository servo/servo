'use strict';
const { parseFromString } = require('../../lib/parser.js');

// Local modifications from upstream:
// Currently warnings and scopes are not checked in expectSpecifierMap().
exports.expectSpecifierMap = (input, baseURL, output, warnings = []) => {
  expect(parseFromString(`{ "imports": ${input} }`, baseURL))
    .toEqual({ imports: output, scopes: {} });
};

exports.expectScopes = (inputArray, baseURL, outputArray, warnings = []) => {
  const checkWarnings = testWarningHandler(warnings);

  const inputScopesAsStrings = inputArray.map(scopePrefix => `${JSON.stringify(scopePrefix)}: {}`);
  const inputString = `{ "scopes": { ${inputScopesAsStrings.join(', ')} } }`;

  const outputScopesObject = {};
  for (const outputScopePrefix of outputArray) {
    outputScopesObject[outputScopePrefix] = {};
  }

  expect(parseFromString(inputString, baseURL)).toEqual({ imports: {}, scopes: outputScopesObject });

  checkWarnings();
};

exports.expectBad = (input, baseURL, warnings = []) => {
  const checkWarnings = testWarningHandler(warnings);
  expect(() => parseFromString(input, baseURL)).toThrow(TypeError);
  checkWarnings();
};

exports.expectWarnings = (input, baseURL, output, warnings = []) => {
  const checkWarnings = testWarningHandler(warnings);
  expect(parseFromString(input, baseURL)).toEqual(output);

  checkWarnings();
};

function testWarningHandler(expectedWarnings) {
  const warnings = [];
  const { warn } = console;
  console.warn = warning => {
    warnings.push(warning);
  };
  return () => {
    console.warn = warn;
    expect(warnings).toEqual(expectedWarnings);
  };
}
