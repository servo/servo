const client_hints_list = [
  "device-memory",
  "dpr",
  // "width", (only available for images)
  "viewport-width",
  "rtt",
  "downlink",
  "ect",
  "sec-ch-ua",
  "sec-ch-ua-arch",
  "sec-ch-ua-platform",
  "sec-ch-ua-model",
  "sec-ch-ua-mobile",
  "sec-ch-ua-full-version",
  "sec-ch-ua-platform-version",
  "sec-ch-prefers-color-scheme",
  "sec-ch-prefers-reduced-motion",
  "sec-ch-ua-bitness",
  "sec-ch-viewport-height",
  "sec-ch-device-memory",
  "sec-ch-dpr",
  // "sec-ch-width", (Only available for images)
  "sec-ch-viewport-width",
  "sec-ch-ua-full-version-list",
  "sec-ch-ua-wow64",
  "sec-ch-prefers-reduced-transparency",
];

const client_hints_full_list = client_hints_list.concat(["width", "sec-ch-width"])

const default_on_client_hints = [
  "sec-ch-ua",
  "sec-ch-ua-mobile",
  "sec-ch-ua-platform",
];

const iframe_src =
    "/client-hints/resources/expect-client-hints-headers-iframe.py?";

const expect_iframe_no_hints = iframe_src +
    client_hints_list.map((e) => {
      if(default_on_client_hints.includes(e)) {
        return e+"=true";
      } else {
        return e+"=false";
      }
    }).join("&");

const expect_iframe_hints = iframe_src +
    client_hints_list.map(e => e+"=true").join("&");
