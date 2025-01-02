// META: title=XMLHttpRequest: upload progress event
// META: script=/common/get-host-info.sub.js

const remote = get_host_info().HTTP_REMOTE_ORIGIN + "/xhr/resources/corsenabled.py",
  redirect = "resources/redirect.py?code=307&location=" + remote;

[remote, redirect].forEach(url => {
  async_test(test => {
    const client = new XMLHttpRequest();
    const data = "On time: " + url;
    client.upload.onprogress = test.step_func_done(e => {
      assert_true(e.lengthComputable);
      assert_equals(e.total, data.length);
    });
    client.onload = test.unreached_func();
    client.open("POST", url);
    client.send(data);
  }, "Upload events registered on time (" + url + ")");
});

[remote, redirect].forEach(url => {
  async_test(test => {
    const client = new XMLHttpRequest();
    client.onload = test.step_func_done();
    client.open("POST", url);
    client.send("Too late: " + url);
    client.upload.onloadstart = test.unreached_func(); // registered too late
    client.upload.onprogress = test.unreached_func(); // registered too late
  }, "Upload events registered too late (" + url + ")");
});
