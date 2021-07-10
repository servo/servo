These policies are served via the Python script at /.well-known/origin-policy. Their filenames must be in the form `subdomain human-facing-string-with-no-spaces.json`. They will be served in response to requests to that subdomain.

The human-facing string has no impact on the tests, and just makes it easier to scroll through the list.

The list of potential hostnames is created by `tools/serve/serve.py`'s `_make_origin_policy_subdomains` function, and can be expanded as necessary.

At the moment, the origin policies starting at 100 downward have special handling in the `/.well-known/origin-policy` handler, and might require consulting that file to get the full context. The ones starting at 1 upward are handled generically. If they ever start meeting in the middle we can reevaluate this scheme.
