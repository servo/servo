"use strict";
setup({ allow_uncaught_exception: true });

const invalid_namespace_test = (t, resolver) => {
  const result = new Promise((resolve, reject) => {
    const handler = event => {
      reject(event.error);
    };

    window.addEventListener("error", handler);
    t.add_cleanup(() => {
      window.removeEventListener("error", handler);
    });

    t.step_timeout(resolve, 0);
  });

  assert_throws_dom("NAMESPACE_ERR", () => {
    document.evaluate("/foo:bar", document.documentElement, resolver);
  });

  return result;
};
