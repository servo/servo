These policies are served via the Python script at /.well-known/origin-policy. Their filenames must be in the form `subdomain human-facing-string-with-no-spaces.json`. They will be served in response to requests to that subdomain.

The human-facing string has no impact on the tests, and just makes it easier to scroll through the list.

The list of potential hostnames is created by `tools/serve/serve.py`'s `_make_origin_policy_subdomains` function, and can be expanded as necessary.
