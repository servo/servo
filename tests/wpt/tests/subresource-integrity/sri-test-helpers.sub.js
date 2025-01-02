// This horrible hack is needed for the 'use-credentials' tests because, on
// response, if port 80 or 443 is the current port, it will not appear to
// the browser as part of the origin string. Since the origin *string* is
// used for CORS access control, instead of the origin itself, if there
// isn't an exact string match, the check will fail. For example,
// "http://example.com" would not match "http://example.com:80", because
// they are not exact string matches, even though the origins are the same.
//
// Thus, we only want the Access-Control-Allow-Origin header to have
// the port if it's not port 80 or 443, since the user agent will elide the
// ports in those cases.
const main_domain = '{{domains[]}}';
const www_domain = '{{domains[www]}}';
const default_port = (location.protocol === 'https:') ? '{{ports[https][0]}}' :
                                                        '{{ports[http][0]}}';

const port_string = (default_port !== '80' && default_port !== '443') ?
                      `:${default_port}` : '';
const www_host_and_port = www_domain + port_string;

// General resource prefixes.
const same_origin_prefix = '/subresource-integrity/';
const xorigin_prefix = `${location.protocol}//${www_host_and_port}/subresource-integrity/`;

// General resource suffixes, for piping CORS headers.
const anonymous = '&pipe=header(Access-Control-Allow-Origin,*)';
const use_credentials = "&pipe=header(Access-Control-Allow-Credentials,true)|" +
                        "header(Access-Control-Allow-Origin," + location.origin + ")";

// Note that all of these style URLs have query parameters started, so any
// additional parameters should be appended starting with '&'.
const xorigin_anon_style = location.protocol
  + '//' + www_host_and_port
  + '/subresource-integrity/crossorigin-anon-style.css?';

const xorigin_creds_style = location.protocol
  + '//' + www_host_and_port
  + '/subresource-integrity/crossorigin-creds-style.css?acao_port='
  + port_string;

const xorigin_ineligible_style = location.protocol
  + '//' + www_host_and_port
  + '/subresource-integrity/crossorigin-ineligible-style.css?';
