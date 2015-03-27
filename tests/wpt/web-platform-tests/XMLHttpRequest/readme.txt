Currently this testsuite tries to provide tests for XMLHttpRequest level 1.
This test suite is not stable and is still under development. Tests may
contain bugs and may change over time as a result of those bugs being fixed.

When more browsers implement XMLHttpRequest level 2 this testsuite will
slowly evolve most likely.

  http://dev.w3.org/2006/webapi/XMLHttpRequest/
  http://dev.w3.org/2006/webapi/XMLHttpRequest-2/

If the folders above give the status of the feature tested you can assume
this is against level 1 unless explicitly stated otherwise.

NOTE: readyState and onreadystatechange are tested throughout the various
tests. statusText is tested together with status.

NOTE: open-url-base* have absolute paths in them. They need to be adjusted
on a per location basis.

NOTE: open-url-base-inserted-after-open.htm, open-url-base-inserted.htm,
send-authentication.htm and open-url-base.htm refer to localhost.


TESTS THAT ARE UNSTABLE AND (PROBABLY) NEED CHANGES
  responsexml-basic (see email WHATWG)
  send-authentication (see "user:password" debacle)


TESTS NOT STARTED ON YET

<iframe> document.domain = w3.org create cross-origin xhr object
