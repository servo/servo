/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { assert } from '../../util/util.js';import {

  badParamValueChars,
  paramKeyIsPublic } from
'../params_utils.js';

import { parseParamValue } from './json_param_value.js';
import {

  TestQueryMultiFile,
  TestQueryMultiTest,
  TestQueryMultiCase,
  TestQuerySingleCase } from
'./query.js';
import { kBigSeparator, kWildcard, kPathSeparator, kParamSeparator } from './separators.js';
import { validQueryPart } from './validQueryPart.js';

/**
 * converts foo/bar/src/webgpu/this/that/file.spec.ts to webgpu:this,that,file,*
 */
function convertPathToQuery(path) {
  // removes .spec.ts and splits by directory separators.
  const parts = path.substring(0, path.length - 8).split(/\/|\\/g);
  // Gets parts only after the last `src`. Example: returns ['webgpu', 'foo', 'bar', 'test']
  // for ['Users', 'me', 'src', 'cts', 'src', 'webgpu', 'foo', 'bar', 'test']
  const partsAfterSrc = parts.slice(parts.lastIndexOf('src') + 1);
  const suite = partsAfterSrc.shift();
  return `${suite}:${partsAfterSrc.join(',')},*`;
}

/**
 * If a query looks like a path (ends in .spec.ts and has directory separators)
 * then convert try to convert it to a query.
 */
function convertPathLikeToQuery(queryOrPath) {
  return queryOrPath.endsWith('.spec.ts') && (
  queryOrPath.includes('/') || queryOrPath.includes('\\')) ?
  convertPathToQuery(queryOrPath) :
  queryOrPath;
}

/**
 * Convert long suite names (the part before the first colon) to the
 * shortest last word
 *    foo.bar.moo:test,subtest,foo -> moo:test,subtest,foo
 */
function shortenSuiteName(query) {
  const parts = query.split(':');
  // converts foo.bar.moo to moo
  const suite = parts.shift()?.replace(/.*\.(\w+)$/, '$1');
  return [suite, ...parts].join(':');
}

export function parseQuery(queryLike) {
  try {
    const query = shortenSuiteName(convertPathLikeToQuery(queryLike));
    return parseQueryImpl(query);
  } catch (ex) {
    if (ex instanceof Error) {
      ex.message += `\n  on: ${queryLike}`;
    }
    throw ex;
  }
}

function parseQueryImpl(s) {
  // Undo encodeURIComponentSelectively
  s = decodeURIComponent(s);

  // bigParts are: suite, file, test, params (note kBigSeparator could appear in params)
  let suite;
  let fileString;
  let testString;
  let paramsString;
  {
    const i1 = s.indexOf(kBigSeparator);
    assert(i1 !== -1, `query string must have at least one ${kBigSeparator}`);
    suite = s.substring(0, i1);
    const i2 = s.indexOf(kBigSeparator, i1 + 1);
    if (i2 === -1) {
      fileString = s.substring(i1 + 1);
    } else {
      fileString = s.substring(i1 + 1, i2);
      const i3 = s.indexOf(kBigSeparator, i2 + 1);
      if (i3 === -1) {
        testString = s.substring(i2 + 1);
      } else {
        testString = s.substring(i2 + 1, i3);
        paramsString = s.substring(i3 + 1);
      }
    }
  }

  const { parts: file, wildcard: filePathHasWildcard } = parseBigPart(fileString, kPathSeparator);

  if (testString === undefined) {
    // Query is file-level
    assert(
      filePathHasWildcard,
      `File-level query without wildcard ${kWildcard}. Did you want a file-level query \
(append ${kPathSeparator}${kWildcard}) or test-level query (append ${kBigSeparator}${kWildcard})?`
    );
    return new TestQueryMultiFile(suite, file);
  }
  assert(!filePathHasWildcard, `Wildcard ${kWildcard} must be at the end of the query string`);

  const { parts: test, wildcard: testPathHasWildcard } = parseBigPart(testString, kPathSeparator);

  if (paramsString === undefined) {
    // Query is test-level
    assert(
      testPathHasWildcard,
      `Test-level query without wildcard ${kWildcard}; did you want a test-level query \
(append ${kPathSeparator}${kWildcard}) or case-level query (append ${kBigSeparator}${kWildcard})?`
    );
    assert(file.length > 0, 'File part of test-level query was empty (::)');
    return new TestQueryMultiTest(suite, file, test);
  }

  // Query is case-level
  assert(!testPathHasWildcard, `Wildcard ${kWildcard} must be at the end of the query string`);

  const { parts: paramsParts, wildcard: paramsHasWildcard } = parseBigPart(
    paramsString,
    kParamSeparator
  );

  assert(test.length > 0, 'Test part of case-level query was empty (::)');

  const params = {};
  for (const paramPart of paramsParts) {
    const [k, v] = parseSingleParam(paramPart);
    assert(validQueryPart.test(k), `param key names must match ${validQueryPart}`);
    params[k] = v;
  }
  if (paramsHasWildcard) {
    return new TestQueryMultiCase(suite, file, test, params);
  } else {
    return new TestQuerySingleCase(suite, file, test, params);
  }
}

// webgpu:a,b,* or webgpu:a,b,c:*
const kExampleQueries = `\
webgpu${kBigSeparator}a${kPathSeparator}b${kPathSeparator}${kWildcard} or \
webgpu${kBigSeparator}a${kPathSeparator}b${kPathSeparator}c${kBigSeparator}${kWildcard}`;

function parseBigPart(
s,
separator)
{
  if (s === '') {
    return { parts: [], wildcard: false };
  }
  const parts = s.split(separator);

  let endsWithWildcard = false;
  for (const [i, part] of parts.entries()) {
    if (i === parts.length - 1) {
      endsWithWildcard = part === kWildcard;
    }
    assert(
      part.indexOf(kWildcard) === -1 || endsWithWildcard,
      `Wildcard ${kWildcard} must be complete last part of a path (e.g. ${kExampleQueries})`
    );
  }
  if (endsWithWildcard) {
    // Remove the last element of the array (which is just the wildcard).
    parts.length = parts.length - 1;
  }
  return { parts, wildcard: endsWithWildcard };
}

function parseSingleParam(paramSubstring) {
  assert(paramSubstring !== '', 'Param in a query must not be blank (is there a trailing comma?)');
  const i = paramSubstring.indexOf('=');
  assert(i !== -1, 'Param in a query must be of form key=value');
  const k = paramSubstring.substring(0, i);
  assert(paramKeyIsPublic(k), 'Param in a query must not be private (start with _)');
  const v = paramSubstring.substring(i + 1);
  return [k, parseSingleParamValue(v)];
}

function parseSingleParamValue(s) {
  assert(
    !badParamValueChars.test(s),
    `param value must not match ${badParamValueChars} - was ${s}`
  );
  return parseParamValue(s);
}