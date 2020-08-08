/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { badParamValueChars, paramKeyIsPublic } from '../params_utils.js';
import { assert } from '../util/util.js';

import { stringifyParamValue } from './json_param_value.js';
import { kParamKVSeparator } from './separators.js';

export function stringifyPublicParams(p) {
  return Object.keys(p)
    .filter(k => paramKeyIsPublic(k))
    .map(k => stringifySingleParam(k, p[k]));
}

export function stringifySingleParam(k, v) {
  return `${k}${kParamKVSeparator}${stringifySingleParamValue(v)}`;
}

function stringifySingleParamValue(v) {
  const s = stringifyParamValue(v);
  assert(
    !badParamValueChars.test(s),
    `JSON.stringified param value must not match ${badParamValueChars} - was ${s}`
  );

  return s;
}
