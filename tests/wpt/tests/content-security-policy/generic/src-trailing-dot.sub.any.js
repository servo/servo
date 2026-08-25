// META: global=window,worker

promise_test(t => {
    return fetch("{{location[scheme]}}://{{host}}.:{{location[port]}}"
        + "/content-security-policy/support/resource.py");
}, "Fetch from host with trailing dot should be allowed by CSP.");
