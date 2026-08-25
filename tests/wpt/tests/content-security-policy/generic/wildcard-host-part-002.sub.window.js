setup(_ => {
  const meta = document.createElement("meta");
  meta.httpEquiv = "content-security-policy";
  meta.content = "img-src http://*:{{ports[http][0]}}/content-security-policy/support/pass.png";
  document.head.appendChild(meta);
});

async_test((t) => {
  const img = document.createElement("img");
  img.onload = t.step_func_done();
  img.onerror = t.unreached_func("Image from default host should have loaded.");
  img.src = "http://{{domains[]}}:{{ports[http][0]}}/content-security-policy/support/pass.png";
}, "Host wildcard allows arbitrary hosts (default host).");

async_test((t) => {
  const img = document.createElement("img");
  img.onload = t.step_func_done();
  img.onerror = t.unreached_func("Image from www2 host should have loaded.");
  img.src = "http://{{domains[www1]}}:{{ports[http][0]}}/content-security-policy/support/pass.png";
}, "Host wildcard allows arbitrary hosts (www1).");

async_test((t) => {
  const img = document.createElement("img");
  img.onload = t.step_func_done();
  img.onerror = t.unreached_func("Image from www2 host should have loaded.");
  img.src = "http://{{domains[www2]}}:{{ports[http][0]}}/content-security-policy/support/pass.png";
}, "Host wildcard allows arbitrary hosts (www2).");

async_test((t) => {
  const img = document.createElement("img");
  img.onerror = t.step_func_done();
  img.onload = t.unreached_func("image from {{domains[]}}:{{ports[http][1]}} should have been blocked.");
  img.src = "http://{{domains[]}}:{{ports[http][1]}}/content-security-policy/support/pass.png"
}, "Host wildcard doesn't affect port matching (port {{ports[http][1]}}).");

async_test((t) => {
  const img = document.createElement("img");
  img.onerror = t.step_func_done();
  img.onload = t.unreached_func("image fail.png should have been blocked.");
  img.src = "http://{{domains[]}}:{{ports[http][0]}}/content-security-policy/support/fail.png"
}, "Host wildcard doesn't affect path matching.");
