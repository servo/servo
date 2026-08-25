async function assertSimplestScriptRuns(remoteContextWrapper) {
  assert_equals(
      await remoteContextWrapper.executeScript(() => {
        return 1;
      }),
      1, 'simplest script runs');
}

async function assertFunctionRuns(
    remoteContextWrapper, functionToRun, expectedReturn) {
  assert_equals(
      await remoteContextWrapper.executeScript(functionToRun), expectedReturn,
      'function runs');
}

async function assertOriginIsAsExpected(remoteContextWrapper, expectedOrigin) {
  assert_equals(
      await remoteContextWrapper.executeScript(() => {
        return location.origin;
      }),
      expectedOrigin, 'verify origin');
}

async function assertWindowNameEquals(remoteContextWrapper, expectedName) {
  assert_equals(
      await remoteContextWrapper.executeScript(() => {
        return window.name;
      }),
      expectedName, 'verify name');
}

async function assertWindowHasOpenerEquals(remoteContextWrapper, hasParent) {
  assert_equals(
      await remoteContextWrapper.executeScript(() => {
        return !!window.opener;
      }),
      hasParent, 'verify opener');
}

async function assertHeaderIsAsExpected(
    remoteContextWrapper, headerName, headerValue) {
  assert_equals(
      headerValue,
      await remoteContextWrapper.executeScript(async (headerName) => {
        const res = await fetch(location);
        return res.headers.get(headerName);
      }, [headerName]), 'header is set');
}

function getUrlType(location) {
  const url = new URL(location);
  return url.searchParams.get('urlType');
}
