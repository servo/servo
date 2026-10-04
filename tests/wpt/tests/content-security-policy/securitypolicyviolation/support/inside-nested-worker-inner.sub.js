// The cross-origin static import violates the "script-src 'self'" policy of the
// outer worker.
import '{{location[scheme]}}://{{domains[www1]}}:{{location[port]}}/content-security-policy/support/ping.js';
