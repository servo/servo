/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export function encodeSelectively(s) {
  let ret = encodeURIComponent(s);
  ret = ret.replace(/%20/g, '+'); // Encode space with + (equivalent but more readable)

  ret = ret.replace(/%22/g, '"');
  ret = ret.replace(/%2C/g, ',');
  ret = ret.replace(/%2F/g, '/');
  ret = ret.replace(/%3A/g, ':');
  ret = ret.replace(/%3D/g, '=');
  ret = ret.replace(/%7B/g, '{');
  ret = ret.replace(/%7D/g, '}');
  return ret;
}
export function makeQueryString(spec, testcase) {
  let s = spec.suite + ':';
  s += spec.path + ':';

  if (testcase !== undefined) {
    s += testcase.test + '=';

    if (testcase.params) {
      s += JSON.stringify(testcase.params);
    }
  }

  return encodeSelectively(s);
}
//# sourceMappingURL=url_query.js.map