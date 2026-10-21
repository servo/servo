// META: title=Link preload and prefetch of a blob URL succeed when the URL is revoked after the link is inserted

function link_blob_url_test(rel, as, revoke, description) {
  promise_test(async t => {
    const url = URL.createObjectURL(new Blob(["hello"], { type: "text/plain" }));
    const link = document.createElement("link");
    link.rel = rel;
    if (as) {
      link.as = as;
    }
    link.href = url;
    const event = new Promise(resolve => {
      link.onload = link.onerror = e => resolve(e.type);
    });
    t.add_cleanup(() => {
      link.remove();
      URL.revokeObjectURL(url);
    });

    // Inserting the link creates the request, which resolves the blob URL.
    // Revoking the URL afterwards must not affect that request.
    document.head.appendChild(link);
    if (revoke) {
      URL.revokeObjectURL(url);
    }

    assert_equals(await event, "load", "link fired a load event");
  }, description);
}

link_blob_url_test("preload", "fetch", false, "rel=preload of a blob URL");
link_blob_url_test("preload", "fetch", true, "rel=preload of a blob URL revoked after insertion");
link_blob_url_test("prefetch", null, false, "rel=prefetch of a blob URL");
link_blob_url_test("prefetch", null, true, "rel=prefetch of a blob URL revoked after insertion");
