# Server-Sent Events Test Collection

Server-Sent Events [latest draft](http://dev.w3.org/html5/eventsource/).

Following up work done during the TestTWF 2012 Paris event:

Most tests comes from [Opera](http://tc.labs.opera.com/apis/EventSource/), are from august 2010 and probably only valid against [spec rev. ~1.139](http://dev.w3.org/cvsweb/~checkout~/html5/eventsource/Overview.html?rev=1.139;content-type=text%2Fhtml). You can check the following diff :

[diff between 1.139 (23 Jul 2010) and 1.229 (25 Oct. 2012) revisions](http://dev.w3.org/cvsweb/html5/eventsource/Overview.html.diff?r1=text&tr1=1.139&r2=text&tr2=1.229)

to get an idea of what needs to get updated.

##DONE (updated against rev. 1.229):
- **eventsource-constructor-url-bogus.htm**: whatwg r6602: renamed SYNTAX_ERR to SyntaxError

- **eventsource-constructor-stringify.htm**: still valid. bugfix.

##TODO (need to be updated against rev. 1.229):
- **eventsource-cross-origin.htm**, **eventsource-constructor-non-same-origin.htm**: whatwg 6255 6257: allow CORS

##TOCHECK (need to check if the test is still valid against rev.1.229):
eventsource-close.htm
eventsource-constructor-document-domain.htm
eventsource-constructor-url-multi-window.htm
eventsource-eventtarget.htm
eventsource-onmessage.htm
eventsource-onopen.htm
eventsource-prototype.htm
eventsource-reconnect.htm
eventsource-url.htm
format-bom-2.htm
format-bom.htm
format-comments.htm
format-field-data.htm
format-field-event-empty.htm
format-field-event.htm
format-field-id-2.htm
format-field-id.htm
format-field-parsing.htm
format-field-retry-bogus.htm
format-field-retry-empty.htm
format-field-retry.htm
format-field-unknown.htm
format-leading-space.htm
format-mime-bogus.htm
format-mime-trailing-semicolon.htm
format-mime-valid-bogus.htm
format-newlines.htm
format-utf-8.htm
request-accept.htm
request-cache-control.htm
request-credentials.htm
request-redirect.htm
request-status-error.htm
