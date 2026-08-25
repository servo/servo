function expect_fetched_onload(uuid, expectation) {
  return new Promise(resolve => {
    addEventListener('load', resolve);
  }).then(async () => {
    const result = await get_result(uuid);
    if (expectation) {
      assert_not_equals(result, '', 'speculative case did not fetch');
    } else {
      assert_equals(result, '', 'speculative case incorrectly fetched');
    }
    return result;
  });
}

function compare_with_nonspeculative(uuid, title, test_nonspeculative) {
  return function(speculative_result) {
    if (!test_nonspeculative) {
      return Promise.resolve();
    }
    return new Promise(resolve => {
      const iframe = document.createElement('iframe');
      iframe.onload = resolve;
      iframe.src = `../resources/${title}-nonspeculative.sub.html?uuid=${uuid}`;
      document.body.appendChild(iframe);
    }).then(async () => {
      const result = await get_result(uuid);
      if (speculative_result === '') {
        assert_equals(result, '', 'non-speculative case incorrectly fetched')
      } else {
        assert_not_equals(result, '', 'non-speculative case did not fetch');
        const speculative_headers = speculative_result.trim().split("\n");
        const nonspeculative_headers = result.trim().split("\n");
        assert_equals(speculative_headers.length, nonspeculative_headers.length, 'expected the same number of headers between speculative and non-speculative')
        for (let i = 0; i < speculative_headers.length; ++i) {
          let [s_header, s_value] = split_header(speculative_headers[i]);
          let [ns_header, ns_value] = split_header(nonspeculative_headers[i]);
          assert_equals(s_header, ns_header, 'expected the order of headers to match between speculative and non-speculative');
          assert_equals(s_value, ns_value, `expected \`${s_header}\` values to match between speculative and non-speculative`);
        }
      }
    });
  }
}

function split_header(line) {
  let [header, value] = line.split(': ');
  header = header.toLowerCase();
  value = value.trim();
  if (header === 'referer') {
    value = value.replace(/\/generated\/.+$/, '/generated/...');
  }
  return [header, value];
}

async function get_result(uuid) {
  const response = await fetch(`/html/syntax/speculative-parsing/resources/stash.py?action=take&uuid=${uuid}`);
  return await response.text();
}
