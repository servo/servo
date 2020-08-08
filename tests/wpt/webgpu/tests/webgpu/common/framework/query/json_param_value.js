/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { assert } from '../util/util.js';
// JSON can't represent `undefined` and by default stores it as `null`.
// Instead, store `undefined` as this magic string value in JSON.
const jsUndefinedMagicValue = '✗undefined';

export function stringifyParamValue(value) {
  return JSON.stringify(value, (k, v) => {
    assert(v !== jsUndefinedMagicValue);

    return v === undefined ? jsUndefinedMagicValue : v;
  });
}

export function parseParamValue(s) {
  return JSON.parse(s, (k, v) => (v === jsUndefinedMagicValue ? undefined : v));
}
