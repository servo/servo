// META: script=/fetch/orb/resources/utils.js

const url =
  "http://{{domains[www1]}}:{{ports[http][0]}}/fetch/orb/resources/image.png";

promise_test(async () => {
  let headers = new Headers([["Range", "bytes=0-99"]]);
  await fetchORB(
    url,
    { headers },
    header("Content-Range", "bytes 0-99/1010"),
    "slice(null,100)",
    "status(206)"
  );
}, "ORB shouldn't block opaque range of image/png starting at zero");

promise_test(
  t =>
    promise_rejects_js(
      t,
      TypeError,
      fetchORB(
        url,
        { headers: new Headers([["Range", "bytes 10-99"]]) },
        header("Content-Range", "bytes 10-99/1010"),
        "slice(10,100)",
        "status(206)"
      )
    ),
  "ORB should block opaque range of image/png not starting at zero, that isn't subsequent"
);
