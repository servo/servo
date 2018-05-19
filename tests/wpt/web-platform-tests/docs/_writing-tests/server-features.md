---
layout: page
title: Server Features
order: 12
---

## Advanced Testing Features

Certain test scenarios require more than just static HTML
generation. This is supported through the
[wptserve](http://wptserve.readthedocs.io) server, and controlled by
[file name flags][file names]. Several scenarios in particular are common:


### Tests Involving Multiple Origins

Our test servers are guaranteed to be accessible through two domains
and five subdomains under each. The 'main' domain is unnamed; the
other is called 'alt'. These subdomains are: `www`, `www1`, `www2`,
`天気の良い日`, and `élève`; there is also `nonexistent` which is
guaranteed not to resolve. In addition, the HTTP server listens on two
ports, and the WebSockets server on one. These subdomains and ports
must be used for cross-origin tests.

Tests must not hardcode the hostname of the server that they expect to
be running on or the port numbers, as these are not guaranteed by the
test environment. Instead they can get this information in one of two
ways:

* From script, using the `location` API.

* By using a textual substitution feature of the server.

In order for the latter to work, a file must either have a name of the
form `{name}.sub.{ext}` e.g. `example-test.sub.html` or be referenced
through a URL containing `pipe=sub` in the query string
e.g. `example-test.html?pipe=sub`. The substitution syntax uses `{%
raw %}{{ }}{% endraw %}` to delimit items for substitution. For
example to substitute in the main host name, one would write:
`{% raw %}{{host}}{% endraw %}`.

To get full domains, including subdomains, there is the `hosts`
dictionary, where the first dimension is the name of the domain, and
the second the subdomain. For example, `{% raw %}{{hosts[][www]}}{%
endraw %}` would give the `www` subdomain under the main (unnamed)
domain, and `{% raw %}{{hosts[alt][élève]}}{% endraw %}` would give
the `élève` subdomain under the alt domain.

For mostly historic reasons, the subdomains of the main domain are
also available under the `domains` dictionary; this is identical to
`hosts[]`.

Ports are also available on a per-protocol basis. For example, `{% raw
%}{{ports[ws][0]}}{% endraw %}` is replaced with the first (and only)
WebSockets port, while `{% raw %}{{ports[http][1]}}{% endraw %}` is
replaced with the second HTTP port.

The request URL itself can be used as part of the substitution using
the `location` dictionary, which has entries matching the
`window.location` API. For example, `{% raw %}{{location[host]}}{%
endraw %}`is replaced by `hostname:port` for the current request,
matching `location.host`.


### Tests Requiring Special Headers

For tests requiring that a certain HTTP header is set to some static
value, a file with the same path as the test file except for an an
additional `.headers` suffix may be created. For example for
`/example/test.html`, the headers file would be
`/example/test.html.headers`. This file consists of lines of the form

    header-name: header-value

For example

    Content-Type: text/html; charset=big5

To apply the same headers to all files in a directory use a
`__dir__.headers` file. This will only apply to the immediate
directory and not subdirectories.

Headers files may be used in combination with substitutions by naming
the file e.g. `test.html.sub.headers`.


### Tests Requiring Full Control Over The HTTP Response

For full control over the request and response the server provides the
ability to write `.asis` files; these are served as literal HTTP
responses. It also provides the ability to write Python scripts that
have access to request data and can manipulate the content and timing
of the response. For details see the
[wptserve documentation](https://wptserve.readthedocs.org).


[file names]: {{ site.baseurl }}{% link _writing-tests/file-names.md %}
