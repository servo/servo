// META: timeout=long

const dataURL = `data:text/html,...`;
const encodedDataURL = encodeURIComponent(dataURL);

[dataURL, `resources/redirect.py?location=${encodedDataURL}`].forEach(url => {
  [undefined, "opener", "noopener", "noreferrer"].forEach(opener => {
    async_test(t => {
      const popup = window.open(url, "", opener);
      t.step_timeout(() => {
        if (opener === "noopener" || opener == "noreferrer") {
          assert_equals(popup, null);
        } else {
          assert_true(popup.closed);
        }
        t.done();
      }, 1500);
    }, `Navigating a popup using window.open("${url}", "", "${opener}")`);
  });
});
