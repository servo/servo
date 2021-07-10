
import py
class ns(py.xml.Namespace):
    pass

doc = ns.books(
    ns.book(
        ns.author("May Day"),
        ns.title("python for java programmers"),),
    ns.book(
        ns.author("why", class_="somecssclass"),
        ns.title("Java for Python programmers"),),
    publisher="N.N",
    )
print doc.unicode(indent=2).encode('utf8')


