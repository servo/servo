// This will transmit two chunks TEST_CHUNK and then garbage, which should result in an error.
const url = "/fetch/api/resources/bad-chunk-encoding.py?ms=1&count=2";

test(() => {
  client = new XMLHttpRequest();
  client.open("GET", url, false);
  assert_throws_dom("NetworkError", () => client.send());
}, "Synchronous XMLHttpRequest should throw on bad chunk");

async_test(t => {
  client = new XMLHttpRequest();
  client.open("GET", url, true);
  client.onreadystatechange = t.step_func(() => {
    if (client.readyState === 3) {
      assert_true(client.responseText.indexOf("TEST_CHUNK") !== -1);
    }
  });
  client.onerror = t.step_func_done(() => {
    assert_equals(client.responseText, "");
  });
  client.onload = t.unreached_func();
  client.send();
}, "Asynchronous XMLHttpRequest should clear response on bad chunk");
