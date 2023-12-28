/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/ /**
 * Encodes a stringified TestQuery so that it can be placed in a `?q=` parameter in a URL.
 *
 * `encodeURIComponent` encodes in accordance with `application/x-www-form-urlencoded`,
 * but URLs don't actually have to be as strict as HTML form encoding
 * (we interpret this purely from JavaScript).
 * So we encode the component, then selectively convert some %-encoded escape codes
 * back to their original form for readability/copyability.
 */export function encodeURIComponentSelectively(s) {let ret = encodeURIComponent(s);
  ret = ret.replace(/%22/g, '"'); // for JSON strings
  ret = ret.replace(/%2C/g, ','); // for path separator, and JSON arrays
  ret = ret.replace(/%3A/g, ':'); // for big separator
  ret = ret.replace(/%3B/g, ';'); // for param separator
  ret = ret.replace(/%3D/g, '='); // for params (k=v)
  ret = ret.replace(/%5B/g, '['); // for JSON arrays
  ret = ret.replace(/%5D/g, ']'); // for JSON arrays
  ret = ret.replace(/%7B/g, '{'); // for JSON objects
  ret = ret.replace(/%7D/g, '}'); // for JSON objects
  ret = ret.replace(/%E2%9C%97/g, '✗'); // for jsUndefinedMagicValue
  return ret;
}