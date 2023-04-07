// META: script=/common/get-host-info.sub.js

const t1 = async_test("HTTP fetch");
const t2 = async_test("HTTPS fetch");

onmessage = function(e) {
  const {protocol, success} = e.data;
  if (protocol == "http:") {
    t1.step(() => assert_false(success, "success"));
    t1.done();
  } else if (protocol == "https:") {
    t2.step(() => assert_true(success, "success"));
    t2.done();
  } else {
    [t1, t2].forEach(t => {
      t.step(() => assert_unreached("Unknown message"));
      t.done();
    });
  }
};

const httpsFrame = document.createElement("iframe");
httpsFrame.src = get_host_info().HTTPS_ORIGIN + "/mixed-content/resources/middle-frame.html";

document.body.appendChild(httpsFrame);
