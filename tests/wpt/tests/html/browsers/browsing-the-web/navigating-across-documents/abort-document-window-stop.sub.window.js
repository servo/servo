// META: title=Child that immediately stops window during loading does not crash
// META: script=../resources/helpers.js

// This test reproduces Servo issue https://github.com/servo/servo/issues/44720

async_test((test) => {
  addIframe().then(iframe => {
    iframe.contentWindow.addEventListener("unload", test.step_func_done(() => {
      assert_equals(
        iframe.contentWindow.location.href,
        "http://{{hosts[][]}}:{{ports[http][0]}}/html/browsers/browsing-the-web/navigating-across-documents/resources/child-immediately-stops-during-loading.html"
      );
    }));

    iframe.src = "./resources/child-immediately-stops-during-loading.html";
  });
});
