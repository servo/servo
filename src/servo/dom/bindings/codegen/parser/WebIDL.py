# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

""" A WebIDL parser. """

from ply import lex, yacc
import re
import os
import traceback

# Machinery

def parseInt(literal):
    string = literal
    sign = 0
    base = 0

    if string[0] == '-':
        sign = -1
        string = string[1:]
    else:
        sign = 1

    if string[0] == '0' and len(string) > 1:
        if string[1] == 'x' or string[1] == 'X':
            base = 16
            string = string[2:]
        else:
            base = 8
            string = string[1:]
    else:
        base = 10

    value = int(string, base)
    return value * sign

# Magic for creating enums
def M_add_class_attribs(attribs):
    def foo(name, bases, dict_):
        for v, k in enumerate(attribs):
            dict_[k] = v
        assert 'length' not in dict_
        dict_['length'] = len(attribs)
        return type(name, bases, dict_)
    return foo

def enum(*names):
    class Foo(object):
        __metaclass__ = M_add_class_attribs(names)
        def __setattr__(self, name, value):  # this makes it read-only
            raise NotImplementedError
    return Foo()

class WebIDLError(Exception):
    def __init__(self, message, locations, warning=False):
        self.message = message
        self.locations = [str(loc) for loc in locations]
        self.warning = warning

    def __str__(self):
        return "%s: %s%s%s" % (self.warning and 'warning' or 'error',
                                 self.message,
                                 ", " if len(self.locations) != 0 else "",
                                 "\n".join(self.locations))

class Location(object):
    def __init__(self, lexer, lineno, lexpos, filename):
        self._line = None
        self._lineno = lineno
        self._lexpos = lexpos
        self._lexdata = lexer.lexdata
        self._file = filename if filename else "<unknown>"

    def __eq__(self, other):
        return self._lexpos == other._lexpos and \
               self._file == other._file

    def filename(self):
        return self._file

    def resolve(self):
        if self._line:
            return

        startofline = self._lexdata.rfind('\n', 0, self._lexpos) + 1
        endofline = self._lexdata.find('\n', self._lexpos, self._lexpos + 80)
        if endofline != -1:
            self._line = self._lexdata[startofline:endofline]
        else:
            self._line = self._lexdata[startofline:]
        self._colno = self._lexpos - startofline

        # Our line number seems to point to the start of self._lexdata
        self._lineno += self._lexdata.count('\n', 0, startofline)

    def get(self):
        self.resolve()
        return "%s line %s:%s" % (self._file, self._lineno, self._colno)

    def _pointerline(self):
        return " " * self._colno + "^"

    def __str__(self):
        self.resolve()
        return "%s line %s:%s\n%s\n%s" % (self._file, self._lineno, self._colno,
                                          self._line, self._pointerline())

class BuiltinLocation(object):
    def __init__(self, text):
        self.msg = text + "\n"

    def __eq__(self, other):
        return isinstance(other, BuiltinLocation) and \
               self.msg == other.msg

    def filename(self):
        return '<builtin>'

    def resolve(self):
        pass

    def get(self):
        return self.msg

    def __str__(self):
        return self.get()


# Data Model

class IDLObject(object):
    def __init__(self, location):
        self.location = location
        self.userData = dict()

    def filename(self):
        return self.location.filename()

    def isInterface(self):
        return False

    def isEnum(self):
        return False

    def isCallback(self):
        return False

    def isType(self):
        return False

    def isDictionary(self):
        return False;

    def isUnion(self):
        return False

    def getUserData(self, key, default):
        return self.userData.get(key, default)

    def setUserData(self, key, value):
        self.userData[key] = value

    def addExtendedAttributes(self, attrs):
        assert False # Override me!

    def handleExtendedAttribute(self, attr):
        assert False # Override me!

class IDLScope(IDLObject):
    def __init__(self, location, parentScope, identifier):
        IDLObject.__init__(self, location)

        self.parentScope = parentScope
        if identifier:
            assert isinstance(identifier, IDLIdentifier)
            self._name = identifier
        else:
            self._name = None

        self._dict = {}

    def __str__(self):
        return self.QName()

    def QName(self):
        if self._name:
            return self._name.QName() + "::"
        return "::"

    def ensureUnique(self, identifier, object):
        """
            Ensure that there is at most one 'identifier' in scope ('self').
            Note that object can be None.  This occurs if we end up here for an
            interface type we haven't seen yet.
        """
        assert isinstance(identifier, IDLUnresolvedIdentifier)
        assert not object or isinstance(object, IDLObjectWithIdentifier)
        assert not object or object.identifier == identifier

        if identifier.name in self._dict:
            if not object:
                return

            # ensureUnique twice with the same object is not allowed
            assert object != self._dict[identifier.name]

            replacement = self.resolveIdentifierConflict(self, identifier,
                                                         self._dict[identifier.name],
                                                         object)
            self._dict[identifier.name] = replacement
            return

        assert object

        self._dict[identifier.name] = object

    def resolveIdentifierConflict(self, scope, identifier, originalObject, newObject):
        if isinstance(originalObject, IDLExternalInterface) and \
           isinstance(newObject, IDLExternalInterface) and \
           originalObject.identifier.name == newObject.identifier.name:
            return originalObject
            
        # Default to throwing, derived classes can override.
        conflictdesc = "\n\t%s at %s\n\t%s at %s" % \
          (originalObject, originalObject.location, newObject, newObject.location)

        raise WebIDLError(
            "Multiple unresolvable definitions of identifier '%s' in scope '%s%s"
            % (identifier.name, str(self), conflictdesc), [])

    def _lookupIdentifier(self, identifier):
        return self._dict[identifier.name]

    def lookupIdentifier(self, identifier):
        assert isinstance(identifier, IDLIdentifier)
        assert identifier.scope == self
        return self._lookupIdentifier(identifier)

class IDLIdentifier(IDLObject):
    def __init__(self, location, scope, name):
        IDLObject.__init__(self, location)

        self.name = name
        assert isinstance(scope, IDLScope)
        self.scope = scope

    def __str__(self):
        return self.QName()

    def QName(self):
        return self.scope.QName() + self.name

    def __hash__(self):
        return self.QName().__hash__()

    def __eq__(self, other):
        return self.QName() == other.QName()

    def object(self):
        return self.scope.lookupIdentifier(self)

class IDLUnresolvedIdentifier(IDLObject):
    def __init__(self, location, name, allowDoubleUnderscore = False,
                 allowForbidden = False):
        IDLObject.__init__(self, location)

        assert len(name) > 0

        if name[:2] == "__" and not allowDoubleUnderscore:
            raise WebIDLError("Identifiers beginning with __ are reserved",
                              [location])
        if name[0] == '_' and not allowDoubleUnderscore:
            name = name[1:]
        if name in ["prototype", "constructor", "toString"] and not allowForbidden:
            raise WebIDLError("Cannot use reserved identifier '%s'" % (name),
                              [location])

        self.name = name

    def __str__(self):
        return self.QName()

    def QName(self):
        return "<unresolved scope>::" + self.name

    def resolve(self, scope, object):
        assert isinstance(scope, IDLScope)
        assert not object or isinstance(object, IDLObjectWithIdentifier)
        assert not object or object.identifier == self

        scope.ensureUnique(self, object)

        identifier = IDLIdentifier(self.location, scope, self.name)
        if object:
            object.identifier = identifier
        return identifier

    def finish(self):
        assert False # Should replace with a resolved identifier first.

class IDLObjectWithIdentifier(IDLObject):
    def __init__(self, location, parentScope, identifier):
        IDLObject.__init__(self, location)

        assert isinstance(identifier, IDLUnresolvedIdentifier)

        self.identifier = identifier

        if parentScope:
            self.resolve(parentScope)

        self.treatNullAs = "Default"
        self.treatUndefinedAs = "Default"

    def resolve(self, parentScope):
        assert isinstance(parentScope, IDLScope)
        assert isinstance(self.identifier, IDLUnresolvedIdentifier)
        self.identifier.resolve(parentScope, self)

    def checkForStringHandlingExtendedAttributes(self, attrs,
                                                 isDictionaryMember=False,
                                                 isOptional=False):
        """
        A helper function to deal with TreatNullAs and TreatUndefinedAs.  Returns the list
        of attrs it didn't handle itself.
        """
        assert isinstance(self, IDLArgument) or isinstance(self, IDLAttribute)
        unhandledAttrs = list()
        for attr in attrs:
            if not attr.hasValue():
                unhandledAttrs.append(attr)
                continue

            identifier = attr.identifier()
            value = attr.value()
            if identifier == "TreatNullAs":
                if not self.type.isString() or self.type.nullable():
                    raise WebIDLError("[TreatNullAs] is only allowed on "
                                      "arguments or attributes whose type is "
                                      "DOMString",
                                      [self.location])
                if isDictionaryMember:
                    raise WebIDLError("[TreatNullAs] is not allowed for "
                                      "dictionary members", [self.location])
                if value != 'EmptyString':
                    raise WebIDLError("[TreatNullAs] must take the identifier "
                                      "'EmptyString', not '%s'" % value,
                                      [self.location])
                self.treatNullAs = value
            elif identifier == "TreatUndefinedAs":
                if not self.type.isString():
                    raise WebIDLError("[TreatUndefinedAs] is only allowed on "
                                      "arguments or attributes whose type is "
                                      "DOMString or DOMString?",
                                      [self.location])
                if isDictionaryMember:
                    raise WebIDLError("[TreatUndefinedAs] is not allowed for "
                                      "dictionary members", [self.location])
                if value == 'Null':
                    if not self.type.nullable():
                        raise WebIDLError("[TreatUndefinedAs=Null] is only "
                                          "allowed on arguments whose type is "
                                          "DOMString?", [self.location])
                elif value == 'Missing':
                    if not isOptional:
                        raise WebIDLError("[TreatUndefinedAs=Missing] is only "
                                          "allowed on optional arguments",
                                          [self.location])
                elif value != 'EmptyString':
                    raise WebIDLError("[TreatUndefinedAs] must take the "
                                      "identifiers EmptyString or Null or "
                                      "Missing", [self.location])
                self.treatUndefinedAs = value
            else:
                unhandledAttrs.append(attr)

        return unhandledAttrs

class IDLObjectWithScope(IDLObjectWithIdentifier, IDLScope):
    def __init__(self, location, parentScope, identifier):
        assert isinstance(identifier, IDLUnresolvedIdentifier)

        IDLObjectWithIdentifier.__init__(self, location, parentScope, identifier)
        IDLScope.__init__(self, location, parentScope, self.identifier)

class IDLIdentifierPlaceholder(IDLObjectWithIdentifier):
    def __init__(self, location, identifier):
        assert isinstance(identifier, IDLUnresolvedIdentifier)
        IDLObjectWithIdentifier.__init__(self, location, None, identifier)

    def finish(self, scope):
        try:
            scope._lookupIdentifier(self.identifier)
        except:
            raise WebIDLError("Unresolved type '%s'." % self.identifier,
                              [self.location])

        obj = self.identifier.resolve(scope, None)
        return scope.lookupIdentifier(obj)

class IDLExternalInterface(IDLObjectWithIdentifier):
    def __init__(self, location, parentScope, identifier):
        assert isinstance(identifier, IDLUnresolvedIdentifier)
        assert isinstance(parentScope, IDLScope)
        self.parent = None
        IDLObjectWithIdentifier.__init__(self, location, parentScope, identifier)
        IDLObjectWithIdentifier.resolve(self, parentScope)

    def finish(self, scope):
        pass

    def validate(self):
        pass

    def isExternal(self):
        return True

    def isInterface(self):
        return True

    def isConsequential(self):
        return False

    def addExtendedAttributes(self, attrs):
        assert len(attrs) == 0

    def resolve(self, parentScope):
        pass

class IDLInterface(IDLObjectWithScope):
    def __init__(self, location, parentScope, name, parent, members):
        assert isinstance(parentScope, IDLScope)
        assert isinstance(name, IDLUnresolvedIdentifier)
        assert not parent or isinstance(parent, IDLIdentifierPlaceholder)

        self.parent = parent
        self._callback = False
        self._finished = False
        self.members = list(members) # clone the list
        self.implementedInterfaces = set()
        self._consequential = False
        # self.interfacesBasedOnSelf is the set of interfaces that inherit from
        # self or have self as a consequential interface, including self itself.
        # Used for distinguishability checking.
        self.interfacesBasedOnSelf = set([self])

        IDLObjectWithScope.__init__(self, location, parentScope, name)

    def __str__(self):
        return "Interface '%s'" % self.identifier.name

    def ctor(self):
        identifier = IDLUnresolvedIdentifier(self.location, "constructor",
                                             allowForbidden=True)
        try:
            return self._lookupIdentifier(identifier)
        except:
            return None

    def resolveIdentifierConflict(self, scope, identifier, originalObject, newObject):
        assert isinstance(scope, IDLScope)
        assert isinstance(originalObject, IDLInterfaceMember)
        assert isinstance(newObject, IDLInterfaceMember)

        if originalObject.tag != IDLInterfaceMember.Tags.Method or \
           newObject.tag != IDLInterfaceMember.Tags.Method:
            # Call the base class method, which will throw
            IDLScope.resolveIdentifierConflict(self, identifier, originalObject,
                                               newObject)
            assert False # Not reached

        retval = originalObject.addOverload(newObject)
        # Might be a ctor, which isn't in self.members
        if newObject in self.members:
            self.members.remove(newObject)
        return retval

    def finish(self, scope):
        if self._finished:
            return

        self._finished = True

        assert not self.parent or isinstance(self.parent, IDLIdentifierPlaceholder)
        parent = self.parent.finish(scope) if self.parent else None
        if parent and isinstance(parent, IDLExternalInterface):
            raise WebIDLError("%s inherits from %s which does not have "
                              "a definition" %
                              (self.identifier.name,
                               self.parent.identifier.name),
                              [self.location])
        assert not parent or isinstance(parent, IDLInterface)

        self.parent = parent

        assert iter(self.members)

        if self.parent:
            self.parent.finish(scope)

            # Callbacks must not inherit from non-callbacks or inherit from
            # anything that has consequential interfaces.
            # XXXbz Can non-callbacks inherit from callbacks?  Spec issue pending.
            # XXXbz Can callbacks have consequential interfaces?  Spec issue pending
            if self.isCallback():
                if not self.parent.isCallback():
                    raise WebIDLError("Callback interface %s inheriting from "
                                      "non-callback interface %s" %
                                      (self.identifier.name,
                                       self.parent.identifier.name),
                                      [self.location, self.parent.location])
            elif self.parent.isCallback():
                raise WebIDLError("Non-callback interface %s inheriting from "
                                  "callback interface %s" %
                                  (self.identifier.name,
                                   self.parent.identifier.name),
                                  [self.location, self.parent.location])

        for iface in self.implementedInterfaces:
            iface.finish(scope)

        cycleInGraph = self.findInterfaceLoopPoint(self)
        if cycleInGraph:
            raise WebIDLError("Interface %s has itself as ancestor or "
                              "implemented interface" % self.identifier.name,
                              [self.location, cycleInGraph.location])

        if self.isCallback():
            # "implements" should have made sure we have no
            # consequential interfaces.
            assert len(self.getConsequentialInterfaces()) == 0
            # And that we're not consequential.
            assert not self.isConsequential()

        # Now resolve() and finish() our members before importing the
        # ones from our implemented interfaces.

        # resolve() will modify self.members, so we need to iterate
        # over a copy of the member list here.
        for member in list(self.members):
            member.resolve(self)

        for member in self.members:
            member.finish(scope)

        ctor = self.ctor()
        if ctor is not None:
            ctor.finish(scope)

        # Make a copy of our member list, so things tht implement us
        # can get those without all the stuff we implement ourselves
        # admixed.
        self.originalMembers = list(self.members)

        # Import everything from our consequential interfaces into
        # self.members.  Sort our consequential interfaces by name
        # just so we have a consistent order.
        for iface in sorted(self.getConsequentialInterfaces(),
                            cmp=cmp,
                            key=lambda x: x.identifier.name):
            # Flag the interface as being someone's consequential interface
            iface.setIsConsequentialInterfaceOf(self)
            additionalMembers = iface.originalMembers;
            for additionalMember in additionalMembers:
                for member in self.members:
                    if additionalMember.identifier.name == member.identifier.name:
                        raise WebIDLError(
                            "Multiple definitions of %s on %s coming from 'implements' statements" %
                            (member.identifier.name, self),
                            [additionalMember.location, member.location])
            self.members.extend(additionalMembers)

        for ancestor in self.getInheritedInterfaces():
            ancestor.interfacesBasedOnSelf.add(self)
            for ancestorConsequential in ancestor.getConsequentialInterfaces():
                ancestorConsequential.interfacesBasedOnSelf.add(self)

        # Ensure that there's at most one of each {named,indexed}
        # {getter,setter,creator,deleter}.
        specialMembersSeen = set()
        for member in self.members:
            if member.tag != IDLInterfaceMember.Tags.Method:
                continue

            if member.isGetter():
                memberType = "getters"
            elif member.isSetter():
                memberType = "setters"
            elif member.isCreator():
                memberType = "creators"
            elif member.isDeleter():
                memberType = "deleters"
            else:
                continue

            if member.isNamed():
                memberType = "named " + memberType
            elif member.isIndexed():
                memberType = "indexed " + memberType
            else:
                continue

            if memberType in specialMembersSeen:
                raise WebIDLError("Multiple " + memberType + " on %s" % (self),
                                   [self.location])

            specialMembersSeen.add(memberType)

    def validate(self):
        for member in self.members:
            member.validate()

    def isInterface(self):
        return True

    def isExternal(self):
        return False

    def setIsConsequentialInterfaceOf(self, other):
        self._consequential = True
        self.interfacesBasedOnSelf.add(other)

    def isConsequential(self):
        return self._consequential

    def setCallback(self, value):
        self._callback = value

    def isCallback(self):
        return self._callback

    def inheritanceDepth(self):
        depth = 0
        parent = self.parent
        while parent:
            depth = depth + 1
            parent = parent.parent
        return depth

    def hasConstants(self):
        return any(m.isConst() for m in self.members)

    def hasInterfaceObject(self):
        if self.isCallback():
            return self.hasConstants()
        return not hasattr(self, "_noInterfaceObject")

    def hasInterfacePrototypeObject(self):
        return not self.isCallback() and self.getUserData('hasConcreteDescendant', False)

    def addExtendedAttributes(self, attrs):
        self._extendedAttrDict = {}
        for attr in attrs:
            identifier = attr.identifier()

            # Special cased attrs
            if identifier == "TreatNonCallableAsNull":
                raise WebIDLError("TreatNonCallableAsNull cannot be specified on interfaces",
                                  [attr.location, self.location])
            elif identifier == "NoInterfaceObject":
                if not attr.noArguments():
                    raise WebIDLError("[NoInterfaceObject] must take no arguments",
                                      [attr.location])

                if self.ctor():
                    raise WebIDLError("Constructor and NoInterfaceObject are incompatible",
                                      [self.location])

                self._noInterfaceObject = True
            elif identifier == "Constructor":
                if not self.hasInterfaceObject():
                    raise WebIDLError("Constructor and NoInterfaceObject are incompatible",
                                      [self.location])

                args = attr.args() if attr.hasArgs() else []

                retType = IDLWrapperType(self.location, self)
                
                identifier = IDLUnresolvedIdentifier(self.location, "constructor",
                                                     allowForbidden=True)

                method = IDLMethod(self.location, identifier, retType, args)
                # Constructors are always Creators and are always
                # assumed to be able to throw (since there's no way to
                # indicate otherwise) and never have any other
                # extended attributes.
                method.addExtendedAttributes(
                    [IDLExtendedAttribute(self.location, ("Creator",)),
                     IDLExtendedAttribute(self.location, ("Throws",))])
                method.resolve(self)

            attrlist = attr.listValue()
            self._extendedAttrDict[identifier] = attrlist if len(attrlist) else True

    def addImplementedInterface(self, implementedInterface):
        assert(isinstance(implementedInterface, IDLInterface))
        self.implementedInterfaces.add(implementedInterface)

    def getInheritedInterfaces(self):
        """
        Returns a list of the interfaces this interface inherits from
        (not including this interface itself).  The list is in order
        from most derived to least derived.
        """
        assert(self._finished)
        if not self.parent:
            return []
        parentInterfaces = self.parent.getInheritedInterfaces()
        parentInterfaces.insert(0, self.parent)
        return parentInterfaces

    def getConsequentialInterfaces(self):
        assert(self._finished)
        # The interfaces we implement directly
        consequentialInterfaces = set(self.implementedInterfaces)

        # And their inherited interfaces
        for iface in self.implementedInterfaces:
            consequentialInterfaces |= set(iface.getInheritedInterfaces())

        # And now collect up the consequential interfaces of all of those
        temp = set()
        for iface in consequentialInterfaces:
            temp |= iface.getConsequentialInterfaces()

        return consequentialInterfaces | temp

    def findInterfaceLoopPoint(self, otherInterface):
        """
        Finds an interface, amongst our ancestors and consequential interfaces,
        that inherits from otherInterface or implements otherInterface
        directly.  If there is no such interface, returns None.
        """
        if self.parent:
            if self.parent == otherInterface:
                return self
            loopPoint = self.parent.findInterfaceLoopPoint(otherInterface)
            if loopPoint:
                return loopPoint
        if otherInterface in self.implementedInterfaces:
            return self
        for iface in self.implementedInterfaces:
            loopPoint = iface.findInterfaceLoopPoint(otherInterface)
            if loopPoint:
                return loopPoint
        return None

    def getExtendedAttribute(self, name):
        return self._extendedAttrDict.get(name, None)

class IDLDictionary(IDLObjectWithScope):
    def __init__(self, location, parentScope, name, parent, members):
        assert isinstance(parentScope, IDLScope)
        assert isinstance(name, IDLUnresolvedIdentifier)
        assert not parent or isinstance(parent, IDLIdentifierPlaceholder)

        self.parent = parent
        self._finished = False
        self.members = list(members)

        IDLObjectWithScope.__init__(self, location, parentScope, name)

    def __str__(self):
        return "Dictionary '%s'" % self.identifier.name

    def isDictionary(self):
        return True;

    def finish(self, scope):
        if self._finished:
            return

        self._finished = True

        if self.parent:
            assert isinstance(self.parent, IDLIdentifierPlaceholder)
            oldParent = self.parent
            self.parent = self.parent.finish(scope)
            if not isinstance(self.parent, IDLDictionary):
                raise WebIDLError("Dictionary %s has parent that is not a dictionary" %
                                  self.identifier.name,
                                  [oldParent.location, self.parent.location])

            # Make sure the parent resolves all its members before we start
            # looking at them.
            self.parent.finish(scope)

        for member in self.members:
            member.resolve(self)
            if not member.isComplete():
                member.complete(scope)
                assert member.type.isComplete()

        # Members of a dictionary are sorted in lexicographic order
        self.members.sort(cmp=cmp, key=lambda x: x.identifier.name)

        inheritedMembers = []
        ancestor = self.parent
        while ancestor:
            if ancestor == self:
                raise WebIDLError("Dictionary %s has itself as an ancestor" %
                                  self.identifier.name,
                                  [self.identifier.location])
            inheritedMembers.extend(ancestor.members)
            ancestor = ancestor.parent

        # Catch name duplication
        for inheritedMember in inheritedMembers:
            for member in self.members:
                if member.identifier.name == inheritedMember.identifier.name:
                    raise WebIDLError("Dictionary %s has two members with name %s" %
                                      (self.identifier.name, member.identifier.name),
                                      [member.location, inheritedMember.location])

    def validate(self):
        pass

    def addExtendedAttributes(self, attrs):
        assert len(attrs) == 0


class IDLEnum(IDLObjectWithIdentifier):
    def __init__(self, location, parentScope, name, values):
        assert isinstance(parentScope, IDLScope)
        assert isinstance(name, IDLUnresolvedIdentifier)

        if len(values) != len(set(values)):
            raise WebIDLError("Enum %s has multiple identical strings" % name.name,
                              [location])

        IDLObjectWithIdentifier.__init__(self, location, parentScope, name)
        self._values = values

    def values(self):
        return self._values

    def finish(self, scope):
        pass

    def validate(self):
        pass

    def isEnum(self):
        return True

    def addExtendedAttributes(self, attrs):
        assert len(attrs) == 0

class IDLType(IDLObject):
    Tags = enum(
        # The integer types
        'int8',
        'uint8',
        'int16',
        'uint16',
        'int32',
        'uint32',
        'int64',
        'uint64',
        # Additional primitive types
        'bool',
        'float',
        'double',
        # Other types
        'any',
        'domstring',
        'object',
        'date',
        'void',
        # Funny stuff
        'interface',
        'dictionary',
        'enum',
        'callback',
        'union'
        )

    def __init__(self, location, name):
        IDLObject.__init__(self, location)
        self.name = name
        self.builtin = False

    def __eq__(self, other):
        return other and self.builtin == other.builtin and self.name == other.name

    def __ne__(self, other):
        return not self == other

    def __str__(self):
        return str(self.name)

    def isType(self):
        return True

    def nullable(self):
        return False

    def isPrimitive(self):
        return False

    def isString(self):
        return False

    def isVoid(self):
        return self.name == "Void"

    def isSequence(self):
        return False

    def isArray(self):
        return False

    def isArrayBuffer(self):
        return False

    def isArrayBufferView(self):
        return False

    def isTypedArray(self):
        return False

    def isCallbackInterface(self):
        return False

    def isNonCallbackInterface(self):
        return False

    def isGeckoInterface(self):
        """ Returns a boolean indicating whether this type is an 'interface'
            type that is implemented in Gecko. At the moment, this returns
            true for all interface types that are not types from the TypedArray
            spec."""
        return self.isInterface() and not self.isSpiderMonkeyInterface()

    def isSpiderMonkeyInterface(self):
        """ Returns a boolean indicating whether this type is an 'interface'
            type that is implemented in Spidermonkey.  At the moment, this
            only returns true for the types from the TypedArray spec. """
        return self.isInterface() and (self.isArrayBuffer() or \
                                       self.isArrayBufferView() or \
                                       self.isTypedArray())

    def isDictionary(self):
        return False

    def isInterface(self):
        return False

    def isAny(self):
        return self.tag() == IDLType.Tags.any and not self.isSequence()

    def isDate(self):
        return self.tag() == IDLType.Tags.date

    def isObject(self):
        return self.tag() == IDLType.Tags.object

    def isComplete(self):
        return True

    def tag(self):
        assert False # Override me!

    def treatNonCallableAsNull(self):
        if not (self.nullable() and self.tag() == IDLType.Tags.callback):
            raise WebIDLError("Type %s cannot be TreatNonCallableAsNull" % self,
                              [self.location])

        return hasattr(self, "_treatNonCallableAsNull")

    def markTreatNonCallableAsNull(self):
        assert not self.treatNonCallableAsNull()
        self._treatNonCallableAsNull = True

    def addExtendedAttributes(self, attrs):
        assert len(attrs) == 0

    def resolveType(self, parentScope):
        pass

    def unroll(self):
        return self

    def isDistinguishableFrom(self, other):
        raise TypeError("Can't tell whether a generic type is or is not "
                        "distinguishable from other things")

class IDLUnresolvedType(IDLType):
    """
        Unresolved types are interface types 
    """

    def __init__(self, location, name):
        IDLType.__init__(self, location, name)

    def isComplete(self):
        return False

    def complete(self, scope):
        obj = None
        try:
            obj = scope._lookupIdentifier(self.name)
        except:
            raise WebIDLError("Unresolved type '%s'." % self.name,
                              [self.location])

        assert obj
        if obj.isType():
            # obj itself might not be complete; deal with that.
            assert obj != self
            if not obj.isComplete():
                obj = obj.complete(scope)
            return obj

        name = self.name.resolve(scope, None)
        return IDLWrapperType(self.location, obj)

    def isDistinguishableFrom(self, other):
        raise TypeError("Can't tell whether an unresolved type is or is not "
                        "distinguishable from other things")

class IDLNullableType(IDLType):
    def __init__(self, location, innerType):
        assert not innerType.isVoid()
        assert not innerType == BuiltinTypes[IDLBuiltinType.Types.any]

        IDLType.__init__(self, location, innerType.name)
        self.inner = innerType
        self.builtin = False

    def __eq__(self, other):
        return isinstance(other, IDLNullableType) and self.inner == other.inner

    def __str__(self):
        return self.inner.__str__() + "OrNull"

    def nullable(self):
        return True

    def isCallback(self):
        return self.inner.isCallback()

    def isPrimitive(self):
        return self.inner.isPrimitive()

    def isString(self):
        return self.inner.isString()

    def isFloat(self):
        return self.inner.isFloat()

    def isInteger(self):
        return self.inner.isInteger()

    def isVoid(self):
        return False

    def isSequence(self):
        return self.inner.isSequence()

    def isArray(self):
        return self.inner.isArray()

    def isArrayBuffer(self):
        return self.inner.isArrayBuffer()

    def isArrayBufferView(self):
        return self.inner.isArrayBufferView()

    def isTypedArray(self):
        return self.inner.isTypedArray()

    def isDictionary(self):
        return self.inner.isDictionary()

    def isInterface(self):
        return self.inner.isInterface()

    def isCallbackInterface(self):
        return self.inner.isCallbackInterface()

    def isNonCallbackInterface(self):
        return self.inner.isNonCallbackInterface()

    def isEnum(self):
        return self.inner.isEnum()

    def isUnion(self):
        return self.inner.isUnion()

    def tag(self):
        return self.inner.tag()

    def resolveType(self, parentScope):
        assert isinstance(parentScope, IDLScope)
        self.inner.resolveType(parentScope)

    def isComplete(self):
        return self.inner.isComplete()

    def complete(self, scope):
        self.inner = self.inner.complete(scope)
        if self.inner.nullable():
            raise WebIDLError("The inner type of a nullable type must not be "
                              "a nullable type",
                              [self.location, self.inner.location])
        if self.inner.isUnion():
            if self.inner.hasNullableType:
                raise WebIDLError("The inner type of a nullable type must not "
                                  "be a union type that itself has a nullable "
                                  "type as a member type", [self.location])
            # Check for dictionaries in the union
            for memberType in self.inner.flatMemberTypes:
                if memberType.isDictionary():
                    raise WebIDLError("The inner type of a nullable type must "
                                      "not be a union type containing a "
                                      "dictionary type",
                                      [self.location, memberType.location])
                    
        if self.inner.isDictionary():
            raise WebIDLError("The inner type of a nullable type must not be a "
                              "dictionary type", [self.location])

        self.name = self.inner.name
        return self

    def unroll(self):
        return self.inner.unroll()

    def isDistinguishableFrom(self, other):
        if (other.nullable() or (other.isUnion() and other.hasNullableType) or
            other.isDictionary()):
            # Can't tell which type null should become
            return False
        return self.inner.isDistinguishableFrom(other)

class IDLSequenceType(IDLType):
    def __init__(self, location, parameterType):
        assert not parameterType.isVoid()

        IDLType.__init__(self, location, parameterType.name)
        self.inner = parameterType
        self.builtin = False

    def __eq__(self, other):
        return isinstance(other, IDLSequenceType) and self.inner == other.inner

    def __str__(self):
        return self.inner.__str__() + "Sequence"

    def nullable(self):
        return False

    def isPrimitive(self):
        return False;

    def isString(self):
        return False;

    def isVoid(self):
        return False

    def isSequence(self):
        return True

    def isArray(self):
        return False

    def isDictionary(self):
        return False

    def isInterface(self):
        return False

    def isEnum(self):
        return False

    def tag(self):
        # XXXkhuey this is probably wrong.
        return self.inner.tag()

    def resolveType(self, parentScope):
        assert isinstance(parentScope, IDLScope)
        self.inner.resolveType(parentScope)

    def isComplete(self):
        return self.inner.isComplete()

    def complete(self, scope):
        self.inner = self.inner.complete(scope)
        self.name = self.inner.name
        return self

    def unroll(self):
        return self.inner.unroll()

    def isDistinguishableFrom(self, other):
        if other.isUnion():
            # Just forward to the union; it'll deal
            return other.isDistinguishableFrom(self)
        return (other.isPrimitive() or other.isString() or other.isEnum() or
                other.isDictionary() or other.isDate() or
                other.isNonCallbackInterface())

class IDLUnionType(IDLType):
    def __init__(self, location, memberTypes):
        IDLType.__init__(self, location, "")
        self.memberTypes = memberTypes
        self.hasNullableType = False
        self.flatMemberTypes = None
        self.builtin = False

    def __eq__(self, other):
        return isinstance(other, IDLUnionType) and self.memberTypes == other.memberTypes

    def isVoid(self):
        return False

    def isUnion(self):
        return True

    def tag(self):
        return IDLType.Tags.union

    def resolveType(self, parentScope):
        assert isinstance(parentScope, IDLScope)
        for t in self.memberTypes:
            t.resolveType(parentScope)

    def isComplete(self):
        return self.flatMemberTypes is not None

    def complete(self, scope):
        def typeName(type):
            if isinstance(type, IDLNullableType):
                return typeName(type.inner) + "OrNull"
            if isinstance(type, IDLWrapperType):
                return typeName(type._identifier.object())
            if isinstance(type, IDLObjectWithIdentifier):
                return typeName(type.identifier)
            if isinstance(type, IDLType) and (type.isArray() or type.isSequence()):
                return str(type)
            return type.name

        for (i, type) in enumerate(self.memberTypes):
            if not type.isComplete():
                self.memberTypes[i] = type.complete(scope)

        self.name = "Or".join(typeName(type) for type in self.memberTypes)
        self.flatMemberTypes = list(self.memberTypes)
        i = 0
        while i < len(self.flatMemberTypes):
            if self.flatMemberTypes[i].nullable():
                if self.hasNullableType:
                    raise WebIDLError("Can't have more than one nullable types in a union",
                                      [nullableType.location, self.flatMemberTypes[i].location])
                self.hasNullableType = True
                nullableType = self.flatMemberTypes[i]
                self.flatMemberTypes[i] = self.flatMemberTypes[i].inner
                continue
            if self.flatMemberTypes[i].isUnion():
                self.flatMemberTypes[i:i + 1] = self.flatMemberTypes[i].memberTypes
                continue
            i += 1

        for (i, t) in enumerate(self.flatMemberTypes[:-1]):
            for u in self.flatMemberTypes[i + 1:]:
                if not t.isDistinguishableFrom(u):
                    raise WebIDLError("Flat member types of a union should be "
                                      "distinguishable, " + str(t) + " is not "
                                      "distinguishable from " + str(u),
                                      [self.location, t.location, u.location])

        return self

    def isDistinguishableFrom(self, other):
        if self.hasNullableType and other.nullable():
            # Can't tell which type null should become
            return False
        if other.isUnion():
            otherTypes = other.unroll().memberTypes
        else:
            otherTypes = [other]
        # For every type in otherTypes, check that it's distinguishable from
        # every type in our types
        for u in otherTypes:
            if any(not t.isDistinguishableFrom(u) for t in self.memberTypes):
                return False
        return True

class IDLArrayType(IDLType):
    def __init__(self, location, parameterType):
        assert not parameterType.isVoid()
        if parameterType.isSequence():
            raise WebIDLError("Array type cannot parameterize over a sequence type",
                              [location])
        if parameterType.isDictionary():
            raise WebIDLError("Array type cannot parameterize over a dictionary type",
                              [location])

        IDLType.__init__(self, location, parameterType.name)
        self.inner = parameterType
        self.builtin = False

    def __eq__(self, other):
        return isinstance(other, IDLArrayType) and self.inner == other.inner

    def __str__(self):
        return self.inner.__str__() + "Array"

    def nullable(self):
        return False

    def isPrimitive(self):
        return False

    def isString(self):
        return False

    def isVoid(self):
        return False

    def isSequence(self):
        assert not self.inner.isSequence()
        return False

    def isArray(self):
        return True

    def isDictionary(self):
        assert not self.inner.isDictionary()
        return False

    def isInterface(self):
        return False

    def isEnum(self):
        return False

    def tag(self):
        # XXXkhuey this is probably wrong.
        return self.inner.tag()

    def resolveType(self, parentScope):
        assert isinstance(parentScope, IDLScope)
        self.inner.resolveType(parentScope)

    def isComplete(self):
        return self.inner.isComplete()

    def complete(self, scope):
        self.inner = self.inner.complete(scope)
        self.name = self.inner.name
        return self

    def unroll(self):
        return self.inner.unroll()

    def isDistinguishableFrom(self, other):
        if other.isUnion():
            # Just forward to the union; it'll deal
            return other.isDistinguishableFrom(self)
        return (other.isPrimitive() or other.isString() or other.isEnum() or
                other.isDictionary() or other.isDate() or
                other.isNonCallbackInterface())

class IDLTypedefType(IDLType, IDLObjectWithIdentifier):
    def __init__(self, location, innerType, name):
        IDLType.__init__(self, location, innerType.name)

        identifier = IDLUnresolvedIdentifier(location, name)

        IDLObjectWithIdentifier.__init__(self, location, None, identifier)

        self.inner = innerType
        self.name = name
        self.builtin = False

    def __eq__(self, other):
        return isinstance(other, IDLTypedefType) and self.inner == other.inner

    def __str__(self):
        return self.identifier.name

    def nullable(self):
        return self.inner.nullable()

    def isPrimitive(self):
        return self.inner.isPrimitive()

    def isString(self):
        return self.inner.isString()

    def isVoid(self):
        return self.inner.isVoid()

    def isSequence(self):
        return self.inner.isSequence()

    def isArray(self):
        return self.inner.isArray()

    def isDictionary(self):
        return self.inner.isDictionary()

    def isArrayBuffer(self):
        return self.inner.isArrayBuffer()

    def isArrayBufferView(self):
        return self.inner.isArrayBufferView()

    def isTypedArray(self):
        return self.inner.isTypedArray()

    def isInterface(self):
        return self.inner.isInterface()

    def isCallbackInterface(self):
        return self.inner.isCallbackInterface()

    def isNonCallbackInterface(self):
        return self.inner.isNonCallbackInterface()

    def isComplete(self):
        return False

    def complete(self, parentScope):
        if not self.inner.isComplete():
            self.inner = self.inner.complete(parentScope)
        assert self.inner.isComplete()
        return self.inner

    def finish(self, parentScope):
        # Maybe the IDLObjectWithIdentifier for the typedef should be
        # a separate thing from the type?
        self.complete(parentScope)

    def validate(self):
        pass

    # Do we need a resolveType impl?  I don't think it's particularly useful....

    def tag(self):
        return self.inner.tag()

    def unroll(self):
        return self.inner.unroll()

    def isDistinguishableFrom(self, other):
        return self.inner.isDistinguishableFrom(other)

class IDLWrapperType(IDLType):
    def __init__(self, location, inner):
        IDLType.__init__(self, location, inner.identifier.name)
        self.inner = inner
        self._identifier = inner.identifier
        self.builtin = False

    def __eq__(self, other):
        return isinstance(other, IDLWrapperType) and \
               self._identifier == other._identifier and \
               self.builtin == other.builtin

    def __str__(self):
        return str(self.name) + " (Wrapper)"

    def nullable(self):
        return False

    def isPrimitive(self):
        return False

    def isString(self):
        return False

    def isVoid(self):
        return False

    def isSequence(self):
        return False

    def isArray(self):
        return False

    def isDictionary(self):
        return isinstance(self.inner, IDLDictionary)

    def isInterface(self):
        return isinstance(self.inner, IDLInterface) or \
               isinstance(self.inner, IDLExternalInterface)

    def isCallbackInterface(self):
        return self.isInterface() and self.inner.isCallback()

    def isNonCallbackInterface(self):
        return self.isInterface() and not self.inner.isCallback()

    def isEnum(self):
        return isinstance(self.inner, IDLEnum)

    def resolveType(self, parentScope):
        assert isinstance(parentScope, IDLScope)
        self.inner.resolve(parentScope)

    def isComplete(self):
        return True

    def tag(self):
        if self.isInterface():
            return IDLType.Tags.interface
        elif self.isEnum():
            return IDLType.Tags.enum
        elif self.isDictionary():
            return IDLType.Tags.dictionary
        else:
            assert False

    def isDistinguishableFrom(self, other):
        if other.isUnion():
            # Just forward to the union; it'll deal
            return other.isDistinguishableFrom(self)
        assert self.isInterface() or self.isEnum() or self.isDictionary()
        if self.isEnum():
            return (other.isInterface() or other.isObject() or
                    other.isCallback() or other.isDictionary() or
                    other.isSequence() or other.isArray() or
                    other.isDate())
        if other.isPrimitive() or other.isString() or other.isEnum() or other.isDate():
            return True
        if self.isDictionary():
            return (not other.nullable() and
                    (other.isNonCallbackInterface() or other.isSequence() or
                     other.isArray()))

        assert self.isInterface()
        # XXXbz need to check that the interfaces can't be implemented
        # by the same object
        if other.isInterface():
            if other.isSpiderMonkeyInterface():
                # Just let |other| handle things
                return other.isDistinguishableFrom(self)
            assert self.isGeckoInterface() and other.isGeckoInterface()
            if self.inner.isExternal() or other.unroll().inner.isExternal():
                return self != other
            return (len(self.inner.interfacesBasedOnSelf &
                        other.unroll().inner.interfacesBasedOnSelf) == 0 and
                    (self.isNonCallbackInterface() or
                     other.isNonCallbackInterface()))
        if (other.isDictionary() or other.isCallback() or
            other.isSequence() or other.isArray()):
            return self.isNonCallbackInterface()

        # Not much else |other| can be
        assert other.isObject()
        return False

class IDLBuiltinType(IDLType):

    Types = enum(
        # The integer types
        'byte',
        'octet',
        'short',
        'unsigned_short',
        'long',
        'unsigned_long',
        'long_long',
        'unsigned_long_long',
        # Additional primitive types
        'boolean',
        'float',
        'double',
        # Other types
        'any',
        'domstring',
        'object',
        'date',
        'void',
        # Funny stuff
        'ArrayBuffer',
        'ArrayBufferView',
        'Int8Array',
        'Uint8Array',
        'Uint8ClampedArray',
        'Int16Array',
        'Uint16Array',
        'Int32Array',
        'Uint32Array',
        'Float32Array',
        'Float64Array'
        )

    TagLookup = {
            Types.byte: IDLType.Tags.int8,
            Types.octet: IDLType.Tags.uint8,
            Types.short: IDLType.Tags.int16,
            Types.unsigned_short: IDLType.Tags.uint16,
            Types.long: IDLType.Tags.int32,
            Types.unsigned_long: IDLType.Tags.uint32,
            Types.long_long: IDLType.Tags.int64,
            Types.unsigned_long_long: IDLType.Tags.uint64,
            Types.boolean: IDLType.Tags.bool,
            Types.float: IDLType.Tags.float,
            Types.double: IDLType.Tags.double,
            Types.any: IDLType.Tags.any,
            Types.domstring: IDLType.Tags.domstring,
            Types.object: IDLType.Tags.object,
            Types.date: IDLType.Tags.date,
            Types.void: IDLType.Tags.void,
            Types.ArrayBuffer: IDLType.Tags.interface,
            Types.ArrayBufferView: IDLType.Tags.interface,
            Types.Int8Array: IDLType.Tags.interface,
            Types.Uint8Array: IDLType.Tags.interface,
            Types.Uint8ClampedArray: IDLType.Tags.interface,
            Types.Int16Array: IDLType.Tags.interface,
            Types.Uint16Array: IDLType.Tags.interface,
            Types.Int32Array: IDLType.Tags.interface,
            Types.Uint32Array: IDLType.Tags.interface,
            Types.Float32Array: IDLType.Tags.interface,
            Types.Float64Array: IDLType.Tags.interface
        }

    def __init__(self, location, name, type):
        IDLType.__init__(self, location, name)
        self.builtin = True
        self._typeTag = type

    def isPrimitive(self):
        return self._typeTag <= IDLBuiltinType.Types.double

    def isString(self):
        return self._typeTag == IDLBuiltinType.Types.domstring

    def isInteger(self):
        return self._typeTag <= IDLBuiltinType.Types.unsigned_long_long

    def isArrayBuffer(self):
        return self._typeTag == IDLBuiltinType.Types.ArrayBuffer

    def isArrayBufferView(self):
        return self._typeTag == IDLBuiltinType.Types.ArrayBufferView

    def isTypedArray(self):
        return self._typeTag >= IDLBuiltinType.Types.Int8Array and \
               self._typeTag <= IDLBuiltinType.Types.Float64Array

    def isInterface(self):
        # TypedArray things are interface types per the TypedArray spec,
        # but we handle them as builtins because SpiderMonkey implements
        # all of it internally.
        return self.isArrayBuffer() or \
               self.isArrayBufferView() or \
               self.isTypedArray()

    def isNonCallbackInterface(self):
        # All the interfaces we can be are non-callback
        return self.isInterface()

    def isFloat(self):
        return self._typeTag == IDLBuiltinType.Types.float or \
               self._typeTag == IDLBuiltinType.Types.double

    def tag(self):
        return IDLBuiltinType.TagLookup[self._typeTag]

    def isDistinguishableFrom(self, other):
        if other.isUnion():
            # Just forward to the union; it'll deal
            return other.isDistinguishableFrom(self)
        if self.isPrimitive() or self.isString():
            return (other.isInterface() or other.isObject() or
                    other.isCallback() or other.isDictionary() or
                    other.isSequence() or other.isArray() or
                    other.isDate())
        if self.isAny():
            # Can't tell "any" apart from anything
            return False
        if self.isObject():
            return other.isPrimitive() or other.isString() or other.isEnum()
        if self.isDate():
            return (other.isPrimitive() or other.isString() or other.isEnum() or
                    other.isInterface() or other.isCallback() or
                    other.isDictionary() or other.isSequence() or
                    other.isArray())
        if self.isVoid():
            return not other.isVoid()
        # Not much else we could be!
        assert self.isSpiderMonkeyInterface()
        # Like interfaces, but we know we're not a callback
        return (other.isPrimitive() or other.isString() or other.isEnum() or
                other.isCallback() or other.isDictionary() or
                other.isSequence() or other.isArray() or other.isDate() or
                (other.isInterface() and (
                 # ArrayBuffer is distinguishable from everything
                 # that's not an ArrayBuffer or a callback interface
                 (self.isArrayBuffer() and not other.isArrayBuffer()) or
                 # ArrayBufferView is distinguishable from everything
                 # that's not an ArrayBufferView or typed array.
                 (self.isArrayBufferView() and not other.isArrayBufferView() and
                  not other.isTypedArray()) or
                 # Typed arrays are distinguishable from everything
                 # except ArrayBufferView and the same type of typed
                 # array
                 (self.isTypedArray() and not other.isArrayBufferView() and not
                  (other.isTypedArray() and other.name == self.name)))))

BuiltinTypes = {
      IDLBuiltinType.Types.byte:
          IDLBuiltinType(BuiltinLocation("<builtin type>"), "Byte",
                         IDLBuiltinType.Types.byte),
      IDLBuiltinType.Types.octet:
          IDLBuiltinType(BuiltinLocation("<builtin type>"), "Octet",
                         IDLBuiltinType.Types.octet),
      IDLBuiltinType.Types.short:
          IDLBuiltinType(BuiltinLocation("<builtin type>"), "Short",
                         IDLBuiltinType.Types.short),
      IDLBuiltinType.Types.unsigned_short:
          IDLBuiltinType(BuiltinLocation("<builtin type>"), "UnsignedShort",
                         IDLBuiltinType.Types.unsigned_short),
      IDLBuiltinType.Types.long:
          IDLBuiltinType(BuiltinLocation("<builtin type>"), "Long",
                         IDLBuiltinType.Types.long),
      IDLBuiltinType.Types.unsigned_long:
          IDLBuiltinType(BuiltinLocation("<builtin type>"), "UnsignedLong",
                         IDLBuiltinType.Types.unsigned_long),
      IDLBuiltinType.Types.long_long:
          IDLBuiltinType(BuiltinLocation("<builtin type>"), "LongLong",
                         IDLBuiltinType.Types.long_long),
      IDLBuiltinType.Types.unsigned_long_long:
          IDLBuiltinType(BuiltinLocation("<builtin type>"), "UnsignedLongLong",
                         IDLBuiltinType.Types.unsigned_long_long),
      IDLBuiltinType.Types.boolean:
          IDLBuiltinType(BuiltinLocation("<builtin type>"), "Boolean",
                         IDLBuiltinType.Types.boolean),
      IDLBuiltinType.Types.float:
          IDLBuiltinType(BuiltinLocation("<builtin type>"), "Float",
                         IDLBuiltinType.Types.float),
      IDLBuiltinType.Types.double:
          IDLBuiltinType(BuiltinLocation("<builtin type>"), "Double",
                         IDLBuiltinType.Types.double),
      IDLBuiltinType.Types.any:
          IDLBuiltinType(BuiltinLocation("<builtin type>"), "Any",
                         IDLBuiltinType.Types.any),
      IDLBuiltinType.Types.domstring:
          IDLBuiltinType(BuiltinLocation("<builtin type>"), "String",
                         IDLBuiltinType.Types.domstring),
      IDLBuiltinType.Types.object:
          IDLBuiltinType(BuiltinLocation("<builtin type>"), "Object",
                         IDLBuiltinType.Types.object),
      IDLBuiltinType.Types.date:
          IDLBuiltinType(BuiltinLocation("<builtin type>"), "Date",
                         IDLBuiltinType.Types.date),
      IDLBuiltinType.Types.void:
          IDLBuiltinType(BuiltinLocation("<builtin type>"), "Void",
                         IDLBuiltinType.Types.void),
      IDLBuiltinType.Types.ArrayBuffer:
          IDLBuiltinType(BuiltinLocation("<builtin type>"), "ArrayBuffer",
                         IDLBuiltinType.Types.ArrayBuffer),
      IDLBuiltinType.Types.ArrayBufferView:
          IDLBuiltinType(BuiltinLocation("<builtin type>"), "ArrayBufferView",
                         IDLBuiltinType.Types.ArrayBufferView),
      IDLBuiltinType.Types.Int8Array:
          IDLBuiltinType(BuiltinLocation("<builtin type>"), "Int8Array",
                         IDLBuiltinType.Types.Int8Array),
      IDLBuiltinType.Types.Uint8Array:
          IDLBuiltinType(BuiltinLocation("<builtin type>"), "Uint8Array",
                         IDLBuiltinType.Types.Uint8Array),
      IDLBuiltinType.Types.Uint8ClampedArray:
          IDLBuiltinType(BuiltinLocation("<builtin type>"), "Uint8ClampedArray",
                         IDLBuiltinType.Types.Uint8ClampedArray),
      IDLBuiltinType.Types.Int16Array:
          IDLBuiltinType(BuiltinLocation("<builtin type>"), "Int16Array",
                         IDLBuiltinType.Types.Int16Array),
      IDLBuiltinType.Types.Uint16Array:
          IDLBuiltinType(BuiltinLocation("<builtin type>"), "Uint16Array",
                         IDLBuiltinType.Types.Uint16Array),
      IDLBuiltinType.Types.Int32Array:
          IDLBuiltinType(BuiltinLocation("<builtin type>"), "Int32Array",
                         IDLBuiltinType.Types.Int32Array),
      IDLBuiltinType.Types.Uint32Array:
          IDLBuiltinType(BuiltinLocation("<builtin type>"), "Uint32Array",
                         IDLBuiltinType.Types.Uint32Array),
      IDLBuiltinType.Types.Float32Array:
          IDLBuiltinType(BuiltinLocation("<builtin type>"), "Float32Array",
                         IDLBuiltinType.Types.Float32Array),
      IDLBuiltinType.Types.Float64Array:
          IDLBuiltinType(BuiltinLocation("<builtin type>"), "Float64Array",
                         IDLBuiltinType.Types.Float64Array)
    }


integerTypeSizes = {
        IDLBuiltinType.Types.byte: (-128, 127),
        IDLBuiltinType.Types.octet:  (0, 255),
        IDLBuiltinType.Types.short: (-32768, 32767),
        IDLBuiltinType.Types.unsigned_short: (0, 65535),
        IDLBuiltinType.Types.long: (-2147483648, 2147483647),
        IDLBuiltinType.Types.unsigned_long: (0, 4294967295),
        IDLBuiltinType.Types.long_long: (-9223372036854775808,
                                         9223372036854775807),
        IDLBuiltinType.Types.unsigned_long_long: (0, 18446744073709551615)
    }

def matchIntegerValueToType(value):
    for type, extremes in integerTypeSizes.items():
        (min, max) = extremes
        if value <= max and value >= min:
            return BuiltinTypes[type]

    return None

class IDLValue(IDLObject):
    def __init__(self, location, type, value):
        IDLObject.__init__(self, location)
        self.type = type
        assert isinstance(type, IDLType)

        self.value = value

    def coerceToType(self, type, location):
        if type == self.type:
            return self # Nothing to do

        # If the type allows null, rerun this matching on the inner type
        if type.nullable():
            innerValue = self.coerceToType(type.inner, location)
            return IDLValue(self.location, type, innerValue.value)

        # Else, see if we can coerce to 'type'.
        if self.type.isInteger() and type.isInteger():
            # We're both integer types.  See if we fit.

            (min, max) = integerTypeSizes[type._typeTag]
            if self.value <= max and self.value >= min:
                # Promote
                return IDLValue(self.location, type, self.value)
            else:
                raise WebIDLError("Value %s is out of range for type %s." %
                                  (self.value, type), [location])
        elif self.type.isString() and type.isEnum():
            # Just keep our string, but make sure it's a valid value for this enum
            if self.value not in type.inner.values():
                raise WebIDLError("'%s' is not a valid default value for enum %s"
                                  % (self.value, type.inner.identifier.name),
                                  [location, type.inner.location])
            return self
        else:
            raise WebIDLError("Cannot coerce type %s to type %s." %
                              (self.type, type), [location])

class IDLNullValue(IDLObject):
    def __init__(self, location):
        IDLObject.__init__(self, location)
        self.type = None
        self.value = None

    def coerceToType(self, type, location):
        if (not isinstance(type, IDLNullableType) and
            not (type.isUnion() and type.hasNullableType) and
            not type.isDictionary() and
            not type.isAny()):
            raise WebIDLError("Cannot coerce null value to type %s." % type,
                              [location])

        nullValue = IDLNullValue(self.location)
        nullValue.type = type
        return nullValue
        

class IDLInterfaceMember(IDLObjectWithIdentifier):

    Tags = enum(
        'Const',
        'Attr',
        'Method'
    )

    def __init__(self, location, identifier, tag):
        IDLObjectWithIdentifier.__init__(self, location, None, identifier)
        self.tag = tag
        self._extendedAttrDict = {}

    def isMethod(self):
        return self.tag == IDLInterfaceMember.Tags.Method

    def isAttr(self):
        return self.tag == IDLInterfaceMember.Tags.Attr

    def isConst(self):
        return self.tag == IDLInterfaceMember.Tags.Const

    def addExtendedAttributes(self, attrs):
        for attr in attrs:
            self.handleExtendedAttribute(attr)
            attrlist = attr.listValue()
            self._extendedAttrDict[attr.identifier()] = attrlist if len(attrlist) else True

    def handleExtendedAttribute(self, attr):
        pass

    def getExtendedAttribute(self, name):
        return self._extendedAttrDict.get(name, None)

class IDLConst(IDLInterfaceMember):
    def __init__(self, location, identifier, type, value):
        IDLInterfaceMember.__init__(self, location, identifier,
                                    IDLInterfaceMember.Tags.Const)

        assert isinstance(type, IDLType)
        if type.isDictionary():
            raise WebIDLError("A constant cannot be of a dictionary type",
                              [self.location])
        self.type = type
        self.value = value

    def __str__(self):
        return "'%s' const '%s'" % (self.type, self.identifier)

    def finish(self, scope):
        if not self.type.isComplete():
            type = self.type.complete(scope)
            if not type.isPrimitive() and not type.isString():
                locations = [self.type.location, type.location]
                try:
                    locations.append(type.inner.location)
                except:
                    pass
                raise WebIDLError("Incorrect type for constant", locations)
            self.type = type

        # The value might not match the type
        coercedValue = self.value.coerceToType(self.type, self.location)
        assert coercedValue

        self.value = coercedValue

    def validate(self):
        pass

class IDLAttribute(IDLInterfaceMember):
    def __init__(self, location, identifier, type, readonly, inherit,
                 static=False):
        IDLInterfaceMember.__init__(self, location, identifier,
                                    IDLInterfaceMember.Tags.Attr)

        assert isinstance(type, IDLType)
        self.type = type
        self.readonly = readonly
        self.inherit = inherit
        self.static = static
        self.lenientThis = False

        if readonly and inherit:
            raise WebIDLError("An attribute cannot be both 'readonly' and 'inherit'",
                              [self.location])

    def isStatic(self):
        return self.static

    def __str__(self):
        return "'%s' attribute '%s'" % (self.type, self.identifier)

    def finish(self, scope):
        if not self.type.isComplete():
            t = self.type.complete(scope)

            assert not isinstance(t, IDLUnresolvedType)
            assert not isinstance(t, IDLTypedefType)
            assert not isinstance(t.name, IDLUnresolvedIdentifier)
            self.type = t

        if self.type.isDictionary():
            raise WebIDLError("An attribute cannot be of a dictionary type",
                              [self.location])
        if self.type.isSequence():
            raise WebIDLError("An attribute cannot be of a sequence type",
                              [self.location])
        if self.type.isUnion():
            for f in self.type.flatMemberTypes:
                if f.isDictionary():
                    raise WebIDLError("An attribute cannot be of a union "
                                      "type if one of its member types (or "
                                      "one of its member types's member "
                                      "types, and so on) is a dictionary "
                                      "type", [self.location, f.location])
                if f.isSequence():
                    raise WebIDLError("An attribute cannot be of a union "
                                      "type if one of its member types (or "
                                      "one of its member types's member "
                                      "types, and so on) is a sequence "
                                      "type", [self.location, f.location])

    def validate(self):
        pass

    def handleExtendedAttribute(self, attr):
        identifier = attr.identifier()
        if identifier == "TreatNonCallableAsNull":
            self.type.markTreatNonCallableAsNull();
        elif identifier == "SetterInfallible" and self.readonly:
            raise WebIDLError("Readonly attributes must not be flagged as "
                              "[SetterInfallible]",
                              [self.location])
        elif identifier == "LenientThis":
            if not attr.noArguments():
                raise WebIDLError("[LenientThis] must take no arguments",
                                  [attr.location])
            if self.isStatic():
                raise WebIDLError("[LenientThis] is only allowed on non-static "
                                  "attributes", [attr.location, self.location])
            self.lenientThis = True
        IDLInterfaceMember.handleExtendedAttribute(self, attr)

    def resolve(self, parentScope):
        assert isinstance(parentScope, IDLScope)
        self.type.resolveType(parentScope)
        IDLObjectWithIdentifier.resolve(self, parentScope)

    def addExtendedAttributes(self, attrs):
        attrs = self.checkForStringHandlingExtendedAttributes(attrs)
        IDLInterfaceMember.addExtendedAttributes(self, attrs)

    def hasLenientThis(self):
        return self.lenientThis

class IDLArgument(IDLObjectWithIdentifier):
    def __init__(self, location, identifier, type, optional=False, defaultValue=None, variadic=False, dictionaryMember=False):
        IDLObjectWithIdentifier.__init__(self, location, None, identifier)

        assert isinstance(type, IDLType)
        self.type = type

        self.optional = optional
        self.defaultValue = defaultValue
        self.variadic = variadic
        self.dictionaryMember = dictionaryMember
        self._isComplete = False
        self.enforceRange = False
        self.clamp = False

        assert not variadic or optional

    def addExtendedAttributes(self, attrs):
        attrs = self.checkForStringHandlingExtendedAttributes(
            attrs,
            isDictionaryMember=self.dictionaryMember,
            isOptional=self.optional)
        for attribute in attrs:
            identifier = attribute.identifier()
            if identifier == "Clamp":
                if not attribute.noArguments():
                    raise WebIDLError("[Clamp] must take no arguments",
                                      [attribute.location])
                if self.enforceRange:
                    raise WebIDLError("[EnforceRange] and [Clamp] are mutually exclusive",
                                      [self.location]);
                self.clamp = True
            elif identifier == "EnforceRange":
                if not attribute.noArguments():
                    raise WebIDLError("[EnforceRange] must take no arguments",
                                      [attribute.location])
                if self.clamp:
                    raise WebIDLError("[EnforceRange] and [Clamp] are mutually exclusive",
                                      [self.location]);
                self.enforceRange = True
            else:
                raise WebIDLError("Unhandled extended attribute on an argument",
                                  [attribute.location])

    def isComplete(self):
        return self._isComplete

    def complete(self, scope):
        if self._isComplete:
            return

        self._isComplete = True

        if not self.type.isComplete():
            type = self.type.complete(scope)
            assert not isinstance(type, IDLUnresolvedType)
            assert not isinstance(type, IDLTypedefType)
            assert not isinstance(type.name, IDLUnresolvedIdentifier)
            self.type = type

        if self.type.isDictionary() and self.optional and not self.defaultValue:
            # Default optional dictionaries to null, for simplicity,
            # so the codegen doesn't have to special-case this.
            self.defaultValue = IDLNullValue(self.location)

        # Now do the coercing thing; this needs to happen after the
        # above creation of a default value.
        if self.defaultValue:
            self.defaultValue = self.defaultValue.coerceToType(self.type,
                                                               self.location)
            assert self.defaultValue

class IDLCallbackType(IDLType, IDLObjectWithScope):
    def __init__(self, location, parentScope, identifier, returnType, arguments):
        assert isinstance(returnType, IDLType)

        IDLType.__init__(self, location, identifier.name)

        self._returnType = returnType
        # Clone the list
        self._arguments = list(arguments)

        IDLObjectWithScope.__init__(self, location, parentScope, identifier)

        for (returnType, arguments) in self.signatures():
            for argument in arguments:
                argument.resolve(self)

    def isCallback(self):
        return True

    def signatures(self):
        return [(self._returnType, self._arguments)]

    def tag(self):
        return IDLType.Tags.callback

    def finish(self, scope):
        if not self._returnType.isComplete():
            type = returnType.complete(scope)

            assert not isinstance(type, IDLUnresolvedType)
            assert not isinstance(type, IDLTypedefType)
            assert not isinstance(type.name, IDLUnresolvedIdentifier)
            self._returnType = type

        for argument in self._arguments:
            if argument.type.isComplete():
                continue

            type = argument.type.complete(scope)

            assert not isinstance(type, IDLUnresolvedType)
            assert not isinstance(type, IDLTypedefType)
            assert not isinstance(type.name, IDLUnresolvedIdentifier)
            argument.type = type

    def validate(self):
        pass

    def isDistinguishableFrom(self, other):
        if other.isUnion():
            # Just forward to the union; it'll deal
            return other.isDistinguishableFrom(self)
        return (other.isPrimitive() or other.isString() or other.isEnum() or
                other.isNonCallbackInterface() or other.isDate())

class IDLMethodOverload:
    """
    A class that represents a single overload of a WebIDL method.  This is not
    quite the same as an element of the "effective overload set" in the spec,
    because separate IDLMethodOverloads are not created based on arguments being
    optional.  Rather, when multiple methods have the same name, there is an
    IDLMethodOverload for each one, all hanging off an IDLMethod representing
    the full set of overloads.
    """
    def __init__(self, returnType, arguments, location):
        self.returnType = returnType
        # Clone the list of arguments, just in case
        self.arguments = list(arguments)
        self.location = location

class IDLMethod(IDLInterfaceMember, IDLScope):

    Special = enum(
        'None',
        'Getter',
        'Setter',
        'Creator',
        'Deleter',
        'LegacyCaller',
        'Stringifier',
        'Static'
    )

    TypeSuffixModifier = enum(
        'None',
        'QMark',
        'Brackets'
    )

    NamedOrIndexed = enum(
        'Neither',
        'Named',
        'Indexed'
    )

    def __init__(self, location, identifier, returnType, arguments,
                 static=False, getter=False, setter=False, creator=False,
                 deleter=False, specialType=NamedOrIndexed.Neither,
                 legacycaller=False, stringifier=False):
        # REVIEW: specialType is NamedOrIndexed -- wow, this is messed up.
        IDLInterfaceMember.__init__(self, location, identifier,
                                    IDLInterfaceMember.Tags.Method)

        self._hasOverloads = False

        assert isinstance(returnType, IDLType)

        # self._overloads is a list of IDLMethodOverloads
        self._overloads = [IDLMethodOverload(returnType, arguments, location)]

        assert isinstance(static, bool)
        self._static = static
        assert isinstance(getter, bool)
        self._getter = getter
        assert isinstance(setter, bool)
        self._setter = setter
        assert isinstance(creator, bool)
        self._creator = creator
        assert isinstance(deleter, bool)
        self._deleter = deleter
        assert isinstance(legacycaller, bool)
        self._legacycaller = legacycaller
        assert isinstance(stringifier, bool)
        self._stringifier = stringifier
        self._specialType = specialType

        self.assertSignatureConstraints()

    def __str__(self):
        return "Method '%s'" % self.identifier

    def assertSignatureConstraints(self):
        if self._getter or self._deleter:
            assert len(self._overloads) == 1
            overload = self._overloads[0]
            arguments =  overload.arguments
            assert len(arguments) == 1
            assert arguments[0].type == BuiltinTypes[IDLBuiltinType.Types.domstring] or \
                   arguments[0].type == BuiltinTypes[IDLBuiltinType.Types.unsigned_long]
            assert not arguments[0].optional and not arguments[0].variadic
            assert not self._getter or not overload.returnType.isVoid()

        if self._setter or self._creator:
            assert len(self._overloads) == 1
            arguments = self._overloads[0].arguments
            assert len(arguments) == 2
            assert arguments[0].type == BuiltinTypes[IDLBuiltinType.Types.domstring] or \
                   arguments[0].type == BuiltinTypes[IDLBuiltinType.Types.unsigned_long]
            assert not arguments[0].optional and not arguments[0].variadic
            assert not arguments[1].optional and not arguments[1].variadic

        if self._stringifier:
            assert len(self._overloads) == 1
            overload = self._overloads[0]
            assert len(overload.arguments) == 0
            assert overload.returnType == BuiltinTypes[IDLBuiltinType.Types.domstring]

    def isStatic(self):
        return self._static

    def isGetter(self):
        return self._getter

    def isSetter(self):
        return self._setter

    def isCreator(self):
        return self._creator

    def isDeleter(self):
        return self._deleter

    def isNamed(self):
        assert self._specialType == IDLMethod.NamedOrIndexed.Named or \
               self._specialType == IDLMethod.NamedOrIndexed.Indexed
        return self._specialType == IDLMethod.NamedOrIndexed.Named

    def isIndexed(self):
        assert self._specialType == IDLMethod.NamedOrIndexed.Named or \
               self._specialType == IDLMethod.NamedOrIndexed.Indexed
        return self._specialType == IDLMethod.NamedOrIndexed.Indexed

    def isLegacycaller(self):
        return self._legacycaller

    def isStringifier(self):
        return self._stringifier

    def hasOverloads(self):
        return self._hasOverloads

    def isIdentifierLess(self):
        return self.identifier.name[:2] == "__"

    def resolve(self, parentScope):
        assert isinstance(parentScope, IDLScope)
        IDLObjectWithIdentifier.resolve(self, parentScope)
        IDLScope.__init__(self, self.location, parentScope, self.identifier)
        for (returnType, arguments) in self.signatures():
            for argument in arguments:
                argument.resolve(self)

    def addOverload(self, method):
        assert len(method._overloads) == 1

        if self._extendedAttrDict != method ._extendedAttrDict:
            raise WebIDLError("Extended attributes differ on different "
                              "overloads of %s" % method.identifier,
                              [self.location, method.location])

        self._overloads.extend(method._overloads)

        self._hasOverloads = True

        if self.isStatic() != method.isStatic():
            raise WebIDLError("Overloaded identifier %s appears with different values of the 'static' attribute" % method.identifier,
                              [method.location])

        if self.isLegacycaller() != method.isLegacycaller():
            raise WebIDLError("Overloaded identifier %s appears with different values of the 'legacycaller' attribute" % method.identifier,
                              [method.location])

        # Can't overload special things!
        assert not self.isGetter()
        assert not method.isGetter()
        assert not self.isSetter()
        assert not method.isSetter()
        assert not self.isCreator()
        assert not method.isCreator()
        assert not self.isDeleter()
        assert not method.isDeleter()
        assert not self.isStringifier()
        assert not method.isStringifier()

        return self

    def signatures(self):
        return [(overload.returnType, overload.arguments) for overload in
                self._overloads]

    def finish(self, scope):
        for overload in self._overloads:
            inOptionalArguments = False
            variadicArgument = None
            sawOptionalWithNoDefault = False

            arguments = overload.arguments
            for (idx, argument) in enumerate(arguments):
                if argument.isComplete():
                    continue

                argument.complete(scope)
                assert argument.type.isComplete()

                if argument.type.isDictionary():
                    # Dictionaries at the end of the list or followed by
                    # optional arguments must be optional.
                    if (not argument.optional and
                        (idx == len(arguments) - 1 or arguments[idx+1].optional)):
                        raise WebIDLError("Dictionary argument not followed by "
                                          "a required argument must be "
                                          "optional", [argument.location])

                # Only the last argument can be variadic
                if variadicArgument:
                    raise WebIDLError("Variadic argument is not last argument",
                                      [variadicArgument.location])
                # Once we see an optional argument, there can't be any non-optional
                # arguments.
                if inOptionalArguments and not argument.optional:
                    raise WebIDLError("Non-optional argument after optional "
                                      "arguments",
                                      [argument.location])
                # Once we see an argument with no default value, there can
                # be no more default values.
                if sawOptionalWithNoDefault and argument.defaultValue:
                    raise WebIDLError("Argument with default value after "
                                      "optional arguments with no default "
                                      "values",
                                      [argument.location])
                inOptionalArguments = argument.optional
                if argument.variadic:
                    variadicArgument = argument
                sawOptionalWithNoDefault = (argument.optional and
                                            not argument.defaultValue)

            returnType = overload.returnType
            if returnType.isComplete():
                continue

            type = returnType.complete(scope)

            assert not isinstance(type, IDLUnresolvedType)
            assert not isinstance(type, IDLTypedefType)
            assert not isinstance(type.name, IDLUnresolvedIdentifier)
            overload.returnType = type

        # Now compute various information that will be used by the
        # WebIDL overload resolution algorithm.
        self.maxArgCount = max(len(s[1]) for s in self.signatures())
        self.allowedArgCounts = [ i for i in range(self.maxArgCount+1)
                                  if len(self.signaturesForArgCount(i)) != 0 ]

    def validate(self):
        # Make sure our overloads are properly distinguishable and don't have
        # different argument types before the distinguishing args.
        for argCount in self.allowedArgCounts:
            possibleOverloads = self.overloadsForArgCount(argCount)
            if len(possibleOverloads) == 1:
                continue
            distinguishingIndex = self.distinguishingIndexForArgCount(argCount)
            for idx in range(distinguishingIndex):
                firstSigType = possibleOverloads[0].arguments[idx].type
                for overload in possibleOverloads[1:]:
                    if overload.arguments[idx].type != firstSigType:
                        raise WebIDLError(
                            "Signatures for method '%s' with %d arguments have "
                            "different types of arguments at index %d, which "
                            "is before distinguishing index %d" %
                            (self.identifier.name, argCount, idx,
                             distinguishingIndex),
                            [self.location, overload.location])

    def overloadsForArgCount(self, argc):
        return [overload for overload in self._overloads if
                len(overload.arguments) == argc or
                (len(overload.arguments) > argc and
                 overload.arguments[argc].optional)]

    def signaturesForArgCount(self, argc):
        return [(overload.returnType, overload.arguments) for overload
                in self.overloadsForArgCount(argc)]

    def locationsForArgCount(self, argc):
        return [overload.location for overload in self._overloads if
                len(overload.arguments) == argc or
                (len(overload.arguments) > argc and
                 overload.arguments[argc].optional)]

    def distinguishingIndexForArgCount(self, argc):
        def isValidDistinguishingIndex(idx, signatures):
            for (firstSigIndex, (firstRetval, firstArgs)) in enumerate(signatures[:-1]):
                for (secondRetval, secondArgs) in signatures[firstSigIndex+1:]:
                    firstType = firstArgs[idx].type
                    secondType = secondArgs[idx].type
                    if not firstType.isDistinguishableFrom(secondType):
                        return False
            return True
        signatures = self.signaturesForArgCount(argc)
        for idx in range(argc):
            if isValidDistinguishingIndex(idx, signatures):
                return idx
        # No valid distinguishing index.  Time to throw
        locations = self.locationsForArgCount(argc)
        raise WebIDLError("Signatures with %d arguments for method '%s' are not "
                          "distinguishable" % (argc, self.identifier.name),
                          locations)

    def handleExtendedAttribute(self, attr):
        identifier = attr.identifier()
        if identifier == "GetterInfallible":
            raise WebIDLError("Methods must not be flagged as "
                              "[GetterInfallible]",
                              [attr.location, self.location])
        if identifier == "SetterInfallible":
            raise WebIDLError("Methods must not be flagged as "
                              "[SetterInfallible]",
                              [attr.location, self.location])
        IDLInterfaceMember.handleExtendedAttribute(self, attr)

class IDLImplementsStatement(IDLObject):
    def __init__(self, location, implementor, implementee):
        IDLObject.__init__(self, location)
        self.implementor = implementor;
        self.implementee = implementee

    def finish(self, scope):
        assert(isinstance(self.implementor, IDLIdentifierPlaceholder))
        assert(isinstance(self.implementee, IDLIdentifierPlaceholder))
        implementor = self.implementor.finish(scope)
        implementee = self.implementee.finish(scope)
        # NOTE: we depend on not setting self.implementor and
        # self.implementee here to keep track of the original
        # locations.
        if not isinstance(implementor, IDLInterface):
            raise WebIDLError("Left-hand side of 'implements' is not an "
                              "interface",
                              [self.implementor.location])
        if implementor.isCallback():
            raise WebIDLError("Left-hand side of 'implements' is a callback "
                              "interface",
                              [self.implementor.location])
        if not isinstance(implementee, IDLInterface):
            raise WebIDLError("Right-hand side of 'implements' is not an "
                              "interface",
                              [self.implementee.location])
        if implementee.isCallback():
            raise WebIDLError("Right-hand side of 'implements' is a callback "
                              "interface",
                              [self.implementee.location])
        implementor.addImplementedInterface(implementee)

    def validate(self):
        pass

    def addExtendedAttributes(self, attrs):
        assert len(attrs) == 0

class IDLExtendedAttribute(IDLObject):
    """
    A class to represent IDL extended attributes so we can give them locations
    """
    def __init__(self, location, tuple):
        IDLObject.__init__(self, location)
        self._tuple = tuple

    def identifier(self):
        return self._tuple[0]

    def noArguments(self):
        return len(self._tuple) == 1

    def hasValue(self):
        return len(self._tuple) == 2 and isinstance(self._tuple[1], str)

    def value(self):
        assert(self.hasValue())
        return self._tuple[1]

    def hasArgs(self):
        return (len(self._tuple) == 2 and isinstance(self._tuple[1], list) or
                len(self._tuple) == 3)

    def args(self):
        assert(self.hasArgs())
        # Our args are our last element
        return self._tuple[-1]

    def listValue(self):
        """
        Backdoor for storing random data in _extendedAttrDict
        """
        return list(self._tuple)[1:]

# Parser

class Tokenizer(object):
    tokens = [
        "INTEGER",
        "FLOATLITERAL",
        "IDENTIFIER",
        "STRING",
        "WHITESPACE",
        "OTHER"
        ]

    def t_INTEGER(self, t):
        r'-?(0([0-7]+|[Xx][0-9A-Fa-f]+)?|[1-9][0-9]*)'
        try:
            # Can't use int(), because that doesn't handle octal properly.
            t.value = parseInt(t.value)
        except:
            raise WebIDLError("Invalid integer literal",
                              [Location(lexer=self.lexer,
                                        lineno=self.lexer.lineno,
                                        lexpos=self.lexer.lexpos,
                                        filename=self._filename)])
        return t

    def t_FLOATLITERAL(self, t):
        r'-?(([0-9]+\.[0-9]*|[0-9]*\.[0-9]+)([Ee][+-]?[0-9]+)?|[0-9]+[Ee][+-]?[0-9]+)'
        assert False
        return t

    def t_IDENTIFIER(self, t):
        r'[A-Z_a-z][0-9A-Z_a-z]*'
        t.type = self.keywords.get(t.value, 'IDENTIFIER')
        return t

    def t_STRING(self, t):
        r'"[^"]*"'
        t.value = t.value[1:-1]
        return t

    def t_WHITESPACE(self, t):
        r'[\t\n\r ]+|[\t\n\r ]*((//[^\n]*|/\*.*?\*/)[\t\n\r ]*)+'
        pass

    def t_ELLIPSIS(self, t):
        r'\.\.\.'
        t.type = self.keywords.get(t.value)
        return t

    def t_OTHER(self, t):
        r'[^\t\n\r 0-9A-Z_a-z]'
        t.type = self.keywords.get(t.value, 'OTHER')
        return t

    keywords = {
        "module": "MODULE",
        "interface": "INTERFACE",
        "partial": "PARTIAL",
        "dictionary": "DICTIONARY",
        "exception": "EXCEPTION",
        "enum": "ENUM",
        "callback": "CALLBACK",
        "typedef": "TYPEDEF",
        "implements": "IMPLEMENTS",
        "const": "CONST",
        "null": "NULL",
        "true": "TRUE",
        "false": "FALSE",
        "stringifier": "STRINGIFIER",
        "attribute": "ATTRIBUTE",
        "readonly": "READONLY",
        "inherit": "INHERIT",
        "static": "STATIC",
        "getter": "GETTER",
        "setter": "SETTER",
        "creator": "CREATOR",
        "deleter": "DELETER",
        "legacycaller": "LEGACYCALLER",
        "optional": "OPTIONAL",
        "...": "ELLIPSIS",
        "::": "SCOPE",
        "Date": "DATE",
        "DOMString": "DOMSTRING",
        "any": "ANY",
        "boolean": "BOOLEAN",
        "byte": "BYTE",
        "double": "DOUBLE",
        "float": "FLOAT",
        "long": "LONG",
        "object": "OBJECT",
        "octet": "OCTET",
        "optional": "OPTIONAL",
        "sequence": "SEQUENCE",
        "short": "SHORT",
        "unsigned": "UNSIGNED",
        "void": "VOID",
        ":": "COLON",
        ";": "SEMICOLON",
        "{": "LBRACE",
        "}": "RBRACE",
        "(": "LPAREN",
        ")": "RPAREN",
        "[": "LBRACKET",
        "]": "RBRACKET",
        "?": "QUESTIONMARK",
        ",": "COMMA",
        "=": "EQUALS",
        "<": "LT",
        ">": "GT",
        "ArrayBuffer": "ARRAYBUFFER",
        "or": "OR"
        }

    tokens.extend(keywords.values())

    def t_error(self, t):
        raise WebIDLError("Unrecognized Input",
               [Location(lexer=self.lexer,
                         lineno=self.lexer.lineno,
                         lexpos=self.lexer.lexpos,
                         filename = self.filename)])

    def __init__(self, outputdir, lexer=None):
        if lexer:
            self.lexer = lexer
        else:
            self.lexer = lex.lex(object=self,
                                 outputdir=outputdir,
                                 lextab='webidllex',
                                 reflags=re.DOTALL)

class Parser(Tokenizer):
    def getLocation(self, p, i):
        return Location(self.lexer, p.lineno(i), p.lexpos(i), self._filename)

    def globalScope(self):
        return self._globalScope

    # The p_Foo functions here must match the WebIDL spec's grammar.
    # It's acceptable to split things at '|' boundaries.
    def p_Definitions(self, p):
        """ 
            Definitions : ExtendedAttributeList Definition Definitions
        """
        if p[2]:
            p[0] = [p[2]]
            p[2].addExtendedAttributes(p[1])
        else:
            assert not p[1]
            p[0] = []

        p[0].extend(p[3])

    def p_DefinitionsEmpty(self, p):
        """
            Definitions :
        """
        p[0] = []

    def p_Definition(self, p):
        """
            Definition : CallbackOrInterface
                       | PartialInterface
                       | Dictionary
                       | Exception
                       | Enum
                       | Typedef
                       | ImplementsStatement
        """
        p[0] = p[1]
        assert p[1] # We might not have implemented something ...

    def p_CallbackOrInterfaceCallback(self, p):
        """
            CallbackOrInterface : CALLBACK CallbackRestOrInterface
        """
        if p[2].isInterface():
            assert isinstance(p[2], IDLInterface)
            p[2].setCallback(True)

        p[0] = p[2]

    def p_CallbackOrInterfaceInterface(self, p):
        """
            CallbackOrInterface : Interface
        """
        p[0] = p[1]

    def p_CallbackRestOrInterface(self, p):
        """
            CallbackRestOrInterface : CallbackRest
                                    | Interface
        """
        assert p[1]
        p[0] = p[1]

    def p_Interface(self, p):
        """
            Interface : INTERFACE IDENTIFIER Inheritance LBRACE InterfaceMembers RBRACE SEMICOLON
        """
        location = self.getLocation(p, 1)
        identifier = IDLUnresolvedIdentifier(self.getLocation(p, 2), p[2])

        members = p[5]
        p[0] = IDLInterface(location, self.globalScope(), identifier, p[3], members)

    def p_InterfaceForwardDecl(self, p):
        """
            Interface : INTERFACE IDENTIFIER SEMICOLON
        """
        location = self.getLocation(p, 1)
        identifier = IDLUnresolvedIdentifier(self.getLocation(p, 2), p[2])

        try:
            if self.globalScope()._lookupIdentifier(identifier):
                p[0] = self.globalScope()._lookupIdentifier(identifier)
                return
        except:
            pass

        p[0] = IDLExternalInterface(location, self.globalScope(), identifier)

    def p_PartialInterface(self, p):
        """
            PartialInterface : PARTIAL INTERFACE IDENTIFIER LBRACE InterfaceMembers RBRACE SEMICOLON
        """
        pass

    def p_Inheritance(self, p):
        """
            Inheritance : COLON ScopedName
        """
        p[0] = IDLIdentifierPlaceholder(self.getLocation(p, 2), p[2])

    def p_InheritanceEmpty(self, p):
        """
            Inheritance :
        """
        pass

    def p_InterfaceMembers(self, p):
        """
            InterfaceMembers : ExtendedAttributeList InterfaceMember InterfaceMembers
        """
        p[0] = [p[2]] if p[2] else []

        assert not p[1] or p[2]
        p[2].addExtendedAttributes(p[1])

        p[0].extend(p[3])

    def p_InterfaceMembersEmpty(self, p):
        """
            InterfaceMembers :
        """
        p[0] = []

    def p_InterfaceMember(self, p):
        """
            InterfaceMember : Const
                            | AttributeOrOperation
        """
        p[0] = p[1]

    def p_Dictionary(self, p):
        """
            Dictionary : DICTIONARY IDENTIFIER Inheritance LBRACE DictionaryMembers RBRACE SEMICOLON
        """
        location = self.getLocation(p, 1)
        identifier = IDLUnresolvedIdentifier(self.getLocation(p, 2), p[2])
        members = p[5]
        p[0] = IDLDictionary(location, self.globalScope(), identifier, p[3], members)

    def p_DictionaryMembers(self, p):
        """
            DictionaryMembers : ExtendedAttributeList DictionaryMember DictionaryMembers
                             |
        """
        if len(p) == 1:
            # We're at the end of the list
            p[0] = []
            return
        # Add our extended attributes
        p[2].addExtendedAttributes(p[1])
        p[0] = [p[2]]
        p[0].extend(p[3])

    def p_DictionaryMember(self, p):
        """
            DictionaryMember : Type IDENTIFIER DefaultValue SEMICOLON
        """
        # These quack a lot like optional arguments, so just treat them that way.
        t = p[1]
        assert isinstance(t, IDLType)
        identifier = IDLUnresolvedIdentifier(self.getLocation(p, 2), p[2])
        defaultValue = p[3]

        p[0] = IDLArgument(self.getLocation(p, 2), identifier, t, optional=True,
                           defaultValue=defaultValue, variadic=False,
                           dictionaryMember=True)

    def p_DefaultValue(self, p):
        """
            DefaultValue : EQUALS ConstValue
                         |
        """
        if len(p) > 1:
            p[0] = p[2]
        else:
            p[0] = None

    def p_Exception(self, p):
        """
            Exception : EXCEPTION IDENTIFIER Inheritance LBRACE ExceptionMembers RBRACE SEMICOLON
        """
        pass

    def p_Enum(self, p):
        """
            Enum : ENUM IDENTIFIER LBRACE EnumValueList RBRACE SEMICOLON
        """
        location = self.getLocation(p, 1)
        identifier = IDLUnresolvedIdentifier(self.getLocation(p, 2), p[2])

        values = p[4]
        assert values
        p[0] = IDLEnum(location, self.globalScope(), identifier, values)

    def p_EnumValueList(self, p):
        """
            EnumValueList : STRING EnumValues
        """
        p[0] = [p[1]]
        p[0].extend(p[2])

    def p_EnumValues(self, p):
        """
            EnumValues : COMMA STRING EnumValues
        """
        p[0] = [p[2]]
        p[0].extend(p[3])

    def p_EnumValuesEmpty(self, p):
        """
            EnumValues :
        """
        p[0] = []

    def p_CallbackRest(self, p):
        """
            CallbackRest : IDENTIFIER EQUALS ReturnType LPAREN ArgumentList RPAREN SEMICOLON
        """
        identifier = IDLUnresolvedIdentifier(self.getLocation(p, 1), p[1])
        p[0] = IDLCallbackType(self.getLocation(p, 1), self.globalScope(),
                               identifier, p[3], p[5])

    def p_ExceptionMembers(self, p):
        """
            ExceptionMembers : ExtendedAttributeList ExceptionMember ExceptionMembers
                             |
        """
        pass

    def p_Typedef(self, p):
        """
            Typedef : TYPEDEF Type IDENTIFIER SEMICOLON
        """
        typedef = IDLTypedefType(self.getLocation(p, 1), p[2], p[3])
        typedef.resolve(self.globalScope())
        p[0] = typedef

    def p_ImplementsStatement(self, p):
        """
            ImplementsStatement : ScopedName IMPLEMENTS ScopedName SEMICOLON
        """
        assert(p[2] == "implements")
        implementor = IDLIdentifierPlaceholder(self.getLocation(p, 1), p[1])
        implementee = IDLIdentifierPlaceholder(self.getLocation(p, 3), p[3])
        p[0] = IDLImplementsStatement(self.getLocation(p, 1), implementor,
                                      implementee)

    def p_Const(self, p):
        """
            Const : CONST ConstType IDENTIFIER EQUALS ConstValue SEMICOLON
        """
        location = self.getLocation(p, 1)
        type = p[2]
        identifier = IDLUnresolvedIdentifier(self.getLocation(p, 3), p[3])
        value = p[5]
        p[0] = IDLConst(location, identifier, type, value)

    def p_ConstValueBoolean(self, p):
        """
            ConstValue : BooleanLiteral
        """
        location = self.getLocation(p, 1)
        booleanType = BuiltinTypes[IDLBuiltinType.Types.boolean]
        p[0] = IDLValue(location, booleanType, p[1])

    def p_ConstValueInteger(self, p):
        """
            ConstValue : INTEGER
        """
        location = self.getLocation(p, 1)

        # We don't know ahead of time what type the integer literal is.
        # Determine the smallest type it could possibly fit in and use that.
        integerType = matchIntegerValueToType(p[1])
        if integerType == None:
            raise WebIDLError("Integer literal out of range", [location])

        p[0] = IDLValue(location, integerType, p[1])

    def p_ConstValueFloat(self, p):
        """
            ConstValue : FLOATLITERAL
        """
        assert False
        pass

    def p_ConstValueString(self, p):
        """
            ConstValue : STRING
        """
        location = self.getLocation(p, 1)
        stringType = BuiltinTypes[IDLBuiltinType.Types.domstring]
        p[0] = IDLValue(location, stringType, p[1])

    def p_ConstValueNull(self, p):
        """
            ConstValue : NULL
        """
        p[0] = IDLNullValue(self.getLocation(p, 1))

    def p_BooleanLiteralTrue(self, p):
        """
            BooleanLiteral : TRUE
        """
        p[0] = True

    def p_BooleanLiteralFalse(self, p):
        """
            BooleanLiteral : FALSE
        """
        p[0] = False

    def p_AttributeOrOperation(self, p):
        """
            AttributeOrOperation : Attribute
                                 | Operation
        """
        p[0] = p[1]

    def p_Attribute(self, p):
        """
            Attribute : Inherit ReadOnly ATTRIBUTE Type IDENTIFIER SEMICOLON
        """
        location = self.getLocation(p, 3)
        inherit = p[1]
        readonly = p[2]
        t = p[4]
        identifier = IDLUnresolvedIdentifier(self.getLocation(p, 5), p[5])
        p[0] = IDLAttribute(location, identifier, t, readonly, inherit)

    def p_ReadOnly(self, p):
        """
            ReadOnly : READONLY
        """
        p[0] = True

    def p_ReadOnlyEmpty(self, p):
        """
            ReadOnly :
        """
        p[0] = False

    def p_Inherit(self, p):
        """
            Inherit : INHERIT
        """
        p[0] = True

    def p_InheritEmpty(self, p):
        """
            Inherit :
        """
        p[0] = False

    def p_Operation(self, p):
        """
            Operation : Qualifiers OperationRest
        """
        qualifiers = p[1]

        # Disallow duplicates in the qualifier set
        if not len(set(qualifiers)) == len(qualifiers):
            raise WebIDLError("Duplicate qualifiers are not allowed",
                              [self.getLocation(p, 1)])

        static = True if IDLMethod.Special.Static in p[1] else False
        # If static is there that's all that's allowed.  This is disallowed
        # by the parser, so we can assert here.
        assert not static or len(qualifiers) == 1

        getter = True if IDLMethod.Special.Getter in p[1] else False
        setter = True if IDLMethod.Special.Setter in p[1] else False
        creator = True if IDLMethod.Special.Creator in p[1] else False
        deleter = True if IDLMethod.Special.Deleter in p[1] else False
        legacycaller = True if IDLMethod.Special.LegacyCaller in p[1] else False
        stringifier = True if IDLMethod.Special.Stringifier in p[1] else False

        if getter or deleter:
            if setter or creator:
                raise WebIDLError("getter and deleter are incompatible with setter and creator",
                                  [self.getLocation(p, 1)])

        (returnType, identifier, arguments) = p[2]

        assert isinstance(returnType, IDLType)

        specialType = IDLMethod.NamedOrIndexed.Neither

        if getter or deleter:
            if len(arguments) != 1:
                raise WebIDLError("%s has wrong number of arguments" %
                                  ("getter" if getter else "deleter"),
                                  [self.getLocation(p, 2)])
            argType = arguments[0].type
            if argType == BuiltinTypes[IDLBuiltinType.Types.domstring]:
                specialType = IDLMethod.NamedOrIndexed.Named
            elif argType == BuiltinTypes[IDLBuiltinType.Types.unsigned_long]:
                specialType = IDLMethod.NamedOrIndexed.Indexed
            else:
                raise WebIDLError("%s has wrong argument type (must be DOMString or UnsignedLong)" %
                                  ("getter" if getter else "deleter"),
                                  [arguments[0].location])
            if arguments[0].optional or arguments[0].variadic:
                raise WebIDLError("%s cannot have %s argument" %
                                  ("getter" if getter else "deleter",
                                   "optional" if arguments[0].optional else "variadic"),
                                   [arguments[0].location])
        if getter:
            if returnType.isVoid():
                raise WebIDLError("getter cannot have void return type",
                                  [self.getLocation(p, 2)])
        if setter or creator:
            if len(arguments) != 2:
                raise WebIDLError("%s has wrong number of arguments" %
                                  ("setter" if setter else "creator"),
                                  [self.getLocation(p, 2)])
            argType = arguments[0].type
            if argType == BuiltinTypes[IDLBuiltinType.Types.domstring]:
                specialType = IDLMethod.NamedOrIndexed.Named
            elif argType == BuiltinTypes[IDLBuiltinType.Types.unsigned_long]:
                specialType = IDLMethod.NamedOrIndexed.Indexed
            else:
                raise WebIDLError("%s has wrong argument type (must be DOMString or UnsignedLong)" %
                                  ("setter" if setter else "creator"),
                                  [arguments[0].location])
            if arguments[0].optional or arguments[0].variadic:
                raise WebIDLError("%s cannot have %s argument" %
                                  ("setter" if setter else "creator",
                                   "optional" if arguments[0].optional else "variadic"),
                                  [arguments[0].location])
            if arguments[1].optional or arguments[1].variadic:
                raise WebIDLError("%s cannot have %s argument" %
                                  ("setter" if setter else "creator",
                                   "optional" if arguments[1].optional else "variadic"),
                                  [arguments[1].location])

        if stringifier:
            if len(arguments) != 0:
                raise WebIDLError("stringifier has wrong number of arguments",
                                  [self.getLocation(p, 2)])
            if not returnType.isString():
                raise WebIDLError("stringifier must have string return type",
                                  [self.getLocation(p, 2)])

        inOptionalArguments = False
        variadicArgument = False
        for argument in arguments:
            # Only the last argument can be variadic
            if variadicArgument:
                raise WebIDLError("Only the last argument can be variadic",
                                  [variadicArgument.location])
            # Once we see an optional argument, there can't be any non-optional
            # arguments.
            if inOptionalArguments and not argument.optional:
                raise WebIDLError("Cannot have a non-optional argument following an optional argument",
                                  [argument.location])
            inOptionalArguments = argument.optional
            variadicArgument = argument if argument.variadic else None

        # identifier might be None.  This is only permitted for special methods.
        if not identifier:
            if not getter and not setter and not creator and \
               not deleter and not legacycaller and not stringifier:
                raise WebIDLError("Identifier required for non-special methods",
                                  [self.getLocation(p, 2)])

            location = BuiltinLocation("<auto-generated-identifier>")
            identifier = IDLUnresolvedIdentifier(location, "__%s%s%s%s%s%s%s" %
                ("named" if specialType == IDLMethod.NamedOrIndexed.Named else \
                 "indexed" if specialType == IDLMethod.NamedOrIndexed.Indexed else "",
                 "getter" if getter else "",
                 "setter" if setter else "",
                 "deleter" if deleter else "",
                 "creator" if creator else "",
                 "legacycaller" if legacycaller else "",
                 "stringifier" if stringifier else ""), allowDoubleUnderscore=True)

        method = IDLMethod(self.getLocation(p, 2), identifier, returnType, arguments,
                           static=static, getter=getter, setter=setter, creator=creator,
                           deleter=deleter, specialType=specialType,
                           legacycaller=legacycaller, stringifier=stringifier)
        p[0] = method

    def p_QualifiersStatic(self, p):
        """
            Qualifiers : STATIC
        """
        p[0] = [IDLMethod.Special.Static]

    def p_QualifiersSpecials(self, p):
        """
            Qualifiers : Specials
        """
        p[0] = p[1]

    def p_Specials(self, p):
        """
            Specials : Special Specials
        """
        p[0] = [p[1]]
        p[0].extend(p[2])

    def p_SpecialsEmpty(self, p):
        """
            Specials :
        """
        p[0] = []

    def p_SpecialGetter(self, p):
        """
            Special : GETTER
        """
        p[0] = IDLMethod.Special.Getter

    def p_SpecialSetter(self, p):
        """
            Special : SETTER
        """
        p[0] = IDLMethod.Special.Setter

    def p_SpecialCreator(self, p):
        """
            Special : CREATOR
        """
        p[0] = IDLMethod.Special.Creator

    def p_SpecialDeleter(self, p):
        """
            Special : DELETER
        """
        p[0] = IDLMethod.Special.Deleter

    def p_SpecialLegacyCaller(self, p):
        """
            Special : LEGACYCALLER
        """
        p[0] = IDLMethod.Special.LegacyCaller

    def p_SpecialStringifier(self, p):
        """
            Special : STRINGIFIER
        """
        p[0] = IDLMethod.Special.Stringifier

    def p_OperationRest(self, p):
        """
            OperationRest : ReturnType OptionalIdentifier LPAREN ArgumentList RPAREN SEMICOLON
        """
        p[0] = (p[1], p[2], p[4])

    def p_OptionalIdentifier(self, p):
        """
            OptionalIdentifier : IDENTIFIER
        """
        p[0] = IDLUnresolvedIdentifier(self.getLocation(p, 1), p[1])

    def p_OptionalIdentifierEmpty(self, p):
        """
            OptionalIdentifier :
        """
        pass

    def p_ArgumentList(self, p):
        """
            ArgumentList : Argument Arguments
        """
        p[0] = [p[1]] if p[1] else []
        p[0].extend(p[2])

    def p_ArgumentListEmpty(self, p):
        """
            ArgumentList :
        """
        p[0] = []

    def p_Arguments(self, p):
        """
            Arguments : COMMA Argument Arguments
        """
        p[0] = [p[2]] if p[2] else []
        p[0].extend(p[3])

    def p_ArgumentsEmpty(self, p):
        """
            Arguments :
        """
        p[0] = []

    def p_Argument(self, p):
        """
            Argument : ExtendedAttributeList Optional Type Ellipsis IDENTIFIER DefaultValue
        """
        t = p[3]
        assert isinstance(t, IDLType)
        identifier = IDLUnresolvedIdentifier(self.getLocation(p, 5), p[5])

        optional = p[2]
        variadic = p[4]
        defaultValue = p[6]

        if not optional and defaultValue:
            raise WebIDLError("Mandatory arguments can't have a default value.",
                              [self.getLocation(p, 6)])

        if variadic:
            if optional:
                raise WebIDLError("Variadic arguments should not be marked optional.",
                                  [self.getLocation(p, 2)])
            optional = variadic

        p[0] = IDLArgument(self.getLocation(p, 5), identifier, t, optional, defaultValue, variadic)
        p[0].addExtendedAttributes(p[1])

    def p_Optional(self, p):
        """
            Optional : OPTIONAL
        """
        p[0] = True

    def p_OptionalEmpty(self, p):
        """
            Optional :
        """
        p[0] = False

    def p_Ellipsis(self, p):
        """
            Ellipsis : ELLIPSIS
        """
        p[0] = True

    def p_EllipsisEmpty(self, p):
        """
            Ellipsis :
        """
        p[0] = False

    def p_ExceptionMember(self, p):
        """
            ExceptionMember : Const
                            | ExceptionField
        """
        pass

    def p_ExceptionField(self, p):
        """
            ExceptionField : Type IDENTIFIER SEMICOLON
        """
        pass

    def p_ExtendedAttributeList(self, p):
        """
            ExtendedAttributeList : LBRACKET ExtendedAttribute ExtendedAttributes RBRACKET
        """
        p[0] = [p[2]]
        if p[3]:
            p[0].extend(p[3])

    def p_ExtendedAttributeListEmpty(self, p):
        """
            ExtendedAttributeList :
        """
        p[0] = []

    def p_ExtendedAttribute(self, p):
        """
            ExtendedAttribute : ExtendedAttributeNoArgs
                              | ExtendedAttributeArgList
                              | ExtendedAttributeIdent
                              | ExtendedAttributeNamedArgList
        """
        p[0] = IDLExtendedAttribute(self.getLocation(p, 1), p[1])

    def p_ExtendedAttributeEmpty(self, p):
        """
            ExtendedAttribute :
        """
        pass

    def p_ExtendedAttributes(self, p):
        """
            ExtendedAttributes : COMMA ExtendedAttribute ExtendedAttributes
        """
        p[0] = [p[2]] if p[2] else []
        p[0].extend(p[3])

    def p_ExtendedAttributesEmpty(self, p):
        """
            ExtendedAttributes :
        """
        p[0] = []

    def p_Other(self, p):
        """
            Other : INTEGER
                  | FLOATLITERAL
                  | IDENTIFIER
                  | STRING
                  | OTHER
                  | ELLIPSIS
                  | COLON
                  | SCOPE
                  | SEMICOLON
                  | LT
                  | EQUALS
                  | GT
                  | QUESTIONMARK
                  | DATE
                  | DOMSTRING
                  | ANY
                  | ATTRIBUTE
                  | BOOLEAN
                  | BYTE
                  | LEGACYCALLER
                  | CONST
                  | CREATOR
                  | DELETER
                  | DOUBLE
                  | EXCEPTION
                  | FALSE
                  | FLOAT
                  | GETTER
                  | IMPLEMENTS
                  | INHERIT
                  | INTERFACE
                  | LONG
                  | MODULE
                  | NULL
                  | OBJECT
                  | OCTET
                  | OPTIONAL
                  | SEQUENCE
                  | SETTER
                  | SHORT
                  | STATIC
                  | STRINGIFIER
                  | TRUE
                  | TYPEDEF
                  | UNSIGNED
                  | VOID
        """
        pass

    def p_OtherOrComma(self, p):
        """
            OtherOrComma : Other
                         | COMMA
        """
        pass

    def p_TypeSingleType(self, p):
        """
            Type : SingleType
        """
        p[0] = p[1]

    def p_TypeUnionType(self, p):
        """
            Type : UnionType TypeSuffix
        """
        p[0] = self.handleModifiers(p[1], p[2])

    def p_SingleTypeNonAnyType(self, p):
        """
            SingleType : NonAnyType
        """
        p[0] = p[1]

    def p_SingleTypeAnyType(self, p):
        """
            SingleType : ANY TypeSuffixStartingWithArray
        """
        p[0] = self.handleModifiers(BuiltinTypes[IDLBuiltinType.Types.any], p[2])

    def p_UnionType(self, p):
        """
            UnionType : LPAREN UnionMemberType OR UnionMemberType UnionMemberTypes RPAREN
        """
        types = [p[2], p[4]]
        types.extend(p[5])
        p[0] = IDLUnionType(self.getLocation(p, 1), types)

    def p_UnionMemberTypeNonAnyType(self, p):
        """
            UnionMemberType : NonAnyType
        """
        p[0] = p[1]

    def p_UnionMemberTypeArrayOfAny(self, p):
        """
            UnionMemberTypeArrayOfAny : ANY LBRACKET RBRACKET
        """
        p[0] = IDLArrayType(self.getLocation(p, 2),
                            BuiltinTypes[IDLBuiltinType.Types.any])

    def p_UnionMemberType(self, p):
        """
            UnionMemberType : UnionType TypeSuffix
                            | UnionMemberTypeArrayOfAny TypeSuffix
        """
        p[0] = self.handleModifiers(p[1], p[2])

    def p_UnionMemberTypes(self, p):
        """
            UnionMemberTypes : OR UnionMemberType UnionMemberTypes
        """
        p[0] = [p[2]]
        p[0].extend(p[3])

    def p_UnionMemberTypesEmpty(self, p):
        """
            UnionMemberTypes : 
        """
        p[0] = []

    def p_NonAnyType(self, p):
        """
            NonAnyType : PrimitiveOrStringType TypeSuffix
                       | ARRAYBUFFER TypeSuffix
                       | OBJECT TypeSuffix
        """
        if p[1] == "object":
            type = BuiltinTypes[IDLBuiltinType.Types.object]
        elif p[1] == "ArrayBuffer":
            type = BuiltinTypes[IDLBuiltinType.Types.ArrayBuffer]
        else:
            type = BuiltinTypes[p[1]]

        p[0] = self.handleModifiers(type, p[2])

    def p_NonAnyTypeSequenceType(self, p):
        """
            NonAnyType : SEQUENCE LT Type GT Null
        """
        innerType = p[3]
        type = IDLSequenceType(self.getLocation(p, 1), innerType)
        if p[5]:
            type = IDLNullableType(self.getLocation(p, 5), type)
        p[0] = type

    def p_NonAnyTypeScopedName(self, p):
        """
            NonAnyType : ScopedName TypeSuffix
        """
        assert isinstance(p[1], IDLUnresolvedIdentifier)

        type = None

        try:
            if self.globalScope()._lookupIdentifier(p[1]):
                obj = self.globalScope()._lookupIdentifier(p[1])
                if obj.isType():
                    type = obj
                else:
                    type = IDLWrapperType(self.getLocation(p, 1), p[1])
                p[0] = self.handleModifiers(type, p[2])
                return
        except:
            pass

        type = IDLUnresolvedType(self.getLocation(p, 1), p[1])
        p[0] = self.handleModifiers(type, p[2])

    def p_NonAnyTypeDate(self, p):
        """
            NonAnyType : DATE TypeSuffix
        """
        assert False
        pass

    def p_ConstType(self, p):
        """
            ConstType : PrimitiveOrStringType Null
        """
        type = BuiltinTypes[p[1]]
        if p[2]:
            type = IDLNullableType(self.getLocation(p, 1), type)
        p[0] = type

    def p_ConstTypeIdentifier(self, p):
        """
            ConstType : IDENTIFIER Null
        """
        identifier = IDLUnresolvedIdentifier(self.getLocation(p, 1), p[1])

        type = IDLUnresolvedType(self.getLocation(p, 1), identifier)
        if p[2]:
            type = IDLNullableType(self.getLocation(p, 1), type)
        p[0] = type

    def p_PrimitiveOrStringTypeUint(self, p):
        """
            PrimitiveOrStringType : UnsignedIntegerType
        """
        p[0] = p[1]

    def p_PrimitiveOrStringTypeBoolean(self, p):
        """
            PrimitiveOrStringType : BOOLEAN
        """
        p[0] = IDLBuiltinType.Types.boolean

    def p_PrimitiveOrStringTypeByte(self, p):
        """
            PrimitiveOrStringType : BYTE
        """
        p[0] = IDLBuiltinType.Types.byte

    def p_PrimitiveOrStringTypeOctet(self, p):
        """
            PrimitiveOrStringType : OCTET
        """
        p[0] = IDLBuiltinType.Types.octet

    def p_PrimitiveOrStringTypeFloat(self, p):
        """
            PrimitiveOrStringType : FLOAT
        """
        p[0] = IDLBuiltinType.Types.float

    def p_PrimitiveOrStringTypeDouble(self, p):
        """
            PrimitiveOrStringType : DOUBLE
        """
        p[0] = IDLBuiltinType.Types.double

    def p_PrimitiveOrStringTypeDOMString(self, p):
        """
            PrimitiveOrStringType : DOMSTRING
        """
        p[0] = IDLBuiltinType.Types.domstring

    def p_UnsignedIntegerTypeUnsigned(self, p):
        """
            UnsignedIntegerType : UNSIGNED IntegerType
        """
        p[0] = p[2] + 1 # Adding one to a given signed integer type
                        # gets you the unsigned type.

    def p_UnsignedIntegerType(self, p):
        """
            UnsignedIntegerType : IntegerType
        """
        p[0] = p[1]

    def p_IntegerTypeShort(self, p):
        """
            IntegerType : SHORT
        """
        p[0] = IDLBuiltinType.Types.short

    def p_IntegerTypeLong(self, p):
        """
            IntegerType : LONG OptionalLong
        """
        if p[2]:
            p[0] = IDLBuiltinType.Types.long_long
        else:
            p[0] = IDLBuiltinType.Types.long

    def p_OptionalLong(self, p):
        """
            OptionalLong : LONG
        """
        p[0] = True

    def p_OptionalLongEmpty(self, p):
        """
            OptionalLong :
        """
        p[0] = False

    def p_TypeSuffixBrackets(self, p):
        """
            TypeSuffix : LBRACKET RBRACKET TypeSuffix
        """
        p[0] = [(IDLMethod.TypeSuffixModifier.Brackets, self.getLocation(p, 1))]
        p[0].extend(p[3])

    def p_TypeSuffixQMark(self, p):
        """
            TypeSuffix : QUESTIONMARK TypeSuffixStartingWithArray
        """
        p[0] = [(IDLMethod.TypeSuffixModifier.QMark, self.getLocation(p, 1))]
        p[0].extend(p[2])

    def p_TypeSuffixEmpty(self, p):
        """
            TypeSuffix :
        """
        p[0] = []

    def p_TypeSuffixStartingWithArray(self, p):
        """
            TypeSuffixStartingWithArray : LBRACKET RBRACKET TypeSuffix
        """
        p[0] = [(IDLMethod.TypeSuffixModifier.Brackets, self.getLocation(p, 1))]
        p[0].extend(p[3])

    def p_TypeSuffixStartingWithArrayEmpty(self, p):
        """
            TypeSuffixStartingWithArray :
        """
        p[0] = []

    def p_Null(self, p):
        """
            Null : QUESTIONMARK
                 |
        """
        if len(p) > 1:
            p[0] = True
        else:
            p[0] = False

    def p_ReturnTypeType(self, p):
        """
            ReturnType : Type
        """
        p[0] = p[1]

    def p_ReturnTypeVoid(self, p):
        """
            ReturnType : VOID
        """
        p[0] = BuiltinTypes[IDLBuiltinType.Types.void]

    def p_ScopedName(self, p):
        """
            ScopedName : AbsoluteScopedName
                       | RelativeScopedName
        """
        p[0] = p[1]

    def p_AbsoluteScopedName(self, p):
        """
            AbsoluteScopedName : SCOPE IDENTIFIER ScopedNameParts
        """
        assert False
        pass

    def p_RelativeScopedName(self, p):
        """
            RelativeScopedName : IDENTIFIER ScopedNameParts
        """
        assert not p[2] # Not implemented!

        p[0] = IDLUnresolvedIdentifier(self.getLocation(p, 1), p[1])

    def p_ScopedNameParts(self, p):
        """
            ScopedNameParts : SCOPE IDENTIFIER ScopedNameParts
        """
        assert False
        pass

    def p_ScopedNamePartsEmpty(self, p):
        """
            ScopedNameParts :
        """
        p[0] = None

    def p_ExtendedAttributeNoArgs(self, p):
        """
            ExtendedAttributeNoArgs : IDENTIFIER
        """
        p[0] = (p[1],)

    def p_ExtendedAttributeArgList(self, p):
        """
            ExtendedAttributeArgList : IDENTIFIER LPAREN ArgumentList RPAREN
        """
        p[0] = (p[1], p[3])

    def p_ExtendedAttributeIdent(self, p):
        """
            ExtendedAttributeIdent : IDENTIFIER EQUALS STRING
                                   | IDENTIFIER EQUALS IDENTIFIER
        """
        p[0] = (p[1], p[3])

    def p_ExtendedAttributeNamedArgList(self, p):
        """
            ExtendedAttributeNamedArgList : IDENTIFIER EQUALS IDENTIFIER LPAREN ArgumentList RPAREN
        """
        p[0] = (p[1], p[3], p[5])

    def p_error(self, p):
        if not p:
            raise WebIDLError("Syntax Error at end of file. Possibly due to missing semicolon(;), braces(}) or both", [])
        else:
            raise WebIDLError("invalid syntax", [Location(self.lexer, p.lineno, p.lexpos, self._filename)])

    def __init__(self, outputdir='', lexer=None):
        Tokenizer.__init__(self, outputdir, lexer)
        self.parser = yacc.yacc(module=self,
                                outputdir=outputdir,
                                tabmodule='webidlyacc',
                                errorlog=yacc.NullLogger(),
                                picklefile='WebIDLGrammar.pkl')
        self._globalScope = IDLScope(BuiltinLocation("<Global Scope>"), None, None)
        self._installBuiltins(self._globalScope)
        self._productions = []

        self._filename = "<builtin>"
        self.lexer.input(Parser._builtins)
        self._filename = None

        self.parser.parse(lexer=self.lexer,tracking=True)

    def _installBuiltins(self, scope):
        assert isinstance(scope, IDLScope)

        # xrange omits the last value.
        for x in xrange(IDLBuiltinType.Types.ArrayBuffer, IDLBuiltinType.Types.Float64Array + 1):
            builtin = BuiltinTypes[x]
            name = builtin.name

            typedef = IDLTypedefType(BuiltinLocation("<builtin type>"), builtin, name)
            typedef.resolve(scope)

    @ staticmethod
    def handleModifiers(type, modifiers):
        for (modifier, modifierLocation) in modifiers:
            assert modifier == IDLMethod.TypeSuffixModifier.QMark or \
                   modifier == IDLMethod.TypeSuffixModifier.Brackets

            if modifier == IDLMethod.TypeSuffixModifier.QMark:
                type = IDLNullableType(modifierLocation, type)
            elif modifier == IDLMethod.TypeSuffixModifier.Brackets:
                type = IDLArrayType(modifierLocation, type)

        return type

    def parse(self, t, filename=None):
        self.lexer.input(t)

        #for tok in iter(self.lexer.token, None):
        #    print tok

        self._filename = filename
        self._productions.extend(self.parser.parse(lexer=self.lexer,tracking=True))
        self._filename = None

    def finish(self):
        # First, finish all the IDLImplementsStatements.  In particular, we
        # have to make sure we do those before we do the IDLInterfaces.
        # XXX khuey hates this bit and wants to nuke it from orbit.
        implementsStatements = [ p for p in self._productions if
                                 isinstance(p, IDLImplementsStatement)]
        otherStatements = [ p for p in self._productions if
                            not isinstance(p, IDLImplementsStatement)]
        for production in implementsStatements:
            production.finish(self.globalScope())
        for production in otherStatements:
            production.finish(self.globalScope())

        # Do any post-finish validation we need to do
        for production in self._productions:
            production.validate()

        # De-duplicate self._productions, without modifying its order.
        seen = set()
        result = []
        for p in self._productions:
            if p not in seen:
                seen.add(p)
                result.append(p)
        return result

    def reset(self):
        return Parser(lexer=self.lexer)

    # Builtin IDL defined by WebIDL
    _builtins = """
        typedef unsigned long long DOMTimeStamp;
    """

def main():
    # Parse arguments.
    from optparse import OptionParser
    usageString = "usage: %prog [options] files"
    o = OptionParser(usage=usageString)
    o.add_option("--cachedir", dest='cachedir', default=None,
                 help="Directory in which to cache lex/parse tables.")
    o.add_option("--verbose-errors", action='store_true', default=False,
                 help="When an error happens, display the Python traceback.")
    (options, args) = o.parse_args()

    if len(args) < 1:
        o.error(usageString)

    fileList = args
    baseDir = os.getcwd()

    # Parse the WebIDL.
    parser = Parser(options.cachedir)
    try:
        for filename in fileList:
            fullPath = os.path.normpath(os.path.join(baseDir, filename))
            f = open(fullPath, 'rb')
            lines = f.readlines()
            f.close()
            print fullPath
            parser.parse(''.join(lines), fullPath)
        parser.finish()
    except WebIDLError, e:
        if options.verbose_errors:
            traceback.print_exc()
        else:
            print e

if __name__ == '__main__':
    main()
