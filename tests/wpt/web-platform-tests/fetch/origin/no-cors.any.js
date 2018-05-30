// META: script=/common/utils.js
// META: script=/common/get-host-info.sub.js

promise_test(async function() {
  const stash = token(),
        origins = get_host_info(),
        redirectPath = "/fetch/origin/resources/redirect-and-stash.py";

  // Cross-origin -> same-origin will result in setting the tainted origin flag for the second
  // request.
  let url = origins.HTTP_ORIGIN + redirectPath + "?stash=" + stash;
  url = origins.HTTP_REMOTE_ORIGIN + redirectPath + "?stash=" + stash + "&location=" + encodeURIComponent(url);

  await fetch(url, { mode: "no-cors", method: "POST" });

  const json = await (await fetch(redirectPath + "?dump&stash=" + stash)).json();

  assert_equals(json[0], origins.HTTP_ORIGIN, "first origin should equal this origin");
  assert_equals(json[1], "null", "second origin should be opaque and therefore null");
}, "Origin header and 308 redirect");
