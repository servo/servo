#!/usr/bin/python
# CSS Test Source Manipulation Library
# Initial code by fantasai, joint copyright 2010 W3C and Microsoft
# additions by peter.linss@hp.com copyright 2013 Hewlett-Packard
# Licensed under BSD 3-Clause: <http://www.w3.org/Consortium/Legal/2008/03-bsd-license>

import lxml
from lxml import etree
import htmlentitydefs
import copy


class HTMLSerializer(object):

    gXMLns = 'http://www.w3.org/XML/1998/namespace'
    gHTMLns = 'http://www.w3.org/1999/xhtml'
  
    gDefaultNamespaces = {'http://www.w3.org/XML/1998/namespace': 'xmlns',
                          'http://www.w3.org/2000/xmlns/': 'xmlns',
                          'http://www.w3.org/1999/xlink': 'xlink'}

    gVoidElements = frozenset((
        'base',
        'command',
        'event-source',
        'link',
        'meta',
        'hr',
        'br',
        'img',
        'embed',
        'param',
        'area',
        'col',
        'input',
        'source'
    ))

    gCDataElements = frozenset((
        'style',
        'script'
    ))
  
    gInvisibleChars = frozenset(
        # ASCII control chars
        range(0x0, 0x9) + range(0xB, 0xD) + range(0xE, 0x20) +
        # Other control chars
        # fixed-width spaces, zero-width marks, bidi marks
        range(0x2000, 0x2010) +
        # LS, PS, bidi control codes
        range(0x2028, 0x2030) +
        # nbsp, mathsp, ideosp, WJ, interlinear
        [0x00A0, 0x205F, 0x3000, 0x2060, 0xFFF9, 0xFFFA, 0xFFFB]
    )

    gXMLEscapes = frozenset(gInvisibleChars |
                            frozenset((ord('&'), ord('<'), ord('>'))))

    gXMLEntityNames = {'"': 'quot', '&': 'amp', "'": 'apos', '<': 'lt', '>': 'gt'}

    gDocTypes = {
        'html': '<!DOCTYPE html>',
        'html4':
            '<!DOCTYPE html PUBLIC "-//W3C//DTD HTML 4.01//EN" "http://www.w3.org/TR/html4/strict.dtd">',
        'html4-transitional':
            '<!DOCTYPE html PUBLIC "-//W3C//DTD HTML 4.01 Transitional//EN" "http://www.w3.org/TR/html4/loose.dtd">',
        'html4-frameset':
            '<!DOCTYPE html PUBLIC "-//W3C//DTD HTML 4.01 Frameset//EN" "http://www.w3.org/TR/html4/frameset.dtd">',
        'svg11':
            '<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1 Basic//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11-basic.dtd">',
        'svg11-tiny':
            '<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1 Tiny//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11-tiny.dtd">',
        'xhtml10':
            '<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Strict//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd">',
        'xhtml10-transitional':
            '<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Transitional//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd">',
        'xhtml10-frameset':
            '<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Frameset//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-frameset.dtd">',
        'xhtml11':
            '<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.1//EN" "http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd">',
        'xhtml-basic11':
            '<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML Basic 1.1//EN" "http://www.w3.org/TR/xhtml-basic/xhtml-basic11.dtd">'
    }
  

    def __init__(self):
        self._reset()
  
    def _reset(self, xhtml = False):
        self.mOutput = u''
        self.mXHTML = xhtml

    def _output(self, *args):
        for arg in args:
            self.mOutput += unicode(arg)

    def _escape(self, text, escapeChars):
        # This algorithm is O(MN) for M len(text) and N num escapable
        # But it doesn't modify the text when N is zero (common case) and
        # N is expected to be small (usually 1 or 2) in most other cases.
        escapable = set()
        for char in text:
            if ord(char) in escapeChars:
                escapable.add(char)
        for char in escapable:
            if (self.mXHTML):
                name = self.gXMLEntityNames.get(char)
            else:
                name = htmlentitydefs.codepoint2name.get(ord(char))
            escape = u'&%s;' % name if name else u'&#x%X;' % ord(char)
            text = text.replace(char, escape)
        return text

    def _escapeXML(self, text):
        return self._escape(text, self.gXMLEscapes)

    def _escapeInvisible(self, text):
        return self._escape(text, self.gInvisibleChars)

    def _serializeElement(self, element, namespacePrefixes):
        qName = etree.QName(element)
        attrs = element.attrib.items()  # in tree order
      
        if (not namespacePrefixes):
            namespacePrefixes = self.gDefaultNamespaces
      
        if (self.mXHTML):
            namespacePrefixes = copy.copy(namespacePrefixes)
            for attr, value in attrs:
                attrQName = etree.QName(attr)
                if (self.gXMLns == attrQName.namespace):
                    namespacePrefixes[value] = attrQName.localname
                elif ('xmlns' == attrQName.localname):
                    namespacePrefixes[value] = ''

        if (self.mXHTML and qName.namespace and namespacePrefixes[qName.namespace]):
            self._output('<', namespacePrefixes[qName.namespace], ':', qName.localname)
        else:
            self._output('<', qName.localname)

        for attr, value in attrs:
            attrQName = etree.QName(attr)
            if ((attrQName.namespace == self.gXMLns) and ('lang' == attrQName.localname)):
                if (self.mXHTML):
                    attr = 'xml:lang'
                else:
                    attr = 'lang'
            elif (attrQName.namespace and namespacePrefixes[attrQName.namespace]):
                attr = namespacePrefixes[attrQName.namespace] + ':' + attrQName.localname
            else:
                attr = attrQName.localname

            self._output(' ', attr, '=')
            value = value.replace('&', '&amp;')
            if (self.mXHTML):
                value = value.replace('<', '&lt;')

            if (('"' in value) and ("'" not in value)):
                self._output("'", self._escapeInvisible(value), "'")
            else:
                self._output('"', self._escapeInvisible(value.replace('"', '&quot;')), '"')

        if ((qName.namespace == self.gHTMLns) and (qName.localname in self.gVoidElements)):
            if (self.mXHTML):
                self._output(' />')
            else:
                self._output('>')
        else:
            self._output('>')

            if (None != element.text):
                if ((qName.namespace == self.gHTMLns) and (qName.localname in self.gCDataElements)):
                    if (self.mXHTML):
                        self._output(self._escapeXML(element.text)) # or self._output('<![CDATA[', element.text, ']]>')
                    else:
                        self._output(element.text)
                else:
                    self._output(self._escapeXML(element.text))

            for child in list(element):
                self._serializeNode(child, namespacePrefixes)

            self._output('</', qName.localname, '>')

        if (None != element.tail):
            self._output(self._escapeXML(element.tail))

    def _serializeEntity(self, entity):
        self._output(entity.text)
        if (None != entity.tail):
            self._output(self._escapeXML(entity.tail))
        
    def _serializePI(self, pi):
        if (self.mXHTML):
            self._output('<?', pi.target, ' ', pi.text, '?>')
        else:
            raise Exception("Processing Instructions can't be converted to HTML")
        if (None != pi.tail):
            self._output(self._escapeXML(pi.tail))
        
    def _serializeComment(self, comment):
        self._output('<!--', comment.text, '-->') # XXX escape comment?
        if (None != comment.tail):
            self._output(self._escapeXML(comment.tail))
        
    def _serializeNode(self, node, namespacePrefixes = None):
        if (isinstance(node, etree._Entity)):
            self._serializeEntity(node)
        elif (isinstance(node, etree._ProcessingInstruction)):
            self._serializePI(node)
        elif (isinstance(node, etree._Comment)):
            self._serializeComment(node)
        else:
            self._serializeElement(node, namespacePrefixes)


    def _serializeTree(self, tree):
        root = tree.getroot()
        preceding = [node for node in root.itersiblings(preceding = True)]
        preceding.reverse()
        for node in preceding:
            self._serializeNode(node)
        self._serializeNode(root)
        for node in root.itersiblings():
            self._serializeNode(node)
  
    def _serializeDoctype(self, tree, doctype, default):
        if (doctype):
            self._output(self.gDocTypes[doctype], '\n')
        else:
            if (hasattr(tree, 'docinfo') and tree.docinfo and tree.docinfo.doctype):
                doctypeSearch = tree.docinfo.doctype.lower()
                for doctype in self.gDocTypes:
                    if (self.gDocTypes[doctype].lower() == doctypeSearch):
                        break
                else:
                    doctype = None
                if (self.mXHTML):
                    if ('html' == doctype):
                        doctype = 'xhtml10'
                    elif ('html4' == doctype):
                        doctype = 'xhtml10'
                    elif ('html4-transitional' == doctype):
                        doctype = 'xhtml10-transitional'
                    elif ('html4-frameset' == doctype):
                        doctype = 'xhtml10-frameset'
                else:
                    if ('xhtml10' == doctype):
                        doctype = 'html4'
                    elif ('xhtml10-transitional' == doctype):
                        doctype = 'html4-transitional'
                    elif ('xhtml10-frameset' == doctype):
                        doctype = 'html4-frameset'
                    elif ('xhtml11' == doctype):
                        doctype = 'html4'
                if (doctype):
                    self._output(self.gDocTypes[doctype], '\n')
                else:
                    self._output(tree.docinfo.doctype, '\n')
            else:
                self._output(self.gDocTypes[default], '\n')


    def serializeHTML(self, tree, doctype = None):
        self._reset()
        self._serializeDoctype(tree, doctype, 'html')
        self._serializeTree(tree)
        return self.mOutput

    def serializeXHTML(self, tree, doctype = None):
        self._reset(True)
        # XXX '<!xml ...' ??
        self._serializeDoctype(tree, doctype, 'xhtml11')
        self._serializeTree(tree)
        return self.mOutput


