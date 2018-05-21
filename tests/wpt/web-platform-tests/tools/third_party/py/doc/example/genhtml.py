from py.xml import html

paras = "First Para", "Second para"

doc = html.html(
   html.head(
        html.meta(name="Content-Type", value="text/html; charset=latin1")),
   html.body(
        [html.p(p) for p in paras]))

print unicode(doc).encode('latin1')


