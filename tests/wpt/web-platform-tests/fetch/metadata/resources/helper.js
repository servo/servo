function wrap_by_tag(tag, text) {
  return tag ? `${tag}: ${text}`: text;
}

function validate_expectations(key, expected, tag) {
  return fetch("/fetch/metadata/resources/record-header.py?retrieve=true&file=" + key)
    .then(response => response.text())
    .then(text => {
      assert_not_equals(text, "No header has been recorded");
      let value = JSON.parse(text);
      test(t => assert_equals(value.dest, expected.dest), `${tag}: sec-fetch-dest`);
      test(t => assert_equals(value.mode, expected.mode), `${tag}: sec-fetch-mode`);
      test(t => assert_equals(value.site, expected.site), `${tag}: sec-fetch-site`);
      test(t => assert_equals(value.user, expected.user), `${tag}: sec-fetch-user`);
    });
};

/**
 * @param {object} value
 * @param {object} expected
 * @param {string} tag
 **/
function assert_header_equals(value, expected, tag) {
  if (typeof(value) === "string"){
    assert_not_equals(value, "No header has been recorded");
    value = JSON.parse(value);
  }

  assert_equals(value.mode, expected.mode, wrap_by_tag(tag, "mode"));
  assert_equals(value.site, expected.site, wrap_by_tag(tag, "site"));
  if (expected.hasOwnProperty("user"))
    assert_equals(value.user, expected.user, wrap_by_tag(tag, "user"));
}

/**
 * @param {string} header
 * @param {object} value
 * @param {string} expected
 * @param {string} tag
 **/
function assert_header(header, value, expected, tag) {
  if (typeof(value) === "string"){
    assert_not_equals(value, "No header has been recorded");
    value = JSON.parse(value);
  }

  assert_equals(value[header], expected, wrap_by_tag(tag, header));
}

/**
 *
 * @param {object} value
 * @param {string} expected
 * @param {string} tag
 **/
function assert_header_dest_equals(value, expected, tag) {
  assert_header("dest", value, expected, tag);
}

/**
 * Test fetch record-header.py
 * @param {string} key
 * @param {string} expected
 * @param {function} assert
 * @return {Promise<string | never>}
 */
function fetch_record_header(key, expected, assert) {
  return fetch("/fetch/metadata/resources/record-header.py?retrieve=true&file=" + key)
      .then(response => response.text())
      .then(text => assert(text, expected))
}

/**
 *
 * @param {string} key
 * @param {string} expected
 * @param {function} assert
 * @param {function} resolve
 * @param {function} reject
 * @return {Promise<any>}
 */
function fetch_record_header_with_catch(key, expected, assert, resolve, reject) {
  return fetch_record_header(key, expected, assert, resolve)
      .then(_ => resolve())
      .catch(e => reject(e));
}
