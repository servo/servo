/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/

export function keysOf(obj) {
  return Object.keys(obj);
}

export function numericKeysOf(obj) {
  return Object.keys(obj).map(n => Number(n));
}

/**
 * Creates an info lookup object from a more nicely-formatted table. See below for examples.
 *
 * Note: Using `as const` on the arguments to this function is necessary to infer the correct type.
 */
export function makeTable(members, defaults, table) {
  const result = {};
  for (const [k, v] of Object.entries(table)) {
    const item = {};
    for (let i = 0; i < members.length; ++i) {
      item[members[i]] = v[i] ?? defaults[i];
    }
    result[k] = item;
  }

  return result;
}
