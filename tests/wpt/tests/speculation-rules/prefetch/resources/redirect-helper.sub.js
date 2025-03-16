// Prefetched redirect chain (as test variant parameters):
// origin=same-origin:
// initiator-(same-origin)->prefetchInitialUrl-(same-origin)->prefetchFinalUrl
// origin=cross-site-initial:
// initiator-(cross-site)-->prefetchInitialUrl-(same-origin)->prefetchFinalUrl
// origin=cross-site-redirect:
// initiator-(same-origin)->prefetchInitialUrl-(cross-site)-->prefetchFinalUrl
const { origin } = Object.fromEntries(new URLSearchParams(location.search));

// `prefetchTiming`:
// - 'redirect-received-after-navigation-start':
//   prefetch is started before navigation starts but the redirect response
//   is received after navigation starts.
// - Otherwise:
//   prefetch (including its redirects) is completed before navigation starts.
async function prepare(t, prefetchTiming) {
  const agent = await spawnWindow(t);

  let prefetchFinalUrl;
  let prefetchInitialOrigin;
  if (origin === 'same-origin') {
    prefetchFinalUrl = agent.getExecutorURL({ page: 2 });
    prefetchInitialOrigin = location.origin;
  } else if (origin === 'cross-site-initial') {
    prefetchFinalUrl = agent.getExecutorURL(
        { page: 2, hostname: '{{hosts[alt][www]}}' });
    prefetchInitialOrigin = prefetchFinalUrl.origin;
  } else if (origin === 'cross-site-redirect') {
    prefetchFinalUrl = agent.getExecutorURL(
        { page: 2, hostname: '{{hosts[alt][www]}}' });
    prefetchInitialOrigin = location.origin;
  } else {
    t.assert_unreached('Invalid origin option: ' + origin);
  }

  let prefetchInitialUrl;
  if (prefetchTiming === 'redirect-received-after-navigation-start') {
    // Because `forceSinglePrefetch()` waits for 2 seconds, we put 4-second
    // delay here to make the redirect response is received after `navigate()`
    // below.
    prefetchInitialUrl = new URL('/common/slow-redirect.py?delay=4',
                                 prefetchInitialOrigin);
    prefetchInitialUrl.searchParams.set('location', prefetchFinalUrl);
  } else {
    // Because `forceSinglePrefetch()` waits for 2 seconds, the redirect and
    // final responses are expected to be received before `navigate()` below.
    prefetchInitialUrl = new URL('/common/redirect.py', prefetchInitialOrigin);
    prefetchInitialUrl.searchParams.set('location', prefetchFinalUrl);
  }

  const redirectToPrefetchInitialUrl = new URL('/common/redirect.py',
                                               location.href);
  redirectToPrefetchInitialUrl.searchParams.set(
      'location', prefetchInitialUrl);

  const redirectToPrefetchFinalUrl = new URL('/common/redirect.py',
                                             location.href);
  redirectToPrefetchFinalUrl.searchParams.set(
      'location', prefetchFinalUrl);

  // `type` is set just to make `redirectToPrefetchFinalUrl` different
  // from `prefetchInitialUrl`.
  redirectToPrefetchFinalUrl.searchParams.set(
      'type', 'navigation');

  return {agent,
          prefetchInitialUrl,
          prefetchFinalUrl,
          redirectToPrefetchInitialUrl,
          redirectToPrefetchFinalUrl};
}
