/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

import { stringifyPublicParams } from './params_utils.js';
import { unreachable } from './util/util.js';
export function encodeSelectively(s) {
  let ret = encodeURIComponent(s);
  ret = ret.replace(/%22/g, '"');
  ret = ret.replace(/%2C/g, ',');
  ret = ret.replace(/%2F/g, '/');
  ret = ret.replace(/%3A/g, ':');
  ret = ret.replace(/%3D/g, '=');
  ret = ret.replace(/%5B/g, '[');
  ret = ret.replace(/%5D/g, ']');
  ret = ret.replace(/%7B/g, '{');
  ret = ret.replace(/%7D/g, '}');
  return ret;
}
export function checkPublicParamType(v) {
  if (typeof v === 'number' || typeof v === 'string' || typeof v === 'boolean' || v === undefined) {
    return;
  }

  if (v instanceof Array) {
    for (const x of v) {
      if (typeof x !== 'number') {
        break;
      }
    }

    return;
  }

  unreachable('Invalid type for test case params ' + v);
}
export function makeQueryString(spec, testcase) {
  let s = spec.suite + ':';
  s += spec.path + ':';

  if (testcase !== undefined) {
    s += testcase.test + '=';
    s += stringifyPublicParams(testcase.params);
  }

  return encodeSelectively(s);
}
//# sourceMappingURL=url_query.js.map