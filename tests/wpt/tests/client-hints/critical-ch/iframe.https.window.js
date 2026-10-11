// META: script=resources/util.js

make_iframe_test = (url) =>
  async (t) => {
    await clear_client_hints();
    var iframe = document.createElement("iframe");
    t.add_cleanup(() => iframe.remove());
    var message = new Promise((resolve) => {
      window.addEventListener('message', (e) => resolve(e.data), {once: true});
    });
    iframe.src = url;
    document.body.appendChild(iframe);
    assert_equals(await message, "FAIL");
  }

promise_test(make_iframe_test(ECHO_URL), "Critical-CH iframe");

promise_test(make_iframe_test(ECHO_URL+"?multiple=true"), "Critical-CH w/ multiple headers and iframe");
