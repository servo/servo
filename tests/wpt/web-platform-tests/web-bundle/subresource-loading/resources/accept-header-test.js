promise_test(async () => {
  await addWebBundleElementAndWaitForLoad(
    '../resources/check-accept-header-and-return-bundle.py',
    /*resources=*/[]);
},
'"Accept:" header in a request for a bundle should contain ' +
'application/webbundle MIME type.');
