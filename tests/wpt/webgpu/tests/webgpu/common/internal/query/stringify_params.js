/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { assert } from '../../util/util.js';
import { badParamValueChars, paramKeyIsPublic } from '../params_utils.js';

import { stringifyParamValue, stringifyParamValueUniquely } from './json_param_value.js';
import { kParamKVSeparator, kParamSeparator, kWildcard } from './separators.js';

export function stringifyPublicParams(p, addWildcard = false) {
  const parts = Object.keys(p)
    .filter(k => paramKeyIsPublic(k))
    .map(k => stringifySingleParam(k, p[k]));

  if (addWildcard) parts.push(kWildcard);

  return parts.join(kParamSeparator);
}

/**
 * An _approximately_ unique string representing a CaseParams value.
 */
export function stringifyPublicParamsUniquely(p) {
  const keys = Object.keys(p).sort();
  return keys
    .filter(k => paramKeyIsPublic(k))
    .map(k => stringifySingleParamUniquely(k, p[k]))
    .join(kParamSeparator);
}

export function stringifySingleParam(k, v) {
  return `${k}${kParamKVSeparator}${stringifySingleParamValue(v)}`;
}

function stringifySingleParamUniquely(k, v) {
  return `${k}${kParamKVSeparator}${stringifyParamValueUniquely(v)}`;
}

function stringifySingleParamValue(v) {
  const s = stringifyParamValue(v);
  assert(
    !badParamValueChars.test(s),
    `JSON.stringified param value must not match ${badParamValueChars} - was ${s}`
  );

  return s;
}
