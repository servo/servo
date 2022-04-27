import py
html = py.xml.html

class my(html):
    "a custom style"
    class body(html.body):
        style = html.Style(font_size = "120%")

    class h2(html.h2):
        style = html.Style(background = "grey")

    class p(html.p):
        style = html.Style(font_weight="bold")

doc = my.html(
    my.head(),
    my.body(
        my.h2("hello world"),
        my.p("bold as bold can")
    )
)

print(doc.unicode(indent=2))
