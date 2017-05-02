# coding=utf-8
#
#  Copyright © 2013 Hewlett-Packard Development Company, L.P.
#
#  This work is distributed under the W3C® Software License [1]
#  in the hope that it will be useful, but WITHOUT ANY
#  WARRANTY; without even the implied warranty of
#  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
#
#  [1] http://www.w3.org/Consortium/Legal/2002/copyright-software-20021231
#

# Process URI templates per http://tools.ietf.org/html/rfc6570

import re


class UnsupportedExpression(Exception):
    def __init__(self, expression):
        self.expression = expression

    def __unicode__(self):
        return u'Unsopported expression: ' + self.expression

class BadExpression(Exception):
    def __init__(self, expression):
        self.expression = expression

    def __unicode__(self):
        return u'Bad expression: ' + self.expression

class BadVariable(Exception):
    def __init__(self, variable):
        self.variable = variable

    def __unicode__(self):
        return u'Bad variable: ' + self.variable

class BadExpansion(Exception):
    def __init__(self, variable):
        self.variable = variable

    def __unicode__(self):
        return u'Bad expansion: ' + self.variable

class URITemplate(object):
    alpha = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ'
    digit = '0123456789'
    hexdigit = '0123456789ABCDEFabcdef'
    genDelims = ':/?#[]@'
    subDelims = "!$&'()*+,;="
    varstart = alpha + digit + '_'
    varchar = varstart + '.'
    unreserved = alpha + digit + '-._~'
    reserved = genDelims + subDelims

    def __init__(self, template):
        self.template = template

        self.parts = []
        parts = re.split(r'(\{[^\}]*\})', self.template)
        for part in parts:
            if (part):
                if (('{' == part[0]) and ('}' == part[-1])):
                    expression = part[1:-1]
                    if (re.match('^([a-zA-Z0-9_]|%[0-9a-fA-F][0-9a-fA-F]).*$', expression)):
                        self.parts.append(SimpleExpansion(expression))
                    elif ('+' == part[1]):
                        self.parts.append(ReservedExpansion(expression))
                    elif ('#' == part[1]):
                        self.parts.append(FragmentExpansion(expression))
                    elif ('.' == part[1]):
                        self.parts.append(LabelExpansion(expression))
                    elif ('/' == part[1]):
                        self.parts.append(PathExpansion(expression))
                    elif (';' == part[1]):
                        self.parts.append(PathStyleExpansion(expression))
                    elif ('?' == part[1]):
                        self.parts.append(FormStyleQueryExpansion(expression))
                    elif ('&' == part[1]):
                        self.parts.append(FormStyleQueryContinuation(expression))
                    elif (part[1] in '=,!@|'):
                        raise UnsupportedExpression(part)
                    else:
                        raise BadExpression(part)
                else:
                    if (('{' not in part) and ('}' not in part)):
                        self.parts.append(Literal(part))
                    else:
                        raise BadExpression(part)

    @property
    def variables(self):
        vars = set()
        for part in self.parts:
            vars.update(part.variables)
        return vars
    
    def expand(self, **kwargs):
        try:
            expanded = [part.expand(kwargs) for part in self.parts]
        except (BadExpansion):
            return None
        return ''.join([expandedPart for expandedPart in expanded if (expandedPart is not None)])

    def __str__(self):
        return self.template.encode('ascii', 'replace')

    def __unicode__(self):
        return unicode(self.template)


class Variable(object):
    def __init__(self, name):
        self.name = ''
        self.maxLength = None
        self.explode = False
        self.array = False
        
        if (name[0:1] not in URITemplate.varstart):
            raise BadVariable(name)
        
        if (':' in name):
            name, maxLength = name.split(':', 1)
            if ((0 < len(maxLength)) and (len(maxLength) < 4)):
                for digit in maxLength:
                    if (digit not in URITemplate.digit):
                        raise BadVariable(name + ':' + maxLength)
                self.maxLength = int(maxLength)
                if (not self.maxLength):
                    raise BadVariable(name + ':' + maxLength)
            else:
                raise BadVariable(name + ':' + maxLength)
        elif ('*' == name[-1]):
            name = name[:-1]
            self.explode = True
        elif ('[]' == name[-2:]):
            name = name[:-2]
            self.array = True
            self.explode = True
        
        index = 0
        while (index < len(name)):
            codepoint = name[index]
            if (('%' == codepoint) and
                ((index + 2) < len(name)) and
                (name[index + 1] in URITemplate.hexdigit) and
                (name[index + 2] in URITemplate.hexdigit)):
                self.name += name[index:index + 3]
                index += 2
            elif (codepoint in URITemplate.varchar):
                self.name += codepoint
            else:
                raise BadVariable(name + ((':' + self.maxLength) if (self.maxLength) else '') + ('[]' if (self.array) else ('*' if (self.explode) else '')))
            index += 1


class Expression(object):
    def __init__(self):
        pass
    
    @property
    def variables(self):
        return []

    def _encode(self, value, legal, pctEncoded):
        output = ''
        index = 0
        while (index < len(value)):
            codepoint = value[index]
            if (codepoint in legal):
                output += codepoint
            elif (pctEncoded and ('%' == codepoint) and
                  ((index + 2) < len(value)) and
                  (value[index + 1] in URITemplate.hexdigit) and
                  (value[index + 2] in URITemplate.hexdigit)):
                output += value[index:index + 3]
                index += 2
            else:
                utf8 = codepoint.encode('utf8')
                for byte in utf8:
                    output += '%' + URITemplate.hexdigit[ord(byte) / 16] + URITemplate.hexdigit[ord(byte) % 16]
            index += 1
        return output

    def _uriEncodeValue(self, value):
        return self._encode(value, URITemplate.unreserved, False)

    def _uriEncodeName(self, name):
        return self._encode(unicode(name), URITemplate.unreserved + URITemplate.reserved, True) if (name) else ''
    
    def _join(self, prefix, joiner, value):
        if (prefix):
            return prefix + joiner + value
        return value
    
    def _encodeStr(self, variable, name, value, prefix, joiner, first):
        if (variable.maxLength):
            if (not first):
                raise BadExpansion(variable)
            return self._join(prefix, joiner, self._uriEncodeValue(value[:variable.maxLength]))
        return self._join(prefix, joiner, self._uriEncodeValue(value))
    
    def _encodeDictItem(self, variable, name, key, item, delim, prefix, joiner, first):
        joiner = '=' if (variable.explode) else ','
        if (variable.array):
            prefix = (prefix + '[' + self._uriEncodeName(key) + ']') if (prefix and not first) else self._uriEncodeName(key)
        else:
            prefix = self._join(prefix, '.', self._uriEncodeName(key))
        return self._encodeVar(variable, key, item, delim, prefix, joiner, False)

    def _encodeListItem(self, variable, name, index, item, delim, prefix, joiner, first):
        if (variable.array):
            prefix = prefix + '[' + unicode(index) + ']' if (prefix) else ''
            return self._encodeVar(variable, None, item, delim, prefix, joiner, False)
        return self._encodeVar(variable, name, item, delim, prefix, '.', False)
    
    def _encodeVar(self, variable, name, value, delim = ',', prefix = '', joiner = '=', first = True):
        if (isinstance(value, basestring)):
            return self._encodeStr(variable, name, value, prefix, joiner, first)
        elif (hasattr(value, 'keys') and hasattr(value, '__getitem__')):    # dict-like
            if (len(value)):
                encodedItems = [self._encodeDictItem(variable, name, key, value[key], delim, prefix, joiner, first) for key in value.keys()]
                return delim.join([item for item in encodedItems if (item is not None)])
            return None
        elif (hasattr(value, '__getitem__')):   # list-like
            if (len(value)):
                encodedItems = [self._encodeListItem(variable, name, index, item, delim, prefix, joiner, first) for index, item in enumerate(value)]
                return delim.join([item for item in encodedItems if (item is not None)])
            return None
        else:
            return self._encodeStr(variable, name, unicode(value).lower(), prefix, joiner, first)

    def expand(self, values):
        return None


class Literal(Expression):
    def __init__(self, value):
        Expression.__init__(self)
        self.value = value

    def expand(self, values):
        return self._encode(self.value, (URITemplate.unreserved + URITemplate.reserved), True)


class Expansion(Expression):
    operator = ''
    varJoiner = ','
    
    def __init__(self, variables):
        Expression.__init__(self)
        self.vars = [Variable(var) for var in variables.split(',')]

    @property
    def variables(self):
        return [var.name for var in self.vars]
    
    def _expandVar(self, variable, value):
        return self._encodeVar(variable, self._uriEncodeName(variable.name), value)
    
    def expand(self, values):
        expandedVars = []
        for var in self.vars:
            if ((var.name in values) and (values[var.name] is not None)):
                expandedVar = self._expandVar(var, values[var.name])
                if (expandedVar is not None):
                    expandedVars.append(expandedVar)
        if (len(expandedVars)):
            expanded = self.varJoiner.join(expandedVars)
            if (expanded is not None):
                return self.operator + expanded
        return None
    
    
class SimpleExpansion(Expansion):
    def __init__(self, variables):
        Expansion.__init__(self, variables)


class ReservedExpansion(Expansion):
    def __init__(self, variables):
        Expansion.__init__(self, variables[1:])

    def _uriEncodeValue(self, value):
        return self._encode(value, (URITemplate.unreserved + URITemplate.reserved), True)


class FragmentExpansion(ReservedExpansion):
    operator = '#'
    
    def __init__(self, variables):
        ReservedExpansion.__init__(self, variables)


class LabelExpansion(Expansion):
    operator = '.'
    varJoiner = '.'
    
    def __init__(self, variables):
        Expansion.__init__(self, variables[1:])

    def _expandVar(self, variable, value):
        return self._encodeVar(variable, self._uriEncodeName(variable.name), value, delim = ('.' if variable.explode else ','))


class PathExpansion(Expansion):
    operator = '/'
    varJoiner = '/'
    
    def __init__(self, variables):
        Expansion.__init__(self, variables[1:])

    def _expandVar(self, variable, value):
        return self._encodeVar(variable, self._uriEncodeName(variable.name), value, delim = ('/' if variable.explode else ','))


class PathStyleExpansion(Expansion):
    operator = ';'
    varJoiner = ';'
    
    def __init__(self, variables):
        Expansion.__init__(self, variables[1:])

    def _encodeStr(self, variable, name, value, prefix, joiner, first):
        if (variable.array):
            if (name):
                prefix = prefix + '[' + name + ']' if (prefix) else name
        elif (variable.explode):
            prefix = self._join(prefix, '.', name)
        return Expansion._encodeStr(self, variable, name, value, prefix, joiner, first)
    
    def _encodeDictItem(self, variable, name, key, item, delim, prefix, joiner, first):
        if (variable.array):
            if (name):
                prefix = prefix + '[' + name + ']' if (prefix) else name
            prefix = (prefix + '[' + self._uriEncodeName(key) + ']') if (prefix and not first) else self._uriEncodeName(key)
        elif (variable.explode):
            prefix = self._join(prefix, '.', name) if (not first) else ''
        else:
            prefix = self._join(prefix, '.', self._uriEncodeName(key))
            joiner = ','
        return self._encodeVar(variable, self._uriEncodeName(key) if (not variable.array) else '', item, delim, prefix, joiner, False)

    def _encodeListItem(self, variable, name, index, item, delim, prefix, joiner, first):
        if (variable.array):
            if (name):
                prefix = prefix + '[' + name + ']' if (prefix) else name
            return self._encodeVar(variable, unicode(index), item, delim, prefix, joiner, False)
        return self._encodeVar(variable, name, item, delim, prefix, '=' if (variable.explode) else '.', False)

    def _expandVar(self, variable, value):
        if (variable.explode):
            return self._encodeVar(variable, self._uriEncodeName(variable.name), value, delim = ';')
        value = self._encodeVar(variable, self._uriEncodeName(variable.name), value, delim = ',')
        return (self._uriEncodeName(variable.name) + '=' + value) if (value) else variable.name


class FormStyleQueryExpansion(PathStyleExpansion):
    operator = '?'
    varJoiner = '&'
    
    def __init__(self, variables):
        PathStyleExpansion.__init__(self, variables)

    def _expandVar(self, variable, value):
        if (variable.explode):
            return self._encodeVar(variable, self._uriEncodeName(variable.name), value, delim = '&')
        value = self._encodeVar(variable, self._uriEncodeName(variable.name), value, delim = ',')
        return (self._uriEncodeName(variable.name) + '=' + value) if (value is not None) else None


class FormStyleQueryContinuation(FormStyleQueryExpansion):
    operator = '&'
    
    def __init__(self, variables):
        FormStyleQueryExpansion.__init__(self, variables)


