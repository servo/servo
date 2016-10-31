# -*- coding: utf-8 -*-
import os
ccdir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
template = """<!DOCTYPE html>
<meta charset=utf-8>
"""

errors = {
    "dl-in-p": "<p><dl><dt>text<dd>text</dl></p>",
    "header-in-dt": "<dl><dt><header>text</header><dd>text</dl>",
    "footer-in-dt": "<dl><dt><footer>text</footer><dd>text</dl>",
    "article-in-dt": "<dl><dt><article><h2>text</h2></article><dd>text</dl>",
    "aside-in-dt": "<dl><dt><aside><h2>text</h2></aside><dd>text</dl>",
    "nav-in-dt": "<dl><dt><nav><h2>text</h2></nav><dd>text</dl>",
    "section-in-dt": "<dl><dt><section><h2>text</h2></section><dd>text</dl>",
    "h1-in-dt": "<dl><dt><h1>text</h1><dd>text</dl>",
    "h2-in-dt": "<dl><dt><h2>text</h2><dd>text</dl>",
    "h3-in-dt": "<dl><dt><h3>text</h3><dd>text</dl>",
    "h4-in-dt": "<dl><dt><h4>text</h4><dd>text</dl>",
    "h5-in-dt": "<dl><dt><h5>text</h5><dd>text</dl>",
    "h6-in-dt": "<dl><dt><h6>text</h6><dd>text</dl>",
    "hgroup-in-dt": "<dl><dt><hgroup><h1>text</h1></hgroup><dd>text</dl>",
    "only-dt": "<dl><dt>1</dl>",
    "only-dd": "<dl><dd>a</dl>",
    "first-dd": "<dl><dd>a<dt>2<dd>b</dl>",
    "last-dt": "<dl><dt>1<dd>a<dt>2</dl>",
    "dd-in-template": "<dl><dt>1</dt><template><dd>a</dd></template></dl>",
    "dt-in-template": "<dl><template><dt>1</dt></template><dd>a</dl>",
    "dl-contains-text": "<dl><dt>1</dt>x</dl>",
    "dl-contains-text-2": "<dl><dt>1<dd>a</dd>x</dl>",
    "dl-contains-dl": "<dl><dt>1<dd>a</dd><dl></dl></dl>",
    # div
    "empty-div": "<dl><div></div></dl>",
    "empty-div-2": "<dl><div></div><div><dt>2<dd>b</div></dl>",
    "mixed-dt-dd-div": "<dl><dt>1<dd>a</dd><div><dt>2<dd>b</div></dl>",
    "mixed-div-dt-dd": "<dl><div><dt>1<dd>a</div><dt>2<dd>b</dd></dl>",
    "nested-divs": "<dl><div><div><dt>1<dd>a</div></div></dl>",
    "div-splitting-groups": "<dl><div><dt>1</div><div><dd>a</div></dl>",
    "div-splitting-groups-2": "<dl><div><dt>1<dd>a</div><div><dd>b</div></dl>",
    "div-splitting-groups-3": "<dl><div><dt>1</div><div><dt>2<dd>b</div></dl>",
    "div-contains-text": "<dl><div>x</div><dt>2<dd>b</div></dl>",
    "div-contains-dl": "<dl><div><dl></dl></div><dt>2<dd>b</div></dl>",
    "div-multiple-groups": "<dl><div><dt>1<dd>a<dt>2<dd>a<dd>b<dt>3<dt>4<dt>5<dd>a</div></dl>",
}

non_errors_in_head = {
    "parent-template-in-head": "<template><dl><dt>text<dd>text</dl></template>",
}

non_errors = {
    "basic": "<dl><dt>text<dd>text</dl>",
    "empty": "<dl></dl>",
    "empty-dt-dd": "<dl><dt><dd></dl>",
    "multiple-groups": "<dl><dt>1<dd>a<dt>2<dd>a<dd>b<dt>3<dt>4<dt>5<dd>a</dl>",
    "header-in-dd": "<dl><dt>text<dd><header>text</header></dl>",
    "footer-in-dd": "<dl><dt>text<dd><footer>text</footer></dl>",
    "article-in-dd": "<dl><dt>text<dd><article><h2>text</h2></article></dl>",
    "aside-in-dd": "<dl><dt>text<dd><aside><h2>text</h2></aside></dl>",
    "nav-in-dd": "<dl><dt>text<dd><nav><h2>text</h2></nav></dl>",
    "section-in-dd": "<dl><dt>text<dd><section><h2>text</h2></section></dl>",
    "h1-in-dd": "<dl><dt>text<dd><h1>text</h1></dl>",
    "h2-in-dd": "<dl><dt>text<dd><h2>text</h2></dl>",
    "h3-in-dd": "<dl><dt>text<dd><h3>text</h3></dl>",
    "h4-in-dd": "<dl><dt>text<dd><h4>text</h4></dl>",
    "h5-in-dd": "<dl><dt>text<dd><h5>text</h5></dl>",
    "h6-in-dd": "<dl><dt>text<dd><h6>text</h6></dl>",
    "p-in-dt": "<dl><dt><p>1<p>1<dd>a</dl>",
    "dl-in-dt": "<dl><dt><dl><dt>1<dd>a</dl><dd>b</dl>",
    "dl-in-dd": "<dl><dt>1<dd><dl><dt>2<dd>a</dl></dl>",
    "interactive": "<dl><dt><a href='#'>1</a><dd><a href='#'>a</a></dl>",
    "script": "<dl><script></script></dl>",
    "dt-script-dd": "<dl><dt>1</dt><script></script><dd>a</dl>",
    "dt-template-dd": "<dl><dt>1</dt><template></template><dd>a</dl>",
    # div
    "div-basic": "<dl><div><dt>1<dd>a</div></dl>",
    "div-script": "<dl><div><dt>1<dd>a</div><script></script></dl>",
    "div-script-2": "<dl><div><dt>1</dt><script></script><dd>a</div></dl>",
    "div-template": "<dl><div><dt>1<dd>a</div><template></template></dl>",
    "div-template-2": "<dl><div><dt>1</dt><template></template><dd>a</div></dl>",
    "div-multiple-groups": "<dl><div><dt>1<dd>a</div><div><dt>2<dd>a<dd>b</div><div><dt>3<dt>4<dt>5<dd>a</div></dl>",
}

for key in errors.keys():
    template_error = template
    template_error += '<title>invalid %s</title>\n' % key
    template_error += errors[key]
    file = open(os.path.join(ccdir, "html/elements/dl/%s-novalid.html" % key), 'wb')
    file.write(template_error)
    file.close()

file = open(os.path.join(ccdir, "html/elements/dl/dl-isvalid.html"), 'wb')
file.write(template + '<title>valid dl</title>\n')
for key in non_errors_in_head.keys():
    file.write('%s <!-- %s -->\n' % (non_errors_in_head[key], key))
file.write('<body>\n')
for key in non_errors.keys():
    file.write('%s <!-- %s -->\n' % (non_errors[key], key))
file.close()
# vim: ts=4:sw=4
