// Perform real navigations as well as history.pushState navigations to the
// loaded page until we reach the specified navigation limit. This will be the
// first function that runs in the test, and will result in the test
// reloading/restarting until we reach the desired history length.
function maybeNavigateForHistory() {
  const kNavigationLimit = 5

  const url = new URL(location.href);

  // First, perform some real navigations as well as history.pushState to this
  // same page. Normally this would increase `history.length`.
  if (url.searchParams.get("navigationCount") == null)
    url.searchParams.append("navigationCount", 1);

  let navigationCount = parseInt(url.searchParams.get("navigationCount"));

  if (navigationCount <= kNavigationLimit) {
    url.searchParams.set('navigationCount', ++navigationCount);
    location.href = url;
    history.pushState({} , "");
    return;
  }
}
