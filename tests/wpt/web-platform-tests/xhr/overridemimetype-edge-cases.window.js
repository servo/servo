const testURL = "resources/status.py?type=" + encodeURIComponent("text/plain;charset=windows-1252") + "&content=%C2%F0";

async_test(t => {
  const client = new XMLHttpRequest();
  let secondTime = false;
  client.onload = t.step_func(() => {
    if(!secondTime) {
      assert_equals(client.responseText, "\uFFFD");
      secondTime = true;
      client.open("GET", testURL);
      client.send();
    } else {
      assert_equals(client.responseText, "Âð");
      t.done();
    }
  });
  client.open("GET", testURL);
  client.overrideMimeType("text/plain;charset=UTF-8")
  client.send();
}, "overrideMimeType() state needs to be reset across requests");

async_test(t => {
  const client = new XMLHttpRequest();
  client.onload = t.step_func_done(() => {
    assert_equals(client.responseText, "Âð")
  });
  client.open("GET", testURL);
  client.overrideMimeType("text/xml");
  client.send();
}, "If charset is not overridden by overrideMimeType() the original continues to be used");

async_test(t => {
  const client = new XMLHttpRequest();
  client.onload = t.step_func_done(() => {
    assert_equals(client.responseText, "\uFFFD")
  });
  client.open("GET", testURL);
  client.overrideMimeType("text/plain;charset=342");
  client.send();
}, "Charset can be overridden by overrideMimeType() with a bogus charset");
