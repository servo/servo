/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { compare } from '../../../util/compare.js';import {
  ArrayValue,
  MatrixValue,

  VectorValue,
  isScalarValue } from
'../../../util/conversion.js';
import { FPInterval } from '../../../util/floating_point.js';








/** @returns if this Expectation actually a Comparator */
export function isComparator(e) {
  return !(
  e instanceof FPInterval ||
  isScalarValue(e) ||
  e instanceof VectorValue ||
  e instanceof MatrixValue ||
  e instanceof ArrayValue ||
  e instanceof Array);

}

/** @returns the input if it is already a Comparator, otherwise wraps it in a 'value' comparator */
export function toComparator(input) {
  if (isComparator(input)) {
    return input;
  }

  return { compare: (got) => compare(got, input), kind: 'value' };
}