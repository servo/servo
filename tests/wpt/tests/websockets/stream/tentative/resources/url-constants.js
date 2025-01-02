// The file including this must also include ../constants.sub.js to pick up the
// necessary constants.

const {BASEURL, ECHOURL} = (() => {
  const BASEURL = SCHEME_DOMAIN_PORT;
  const ECHOURL = `${BASEURL}/echo`;
  return {BASEURL, ECHOURL};
})();
