# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

""" A WebIDL parser. """

from ply import lex, yacc
import re
import os
import traceback
import math
from collections import defaultdict

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
def M_add_class_attribs(attribs, start):
    def foo(name, bases, dict_):
        for v, k in enumerate(attribs):
            dict_[k] = start + v
        assert 'length' not in dict_
        dict_['length'] = start + len(attribs)
        return type(name, bases, dict_)
    return foo


def enum(*names, **kw):
    if len(kw) == 1:
        base = kw['base'].__class__
        start = base.length
    else:
        assert len(kw) == 0
        base = object
        start = 0

    class Foo(base):
        __metaclass__ = M_add_class_attribs(names, start)

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
        return (self._lexpos == other._lexpos and
                self._file == other._file)

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
        return (isinstance(other, BuiltinLocation) and
                self.msg == other.msg)

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
        return False

    def isUnion(self):
        return False

    def isTypedef(self):
        return False

    def getUserData(self, key, default):
        return self.userData.get(key, default)

    def setUserData(self, key, value):
        self.userData[key] = value

    def addExtendedAttributes(self, attrs):
        assert False  # Override me!

    def handleExtendedAttribute(self, attr):
        assert False  # Override me!

    def _getDependentObjects(self):
        assert False  # Override me!

    def getDeps(self, visited=None):
        """ Return a set of files that this object depends on.  If any of
            these files are changed the parser needs to be rerun to regenerate
            a new IDLObject.

            The visited argument is a set of all the objects already visited.
            We must test to see if we are in it, and if so, do nothing.  This
            prevents infinite recursion."""

        # NB: We can't use visited=set() above because the default value is
        # evaluated when the def statement is evaluated, not when the function
        # is executed, so there would be one set for all invocations.
        if visited is None:
            visited = set()

        if self in visited:
            return set()

        visited.add(self)

        deps = set()
        if self.filename() != "<builtin>":
            deps.add(self.filename())

        for d in self._getDependentObjects():
            deps.update(d.getDeps(visited))

        return deps


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
        self.globalNames = set()
        # A mapping from global name to the set of global interfaces
        # that have that global name.
        self.globalNameMapping = defaultdict(set)
        self.primaryGlobalAttr = None
        self.primaryGlobalName = None

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
            assert id(object) != id(self._dict[identifier.name])

            replacement = self.resolveIdentifierConflict(self, identifier,
                                                         self._dict[identifier.name],
                                                         object)
            self._dict[identifier.name] = replacement
            return

        assert object

        self._dict[identifier.name] = object

    def resolveIdentifierConflict(self, scope, identifier, originalObject, newObject):
        if (isinstance(originalObject, IDLExternalInterface) and
            isinstance(newObject, IDLExternalInterface) and
            originalObject.identifier.name == newObject.identifier.name):
            return originalObject

        if (isinstance(originalObject, IDLExternalInterface) or
            isinstance(newObject, IDLExternalInterface)):
            raise WebIDLError(
                "Name collision between "
                "interface declarations for identifier '%s' at '%s' and '%s'"
                % (identifier.name,
                    originalObject.location, newObject.location), [])

        if (isinstance(originalObject, IDLDictionary) or
            isinstance(newObject, IDLDictionary)):
            raise WebIDLError(
                "Name collision between dictionary declarations for "
                "identifier '%s'.\n%s\n%s"
                % (identifier.name,
                   originalObject.location, newObject.location), [])

        # We do the merging of overloads here as opposed to in IDLInterface
        # because we need to merge overloads of NamedConstructors and we need to
        # detect conflicts in those across interfaces. See also the comment in
        # IDLInterface.addExtendedAttributes for "NamedConstructor".
        if (originalObject.tag == IDLInterfaceMember.Tags.Method and
           newObject.tag == IDLInterfaceMember.Tags.Method):
            return originalObject.addOverload(newObject)

        # Default to throwing, derived classes can override.
        conflictdesc = "\n\t%s at %s\n\t%s at %s" % (originalObject,
                                                     originalObject.location,
                                                     newObject,
                                                     newObject.location)

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
    def __init__(self, location, name, allowDoubleUnderscore=False,
                 allowForbidden=False):
        IDLObject.__init__(self, location)

        assert len(name) > 0

        if name == "__noSuchMethod__":
            raise WebIDLError("__noSuchMethod__ is deprecated", [location])

        if name[:2] == "__" and name != "__content" and not allowDoubleUnderscore:
            raise WebIDLError("Identifiers beginning with __ are reserved",
                              [location])
        if name[0] == '_' and not allowDoubleUnderscore:
            name = name[1:]
        # TODO: Bug 872377, Restore "toJSON" to below list.
        # We sometimes need custom serialization, so allow toJSON for now.
        if (name in ["constructor", "toString"] and
            not allowForbidden):
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
        assert False  # Should replace with a resolved identifier first.


class IDLObjectWithIdentifier(IDLObject):
    def __init__(self, location, parentScope, identifier):
        IDLObject.__init__(self, location)

        assert isinstance(identifier, IDLUnresolvedIdentifier)

        self.identifier = identifier

        if parentScope:
            self.resolve(parentScope)

        self.treatNullAs = "Default"

    def resolve(self, parentScope):
        assert isinstance(parentScope, IDLScope)
        assert isinstance(self.identifier, IDLUnresolvedIdentifier)
        self.identifier.resolve(parentScope, self)

    def checkForStringHandlingExtendedAttributes(self, attrs,
                                                 isDictionaryMember=False,
                                                 isOptional=False):
        """
        A helper function to deal with TreatNullAs.  Returns the list
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
                if not self.type.isDOMString() or self.type.nullable():
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


class IDLExposureMixins():
    def __init__(self, location):
        # _exposureGlobalNames are the global names listed in our [Exposed]
        # extended attribute.  exposureSet is the exposure set as defined in the
        # Web IDL spec: it contains interface names.
        self._exposureGlobalNames = set()
        self.exposureSet = set()
        self._location = location
        self._globalScope = None

    def finish(self, scope):
        assert scope.parentScope is None
        self._globalScope = scope

        # Verify that our [Exposed] value, if any, makes sense.
        for globalName in self._exposureGlobalNames:
            if globalName not in scope.globalNames:
                raise WebIDLError("Unknown [Exposed] value %s" % globalName,
                                  [self._location])

        if len(self._exposureGlobalNames) == 0:
            self._exposureGlobalNames.add(scope.primaryGlobalName)

        globalNameSetToExposureSet(scope, self._exposureGlobalNames,
                                   self.exposureSet)

    def isExposedInWindow(self):
        return 'Window' in self.exposureSet

    def isExposedInAnyWorker(self):
        return len(self.getWorkerExposureSet()) > 0

    def isExposedInSystemGlobals(self):
        return 'BackstagePass' in self.exposureSet

    def isExposedInSomeButNotAllWorkers(self):
        """
        Returns true if the Exposed extended attribute for this interface
        exposes it in some worker globals but not others.  The return value does
        not depend on whether the interface is exposed in Window or System
        globals.
        """
        if not self.isExposedInAnyWorker():
            return False
        workerScopes = self.parentScope.globalNameMapping["Worker"]
        return len(workerScopes.difference(self.exposureSet)) > 0

    def getWorkerExposureSet(self):
        workerScopes = self._globalScope.globalNameMapping["Worker"]
        return workerScopes.intersection(self.exposureSet)


class IDLExternalInterface(IDLObjectWithIdentifier, IDLExposureMixins):
    def __init__(self, location, parentScope, identifier):
        raise WebIDLError("Servo does not support external interfaces.",
                          [self.location])


class IDLPartialInterface(IDLObject):
    def __init__(self, location, name, members, nonPartialInterface):
        assert isinstance(name, IDLUnresolvedIdentifier)

        IDLObject.__init__(self, location)
        self.identifier = name
        self.members = members
        # propagatedExtendedAttrs are the ones that should get
        # propagated to our non-partial interface.
        self.propagatedExtendedAttrs = []
        self._nonPartialInterface = nonPartialInterface
        self._finished = False
        nonPartialInterface.addPartialInterface(self)

    def addExtendedAttributes(self, attrs):
        for attr in attrs:
            identifier = attr.identifier()

            if identifier in ["Constructor", "NamedConstructor"]:
                self.propagatedExtendedAttrs.append(attr)
            elif identifier == "Exposed":
                # This just gets propagated to all our members.
                for member in self.members:
                    if len(member._exposureGlobalNames) != 0:
                        raise WebIDLError("[Exposed] specified on both a "
                                          "partial interface member and on the "
                                          "partial interface itself",
                                          [member.location, attr.location])
                    member.addExtendedAttributes([attr])
            else:
                raise WebIDLError("Unknown extended attribute %s on partial "
                                  "interface" % identifier,
                                  [attr.location])

    def finish(self, scope):
        if self._finished:
            return
        self._finished = True
        # Need to make sure our non-partial interface gets finished so it can
        # report cases when we only have partial interfaces.
        self._nonPartialInterface.finish(scope)

    def validate(self):
        pass


def convertExposedAttrToGlobalNameSet(exposedAttr, targetSet):
    assert len(targetSet) == 0
    if exposedAttr.hasValue():
        targetSet.add(exposedAttr.value())
    else:
        assert exposedAttr.hasArgs()
        targetSet.update(exposedAttr.args())


def globalNameSetToExposureSet(globalScope, nameSet, exposureSet):
    for name in nameSet:
        exposureSet.update(globalScope.globalNameMapping[name])


class IDLInterface(IDLObjectWithScope, IDLExposureMixins):
    def __init__(self, location, parentScope, name, parent, members,
                 isKnownNonPartial):
        assert isinstance(parentScope, IDLScope)
        assert isinstance(name, IDLUnresolvedIdentifier)
        assert isKnownNonPartial or not parent
        assert isKnownNonPartial or len(members) == 0

        self.parent = None
        self._callback = False
        self._finished = False
        self.members = []
        self.maplikeOrSetlike = None
        self._partialInterfaces = []
        self._extendedAttrDict = {}
        # namedConstructors needs deterministic ordering because bindings code
        # outputs the constructs in the order that namedConstructors enumerates
        # them.
        self.namedConstructors = list()
        self.implementedInterfaces = set()
        self._consequential = False
        self._isKnownNonPartial = False
        # self.interfacesBasedOnSelf is the set of interfaces that inherit from
        # self or have self as a consequential interface, including self itself.
        # Used for distinguishability checking.
        self.interfacesBasedOnSelf = set([self])
        # self.interfacesImplementingSelf is the set of interfaces that directly
        # have self as a consequential interface
        self.interfacesImplementingSelf = set()
        self._hasChildInterfaces = False
        self._isOnGlobalProtoChain = False
        # Tracking of the number of reserved slots we need for our
        # members and those of ancestor interfaces.
        self.totalMembersInSlots = 0
        # Tracking of the number of own own members we have in slots
        self._ownMembersInSlots = 0

        IDLObjectWithScope.__init__(self, location, parentScope, name)
        IDLExposureMixins.__init__(self, location)

        if isKnownNonPartial:
            self.setNonPartial(location, parent, members)

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

        retval = IDLScope.resolveIdentifierConflict(self, scope, identifier,
                                                    originalObject, newObject)

        # Might be a ctor, which isn't in self.members
        if newObject in self.members:
            self.members.remove(newObject)
        return retval

    def finish(self, scope):
        if self._finished:
            return

        self._finished = True

        if not self._isKnownNonPartial:
            raise WebIDLError("Interface %s does not have a non-partial "
                              "declaration" % self.identifier.name,
                              [self.location])

        IDLExposureMixins.finish(self, scope)

        # Now go ahead and merge in our partial interfaces.
        for partial in self._partialInterfaces:
            partial.finish(scope)
            self.addExtendedAttributes(partial.propagatedExtendedAttrs)
            self.members.extend(partial.members)

        # Generate maplike/setlike interface members. Since generated members
        # need to be treated like regular interface members, do this before
        # things like exposure setting.
        for member in self.members:
            if member.isMaplikeOrSetlike():
                # Check that we only have one interface declaration (currently
                # there can only be one maplike/setlike declaration per
                # interface)
                if self.maplikeOrSetlike:
                    raise WebIDLError("%s declaration used on "
                                      "interface that already has %s "
                                      "declaration" %
                                      (member.maplikeOrSetlikeType,
                                       self.maplikeOrSetlike.maplikeOrSetlikeType),
                                      [self.maplikeOrSetlike.location,
                                       member.location])
                self.maplikeOrSetlike = member
                # If we've got a maplike or setlike declaration, we'll be building all of
                # our required methods in Codegen. Generate members now.
                self.maplikeOrSetlike.expand(self.members, self.isJSImplemented())

        # Now that we've merged in our partial interfaces, set the
        # _exposureGlobalNames on any members that don't have it set yet.  Note
        # that any partial interfaces that had [Exposed] set have already set up
        # _exposureGlobalNames on all the members coming from them, so this is
        # just implementing the "members default to interface that defined them"
        # and "partial interfaces default to interface they're a partial for"
        # rules from the spec.
        for m in self.members:
            # If m, or the partial interface m came from, had [Exposed]
            # specified, it already has a nonempty exposure global names set.
            if len(m._exposureGlobalNames) == 0:
                m._exposureGlobalNames.update(self._exposureGlobalNames)

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
            self.parent._hasChildInterfaces = True

            self.totalMembersInSlots = self.parent.totalMembersInSlots

            # Interfaces with [Global] or [PrimaryGlobal] must not
            # have anything inherit from them
            if (self.parent.getExtendedAttribute("Global") or
                self.parent.getExtendedAttribute("PrimaryGlobal")):
                # Note: This is not a self.parent.isOnGlobalProtoChain() check
                # because ancestors of a [Global] interface can have other
                # descendants.
                raise WebIDLError("[Global] interface has another interface "
                                  "inheriting from it",
                                  [self.location, self.parent.location])

            # Make sure that we're not exposed in places where our parent is not
            if not self.exposureSet.issubset(self.parent.exposureSet):
                raise WebIDLError("Interface %s is exposed in globals where its "
                                  "parent interface %s is not exposed." %
                                  (self.identifier.name,
                                   self.parent.identifier.name),
                                  [self.location, self.parent.location])

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

            # Interfaces which have interface objects can't inherit
            # from [NoInterfaceObject] interfaces.
            if (self.parent.getExtendedAttribute("NoInterfaceObject") and
                not self.getExtendedAttribute("NoInterfaceObject")):
                raise WebIDLError("Interface %s does not have "
                                  "[NoInterfaceObject] but inherits from "
                                  "interface %s which does" %
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

        # Now that we've finished our members, which has updated their exposure
        # sets, make sure they aren't exposed in places where we are not.
        for member in self.members:
            if not member.exposureSet.issubset(self.exposureSet):
                raise WebIDLError("Interface member has larger exposure set "
                                  "than the interface itself",
                                  [member.location, self.location])

        ctor = self.ctor()
        if ctor is not None:
            assert len(ctor._exposureGlobalNames) == 0
            ctor._exposureGlobalNames.update(self._exposureGlobalNames)
            ctor.finish(scope)

        for ctor in self.namedConstructors:
            assert len(ctor._exposureGlobalNames) == 0
            ctor._exposureGlobalNames.update(self._exposureGlobalNames)
            ctor.finish(scope)

        # Make a copy of our member list, so things that implement us
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
            # Verify that we're not exposed somewhere where iface is not exposed
            if not self.exposureSet.issubset(iface.exposureSet):
                raise WebIDLError("Interface %s is exposed in globals where its "
                                  "consequential interface %s is not exposed." %
                                  (self.identifier.name, iface.identifier.name),
                                  [self.location, iface.location])

            # If we have a maplike or setlike, and the consequential interface
            # also does, throw an error.
            if iface.maplikeOrSetlike and self.maplikeOrSetlike:
                raise WebIDLError("Maplike/setlike interface %s cannot have "
                                  "maplike/setlike interface %s as a "
                                  "consequential interface" %
                                  (self.identifier.name,
                                   iface.identifier.name),
                                  [self.maplikeOrSetlike.location,
                                   iface.maplikeOrSetlike.location])
            additionalMembers = iface.originalMembers
            for additionalMember in additionalMembers:
                for member in self.members:
                    if additionalMember.identifier.name == member.identifier.name:
                        raise WebIDLError(
                            "Multiple definitions of %s on %s coming from 'implements' statements" %
                            (member.identifier.name, self),
                            [additionalMember.location, member.location])
            self.members.extend(additionalMembers)
            iface.interfacesImplementingSelf.add(self)

        for ancestor in self.getInheritedInterfaces():
            ancestor.interfacesBasedOnSelf.add(self)
            if (ancestor.maplikeOrSetlike is not None and
                self.maplikeOrSetlike is not None):
                raise WebIDLError("Cannot have maplike/setlike on %s that "
                                  "inherits %s, which is already "
                                  "maplike/setlike" %
                                  (self.identifier.name,
                                   ancestor.identifier.name),
                                  [self.maplikeOrSetlike.location,
                                   ancestor.maplikeOrSetlike.location])
            for ancestorConsequential in ancestor.getConsequentialInterfaces():
                ancestorConsequential.interfacesBasedOnSelf.add(self)

        # Deal with interfaces marked [Unforgeable], now that we have our full
        # member list, except unforgeables pulled in from parents.  We want to
        # do this before we set "originatingInterface" on our unforgeable
        # members.
        if self.getExtendedAttribute("Unforgeable"):
            # Check that the interface already has all the things the
            # spec would otherwise require us to synthesize and is
            # missing the ones we plan to synthesize.
            if not any(m.isMethod() and m.isStringifier() for m in self.members):
                raise WebIDLError("Unforgeable interface %s does not have a "
                                  "stringifier" % self.identifier.name,
                                  [self.location])

            for m in self.members:
                if ((m.isMethod() and m.isJsonifier()) or
                    m.identifier.name == "toJSON"):
                    raise WebIDLError("Unforgeable interface %s has a "
                                      "jsonifier so we won't be able to add "
                                      "one ourselves" % self.identifier.name,
                                      [self.location, m.location])

                if m.identifier.name == "valueOf" and not m.isStatic():
                    raise WebIDLError("Unforgeable interface %s has a valueOf "
                                      "member so we won't be able to add one "
                                      "ourselves" % self.identifier.name,
                                      [self.location, m.location])

        for member in self.members:
            if ((member.isAttr() or member.isMethod()) and
                member.isUnforgeable() and
                not hasattr(member, "originatingInterface")):
                member.originatingInterface = self

        # Compute slot indices for our members before we pull in unforgeable
        # members from our parent. Also, maplike/setlike declarations get a
        # slot to hold their backing object.
        for member in self.members:
            if ((member.isAttr() and
                 (member.getExtendedAttribute("StoreInSlot") or
                  member.getExtendedAttribute("Cached"))) or
                member.isMaplikeOrSetlike()):
                member.slotIndex = self.totalMembersInSlots
                self.totalMembersInSlots += 1
                if member.getExtendedAttribute("StoreInSlot"):
                    self._ownMembersInSlots += 1

        if self.parent:
            # Make sure we don't shadow any of the [Unforgeable] attributes on
            # our ancestor interfaces.  We don't have to worry about
            # consequential interfaces here, because those have already been
            # imported into the relevant .members lists.  And we don't have to
            # worry about anything other than our parent, because it has already
            # imported its ancestors unforgeable attributes into its member
            # list.
            for unforgeableMember in (member for member in self.parent.members if
                                      (member.isAttr() or member.isMethod()) and
                                      member.isUnforgeable()):
                shadows = [m for m in self.members if
                           (m.isAttr() or m.isMethod()) and
                           not m.isStatic() and
                           m.identifier.name == unforgeableMember.identifier.name]
                if len(shadows) != 0:
                    locs = [unforgeableMember.location] + [s.location for s
                                                           in shadows]
                    raise WebIDLError("Interface %s shadows [Unforgeable] "
                                      "members of %s" %
                                      (self.identifier.name,
                                       ancestor.identifier.name),
                                      locs)
                # And now just stick it in our members, since we won't be
                # inheriting this down the proto chain.  If we really cared we
                # could try to do something where we set up the unforgeable
                # attributes/methods of ancestor interfaces, with their
                # corresponding getters, on our interface, but that gets pretty
                # complicated and seems unnecessary.
                self.members.append(unforgeableMember)

        # At this point, we have all of our members. If the current interface
        # uses maplike/setlike, check for collisions anywhere in the current
        # interface or higher in the inheritance chain.
        if self.maplikeOrSetlike:
            testInterface = self
            isAncestor = False
            while testInterface:
                self.maplikeOrSetlike.checkCollisions(testInterface.members,
                                                      isAncestor)
                isAncestor = True
                testInterface = testInterface.parent

        # Ensure that there's at most one of each {named,indexed}
        # {getter,setter,creator,deleter}, at most one stringifier,
        # and at most one legacycaller.  Note that this last is not
        # quite per spec, but in practice no one overloads
        # legacycallers.
        specialMembersSeen = {}
        for member in self.members:
            if not member.isMethod():
                continue

            if member.isGetter():
                memberType = "getters"
            elif member.isSetter():
                memberType = "setters"
            elif member.isCreator():
                memberType = "creators"
            elif member.isDeleter():
                memberType = "deleters"
            elif member.isStringifier():
                memberType = "stringifiers"
            elif member.isJsonifier():
                memberType = "jsonifiers"
            elif member.isLegacycaller():
                memberType = "legacycallers"
            else:
                continue

            if (memberType != "stringifiers" and memberType != "legacycallers" and
                memberType != "jsonifiers"):
                if member.isNamed():
                    memberType = "named " + memberType
                else:
                    assert member.isIndexed()
                    memberType = "indexed " + memberType

            if memberType in specialMembersSeen:
                raise WebIDLError("Multiple " + memberType + " on %s" % (self),
                                  [self.location,
                                   specialMembersSeen[memberType].location,
                                   member.location])

            specialMembersSeen[memberType] = member

        if self._isOnGlobalProtoChain:
            # Make sure we have no named setters, creators, or deleters
            for memberType in ["setter", "creator", "deleter"]:
                memberId = "named " + memberType + "s"
                if memberId in specialMembersSeen:
                    raise WebIDLError("Interface with [Global] has a named %s" %
                                      memberType,
                                      [self.location,
                                       specialMembersSeen[memberId].location])
            # Make sure we're not [OverrideBuiltins]
            if self.getExtendedAttribute("OverrideBuiltins"):
                raise WebIDLError("Interface with [Global] also has "
                                  "[OverrideBuiltins]",
                                  [self.location])
            # Mark all of our ancestors as being on the global's proto chain too
            parent = self.parent
            while parent:
                # Must not inherit from an interface with [OverrideBuiltins]
                if parent.getExtendedAttribute("OverrideBuiltins"):
                    raise WebIDLError("Interface with [Global] inherits from "
                                      "interface with [OverrideBuiltins]",
                                      [self.location, parent.location])
                parent._isOnGlobalProtoChain = True
                parent = parent.parent

    def validate(self):
        # We don't support consequential unforgeable interfaces.  Need to check
        # this here, becaue in finish() an interface might not know yet that
        # it's consequential.
        if self.getExtendedAttribute("Unforgeable") and self.isConsequential():
            raise WebIDLError(
                "%s is an unforgeable consequential interface" %
                self.identifier.name,
                [self.location] +
                list(i.location for i in
                     (self.interfacesBasedOnSelf - {self})))

        # We also don't support inheriting from unforgeable interfaces.
        if self.getExtendedAttribute("Unforgeable") and self.hasChildInterfaces():
            locations = ([self.location] +
                         list(i.location for i in
                              self.interfacesBasedOnSelf if i.parent == self))
            raise WebIDLError("%s is an unforgeable ancestor interface" %
                              self.identifier.name,
                              locations)

        for member in self.members:
            member.validate()

            if self.isCallback() and member.getExtendedAttribute("Replaceable"):
                raise WebIDLError("[Replaceable] used on an attribute on "
                                  "interface %s which is a callback interface" %
                                  self.identifier.name,
                                  [self.location, member.location])

            # Check that PutForwards refers to another attribute and that no
            # cycles exist in forwarded assignments.
            if member.isAttr():
                iface = self
                attr = member
                putForwards = attr.getExtendedAttribute("PutForwards")
                if putForwards and self.isCallback():
                    raise WebIDLError("[PutForwards] used on an attribute "
                                      "on interface %s which is a callback "
                                      "interface" % self.identifier.name,
                                      [self.location, member.location])

                while putForwards is not None:
                    forwardIface = attr.type.unroll().inner
                    fowardAttr = None

                    for forwardedMember in forwardIface.members:
                        if (not forwardedMember.isAttr() or
                            forwardedMember.identifier.name != putForwards[0]):
                            continue
                        if forwardedMember == member:
                            raise WebIDLError("Cycle detected in forwarded "
                                              "assignments for attribute %s on "
                                              "%s" %
                                              (member.identifier.name, self),
                                              [member.location])
                        fowardAttr = forwardedMember
                        break

                    if fowardAttr is None:
                        raise WebIDLError("Attribute %s on %s forwards to "
                                          "missing attribute %s" %
                                          (attr.identifier.name, iface, putForwards),
                                          [attr.location])

                    iface = forwardIface
                    attr = fowardAttr
                    putForwards = attr.getExtendedAttribute("PutForwards")

            # Check that the name of an [Alias] doesn't conflict with an
            # interface member.
            if member.isMethod():
                for alias in member.aliases:
                    if self.isOnGlobalProtoChain():
                        raise WebIDLError("[Alias] must not be used on a "
                                          "[Global] interface operation",
                                          [member.location])
                    if (member.getExtendedAttribute("Exposed") or
                        member.getExtendedAttribute("ChromeOnly") or
                        member.getExtendedAttribute("Pref") or
                        member.getExtendedAttribute("Func") or
                        member.getExtendedAttribute("AvailableIn") or
                        member.getExtendedAttribute("CheckAnyPermissions") or
                        member.getExtendedAttribute("CheckAllPermissions")):
                        raise WebIDLError("[Alias] must not be used on a "
                                          "conditionally exposed operation",
                                          [member.location])
                    if member.isStatic():
                        raise WebIDLError("[Alias] must not be used on a "
                                          "static operation",
                                          [member.location])
                    if member.isIdentifierLess():
                        raise WebIDLError("[Alias] must not be used on an "
                                          "identifierless operation",
                                          [member.location])
                    if member.isUnforgeable():
                        raise WebIDLError("[Alias] must not be used on an "
                                          "[Unforgeable] operation",
                                          [member.location])
                    for m in self.members:
                        if m.identifier.name == alias:
                            raise WebIDLError("[Alias=%s] has same name as "
                                              "interface member" % alias,
                                              [member.location, m.location])
                        if m.isMethod() and m != member and alias in m.aliases:
                            raise WebIDLError("duplicate [Alias=%s] definitions" %
                                              alias,
                                              [member.location, m.location])

        if (self.getExtendedAttribute("Pref") and
            self._exposureGlobalNames != set([self.parentScope.primaryGlobalName])):
            raise WebIDLError("[Pref] used on an interface that is not %s-only" %
                              self.parentScope.primaryGlobalName,
                              [self.location])

        for attribute in ["CheckAnyPermissions", "CheckAllPermissions"]:
            if (self.getExtendedAttribute(attribute) and
                self._exposureGlobalNames != set([self.parentScope.primaryGlobalName])):
                raise WebIDLError("[%s] used on an interface that is "
                                  "not %s-only" %
                                  (attribute, self.parentScope.primaryGlobalName),
                                  [self.location])

        # Conditional exposure makes no sense for interfaces with no
        # interface object, unless they're navigator properties.
        if (self.isExposedConditionally() and
            not self.hasInterfaceObject() and
            not self.getNavigatorProperty()):
            raise WebIDLError("Interface with no interface object is "
                              "exposed conditionally",
                              [self.location])

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

    def isSingleOperationInterface(self):
        assert self.isCallback() or self.isJSImplemented()
        return (
            # JS-implemented things should never need the
            # this-handling weirdness of single-operation interfaces.
            not self.isJSImplemented() and
            # Not inheriting from another interface
            not self.parent and
            # No consequential interfaces
            len(self.getConsequentialInterfaces()) == 0 and
            # No attributes of any kinds
            not any(m.isAttr() for m in self.members) and
            # There is at least one regular operation, and all regular
            # operations have the same identifier
            len(set(m.identifier.name for m in self.members if
                    m.isMethod() and not m.isStatic())) == 1)

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
        for attr in attrs:
            identifier = attr.identifier()

            # Special cased attrs
            if identifier == "TreatNonCallableAsNull":
                raise WebIDLError("TreatNonCallableAsNull cannot be specified on interfaces",
                                  [attr.location, self.location])
            if identifier == "TreatNonObjectAsNull":
                raise WebIDLError("TreatNonObjectAsNull cannot be specified on interfaces",
                                  [attr.location, self.location])
            elif identifier == "NoInterfaceObject":
                if not attr.noArguments():
                    raise WebIDLError("[NoInterfaceObject] must take no arguments",
                                      [attr.location])

                if self.ctor():
                    raise WebIDLError("Constructor and NoInterfaceObject are incompatible",
                                      [self.location])

                self._noInterfaceObject = True
            elif identifier == "Constructor" or identifier == "NamedConstructor" or identifier == "ChromeConstructor":
                if identifier == "Constructor" and not self.hasInterfaceObject():
                    raise WebIDLError(str(identifier) + " and NoInterfaceObject are incompatible",
                                      [self.location])

                if identifier == "NamedConstructor" and not attr.hasValue():
                    raise WebIDLError("NamedConstructor must either take an identifier or take a named argument list",
                                      [attr.location])

                if identifier == "ChromeConstructor" and not self.hasInterfaceObject():
                    raise WebIDLError(str(identifier) + " and NoInterfaceObject are incompatible",
                                      [self.location])

                args = attr.args() if attr.hasArgs() else []

                if self.identifier.name == "Promise":
                    promiseType = BuiltinTypes[IDLBuiltinType.Types.any]
                else:
                    promiseType = None
                retType = IDLWrapperType(self.location, self, promiseType)

                if identifier == "Constructor" or identifier == "ChromeConstructor":
                    name = "constructor"
                    allowForbidden = True
                else:
                    name = attr.value()
                    allowForbidden = False

                methodIdentifier = IDLUnresolvedIdentifier(self.location, name,
                                                           allowForbidden=allowForbidden)

                method = IDLMethod(self.location, methodIdentifier, retType,
                                   args, static=True)
                # Constructors are always NewObject and are always
                # assumed to be able to throw (since there's no way to
                # indicate otherwise) and never have any other
                # extended attributes.
                method.addExtendedAttributes(
                    [IDLExtendedAttribute(self.location, ("NewObject",)),
                     IDLExtendedAttribute(self.location, ("Throws",))])
                if identifier == "ChromeConstructor":
                    method.addExtendedAttributes(
                        [IDLExtendedAttribute(self.location, ("ChromeOnly",))])

                if identifier == "Constructor" or identifier == "ChromeConstructor":
                    method.resolve(self)
                else:
                    # We need to detect conflicts for NamedConstructors across
                    # interfaces. We first call resolve on the parentScope,
                    # which will merge all NamedConstructors with the same
                    # identifier accross interfaces as overloads.
                    method.resolve(self.parentScope)

                    # Then we look up the identifier on the parentScope. If the
                    # result is the same as the method we're adding then it
                    # hasn't been added as an overload and it's the first time
                    # we've encountered a NamedConstructor with that identifier.
                    # If the result is not the same as the method we're adding
                    # then it has been added as an overload and we need to check
                    # whether the result is actually one of our existing
                    # NamedConstructors.
                    newMethod = self.parentScope.lookupIdentifier(method.identifier)
                    if newMethod == method:
                        self.namedConstructors.append(method)
                    elif newMethod not in self.namedConstructors:
                        raise WebIDLError("NamedConstructor conflicts with a NamedConstructor of a different interface",
                                          [method.location, newMethod.location])
            elif (identifier == "ArrayClass"):
                if not attr.noArguments():
                    raise WebIDLError("[ArrayClass] must take no arguments",
                                      [attr.location])
                if self.parent:
                    raise WebIDLError("[ArrayClass] must not be specified on "
                                      "an interface with inherited interfaces",
                                      [attr.location, self.location])
            elif (identifier == "ExceptionClass"):
                if not attr.noArguments():
                    raise WebIDLError("[ExceptionClass] must take no arguments",
                                      [attr.location])
                if self.parent:
                    raise WebIDLError("[ExceptionClass] must not be specified on "
                                      "an interface with inherited interfaces",
                                      [attr.location, self.location])
            elif identifier == "Global":
                if attr.hasValue():
                    self.globalNames = [attr.value()]
                elif attr.hasArgs():
                    self.globalNames = attr.args()
                else:
                    self.globalNames = [self.identifier.name]
                self.parentScope.globalNames.update(self.globalNames)
                for globalName in self.globalNames:
                    self.parentScope.globalNameMapping[globalName].add(self.identifier.name)
                self._isOnGlobalProtoChain = True
            elif identifier == "PrimaryGlobal":
                if not attr.noArguments():
                    raise WebIDLError("[PrimaryGlobal] must take no arguments",
                                      [attr.location])
                if self.parentScope.primaryGlobalAttr is not None:
                    raise WebIDLError(
                        "[PrimaryGlobal] specified twice",
                        [attr.location,
                         self.parentScope.primaryGlobalAttr.location])
                self.parentScope.primaryGlobalAttr = attr
                self.parentScope.primaryGlobalName = self.identifier.name
                self.parentScope.globalNames.add(self.identifier.name)
                self.parentScope.globalNameMapping[self.identifier.name].add(self.identifier.name)
                self._isOnGlobalProtoChain = True
            elif (identifier == "NeedResolve" or
                  identifier == "OverrideBuiltins" or
                  identifier == "ChromeOnly" or
                  identifier == "Unforgeable" or
                  identifier == "UnsafeInPrerendering" or
                  identifier == "LegacyEventInit" or
                  identifier == "Abstract"):
                # Known extended attributes that do not take values
                if not attr.noArguments():
                    raise WebIDLError("[%s] must take no arguments" % identifier,
                                      [attr.location])
            elif identifier == "Exposed":
                convertExposedAttrToGlobalNameSet(attr,
                                                  self._exposureGlobalNames)
            elif (identifier == "Pref" or
                  identifier == "JSImplementation" or
                  identifier == "HeaderFile" or
                  identifier == "NavigatorProperty" or
                  identifier == "AvailableIn" or
                  identifier == "Func" or
                  identifier == "CheckAnyPermissions" or
                  identifier == "CheckAllPermissions"):
                # Known extended attributes that take a string value
                if not attr.hasValue():
                    raise WebIDLError("[%s] must have a value" % identifier,
                                      [attr.location])
            else:
                raise WebIDLError("Unknown extended attribute %s on interface" % identifier,
                                  [attr.location])

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

    def setNonPartial(self, location, parent, members):
        assert not parent or isinstance(parent, IDLIdentifierPlaceholder)
        if self._isKnownNonPartial:
            raise WebIDLError("Two non-partial definitions for the "
                              "same interface",
                              [location, self.location])
        self._isKnownNonPartial = True
        # Now make it look like we were parsed at this new location, since
        # that's the place where the interface is "really" defined
        self.location = location
        assert not self.parent
        self.parent = parent
        # Put the new members at the beginning
        self.members = members + self.members

    def addPartialInterface(self, partial):
        assert self.identifier.name == partial.identifier.name
        self._partialInterfaces.append(partial)

    def getJSImplementation(self):
        classId = self.getExtendedAttribute("JSImplementation")
        if not classId:
            return classId
        assert isinstance(classId, list)
        assert len(classId) == 1
        return classId[0]

    def isJSImplemented(self):
        return bool(self.getJSImplementation())

    def getNavigatorProperty(self):
        naviProp = self.getExtendedAttribute("NavigatorProperty")
        if not naviProp:
            return None
        assert len(naviProp) == 1
        assert isinstance(naviProp, list)
        assert len(naviProp[0]) != 0
        return naviProp[0]

    def hasChildInterfaces(self):
        return self._hasChildInterfaces

    def isOnGlobalProtoChain(self):
        return self._isOnGlobalProtoChain

    def _getDependentObjects(self):
        deps = set(self.members)
        deps.update(self.implementedInterfaces)
        if self.parent:
            deps.add(self.parent)
        return deps

    def hasMembersInSlots(self):
        return self._ownMembersInSlots != 0

    def isExposedConditionally(self):
        return (self.getExtendedAttribute("Pref") or
                self.getExtendedAttribute("ChromeOnly") or
                self.getExtendedAttribute("Func") or
                self.getExtendedAttribute("AvailableIn") or
                self.getExtendedAttribute("CheckAnyPermissions") or
                self.getExtendedAttribute("CheckAllPermissions"))


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
        return True

    def canBeEmpty(self):
        """
        Returns true if this dictionary can be empty (that is, it has no
        required members and neither do any of its ancestors).
        """
        return (all(member.optional for member in self.members) and
                (not self.parent or self.parent.canBeEmpty()))

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
        def typeContainsDictionary(memberType, dictionary):
            """
            Returns a tuple whose:

                - First element is a Boolean value indicating whether
                  memberType contains dictionary.

                - Second element is:
                    A list of locations that leads from the type that was passed in
                    the memberType argument, to the dictionary being validated,
                    if the boolean value in the first element is True.

                    None, if the boolean value in the first element is False.
            """

            if (memberType.nullable() or
                memberType.isArray() or
                memberType.isSequence() or
                memberType.isMozMap()):
                return typeContainsDictionary(memberType.inner, dictionary)

            if memberType.isDictionary():
                if memberType.inner == dictionary:
                    return (True, [memberType.location])

                (contains, locations) = dictionaryContainsDictionary(memberType.inner,
                                                                     dictionary)
                if contains:
                    return (True, [memberType.location] + locations)

            if memberType.isUnion():
                for member in memberType.flatMemberTypes:
                    (contains, locations) = typeContainsDictionary(member, dictionary)
                    if contains:
                        return (True, locations)

            return (False, None)

        def dictionaryContainsDictionary(dictMember, dictionary):
            for member in dictMember.members:
                (contains, locations) = typeContainsDictionary(member.type, dictionary)
                if contains:
                    return (True, [member.location] + locations)

            if dictMember.parent:
                if dictMember.parent == dictionary:
                    return (True, [dictMember.location])
                else:
                    (contains, locations) = dictionaryContainsDictionary(dictMember.parent, dictionary)
                    if contains:
                        return (True, [dictMember.location] + locations)

            return (False, None)

        for member in self.members:
            if member.type.isDictionary() and member.type.nullable():
                raise WebIDLError("Dictionary %s has member with nullable "
                                  "dictionary type" % self.identifier.name,
                                  [member.location])
            (contains, locations) = typeContainsDictionary(member.type, self)
            if contains:
                raise WebIDLError("Dictionary %s has member with itself as type." %
                                  self.identifier.name,
                                  [member.location] + locations)

    def module(self):
        return self.location.filename().split('/')[-1].split('.webidl')[0] + 'Binding'

    def addExtendedAttributes(self, attrs):
        assert len(attrs) == 0

    def _getDependentObjects(self):
        deps = set(self.members)
        if (self.parent):
            deps.add(self.parent)
        return deps


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

    def _getDependentObjects(self):
        return set()


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
        'unrestricted_float',
        'float',
        'unrestricted_double',
        # "double" last primitive type to match IDLBuiltinType
        'double',
        # Other types
        'any',
        'domstring',
        'bytestring',
        'usvstring',
        'object',
        'date',
        'void',
        # Funny stuff
        'interface',
        'dictionary',
        'enum',
        'callback',
        'union',
        'sequence',
        'mozmap',
        'array'
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

    def isBoolean(self):
        return False

    def isNumeric(self):
        return False

    def isString(self):
        return False

    def isByteString(self):
        return False

    def isDOMString(self):
        return False

    def isUSVString(self):
        return False

    def isVoid(self):
        return self.name == "Void"

    def isSequence(self):
        return False

    def isMozMap(self):
        return False

    def isArray(self):
        return False

    def isArrayBuffer(self):
        return False

    def isArrayBufferView(self):
        return False

    def isSharedArrayBuffer(self):
        return False

    def isSharedArrayBufferView(self):
        return False

    def isTypedArray(self):
        return False

    def isSharedTypedArray(self):
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
        return self.isInterface() and (self.isArrayBuffer() or
                                       self.isArrayBufferView() or
                                       self.isSharedArrayBuffer() or
                                       self.isSharedArrayBufferView() or
                                       self.isTypedArray() or
                                       self.isSharedTypedArray())

    def isDictionary(self):
        return False

    def isInterface(self):
        return False

    def isAny(self):
        return self.tag() == IDLType.Tags.any

    def isDate(self):
        return self.tag() == IDLType.Tags.date

    def isObject(self):
        return self.tag() == IDLType.Tags.object

    def isPromise(self):
        return False

    def isComplete(self):
        return True

    def includesRestrictedFloat(self):
        return False

    def isFloat(self):
        return False

    def isUnrestricted(self):
        # Should only call this on float types
        assert self.isFloat()

    def isSerializable(self):
        return False

    def tag(self):
        assert False  # Override me!

    def treatNonCallableAsNull(self):
        assert self.tag() == IDLType.Tags.callback
        return self.nullable() and self.inner.callback._treatNonCallableAsNull

    def treatNonObjectAsNull(self):
        assert self.tag() == IDLType.Tags.callback
        return self.nullable() and self.inner.callback._treatNonObjectAsNull

    def addExtendedAttributes(self, attrs):
        assert len(attrs) == 0

    def resolveType(self, parentScope):
        pass

    def unroll(self):
        return self

    def isDistinguishableFrom(self, other):
        raise TypeError("Can't tell whether a generic type is or is not "
                        "distinguishable from other things")

    def isExposedInAllOf(self, exposureSet):
        return True


class IDLUnresolvedType(IDLType):
    """
        Unresolved types are interface types
    """

    def __init__(self, location, name, promiseInnerType=None):
        IDLType.__init__(self, location, name)
        self._promiseInnerType = promiseInnerType

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
            print obj
        assert not obj.isType()
        if obj.isTypedef():
            assert self.name.name == obj.identifier.name
            typedefType = IDLTypedefType(self.location, obj.innerType,
                                         obj.identifier)
            assert not typedefType.isComplete()
            return typedefType.complete(scope)
        elif obj.isCallback() and not obj.isInterface():
            assert self.name.name == obj.identifier.name
            return IDLCallbackType(self.location, obj)

        if self._promiseInnerType and not self._promiseInnerType.isComplete():
            self._promiseInnerType = self._promiseInnerType.complete(scope)

        name = self.name.resolve(scope, None)
        return IDLWrapperType(self.location, obj, self._promiseInnerType)

    def isDistinguishableFrom(self, other):
        raise TypeError("Can't tell whether an unresolved type is or is not "
                        "distinguishable from other things")


class IDLNullableType(IDLType):
    def __init__(self, location, innerType):
        assert not innerType.isVoid()
        assert not innerType == BuiltinTypes[IDLBuiltinType.Types.any]

        name = innerType.name
        if innerType.isComplete():
            name += "OrNull"
        IDLType.__init__(self, location, name)
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

    def isBoolean(self):
        return self.inner.isBoolean()

    def isNumeric(self):
        return self.inner.isNumeric()

    def isString(self):
        return self.inner.isString()

    def isByteString(self):
        return self.inner.isByteString()

    def isDOMString(self):
        return self.inner.isDOMString()

    def isUSVString(self):
        return self.inner.isUSVString()

    def isFloat(self):
        return self.inner.isFloat()

    def isUnrestricted(self):
        return self.inner.isUnrestricted()

    def includesRestrictedFloat(self):
        return self.inner.includesRestrictedFloat()

    def isInteger(self):
        return self.inner.isInteger()

    def isVoid(self):
        return False

    def isSequence(self):
        return self.inner.isSequence()

    def isMozMap(self):
        return self.inner.isMozMap()

    def isArray(self):
        return self.inner.isArray()

    def isArrayBuffer(self):
        return self.inner.isArrayBuffer()

    def isArrayBufferView(self):
        return self.inner.isArrayBufferView()

    def isSharedArrayBuffer(self):
        return self.inner.isSharedArrayBuffer()

    def isSharedArrayBufferView(self):
        return self.inner.isSharedArrayBufferView()

    def isTypedArray(self):
        return self.inner.isTypedArray()

    def isSharedTypedArray(self):
        return self.inner.isSharedTypedArray()

    def isDictionary(self):
        return self.inner.isDictionary()

    def isInterface(self):
        return self.inner.isInterface()

    def isPromise(self):
        return self.inner.isPromise()

    def isCallbackInterface(self):
        return self.inner.isCallbackInterface()

    def isNonCallbackInterface(self):
        return self.inner.isNonCallbackInterface()

    def isEnum(self):
        return self.inner.isEnum()

    def isUnion(self):
        return self.inner.isUnion()

    def isSerializable(self):
        return self.inner.isSerializable()

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

        self.name = self.inner.name + "OrNull"
        return self

    def unroll(self):
        return self.inner.unroll()

    def isDistinguishableFrom(self, other):
        if (other.nullable() or (other.isUnion() and other.hasNullableType) or
            other.isDictionary()):
            # Can't tell which type null should become
            return False
        return self.inner.isDistinguishableFrom(other)

    def _getDependentObjects(self):
        return self.inner._getDependentObjects()


class IDLSequenceType(IDLType):
    def __init__(self, location, parameterType):
        assert not parameterType.isVoid()

        IDLType.__init__(self, location, parameterType.name)
        self.inner = parameterType
        self.builtin = False
        # Need to set self.name up front if our inner type is already complete,
        # since in that case our .complete() won't be called.
        if self.inner.isComplete():
            self.name = self.inner.name + "Sequence"

    def __eq__(self, other):
        return isinstance(other, IDLSequenceType) and self.inner == other.inner

    def __str__(self):
        return self.inner.__str__() + "Sequence"

    def nullable(self):
        return False

    def isPrimitive(self):
        return False

    def isString(self):
        return False

    def isByteString(self):
        return False

    def isDOMString(self):
        return False

    def isUSVString(self):
        return False

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

    def isSerializable(self):
        return self.inner.isSerializable()

    def includesRestrictedFloat(self):
        return self.inner.includesRestrictedFloat()

    def tag(self):
        return IDLType.Tags.sequence

    def resolveType(self, parentScope):
        assert isinstance(parentScope, IDLScope)
        self.inner.resolveType(parentScope)

    def isComplete(self):
        return self.inner.isComplete()

    def complete(self, scope):
        self.inner = self.inner.complete(scope)
        self.name = self.inner.name + "Sequence"
        return self

    def unroll(self):
        return self.inner.unroll()

    def isDistinguishableFrom(self, other):
        if other.isPromise():
            return False
        if other.isUnion():
            # Just forward to the union; it'll deal
            return other.isDistinguishableFrom(self)
        return (other.isPrimitive() or other.isString() or other.isEnum() or
                other.isDate() or other.isInterface() or
                other.isDictionary() or
                other.isCallback() or other.isMozMap())

    def _getDependentObjects(self):
        return self.inner._getDependentObjects()


class IDLMozMapType(IDLType):
    # XXXbz This is pretty similar to IDLSequenceType in various ways.
    # And maybe to IDLNullableType.  Should we have a superclass for
    # "type containing this other type"?  Bug 1015318.
    def __init__(self, location, parameterType):
        assert not parameterType.isVoid()

        IDLType.__init__(self, location, parameterType.name)
        self.inner = parameterType
        self.builtin = False
        # Need to set self.name up front if our inner type is already complete,
        # since in that case our .complete() won't be called.
        if self.inner.isComplete():
            self.name = self.inner.name + "MozMap"

    def __eq__(self, other):
        return isinstance(other, IDLMozMapType) and self.inner == other.inner

    def __str__(self):
        return self.inner.__str__() + "MozMap"

    def isMozMap(self):
        return True

    def includesRestrictedFloat(self):
        return self.inner.includesRestrictedFloat()

    def tag(self):
        return IDLType.Tags.mozmap

    def resolveType(self, parentScope):
        assert isinstance(parentScope, IDLScope)
        self.inner.resolveType(parentScope)

    def isComplete(self):
        return self.inner.isComplete()

    def complete(self, scope):
        self.inner = self.inner.complete(scope)
        self.name = self.inner.name + "MozMap"
        return self

    def unroll(self):
        # We do not unroll our inner.  Just stop at ourselves.  That
        # lets us add headers for both ourselves and our inner as
        # needed.
        return self

    def isDistinguishableFrom(self, other):
        if other.isPromise():
            return False
        if other.isUnion():
            # Just forward to the union; it'll deal
            return other.isDistinguishableFrom(self)
        return (other.isPrimitive() or other.isString() or other.isEnum() or
                other.isDate() or other.isNonCallbackInterface() or other.isSequence())

    def isExposedInAllOf(self, exposureSet):
        return self.inner.unroll().isExposedInAllOf(exposureSet)

    def _getDependentObjects(self):
        return self.inner._getDependentObjects()


class IDLUnionType(IDLType):
    def __init__(self, location, memberTypes):
        IDLType.__init__(self, location, "")
        self.memberTypes = memberTypes
        self.hasNullableType = False
        self._dictionaryType = None
        self.flatMemberTypes = None
        self.builtin = False

    def __eq__(self, other):
        return isinstance(other, IDLUnionType) and self.memberTypes == other.memberTypes

    def __hash__(self):
        assert self.isComplete()
        return self.name.__hash__()

    def isVoid(self):
        return False

    def isUnion(self):
        return True

    def isSerializable(self):
        return all(m.isSerializable() for m in self.memberTypes)

    def includesRestrictedFloat(self):
        return any(t.includesRestrictedFloat() for t in self.memberTypes)

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
                if self.hasDictionaryType():
                    raise WebIDLError("Can't have a nullable type and a "
                                      "dictionary type in a union",
                                      [self._dictionaryType.location,
                                       self.flatMemberTypes[i].location])
                self.hasNullableType = True
                nullableType = self.flatMemberTypes[i]
                self.flatMemberTypes[i] = self.flatMemberTypes[i].inner
                continue
            if self.flatMemberTypes[i].isDictionary():
                if self.hasNullableType:
                    raise WebIDLError("Can't have a nullable type and a "
                                      "dictionary type in a union",
                                      [nullableType.location,
                                       self.flatMemberTypes[i].location])
                self._dictionaryType = self.flatMemberTypes[i]
            elif self.flatMemberTypes[i].isUnion():
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

    def isExposedInAllOf(self, exposureSet):
        # We could have different member types in different globals.  Just make sure that each thing in exposureSet has one of our member types exposed in it.
        for globalName in exposureSet:
            if not any(t.unroll().isExposedInAllOf(set([globalName])) for t
                       in self.flatMemberTypes):
                return False
        return True

    def hasDictionaryType(self):
        return self._dictionaryType is not None

    def hasPossiblyEmptyDictionaryType(self):
        return (self._dictionaryType is not None and
                self._dictionaryType.inner.canBeEmpty())

    def _getDependentObjects(self):
        return set(self.memberTypes)


class IDLArrayType(IDLType):
    def __init__(self, location, parameterType):
        assert not parameterType.isVoid()
        if parameterType.isSequence():
            raise WebIDLError("Array type cannot parameterize over a sequence type",
                              [location])
        if parameterType.isMozMap():
            raise WebIDLError("Array type cannot parameterize over a MozMap type",
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

    def isByteString(self):
        return False

    def isDOMString(self):
        return False

    def isUSVString(self):
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
        return IDLType.Tags.array

    def resolveType(self, parentScope):
        assert isinstance(parentScope, IDLScope)
        self.inner.resolveType(parentScope)

    def isComplete(self):
        return self.inner.isComplete()

    def complete(self, scope):
        self.inner = self.inner.complete(scope)
        self.name = self.inner.name

        if self.inner.isDictionary():
            raise WebIDLError("Array type must not contain "
                              "dictionary as element type.",
                              [self.inner.location])

        assert not self.inner.isSequence()

        return self

    def unroll(self):
        return self.inner.unroll()

    def isDistinguishableFrom(self, other):
        if other.isPromise():
            return False
        if other.isUnion():
            # Just forward to the union; it'll deal
            return other.isDistinguishableFrom(self)
        return (other.isPrimitive() or other.isString() or other.isEnum() or
                other.isDate() or other.isNonCallbackInterface())

    def _getDependentObjects(self):
        return self.inner._getDependentObjects()


class IDLTypedefType(IDLType):
    def __init__(self, location, innerType, name):
        IDLType.__init__(self, location, name)
        self.inner = innerType
        self.builtin = False

    def __eq__(self, other):
        return isinstance(other, IDLTypedefType) and self.inner == other.inner

    def __str__(self):
        return self.name

    def nullable(self):
        return self.inner.nullable()

    def isPrimitive(self):
        return self.inner.isPrimitive()

    def isBoolean(self):
        return self.inner.isBoolean()

    def isNumeric(self):
        return self.inner.isNumeric()

    def isString(self):
        return self.inner.isString()

    def isByteString(self):
        return self.inner.isByteString()

    def isDOMString(self):
        return self.inner.isDOMString()

    def isUSVString(self):
        return self.inner.isUSVString()

    def isVoid(self):
        return self.inner.isVoid()

    def isSequence(self):
        return self.inner.isSequence()

    def isMozMap(self):
        return self.inner.isMozMap()

    def isArray(self):
        return self.inner.isArray()

    def isDictionary(self):
        return self.inner.isDictionary()

    def isArrayBuffer(self):
        return self.inner.isArrayBuffer()

    def isArrayBufferView(self):
        return self.inner.isArrayBufferView()

    def isSharedArrayBuffer(self):
        return self.inner.isSharedArrayBuffer()

    def isSharedArrayBufferView(self):
        return self.inner.isSharedArrayBufferView()

    def isTypedArray(self):
        return self.inner.isTypedArray()

    def isSharedTypedArray(self):
        return self.inner.isSharedTypedArray()

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

    # Do we need a resolveType impl?  I don't think it's particularly useful....

    def tag(self):
        return self.inner.tag()

    def unroll(self):
        return self.inner.unroll()

    def isDistinguishableFrom(self, other):
        return self.inner.isDistinguishableFrom(other)

    def _getDependentObjects(self):
        return self.inner._getDependentObjects()


class IDLTypedef(IDLObjectWithIdentifier):
    def __init__(self, location, parentScope, innerType, name):
        identifier = IDLUnresolvedIdentifier(location, name)
        IDLObjectWithIdentifier.__init__(self, location, parentScope, identifier)
        self.innerType = innerType

    def __str__(self):
        return "Typedef %s %s" % (self.identifier.name, self.innerType)

    def finish(self, parentScope):
        if not self.innerType.isComplete():
            self.innerType = self.innerType.complete(parentScope)

    def validate(self):
        pass

    def isTypedef(self):
        return True

    def addExtendedAttributes(self, attrs):
        assert len(attrs) == 0

    def _getDependentObjects(self):
        return self.innerType._getDependentObjects()


class IDLWrapperType(IDLType):
    def __init__(self, location, inner, promiseInnerType=None):
        IDLType.__init__(self, location, inner.identifier.name)
        self.inner = inner
        self._identifier = inner.identifier
        self.builtin = False
        assert not promiseInnerType or inner.identifier.name == "Promise"
        self._promiseInnerType = promiseInnerType

    def __eq__(self, other):
        return (isinstance(other, IDLWrapperType) and
                self._identifier == other._identifier and
                self.builtin == other.builtin)

    def __str__(self):
        return str(self.name) + " (Wrapper)"

    def nullable(self):
        return False

    def isPrimitive(self):
        return False

    def isString(self):
        return False

    def isByteString(self):
        return False

    def isDOMString(self):
        return False

    def isUSVString(self):
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
        return (isinstance(self.inner, IDLInterface) or
                isinstance(self.inner, IDLExternalInterface))

    def isCallbackInterface(self):
        return self.isInterface() and self.inner.isCallback()

    def isNonCallbackInterface(self):
        return self.isInterface() and not self.inner.isCallback()

    def isEnum(self):
        return isinstance(self.inner, IDLEnum)

    def isPromise(self):
        return (isinstance(self.inner, IDLInterface) and
                self.inner.identifier.name == "Promise")

    def promiseInnerType(self):
        assert self.isPromise()
        return self._promiseInnerType

    def isSerializable(self):
        if self.isInterface():
            if self.inner.isExternal():
                return False
            return any(m.isMethod() and m.isJsonifier() for m in self.inner.members)
        elif self.isEnum():
            return True
        elif self.isDictionary():
            return all(m.type.isSerializable() for m in self.inner.members)
        else:
            raise WebIDLError("IDLWrapperType wraps type %s that we don't know if "
                              "is serializable" % type(self.inner), [self.location])

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
        if self.isPromise():
            return False
        if other.isPromise():
            return False
        if other.isUnion():
            # Just forward to the union; it'll deal
            return other.isDistinguishableFrom(self)
        assert self.isInterface() or self.isEnum() or self.isDictionary()
        if self.isEnum():
            return (other.isPrimitive() or other.isInterface() or other.isObject() or
                    other.isCallback() or other.isDictionary() or
                    other.isSequence() or other.isMozMap() or other.isArray() or
                    other.isDate())
        if self.isDictionary() and other.nullable():
            return False
        if (other.isPrimitive() or other.isString() or other.isEnum() or
            other.isDate() or other.isSequence()):
            return True
        if self.isDictionary():
            return other.isNonCallbackInterface()

        assert self.isInterface()
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
            other.isMozMap() or other.isArray()):
            return self.isNonCallbackInterface()

        # Not much else |other| can be
        assert other.isObject()
        return False

    def isExposedInAllOf(self, exposureSet):
        if not self.isInterface():
            return True
        iface = self.inner
        if iface.isExternal():
            # Let's say true, though ideally we'd only do this when
            # exposureSet contains the primary global's name.
            return True
        if (self.isPromise() and
            # Check the internal type
            not self.promiseInnerType().unroll().isExposedInAllOf(exposureSet)):
            return False
        return iface.exposureSet.issuperset(exposureSet)

    def _getDependentObjects(self):
        # NB: The codegen for an interface type depends on
        #  a) That the identifier is in fact an interface (as opposed to
        #     a dictionary or something else).
        #  b) The native type of the interface.
        #  If we depend on the interface object we will also depend on
        #  anything the interface depends on which is undesirable.  We
        #  considered implementing a dependency just on the interface type
        #  file, but then every modification to an interface would cause this
        #  to be regenerated which is still undesirable.  We decided not to
        #  depend on anything, reasoning that:
        #  1) Changing the concrete type of the interface requires modifying
        #     Bindings.conf, which is still a global dependency.
        #  2) Changing an interface to a dictionary (or vice versa) with the
        #     same identifier should be incredibly rare.
        #
        # On the other hand, if our type is a dictionary, we should
        # depend on it, because the member types of a dictionary
        # affect whether a method taking the dictionary as an argument
        # takes a JSContext* argument or not.
        if self.isDictionary():
            return set([self.inner])
        return set()


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
        'unrestricted_float',
        'float',
        'unrestricted_double',
        # IMPORTANT: "double" must be the last primitive type listed
        'double',
        # Other types
        'any',
        'domstring',
        'bytestring',
        'usvstring',
        'object',
        'date',
        'void',
        # Funny stuff
        'ArrayBuffer',
        'ArrayBufferView',
        'SharedArrayBuffer',
        'SharedArrayBufferView',
        'Int8Array',
        'Uint8Array',
        'Uint8ClampedArray',
        'Int16Array',
        'Uint16Array',
        'Int32Array',
        'Uint32Array',
        'Float32Array',
        'Float64Array',
        'SharedInt8Array',
        'SharedUint8Array',
        'SharedUint8ClampedArray',
        'SharedInt16Array',
        'SharedUint16Array',
        'SharedInt32Array',
        'SharedUint32Array',
        'SharedFloat32Array',
        'SharedFloat64Array'
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
        Types.unrestricted_float: IDLType.Tags.unrestricted_float,
        Types.float: IDLType.Tags.float,
        Types.unrestricted_double: IDLType.Tags.unrestricted_double,
        Types.double: IDLType.Tags.double,
        Types.any: IDLType.Tags.any,
        Types.domstring: IDLType.Tags.domstring,
        Types.bytestring: IDLType.Tags.bytestring,
        Types.usvstring: IDLType.Tags.usvstring,
        Types.object: IDLType.Tags.object,
        Types.date: IDLType.Tags.date,
        Types.void: IDLType.Tags.void,
        Types.ArrayBuffer: IDLType.Tags.interface,
        Types.ArrayBufferView: IDLType.Tags.interface,
        Types.SharedArrayBuffer: IDLType.Tags.interface,
        Types.SharedArrayBufferView: IDLType.Tags.interface,
        Types.Int8Array: IDLType.Tags.interface,
        Types.Uint8Array: IDLType.Tags.interface,
        Types.Uint8ClampedArray: IDLType.Tags.interface,
        Types.Int16Array: IDLType.Tags.interface,
        Types.Uint16Array: IDLType.Tags.interface,
        Types.Int32Array: IDLType.Tags.interface,
        Types.Uint32Array: IDLType.Tags.interface,
        Types.Float32Array: IDLType.Tags.interface,
        Types.Float64Array: IDLType.Tags.interface,
        Types.SharedInt8Array: IDLType.Tags.interface,
        Types.SharedUint8Array: IDLType.Tags.interface,
        Types.SharedUint8ClampedArray: IDLType.Tags.interface,
        Types.SharedInt16Array: IDLType.Tags.interface,
        Types.SharedUint16Array: IDLType.Tags.interface,
        Types.SharedInt32Array: IDLType.Tags.interface,
        Types.SharedUint32Array: IDLType.Tags.interface,
        Types.SharedFloat32Array: IDLType.Tags.interface,
        Types.SharedFloat64Array: IDLType.Tags.interface
    }

    def __init__(self, location, name, type):
        IDLType.__init__(self, location, name)
        self.builtin = True
        self._typeTag = type

    def isPrimitive(self):
        return self._typeTag <= IDLBuiltinType.Types.double

    def isBoolean(self):
        return self._typeTag == IDLBuiltinType.Types.boolean

    def isNumeric(self):
        return self.isPrimitive() and not self.isBoolean()

    def isString(self):
        return (self._typeTag == IDLBuiltinType.Types.domstring or
                self._typeTag == IDLBuiltinType.Types.bytestring or
                self._typeTag == IDLBuiltinType.Types.usvstring)

    def isByteString(self):
        return self._typeTag == IDLBuiltinType.Types.bytestring

    def isDOMString(self):
        return self._typeTag == IDLBuiltinType.Types.domstring

    def isUSVString(self):
        return self._typeTag == IDLBuiltinType.Types.usvstring

    def isInteger(self):
        return self._typeTag <= IDLBuiltinType.Types.unsigned_long_long

    def isArrayBuffer(self):
        return self._typeTag == IDLBuiltinType.Types.ArrayBuffer

    def isArrayBufferView(self):
        return self._typeTag == IDLBuiltinType.Types.ArrayBufferView

    def isSharedArrayBuffer(self):
        return self._typeTag == IDLBuiltinType.Types.SharedArrayBuffer

    def isSharedArrayBufferView(self):
        return self._typeTag == IDLBuiltinType.Types.SharedArrayBufferView

    def isTypedArray(self):
        return (self._typeTag >= IDLBuiltinType.Types.Int8Array and
                self._typeTag <= IDLBuiltinType.Types.Float64Array)

    def isSharedTypedArray(self):
        return (self._typeTag >= IDLBuiltinType.Types.SharedInt8Array and
                self._typeTag <= IDLBuiltinType.Types.SharedFloat64Array)

    def isInterface(self):
        # TypedArray things are interface types per the TypedArray spec,
        # but we handle them as builtins because SpiderMonkey implements
        # all of it internally.
        return (self.isArrayBuffer() or
                self.isArrayBufferView() or
                self.isSharedArrayBuffer() or
                self.isSharedArrayBufferView() or
                self.isTypedArray() or
                self.isSharedTypedArray())

    def isNonCallbackInterface(self):
        # All the interfaces we can be are non-callback
        return self.isInterface()

    def isFloat(self):
        return (self._typeTag == IDLBuiltinType.Types.float or
                self._typeTag == IDLBuiltinType.Types.double or
                self._typeTag == IDLBuiltinType.Types.unrestricted_float or
                self._typeTag == IDLBuiltinType.Types.unrestricted_double)

    def isUnrestricted(self):
        assert self.isFloat()
        return (self._typeTag == IDLBuiltinType.Types.unrestricted_float or
                self._typeTag == IDLBuiltinType.Types.unrestricted_double)

    def isSerializable(self):
        return self.isPrimitive() or self.isString() or self.isDate()

    def includesRestrictedFloat(self):
        return self.isFloat() and not self.isUnrestricted()

    def tag(self):
        return IDLBuiltinType.TagLookup[self._typeTag]

    def isDistinguishableFrom(self, other):
        if other.isPromise():
            return False
        if other.isUnion():
            # Just forward to the union; it'll deal
            return other.isDistinguishableFrom(self)
        if self.isBoolean():
            return (other.isNumeric() or other.isString() or other.isEnum() or
                    other.isInterface() or other.isObject() or
                    other.isCallback() or other.isDictionary() or
                    other.isSequence() or other.isMozMap() or other.isArray() or
                    other.isDate())
        if self.isNumeric():
            return (other.isBoolean() or other.isString() or other.isEnum() or
                    other.isInterface() or other.isObject() or
                    other.isCallback() or other.isDictionary() or
                    other.isSequence() or other.isMozMap() or other.isArray() or
                    other.isDate())
        if self.isString():
            return (other.isPrimitive() or other.isInterface() or
                    other.isObject() or
                    other.isCallback() or other.isDictionary() or
                    other.isSequence() or other.isMozMap() or other.isArray() or
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
                    other.isMozMap() or other.isArray())
        if self.isVoid():
            return not other.isVoid()
        # Not much else we could be!
        assert self.isSpiderMonkeyInterface()
        # Like interfaces, but we know we're not a callback
        return (other.isPrimitive() or other.isString() or other.isEnum() or
                other.isCallback() or other.isDictionary() or
                other.isSequence() or other.isMozMap() or other.isArray() or
                other.isDate() or
                (other.isInterface() and (
                 # ArrayBuffer is distinguishable from everything
                 # that's not an ArrayBuffer or a callback interface
                 (self.isArrayBuffer() and not other.isArrayBuffer()) or
                 (self.isSharedArrayBuffer() and not other.isSharedArrayBuffer()) or
                 # ArrayBufferView is distinguishable from everything
                 # that's not an ArrayBufferView or typed array.
                 (self.isArrayBufferView() and not other.isArrayBufferView() and
                  not other.isTypedArray()) or
                 (self.isSharedArrayBufferView() and not other.isSharedArrayBufferView() and
                  not other.isSharedTypedArray()) or
                 # Typed arrays are distinguishable from everything
                 # except ArrayBufferView and the same type of typed
                 # array
                 (self.isTypedArray() and not other.isArrayBufferView() and not
                  (other.isTypedArray() and other.name == self.name)) or
                 (self.isSharedTypedArray() and not other.isSharedArrayBufferView() and not
                  (other.isSharedTypedArray() and other.name == self.name)))))

    def _getDependentObjects(self):
        return set()

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
    IDLBuiltinType.Types.unrestricted_float:
        IDLBuiltinType(BuiltinLocation("<builtin type>"), "UnrestrictedFloat",
                       IDLBuiltinType.Types.unrestricted_float),
    IDLBuiltinType.Types.double:
        IDLBuiltinType(BuiltinLocation("<builtin type>"), "Double",
                       IDLBuiltinType.Types.double),
    IDLBuiltinType.Types.unrestricted_double:
        IDLBuiltinType(BuiltinLocation("<builtin type>"), "UnrestrictedDouble",
                       IDLBuiltinType.Types.unrestricted_double),
    IDLBuiltinType.Types.any:
        IDLBuiltinType(BuiltinLocation("<builtin type>"), "Any",
                       IDLBuiltinType.Types.any),
    IDLBuiltinType.Types.domstring:
        IDLBuiltinType(BuiltinLocation("<builtin type>"), "String",
                       IDLBuiltinType.Types.domstring),
    IDLBuiltinType.Types.bytestring:
        IDLBuiltinType(BuiltinLocation("<builtin type>"), "ByteString",
                       IDLBuiltinType.Types.bytestring),
    IDLBuiltinType.Types.usvstring:
        IDLBuiltinType(BuiltinLocation("<builtin type>"), "USVString",
                       IDLBuiltinType.Types.usvstring),
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
    IDLBuiltinType.Types.SharedArrayBuffer:
        IDLBuiltinType(BuiltinLocation("<builtin type>"), "SharedArrayBuffer",
                       IDLBuiltinType.Types.SharedArrayBuffer),
    IDLBuiltinType.Types.SharedArrayBufferView:
        IDLBuiltinType(BuiltinLocation("<builtin type>"), "SharedArrayBufferView",
                       IDLBuiltinType.Types.SharedArrayBufferView),
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
                       IDLBuiltinType.Types.Float64Array),
    IDLBuiltinType.Types.SharedInt8Array:
        IDLBuiltinType(BuiltinLocation("<builtin type>"), "SharedInt8Array",
                       IDLBuiltinType.Types.SharedInt8Array),
    IDLBuiltinType.Types.SharedUint8Array:
        IDLBuiltinType(BuiltinLocation("<builtin type>"), "SharedUint8Array",
                       IDLBuiltinType.Types.SharedUint8Array),
    IDLBuiltinType.Types.SharedUint8ClampedArray:
        IDLBuiltinType(BuiltinLocation("<builtin type>"), "SharedUint8ClampedArray",
                       IDLBuiltinType.Types.SharedUint8ClampedArray),
    IDLBuiltinType.Types.SharedInt16Array:
        IDLBuiltinType(BuiltinLocation("<builtin type>"), "SharedInt16Array",
                       IDLBuiltinType.Types.SharedInt16Array),
    IDLBuiltinType.Types.SharedUint16Array:
        IDLBuiltinType(BuiltinLocation("<builtin type>"), "SharedUint16Array",
                       IDLBuiltinType.Types.SharedUint16Array),
    IDLBuiltinType.Types.SharedInt32Array:
        IDLBuiltinType(BuiltinLocation("<builtin type>"), "SharedInt32Array",
                       IDLBuiltinType.Types.SharedInt32Array),
    IDLBuiltinType.Types.SharedUint32Array:
        IDLBuiltinType(BuiltinLocation("<builtin type>"), "SharedUint32Array",
                       IDLBuiltinType.Types.SharedUint32Array),
    IDLBuiltinType.Types.SharedFloat32Array:
        IDLBuiltinType(BuiltinLocation("<builtin type>"), "SharedFloat32Array",
                       IDLBuiltinType.Types.SharedFloat32Array),
    IDLBuiltinType.Types.SharedFloat64Array:
        IDLBuiltinType(BuiltinLocation("<builtin type>"), "SharedFloat64Array",
                       IDLBuiltinType.Types.SharedFloat64Array)
}


integerTypeSizes = {
    IDLBuiltinType.Types.byte: (-128, 127),
    IDLBuiltinType.Types.octet:  (0, 255),
    IDLBuiltinType.Types.short: (-32768, 32767),
    IDLBuiltinType.Types.unsigned_short: (0, 65535),
    IDLBuiltinType.Types.long: (-2147483648, 2147483647),
    IDLBuiltinType.Types.unsigned_long: (0, 4294967295),
    IDLBuiltinType.Types.long_long: (-9223372036854775808, 9223372036854775807),
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
            return self  # Nothing to do

        # We first check for unions to ensure that even if the union is nullable
        # we end up with the right flat member type, not the union's type.
        if type.isUnion():
            # We use the flat member types here, because if we have a nullable
            # member type, or a nested union, we want the type the value
            # actually coerces to, not the nullable or nested union type.
            for subtype in type.unroll().flatMemberTypes:
                try:
                    coercedValue = self.coerceToType(subtype, location)
                    # Create a new IDLValue to make sure that we have the
                    # correct float/double type.  This is necessary because we
                    # use the value's type when it is a default value of a
                    # union, and the union cares about the exact float type.
                    return IDLValue(self.location, subtype, coercedValue.value)
                except:
                    pass
        # If the type allows null, rerun this matching on the inner type, except
        # nullable enums.  We handle those specially, because we want our
        # default string values to stay strings even when assigned to a nullable
        # enum.
        elif type.nullable() and not type.isEnum():
            innerValue = self.coerceToType(type.inner, location)
            return IDLValue(self.location, type, innerValue.value)

        elif self.type.isInteger() and type.isInteger():
            # We're both integer types.  See if we fit.

            (min, max) = integerTypeSizes[type._typeTag]
            if self.value <= max and self.value >= min:
                # Promote
                return IDLValue(self.location, type, self.value)
            else:
                raise WebIDLError("Value %s is out of range for type %s." %
                                  (self.value, type), [location])
        elif self.type.isInteger() and type.isFloat():
            # Convert an integer literal into float
            if -2**24 <= self.value <= 2**24:
                return IDLValue(self.location, type, float(self.value))
            else:
                raise WebIDLError("Converting value %s to %s will lose precision." %
                                  (self.value, type), [location])
        elif self.type.isString() and type.isEnum():
            # Just keep our string, but make sure it's a valid value for this enum
            enum = type.unroll().inner
            if self.value not in enum.values():
                raise WebIDLError("'%s' is not a valid default value for enum %s"
                                  % (self.value, enum.identifier.name),
                                  [location, enum.location])
            return self
        elif self.type.isFloat() and type.isFloat():
            if (not type.isUnrestricted() and
                (self.value == float("inf") or self.value == float("-inf") or
                 math.isnan(self.value))):
                raise WebIDLError("Trying to convert unrestricted value %s to non-unrestricted"
                                  % self.value, [location]);
            return IDLValue(self.location, type, self.value)
        elif self.type.isString() and type.isUSVString():
            # Allow USVStrings to use default value just like
            # DOMString.  No coercion is required in this case as Codegen.py
            # treats USVString just like DOMString, but with an
            # extra normalization step.
            assert self.type.isDOMString()
            return self
        raise WebIDLError("Cannot coerce type %s to type %s." %
                          (self.type, type), [location])

    def _getDependentObjects(self):
        return set()


class IDLNullValue(IDLObject):
    def __init__(self, location):
        IDLObject.__init__(self, location)
        self.type = None
        self.value = None

    def coerceToType(self, type, location):
        if (not isinstance(type, IDLNullableType) and
            not (type.isUnion() and type.hasNullableType) and
            not (type.isUnion() and type.hasDictionaryType()) and
            not type.isDictionary() and
            not type.isAny()):
            raise WebIDLError("Cannot coerce null value to type %s." % type,
                              [location])

        nullValue = IDLNullValue(self.location)
        if type.isUnion() and not type.nullable() and type.hasDictionaryType():
            # We're actually a default value for the union's dictionary member.
            # Use its type.
            for t in type.flatMemberTypes:
                if t.isDictionary():
                    nullValue.type = t
                    return nullValue
        nullValue.type = type
        return nullValue

    def _getDependentObjects(self):
        return set()


class IDLEmptySequenceValue(IDLObject):
    def __init__(self, location):
        IDLObject.__init__(self, location)
        self.type = None
        self.value = None

    def coerceToType(self, type, location):
        if type.isUnion():
            # We use the flat member types here, because if we have a nullable
            # member type, or a nested union, we want the type the value
            # actually coerces to, not the nullable or nested union type.
            for subtype in type.unroll().flatMemberTypes:
                try:
                    return self.coerceToType(subtype, location)
                except:
                    pass

        if not type.isSequence():
            raise WebIDLError("Cannot coerce empty sequence value to type %s." % type,
                              [location])

        emptySequenceValue = IDLEmptySequenceValue(self.location)
        emptySequenceValue.type = type
        return emptySequenceValue

    def _getDependentObjects(self):
        return set()


class IDLUndefinedValue(IDLObject):
    def __init__(self, location):
        IDLObject.__init__(self, location)
        self.type = None
        self.value = None

    def coerceToType(self, type, location):
        if not type.isAny():
            raise WebIDLError("Cannot coerce undefined value to type %s." % type,
                              [location])

        undefinedValue = IDLUndefinedValue(self.location)
        undefinedValue.type = type
        return undefinedValue

    def _getDependentObjects(self):
        return set()


class IDLInterfaceMember(IDLObjectWithIdentifier, IDLExposureMixins):

    Tags = enum(
        'Const',
        'Attr',
        'Method',
        'MaplikeOrSetlike'
    )

    Special = enum(
        'Static',
        'Stringifier'
    )

    AffectsValues = ("Nothing", "Everything")
    DependsOnValues = ("Nothing", "DOMState", "DeviceState", "Everything")

    def __init__(self, location, identifier, tag):
        IDLObjectWithIdentifier.__init__(self, location, None, identifier)
        IDLExposureMixins.__init__(self, location)
        self.tag = tag
        self._extendedAttrDict = {}

    def isMethod(self):
        return self.tag == IDLInterfaceMember.Tags.Method

    def isAttr(self):
        return self.tag == IDLInterfaceMember.Tags.Attr

    def isConst(self):
        return self.tag == IDLInterfaceMember.Tags.Const

    def isMaplikeOrSetlike(self):
        return self.tag == IDLInterfaceMember.Tags.MaplikeOrSetlike

    def addExtendedAttributes(self, attrs):
        for attr in attrs:
            self.handleExtendedAttribute(attr)
            attrlist = attr.listValue()
            self._extendedAttrDict[attr.identifier()] = attrlist if len(attrlist) else True

    def handleExtendedAttribute(self, attr):
        pass

    def getExtendedAttribute(self, name):
        return self._extendedAttrDict.get(name, None)

    def finish(self, scope):
        # We better be exposed _somewhere_.
        if (len(self._exposureGlobalNames) == 0):
            print self.identifier.name
        assert len(self._exposureGlobalNames) != 0
        IDLExposureMixins.finish(self, scope)

    def validate(self):
        if (self.getExtendedAttribute("Pref") and
            self.exposureSet != set([self._globalScope.primaryGlobalName])):
            raise WebIDLError("[Pref] used on an interface member that is not "
                              "%s-only" % self._globalScope.primaryGlobalName,
                              [self.location])

        for attribute in ["CheckAnyPermissions", "CheckAllPermissions"]:
            if (self.getExtendedAttribute(attribute) and
                self.exposureSet != set([self._globalScope.primaryGlobalName])):
                raise WebIDLError("[%s] used on an interface member that is "
                                  "not %s-only" %
                                  (attribute, self.parentScope.primaryGlobalName),
                                  [self.location])

        if self.isAttr() or self.isMethod():
            if self.affects == "Everything" and self.dependsOn != "Everything":
                raise WebIDLError("Interface member is flagged as affecting "
                                  "everything but not depending on everything. "
                                  "That seems rather unlikely.",
                                  [self.location])

        if self.getExtendedAttribute("NewObject"):
            if self.dependsOn == "Nothing" or self.dependsOn == "DOMState":
                raise WebIDLError("A [NewObject] method is not idempotent, "
                                  "so it has to depend on something other than DOM state.",
                                  [self.location])

    def _setDependsOn(self, dependsOn):
        if self.dependsOn != "Everything":
            raise WebIDLError("Trying to specify multiple different DependsOn, "
                              "Pure, or Constant extended attributes for "
                              "attribute", [self.location])
        if dependsOn not in IDLInterfaceMember.DependsOnValues:
            raise WebIDLError("Invalid [DependsOn=%s] on attribute" % dependsOn,
                              [self.location])
        self.dependsOn = dependsOn

    def _setAffects(self, affects):
        if self.affects != "Everything":
            raise WebIDLError("Trying to specify multiple different Affects, "
                              "Pure, or Constant extended attributes for "
                              "attribute", [self.location])
        if affects not in IDLInterfaceMember.AffectsValues:
            raise WebIDLError("Invalid [Affects=%s] on attribute" % dependsOn,
                              [self.location])
        self.affects = affects

    def _addAlias(self, alias):
        if alias in self.aliases:
            raise WebIDLError("Duplicate [Alias=%s] on attribute" % alias,
                              [self.location])
        self.aliases.append(alias)


# MaplikeOrSetlike adds a trait to an interface, like map or iteration
# functions. To handle them while still getting all of the generated binding
# code taken care of, we treat them as macros that are expanded into members
# based on parsed values.
class IDLMaplikeOrSetlike(IDLInterfaceMember):

    MaplikeOrSetlikeTypes = enum(
        'maplike',
        'setlike'
    )

    def __init__(self, location, identifier, maplikeOrSetlikeType,
                 readonly, keyType, valueType):
        IDLInterfaceMember.__init__(self, location, identifier,
                                    IDLInterfaceMember.Tags.MaplikeOrSetlike)

        assert isinstance(keyType, IDLType)
        assert isinstance(valueType, IDLType)
        self.maplikeOrSetlikeType = maplikeOrSetlikeType
        self.readonly = readonly
        self.keyType = keyType
        self.valueType = valueType
        self.slotIndex = None
        self.disallowedMemberNames = []
        self.disallowedNonMethodNames = []

        # When generating JSAPI access code, we need to know the backing object
        # type prefix to create the correct function. Generate here for reuse.
        if self.isMaplike():
            self.prefix = 'Map'
        elif self.isSetlike():
            self.prefix = 'Set'

    def __str__(self):
        return "declared '%s' with key '%s'" % (self.maplikeOrSetlikeType, self.keyType)

    def isMaplike(self):
        return self.maplikeOrSetlikeType == "maplike"

    def isSetlike(self):
        return self.maplikeOrSetlikeType == "setlike"

    def checkCollisions(self, members, isAncestor):
        for member in members:
            # Check that there are no disallowed members
            if (member.identifier.name in self.disallowedMemberNames and
                not ((member.isMethod() and member.isMaplikeOrSetlikeMethod()) or
                     (member.isAttr() and member.isMaplikeOrSetlikeAttr()))):
                raise WebIDLError("Member '%s' conflicts "
                                  "with reserved %s name." %
                                  (member.identifier.name,
                                   self.maplikeOrSetlikeType),
                                  [self.location, member.location])
            # Check that there are no disallowed non-method members
            if (isAncestor or (member.isAttr() or member.isConst()) and
                member.identifier.name in self.disallowedNonMethodNames):
                raise WebIDLError("Member '%s' conflicts "
                                  "with reserved %s method." %
                                  (member.identifier.name,
                                   self.maplikeOrSetlikeType),
                                  [self.location, member.location])

    def expand(self, members, isJSImplemented):
        """
        In order to take advantage of all of the method machinery in Codegen,
        we generate our functions as if they were part of the interface
        specification during parsing.
        """
        def addMethod(name, allowExistingOperations, returnType, args=[],
                      chromeOnly=False, isPure=False, affectsNothing=False):
            """
            Create an IDLMethod based on the parameters passed in. chromeOnly is only
            True for read-only js implemented classes, to implement underscore
            prefixed convenience functions would otherwise not be available,
            unlike the case of C++ bindings. isPure is only True for
            idempotent functions, so it is not valid for things like keys,
            values, etc. that return a new object every time.

            """

            # Only add name to lists for collision checks if it's not chrome
            # only.
            if chromeOnly:
                name = "__" + name
            else:
                if not allowExistingOperations:
                    self.disallowedMemberNames.append(name)
                else:
                    self.disallowedNonMethodNames.append(name)

            # If allowExistingOperations is True, and another operation exists
            # with the same name as the one we're trying to add, don't add the
            # maplike/setlike operation. However, if the operation is static,
            # then fail by way of creating the function, which will cause a
            # naming conflict, per the spec.
            if allowExistingOperations:
                for m in members:
                    if m.identifier.name == name and m.isMethod() and not m.isStatic():
                        return

            method = IDLMethod(self.location,
                               IDLUnresolvedIdentifier(self.location, name, allowDoubleUnderscore=chromeOnly),
                               returnType, args, maplikeOrSetlike=self)

            # We need to be able to throw from declaration methods
            method.addExtendedAttributes(
                [IDLExtendedAttribute(self.location, ("Throws",))])
            if chromeOnly:
                method.addExtendedAttributes(
                    [IDLExtendedAttribute(self.location, ("ChromeOnly",))])
            if isPure:
                method.addExtendedAttributes(
                    [IDLExtendedAttribute(self.location, ("Pure",))])
            # Following attributes are used for keys/values/entries. Can't mark
            # them pure, since they return a new object each time they are run.
            if affectsNothing:
                method.addExtendedAttributes(
                    [IDLExtendedAttribute(self.location, ("DependsOn", "Everything")),
                     IDLExtendedAttribute(self.location, ("Affects", "Nothing"))])
            members.append(method)

        # Both maplike and setlike have a size attribute
        members.append(IDLAttribute(self.location,
                                    IDLUnresolvedIdentifier(BuiltinLocation("<auto-generated-identifier>"), "size"),
                                    BuiltinTypes[IDLBuiltinType.Types.unsigned_long],
                                    True,
                                    maplikeOrSetlike=self))
        self.reserved_ro_names = ["size"]

        # object entries()
        addMethod("entries", False, BuiltinTypes[IDLBuiltinType.Types.object],
                  affectsNothing=True)
        # object keys()
        addMethod("keys", False, BuiltinTypes[IDLBuiltinType.Types.object],
                  affectsNothing=True)
        # object values()
        addMethod("values", False, BuiltinTypes[IDLBuiltinType.Types.object],
                  affectsNothing=True)

        # void forEach(callback(valueType, keyType), thisVal)
        foreachArguments = [IDLArgument(self.location,
                                        IDLUnresolvedIdentifier(BuiltinLocation("<auto-generated-identifier>"),
                                                                "callback"),
                                        BuiltinTypes[IDLBuiltinType.Types.object]),
                            IDLArgument(self.location,
                                        IDLUnresolvedIdentifier(BuiltinLocation("<auto-generated-identifier>"),
                                                                "thisArg"),
                                        BuiltinTypes[IDLBuiltinType.Types.any],
                                        optional=True)]
        addMethod("forEach", False, BuiltinTypes[IDLBuiltinType.Types.void],
                  foreachArguments)

        def getKeyArg():
            return IDLArgument(self.location,
                               IDLUnresolvedIdentifier(self.location, "key"),
                               self.keyType)

        # boolean has(keyType key)
        addMethod("has", False, BuiltinTypes[IDLBuiltinType.Types.boolean],
                  [getKeyArg()], isPure=True)

        if not self.readonly:
            # void clear()
            addMethod("clear", True, BuiltinTypes[IDLBuiltinType.Types.void],
                      [])
            # boolean delete(keyType key)
            addMethod("delete", True,
                      BuiltinTypes[IDLBuiltinType.Types.boolean], [getKeyArg()])

        # Always generate underscored functions (e.g. __add, __clear) for js
        # implemented interfaces as convenience functions.
        if isJSImplemented:
            # void clear()
            addMethod("clear", True, BuiltinTypes[IDLBuiltinType.Types.void],
                      [], chromeOnly=True)
            # boolean delete(keyType key)
            addMethod("delete", True,
                      BuiltinTypes[IDLBuiltinType.Types.boolean], [getKeyArg()],
                      chromeOnly=True)

        if self.isSetlike():
            if not self.readonly:
                # Add returns the set object it just added to.
                # object add(keyType key)

                addMethod("add", True,
                          BuiltinTypes[IDLBuiltinType.Types.object], [getKeyArg()])
            if isJSImplemented:
                addMethod("add", True,
                          BuiltinTypes[IDLBuiltinType.Types.object], [getKeyArg()],
                          chromeOnly=True)
            return

        # If we get this far, we're a maplike declaration.

        # valueType get(keyType key)
        #
        # Note that instead of the value type, we're using any here. The
        # validity checks should happen as things are inserted into the map,
        # and using any as the return type makes code generation much simpler.
        #
        # TODO: Bug 1155340 may change this to use specific type to provide
        # more info to JIT.
        addMethod("get", False, BuiltinTypes[IDLBuiltinType.Types.any],
                  [getKeyArg()], isPure=True)

        def getValueArg():
            return IDLArgument(self.location,
                               IDLUnresolvedIdentifier(self.location, "value"),
                               self.valueType)

        if not self.readonly:
            addMethod("set", True, BuiltinTypes[IDLBuiltinType.Types.object],
                      [getKeyArg(), getValueArg()])
        if isJSImplemented:
            addMethod("set", True, BuiltinTypes[IDLBuiltinType.Types.object],
                      [getKeyArg(), getValueArg()], chromeOnly=True)

    def resolve(self, parentScope):
        self.keyType.resolveType(parentScope)
        self.valueType.resolveType(parentScope)

    def finish(self, scope):
        IDLInterfaceMember.finish(self, scope)
        if not self.keyType.isComplete():
            t = self.keyType.complete(scope)

            assert not isinstance(t, IDLUnresolvedType)
            assert not isinstance(t, IDLTypedefType)
            assert not isinstance(t.name, IDLUnresolvedIdentifier)
            self.keyType = t
        if not self.valueType.isComplete():
            t = self.valueType.complete(scope)

            assert not isinstance(t, IDLUnresolvedType)
            assert not isinstance(t, IDLTypedefType)
            assert not isinstance(t.name, IDLUnresolvedIdentifier)
            self.valueType = t

    def validate(self):
        IDLInterfaceMember.validate(self)

    def handleExtendedAttribute(self, attr):
        IDLInterfaceMember.handleExtendedAttribute(self, attr)

    def _getDependentObjects(self):
        return set([self.keyType, self.valueType])


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

        if identifier.name == "prototype":
            raise WebIDLError("The identifier of a constant must not be 'prototype'",
                              [location])

    def __str__(self):
        return "'%s' const '%s'" % (self.type, self.identifier)

    def finish(self, scope):
        IDLInterfaceMember.finish(self, scope)

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
        IDLInterfaceMember.validate(self)

    def handleExtendedAttribute(self, attr):
        identifier = attr.identifier()
        if identifier == "Exposed":
            convertExposedAttrToGlobalNameSet(attr, self._exposureGlobalNames)
        elif (identifier == "Pref" or
              identifier == "ChromeOnly" or
              identifier == "Func" or
              identifier == "AvailableIn" or
              identifier == "CheckAnyPermissions" or
              identifier == "CheckAllPermissions"):
            # Known attributes that we don't need to do anything with here
            pass
        else:
            raise WebIDLError("Unknown extended attribute %s on constant" % identifier,
                              [attr.location])
        IDLInterfaceMember.handleExtendedAttribute(self, attr)

    def _getDependentObjects(self):
        return set([self.type, self.value])


class IDLAttribute(IDLInterfaceMember):
    def __init__(self, location, identifier, type, readonly, inherit=False,
                 static=False, stringifier=False, maplikeOrSetlike=None):
        IDLInterfaceMember.__init__(self, location, identifier,
                                    IDLInterfaceMember.Tags.Attr)

        assert isinstance(type, IDLType)
        self.type = type
        self.readonly = readonly
        self.inherit = inherit
        self.static = static
        self.lenientThis = False
        self._unforgeable = False
        self.stringifier = stringifier
        self.enforceRange = False
        self.clamp = False
        self.slotIndex = None
        assert maplikeOrSetlike is None or isinstance(maplikeOrSetlike, IDLMaplikeOrSetlike)
        self.maplikeOrSetlike = maplikeOrSetlike
        self.dependsOn = "Everything"
        self.affects = "Everything"

        if static and identifier.name == "prototype":
            raise WebIDLError("The identifier of a static attribute must not be 'prototype'",
                              [location])

        if readonly and inherit:
            raise WebIDLError("An attribute cannot be both 'readonly' and 'inherit'",
                              [self.location])

    def isStatic(self):
        return self.static

    def __str__(self):
        return "'%s' attribute '%s'" % (self.type, self.identifier)

    def finish(self, scope):
        IDLInterfaceMember.finish(self, scope)

        if not self.type.isComplete():
            t = self.type.complete(scope)

            assert not isinstance(t, IDLUnresolvedType)
            assert not isinstance(t, IDLTypedefType)
            assert not isinstance(t.name, IDLUnresolvedIdentifier)
            self.type = t

        if self.type.isDictionary() and not self.getExtendedAttribute("Cached"):
            raise WebIDLError("An attribute cannot be of a dictionary type",
                              [self.location])
        if self.type.isSequence() and not self.getExtendedAttribute("Cached"):
            raise WebIDLError("A non-cached attribute cannot be of a sequence "
                              "type", [self.location])
        if self.type.isMozMap() and not self.getExtendedAttribute("Cached"):
            raise WebIDLError("A non-cached attribute cannot be of a MozMap "
                              "type", [self.location])
        if self.type.isUnion():
            for f in self.type.unroll().flatMemberTypes:
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
                if f.isMozMap():
                    raise WebIDLError("An attribute cannot be of a union "
                                      "type if one of its member types (or "
                                      "one of its member types's member "
                                      "types, and so on) is a MozMap "
                                      "type", [self.location, f.location])
        if not self.type.isInterface() and self.getExtendedAttribute("PutForwards"):
            raise WebIDLError("An attribute with [PutForwards] must have an "
                              "interface type as its type", [self.location])

        if not self.type.isInterface() and self.getExtendedAttribute("SameObject"):
            raise WebIDLError("An attribute with [SameObject] must have an "
                              "interface type as its type", [self.location])

    def validate(self):
        IDLInterfaceMember.validate(self)

        if ((self.getExtendedAttribute("Cached") or
             self.getExtendedAttribute("StoreInSlot")) and
            not self.affects == "Nothing"):
            raise WebIDLError("Cached attributes and attributes stored in "
                              "slots must be Constant or Pure or "
                              "Affects=Nothing, since the getter won't always "
                              "be called.",
                              [self.location])
        if self.getExtendedAttribute("Frozen"):
            if (not self.type.isSequence() and not self.type.isDictionary() and
                not self.type.isMozMap()):
                raise WebIDLError("[Frozen] is only allowed on "
                                  "sequence-valued, dictionary-valued, and "
                                  "MozMap-valued attributes",
                                  [self.location])
        if not self.type.unroll().isExposedInAllOf(self.exposureSet):
            raise WebIDLError("Attribute returns a type that is not exposed "
                              "everywhere where the attribute is exposed",
                              [self.location])

    def handleExtendedAttribute(self, attr):
        identifier = attr.identifier()
        if identifier == "SetterThrows" and self.readonly:
            raise WebIDLError("Readonly attributes must not be flagged as "
                              "[SetterThrows]",
                              [self.location])
        elif (((identifier == "Throws" or identifier == "GetterThrows") and
               self.getExtendedAttribute("StoreInSlot")) or
              (identifier == "StoreInSlot" and
               (self.getExtendedAttribute("Throws") or
                self.getExtendedAttribute("GetterThrows")))):
            raise WebIDLError("Throwing things can't be [StoreInSlot]",
                              [attr.location])
        elif identifier == "LenientThis":
            if not attr.noArguments():
                raise WebIDLError("[LenientThis] must take no arguments",
                                  [attr.location])
            if self.isStatic():
                raise WebIDLError("[LenientThis] is only allowed on non-static "
                                  "attributes", [attr.location, self.location])
            if self.getExtendedAttribute("CrossOriginReadable"):
                raise WebIDLError("[LenientThis] is not allowed in combination "
                                  "with [CrossOriginReadable]",
                                  [attr.location, self.location])
            if self.getExtendedAttribute("CrossOriginWritable"):
                raise WebIDLError("[LenientThis] is not allowed in combination "
                                  "with [CrossOriginWritable]",
                                  [attr.location, self.location])
            self.lenientThis = True
        elif identifier == "Unforgeable":
            if self.isStatic():
                raise WebIDLError("[Unforgeable] is only allowed on non-static "
                                  "attributes", [attr.location, self.location])
            self._unforgeable = True
        elif identifier == "SameObject" and not self.readonly:
            raise WebIDLError("[SameObject] only allowed on readonly attributes",
                              [attr.location, self.location])
        elif identifier == "Constant" and not self.readonly:
            raise WebIDLError("[Constant] only allowed on readonly attributes",
                              [attr.location, self.location])
        elif identifier == "PutForwards":
            if not self.readonly:
                raise WebIDLError("[PutForwards] is only allowed on readonly "
                                  "attributes", [attr.location, self.location])
            if self.isStatic():
                raise WebIDLError("[PutForwards] is only allowed on non-static "
                                  "attributes", [attr.location, self.location])
            if self.getExtendedAttribute("Replaceable") is not None:
                raise WebIDLError("[PutForwards] and [Replaceable] can't both "
                                  "appear on the same attribute",
                                  [attr.location, self.location])
            if not attr.hasValue():
                raise WebIDLError("[PutForwards] takes an identifier",
                                  [attr.location, self.location])
        elif identifier == "Replaceable":
            if not attr.noArguments():
                raise WebIDLError("[Replaceable] must take no arguments",
                                  [attr.location])
            if not self.readonly:
                raise WebIDLError("[Replaceable] is only allowed on readonly "
                                  "attributes", [attr.location, self.location])
            if self.isStatic():
                raise WebIDLError("[Replaceable] is only allowed on non-static "
                                  "attributes", [attr.location, self.location])
            if self.getExtendedAttribute("PutForwards") is not None:
                raise WebIDLError("[PutForwards] and [Replaceable] can't both "
                                  "appear on the same attribute",
                                  [attr.location, self.location])
        elif identifier == "LenientFloat":
            if self.readonly:
                raise WebIDLError("[LenientFloat] used on a readonly attribute",
                                  [attr.location, self.location])
            if not self.type.includesRestrictedFloat():
                raise WebIDLError("[LenientFloat] used on an attribute with a "
                                  "non-restricted-float type",
                                  [attr.location, self.location])
        elif identifier == "EnforceRange":
            if self.readonly:
                raise WebIDLError("[EnforceRange] used on a readonly attribute",
                                  [attr.location, self.location])
            self.enforceRange = True
        elif identifier == "Clamp":
            if self.readonly:
                raise WebIDLError("[Clamp] used on a readonly attribute",
                                  [attr.location, self.location])
            self.clamp = True
        elif identifier == "StoreInSlot":
            if self.getExtendedAttribute("Cached"):
                raise WebIDLError("[StoreInSlot] and [Cached] must not be "
                                  "specified on the same attribute",
                                  [attr.location, self.location])
        elif identifier == "Cached":
            if self.getExtendedAttribute("StoreInSlot"):
                raise WebIDLError("[Cached] and [StoreInSlot] must not be "
                                  "specified on the same attribute",
                                  [attr.location, self.location])
        elif (identifier == "CrossOriginReadable" or
              identifier == "CrossOriginWritable"):
            if not attr.noArguments() and identifier == "CrossOriginReadable":
                raise WebIDLError("[%s] must take no arguments" % identifier,
                                  [attr.location])
            if self.isStatic():
                raise WebIDLError("[%s] is only allowed on non-static "
                                  "attributes" % identifier,
                                  [attr.location, self.location])
            if self.getExtendedAttribute("LenientThis"):
                raise WebIDLError("[LenientThis] is not allowed in combination "
                                  "with [%s]" % identifier,
                                  [attr.location, self.location])
        elif identifier == "Exposed":
            convertExposedAttrToGlobalNameSet(attr, self._exposureGlobalNames)
        elif identifier == "Pure":
            if not attr.noArguments():
                raise WebIDLError("[Pure] must take no arguments",
                                  [attr.location])
            self._setDependsOn("DOMState")
            self._setAffects("Nothing")
        elif identifier == "Constant" or identifier == "SameObject":
            if not attr.noArguments():
                raise WebIDLError("[%s] must take no arguments" % identifier,
                                  [attr.location])
            self._setDependsOn("Nothing")
            self._setAffects("Nothing")
        elif identifier == "Affects":
            if not attr.hasValue():
                raise WebIDLError("[Affects] takes an identifier",
                                  [attr.location])
            self._setAffects(attr.value())
        elif identifier == "DependsOn":
            if not attr.hasValue():
                raise WebIDLError("[DependsOn] takes an identifier",
                                  [attr.location])
            if (attr.value() != "Everything" and attr.value() != "DOMState" and
                not self.readonly):
                raise WebIDLError("[DependsOn=%s] only allowed on "
                                  "readonly attributes" % attr.value(),
                                  [attr.location, self.location])
            self._setDependsOn(attr.value())
        elif (identifier == "Pref" or
              identifier == "Deprecated" or
              identifier == "SetterThrows" or
              identifier == "Throws" or
              identifier == "GetterThrows" or
              identifier == "ChromeOnly" or
              identifier == "Func" or
              identifier == "Frozen" or
              identifier == "AvailableIn" or
              identifier == "NewObject" or
              identifier == "UnsafeInPrerendering" or
              identifier == "CheckAnyPermissions" or
              identifier == "CheckAllPermissions" or
              identifier == "BinaryName"):
            # Known attributes that we don't need to do anything with here
            pass
        else:
            raise WebIDLError("Unknown extended attribute %s on attribute" % identifier,
                              [attr.location])
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

    def isMaplikeOrSetlikeAttr(self):
        """
        True if this attribute was generated from an interface with
        maplike/setlike (e.g. this is the size attribute for
        maplike/setlike)
        """
        return self.maplikeOrSetlike is not None

    def isUnforgeable(self):
        return self._unforgeable

    def _getDependentObjects(self):
        return set([self.type])


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
        self._allowTreatNonCallableAsNull = False

        assert not variadic or optional
        assert not variadic or not defaultValue

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
                                      [self.location])
                self.clamp = True
            elif identifier == "EnforceRange":
                if not attribute.noArguments():
                    raise WebIDLError("[EnforceRange] must take no arguments",
                                      [attribute.location])
                if self.clamp:
                    raise WebIDLError("[EnforceRange] and [Clamp] are mutually exclusive",
                                      [self.location])
                self.enforceRange = True
            elif identifier == "TreatNonCallableAsNull":
                self._allowTreatNonCallableAsNull = True
            else:
                raise WebIDLError("Unhandled extended attribute on %s" %
                                  ("a dictionary member" if self.dictionaryMember else
                                   "an argument"),
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

        if ((self.type.isDictionary() or
             self.type.isUnion() and self.type.unroll().hasDictionaryType()) and
            self.optional and not self.defaultValue and not self.variadic):
            # Default optional non-variadic dictionaries to null,
            # for simplicity, so the codegen doesn't have to special-case this.
            self.defaultValue = IDLNullValue(self.location)
        elif self.type.isAny():
            assert (self.defaultValue is None or
                    isinstance(self.defaultValue, IDLNullValue))
            # optional 'any' values always have a default value
            if self.optional and not self.defaultValue and not self.variadic:
                # Set the default value to undefined, for simplicity, so the
                # codegen doesn't have to special-case this.
                self.defaultValue = IDLUndefinedValue(self.location)

        # Now do the coercing thing; this needs to happen after the
        # above creation of a default value.
        if self.defaultValue:
            self.defaultValue = self.defaultValue.coerceToType(self.type,
                                                               self.location)
            assert self.defaultValue

    def allowTreatNonCallableAsNull(self):
        return self._allowTreatNonCallableAsNull

    def _getDependentObjects(self):
        deps = set([self.type])
        if self.defaultValue:
            deps.add(self.defaultValue)
        return deps

    def canHaveMissingValue(self):
        return self.optional and not self.defaultValue


class IDLCallback(IDLObjectWithScope):
    def __init__(self, location, parentScope, identifier, returnType, arguments):
        assert isinstance(returnType, IDLType)

        self._returnType = returnType
        # Clone the list
        self._arguments = list(arguments)

        IDLObjectWithScope.__init__(self, location, parentScope, identifier)

        for (returnType, arguments) in self.signatures():
            for argument in arguments:
                argument.resolve(self)

        self._treatNonCallableAsNull = False
        self._treatNonObjectAsNull = False

    def module(self):
        return self.location.filename().split('/')[-1].split('.webidl')[0] + 'Binding'

    def isCallback(self):
        return True

    def signatures(self):
        return [(self._returnType, self._arguments)]

    def finish(self, scope):
        if not self._returnType.isComplete():
            type = self._returnType.complete(scope)

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

    def addExtendedAttributes(self, attrs):
        unhandledAttrs = []
        for attr in attrs:
            if attr.identifier() == "TreatNonCallableAsNull":
                self._treatNonCallableAsNull = True
            elif attr.identifier() == "TreatNonObjectAsNull":
                self._treatNonObjectAsNull = True
            else:
                unhandledAttrs.append(attr)
        if self._treatNonCallableAsNull and self._treatNonObjectAsNull:
            raise WebIDLError("Cannot specify both [TreatNonCallableAsNull] "
                              "and [TreatNonObjectAsNull]", [self.location])
        if len(unhandledAttrs) != 0:
            IDLType.addExtendedAttributes(self, unhandledAttrs)

    def _getDependentObjects(self):
        return set([self._returnType] + self._arguments)


class IDLCallbackType(IDLType):
    def __init__(self, location, callback):
        IDLType.__init__(self, location, callback.identifier.name)
        self.callback = callback

    def isCallback(self):
        return True

    def tag(self):
        return IDLType.Tags.callback

    def isDistinguishableFrom(self, other):
        if other.isPromise():
            return False
        if other.isUnion():
            # Just forward to the union; it'll deal
            return other.isDistinguishableFrom(self)
        return (other.isPrimitive() or other.isString() or other.isEnum() or
                other.isNonCallbackInterface() or other.isDate() or
                other.isSequence())

    def _getDependentObjects(self):
        return self.callback._getDependentObjects()


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

    def _getDependentObjects(self):
        deps = set(self.arguments)
        deps.add(self.returnType)
        return deps


class IDLMethod(IDLInterfaceMember, IDLScope):

    Special = enum(
        'Getter',
        'Setter',
        'Creator',
        'Deleter',
        'LegacyCaller',
        base=IDLInterfaceMember.Special
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
                 legacycaller=False, stringifier=False, jsonifier=False,
                 maplikeOrSetlike=None):
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
        assert isinstance(jsonifier, bool)
        self._jsonifier = jsonifier
        assert maplikeOrSetlike is None or isinstance(maplikeOrSetlike, IDLMaplikeOrSetlike)
        self.maplikeOrSetlike = maplikeOrSetlike
        self._specialType = specialType
        self._unforgeable = False
        self.dependsOn = "Everything"
        self.affects = "Everything"
        self.aliases = []

        if static and identifier.name == "prototype":
            raise WebIDLError("The identifier of a static operation must not be 'prototype'",
                              [location])

        self.assertSignatureConstraints()

    def __str__(self):
        return "Method '%s'" % self.identifier

    def assertSignatureConstraints(self):
        if self._getter or self._deleter:
            assert len(self._overloads) == 1
            overload = self._overloads[0]
            arguments = overload.arguments
            assert len(arguments) == 1
            assert (arguments[0].type == BuiltinTypes[IDLBuiltinType.Types.domstring] or
                    arguments[0].type == BuiltinTypes[IDLBuiltinType.Types.unsigned_long])
            assert not arguments[0].optional and not arguments[0].variadic
            assert not self._getter or not overload.returnType.isVoid()

        if self._setter or self._creator:
            assert len(self._overloads) == 1
            arguments = self._overloads[0].arguments
            assert len(arguments) == 2
            assert (arguments[0].type == BuiltinTypes[IDLBuiltinType.Types.domstring] or
                    arguments[0].type == BuiltinTypes[IDLBuiltinType.Types.unsigned_long])
            assert not arguments[0].optional and not arguments[0].variadic
            assert not arguments[1].optional and not arguments[1].variadic

        if self._stringifier:
            assert len(self._overloads) == 1
            overload = self._overloads[0]
            assert len(overload.arguments) == 0
            assert overload.returnType == BuiltinTypes[IDLBuiltinType.Types.domstring]

        if self._jsonifier:
            assert len(self._overloads) == 1
            overload = self._overloads[0]
            assert len(overload.arguments) == 0
            assert overload.returnType == BuiltinTypes[IDLBuiltinType.Types.object]

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
        assert (self._specialType == IDLMethod.NamedOrIndexed.Named or
                self._specialType == IDLMethod.NamedOrIndexed.Indexed)
        return self._specialType == IDLMethod.NamedOrIndexed.Named

    def isIndexed(self):
        assert (self._specialType == IDLMethod.NamedOrIndexed.Named or
                self._specialType == IDLMethod.NamedOrIndexed.Indexed)
        return self._specialType == IDLMethod.NamedOrIndexed.Indexed

    def isLegacycaller(self):
        return self._legacycaller

    def isStringifier(self):
        return self._stringifier

    def isJsonifier(self):
        return self._jsonifier

    def isMaplikeOrSetlikeMethod(self):
        """
        True if this method was generated as part of a
        maplike/setlike/etc interface (e.g. has/get methods)
        """
        return self.maplikeOrSetlike is not None

    def hasOverloads(self):
        return self._hasOverloads

    def isIdentifierLess(self):
        """
        True if the method name started with __, and if the method is not a
        maplike/setlike method. Interfaces with maplike/setlike will generate
        methods starting with __ for chrome only backing object access in JS
        implemented interfaces, so while these functions use what is considered
        an non-identifier name, they actually DO have an identifier.
        """
        return (self.identifier.name[:2] == "__" and
                not self.isMaplikeOrSetlikeMethod())

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
        assert not self.isJsonifier()
        assert not method.isJsonifier()

        return self

    def signatures(self):
        return [(overload.returnType, overload.arguments) for overload in
                self._overloads]

    def finish(self, scope):
        IDLInterfaceMember.finish(self, scope)

        for overload in self._overloads:
            returnType = overload.returnType
            if not returnType.isComplete():
                returnType = returnType.complete(scope)
                assert not isinstance(returnType, IDLUnresolvedType)
                assert not isinstance(returnType, IDLTypedefType)
                assert not isinstance(returnType.name, IDLUnresolvedIdentifier)
                overload.returnType = returnType

            for argument in overload.arguments:
                if not argument.isComplete():
                    argument.complete(scope)
                assert argument.type.isComplete()

        # Now compute various information that will be used by the
        # WebIDL overload resolution algorithm.
        self.maxArgCount = max(len(s[1]) for s in self.signatures())
        self.allowedArgCounts = [i for i in range(self.maxArgCount+1)
                                 if len(self.signaturesForArgCount(i)) != 0]

    def validate(self):
        IDLInterfaceMember.validate(self)

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

        overloadWithPromiseReturnType = None
        overloadWithoutPromiseReturnType = None
        for overload in self._overloads:
            returnType = overload.returnType
            if not returnType.unroll().isExposedInAllOf(self.exposureSet):
                raise WebIDLError("Overload returns a type that is not exposed "
                                  "everywhere where the method is exposed",
                                  [overload.location])

            variadicArgument = None

            arguments = overload.arguments
            for (idx, argument) in enumerate(arguments):
                assert argument.type.isComplete()

                if ((argument.type.isDictionary() and
                     argument.type.inner.canBeEmpty())or
                    (argument.type.isUnion() and
                     argument.type.unroll().hasPossiblyEmptyDictionaryType())):
                    # Optional dictionaries and unions containing optional
                    # dictionaries at the end of the list or followed by
                    # optional arguments must be optional.
                    if (not argument.optional and
                        all(arg.optional for arg in arguments[idx+1:])):
                        raise WebIDLError("Dictionary argument or union "
                                          "argument containing a dictionary "
                                          "not followed by a required argument "
                                          "must be optional",
                                          [argument.location])

                    # An argument cannot be a Nullable Dictionary
                    if argument.type.nullable():
                        raise WebIDLError("An argument cannot be a nullable "
                                          "dictionary or nullable union "
                                          "containing a dictionary",
                                          [argument.location])

                # Only the last argument can be variadic
                if variadicArgument:
                    raise WebIDLError("Variadic argument is not last argument",
                                      [variadicArgument.location])
                if argument.variadic:
                    variadicArgument = argument

            if returnType.isPromise():
                overloadWithPromiseReturnType = overload
            else:
                overloadWithoutPromiseReturnType = overload

        # Make sure either all our overloads return Promises or none do
        if overloadWithPromiseReturnType and overloadWithoutPromiseReturnType:
            raise WebIDLError("We have overloads with both Promise and "
                              "non-Promise return types",
                              [overloadWithPromiseReturnType.location,
                               overloadWithoutPromiseReturnType.location])

        if overloadWithPromiseReturnType and self._legacycaller:
            raise WebIDLError("May not have a Promise return type for a "
                              "legacycaller.",
                              [overloadWithPromiseReturnType.location])

        if self.getExtendedAttribute("StaticClassOverride") and not \
           (self.identifier.scope.isJSImplemented() and self.isStatic()):
            raise WebIDLError("StaticClassOverride can be applied to static"
                              " methods on JS-implemented classes only.",
                              [self.location])

    def overloadsForArgCount(self, argc):
        return [overload for overload in self._overloads if
                len(overload.arguments) == argc or
                (len(overload.arguments) > argc and
                 all(arg.optional for arg in overload.arguments[argc:])) or
                (len(overload.arguments) < argc and
                 len(overload.arguments) > 0 and
                 overload.arguments[-1].variadic)]

    def signaturesForArgCount(self, argc):
        return [(overload.returnType, overload.arguments) for overload
                in self.overloadsForArgCount(argc)]

    def locationsForArgCount(self, argc):
        return [overload.location for overload in self.overloadsForArgCount(argc)]

    def distinguishingIndexForArgCount(self, argc):
        def isValidDistinguishingIndex(idx, signatures):
            for (firstSigIndex, (firstRetval, firstArgs)) in enumerate(signatures[:-1]):
                for (secondRetval, secondArgs) in signatures[firstSigIndex+1:]:
                    if idx < len(firstArgs):
                        firstType = firstArgs[idx].type
                    else:
                        assert(firstArgs[-1].variadic)
                        firstType = firstArgs[-1].type
                    if idx < len(secondArgs):
                        secondType = secondArgs[idx].type
                    else:
                        assert(secondArgs[-1].variadic)
                        secondType = secondArgs[-1].type
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
        if identifier == "GetterThrows":
            raise WebIDLError("Methods must not be flagged as "
                              "[GetterThrows]",
                              [attr.location, self.location])
        elif identifier == "SetterThrows":
            raise WebIDLError("Methods must not be flagged as "
                              "[SetterThrows]",
                              [attr.location, self.location])
        elif identifier == "Unforgeable":
            if self.isStatic():
                raise WebIDLError("[Unforgeable] is only allowed on non-static "
                                  "methods", [attr.location, self.location])
            self._unforgeable = True
        elif identifier == "SameObject":
            raise WebIDLError("Methods must not be flagged as [SameObject]",
                              [attr.location, self.location])
        elif identifier == "Constant":
            raise WebIDLError("Methods must not be flagged as [Constant]",
                              [attr.location, self.location])
        elif identifier == "PutForwards":
            raise WebIDLError("Only attributes support [PutForwards]",
                              [attr.location, self.location])
        elif identifier == "LenientFloat":
            # This is called before we've done overload resolution
            assert len(self.signatures()) == 1
            sig = self.signatures()[0]
            if not sig[0].isVoid():
                raise WebIDLError("[LenientFloat] used on a non-void method",
                                  [attr.location, self.location])
            if not any(arg.type.includesRestrictedFloat() for arg in sig[1]):
                raise WebIDLError("[LenientFloat] used on an operation with no "
                                  "restricted float type arguments",
                                  [attr.location, self.location])
        elif identifier == "Exposed":
            convertExposedAttrToGlobalNameSet(attr, self._exposureGlobalNames)
        elif (identifier == "CrossOriginCallable" or
              identifier == "WebGLHandlesContextLoss"):
            # Known no-argument attributes.
            if not attr.noArguments():
                raise WebIDLError("[%s] must take no arguments" % identifier,
                                  [attr.location])
        elif identifier == "Pure":
            if not attr.noArguments():
                raise WebIDLError("[Pure] must take no arguments",
                                  [attr.location])
            self._setDependsOn("DOMState")
            self._setAffects("Nothing")
        elif identifier == "Affects":
            if not attr.hasValue():
                raise WebIDLError("[Affects] takes an identifier",
                                  [attr.location])
            self._setAffects(attr.value())
        elif identifier == "DependsOn":
            if not attr.hasValue():
                raise WebIDLError("[DependsOn] takes an identifier",
                                  [attr.location])
            self._setDependsOn(attr.value())
        elif identifier == "Alias":
            if not attr.hasValue():
                raise WebIDLError("[Alias] takes an identifier or string",
                                  [attr.location])
            self._addAlias(attr.value())
        elif (identifier == "Throws" or
              identifier == "NewObject" or
              identifier == "ChromeOnly" or
              identifier == "UnsafeInPrerendering" or
              identifier == "Pref" or
              identifier == "Deprecated" or
              identifier == "Func" or
              identifier == "AvailableIn" or
              identifier == "CheckAnyPermissions" or
              identifier == "CheckAllPermissions" or
              identifier == "BinaryName" or
              identifier == "MethodIdentityTestable" or
              identifier == "StaticClassOverride"):
            # Known attributes that we don't need to do anything with here
            pass
        else:
            raise WebIDLError("Unknown extended attribute %s on method" % identifier,
                              [attr.location])
        IDLInterfaceMember.handleExtendedAttribute(self, attr)

    def returnsPromise(self):
        return self._overloads[0].returnType.isPromise()

    def isUnforgeable(self):
        return self._unforgeable

    def _getDependentObjects(self):
        deps = set()
        for overload in self._overloads:
            deps.update(overload._getDependentObjects())
        return deps


class IDLImplementsStatement(IDLObject):
    def __init__(self, location, implementor, implementee):
        IDLObject.__init__(self, location)
        self.implementor = implementor
        self.implementee = implementee
        self._finished = False

    def finish(self, scope):
        if self._finished:
            return
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
        self.implementor = implementor
        self.implementee = implementee

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
        return len(self._tuple) >= 2 and isinstance(self._tuple[1], str)

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

    def t_FLOATLITERAL(self, t):
        r'(-?(([0-9]+\.[0-9]*|[0-9]*\.[0-9]+)([Ee][+-]?[0-9]+)?|[0-9]+[Ee][+-]?[0-9]+|Infinity))|NaN'
        t.value = float(t.value)
        return t

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

    def t_IDENTIFIER(self, t):
        r'[A-Z_a-z][0-9A-Z_a-z-]*'
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
        "serializer": "SERIALIZER",
        "stringifier": "STRINGIFIER",
        "jsonifier": "JSONIFIER",
        "unrestricted": "UNRESTRICTED",
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
        "ByteString": "BYTESTRING",
        "USVString": "USVSTRING",
        "any": "ANY",
        "boolean": "BOOLEAN",
        "byte": "BYTE",
        "double": "DOUBLE",
        "float": "FLOAT",
        "long": "LONG",
        "object": "OBJECT",
        "octet": "OCTET",
        "Promise": "PROMISE",
        "required": "REQUIRED",
        "sequence": "SEQUENCE",
        "MozMap": "MOZMAP",
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
        "SharedArrayBuffer": "SHAREDARRAYBUFFER",
        "or": "OR",
        "maplike": "MAPLIKE",
        "setlike": "SETLIKE"
        }

    tokens.extend(keywords.values())

    def t_error(self, t):
        raise WebIDLError("Unrecognized Input",
                          [Location(lexer=self.lexer,
                                    lineno=self.lexer.lineno,
                                    lexpos=self.lexer.lexpos,
                                    filename=self.filename)])

    def __init__(self, outputdir, lexer=None):
        if lexer:
            self.lexer = lexer
        else:
            self.lexer = lex.lex(object=self,
                                 outputdir=outputdir,
                                 lextab='webidllex',
                                 reflags=re.DOTALL)


class SqueakyCleanLogger(object):
    errorWhitelist = [
        # Web IDL defines the WHITESPACE token, but doesn't actually
        # use it ... so far.
        "Token 'WHITESPACE' defined, but not used",
        # And that means we have an unused token
        "There is 1 unused token",
        # Web IDL defines a OtherOrComma rule that's only used in
        # ExtendedAttributeInner, which we don't use yet.
        "Rule 'OtherOrComma' defined, but not used",
        # And an unused rule
        "There is 1 unused rule",
        # And the OtherOrComma grammar symbol is unreachable.
        "Symbol 'OtherOrComma' is unreachable",
        # Which means the Other symbol is unreachable.
        "Symbol 'Other' is unreachable",
        ]

    def __init__(self):
        self.errors = []

    def debug(self, msg, *args, **kwargs):
        pass
    info = debug

    def warning(self, msg, *args, **kwargs):
        if msg == "%s:%d: Rule '%s' defined, but not used":
            # Munge things so we don't have to hardcode filenames and
            # line numbers in our whitelist.
            whitelistmsg = "Rule '%s' defined, but not used"
            whitelistargs = args[2:]
        else:
            whitelistmsg = msg
            whitelistargs = args
        if (whitelistmsg % whitelistargs) not in SqueakyCleanLogger.errorWhitelist:
            self.errors.append(msg % args)
    error = warning

    def reportGrammarErrors(self):
        if self.errors:
            raise WebIDLError("\n".join(self.errors), [])


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
        assert p[1]  # We might not have implemented something ...

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
        parent = p[3]

        try:
            existingObj = self.globalScope()._lookupIdentifier(identifier)
            if existingObj:
                p[0] = existingObj
                if not isinstance(p[0], IDLInterface):
                    raise WebIDLError("Interface has the same name as "
                                      "non-interface object",
                                      [location, p[0].location])
                p[0].setNonPartial(location, parent, members)
                return
        except Exception, ex:
            if isinstance(ex, WebIDLError):
                raise ex
            pass

        p[0] = IDLInterface(location, self.globalScope(), identifier, parent,
                            members, isKnownNonPartial=True)

    def p_InterfaceForwardDecl(self, p):
        """
            Interface : INTERFACE IDENTIFIER SEMICOLON
        """
        location = self.getLocation(p, 1)
        identifier = IDLUnresolvedIdentifier(self.getLocation(p, 2), p[2])

        try:
            if self.globalScope()._lookupIdentifier(identifier):
                p[0] = self.globalScope()._lookupIdentifier(identifier)
                if not isinstance(p[0], IDLExternalInterface):
                    raise WebIDLError("Name collision between external "
                                      "interface declaration for identifier "
                                      "%s and %s" % (identifier.name, p[0]),
                                      [location, p[0].location])
                return
        except Exception, ex:
            if isinstance(ex, WebIDLError):
                raise ex
            pass

        p[0] = IDLExternalInterface(location, self.globalScope(), identifier)

    def p_PartialInterface(self, p):
        """
            PartialInterface : PARTIAL INTERFACE IDENTIFIER LBRACE InterfaceMembers RBRACE SEMICOLON
        """
        location = self.getLocation(p, 2)
        identifier = IDLUnresolvedIdentifier(self.getLocation(p, 3), p[3])
        members = p[5]

        nonPartialInterface = None
        try:
            nonPartialInterface = self.globalScope()._lookupIdentifier(identifier)
            if nonPartialInterface:
                if not isinstance(nonPartialInterface, IDLInterface):
                    raise WebIDLError("Partial interface has the same name as "
                                      "non-interface object",
                                      [location, nonPartialInterface.location])
        except Exception, ex:
            if isinstance(ex, WebIDLError):
                raise ex
            pass

        if not nonPartialInterface:
            nonPartialInterface = IDLInterface(location, self.globalScope(),
                                               identifier, None,
                                               [], isKnownNonPartial=False)
        partialInterface = IDLPartialInterface(location, identifier, members,
                                               nonPartialInterface)
        p[0] = partialInterface

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
                            | AttributeOrOperationOrMaplikeOrSetlike
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
            DictionaryMember : Required Type IDENTIFIER Default SEMICOLON
        """
        # These quack a lot like optional arguments, so just treat them that way.
        t = p[2]
        assert isinstance(t, IDLType)
        identifier = IDLUnresolvedIdentifier(self.getLocation(p, 3), p[3])
        defaultValue = p[4]
        optional = not p[1]

        if not optional and defaultValue:
            raise WebIDLError("Required dictionary members can't have a default value.",
                              [self.getLocation(p, 4)])

        p[0] = IDLArgument(self.getLocation(p, 3), identifier, t,
                           optional=optional,
                           defaultValue=defaultValue, variadic=False,
                           dictionaryMember=True)

    def p_Default(self, p):
        """
            Default : EQUALS DefaultValue
                    |
        """
        if len(p) > 1:
            p[0] = p[2]
        else:
            p[0] = None

    def p_DefaultValue(self, p):
        """
            DefaultValue : ConstValue
                         | LBRACKET RBRACKET
        """
        if len(p) == 2:
            p[0] = p[1]
        else:
            assert len(p) == 3  # Must be []
            p[0] = IDLEmptySequenceValue(self.getLocation(p, 1))

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
            EnumValueList : STRING EnumValueListComma
        """
        p[0] = [p[1]]
        p[0].extend(p[2])

    def p_EnumValueListComma(self, p):
        """
            EnumValueListComma : COMMA EnumValueListString
        """
        p[0] = p[2]

    def p_EnumValueListCommaEmpty(self, p):
        """
            EnumValueListComma :
        """
        p[0] = []

    def p_EnumValueListString(self, p):
        """
            EnumValueListString : STRING EnumValueListComma
        """
        p[0] = [p[1]]
        p[0].extend(p[2])

    def p_EnumValueListStringEmpty(self, p):
        """
            EnumValueListString :
        """
        p[0] = []

    def p_CallbackRest(self, p):
        """
            CallbackRest : IDENTIFIER EQUALS ReturnType LPAREN ArgumentList RPAREN SEMICOLON
        """
        identifier = IDLUnresolvedIdentifier(self.getLocation(p, 1), p[1])
        p[0] = IDLCallback(self.getLocation(p, 1), self.globalScope(),
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
        typedef = IDLTypedef(self.getLocation(p, 1), self.globalScope(),
                             p[2], p[3])
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
        if integerType is None:
            raise WebIDLError("Integer literal out of range", [location])

        p[0] = IDLValue(location, integerType, p[1])

    def p_ConstValueFloat(self, p):
        """
            ConstValue : FLOATLITERAL
        """
        location = self.getLocation(p, 1)
        p[0] = IDLValue(location, BuiltinTypes[IDLBuiltinType.Types.unrestricted_float], p[1])

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

    def p_AttributeOrOperationOrMaplikeOrSetlike(self, p):
        """
            AttributeOrOperationOrMaplikeOrSetlike : Attribute
                                                   | Maplike
                                                   | Setlike
                                                   | Operation
        """
        p[0] = p[1]

    def p_Setlike(self, p):
        """
            Setlike : ReadOnly SETLIKE LT Type GT SEMICOLON
        """
        readonly = p[1]
        maplikeOrSetlikeType = p[2]
        location = self.getLocation(p, 2)
        identifier = IDLUnresolvedIdentifier(location, "__setlike",
                                             allowDoubleUnderscore=True)
        keyType = p[4]
        valueType = keyType
        p[0] = IDLMaplikeOrSetlike(location, identifier, maplikeOrSetlikeType,
                                   readonly, keyType, valueType)

    def p_Maplike(self, p):
        """
            Maplike : ReadOnly MAPLIKE LT Type COMMA Type GT SEMICOLON
        """
        readonly = p[1]
        maplikeOrSetlikeType = p[2]
        location = self.getLocation(p, 2)
        identifier = IDLUnresolvedIdentifier(location, "__maplike",
                                             allowDoubleUnderscore=True)
        keyType = p[4]
        valueType = p[6]
        p[0] = IDLMaplikeOrSetlike(location, identifier, maplikeOrSetlikeType,
                                   readonly, keyType, valueType)

    def p_AttributeWithQualifier(self, p):
        """
            Attribute : Qualifier AttributeRest
        """
        static = IDLInterfaceMember.Special.Static in p[1]
        stringifier = IDLInterfaceMember.Special.Stringifier in p[1]
        (location, identifier, type, readonly) = p[2]
        p[0] = IDLAttribute(location, identifier, type, readonly,
                            static=static, stringifier=stringifier)

    def p_AttributeInherited(self, p):
        """
            Attribute : INHERIT AttributeRest
        """
        (location, identifier, type, readonly) = p[2]
        p[0] = IDLAttribute(location, identifier, type, readonly, inherit=True)

    def p_Attribute(self, p):
        """
            Attribute : AttributeRest
        """
        (location, identifier, type, readonly) = p[1]
        p[0] = IDLAttribute(location, identifier, type, readonly, inherit=False)

    def p_AttributeRest(self, p):
        """
            AttributeRest : ReadOnly ATTRIBUTE Type AttributeName SEMICOLON
        """
        location = self.getLocation(p, 2)
        readonly = p[1]
        t = p[3]
        identifier = IDLUnresolvedIdentifier(self.getLocation(p, 4), p[4])
        p[0] = (location, identifier, t, readonly)

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

    def p_Operation(self, p):
        """
            Operation : Qualifiers OperationRest
        """
        qualifiers = p[1]

        # Disallow duplicates in the qualifier set
        if not len(set(qualifiers)) == len(qualifiers):
            raise WebIDLError("Duplicate qualifiers are not allowed",
                              [self.getLocation(p, 1)])

        static = IDLInterfaceMember.Special.Static in p[1]
        # If static is there that's all that's allowed.  This is disallowed
        # by the parser, so we can assert here.
        assert not static or len(qualifiers) == 1

        stringifier = IDLInterfaceMember.Special.Stringifier in p[1]
        # If stringifier is there that's all that's allowed.  This is disallowed
        # by the parser, so we can assert here.
        assert not stringifier or len(qualifiers) == 1

        getter = True if IDLMethod.Special.Getter in p[1] else False
        setter = True if IDLMethod.Special.Setter in p[1] else False
        creator = True if IDLMethod.Special.Creator in p[1] else False
        deleter = True if IDLMethod.Special.Deleter in p[1] else False
        legacycaller = True if IDLMethod.Special.LegacyCaller in p[1] else False

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
            if not returnType.isDOMString():
                raise WebIDLError("stringifier must have DOMString return type",
                                  [self.getLocation(p, 2)])

        # identifier might be None.  This is only permitted for special methods.
        if not identifier:
            if (not getter and not setter and not creator and
                not deleter and not legacycaller and not stringifier):
                raise WebIDLError("Identifier required for non-special methods",
                                  [self.getLocation(p, 2)])

            location = BuiltinLocation("<auto-generated-identifier>")
            identifier = IDLUnresolvedIdentifier(
                location,
                "__%s%s%s%s%s%s%s" %
                ("named" if specialType == IDLMethod.NamedOrIndexed.Named else
                 "indexed" if specialType == IDLMethod.NamedOrIndexed.Indexed else "",
                 "getter" if getter else "",
                 "setter" if setter else "",
                 "deleter" if deleter else "",
                 "creator" if creator else "",
                 "legacycaller" if legacycaller else "",
                 "stringifier" if stringifier else ""),
                allowDoubleUnderscore=True)

        method = IDLMethod(self.getLocation(p, 2), identifier, returnType, arguments,
                           static=static, getter=getter, setter=setter, creator=creator,
                           deleter=deleter, specialType=specialType,
                           legacycaller=legacycaller, stringifier=stringifier)
        p[0] = method

    def p_Stringifier(self, p):
        """
            Operation : STRINGIFIER SEMICOLON
        """
        identifier = IDLUnresolvedIdentifier(BuiltinLocation("<auto-generated-identifier>"),
                                             "__stringifier",
                                             allowDoubleUnderscore=True)
        method = IDLMethod(self.getLocation(p, 1),
                           identifier,
                           returnType=BuiltinTypes[IDLBuiltinType.Types.domstring],
                           arguments=[],
                           stringifier=True)
        p[0] = method

    def p_Jsonifier(self, p):
        """
            Operation : JSONIFIER SEMICOLON
        """
        identifier = IDLUnresolvedIdentifier(BuiltinLocation("<auto-generated-identifier>"),
                                             "__jsonifier", allowDoubleUnderscore=True)
        method = IDLMethod(self.getLocation(p, 1),
                           identifier,
                           returnType=BuiltinTypes[IDLBuiltinType.Types.object],
                           arguments=[],
                           jsonifier=True)
        p[0] = method

    def p_QualifierStatic(self, p):
        """
            Qualifier : STATIC
        """
        p[0] = [IDLInterfaceMember.Special.Static]

    def p_QualifierStringifier(self, p):
        """
            Qualifier : STRINGIFIER
        """
        p[0] = [IDLInterfaceMember.Special.Stringifier]

    def p_Qualifiers(self, p):
        """
            Qualifiers : Qualifier
                       | Specials
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
            Argument : ExtendedAttributeList Optional Type Ellipsis ArgumentName Default
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

        # We can't test t.isAny() here and give it a default value as needed,
        # since at this point t is not a fully resolved type yet (e.g. it might
        # be a typedef).  We'll handle the 'any' case in IDLArgument.complete.

        if variadic:
            if optional:
                raise WebIDLError("Variadic arguments should not be marked optional.",
                                  [self.getLocation(p, 2)])
            optional = variadic

        p[0] = IDLArgument(self.getLocation(p, 5), identifier, t, optional, defaultValue, variadic)
        p[0].addExtendedAttributes(p[1])

    def p_ArgumentName(self, p):
        """
            ArgumentName : IDENTIFIER
                         | ATTRIBUTE
                         | CALLBACK
                         | CONST
                         | CREATOR
                         | DELETER
                         | DICTIONARY
                         | ENUM
                         | EXCEPTION
                         | GETTER
                         | IMPLEMENTS
                         | INHERIT
                         | INTERFACE
                         | LEGACYCALLER
                         | MAPLIKE
                         | PARTIAL
                         | REQUIRED
                         | SERIALIZER
                         | SETLIKE
                         | SETTER
                         | STATIC
                         | STRINGIFIER
                         | JSONIFIER
                         | TYPEDEF
                         | UNRESTRICTED
        """
        p[0] = p[1]

    def p_AttributeName(self, p):
        """
            AttributeName : IDENTIFIER
                          | REQUIRED
        """
        p[0] = p[1]

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

    def p_Required(self, p):
        """
            Required : REQUIRED
        """
        p[0] = True

    def p_RequiredEmpty(self, p):
        """
            Required :
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
                              | ExtendedAttributeIdentList
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
                  | BYTESTRING
                  | USVSTRING
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
                  | MOZMAP
                  | SETTER
                  | SHORT
                  | STATIC
                  | STRINGIFIER
                  | JSONIFIER
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
                       | SHAREDARRAYBUFFER TypeSuffix
                       | OBJECT TypeSuffix
        """
        if p[1] == "object":
            type = BuiltinTypes[IDLBuiltinType.Types.object]
        elif p[1] == "ArrayBuffer":
            type = BuiltinTypes[IDLBuiltinType.Types.ArrayBuffer]
        elif p[1] == "SharedArrayBuffer":
            type = BuiltinTypes[IDLBuiltinType.Types.SharedArrayBuffer]
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

    # Note: Promise<void> is allowed, so we want to parametrize on
    # ReturnType, not Type.  Also, we want this to end up picking up
    # the Promise interface for now, hence the games with IDLUnresolvedType.
    def p_NonAnyTypePromiseType(self, p):
        """
            NonAnyType : PROMISE LT ReturnType GT Null
        """
        innerType = p[3]
        promiseIdent = IDLUnresolvedIdentifier(self.getLocation(p, 1), "Promise")
        type = IDLUnresolvedType(self.getLocation(p, 1), promiseIdent, p[3])
        if p[5]:
            type = IDLNullableType(self.getLocation(p, 5), type)
        p[0] = type

    def p_NonAnyTypeMozMapType(self, p):
        """
            NonAnyType : MOZMAP LT Type GT Null
        """
        innerType = p[3]
        type = IDLMozMapType(self.getLocation(p, 1), innerType)
        if p[5]:
            type = IDLNullableType(self.getLocation(p, 5), type)
        p[0] = type

    def p_NonAnyTypeScopedName(self, p):
        """
            NonAnyType : ScopedName TypeSuffix
        """
        assert isinstance(p[1], IDLUnresolvedIdentifier)

        if p[1].name == "Promise":
            raise WebIDLError("Promise used without saying what it's "
                              "parametrized over",
                              [self.getLocation(p, 1)])

        type = None

        try:
            if self.globalScope()._lookupIdentifier(p[1]):
                obj = self.globalScope()._lookupIdentifier(p[1])
                assert not obj.isType()
                if obj.isTypedef():
                    type = IDLTypedefType(self.getLocation(p, 1), obj.innerType,
                                          obj.identifier.name)
                elif obj.isCallback() and not obj.isInterface():
                    type = IDLCallbackType(self.getLocation(p, 1), obj)
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
        p[0] = self.handleModifiers(BuiltinTypes[IDLBuiltinType.Types.date],
                                    p[2])

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

    def p_PrimitiveOrStringTypeUnrestictedFloat(self, p):
        """
            PrimitiveOrStringType : UNRESTRICTED FLOAT
        """
        p[0] = IDLBuiltinType.Types.unrestricted_float

    def p_PrimitiveOrStringTypeDouble(self, p):
        """
            PrimitiveOrStringType : DOUBLE
        """
        p[0] = IDLBuiltinType.Types.double

    def p_PrimitiveOrStringTypeUnrestictedDouble(self, p):
        """
            PrimitiveOrStringType : UNRESTRICTED DOUBLE
        """
        p[0] = IDLBuiltinType.Types.unrestricted_double

    def p_PrimitiveOrStringTypeDOMString(self, p):
        """
            PrimitiveOrStringType : DOMSTRING
        """
        p[0] = IDLBuiltinType.Types.domstring

    def p_PrimitiveOrStringTypeBytestring(self, p):
        """
            PrimitiveOrStringType : BYTESTRING
        """
        p[0] = IDLBuiltinType.Types.bytestring

    def p_PrimitiveOrStringTypeUSVString(self, p):
        """
            PrimitiveOrStringType : USVSTRING
        """
        p[0] = IDLBuiltinType.Types.usvstring

    def p_UnsignedIntegerTypeUnsigned(self, p):
        """
            UnsignedIntegerType : UNSIGNED IntegerType
        """
        # Adding one to a given signed integer type gets you the unsigned type:
        p[0] = p[2] + 1

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
        assert not p[2]  # Not implemented!

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

    def p_ExtendedAttributeIdentList(self, p):
        """
            ExtendedAttributeIdentList : IDENTIFIER EQUALS LPAREN IdentifierList RPAREN
        """
        p[0] = (p[1], p[4])

    def p_IdentifierList(self, p):
        """
            IdentifierList : IDENTIFIER Identifiers
        """
        idents = list(p[2])
        idents.insert(0, p[1])
        p[0] = idents

    def p_IdentifiersList(self, p):
        """
            Identifiers : COMMA IDENTIFIER Identifiers
        """
        idents = list(p[3])
        idents.insert(0, p[2])
        p[0] = idents

    def p_IdentifiersEmpty(self, p):
        """
            Identifiers :
        """
        p[0] = []

    def p_error(self, p):
        if not p:
            raise WebIDLError("Syntax Error at end of file. Possibly due to missing semicolon(;), braces(}) or both",
                              [self._filename])
        else:
            raise WebIDLError("invalid syntax", [Location(self.lexer, p.lineno, p.lexpos, self._filename)])

    def __init__(self, outputdir='', lexer=None):
        Tokenizer.__init__(self, outputdir, lexer)

        logger = SqueakyCleanLogger()
        self.parser = yacc.yacc(module=self,
                                outputdir=outputdir,
                                tabmodule='webidlyacc',
                                errorlog=logger
                                # Pickling the grammar is a speedup in
                                # some cases (older Python?) but a
                                # significant slowdown in others.
                                # We're not pickling for now, until it
                                # becomes a speedup again.
                                # , picklefile='WebIDLGrammar.pkl'
                                )
        logger.reportGrammarErrors()

        self._globalScope = IDLScope(BuiltinLocation("<Global Scope>"), None, None)
        # To make our test harness work, pretend like we have a primary global already.
        # Note that we _don't_ set _globalScope.primaryGlobalAttr,
        # so we'll still be able to detect multiple PrimaryGlobal extended attributes.
        self._globalScope.primaryGlobalName = "FakeTestPrimaryGlobal"
        self._globalScope.globalNames.add("FakeTestPrimaryGlobal")
        self._globalScope.globalNameMapping["FakeTestPrimaryGlobal"].add("FakeTestPrimaryGlobal")
        # And we add the special-cased "System" global name, which
        # doesn't have any corresponding interfaces.
        self._globalScope.globalNames.add("System")
        self._globalScope.globalNameMapping["System"].add("BackstagePass")
        self._installBuiltins(self._globalScope)
        self._productions = []

        self._filename = "<builtin>"
        self.lexer.input(Parser._builtins)
        self._filename = None

        self.parser.parse(lexer=self.lexer, tracking=True)

    def _installBuiltins(self, scope):
        assert isinstance(scope, IDLScope)

        # xrange omits the last value.
        for x in xrange(IDLBuiltinType.Types.ArrayBuffer, IDLBuiltinType.Types.SharedFloat64Array + 1):
            builtin = BuiltinTypes[x]
            name = builtin.name
            typedef = IDLTypedef(BuiltinLocation("<builtin type>"), scope, builtin, name)

    @ staticmethod
    def handleModifiers(type, modifiers):
        for (modifier, modifierLocation) in modifiers:
            assert (modifier == IDLMethod.TypeSuffixModifier.QMark or
                    modifier == IDLMethod.TypeSuffixModifier.Brackets)

            if modifier == IDLMethod.TypeSuffixModifier.QMark:
                type = IDLNullableType(modifierLocation, type)
            elif modifier == IDLMethod.TypeSuffixModifier.Brackets:
                type = IDLArrayType(modifierLocation, type)

        return type

    def parse(self, t, filename=None):
        self.lexer.input(t)

        # for tok in iter(self.lexer.token, None):
        #    print tok

        self._filename = filename
        self._productions.extend(self.parser.parse(lexer=self.lexer, tracking=True))
        self._filename = None

    def finish(self):
        # First, finish all the IDLImplementsStatements.  In particular, we
        # have to make sure we do those before we do the IDLInterfaces.
        # XXX khuey hates this bit and wants to nuke it from orbit.
        implementsStatements = [p for p in self._productions if
                                isinstance(p, IDLImplementsStatement)]
        otherStatements = [p for p in self._productions if
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
        typedef (ArrayBufferView or ArrayBuffer) BufferSource;
        typedef (SharedArrayBufferView or SharedArrayBuffer) SharedBufferSource;
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
