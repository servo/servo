import html5lib
import html5lib.treebuilders.dom
import re

# Expected use:
#   curl --compressed https://html.spec.whatwg.org/multipage/canvas.html >current-work
#   python specextract.py
#
# Generates current-work-canvas.xhtml, for use by gentest.py to create the annotated spec document

def extract():
    parser = html5lib.html5parser.HTMLParser(tree=html5lib.getTreeBuilder("dom"))
    doc = parser.parse(open('current-work', "r"), transport_encoding='utf-8')

    head = doc.getElementsByTagName('head')[0]
    for n in head.childNodes:
        if n.tagName == 'script':
            head.removeChild(n)

    header = doc.getElementsByTagName('header')[0]
    #thecanvas = doc.getElementById('the-canvas') # doesn't work (?!)
    thecanvas = [ n for n in doc.getElementsByTagName('h4') if n.getAttribute('id') == 'the-canvas-element' ][0]

    # Add copyright from https://html.spec.whatwg.org/multipage/acknowledgements.html#acknowledgments
    copy = doc.createElement('p')
    copy.setAttribute('class', 'copyright')
    copy.appendChild(doc.createTextNode(u'Parts of this specification are \xA9 Copyright 2004-2014 Apple Inc., Mozilla Foundation, and Opera Software ASA. You are granted a license to use, reproduce and create derivative works of this document.'))
    header.appendChild(copy)

    keep = [header, thecanvas]
    node = thecanvas.nextSibling
    while node.nodeName != 'nav':
        keep.append(node)
        node = node.nextSibling
    p = thecanvas.parentNode
    for n in p.childNodes[:]:
        if n not in keep:
            p.removeChild(n)

    for n in header.childNodes[3:-4]:
        header.removeChild(n)

    def make_absolute(url):
        match = re.match(r'(\w+:|#)', url)
        if match:
            return url
        elif url[0] == '/':
            return 'https://html.spec.whatwg.org' + url
        else:
            return 'https://html.spec.whatwg.org/multipage/' + url

    # Fix relative URLs
    for e in doc.getElementsByTagName('script'):
        e.setAttribute('src', make_absolute(e.getAttribute('src')))
    for e in doc.getElementsByTagName('iframe'):
        e.setAttribute('src', make_absolute(e.getAttribute('src')))
    for e in doc.getElementsByTagName('img'):
        e.setAttribute('src', make_absolute(e.getAttribute('src')))
    for e in doc.getElementsByTagName('a'):
        e.setAttribute('href', make_absolute(e.getAttribute('href')))

    # Convert to XHTML, because it's quicker to re-parse than HTML5
    doc.documentElement.setAttribute('xmlns', 'http://www.w3.org/1999/xhtml')
    doc.removeChild(doc.firstChild) # remove the DOCTYPE

    open('current-work-canvas.xhtml', 'w').write(doc.toxml(encoding = 'UTF-8'))

extract()
