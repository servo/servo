promise_test(async () => {
  const element = createWebBundleElement(
      '/xhr/resources/delay.py?ms=100000',
      ['/xhr/resources/dummy']);
  document.body.appendChild(element);
  const waitUntilFail = new Promise((resolve) => {
    fetch("/xhr/resources/dummy").then(() => {}, resolve);
  });
  document.body.removeChild(element);
  await waitUntilFail;
},
'On-going subresource loading should fail immediately when the element is ' +
'removed.');
