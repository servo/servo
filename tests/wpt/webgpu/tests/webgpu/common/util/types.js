/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/ /** Forces a type to resolve its type definitions, to make it readable/debuggable. */




/** Returns the type `true` iff X and Y are exactly equal */





export function assertTypeTrue() {}

/** `ReadonlyArray` of `ReadonlyArray`s. */

/** `ReadonlyArray` of `ReadonlyArray`s of `ReadonlyArray`s. */


/**
 * Deep version of the Readonly<> type, with support for tuples (up to length 7).
 * <https://gist.github.com/masterkidan/7322752f569b1bba53e0426266768623>
 */






























/**
 * Computes the intersection of a set of types, given the union of those types.
 *
 * From: https://stackoverflow.com/a/56375136
 */




/** "Type asserts" that `X` is a subtype of `Y`. */






/**
 * Zips a key tuple type and a value tuple type together into an object.
 *
 * @template Keys Keys of the resulting object.
 * @template Values Values of the resulting object. If a key corresponds to a `Values` member that
 *   is undefined or past the end, it defaults to the corresponding `Defaults` member.
 * @template Defaults Default values. If a key corresponds to a `Defaults` member that is past the
 *   end, the default falls back to `undefined`.
 */

















// K exhausted