// See also /fetch/api/response/json.any.js

async_test(t => {
  const xhr = new XMLHttpRequest();
  xhr.responseType = "json";
  xhr.open("GET", `data:,\uFEFF{ "b": 1, "a": 2, "b": 3 }`);
  xhr.send();
  xhr.onload = t.step_func_done(() => {
    assert_array_equals(Object.keys(xhr.response), ["b", "a"]);
    assert_equals(xhr.response.a, 2);
    assert_equals(xhr.response.b, 3);
  });
}, "Ensure the correct JSON parser is used");

async_test(t => {
  const client = new XMLHttpRequest();
  client.responseType = 'json';
  client.open("GET", "resources/utf16-bom.json");
  client.send();
  client.onload = t.step_func_done(() => {
    assert_equals(client.response, null);
  });
}, "Ensure UTF-16 results in an error");
