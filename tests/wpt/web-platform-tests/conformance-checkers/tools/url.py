# -*- coding: utf-8 -*-
import os
ccdir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
# based on https://github.com/w3c/web-platform-tests/blob/275544eab54a0d0c7f74ccc2baae9711293d8908/url/urltestdata.txt
invalid = {
    "scheme-trailing-tab": "a:\tfoo.com",
    "scheme-trailing-newline": "a:\nfoo.com",
    "scheme-trailing-cr": "a:\rfoo.com",
    "scheme-trailing-space": "a: foo.com",
    "scheme-trailing-tab": "a:\tfoo.com",
    "scheme-trailing-newline": "a:\nfoo.com",
    "scheme-trailing-cr": "a:\rfoo.com",
    "scheme-http-no-slash": "http:foo.com",
    "scheme-http-no-slash-colon": "http::@c:29",
    "scheme-http-no-slash-square-bracket": "http:[61:27]/:foo",
    "scheme-http-backslash": "http:\\\\foo.com\\",
    "scheme-http-single-slash": "http:/example.com/",
    "scheme-ftp-single-slash": "ftp:/example.com/",
    "scheme-https-single-slash": "https:/example.com/",
    "scheme-data-single-slash": "data:/example.com/",
    "scheme-ftp-no-slash": "ftp:example.com/",
    "scheme-https-no-slash": "https:example.com/",
    "scheme-javascript-no-slash-malformed": "javascript:example.com/",
    "userinfo-password-bad-chars": "http://&a:foo(b]c@d:2/",
    "userinfo-username-contains-at-sign": "http://::@c@d:2",
    "userinfo-backslash": "http://a\\b:c\\d@foo.com",
    "host-space": "http://example .org",
    "host-tab": "http://example\t.org",
    "host-newline": "http://example.\norg",
    "host-cr": "http://example.\rorg",
    "host-square-brackets-port-contains-colon": "http://[1::2]:3:4",
    "port-single-letter": "http://f:b/c",
    "port-multiple-letters": "http://f:fifty-two/c",
    "port-leading-colon": "http://2001::1",
    "port-leading-colon-bracket-colon": "http://2001::1]:80",
    "path-leading-backslash-at-sign": "http://foo.com/\\@",
    "path-leading-colon-backslash": ":\\",
    "path-leading-colon-chars-backslash": ":foo.com\\",
    "path-relative-square-brackets": "[61:24:74]:98",
    "fragment-contains-hash": "http://foo/path#f#g",
    "path-percent-encoded-malformed": "http://example.com/foo/%2e%2",
    "path-bare-percent-sign": "http://example.com/foo%",
    "path-u0091": u"http://example.com/foo\u0091".encode('utf-8'),
    "userinfo-username-contains-pile-of-poo": "http://💩:foo@example.com",
    "userinfo-password-contains-pile-of-poo": "http://foo:💩@example.com",
    "host-hostname-in-brackets": "http://[www.google.com]/",
    "host-empty": "http://",
    "host-empty-with-userinfo": "http://user:pass@/",
    "port-leading-dash": "http://foo:-80/",
    "host-empty-userinfo-empty": "http://@/www.example.com",
    "host-invalid-unicode": u"http://\ufdd0zyx.com".encode('utf-8'),
    "host-invalid-unicode-percent-encoded": "http://%ef%b7%90zyx.com",
    "host-double-percent-encoded": u"http://\uff05\uff14\uff11.com".encode('utf-8'),
    "host-double-percent-encoded-percent-encoded": "http://%ef%bc%85%ef%bc%94%ef%bc%91.com",
    "host-u0000-percent-encoded": u"http://\uff05\uff10\uff10.com".encode('utf-8'),
    "host-u0000-percent-encoded-percent-encoded": "http://%ef%bc%85%ef%bc%90%ef%bc%90.com",
}
invalid_absolute = invalid.copy()

invalid_url_code_points = {
    "fragment-backslash": "#\\",
    "fragment-leading-space": "http://f:21/b# e",
    "path-contains-space": "/a/ /c",
    "path-leading-space": "http://f:21/ b",
    "path-tab": "http://example.com/foo\tbar",
    "path-trailing-space": "http://f:21/b ?",
    "port-cr": "http://f:\r/c",
    "port-newline": "http://f:\n/c",
    "port-space": "http://f: /c",
    "port-tab": "http://f:\t/c",
    "query-leading-space": "http://f:21/b? d",
    "query-trailing-space": "http://f:21/b?d #",
}
invalid.update(invalid_url_code_points)
invalid_absolute.update(invalid_url_code_points)

valid_absolute = {
    "scheme-private": "a:foo.com",
    "scheme-private-slash": "foo:/",
    "scheme-private-slash-slash": "foo://",
    "scheme-private-path": "foo:/bar.com/",
    "scheme-private-path-leading-slashes-only": "foo://///////",
    "scheme-private-path-leading-slashes-chars": "foo://///////bar.com/",
    "scheme-private-path-leading-slashes-colon-slashes": "foo:////://///",
    "scheme-private-single-letter": "c:/foo",
    "scheme-private-single-slash": "madeupscheme:/example.com/",
    "scheme-file-single-slash": "file:/example.com/",
    "scheme-ftps-single-slash": "ftps:/example.com/",
    "scheme-gopher-single-slash": "gopher:/example.com/",
    "scheme-ws-single-slash": "ws:/example.com/",
    "scheme-wss-single-slash": "wss:/example.com/",
    "scheme-javascript-single-slash": "javascript:/example.com/",
    "scheme-mailto-single-slash": "mailto:/example.com/",
    "scheme-private-no-slash": "madeupscheme:example.com/",
    "scheme-ftps-no-slash": "ftps:example.com/",
    "scheme-gopher-no-slash": "gopher:example.com/",
    "scheme-wss-no-slash": "wss:example.com/",
    "scheme-mailto-no-slash": "mailto:example.com/",
    "scheme-data-no-slash": "data:text/plain,foo",
    "userinfo": "http://user:pass@foo:21/bar;par?b#c",
    "host-ipv6": "http://[2001::1]",
    "host-ipv6-port": "http://[2001::1]:80",
    "port-none-but-colon": "http://f:/c",
    "port-0": "http://f:0/c",
    "port-00000000000000": "http://f:00000000000000/c",
    "port-00000000000000000000080": "http://f:00000000000000000000080/c",
    "port-00000000000000000000080": "http://f:00000000000000000000080/c",
    "userinfo-host-port-path": "http://a:b@c:29/d",
    "userinfo-username-non-alpha": "http://foo.com:b@d/",
    "query-contains-question-mark": "http://foo/abcd?efgh?ijkl",
    "fragment-contains-question-mark": "http://foo/abcd#foo?bar",
    "path-percent-encoded-dot": "http://example.com/foo/%2e",
    "path-percent-encoded-space": "http://example.com/%20foo",
    "path-non-ascii": u"http://example.com/\u00C2\u00A9zbar".encode('utf-8'),
    "path-percent-encoded-multiple": "http://example.com/foo%41%7a",
    "path-percent-encoded-u0091": "http://example.com/foo%91",
    "path-percent-encoded-u0000": "http://example.com/foo%00",
    "path-percent-encoded-mixed-case": "http://example.com/%3A%3a%3C%3c",
    "path-unicode-han": u"http://example.com/\u4F60\u597D\u4F60\u597D".encode('utf-8'),
    "path-uFEFF": u"http://example.com/\uFEFF/foo".encode('utf-8'),
    "path-u202E-u202D": u"http://example.com/\u202E/foo/\u202D/bar".encode('utf-8'),
    "host-is-pile-of-poo": "http://💩",
    "path-contains-pile-of-poo": "http://example.com/foo/💩",
    "query-contains-pile-of-poo": "http://example.com/foo?💩",
    "fragment-contains-pile-of-poo": "http://example.com/foo#💩",
    "host-192.0x00A80001": "http://192.0x00A80001",
    "userinfo-username-contains-percent-encoded": "http://%25DOMAIN:foobar@foodomain.com",
    "userinfo-empty": "http://@www.example.com",
    "userinfo-user-empty": "http://:b@www.example.com",
    "userinfo-password-empty": "http://a:@www.example.com",
    "host-exotic-whitespace": u"http://GOO\u200b\u2060\ufeffgoo.com".encode('utf-8'),
    "host-exotic-dot": u"http://www.foo\u3002bar.com".encode('utf-8'),
    "host-fullwidth": u"http://\uff27\uff4f.com".encode('utf-8'),
    "host-idn-unicode-han": u"http://\u4f60\u597d\u4f60\u597d".encode('utf-8'),
    "host-IP-address-broken": "http://192.168.0.257/",
}
valid = valid_absolute.copy()

valid_relative = {
    "scheme-schemeless-relative": "//foo/bar",
    "path-slash-only-relative": "/",
    "path-simple-relative": "/a/b/c",
    "path-percent-encoded-slash-relative": "/a%2fc",
    "path-percent-encoded-slash-plus-slashes-relative": "/a/%2f/c",
    "query-empty-no-path-relative": "?",
    "fragment-empty-hash-only-no-path-relative": "#",
    "fragment-slash-relative": "#/",
    "fragment-semicolon-question-mark-relative": "#;?",
    "fragment-non-ascii-relative": u"#\u03B2".encode('utf-8'),
}
valid.update(valid_relative)
invalid_absolute.update(valid_relative)

valid_relative_colon_dot = {
    "scheme-none-relative": "foo.com",
    "path-colon-relative": ":",
    "path-leading-colon-letter-relative": ":a",
    "path-leading-colon-chars-relative": ":foo.com",
    "path-leading-colon-slash-relative": ":/",
    "path-leading-colon-hash-relative": ":#",
    "path-leading-colon-number-relative": ":23",
    "path-slash-colon-number-relative": "/:23",
    "path-leading-colon-colon-relative": "::",
    "path-colon-colon-number-relative": "::23",
    "path-starts-with-pile-of-poo": "💩http://foo",
    "path-contains-pile-of-poo": "http💩//:foo",
}
valid.update(valid_relative_colon_dot)

invalid_file = {
    "scheme-file-backslash": "file:c:\\foo\\bar.html",
    "scheme-file-single-slash-c-bar": "file:/C|/foo/bar",
    "scheme-file-triple-slash-c-bar": "file:///C|/foo/bar",
}
invalid.update(invalid_file)

valid_file = {
    "scheme-file-uppercase": "File://foo/bar.html",
    "scheme-file-slash-slash-c-bar": "file://C|/foo/bar",
    "scheme-file-slash-slash-abc-bar": "file://abc|/foo/bar",
    "scheme-file-host-included": "file://server/foo/bar",
    "scheme-file-host-empty": "file:///foo/bar.txt",
    "scheme-file-scheme-only": "file:",
    "scheme-file-slash-only": "file:/",
    "scheme-file-slash-slash-only": "file://",
    "scheme-file-slash-slash-slash-only": "file:///",
    "scheme-file-no-slash": "file:test",
}
valid.update(valid_file)
valid_absolute.update(valid_file)

warnings = {
    "scheme-data-contains-fragment": "data:text/html,test#test",
}

element_attribute_pairs = [
    "a href",
    # "a ping", space-separated list of URLs; tested elsewhere
    "area href",
    # "area ping", space-separated list of URLs; tested elsewhere
    "audio src",
    "base href",
    "blockquote cite",
    "button formaction",
    "del cite",
    "embed src",
    "form action",
    "html manifest",
    "iframe src",
    "img src", # srcset is tested elsewhere
    "input formaction", # type=submit, type=image
    "input src", # type=image
    "input value", # type=url
    "ins cite",
    "link href",
    #"menuitem icon", # skip until parser is updated
    "object data",
    "q cite",
    "script src",
    "source src",
    "track src",
    "video poster",
    "video src",
]

template = "<!DOCTYPE html>\n<meta charset=utf-8>\n"

def write_novalid_files():
    for el, attr in (pair.split() for pair in element_attribute_pairs):
        for desc, url in invalid.items():
            if ("area" == el):
                f = open(os.path.join(ccdir, "html/elements/area/href/%s-novalid.html" % desc), 'wb')
                f.write(template + '<title>invalid href: %s</title>\n' % desc)
                f.write('<map name=foo><%s %s="%s" alt></map>\n' % (el, attr, url))
                f.close()
            elif ("base" == el or "embed" == el):
                f = open(os.path.join(ccdir, "html/elements/%s/%s/%s-novalid.html" % (el, attr, desc)), 'wb')
                f.write(template + '<title>invalid %s: %s</title>\n' % (attr, desc))
                f.write('<%s %s="%s">\n' % (el, attr, url))
                f.close()
            elif ("html" == el):
                f = open(os.path.join(ccdir, "html/elements/html/manifest/%s-novalid.html" % desc), 'wb')
                f.write('<!DOCTYPE html>\n')
                f.write('<html manifest="%s">\n' % url)
                f.write('<meta charset=utf-8>\n')
                f.write('<title>invalid manifest: %s</title>\n' %  desc)
                f.write('</html>\n')
                f.close()
            elif ("img" == el):
                f = open(os.path.join(ccdir, "html/elements/img/src/%s-novalid.html" % desc), 'wb')
                f.write(template + '<title>invalid src: %s</title>\n' %  desc)
                f.write('<img src="%s" alt>\n' % url)
                f.close()
            elif ("input" == el and "src" == attr):
                f = open(os.path.join(ccdir, "html/elements/input/type-image-src/%s-novalid.html" % desc), 'wb')
                f.write(template + '<title>invalid src: %s</title>\n' % desc)
                f.write('<%s type=image alt="foo" %s="%s">\n' % (el, attr, url))
                f.close()
            elif ("input" == el and "formaction" == attr):
                f = open(os.path.join(ccdir, "html/elements/input/type-submit-formaction/%s-novalid.html" % desc), 'wb')
                f.write(template + '<title>invalid formaction: %s</title>\n' % desc)
                f.write('<%s type=submit %s="%s">\n' % (el, attr, url))
                f.close()
                f = open(os.path.join(ccdir, "html/elements/input/type-image-formaction/%s-novalid.html" % desc), 'wb')
                f.write(template + '<title>invalid formaction: %s</title>\n' % desc)
                f.write('<%s type=image alt="foo" %s="%s">\n' % (el, attr, url))
                f.close()
            elif ("input" == el and "value" == attr):
                f = open(os.path.join(ccdir, "html/elements/input/type-url-value/%s-novalid.html" % desc), 'wb')
                f.write(template + '<title>invalid value attribute: %s</title>\n' % desc)
                f.write('<%s type=url %s="%s">\n' % (el, attr, url))
                f.close()
            elif ("link" == el):
                f = open(os.path.join(ccdir, "html/elements/link/href/%s-novalid.html" % desc), 'wb')
                f.write(template + '<title>invalid href: %s</title>\n' %  desc)
                f.write('<link href="%s" rel=help>\n' % url)
                f.close()
            elif ("source" == el or "track" == el):
                f = open(os.path.join(ccdir, "html/elements/%s/%s/%s-novalid.html" % (el, attr, desc)), 'wb')
                f.write(template + '<title>invalid %s: %s</title>\n' % (attr, desc))
                f.write('<video><%s %s="%s"></video>\n' % (el, attr, url))
                f.close()
            else:
                f = open(os.path.join(ccdir, "html/elements/%s/%s/%s-novalid.html" % (el, attr, desc)), 'wb')
                f.write(template + '<title>invalid %s: %s</title>\n' % (attr, desc))
                f.write('<%s %s="%s"></%s>\n' % (el, attr, url, el))
                f.close()
    for desc, url in invalid.items():
        f = open(os.path.join(ccdir, "html/microdata/itemid/%s-novalid.html" % desc), 'wb')
        f.write(template + '<title>invalid itemid: %s</title>\n' % desc)
        f.write('<div itemid="%s" itemtype="http://foo" itemscope></div>\n' % url)
        f.close()
    for desc, url in invalid_absolute.items():
        f = open(os.path.join(ccdir, "html/microdata/itemtype/%s-novalid.html" % desc), 'wb')
        f.write(template + '<title>invalid itemtype: %s</title>\n' % desc)
        f.write('<div itemtype="%s" itemscope></div>\n' % url)
        f.close()
        f = open(os.path.join(ccdir, "html/elements/input/type-url-value/%s-novalid.html" % desc), 'wb')
        f.write(template + '<title>invalid value attribute: %s</title>\n' %desc)
        f.write('<input type=url value="%s">\n' % url)
        f.close()

def write_haswarn_files():
    for el, attr in (pair.split() for pair in element_attribute_pairs):
        for desc, url in warnings.items():
            if ("area" == el):
                f = open(os.path.join(ccdir, "html/elements/area/href/%s-haswarn.html" % desc), 'wb')
                f.write(template + '<title>%s warning: %s</title>\n' % (attr, desc))
                f.write('<map name=foo><%s %s="%s" alt></map>\n' % (el, attr, url))
                f.close()
            elif ("base" == el or "embed" == el):
                f = open(os.path.join(ccdir, "html/elements/%s/%s/%s-haswarn.html" % (el, attr, desc)), 'wb')
                f.write(template + '<title>%s warning: %s</title>\n' % (attr, desc))
                f.write('<%s %s="%s">\n' % (el, attr, url))
                f.close()
            elif ("html" == el):
                f = open(os.path.join(ccdir, "html/elements/html/manifest/%s-haswarn.html" % desc), 'wb')
                f.write('<!DOCTYPE html>\n')
                f.write('<html manifest="%s">\n' % url)
                f.write('<meta charset=utf-8>\n')
                f.write('<title>%s warning: %s</title>\n' % (attr, desc))
                f.write('</html>\n')
                f.close()
            elif ("img" == el):
                f = open(os.path.join(ccdir, "html/elements/img/src/%s-haswarn.html" % desc), 'wb')
                f.write(template + '<title>%s warning: %s</title>\n' % (attr, desc))
                f.write('<%s %s="%s" alt>\n' % (el, attr, url))
                f.close()
            elif ("input" == el and "src" == attr):
                f = open(os.path.join(ccdir, "html/elements/input/type-image-src/%s-haswarn.html" % desc), 'wb')
                f.write(template + '<title>%s warning: %s</title>\n' % (attr, desc))
                f.write('<%s type=image alt="foo" %s="%s">\n' % (el, attr, url))
                f.close()
            elif ("input" == el and "formaction" == attr):
                f = open(os.path.join(ccdir, "html/elements/input/type-submit-formaction/%s-haswarn.html" % desc), 'wb')
                f.write(template + '<title>%s warning: %s</title>\n' % (attr, desc))
                f.write('<%s type=submit %s="%s">\n' % (el, attr, url))
                f.close()
                f = open(os.path.join(ccdir, "html/elements/input/type-image-formaction/%s-haswarn.html" % desc), 'wb')
                f.write(template + '<title>%s warning: %s</title>\n' % (attr, desc))
                f.write('<%s type=image alt="foo" %s="%s">\n' % (el, attr, url))
                f.close()
            elif ("input" == el and "value" == attr):
                f = open(os.path.join(ccdir, "html/elements/input/type-url-value/%s-haswarn.html" % desc), 'wb')
                f.write(template + '<title>%s warning: %s</title>\n' % (attr, desc))
                f.write('<%s type=url %s="%s">\n' % (el, attr, url))
                f.close()
            elif ("link" == el):
                f = open(os.path.join(ccdir, "html/elements/link/href/%s-haswarn.html" % desc), 'wb')
                f.write(template + '<title>%s warning: %s</title>\n' % (attr, desc))
                f.write('<%s %s="%s" rel=help>\n' % (el, attr, url))
                f.close()
            elif ("source" == el or "track" == el):
                f = open(os.path.join(ccdir, "html/elements/%s/%s/%s-haswarn.html" % (el, attr, desc)), 'wb')
                f.write(template + '<title>%s warning: %s</title>\n' % (attr, desc))
                f.write('<video><%s %s="%s"></video>\n' % (el, attr, url))
                f.close()
            else:
                f = open(os.path.join(ccdir, "html/elements/%s/%s/%s-haswarn.html" % (el, attr, desc)), 'wb')
                f.write(template + '<title>%s warning: %s</title>\n' % (url, desc))
                f.write('<%s %s="%s"></%s>\n' % (el, attr, url, el))
                f.close()
    for desc, url in warnings.items():
        f = open(os.path.join(ccdir, "html/microdata/itemtype-%s-haswarn.html" % desc ), 'wb')
        f.write(template + '<title>warning: %s</title>\n' % desc)
        f.write('<div itemtype="%s" itemscope></div>\n' % url)
        f.close()
        f = open(os.path.join(ccdir, "html/microdata/itemid-%s-haswarn.html" % desc), 'wb')
        f.write(template + '<title>warning: %s</title>\n' % desc)
        f.write('<div itemid="%s" itemtype="http://foo" itemscope></div>\n' % url)
        f.close()

def write_isvalid_files():
    for el, attr in (pair.split() for pair in element_attribute_pairs):
        if ("base" == el):
            continue
        if ("html" == el):
            continue
        elif ("input" == el and "value" == attr):
            continue
        elif ("input" == el and "formaction" == attr):
            fs = open(os.path.join(ccdir, "html/elements/input/type-submit-formaction-isvalid.html"), 'wb')
            fs.write(template + '<title>valid formaction</title>\n')
            fi = open(os.path.join(ccdir, "html/elements/input/type-image-formaction-isvalid.html"), 'wb')
            fi.write(template + '<title>valid formaction</title>\n')
        elif ("input" == el and "src" == attr):
            f = open(os.path.join(ccdir, "html/elements/input/type-image-src-isvalid.html"), 'wb')
            f.write(template + '<title>valid src</title>\n')
        else:
            f = open(os.path.join(ccdir, "html/elements/%s/%s-isvalid.html" % (el, attr)), 'wb')
            f.write(template + '<title>valid %s</title>\n' % attr)
        for desc, url in valid.items():
            if ("area" == el):
                f.write('<map name=foo><%s %s="%s" alt></map><!-- %s -->\n' % (el, attr, url, desc))
            elif ("embed" == el):
                f.write('<%s %s="%s"><!-- %s -->\n' % (el, attr, url, desc))
            elif ("img" == el):
                f.write('<%s %s="%s" alt><!-- %s -->\n' % (el, attr, url, desc))
            elif ("input" == el and "src" == attr):
                f.write('<%s type=image alt="foo" %s="%s"><!-- %s -->\n' % (el, attr, url, desc))
            elif ("input" == el and "formaction" == attr):
                fs.write('<%s type=submit %s="%s"><!-- %s -->\n' % (el, attr, url, desc))
                fi.write('<%s type=image alt="foo" %s="%s"><!-- %s -->\n' % (el, attr, url, desc))
            elif ("link" == el):
                f.write('<%s %s="%s" rel=help><!-- %s -->\n' % (el, attr, url, desc))
            elif ("source" == el or "track" == el):
                f.write('<video><%s %s="%s"></video><!-- %s -->\n' % (el, attr, url, desc))
            else:
                f.write('<%s %s="%s"></%s><!-- %s -->\n' % (el, attr, url, el, desc))
        if ("input" == el and "formaction" == attr):
            fs.close()
            fi.close()
        else:
            if ("a" == el and "href" == attr):
                f.write('<a href=""></a><!-- empty-href -->\n')
            f.close()
    for desc, url in valid.items():
        f = open(os.path.join(ccdir, "html/elements/base/href/%s-isvalid.html" % desc), 'wb')
        f.write(template + '<title>valid href: %s</title>\n' % desc)
        f.write('<base href="%s">\n' % url)
        f.close()
        f = open(os.path.join(ccdir, "html/elements/html/manifest/%s-isvalid.html" % desc), 'wb')
        f.write('<!DOCTYPE html>\n')
        f.write('<html manifest="%s">\n' % url)
        f.write('<meta charset=utf-8>\n')
        f.write('<title>valid manifest: %s</title>\n' % desc)
        f.write('</html>\n')
        f.close()
    f = open(os.path.join(ccdir, "html/elements/meta/refresh-isvalid.html"), 'wb')
    f.write(template + '<title>valid meta refresh</title>\n')
    for desc, url in valid.items():
        f.write('<meta http-equiv=refresh content="0; URL=%s"><!-- %s -->\n' % (url, desc))
    f.close()
    f = open(os.path.join(ccdir, "html/microdata/itemid-isvalid.html"), 'wb')
    f.write(template + '<title>valid itemid</title>\n')
    for desc, url in valid.items():
        f.write('<div itemid="%s" itemtype="http://foo" itemscope></div><!-- %s -->\n' % (url, desc))
    f.close()
    f = open(os.path.join(ccdir, "html/microdata/itemtype-isvalid.html"), 'wb')
    f.write(template + '<title>valid itemtype</title>\n')
    for desc, url in valid_absolute.items():
        f.write('<div itemtype="%s" itemscope></div><!-- %s -->\n' % (url, desc))
    f.close()
    f = open(os.path.join(ccdir, "html/elements/input/type-url-value-isvalid.html"), 'wb')
    f.write(template + '<title>valid value attribute</title>\n')
    for desc, url in valid_absolute.items():
        f.write('<input type=url value="%s"><!-- %s -->\n' % (url, desc))
    f.close()

write_novalid_files()
write_haswarn_files()
write_isvalid_files()
# vim: ts=4:sw=4
