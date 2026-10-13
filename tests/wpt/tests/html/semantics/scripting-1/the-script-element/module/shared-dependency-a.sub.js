// The token is substituted server-side (WPT .sub.js) from the ?token= query
// parameter, so each test run uses a unique server-stash key and concurrent
// runs (e.g. base vs virtual test suites) don't collide.
import './shared-dependency-b.sub.js?token={{GET[token]}}';
import '../serve-custom-response.py?key={{GET[token]}}';
