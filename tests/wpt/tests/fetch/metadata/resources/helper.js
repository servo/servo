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
}

function validate_expectations_custom_url(url, header, expected, tag) {
  return fetch(url, header)
    .then(response => response.text())
    .then(text => {
      assert_not_equals(text, "No header has been recorded");
      let value = JSON.parse(text);
      test(t => assert_equals(value.dest, expected.dest), `${tag}: sec-fetch-dest`);
      test(t => assert_equals(value.mode, expected.mode), `${tag}: sec-fetch-mode`);
      test(t => assert_equals(value.site, expected.site), `${tag}: sec-fetch-site`);
      test(t => assert_equals(value.user, expected.user), `${tag}: sec-fetch-user`);
    });
}

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

  test(t => assert_equals(value.dest, expected.dest), `${tag}: sec-fetch-dest`);
  test(t => assert_equals(value.mode, expected.mode), `${tag}: sec-fetch-mode`);
  test(t => assert_equals(value.site, expected.site), `${tag}: sec-fetch-site`);
  test(t => assert_equals(value.user, expected.user), `${tag}: sec-fetch-user`);
}
