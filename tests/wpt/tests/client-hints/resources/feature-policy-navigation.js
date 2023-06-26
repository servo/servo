const meta_name_enabled = [
  "sec-ch-device-memory",
  "device-memory",
  "sec-ch-dpr",
  "dpr",
  "sec-ch-viewport-width",
  "viewport-width",
  "sec-ch-ua",
  "sec-ch-ua-mobile",
  "sec-ch-ua-platform",
];

const meta_name_client_hints = iframe_src +
    client_hints_list.map((e) => {
      if(meta_name_enabled.includes(e)) {
        return e+"=true";
      } else {
        return e+"=false";
      }
    }).join("&");

const cross_origin_enabled = [
  "device-memory",
  "sec-ch-device-memory",
  "sec-ch-ua-platform",
];

const cross_origin_client_hints = iframe_src +
    client_hints_list.map((e) => {
      if(cross_origin_enabled.includes(e)) {
        return e+"=true";
      } else {
        return e+"=false";
      }
    }).join("&");

const same_origin_disabled = [
  "dpr",
  "sec-ch-dpr",
  "sec-ch-ua-mobile",
];

const same_origin_client_hints = iframe_src +
    client_hints_list.map((e) => {
      if(same_origin_disabled.includes(e)) {
        return e+"=false";
      } else {
        return e+"=true";
      }
    }).join("&");

const test_frame = (origin, url, allow, message) => {
  promise_test(() => {
    return new Promise((resolve, reject) => {
      let frame = document.createElement('iframe');
      frame.allow = allow;
      window.addEventListener('message', function(e) {
        try {
          assert_equals(typeof e.data, "string");
          assert_equals(e.data, "PASS");
        } catch {
          reject(e.data);
        }
        resolve();
      });
      document.body.appendChild(frame);
      // Writing to |frame.src| triggers the navigation, so
      // everything else need to happen first.
      frame.src = get_host_info()[origin] + url;
    });
  }, message);
}
