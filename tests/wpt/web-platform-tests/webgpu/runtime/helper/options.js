/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

const url = new URL(window.location.toString());
export function optionEnabled(opt) {
  const val = url.searchParams.get(opt);
  return val !== null && val !== '0';
}
//# sourceMappingURL=options.js.map