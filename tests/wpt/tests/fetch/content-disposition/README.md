# Multiple `Content-Disposition` response headers

`Content-Disposition` ([RFC 6266](https://www.rfc-editor.org/rfc/rfc6266.html)) is not
defined as a list-based field, so a server is not supposed to send it more than once
([RFC 9110 section 5.3](https://www.rfc-editor.org/rfc/rfc9110#section-5.3)), and
RFC 6266 says nothing about how a recipient should combine repeated field lines.
Servers send them anyway.

These tests are `.tentative` because that is unspecified.  They assert the Fetch header
list model: both field lines are kept, they are combined with `, ` when read back, and
the response is not a network error.  The same assertions run over HTTP/1.1 and HTTP/2
so that differences between protocol versions within one implementation are visible.

The "is not a network error" and "are combined" assertions are separate because an
implementation may load the response while exposing only one field line.

See https://bugzilla.mozilla.org/show_bug.cgi?id=1982182 and
https://github.com/whatwg/fetch/issues/1156 for context.
