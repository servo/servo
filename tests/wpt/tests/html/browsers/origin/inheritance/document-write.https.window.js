// META: script=/common/get-host-info.sub.js

// To use document.domain, we need to start from a subdomain.
//
// For document.domain setter to work, some web browser require the
// |Origin-Agent-Cluster: ?0| header to be set uniformly on the origin.
const origin = get_host_info().OTHER_ORIGIN;
const openee = window.open(
  origin + '/html/browsers/origin/inheritance/resources/document-write.html')
fetch_tests_from_window(openee);
