/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { unreachable } from '../../util/util.js';let windowURL = undefined;
function getWindowURL() {
  if (windowURL === undefined) {
    windowURL = new URL(window.location.toString());
  }
  return windowURL;
}

/** Parse a runner option that is always boolean-typed. False if missing or '0'. */
export function optionEnabled(
opt,
searchParams = getWindowURL().searchParams)
{
  const val = searchParams.get(opt);
  return val !== null && val !== '0';
}

/** Parse a runner option that is string-typed. If the option is missing, returns `null`. */
export function optionString(
opt,
searchParams = getWindowURL().searchParams)
{
  return searchParams.get(opt);
}

/** Runtime modes for running tests in different types of workers. */

/** Parse a runner option for different worker modes (as in `?worker=shared`). Null if no worker. */
export function optionWorkerMode(
opt,
searchParams = getWindowURL().searchParams)
{
  const value = searchParams.get(opt);
  if (value === null || value === '0') {
    return null;
  } else if (value === 'service') {
    return 'service';
  } else if (value === 'shared') {
    return 'shared';
  } else if (value === '' || value === '1' || value === 'dedicated') {
    return 'dedicated';
  }
  unreachable('invalid worker= option value');
}

/**
 * The possible options for the tests.
 */










export const kDefaultCTSOptions = {
  worker: null,
  debug: true,
  compatibility: false,
  forceFallbackAdapter: false,
  unrollConstEvalLoops: false,
  powerPreference: null,
  logToWebSocket: false
};

/**
 * Extra per option info.
 */






/**
 * Type for info for every option. This definition means adding an option
 * will generate a compile time error if no extra info is provided.
 */


/**
 * Options to the CTS.
 */
export const kCTSOptionsInfo = {
  worker: {
    description: 'run in a worker',
    parser: optionWorkerMode,
    selectValueDescriptions: [
    { value: null, description: 'no worker' },
    { value: 'dedicated', description: 'dedicated worker' },
    { value: 'shared', description: 'shared worker' },
    { value: 'service', description: 'service worker' }]

  },
  debug: { description: 'show more info' },
  compatibility: { description: 'run in compatibility mode' },
  forceFallbackAdapter: { description: 'pass forceFallbackAdapter: true to requestAdapter' },
  unrollConstEvalLoops: { description: 'unroll const eval loops in WGSL' },
  powerPreference: {
    description: 'set default powerPreference for some tests',
    parser: optionString,
    selectValueDescriptions: [
    { value: null, description: 'default' },
    { value: 'low-power', description: 'low-power' },
    { value: 'high-performance', description: 'high-performance' }]

  },
  logToWebSocket: { description: 'send some logs to ws://localhost:59497/' }
};

/**
 * Converts camel case to snake case.
 * Examples:
 *    fooBar -> foo_bar
 *    parseHTMLFile -> parse_html_file
 */
export function camelCaseToSnakeCase(id) {
  return id.
  replace(/(.)([A-Z][a-z]+)/g, '$1_$2').
  replace(/([a-z0-9])([A-Z])/g, '$1_$2').
  toLowerCase();
}

/**
 * Creates a Options from search parameters.
 */
function getOptionsInfoFromSearchString(
optionsInfos,
searchString)
{
  const searchParams = new URLSearchParams(searchString);
  const optionValues = {};
  for (const [optionName, info] of Object.entries(optionsInfos)) {
    const parser = info.parser || optionEnabled;
    optionValues[optionName] = parser(camelCaseToSnakeCase(optionName), searchParams);
  }
  return optionValues;
}

/**
 * Given a test query string in the form of `suite:foo,bar,moo&opt1=val1&opt2=val2
 * returns the query and the options.
 */
export function parseSearchParamLikeWithOptions(
optionsInfos,
query)



{
  const searchString = query.includes('q=') || query.startsWith('?') ? query : `q=${query}`;
  const queries = new URLSearchParams(searchString).getAll('q');
  const options = getOptionsInfoFromSearchString(optionsInfos, searchString);
  return { queries, options };
}

/**
 * Given a test query string in the form of `suite:foo,bar,moo&opt1=val1&opt2=val2
 * returns the query and the common options.
 */
export function parseSearchParamLikeWithCTSOptions(query) {
  return parseSearchParamLikeWithOptions(kCTSOptionsInfo, query);
}