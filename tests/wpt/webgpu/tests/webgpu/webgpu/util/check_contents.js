/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/ // MAINTENANCE_TODO: The "checkThingTrue" naming is confusing; these must be used with `expectOK`
// or the result is dropped on the floor. Rename these to things like `typedArrayIsOK`(??) to
// make it clearer.
// MAINTENANCE_TODO: Also, audit to make sure we aren't dropping any on the floor. Consider a
// no-ignored-return lint check if we can find one that we can use.
import { assert,
ErrorWithExtra,
iterRange,
range } from


'../../common/util/util.js';
import { Float16Array } from '../../external/petamoriken/float16/float16.js';

import { generatePrettyTable } from './pretty_diff_tables.js';

/** Generate an expected value at `index`, to test for equality with the actual value. */

/** Check whether the actual `value` at `index` is as expected. */

/**
 * Provides a pretty-printing implementation for a particular CheckElementsPredicate.
 * This is an array; each element provides info to print an additional row in the error message.
 */










/**
 * Check whether two `TypedArray`s have equal contents.
 * Returns `undefined` if the check passes, or an `Error` if not.
 */
export function checkElementsEqual(
actual,
expected)
{
  assert(actual.constructor === expected.constructor, 'TypedArray type mismatch');
  assert(
    actual.length === expected.length,
    `length mismatch: expected ${expected.length} got ${actual.length}`
  );

  let failedElementsFirstMaybe = undefined;
  /** Sparse array with `true` for elements that failed. */
  const failedElements = [];
  for (let i = 0; i < actual.length; ++i) {
    if (actual[i] !== expected[i]) {
      failedElementsFirstMaybe ??= i;
      failedElements[i] = true;
    }
  }

  if (failedElementsFirstMaybe === undefined) {
    return undefined;
  }

  const failedElementsFirst = failedElementsFirstMaybe;
  return failCheckElements({
    actual,
    failedElements,
    failedElementsFirst,
    predicatePrinter: [{ leftHeader: 'expected ==', getValueForCell: (index) => expected[index] }]
  });
}

/**
 * Check whether each value in a `TypedArray` is between the two corresponding "expected" values
 * (either `a(i) <= actual[i] <= b(i)` or `a(i) >= actual[i] => b(i)`).
 */
export function checkElementsBetween(
actual,
expected)
{
  const error = checkElementsPassPredicate(
    actual,
    (index, value) =>
    value >= Math.min(expected[0](index), expected[1](index)) &&
    value <= Math.max(expected[0](index), expected[1](index)),
    {
      predicatePrinter: [
      { leftHeader: 'between', getValueForCell: (index) => expected[0](index) },
      { leftHeader: 'and', getValueForCell: (index) => expected[1](index) }]

    }
  );
  // If there was an error, extend it with additional extras.
  return error ? new ErrorWithExtra(error, () => ({ expected })) : undefined;
}

/**
 * Check whether each value in a `TypedArray` is equal to one of the two corresponding "expected"
 * values (either `actual[i] === a[i]` or `actual[i] === b[i]`)
 */
export function checkElementsEqualEither(
actual,
expected)
{
  const error = checkElementsPassPredicate(
    actual,
    (index, value) => value === expected[0][index] || value === expected[1][index],
    {
      predicatePrinter: [
      { leftHeader: 'either', getValueForCell: (index) => expected[0][index] },
      { leftHeader: 'or', getValueForCell: (index) => expected[1][index] }]

    }
  );
  // If there was an error, extend it with additional extras.
  return error ? new ErrorWithExtra(error, () => ({ expected })) : undefined;
}

/**
 * Check whether a `TypedArray`'s contents equal the values produced by a generator function.
 * Returns `undefined` if the check passes, or an `Error` if not.
 *
 * ```text
 * Array had unexpected contents at indices 2 through 19.
 *  Starting at index 1:
 *    actual == 0x: 00 fe ff 00 01 02 03 04 05 06 07 08 09 0a 0b 0c 0d 0e 0f 00
 *    failed ->        xx xx    xx xx xx xx xx xx xx xx xx xx xx xx xx xx xx
 *  expected ==     00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
 * ```
 *
 * ```text
 * Array had unexpected contents at indices 2 through 29.
 *  Starting at index 1:
 *    actual ==  0.000 -2.000e+100 -1.000e+100 0.000 1.000e+100 2.000e+100 3.000e+100 4.000e+100 5.000e+100 6.000e+100 7.000e+100 ...
 *    failed ->                 xx          xx               xx         xx         xx         xx         xx         xx         xx ...
 *  expected ==  0.000       0.000       0.000 0.000      0.000      0.000      0.000      0.000      0.000      0.000      0.000 ...
 * ```
 */
export function checkElementsEqualGenerated(
actual,
generator)
{
  let failedElementsFirstMaybe = undefined;
  /** Sparse array with `true` for elements that failed. */
  const failedElements = [];
  for (let i = 0; i < actual.length; ++i) {
    if (actual[i] !== generator(i)) {
      failedElementsFirstMaybe ??= i;
      failedElements[i] = true;
    }
  }

  if (failedElementsFirstMaybe === undefined) {
    return undefined;
  }

  const failedElementsFirst = failedElementsFirstMaybe;
  const error = failCheckElements({
    actual,
    failedElements,
    failedElementsFirst,
    predicatePrinter: [{ leftHeader: 'expected ==', getValueForCell: (index) => generator(index) }]
  });
  // Add more extras to the error.
  return new ErrorWithExtra(error, () => ({ generator }));
}

/**
 * Check whether a `TypedArray`'s values pass the provided predicate function.
 * Returns `undefined` if the check passes, or an `Error` if not.
 */
export function checkElementsPassPredicate(
actual,
predicate,
{ predicatePrinter })
{
  let failedElementsFirstMaybe = undefined;
  /** Sparse array with `true` for elements that failed. */
  const failedElements = [];
  for (let i = 0; i < actual.length; ++i) {
    if (!predicate(i, actual[i])) {
      failedElementsFirstMaybe ??= i;
      failedElements[i] = true;
    }
  }

  if (failedElementsFirstMaybe === undefined) {
    return undefined;
  }

  const failedElementsFirst = failedElementsFirstMaybe;
  return failCheckElements({ actual, failedElements, failedElementsFirst, predicatePrinter });
}








/**
 * Implements the failure case of some checkElementsX helpers above. This allows those functions to
 * implement their checks directly without too many function indirections in between.
 *
 * Note: Separating this into its own function significantly speeds up the non-error case in
 * Chromium (though this may be V8-specific behavior).
 */
function failCheckElements({
  actual,
  failedElements,
  failedElementsFirst,
  predicatePrinter
}) {
  const size = actual.length;
  const ctor = actual.constructor;
  const printAsFloat = ctor === Float16Array || ctor === Float32Array || ctor === Float64Array;

  const failedElementsLast = failedElements.length - 1;

  // Include one extra non-failed element at the beginning and end (if they exist), for context.
  const printElementsStart = Math.max(0, failedElementsFirst - 1);
  const printElementsEnd = Math.min(size, failedElementsLast + 2);
  const printElementsCount = printElementsEnd - printElementsStart;

  const numericToString = (val) => {
    if (typeof val === 'number' && printAsFloat) {
      return val.toPrecision(4);
    }
    return intToPaddedHex(val, { byteLength: ctor.BYTES_PER_ELEMENT });
  };

  const numberPrefix = printAsFloat ? '' : '0x:';

  const printActual = actual.subarray(printElementsStart, printElementsEnd);
  const printExpected = [];
  if (predicatePrinter) {
    for (const { leftHeader, getValueForCell: cell } of predicatePrinter) {
      printExpected.push(
        function* () {
          yield* [leftHeader, ''];
          yield* iterRange(printElementsCount, (i) => cell(printElementsStart + i));
        }()
      );
    }
  }

  const printFailedValueMarkers = function* () {
    yield* ['failed ->', ''];
    yield* range(printElementsCount, (i) => failedElements[printElementsStart + i] ? 'xx' : '');
  }();

  const opts = {
    fillToWidth: 120,
    numericToString
  };
  const msg = `Array had unexpected contents at indices ${failedElementsFirst} through ${failedElementsLast}.
 Starting at index ${printElementsStart}:
${generatePrettyTable(opts, [
  ['actual ==', numberPrefix, ...printActual],
  printFailedValueMarkers,
  ...printExpected]
  )}`;
  return new ErrorWithExtra(msg, () => ({
    actual: actual.slice()
  }));
}

// Helper helpers

/** Convert an integral `number` into a hex string, padded to the specified `byteLength`. */
function intToPaddedHex(val, { byteLength }) {
  assert(Number.isInteger(val), 'number must be integer');
  const is_negative = typeof val === 'number' ? val < 0 : val < 0n;
  let str = (is_negative ? -val : val).toString(16);
  if (byteLength) str = str.padStart(byteLength * 2, '0');
  if (is_negative) str = '-' + str;
  return str;
}