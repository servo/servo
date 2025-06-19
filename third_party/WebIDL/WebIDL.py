# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

""" A WebIDL parser. """

import copy
import math
import os
import re
import string
import traceback
from collections import OrderedDict, defaultdict
from itertools import chain

from ply import lex, yacc

# Machinery


def parseInt(literal):
    string = literal
    sign = 0
    base = 0

    if string[0] == "-":
        sign = -1
        string = string[1:]
    else:
        sign = 1

    if string[0] == "0" and len(string) > 1:
        if string[1] == "x" or string[1] == "X":
            base = 16
            string = string[2:]
        else:
            base = 8
            string = string[1:]
    else:
        base = 10

    value = int(string, base)
    return value * sign


# This is surprisingly faster than using the enum.IntEnum type (which doesn't
# support 'base' anyway)
def enum(*names, base=None):
    if base is not None:
        names = base.attrs + names

    class CustomEnumType(object):
        attrs = names

        def __setattr__(self, name, value):  # this makes it read-only
            raise NotImplementedError

    for v, k in enumerate(names):
        setattr(CustomEnumType, k, v)

    return CustomEnumType()


class WebIDLError(Exception):
    def __init__(self, message, locations, warning=False):
        self.message = message
        self.locations = [str(loc) for loc in locations]
        self.warning = warning

    def __str__(self):
        return "%s: %s%s%s" % (
            self.warning and "warning" or "error",
            self.message,
            ", " if len(self.locations) != 0 else "",
            "\n".join(self.locations),
        )


class Location(object):
    def __init__(self, lexer, lineno, lexpos, filename):
        self._line = None
        self._lineno = lineno
        self._lexpos = lexpos
        self._lexdata = lexer.lexdata
        self.filename = filename if filename else "<unknown>"

    def __eq__(self, other):
        return self._lexpos == other._lexpos and self.filename == other.filename

    def resolve(self):
        if self._line:
            return

        startofline = self._lexdata.rfind("\n", 0, self._lexpos) + 1
        endofline = self._lexdata.find("\n", self._lexpos, self._lexpos + 80)
        if endofline != -1:
            self._line = self._lexdata[startofline:endofline]
        else:
            self._line = self._lexdata[startofline:]
        self._colno = self._lexpos - startofline

        # Our line number seems to point to the start of self._lexdata
        self._lineno += self._lexdata.count("\n", 0, startofline)

    def get(self):
        self.resolve()
        return "%s line %s:%s" % (self.filename, self._lineno, self._colno)

    def _pointerline(self):
        return " " * self._colno + "^"

    def __str__(self):
        self.resolve()
        return "%s line %s:%s\n%s\n%s" % (
            self.filename,
            self._lineno,
            self._colno,
            self._line,
            self._pointerline(),
        )


class BuiltinLocation(object):
    __slots__ = "msg", "filename"

    def __init__(self, text):
        self.msg = text + "\n"
        self.filename = "<builtin>"

    def __eq__(self, other):
        return isinstance(other, BuiltinLocation) and self.msg == other.msg

    def resolve(self):
        pass

    def get(self):
        return self.msg

    def __str__(self):
        return self.get()


# Data Model


class IDLObject(object):
    __slots__ = "location", "userData", "filename"

    def __init__(self, location):
        self.location = location
        self.userData = {}
        self.filename = location and location.filename

    def isInterface(self):
        return False

    def isNamespace(self):
        return False

    def isInterfaceMixin(self):
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
        """Return a set of files that this object depends on.  If any of
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
        if self.filename != "<builtin>":
            deps.add(self.filename)

        for d in self._getDependentObjects():
            deps.update(d.getDeps(visited))

        return deps


class IDLScope(IDLObject):
    __slots__ = "parentScope", "_name", "_dict", "globalNames", "globalNameMapping"

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

    def __str__(self):
        return self.QName()

    def QName(self):
        # It's possible for us to be called before __init__ has been called, for
        # the IDLObjectWithScope case.  In that case, self._name won't be set yet.
        if hasattr(self, "_name"):
            name = self._name
        else:
            name = None
        if name:
            return name.QName() + "::"
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

            replacement = self.resolveIdentifierConflict(
                self, identifier, self._dict[identifier.name], object
            )
            self._dict[identifier.name] = replacement
            return

        self.addNewIdentifier(identifier, object)

    def addNewIdentifier(self, identifier, object):
        assert object

        self._dict[identifier.name] = object

    def resolveIdentifierConflict(self, scope, identifier, originalObject, newObject):
        if (
            isinstance(originalObject, IDLExternalInterface)
            and isinstance(newObject, IDLExternalInterface)
            and originalObject.identifier.name == newObject.identifier.name
        ):
            return originalObject

        if isinstance(originalObject, IDLExternalInterface) or isinstance(
            newObject, IDLExternalInterface
        ):
            raise WebIDLError(
                "Name collision between "
                "interface declarations for identifier '%s' at '%s' and '%s'"
                % (identifier.name, originalObject.location, newObject.location),
                [],
            )

        if isinstance(originalObject, IDLDictionary) or isinstance(
            newObject, IDLDictionary
        ):
            raise WebIDLError(
                "Name collision between dictionary declarations for "
                "identifier '%s'.\n%s\n%s"
                % (identifier.name, originalObject.location, newObject.location),
                [],
            )

        # We do the merging of overloads here as opposed to in IDLInterface
        # because we need to merge overloads of LegacyFactoryFunctions and we need to
        # detect conflicts in those across interfaces. See also the comment in
        # IDLInterface.addExtendedAttributes for "LegacyFactoryFunction".
        if isinstance(originalObject, IDLMethod) and isinstance(newObject, IDLMethod):
            return originalObject.addOverload(newObject)

        # Default to throwing, derived classes can override.
        raise self.createIdentifierConflictError(identifier, originalObject, newObject)

    def createIdentifierConflictError(self, identifier, originalObject, newObject):
        conflictdesc = "\n\t%s at %s\n\t%s at %s" % (
            originalObject,
            originalObject.location,
            newObject,
            newObject.location,
        )

        return WebIDLError(
            "Multiple unresolvable definitions of identifier '%s' in scope '%s'%s"
            % (identifier.name, str(self), conflictdesc),
            [],
        )

    def _lookupIdentifier(self, identifier):
        return self._dict[identifier.name]

    def lookupIdentifier(self, identifier):
        assert isinstance(identifier, IDLIdentifier)
        assert identifier.scope == self
        return self._lookupIdentifier(identifier)

    def addIfaceGlobalNames(self, interfaceName, globalNames):
        """Record the global names (from |globalNames|) that can be used in
        [Exposed] to expose things in a global named |interfaceName|"""
        self.globalNames.update(globalNames)
        for name in globalNames:
            self.globalNameMapping[name].add(interfaceName)


class IDLIdentifier(IDLObject):
    __slots__ = "name", "scope"

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
    __slots__ = ("name",)

    def __init__(
        self, location, name, allowDoubleUnderscore=False, allowForbidden=False
    ):
        IDLObject.__init__(self, location)

        assert name

        if name == "__noSuchMethod__":
            raise WebIDLError("__noSuchMethod__ is deprecated", [location])

        if name[:2] == "__" and not allowDoubleUnderscore:
            raise WebIDLError("Identifiers beginning with __ are reserved", [location])
        if name[0] == "_" and not allowDoubleUnderscore:
            name = name[1:]
        if name in ["constructor", "toString"] and not allowForbidden:
            raise WebIDLError(
                "Cannot use reserved identifier '%s'" % (name), [location]
            )

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
    # no slots, incompatible with multiple inheritance
    def __init__(self, location, parentScope, identifier):
        IDLObject.__init__(self, location)

        assert isinstance(identifier, IDLUnresolvedIdentifier)

        self.identifier = identifier

        if parentScope:
            self.resolve(parentScope)

    def resolve(self, parentScope):
        assert isinstance(parentScope, IDLScope)
        assert isinstance(self.identifier, IDLUnresolvedIdentifier)
        self.identifier.resolve(parentScope, self)


class IDLObjectWithScope(IDLObjectWithIdentifier, IDLScope):
    __slots__ = ()

    def __init__(self, location, parentScope, identifier):
        assert isinstance(identifier, IDLUnresolvedIdentifier)

        IDLObjectWithIdentifier.__init__(self, location, parentScope, identifier)
        IDLScope.__init__(self, location, parentScope, self.identifier)


class IDLIdentifierPlaceholder(IDLObjectWithIdentifier):
    __slots__ = ()

    def __init__(self, location, identifier):
        assert isinstance(identifier, IDLUnresolvedIdentifier)
        IDLObjectWithIdentifier.__init__(self, location, None, identifier)

    def finish(self, scope):
        try:
            scope._lookupIdentifier(self.identifier)
        except Exception:
            raise WebIDLError(
                "Unresolved type '%s'." % self.identifier, [self.location]
            )

        obj = self.identifier.resolve(scope, None)
        return scope.lookupIdentifier(obj)


class IDLExposureMixins:
    # no slots, incompatible with multiple inheritance
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

        if "*" in self._exposureGlobalNames:
            self._exposureGlobalNames = scope.globalNames
        else:
            # Verify that our [Exposed] value, if any, makes sense.
            for globalName in self._exposureGlobalNames:
                if globalName not in scope.globalNames:
                    raise WebIDLError(
                        "Unknown [Exposed] value %s" % globalName, [self._location]
                    )

        # Verify that we are exposed _somwhere_ if we have some place to be
        # exposed.  We don't want to assert that we're definitely exposed
        # because a lot of our parser tests have small-enough IDL snippets that
        # they don't include any globals, and we don't really want to go through
        # and add global interfaces and [Exposed] annotations to all those
        # tests.
        if len(scope.globalNames) != 0 and len(self._exposureGlobalNames) == 0:
            raise WebIDLError(
                (
                    "'%s' is not exposed anywhere even though we have "
                    "globals to be exposed to"
                )
                % self,
                [self.location],
            )

        globalNameSetToExposureSet(scope, self._exposureGlobalNames, self.exposureSet)

    def isExposedInWindow(self):
        return "Window" in self.exposureSet

    def isExposedInAnyWorker(self):
        return len(self.getWorkerExposureSet()) > 0

    def isExposedInWorkerDebugger(self):
        return len(self.getWorkerDebuggerExposureSet()) > 0

    def isExposedInAnyWorklet(self):
        return len(self.getWorkletExposureSet()) > 0

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

    def isExposedInShadowRealms(self):
        return "ShadowRealmGlobalScope" in self.exposureSet

    def getWorkerExposureSet(self):
        workerScopes = self._globalScope.globalNameMapping["Worker"]
        return workerScopes.intersection(self.exposureSet)

    def getWorkletExposureSet(self):
        workletScopes = self._globalScope.globalNameMapping["Worklet"]
        return workletScopes.intersection(self.exposureSet)

    def getWorkerDebuggerExposureSet(self):
        workerDebuggerScopes = self._globalScope.globalNameMapping["WorkerDebugger"]
        return workerDebuggerScopes.intersection(self.exposureSet)


class IDLExternalInterface(IDLObjectWithIdentifier):
    __slots__ = ("parent",)

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

    def isIteratorInterface(self):
        return False

    def isAsyncIteratorInterface(self):
        return False

    def isExternal(self):
        return True

    def isInterface(self):
        return True

    def addExtendedAttributes(self, attrs):
        if len(attrs) != 0:
            raise WebIDLError(
                "There are no extended attributes that are "
                "allowed on external interfaces",
                [attrs[0].location, self.location],
            )

    def resolve(self, parentScope):
        pass

    def getJSImplementation(self):
        return None

    def isJSImplemented(self):
        return False

    def hasProbablyShortLivingWrapper(self):
        return False

    def _getDependentObjects(self):
        return set()


class IDLPartialDictionary(IDLObject):
    __slots__ = "identifier", "members", "_nonPartialDictionary", "_finished"

    def __init__(self, location, name, members, nonPartialDictionary):
        assert isinstance(name, IDLUnresolvedIdentifier)

        IDLObject.__init__(self, location)
        self.identifier = name
        self.members = members
        self._nonPartialDictionary = nonPartialDictionary
        self._finished = False
        nonPartialDictionary.addPartialDictionary(self)

    def addExtendedAttributes(self, attrs):
        pass

    def finish(self, scope):
        if self._finished:
            return
        self._finished = True

        # Need to make sure our non-partial dictionary gets
        # finished so it can report cases when we only have partial
        # dictionaries.
        self._nonPartialDictionary.finish(scope)

    def validate(self):
        pass


class IDLPartialInterfaceOrNamespace(IDLObject):
    __slots__ = (
        "identifier",
        "members",
        "propagatedExtendedAttrs",
        "_haveSecureContextExtendedAttribute",
        "_nonPartialInterfaceOrNamespace",
        "_finished",
    )

    def __init__(self, location, name, members, nonPartialInterfaceOrNamespace):
        assert isinstance(name, IDLUnresolvedIdentifier)

        IDLObject.__init__(self, location)
        self.identifier = name
        self.members = members
        # propagatedExtendedAttrs are the ones that should get
        # propagated to our non-partial interface.
        self.propagatedExtendedAttrs = []
        self._haveSecureContextExtendedAttribute = False
        self._nonPartialInterfaceOrNamespace = nonPartialInterfaceOrNamespace
        self._finished = False
        nonPartialInterfaceOrNamespace.addPartial(self)

    def addExtendedAttributes(self, attrs):
        for attr in attrs:
            identifier = attr.identifier()

            if identifier == "LegacyFactoryFunction":
                self.propagatedExtendedAttrs.append(attr)
            elif identifier == "SecureContext":
                self._haveSecureContextExtendedAttribute = True
                # This gets propagated to all our members.
                for member in self.members:
                    if member.getExtendedAttribute("SecureContext"):
                        raise WebIDLError(
                            "[SecureContext] specified on both a "
                            "partial interface member and on the "
                            "partial interface itself",
                            [member.location, attr.location],
                        )
                    member.addExtendedAttributes([attr])
            elif identifier == "Exposed":
                # This just gets propagated to all our members.
                for member in self.members:
                    if len(member._exposureGlobalNames) != 0:
                        raise WebIDLError(
                            "[Exposed] specified on both a "
                            "partial interface member and on the "
                            "partial interface itself",
                            [member.location, attr.location],
                        )
                    member.addExtendedAttributes([attr])
            else:
                raise WebIDLError(
                    "Unknown extended attribute %s on partial "
                    "interface" % identifier,
                    [attr.location],
                )

    def finish(self, scope):
        if self._finished:
            return
        self._finished = True
        if (
            not self._haveSecureContextExtendedAttribute
            and self._nonPartialInterfaceOrNamespace.getExtendedAttribute(
                "SecureContext"
            )
        ):
            # This gets propagated to all our members.
            for member in self.members:
                if member.getExtendedAttribute("SecureContext"):
                    raise WebIDLError(
                        "[SecureContext] specified on both a "
                        "partial interface member and on the "
                        "non-partial interface",
                        [
                            member.location,
                            self._nonPartialInterfaceOrNamespace.location,
                        ],
                    )
                member.addExtendedAttributes(
                    [
                        IDLExtendedAttribute(
                            self._nonPartialInterfaceOrNamespace.location,
                            ("SecureContext",),
                        )
                    ]
                )
        # Need to make sure our non-partial interface or namespace gets
        # finished so it can report cases when we only have partial
        # interfaces/namespaces.
        self._nonPartialInterfaceOrNamespace.finish(scope)

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


# Because WebIDL allows static and regular operations with the same identifier
# we use a special class to be able to store them both in the scope for the
# same identifier.
class IDLOperations:
    __slots__ = "static", "regular"

    def __init__(self, static=None, regular=None):
        self.static = static
        self.regular = regular


class IDLInterfaceOrInterfaceMixinOrNamespace(IDLObjectWithScope, IDLExposureMixins):
    __slots__ = (
        "_finished",
        "members",
        "_partials",
        "_extendedAttrDict",
        "_isKnownNonPartial",
    )

    def __init__(self, location, parentScope, name):
        assert isinstance(parentScope, IDLScope)
        assert isinstance(name, IDLUnresolvedIdentifier)

        self._finished = False
        self.members = []
        self._partials = []
        self._extendedAttrDict = {}
        self._isKnownNonPartial = False

        IDLObjectWithScope.__init__(self, location, parentScope, name)
        IDLExposureMixins.__init__(self, location)

    def finish(self, scope):
        if not self._isKnownNonPartial:
            raise WebIDLError(
                "%s does not have a non-partial declaration" % str(self),
                [self.location],
            )

        IDLExposureMixins.finish(self, scope)

        # Now go ahead and merge in our partials.
        for partial in self._partials:
            partial.finish(scope)
            self.addExtendedAttributes(partial.propagatedExtendedAttrs)
            self.members.extend(partial.members)

    def addNewIdentifier(self, identifier, object):
        if isinstance(object, IDLMethod):
            if object.isStatic():
                object = IDLOperations(static=object)
            else:
                object = IDLOperations(regular=object)

        IDLScope.addNewIdentifier(self, identifier, object)

    def resolveIdentifierConflict(self, scope, identifier, originalObject, newObject):
        assert isinstance(scope, IDLScope)
        assert isinstance(newObject, IDLInterfaceMember)

        # The identifier of a regular operation or static operation must not be
        # the same as the identifier of a constant or attribute.
        if isinstance(newObject, IDLMethod) != isinstance(
            originalObject, IDLOperations
        ):
            if isinstance(originalObject, IDLOperations):
                if originalObject.regular is not None:
                    originalObject = originalObject.regular
                else:
                    assert originalObject.static is not None
                    originalObject = originalObject.static

            raise self.createIdentifierConflictError(
                identifier, originalObject, newObject
            )

        if isinstance(newObject, IDLMethod):
            originalOperations = originalObject
            if newObject.isStatic():
                if originalOperations.static is None:
                    originalOperations.static = newObject
                    return originalOperations

                originalObject = originalOperations.static
            else:
                if originalOperations.regular is None:
                    originalOperations.regular = newObject
                    return originalOperations

                originalObject = originalOperations.regular

            assert isinstance(originalObject, IDLMethod)
        else:
            assert isinstance(originalObject, IDLInterfaceMember)

        retval = IDLScope.resolveIdentifierConflict(
            self, scope, identifier, originalObject, newObject
        )

        if isinstance(newObject, IDLMethod):
            if newObject.isStatic():
                originalOperations.static = retval
            else:
                originalOperations.regular = retval

            retval = originalOperations

        # Might be a ctor, which isn't in self.members
        if newObject in self.members:
            self.members.remove(newObject)
        return retval

    def typeName(self):
        if self.isInterface():
            return "interface"
        if self.isNamespace():
            return "namespace"
        assert self.isInterfaceMixin()
        return "interface mixin"

    def getExtendedAttribute(self, name):
        return self._extendedAttrDict.get(name, None)

    def setNonPartial(self, location, members):
        if self._isKnownNonPartial:
            raise WebIDLError(
                "Two non-partial definitions for the " "same %s" % self.typeName(),
                [location, self.location],
            )
        self._isKnownNonPartial = True
        # Now make it look like we were parsed at this new location, since
        # that's the place where the interface is "really" defined
        self.location = location
        # Put the new members at the beginning
        self.members = members + self.members

    def addPartial(self, partial):
        assert self.identifier.name == partial.identifier.name
        self._partials.append(partial)

    def getPartials(self):
        # Don't let people mutate our guts.
        return list(self._partials)

    def finishMembers(self, scope):
        # Assuming we've merged in our partials, set the _exposureGlobalNames on
        # any members that don't have it set yet.  Note that any partial
        # interfaces that had [Exposed] set have already set up
        # _exposureGlobalNames on all the members coming from them, so this is
        # just implementing the "members default to interface or interface mixin
        # that defined them" and "partial interfaces or interface mixins default
        # to interface or interface mixin they're a partial for" rules from the
        # spec.
        for m in self.members:
            # If m, or the partial m came from, had [Exposed]
            # specified, it already has a nonempty exposure global names set.
            if len(m._exposureGlobalNames) == 0:
                m._exposureGlobalNames.update(self._exposureGlobalNames)
            if m.isAttr() and m.stringifier:
                m.expand(self.members)

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
                raise WebIDLError(
                    "Interface or interface mixin member has "
                    "larger exposure set than its container",
                    [member.location, self.location],
                )

    def isExternal(self):
        return False


class IDLInterfaceMixin(IDLInterfaceOrInterfaceMixinOrNamespace):
    __slots__ = ("actualExposureGlobalNames",)

    def __init__(self, location, parentScope, name, members, isKnownNonPartial):
        self.actualExposureGlobalNames = set()

        assert isKnownNonPartial or not members
        IDLInterfaceOrInterfaceMixinOrNamespace.__init__(
            self, location, parentScope, name
        )

        if isKnownNonPartial:
            self.setNonPartial(location, members)

    def __str__(self):
        return "Interface mixin '%s'" % self.identifier.name

    def isInterfaceMixin(self):
        return True

    def finish(self, scope):
        if self._finished:
            return
        self._finished = True

        # Expose to the globals of interfaces that includes this mixin if this
        # mixin has no explicit [Exposed] so that its members can be exposed
        # based on the base interface exposure set.
        #
        # Make sure this is done before IDLExposureMixins.finish call, since
        # that converts our set of exposure global names to an actual exposure
        # set.
        hasImplicitExposure = len(self._exposureGlobalNames) == 0
        if hasImplicitExposure:
            self._exposureGlobalNames.update(self.actualExposureGlobalNames)

        IDLInterfaceOrInterfaceMixinOrNamespace.finish(self, scope)

        self.finishMembers(scope)

    def validate(self):
        for member in self.members:
            if member.isAttr():
                if member.inherit:
                    raise WebIDLError(
                        "Interface mixin member cannot include "
                        "an inherited attribute",
                        [member.location, self.location],
                    )
                if member.isStatic():
                    raise WebIDLError(
                        "Interface mixin member cannot include " "a static member",
                        [member.location, self.location],
                    )

            if member.isMethod():
                if member.isStatic():
                    raise WebIDLError(
                        "Interface mixin member cannot include " "a static operation",
                        [member.location, self.location],
                    )
                if (
                    member.isGetter()
                    or member.isSetter()
                    or member.isDeleter()
                    or member.isLegacycaller()
                ):
                    raise WebIDLError(
                        "Interface mixin member cannot include a " "special operation",
                        [member.location, self.location],
                    )

    def addExtendedAttributes(self, attrs):
        for attr in attrs:
            identifier = attr.identifier()

            if identifier == "SecureContext":
                if not attr.noArguments():
                    raise WebIDLError(
                        "[%s] must take no arguments" % identifier, [attr.location]
                    )
                # This gets propagated to all our members.
                for member in self.members:
                    if member.getExtendedAttribute("SecureContext"):
                        raise WebIDLError(
                            "[SecureContext] specified on both "
                            "an interface mixin member and on"
                            "the interface mixin itself",
                            [member.location, attr.location],
                        )
                    member.addExtendedAttributes([attr])
            elif identifier == "Exposed":
                convertExposedAttrToGlobalNameSet(attr, self._exposureGlobalNames)
            else:
                raise WebIDLError(
                    "Unknown extended attribute %s on interface" % identifier,
                    [attr.location],
                )

            attrlist = attr.listValue()
            self._extendedAttrDict[identifier] = attrlist if len(attrlist) else True

    def _getDependentObjects(self):
        return set(self.members)


class IDLInterfaceOrNamespace(IDLInterfaceOrInterfaceMixinOrNamespace):
    __slots__ = (
        "parent",
        "_callback",
        "maplikeOrSetlikeOrIterable",
        "legacyFactoryFunctions",
        "legacyWindowAliases",
        "includedMixins",
        "interfacesBasedOnSelf",
        "_hasChildInterfaces",
        "_isOnGlobalProtoChain",
        "totalMembersInSlots",
        "_ownMembersInSlots",
        "iterableInterface",
        "asyncIterableInterface",
        "hasCrossOriginMembers",
        "hasDescendantWithCrossOriginMembers",
    )

    def __init__(self, location, parentScope, name, parent, members, isKnownNonPartial):
        assert isKnownNonPartial or not parent
        assert isKnownNonPartial or not members

        self.parent = None
        self._callback = False
        self.maplikeOrSetlikeOrIterable = None
        # legacyFactoryFunctions needs deterministic ordering because bindings code
        # outputs the constructs in the order that legacyFactoryFunctions enumerates
        # them.
        self.legacyFactoryFunctions = []
        self.legacyWindowAliases = []
        self.includedMixins = set()
        # self.interfacesBasedOnSelf is the set of interfaces that inherit from
        # self, including self itself.
        # Used for distinguishability checking.
        self.interfacesBasedOnSelf = {self}
        self._hasChildInterfaces = False
        self._isOnGlobalProtoChain = False

        # Tracking of the number of reserved slots we need for our
        # members and those of ancestor interfaces.
        self.totalMembersInSlots = 0
        # Tracking of the number of own own members we have in slots
        self._ownMembersInSlots = 0
        # If this is an iterator interface, we need to know what iterable
        # interface we're iterating for in order to get its nativeType.
        self.iterableInterface = None
        self.asyncIterableInterface = None
        # True if we have cross-origin members.
        self.hasCrossOriginMembers = False
        # True if some descendant (including ourselves) has cross-origin members
        self.hasDescendantWithCrossOriginMembers = False

        IDLInterfaceOrInterfaceMixinOrNamespace.__init__(
            self, location, parentScope, name
        )

        if isKnownNonPartial:
            self.setNonPartial(location, parent, members)

    def ctor(self):
        identifier = IDLUnresolvedIdentifier(
            self.location, "constructor", allowForbidden=True
        )
        try:
            return self._lookupIdentifier(identifier).static
        except Exception:
            return None

    def isIterable(self):
        return (
            self.maplikeOrSetlikeOrIterable
            and self.maplikeOrSetlikeOrIterable.isIterable()
        )

    def isAsyncIterable(self):
        return (
            self.maplikeOrSetlikeOrIterable
            and self.maplikeOrSetlikeOrIterable.isAsyncIterable()
        )

    def isIteratorInterface(self):
        return self.iterableInterface is not None

    def isAsyncIteratorInterface(self):
        return self.asyncIterableInterface is not None

    def getClassName(self):
        return self.identifier.name

    def finish(self, scope):
        if self._finished:
            return

        self._finished = True

        IDLInterfaceOrInterfaceMixinOrNamespace.finish(self, scope)

        if len(self.legacyWindowAliases) > 0:
            if not self.hasInterfaceObject():
                raise WebIDLError(
                    "Interface %s unexpectedly has [LegacyWindowAlias] "
                    "and [LegacyNoInterfaceObject] together" % self.identifier.name,
                    [self.location],
                )
            if not self.isExposedInWindow():
                raise WebIDLError(
                    "Interface %s has [LegacyWindowAlias] "
                    "but not exposed in Window" % self.identifier.name,
                    [self.location],
                )

        # Generate maplike/setlike interface members. Since generated members
        # need to be treated like regular interface members, do this before
        # things like exposure setting.
        for member in self.members:
            if member.isMaplikeOrSetlikeOrIterable():
                if self.isJSImplemented():
                    raise WebIDLError(
                        "%s declaration used on "
                        "interface that is implemented in JS"
                        % (member.maplikeOrSetlikeOrIterableType),
                        [member.location],
                    )
                if member.valueType.isObservableArray() or (
                    member.hasKeyType() and member.keyType.isObservableArray()
                ):
                    raise WebIDLError(
                        "%s declaration uses ObservableArray as value or key type"
                        % (member.maplikeOrSetlikeOrIterableType),
                        [member.location],
                    )
                # Check that we only have one interface declaration (currently
                # there can only be one maplike/setlike declaration per
                # interface)
                if self.maplikeOrSetlikeOrIterable:
                    raise WebIDLError(
                        "%s declaration used on "
                        "interface that already has %s "
                        "declaration"
                        % (
                            member.maplikeOrSetlikeOrIterableType,
                            self.maplikeOrSetlikeOrIterable.maplikeOrSetlikeOrIterableType,
                        ),
                        [self.maplikeOrSetlikeOrIterable.location, member.location],
                    )
                self.maplikeOrSetlikeOrIterable = member
                # If we've got a maplike or setlike declaration, we'll be building all of
                # our required methods in Codegen. Generate members now.
                self.maplikeOrSetlikeOrIterable.expand(self.members)

        assert not self.parent or isinstance(self.parent, IDLIdentifierPlaceholder)
        parent = self.parent.finish(scope) if self.parent else None
        if parent and isinstance(parent, IDLExternalInterface):
            raise WebIDLError(
                "%s inherits from %s which does not have "
                "a definition" % (self.identifier.name, self.parent.identifier.name),
                [self.location],
            )
        if parent and not isinstance(parent, IDLInterface):
            raise WebIDLError(
                "%s inherits from %s which is not an interface "
                % (self.identifier.name, self.parent.identifier.name),
                [self.location, parent.location],
            )

        self.parent = parent

        assert iter(self.members)

        if self.isNamespace():
            assert not self.parent
            for m in self.members:
                if m.isAttr() or m.isMethod():
                    if m.isStatic():
                        raise WebIDLError(
                            "Don't mark things explicitly static " "in namespaces",
                            [self.location, m.location],
                        )
                    # Just mark all our methods/attributes as static.  The other
                    # option is to duplicate the relevant InterfaceMembers
                    # production bits but modified to produce static stuff to
                    # start with, but that sounds annoying.
                    m.forceStatic()

        if self.parent:
            self.parent.finish(scope)
            self.parent._hasChildInterfaces = True

            self.totalMembersInSlots = self.parent.totalMembersInSlots

            # Interfaces with [Global] must not have anything inherit from them
            if self.parent.getExtendedAttribute("Global"):
                # Note: This is not a self.parent.isOnGlobalProtoChain() check
                # because ancestors of a [Global] interface can have other
                # descendants.
                raise WebIDLError(
                    "[Global] interface has another interface " "inheriting from it",
                    [self.location, self.parent.location],
                )

            # Make sure that we're not exposed in places where our parent is not
            if not self.exposureSet.issubset(self.parent.exposureSet):
                raise WebIDLError(
                    "Interface %s is exposed in globals where its "
                    "parent interface %s is not exposed."
                    % (self.identifier.name, self.parent.identifier.name),
                    [self.location, self.parent.location],
                )

            # Callbacks must not inherit from non-callbacks.
            # XXXbz Can non-callbacks inherit from callbacks?  Spec issue pending.
            if self.isCallback():
                if not self.parent.isCallback():
                    raise WebIDLError(
                        "Callback interface %s inheriting from "
                        "non-callback interface %s"
                        % (self.identifier.name, self.parent.identifier.name),
                        [self.location, self.parent.location],
                    )
            elif self.parent.isCallback():
                raise WebIDLError(
                    "Non-callback interface %s inheriting from "
                    "callback interface %s"
                    % (self.identifier.name, self.parent.identifier.name),
                    [self.location, self.parent.location],
                )

            # Interfaces which have interface objects can't inherit
            # from [LegacyNoInterfaceObject] interfaces.
            if self.parent.getExtendedAttribute(
                "LegacyNoInterfaceObject"
            ) and not self.getExtendedAttribute("LegacyNoInterfaceObject"):
                raise WebIDLError(
                    "Interface %s does not have "
                    "[LegacyNoInterfaceObject] but inherits from "
                    "interface %s which does"
                    % (self.identifier.name, self.parent.identifier.name),
                    [self.location, self.parent.location],
                )

            # Interfaces that are not [SecureContext] can't inherit
            # from [SecureContext] interfaces.
            if self.parent.getExtendedAttribute(
                "SecureContext"
            ) and not self.getExtendedAttribute("SecureContext"):
                raise WebIDLError(
                    "Interface %s does not have "
                    "[SecureContext] but inherits from "
                    "interface %s which does"
                    % (self.identifier.name, self.parent.identifier.name),
                    [self.location, self.parent.location],
                )

        for mixin in self.includedMixins:
            mixin.finish(scope)

        cycleInGraph = self.findInterfaceLoopPoint(self)
        if cycleInGraph:
            raise WebIDLError(
                "Interface %s has itself as ancestor" % self.identifier.name,
                [self.location, cycleInGraph.location],
            )

        self.finishMembers(scope)

        ctor = self.ctor()
        if ctor is not None:
            if not self.hasInterfaceObject():
                raise WebIDLError(
                    "Can't have both a constructor and [LegacyNoInterfaceObject]",
                    [self.location, ctor.location],
                )

            if self.globalNames:
                raise WebIDLError(
                    "Can't have both a constructor and [Global]",
                    [self.location, ctor.location],
                )

            assert ctor._exposureGlobalNames == self._exposureGlobalNames
            ctor._exposureGlobalNames.update(self._exposureGlobalNames)
            # Remove the constructor operation from our member list so
            # it doesn't get in the way later.
            self.members.remove(ctor)

        for ctor in self.legacyFactoryFunctions:
            if self.globalNames:
                raise WebIDLError(
                    "Can't have both a legacy factory function and [Global]",
                    [self.location, ctor.location],
                )
            assert len(ctor._exposureGlobalNames) == 0
            ctor._exposureGlobalNames.update(self._exposureGlobalNames)
            ctor.finish(scope)

        # Make a copy of our member list, so things that implement us
        # can get those without all the stuff we implement ourselves
        # admixed.
        self.originalMembers = list(self.members)

        for mixin in sorted(self.includedMixins, key=lambda x: x.identifier.name):
            for mixinMember in mixin.members:
                for member in self.members:
                    if mixinMember.identifier.name == member.identifier.name and (
                        not mixinMember.isMethod()
                        or not member.isMethod()
                        or mixinMember.isStatic() == member.isStatic()
                    ):
                        raise WebIDLError(
                            "Multiple definitions of %s on %s coming from 'includes' statements"
                            % (member.identifier.name, self),
                            [mixinMember.location, member.location],
                        )
            self.members.extend(mixin.members)

        for ancestor in self.getInheritedInterfaces():
            ancestor.interfacesBasedOnSelf.add(self)
            if (
                ancestor.maplikeOrSetlikeOrIterable is not None
                and self.maplikeOrSetlikeOrIterable is not None
            ):
                raise WebIDLError(
                    "Cannot have maplike/setlike on %s that "
                    "inherits %s, which is already "
                    "maplike/setlike"
                    % (self.identifier.name, ancestor.identifier.name),
                    [
                        self.maplikeOrSetlikeOrIterable.location,
                        ancestor.maplikeOrSetlikeOrIterable.location,
                    ],
                )

        # Deal with interfaces marked [LegacyUnforgeable], now that we have our full
        # member list, except unforgeables pulled in from parents.  We want to
        # do this before we set "originatingInterface" on our unforgeable
        # members.
        if self.getExtendedAttribute("LegacyUnforgeable"):
            # Check that the interface already has all the things the
            # spec would otherwise require us to synthesize and is
            # missing the ones we plan to synthesize.
            if not any(m.isMethod() and m.isStringifier() for m in self.members):
                raise WebIDLError(
                    "LegacyUnforgeable interface %s does not have a "
                    "stringifier" % self.identifier.name,
                    [self.location],
                )

            for m in self.members:
                if m.identifier.name == "toJSON":
                    raise WebIDLError(
                        "LegacyUnforgeable interface %s has a "
                        "toJSON so we won't be able to add "
                        "one ourselves" % self.identifier.name,
                        [self.location, m.location],
                    )

                if m.identifier.name == "valueOf" and not m.isStatic():
                    raise WebIDLError(
                        "LegacyUnforgeable interface %s has a valueOf "
                        "member so we won't be able to add one "
                        "ourselves" % self.identifier.name,
                        [self.location, m.location],
                    )

        for member in self.members:
            if (
                (member.isAttr() or member.isMethod())
                and member.isLegacyUnforgeable()
                and not hasattr(member, "originatingInterface")
            ):
                member.originatingInterface = self

        for member in self.members:
            if (
                member.isMethod() and member.getExtendedAttribute("CrossOriginCallable")
            ) or (
                member.isAttr()
                and (
                    member.getExtendedAttribute("CrossOriginReadable")
                    or member.getExtendedAttribute("CrossOriginWritable")
                )
            ):
                self.hasCrossOriginMembers = True
                break

        if self.hasCrossOriginMembers:
            parent = self
            while parent:
                parent.hasDescendantWithCrossOriginMembers = True
                parent = parent.parent

        # Compute slot indices for our members before we pull in unforgeable
        # members from our parent. Also, maplike/setlike declarations get a
        # slot to hold their backing object.
        for member in self.members:
            if (
                member.isAttr()
                and (
                    member.getExtendedAttribute("StoreInSlot")
                    or member.getExtendedAttribute("Cached")
                    or member.type.isObservableArray()
                )
            ) or member.isMaplikeOrSetlike():
                if self.isJSImplemented() and not member.isMaplikeOrSetlike():
                    raise WebIDLError(
                        "Interface %s is JS-implemented and we "
                        "don't support [Cached] or [StoreInSlot] or ObservableArray "
                        "on JS-implemented interfaces" % self.identifier.name,
                        [self.location, member.location],
                    )
                if member.slotIndices is None:
                    member.slotIndices = dict()
                member.slotIndices[self.identifier.name] = self.totalMembersInSlots
                self.totalMembersInSlots += 1
                if member.getExtendedAttribute("StoreInSlot"):
                    self._ownMembersInSlots += 1

        if self.parent:
            # Make sure we don't shadow any of the [LegacyUnforgeable] attributes on our
            # ancestor interfaces.  We don't have to worry about mixins here, because
            # those have already been imported into the relevant .members lists.  And
            # we don't have to worry about anything other than our parent, because it
            # has already imported its ancestors' unforgeable attributes into its
            # member list.
            for unforgeableMember in (
                member
                for member in self.parent.members
                if (member.isAttr() or member.isMethod())
                and member.isLegacyUnforgeable()
            ):
                shadows = [
                    m
                    for m in self.members
                    if (m.isAttr() or m.isMethod())
                    and not m.isStatic()
                    and m.identifier.name == unforgeableMember.identifier.name
                ]
                if len(shadows) != 0:
                    locs = [unforgeableMember.location] + [s.location for s in shadows]
                    raise WebIDLError(
                        "Interface %s shadows [LegacyUnforgeable] "
                        "members of %s"
                        % (self.identifier.name, ancestor.identifier.name),
                        locs,
                    )
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
        if self.maplikeOrSetlikeOrIterable:
            testInterface = self
            isAncestor = False
            while testInterface:
                self.maplikeOrSetlikeOrIterable.checkCollisions(
                    testInterface.members, isAncestor
                )
                isAncestor = True
                testInterface = testInterface.parent

        # Ensure that there's at most one of each {named,indexed}
        # {getter,setter,deleter}, at most one stringifier,
        # and at most one legacycaller.  Note that this last is not
        # quite per spec, but in practice no one overloads
        # legacycallers.  Also note that in practice we disallow
        # indexed deleters, but it simplifies some other code to
        # treat deleter analogously to getter/setter by
        # prefixing it with "named".
        specialMembersSeen = {}
        for member in self.members:
            if not member.isMethod():
                continue

            if member.isGetter():
                memberType = "getters"
            elif member.isSetter():
                memberType = "setters"
            elif member.isDeleter():
                memberType = "deleters"
            elif member.isStringifier():
                memberType = "stringifiers"
            elif member.isLegacycaller():
                memberType = "legacycallers"
            else:
                continue

            if memberType != "stringifiers" and memberType != "legacycallers":
                if member.isNamed():
                    memberType = "named " + memberType
                else:
                    assert member.isIndexed()
                    memberType = "indexed " + memberType

            if memberType in specialMembersSeen:
                raise WebIDLError(
                    "Multiple " + memberType + " on %s" % (self),
                    [
                        self.location,
                        specialMembersSeen[memberType].location,
                        member.location,
                    ],
                )

            specialMembersSeen[memberType] = member

        if self.getExtendedAttribute("LegacyUnenumerableNamedProperties"):
            # Check that we have a named getter.
            if "named getters" not in specialMembersSeen:
                raise WebIDLError(
                    "Interface with [LegacyUnenumerableNamedProperties] does "
                    "not have a named getter",
                    [self.location],
                )
            ancestor = self.parent
            while ancestor:
                if ancestor.getExtendedAttribute("LegacyUnenumerableNamedProperties"):
                    raise WebIDLError(
                        "Interface with [LegacyUnenumerableNamedProperties] "
                        "inherits from another interface with "
                        "[LegacyUnenumerableNamedProperties]",
                        [self.location, ancestor.location],
                    )
                ancestor = ancestor.parent

        if self._isOnGlobalProtoChain:
            # Make sure we have no named setters or deleters
            for memberType in ["setter", "deleter"]:
                memberId = "named " + memberType + "s"
                if memberId in specialMembersSeen:
                    raise WebIDLError(
                        "Interface with [Global] has a named %s" % memberType,
                        [self.location, specialMembersSeen[memberId].location],
                    )
            # Make sure we're not [LegacyOverrideBuiltIns]
            if self.getExtendedAttribute("LegacyOverrideBuiltIns"):
                raise WebIDLError(
                    "Interface with [Global] also has " "[LegacyOverrideBuiltIns]",
                    [self.location],
                )
            # Mark all of our ancestors as being on the global's proto chain too
            parent = self.parent
            while parent:
                # Must not inherit from an interface with [LegacyOverrideBuiltIns]
                if parent.getExtendedAttribute("LegacyOverrideBuiltIns"):
                    raise WebIDLError(
                        "Interface with [Global] inherits from "
                        "interface with [LegacyOverrideBuiltIns]",
                        [self.location, parent.location],
                    )
                parent._isOnGlobalProtoChain = True
                parent = parent.parent

    def validate(self):
        def checkDuplicateNames(member, name, attributeName):
            for m in self.members:
                if m.identifier.name == name:
                    raise WebIDLError(
                        "[%s=%s] has same name as interface member"
                        % (attributeName, name),
                        [member.location, m.location],
                    )
                if m.isMethod() and m != member and name in m.aliases:
                    raise WebIDLError(
                        "conflicting [%s=%s] definitions" % (attributeName, name),
                        [member.location, m.location],
                    )
                if m.isAttr() and m != member and name in m.bindingAliases:
                    raise WebIDLError(
                        "conflicting [%s=%s] definitions" % (attributeName, name),
                        [member.location, m.location],
                    )

        # We also don't support inheriting from unforgeable interfaces.
        if self.getExtendedAttribute("LegacyUnforgeable") and self.hasChildInterfaces():
            locations = [self.location] + list(
                i.location for i in self.interfacesBasedOnSelf if i.parent == self
            )
            raise WebIDLError(
                "%s is an unforgeable ancestor interface" % self.identifier.name,
                locations,
            )

        ctor = self.ctor()
        if ctor is not None:
            ctor.validate()
        for namedCtor in self.legacyFactoryFunctions:
            namedCtor.validate()

        indexedGetter = None
        hasLengthAttribute = False
        for member in self.members:
            member.validate()

            if self.isCallback() and member.getExtendedAttribute("Replaceable"):
                raise WebIDLError(
                    "[Replaceable] used on an attribute on "
                    "interface %s which is a callback interface" % self.identifier.name,
                    [self.location, member.location],
                )

            # Check that PutForwards refers to another attribute and that no
            # cycles exist in forwarded assignments.  Also check for a
            # integer-typed "length" attribute.
            if member.isAttr():
                if member.identifier.name == "length" and member.type.isInteger():
                    hasLengthAttribute = True

                iface = self
                attr = member
                putForwards = attr.getExtendedAttribute("PutForwards")
                if putForwards and self.isCallback():
                    raise WebIDLError(
                        "[PutForwards] used on an attribute "
                        "on interface %s which is a callback "
                        "interface" % self.identifier.name,
                        [self.location, member.location],
                    )

                while putForwards is not None:
                    forwardIface = attr.type.unroll().inner
                    fowardAttr = None

                    for forwardedMember in forwardIface.members:
                        if (
                            not forwardedMember.isAttr()
                            or forwardedMember.identifier.name != putForwards[0]
                        ):
                            continue
                        if forwardedMember == member:
                            raise WebIDLError(
                                "Cycle detected in forwarded "
                                "assignments for attribute %s on "
                                "%s" % (member.identifier.name, self),
                                [member.location],
                            )
                        fowardAttr = forwardedMember
                        break

                    if fowardAttr is None:
                        raise WebIDLError(
                            "Attribute %s on %s forwards to "
                            "missing attribute %s"
                            % (attr.identifier.name, iface, putForwards),
                            [attr.location],
                        )

                    iface = forwardIface
                    attr = fowardAttr
                    putForwards = attr.getExtendedAttribute("PutForwards")

            # Check that the name of an [Alias] doesn't conflict with an
            # interface member and whether we support indexed properties.
            if member.isMethod():
                if member.isGetter() and member.isIndexed():
                    indexedGetter = member

                for alias in member.aliases:
                    if self.isOnGlobalProtoChain():
                        raise WebIDLError(
                            "[Alias] must not be used on a "
                            "[Global] interface operation",
                            [member.location],
                        )
                    if (
                        member.getExtendedAttribute("Exposed")
                        or member.getExtendedAttribute("ChromeOnly")
                        or member.getExtendedAttribute("Pref")
                        or member.getExtendedAttribute("Func")
                        or member.getExtendedAttribute("Trial")
                        or member.getExtendedAttribute("SecureContext")
                    ):
                        raise WebIDLError(
                            "[Alias] must not be used on a "
                            "conditionally exposed operation",
                            [member.location],
                        )
                    if member.isStatic():
                        raise WebIDLError(
                            "[Alias] must not be used on a " "static operation",
                            [member.location],
                        )
                    if member.isIdentifierLess():
                        raise WebIDLError(
                            "[Alias] must not be used on an "
                            "identifierless operation",
                            [member.location],
                        )
                    if member.isLegacyUnforgeable():
                        raise WebIDLError(
                            "[Alias] must not be used on an "
                            "[LegacyUnforgeable] operation",
                            [member.location],
                        )

                    checkDuplicateNames(member, alias, "Alias")

            # Check that the name of a [BindingAlias] doesn't conflict with an
            # interface member.
            if member.isAttr():
                for bindingAlias in member.bindingAliases:
                    checkDuplicateNames(member, bindingAlias, "BindingAlias")

        # Conditional exposure makes no sense for interfaces with no
        # interface object.
        # And SecureContext makes sense for interfaces with no interface object,
        # since it is also propagated to interface members.
        if (
            self.isExposedConditionally(exclusions=["SecureContext"])
            and not self.hasInterfaceObject()
        ):
            raise WebIDLError(
                "Interface with no interface object is " "exposed conditionally",
                [self.location],
            )

        # Value iterators are only allowed on interfaces with indexed getters,
        # and pair iterators are only allowed on interfaces without indexed
        # getters.
        if self.isIterable():
            iterableDecl = self.maplikeOrSetlikeOrIterable
            if iterableDecl.isValueIterator():
                if not indexedGetter:
                    raise WebIDLError(
                        "Interface with value iterator does not "
                        "support indexed properties",
                        [self.location, iterableDecl.location],
                    )

                if iterableDecl.valueType != indexedGetter.signatures()[0][0]:
                    raise WebIDLError(
                        "Iterable type does not match indexed " "getter type",
                        [iterableDecl.location, indexedGetter.location],
                    )

                if not hasLengthAttribute:
                    raise WebIDLError(
                        "Interface with value iterator does not "
                        'have an integer-typed "length" attribute',
                        [self.location, iterableDecl.location],
                    )
            else:
                assert iterableDecl.isPairIterator()
                if indexedGetter:
                    raise WebIDLError(
                        "Interface with pair iterator supports " "indexed properties",
                        [self.location, iterableDecl.location, indexedGetter.location],
                    )

        if indexedGetter and not hasLengthAttribute:
            raise WebIDLError(
                "Interface with an indexed getter does not have "
                'an integer-typed "length" attribute',
                [self.location, indexedGetter.location],
            )

    def setCallback(self, value):
        self._callback = value

    def isCallback(self):
        return self._callback

    def isSingleOperationInterface(self):
        assert self.isCallback() or self.isJSImplemented()
        return (
            # JS-implemented things should never need the
            # this-handling weirdness of single-operation interfaces.
            not self.isJSImplemented()
            and
            # Not inheriting from another interface
            not self.parent
            and
            # No attributes of any kinds
            not any(m.isAttr() for m in self.members)
            and
            # There is at least one regular operation, and all regular
            # operations have the same identifier
            len(
                set(
                    m.identifier.name
                    for m in self.members
                    if m.isMethod() and not m.isStatic()
                )
            )
            == 1
        )

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
        return (
            not self.isCallback()
            and not self.isNamespace()
            and self.getUserData("hasConcreteDescendant", False)
        )

    def addIncludedMixin(self, includedMixin):
        assert isinstance(includedMixin, IDLInterfaceMixin)
        self.includedMixins.add(includedMixin)

    def getInheritedInterfaces(self):
        """
        Returns a list of the interfaces this interface inherits from
        (not including this interface itself).  The list is in order
        from most derived to least derived.
        """
        assert self._finished
        if not self.parent:
            return []
        parentInterfaces = self.parent.getInheritedInterfaces()
        parentInterfaces.insert(0, self.parent)
        return parentInterfaces

    def findInterfaceLoopPoint(self, otherInterface):
        """
        Finds an interface amongst our ancestors that inherits from otherInterface.
        If there is no such interface, returns None.
        """
        if self.parent:
            if self.parent == otherInterface:
                return self
            loopPoint = self.parent.findInterfaceLoopPoint(otherInterface)
            if loopPoint:
                return loopPoint
        return None

    def setNonPartial(self, location, parent, members):
        assert not parent or isinstance(parent, IDLIdentifierPlaceholder)
        IDLInterfaceOrInterfaceMixinOrNamespace.setNonPartial(self, location, members)
        assert not self.parent
        self.parent = parent

    def getJSImplementation(self):
        classId = self.getExtendedAttribute("JSImplementation")
        if not classId:
            return classId
        assert isinstance(classId, list)
        assert len(classId) == 1
        return classId[0]

    def isJSImplemented(self):
        return bool(self.getJSImplementation())

    def hasProbablyShortLivingWrapper(self):
        current = self
        while current:
            if current.getExtendedAttribute("ProbablyShortLivingWrapper"):
                return True
            current = current.parent
        return False

    def hasChildInterfaces(self):
        return self._hasChildInterfaces

    def isOnGlobalProtoChain(self):
        return self._isOnGlobalProtoChain

    def _getDependentObjects(self):
        deps = set(self.members)
        deps.update(self.includedMixins)
        if self.parent:
            deps.add(self.parent)
        return deps

    def hasMembersInSlots(self):
        return self._ownMembersInSlots != 0

    conditionExtendedAttributes = [
        "Pref",
        "ChromeOnly",
        "Func",
        "Trial",
        "SecureContext",
    ]

    def isExposedConditionally(self, exclusions=[]):
        return any(
            ((a not in exclusions) and self.getExtendedAttribute(a))
            for a in self.conditionExtendedAttributes
        )


class IDLInterface(IDLInterfaceOrNamespace):
    __slots__ = ("classNameOverride",)

    def __init__(
        self,
        location,
        parentScope,
        name,
        parent,
        members,
        isKnownNonPartial,
        classNameOverride=None,
    ):
        IDLInterfaceOrNamespace.__init__(
            self, location, parentScope, name, parent, members, isKnownNonPartial
        )
        self.classNameOverride = classNameOverride

    def __str__(self):
        return "Interface '%s'" % self.identifier.name

    def isInterface(self):
        return True

    def getClassName(self):
        if self.classNameOverride:
            return self.classNameOverride
        return IDLInterfaceOrNamespace.getClassName(self)

    def addExtendedAttributes(self, attrs):
        for attr in attrs:
            identifier = attr.identifier()

            # Special cased attrs
            if identifier == "TreatNonCallableAsNull":
                raise WebIDLError(
                    "TreatNonCallableAsNull cannot be specified on interfaces",
                    [attr.location, self.location],
                )
            if identifier == "LegacyTreatNonObjectAsNull":
                raise WebIDLError(
                    "LegacyTreatNonObjectAsNull cannot be specified on interfaces",
                    [attr.location, self.location],
                )
            elif identifier == "LegacyNoInterfaceObject":
                if not attr.noArguments():
                    raise WebIDLError(
                        "[LegacyNoInterfaceObject] must take no arguments",
                        [attr.location],
                    )

                self._noInterfaceObject = True
            elif identifier == "LegacyFactoryFunction":
                if not attr.hasValue():
                    raise WebIDLError(
                        (
                            "LegacyFactoryFunction must either take an "
                            "identifier or take a named argument list"
                        ),
                        [attr.location],
                    )

                args = attr.args() if attr.hasArgs() else []

                method = IDLConstructor(attr.location, args, attr.value())
                method.reallyInit(self)

                # Legacy factory functions are always assumed to be able to
                # throw (since there's no way to indicate otherwise).
                method.addExtendedAttributes(
                    [IDLExtendedAttribute(self.location, ("Throws",))]
                )

                # We need to detect conflicts for LegacyFactoryFunctions across
                # interfaces. We first call resolve on the parentScope,
                # which will merge all LegacyFactoryFunctions with the same
                # identifier accross interfaces as overloads.
                method.resolve(self.parentScope)

                # Then we look up the identifier on the parentScope. If the
                # result is the same as the method we're adding then it
                # hasn't been added as an overload and it's the first time
                # we've encountered a LegacyFactoryFunction with that identifier.
                # If the result is not the same as the method we're adding
                # then it has been added as an overload and we need to check
                # whether the result is actually one of our existing
                # LegacyFactoryFunctions.
                newMethod = self.parentScope.lookupIdentifier(method.identifier)
                if newMethod == method:
                    self.legacyFactoryFunctions.append(method)
                elif newMethod not in self.legacyFactoryFunctions:
                    raise WebIDLError(
                        "LegacyFactoryFunction conflicts with a "
                        "LegacyFactoryFunction of a different interface",
                        [method.location, newMethod.location],
                    )
            elif identifier == "ExceptionClass":
                if not attr.noArguments():
                    raise WebIDLError(
                        "[ExceptionClass] must take no arguments", [attr.location]
                    )
                if self.parent:
                    raise WebIDLError(
                        "[ExceptionClass] must not be specified on "
                        "an interface with inherited interfaces",
                        [attr.location, self.location],
                    )
            elif identifier == "Global":
                if attr.hasValue():
                    self.globalNames = [attr.value()]
                elif attr.hasArgs():
                    self.globalNames = attr.args()
                else:
                    raise WebIDLError(
                        "[Global] must either take an identifier or take an identifier list",
                        [attr.location, self.location],
                    )
                self.parentScope.addIfaceGlobalNames(
                    self.identifier.name, self.globalNames
                )
                self._isOnGlobalProtoChain = True
            elif identifier == "LegacyWindowAlias":
                if attr.hasValue():
                    self.legacyWindowAliases = [attr.value()]
                elif attr.hasArgs():
                    self.legacyWindowAliases = attr.args()
                else:
                    raise WebIDLError(
                        "[%s] must either take an identifier "
                        "or take an identifier list" % identifier,
                        [attr.location],
                    )
                for alias in self.legacyWindowAliases:
                    unresolved = IDLUnresolvedIdentifier(attr.location, alias)
                    IDLObjectWithIdentifier(attr.location, self.parentScope, unresolved)
            elif identifier == "SecureContext":
                if not attr.noArguments():
                    raise WebIDLError(
                        "[%s] must take no arguments" % identifier, [attr.location]
                    )
                # This gets propagated to all our members.
                for member in self.members:
                    if member.getExtendedAttribute("SecureContext"):
                        raise WebIDLError(
                            "[SecureContext] specified on both "
                            "an interface member and on the "
                            "interface itself",
                            [member.location, attr.location],
                        )
                    member.addExtendedAttributes([attr])
            elif (
                identifier == "NeedResolve"
                or identifier == "LegacyOverrideBuiltIns"
                or identifier == "ChromeOnly"
                or identifier == "LegacyUnforgeable"
                or identifier == "LegacyEventInit"
                or identifier == "ProbablyShortLivingWrapper"
                or identifier == "LegacyUnenumerableNamedProperties"
                or identifier == "RunConstructorInCallerCompartment"
                or identifier == "WantsEventListenerHooks"
                or identifier == "Serializable"
                or identifier == "Abstract"
                or identifier == "Inline"
                or identifier == "Transferable"
                or identifier == "Unimplemented"
            ):
                # Known extended attributes that do not take values
                if not attr.noArguments():
                    raise WebIDLError(
                        "[%s] must take no arguments" % identifier, [attr.location]
                    )
            elif identifier == "Exposed":
                convertExposedAttrToGlobalNameSet(attr, self._exposureGlobalNames)
            elif (
                identifier == "Pref"
                or identifier == "JSImplementation"
                or identifier == "HeaderFile"
                or identifier == "Func"
                or identifier == "Trial"
                or identifier == "Deprecated"
            ):
                # Known extended attributes that take a string value
                if not attr.hasValue():
                    raise WebIDLError(
                        "[%s] must have a value" % identifier, [attr.location]
                    )
            elif identifier == "InstrumentedProps":
                # Known extended attributes that take a list
                if not attr.hasArgs():
                    raise WebIDLError(
                        "[%s] must have arguments" % identifier, [attr.location]
                    )
            else:
                raise WebIDLError(
                    "Unknown extended attribute %s on interface" % identifier,
                    [attr.location],
                )

            attrlist = attr.listValue()
            self._extendedAttrDict[identifier] = attrlist if len(attrlist) else True

    def validate(self):
        IDLInterfaceOrNamespace.validate(self)
        if self.parent and self.isSerializable() and not self.parent.isSerializable():
            raise WebIDLError(
                "Serializable interface inherits from non-serializable "
                "interface.  Per spec, that means the object should not be "
                "serializable, so chances are someone made a mistake here "
                "somewhere.",
                [self.location, self.parent.location],
            )

    def isSerializable(self):
        return self.getExtendedAttribute("Serializable")

    def setNonPartial(self, location, parent, members):
        # Before we do anything else, finish initializing any constructors that
        # might be in "members", so we don't have partially-initialized objects
        # hanging around.  We couldn't do it before now because we needed to have
        # to have the IDLInterface on hand to properly set the return type.
        for member in members:
            if isinstance(member, IDLConstructor):
                member.reallyInit(self)

        IDLInterfaceOrNamespace.setNonPartial(self, location, parent, members)


class IDLNamespace(IDLInterfaceOrNamespace):
    __slots__ = ()

    def __init__(self, location, parentScope, name, members, isKnownNonPartial):
        IDLInterfaceOrNamespace.__init__(
            self, location, parentScope, name, None, members, isKnownNonPartial
        )

    def __str__(self):
        return "Namespace '%s'" % self.identifier.name

    def isNamespace(self):
        return True

    def addExtendedAttributes(self, attrs):
        # The set of things namespaces support is small enough it's simpler
        # to factor out into a separate method than it is to sprinkle
        # isNamespace() checks all through
        # IDLInterfaceOrNamespace.addExtendedAttributes.
        for attr in attrs:
            identifier = attr.identifier()

            if identifier == "Exposed":
                convertExposedAttrToGlobalNameSet(attr, self._exposureGlobalNames)
            elif identifier == "ClassString":
                # Takes a string value to override the default "Object" if
                # desired.
                if not attr.hasValue():
                    raise WebIDLError(
                        "[%s] must have a value" % identifier, [attr.location]
                    )
            elif identifier == "ProtoObjectHack" or identifier == "ChromeOnly":
                if not attr.noArguments():
                    raise WebIDLError(
                        "[%s] must not have arguments" % identifier, [attr.location]
                    )
            elif (
                identifier == "Pref"
                or identifier == "HeaderFile"
                or identifier == "Func"
                or identifier == "Trial"
            ):
                # Known extended attributes that take a string value
                if not attr.hasValue():
                    raise WebIDLError(
                        "[%s] must have a value" % identifier, [attr.location]
                    )
            else:
                raise WebIDLError(
                    "Unknown extended attribute %s on namespace" % identifier,
                    [attr.location],
                )

            attrlist = attr.listValue()
            self._extendedAttrDict[identifier] = attrlist if len(attrlist) else True

    def isSerializable(self):
        return False


class IDLDictionary(IDLObjectWithScope):
    __slots__ = (
        "parent",
        "_finished",
        "members",
        "_partialDictionaries",
        "_extendedAttrDict",
        "needsConversionToJS",
        "needsConversionFromJS",
        "needsEqualityOperator",
    )

    def __init__(self, location, parentScope, name, parent, members):
        assert isinstance(parentScope, IDLScope)
        assert isinstance(name, IDLUnresolvedIdentifier)
        assert not parent or isinstance(parent, IDLIdentifierPlaceholder)

        self.parent = parent
        self._finished = False
        self.members = list(members)
        self._partialDictionaries = []
        self._extendedAttrDict = {}
        self.needsConversionToJS = False
        self.needsConversionFromJS = False
        self.needsEqualityOperator = None

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
        return all(member.optional for member in self.members) and (
            not self.parent or self.parent.canBeEmpty()
        )

    def finish(self, scope):
        if self._finished:
            return

        self._finished = True

        if self.parent:
            assert isinstance(self.parent, IDLIdentifierPlaceholder)
            oldParent = self.parent
            self.parent = self.parent.finish(scope)
            if not isinstance(self.parent, IDLDictionary):
                raise WebIDLError(
                    "Dictionary %s has parent that is not a dictionary"
                    % self.identifier.name,
                    [oldParent.location, self.parent.location],
                )

            # Make sure the parent resolves all its members before we start
            # looking at them.
            self.parent.finish(scope)

        # Now go ahead and merge in our partial dictionaries.
        for partial in self._partialDictionaries:
            partial.finish(scope)
            self.members.extend(partial.members)

        for member in self.members:
            member.resolve(self)
            if not member.isComplete():
                member.complete(scope)
                assert member.type.isComplete()

        # Members of a dictionary are sorted in lexicographic order,
        # unless the dictionary opts out.
        if not self.getExtendedAttribute("Unsorted"):
            self.members.sort(key=lambda x: x.identifier.name)

        inheritedMembers = []
        ancestor = self.parent
        while ancestor:
            if ancestor == self:
                raise WebIDLError(
                    "Dictionary %s has itself as an ancestor" % self.identifier.name,
                    [self.identifier.location],
                )
            inheritedMembers.extend(ancestor.members)
            if (
                self.getExtendedAttribute("GenerateEqualityOperator")
                and ancestor.needsEqualityOperator is None
            ):
                # Store the dictionary that has the [GenerateEqualityOperator]
                # extended attribute, so we can use it when generating error
                # messages.
                ancestor.needsEqualityOperator = self
            ancestor = ancestor.parent

        # Catch name duplication
        for inheritedMember in inheritedMembers:
            for member in self.members:
                if member.identifier.name == inheritedMember.identifier.name:
                    raise WebIDLError(
                        "Dictionary %s has two members with name %s"
                        % (self.identifier.name, member.identifier.name),
                        [member.location, inheritedMember.location],
                    )

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

            if (
                memberType.nullable()
                or memberType.isSequence()
                or memberType.isRecord()
            ):
                return typeContainsDictionary(memberType.inner, dictionary)

            if memberType.isDictionary():
                if memberType.inner == dictionary:
                    return (True, [memberType.location])

                (contains, locations) = dictionaryContainsDictionary(
                    memberType.inner, dictionary
                )
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
                    (contains, locations) = dictionaryContainsDictionary(
                        dictMember.parent, dictionary
                    )
                    if contains:
                        return (True, [dictMember.location] + locations)

            return (False, None)

        for member in self.members:
            if member.type.isDictionary() and member.type.nullable():
                raise WebIDLError(
                    "Dictionary %s has member with nullable "
                    "dictionary type" % self.identifier.name,
                    [member.location],
                )
            (contains, locations) = typeContainsDictionary(member.type, self)
            if contains:
                raise WebIDLError(
                    "Dictionary %s has member with itself as type."
                    % self.identifier.name,
                    [member.location] + locations,
                )

            if member.type.isUndefined():
                raise WebIDLError(
                    "Dictionary %s has member with undefined as its type."
                    % self.identifier.name,
                    [member.location],
                )
            elif member.type.isUnion():
                for unionMember in member.type.unroll().flatMemberTypes:
                    if unionMember.isUndefined():
                        raise WebIDLError(
                            "Dictionary %s has member with a union containing "
                            "undefined as a type." % self.identifier.name,
                            [unionMember.location],
                        )

    def getExtendedAttribute(self, name):
        return self._extendedAttrDict.get(name, None)

    def addExtendedAttributes(self, attrs):
        for attr in attrs:
            identifier = attr.identifier()

            if identifier == "GenerateInitFromJSON" or identifier == "GenerateInit":
                if not attr.noArguments():
                    raise WebIDLError(
                        "[%s] must not have arguments" % identifier, [attr.location]
                    )
                self.needsConversionFromJS = True
            elif (
                identifier == "GenerateConversionToJS" or identifier == "GenerateToJSON"
            ):
                if not attr.noArguments():
                    raise WebIDLError(
                        "[%s] must not have arguments" % identifier, [attr.location]
                    )
                # ToJSON methods require to-JS conversion, because we
                # implement ToJSON by converting to a JS object and
                # then using JSON.stringify.
                self.needsConversionToJS = True
            elif identifier == "GenerateEqualityOperator":
                if not attr.noArguments():
                    raise WebIDLError(
                        "[GenerateEqualityOperator] must take no arguments",
                        [attr.location],
                    )
                self.needsEqualityOperator = self
            elif identifier == "Unsorted":
                if not attr.noArguments():
                    raise WebIDLError(
                        "[Unsorted] must take no arguments", [attr.location]
                    )
            else:
                raise WebIDLError(
                    "[%s] extended attribute not allowed on "
                    "dictionaries" % identifier,
                    [attr.location],
                )

            self._extendedAttrDict[identifier] = True

    def _getDependentObjects(self):
        deps = set(self.members)
        if self.parent:
            deps.add(self.parent)
        return deps

    def addPartialDictionary(self, partial):
        assert self.identifier.name == partial.identifier.name
        self._partialDictionaries.append(partial)


class IDLEnum(IDLObjectWithIdentifier):
    __slots__ = ("_values",)

    def __init__(self, location, parentScope, name, values):
        assert isinstance(parentScope, IDLScope)
        assert isinstance(name, IDLUnresolvedIdentifier)

        if len(values) != len(set(values)):
            raise WebIDLError(
                "Enum %s has multiple identical strings" % name.name, [location]
            )

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
        if len(attrs) != 0:
            raise WebIDLError(
                "There are no extended attributes that are " "allowed on enums",
                [attrs[0].location, self.location],
            )

    def _getDependentObjects(self):
        return set()


class IDLType(IDLObject):
    Tags = enum(
        # The integer types
        "int8",
        "uint8",
        "int16",
        "uint16",
        "int32",
        "uint32",
        "int64",
        "uint64",
        # Additional primitive types
        "bool",
        "unrestricted_float",
        "float",
        "unrestricted_double",
        # "double" last primitive type to match IDLBuiltinType
        "double",
        # Other types
        "any",
        "undefined",
        "domstring",
        "bytestring",
        "usvstring",
        "utf8string",
        "jsstring",
        "object",
        # Funny stuff
        "interface",
        "int8array",
        "uint8array",
        "int16array",
        "uint16array",
        "int32array",
        "uint32array",
        "float32array",
        "float64array",
        "arrayBuffer",
        "arrayBufferView",
        "uint8clampedarray",
        "dictionary",
        "enum",
        "callback",
        "union",
        "sequence",
        "record",
        "promise",
        "observablearray",
    )

    __slots__ = (
        "name",
        "builtin",
        "legacyNullToEmptyString",
        "_clamp",
        "_enforceRange",
        "_allowShared",
        "_extendedAttrDict",
    )

    def __init__(self, location, name):
        IDLObject.__init__(self, location)
        self.name = name
        self.builtin = False
        self.legacyNullToEmptyString = False
        self._clamp = False
        self._enforceRange = False
        self._allowShared = False
        self._extendedAttrDict = {}

    def __hash__(self):
        return (
            hash(self.builtin)
            + hash(self.name)
            + hash(self._clamp)
            + hash(self._enforceRange)
            + hash(self.legacyNullToEmptyString)
            + hash(self._allowShared)
        )

    def __eq__(self, other):
        return (
            other
            and self.builtin == other.builtin
            and self.name == other.name
            and self._clamp == other.hasClamp()
            and self._enforceRange == other.hasEnforceRange()
            and self.legacyNullToEmptyString == other.legacyNullToEmptyString
            and self._allowShared == other.hasAllowShared()
        )

    def __ne__(self, other):
        return not self == other

    def __str__(self):
        return str(self.name)

    def prettyName(self):
        """
        A name that looks like what this type is named in the IDL spec.  By default
        this is just our .name, but types that have more interesting spec
        representations should override this.
        """
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

    def isUTF8String(self):
        return False

    def isJSString(self):
        return False

    def isUndefined(self):
        return False

    def isSequence(self):
        return False

    def isRecord(self):
        return False

    def isArrayBuffer(self):
        return False

    def isArrayBufferView(self):
        return False

    def isTypedArray(self):
        return False

    def isBufferSource(self):
        return self.isArrayBuffer() or self.isArrayBufferView() or self.isTypedArray()

    def isCallbackInterface(self):
        return False

    def isNonCallbackInterface(self):
        return False

    def isGeckoInterface(self):
        """Returns a boolean indicating whether this type is an 'interface'
        type that is implemented in Gecko. At the moment, this returns
        true for all interface types that are not types from the TypedArray
        spec."""
        return self.isInterface() and not self.isSpiderMonkeyInterface()

    def isSpiderMonkeyInterface(self):
        """Returns a boolean indicating whether this type is an 'interface'
        type that is implemented in SpiderMonkey."""
        return self.isInterface() and self.isBufferSource()

    def isAny(self):
        return self.tag() == IDLType.Tags.any

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

    def isJSONType(self):
        return False

    def isObservableArray(self):
        return False

    def isDictionaryLike(self):
        return self.isDictionary() or self.isRecord() or self.isCallbackInterface()

    def hasClamp(self):
        return self._clamp

    def hasEnforceRange(self):
        return self._enforceRange

    def hasAllowShared(self):
        return self._allowShared

    def tag(self):
        assert False  # Override me!

    def treatNonCallableAsNull(self):
        assert self.tag() == IDLType.Tags.callback
        return self.nullable() and self.inner.callback._treatNonCallableAsNull

    def treatNonObjectAsNull(self):
        assert self.tag() == IDLType.Tags.callback
        return self.nullable() and self.inner.callback._treatNonObjectAsNull

    def withExtendedAttributes(self, attrs):
        if len(attrs) > 0:
            raise WebIDLError(
                "Extended attributes on types only supported for builtins",
                [attrs[0].location, self.location],
            )
        return self

    def getExtendedAttribute(self, name):
        return self._extendedAttrDict.get(name, None)

    def resolveType(self, parentScope):
        pass

    def unroll(self):
        return self

    def isDistinguishableFrom(self, other):
        raise TypeError(
            "Can't tell whether a generic type is or is not "
            "distinguishable from other things"
        )

    def isExposedInAllOf(self, exposureSet):
        return True


class IDLUnresolvedType(IDLType):
    """
    Unresolved types are interface types
    """

    __slots__ = ("extraTypeAttributes",)

    def __init__(self, location, name, attrs=[]):
        IDLType.__init__(self, location, name)
        self.extraTypeAttributes = attrs

    def isComplete(self):
        return False

    def complete(self, scope):
        obj = None
        try:
            obj = scope._lookupIdentifier(self.name)
        except Exception:
            raise WebIDLError("Unresolved type '%s'." % self.name, [self.location])

        assert obj
        assert not obj.isType()
        if obj.isTypedef():
            assert self.name.name == obj.identifier.name
            typedefType = IDLTypedefType(self.location, obj.innerType, obj.identifier)
            assert not typedefType.isComplete()
            return typedefType.complete(scope).withExtendedAttributes(
                self.extraTypeAttributes
            )
        elif obj.isCallback() and not obj.isInterface():
            assert self.name.name == obj.identifier.name
            return IDLCallbackType(obj.location, obj)

        return IDLWrapperType(self.location, obj)

    def withExtendedAttributes(self, attrs):
        return IDLUnresolvedType(self.location, self.name, attrs)

    def isDistinguishableFrom(self, other):
        raise TypeError(
            "Can't tell whether an unresolved type is or is not "
            "distinguishable from other things"
        )


class IDLParametrizedType(IDLType):
    __slots__ = "builtin", "inner"

    def __init__(self, location, name, innerType):
        IDLType.__init__(self, location, name)
        self.builtin = False
        self.inner = innerType

    def includesRestrictedFloat(self):
        return self.inner.includesRestrictedFloat()

    def resolveType(self, parentScope):
        assert isinstance(parentScope, IDLScope)
        self.inner.resolveType(parentScope)

    def isComplete(self):
        return self.inner.isComplete()

    def unroll(self):
        return self.inner.unroll()

    def _getDependentObjects(self):
        return self.inner._getDependentObjects()


class IDLNullableType(IDLParametrizedType):
    __slots__ = ()

    def __init__(self, location, innerType):
        assert not innerType == BuiltinTypes[IDLBuiltinType.Types.any]

        IDLParametrizedType.__init__(self, location, None, innerType)

    def __hash__(self):
        return hash(self.inner)

    def __eq__(self, other):
        return isinstance(other, IDLNullableType) and self.inner == other.inner

    def __str__(self):
        return self.inner.__str__() + "OrNull"

    def prettyName(self):
        return self.inner.prettyName() + "?"

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

    def isUTF8String(self):
        return self.inner.isUTF8String()

    def isJSString(self):
        return self.inner.isJSString()

    def isFloat(self):
        return self.inner.isFloat()

    def isUnrestricted(self):
        return self.inner.isUnrestricted()

    def isInteger(self):
        return self.inner.isInteger()

    def isUndefined(self):
        return self.inner.isUndefined()

    def isSequence(self):
        return self.inner.isSequence()

    def isRecord(self):
        return self.inner.isRecord()

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

    def isPromise(self):
        # There is no such thing as a nullable Promise.
        assert not self.inner.isPromise()
        return False

    def isCallbackInterface(self):
        return self.inner.isCallbackInterface()

    def isNonCallbackInterface(self):
        return self.inner.isNonCallbackInterface()

    def isEnum(self):
        return self.inner.isEnum()

    def isUnion(self):
        return self.inner.isUnion()

    def isJSONType(self):
        return self.inner.isJSONType()

    def isObservableArray(self):
        return self.inner.isObservableArray()

    def hasClamp(self):
        return self.inner.hasClamp()

    def hasEnforceRange(self):
        return self.inner.hasEnforceRange()

    def hasAllowShared(self):
        return self.inner.hasAllowShared()

    def isComplete(self):
        return self.name is not None

    def tag(self):
        return self.inner.tag()

    def complete(self, scope):
        if not self.inner.isComplete():
            self.inner = self.inner.complete(scope)
        assert self.inner.isComplete()

        if self.inner.nullable():
            raise WebIDLError(
                "The inner type of a nullable type must not be a nullable type",
                [self.location, self.inner.location],
            )
        if self.inner.isUnion():
            if self.inner.hasNullableType:
                raise WebIDLError(
                    "The inner type of a nullable type must not "
                    "be a union type that itself has a nullable "
                    "type as a member type",
                    [self.location],
                )
        if self.inner.isDOMString():
            if self.inner.legacyNullToEmptyString:
                raise WebIDLError(
                    "[LegacyNullToEmptyString] not allowed on a nullable DOMString",
                    [self.location, self.inner.location],
                )
        if self.inner.isObservableArray():
            raise WebIDLError(
                "The inner type of a nullable type must not be an ObservableArray type",
                [self.location, self.inner.location],
            )

        self.name = self.inner.name + "OrNull"
        return self

    def isDistinguishableFrom(self, other):
        if (
            other.nullable()
            or other.isDictionary()
            or (
                other.isUnion() and (other.hasNullableType or other.hasDictionaryType())
            )
        ):
            # Can't tell which type null should become
            return False
        return self.inner.isDistinguishableFrom(other)

    def withExtendedAttributes(self, attrs):
        # See https://github.com/heycam/webidl/issues/827#issuecomment-565131350
        # Allowing extended attributes to apply to a nullable type is an intermediate solution.
        # A potential longer term solution is to introduce a null type and get rid of nullables.
        # For example, we could do `([Clamp] long or null) foo` in the future.
        return IDLNullableType(self.location, self.inner.withExtendedAttributes(attrs))


class IDLSequenceType(IDLParametrizedType):
    __slots__ = ("name",)

    def __init__(self, location, parameterType):
        assert not parameterType.isUndefined()

        IDLParametrizedType.__init__(self, location, parameterType.name, parameterType)
        # Need to set self.name up front if our inner type is already complete,
        # since in that case our .complete() won't be called.
        if self.inner.isComplete():
            self.name = self.inner.name + "Sequence"

    def __hash__(self):
        return hash(self.inner)

    def __eq__(self, other):
        return isinstance(other, IDLSequenceType) and self.inner == other.inner

    def __str__(self):
        return self.inner.__str__() + "Sequence"

    def prettyName(self):
        return "sequence<%s>" % self.inner.prettyName()

    def isSequence(self):
        return True

    def isJSONType(self):
        return self.inner.isJSONType()

    def tag(self):
        return IDLType.Tags.sequence

    def complete(self, scope):
        if self.inner.isObservableArray():
            raise WebIDLError(
                "The inner type of a sequence type must not be an ObservableArray type",
                [self.location, self.inner.location],
            )

        self.inner = self.inner.complete(scope)
        self.name = self.inner.name + "Sequence"
        return self

    def isDistinguishableFrom(self, other):
        if other.isPromise():
            return False
        if other.isUnion():
            # Just forward to the union; it'll deal
            return other.isDistinguishableFrom(self)
        return (
            other.isUndefined()
            or other.isPrimitive()
            or other.isString()
            or other.isEnum()
            or other.isInterface()
            or other.isDictionary()
            or other.isCallback()
            or other.isRecord()
        )


class IDLRecordType(IDLParametrizedType):
    __slots__ = "keyType", "name"

    def __init__(self, location, keyType, valueType):
        assert keyType.isString()
        assert keyType.isComplete()

        if valueType.isUndefined():
            raise WebIDLError(
                "We don't support undefined as a Record's values' type",
                [location, valueType.location],
            )

        IDLParametrizedType.__init__(self, location, valueType.name, valueType)
        self.keyType = keyType

        # Need to set self.name up front if our inner type is already complete,
        # since in that case our .complete() won't be called.
        if self.inner.isComplete():
            self.name = self.keyType.name + self.inner.name + "Record"

    def __hash__(self):
        return hash(self.inner)

    def __eq__(self, other):
        return isinstance(other, IDLRecordType) and self.inner == other.inner

    def __str__(self):
        return self.keyType.__str__() + self.inner.__str__() + "Record"

    def prettyName(self):
        return "record<%s, %s>" % (self.keyType.prettyName(), self.inner.prettyName())

    def isRecord(self):
        return True

    def isJSONType(self):
        return self.inner.isJSONType()

    def tag(self):
        return IDLType.Tags.record

    def complete(self, scope):
        if self.inner.isObservableArray():
            raise WebIDLError(
                "The value type of a record type must not be an ObservableArray type",
                [self.location, self.inner.location],
            )

        self.inner = self.inner.complete(scope)
        self.name = self.keyType.name + self.inner.name + "Record"
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
        if other.isCallback():
            # Let other determine if it's a LegacyTreatNonObjectAsNull callback
            return other.isDistinguishableFrom(self)
        return (
            other.isPrimitive()
            or other.isString()
            or other.isEnum()
            or other.isNonCallbackInterface()
            or other.isSequence()
        )

    def isExposedInAllOf(self, exposureSet):
        return self.inner.unroll().isExposedInAllOf(exposureSet)


class IDLObservableArrayType(IDLParametrizedType):
    __slots__ = ()

    def __init__(self, location, innerType):
        assert not innerType.isUndefined()
        IDLParametrizedType.__init__(self, location, None, innerType)

    def __hash__(self):
        return hash(self.inner)

    def __eq__(self, other):
        return isinstance(other, IDLObservableArrayType) and self.inner == other.inner

    def __str__(self):
        return self.inner.__str__() + "ObservableArray"

    def prettyName(self):
        return "ObservableArray<%s>" % self.inner.prettyName()

    def isJSONType(self):
        return self.inner.isJSONType()

    def isObservableArray(self):
        return True

    def isComplete(self):
        return self.name is not None

    def tag(self):
        return IDLType.Tags.observablearray

    def complete(self, scope):
        if not self.inner.isComplete():
            self.inner = self.inner.complete(scope)
        assert self.inner.isComplete()

        if self.inner.isDictionary():
            raise WebIDLError(
                "The inner type of an ObservableArray type must not "
                "be a dictionary type",
                [self.location, self.inner.location],
            )
        if self.inner.isSequence():
            raise WebIDLError(
                "The inner type of an ObservableArray type must not "
                "be a sequence type",
                [self.location, self.inner.location],
            )
        if self.inner.isRecord():
            raise WebIDLError(
                "The inner type of an ObservableArray type must not be a record type",
                [self.location, self.inner.location],
            )
        if self.inner.isObservableArray():
            raise WebIDLError(
                "The inner type of an ObservableArray type must not "
                "be an ObservableArray type",
                [self.location, self.inner.location],
            )

        self.name = self.inner.name + "ObservableArray"
        return self

    def isDistinguishableFrom(self, other):
        # ObservableArrays are not distinguishable from anything.
        return False


class IDLUnionType(IDLType):
    __slots__ = (
        "memberTypes",
        "hasNullableType",
        "_dictionaryType",
        "flatMemberTypes",
        "builtin",
    )

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

    def prettyName(self):
        return "(" + " or ".join(m.prettyName() for m in self.memberTypes) + ")"

    def isUnion(self):
        return True

    def isJSONType(self):
        return all(m.isJSONType() for m in self.memberTypes)

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
            if isinstance(type, IDLBuiltinType) and type.hasAllowShared():
                assert type.isBufferSource()
                return "MaybeShared" + type.name
            return type.name

        for i, type in enumerate(self.memberTypes):
            if not type.isComplete():
                self.memberTypes[i] = type.complete(scope)

        self.name = "Or".join(typeName(type) for type in self.memberTypes)
        self.flatMemberTypes = list(self.memberTypes)
        i = 0
        while i < len(self.flatMemberTypes):
            if self.flatMemberTypes[i].nullable():
                if self.hasNullableType:
                    raise WebIDLError(
                        "Can't have more than one nullable types in a union",
                        [nullableType.location, self.flatMemberTypes[i].location],
                    )
                if self.hasDictionaryType():
                    raise WebIDLError(
                        "Can't have a nullable type and a "
                        "dictionary type in a union",
                        [
                            self._dictionaryType.location,
                            self.flatMemberTypes[i].location,
                        ],
                    )
                self.hasNullableType = True
                nullableType = self.flatMemberTypes[i]
                self.flatMemberTypes[i] = self.flatMemberTypes[i].inner
                continue
            if self.flatMemberTypes[i].isDictionary():
                if self.hasNullableType:
                    raise WebIDLError(
                        "Can't have a nullable type and a "
                        "dictionary type in a union",
                        [nullableType.location, self.flatMemberTypes[i].location],
                    )
                self._dictionaryType = self.flatMemberTypes[i]
                self.flatMemberTypes[i].inner.needsConversionFromJS = True
            elif self.flatMemberTypes[i].isUnion():
                self.flatMemberTypes[i : i + 1] = self.flatMemberTypes[i].memberTypes
                continue
            i += 1

        for i, t in enumerate(self.flatMemberTypes[:-1]):
            for u in self.flatMemberTypes[i + 1 :]:
                if not t.isDistinguishableFrom(u):
                    raise WebIDLError(
                        "Flat member types of a union should be "
                        "distinguishable, " + str(t) + " is not "
                        "distinguishable from " + str(u),
                        [self.location, t.location, u.location],
                    )

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
        # We could have different member types in different globals.
        # Just make sure that each thing in exposureSet has one of our member types exposed in it.
        for globalName in exposureSet:
            if not any(
                t.unroll().isExposedInAllOf(set([globalName]))
                for t in self.flatMemberTypes
            ):
                return False
        return True

    def hasDictionaryType(self):
        return self._dictionaryType is not None

    def hasPossiblyEmptyDictionaryType(self):
        return (
            self._dictionaryType is not None and self._dictionaryType.inner.canBeEmpty()
        )

    def _getDependentObjects(self):
        return set(self.memberTypes)


class IDLTypedefType(IDLType):
    __slots__ = "inner", "builtin"

    def __init__(self, location, innerType, name):
        IDLType.__init__(self, location, name)
        self.inner = innerType
        self.builtin = False

    def __hash__(self):
        return hash(self.inner)

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

    def isUTF8String(self):
        return self.inner.isUTF8String()

    def isJSString(self):
        return self.inner.isJSString()

    def isUndefined(self):
        return self.inner.isUndefined()

    def isJSONType(self):
        return self.inner.isJSONType()

    def isSequence(self):
        return self.inner.isSequence()

    def isRecord(self):
        return self.inner.isRecord()

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

    # Do we need a resolveType impl?  I don't think it's particularly useful....

    def tag(self):
        return self.inner.tag()

    def unroll(self):
        return self.inner.unroll()

    def isDistinguishableFrom(self, other):
        return self.inner.isDistinguishableFrom(other)

    def _getDependentObjects(self):
        return self.inner._getDependentObjects()

    def withExtendedAttributes(self, attrs):
        return IDLTypedefType(
            self.location, self.inner.withExtendedAttributes(attrs), self.name
        )


class IDLTypedef(IDLObjectWithIdentifier):
    __slots__ = ("innerType",)

    def __init__(self, location, parentScope, innerType, name):
        # Set self.innerType first, because IDLObjectWithIdentifier.__init__
        # will call our __str__, which wants to use it.
        self.innerType = innerType
        identifier = IDLUnresolvedIdentifier(location, name)
        IDLObjectWithIdentifier.__init__(self, location, parentScope, identifier)

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
        if len(attrs) != 0:
            raise WebIDLError(
                "There are no extended attributes that are " "allowed on typedefs",
                [attrs[0].location, self.location],
            )

    def _getDependentObjects(self):
        return self.innerType._getDependentObjects()


class IDLWrapperType(IDLType):
    __slots__ = "inner", "_identifier", "builtin"

    def __init__(self, location, inner):
        IDLType.__init__(self, location, inner.identifier.name)
        self.inner = inner
        self._identifier = inner.identifier
        self.builtin = False

    def __hash__(self):
        return hash(self._identifier) + hash(self.builtin)

    def __eq__(self, other):
        return (
            isinstance(other, IDLWrapperType)
            and self._identifier == other._identifier
            and self.builtin == other.builtin
        )

    def __str__(self):
        return str(self.name) + " (Wrapper)"

    def isDictionary(self):
        return isinstance(self.inner, IDLDictionary)

    def isInterface(self):
        return isinstance(self.inner, IDLInterface) or isinstance(
            self.inner, IDLExternalInterface
        )

    def isCallbackInterface(self):
        return self.isInterface() and self.inner.isCallback()

    def isNonCallbackInterface(self):
        return self.isInterface() and not self.inner.isCallback()

    def isEnum(self):
        return isinstance(self.inner, IDLEnum)

    def isJSONType(self):
        if self.isInterface():
            if self.inner.isExternal():
                return False
            iface = self.inner
            while iface:
                if any(m.isMethod() and m.isToJSON() for m in iface.members):
                    return True
                iface = iface.parent
            return False
        elif self.isEnum():
            return True
        elif self.isDictionary():
            dictionary = self.inner
            while dictionary:
                if not all(m.type.isJSONType() for m in dictionary.members):
                    return False
                dictionary = dictionary.parent
            return True
        else:
            raise WebIDLError(
                "IDLWrapperType wraps type %s that we don't know if "
                "is serializable" % type(self.inner),
                [self.location],
            )

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
        if other.isPromise():
            return False
        if other.isUnion():
            # Just forward to the union; it'll deal
            return other.isDistinguishableFrom(self)
        assert self.isInterface() or self.isEnum() or self.isDictionary()
        if self.isEnum():
            return (
                other.isUndefined()
                or other.isPrimitive()
                or other.isInterface()
                or other.isObject()
                or other.isCallback()
                or other.isDictionary()
                or other.isSequence()
                or other.isRecord()
            )
        if self.isDictionary() and other.nullable():
            return False
        if (
            other.isPrimitive()
            or other.isString()
            or other.isEnum()
            or other.isSequence()
        ):
            return True

        # If this needs to handle other dictionary-like types we probably need
        # some additional checks first.
        assert self.isDictionaryLike() == (
            self.isDictionary() or self.isCallbackInterface()
        )
        if self.isDictionaryLike():
            if other.isCallback():
                # Let other determine if it's a LegacyTreatNonObjectAsNull callback
                return other.isDistinguishableFrom(self)

            assert (
                other.isNonCallbackInterface()
                or other.isAny()
                or other.isUndefined()
                or other.isObject()
                or other.isDictionaryLike()
            )
            # At this point, dictionary-like (for 'self') and interface-like
            # (for 'other') are the only two that are distinguishable.
            # any is the union of all non-union types, so it's not distinguishable
            # from other unions (because it is a union itself), or from all
            # non-union types (because it has all of them as its members).
            return other.isNonCallbackInterface()

        assert self.isNonCallbackInterface()

        if other.isUndefined() or other.isDictionaryLike() or other.isCallback():
            return True

        if other.isNonCallbackInterface():
            if other.isSpiderMonkeyInterface():
                # Just let |other| handle things
                return other.isDistinguishableFrom(self)

            assert self.isGeckoInterface() and other.isGeckoInterface()
            if self.inner.isExternal() or other.unroll().inner.isExternal():
                return self != other
            return (
                len(
                    self.inner.interfacesBasedOnSelf
                    & other.unroll().inner.interfacesBasedOnSelf
                )
                == 0
            )

        # Not much else |other| can be.
        # any is the union of all non-union types, so it's not distinguishable
        # from other unions (because it is a union itself), or from all
        # non-union types (because it has all of them as its members).
        assert other.isAny() or other.isObject()
        return False

    def isExposedInAllOf(self, exposureSet):
        if not self.isInterface():
            return True
        iface = self.inner
        if iface.isExternal():
            # Let's say true, so we don't have to implement exposure mixins on
            # external interfaces and sprinkle [Exposed=Window] on every single
            # external interface declaration.
            return True
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


class IDLPromiseType(IDLParametrizedType):
    __slots__ = ()

    def __init__(self, location, innerType):
        IDLParametrizedType.__init__(self, location, "Promise", innerType)

    def __hash__(self):
        return hash(self.promiseInnerType())

    def __eq__(self, other):
        return (
            isinstance(other, IDLPromiseType)
            and self.promiseInnerType() == other.promiseInnerType()
        )

    def __str__(self):
        return self.inner.__str__() + "Promise"

    def prettyName(self):
        return "Promise<%s>" % self.inner.prettyName()

    def isPromise(self):
        return True

    def promiseInnerType(self):
        return self.inner

    def tag(self):
        return IDLType.Tags.promise

    def complete(self, scope):
        if self.inner.isObservableArray():
            raise WebIDLError(
                "The inner type of a promise type must not be an ObservableArray type",
                [self.location, self.inner.location],
            )

        self.inner = self.promiseInnerType().complete(scope)
        return self

    def unroll(self):
        # We do not unroll our inner.  Just stop at ourselves.  That
        # lets us add headers for both ourselves and our inner as
        # needed.
        return self

    def isDistinguishableFrom(self, other):
        # Promises are not distinguishable from anything.
        return False

    def isExposedInAllOf(self, exposureSet):
        # Check the internal type
        return self.promiseInnerType().unroll().isExposedInAllOf(exposureSet)


class IDLBuiltinType(IDLType):
    Types = enum(
        # The integer types
        "byte",
        "octet",
        "short",
        "unsigned_short",
        "long",
        "unsigned_long",
        "long_long",
        "unsigned_long_long",
        # Additional primitive types
        "boolean",
        "unrestricted_float",
        "float",
        "unrestricted_double",
        # IMPORTANT: "double" must be the last primitive type listed
        "double",
        # Other types
        "any",
        "undefined",
        "domstring",
        "bytestring",
        "usvstring",
        "utf8string",
        "jsstring",
        "object",
        # Funny stuff
        "ArrayBuffer",
        "ArrayBufferView",
        "Int8Array",
        "Uint8Array",
        "Uint8ClampedArray",
        "Int16Array",
        "Uint16Array",
        "Int32Array",
        "Uint32Array",
        "Float32Array",
        "Float64Array",
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
        Types.undefined: IDLType.Tags.undefined,
        Types.domstring: IDLType.Tags.domstring,
        Types.bytestring: IDLType.Tags.bytestring,
        Types.usvstring: IDLType.Tags.usvstring,
        Types.utf8string: IDLType.Tags.utf8string,
        Types.jsstring: IDLType.Tags.jsstring,
        Types.object: IDLType.Tags.object,
        Types.ArrayBuffer: IDLType.Tags.arrayBuffer,
        Types.ArrayBufferView: IDLType.Tags.arrayBufferView,
        Types.Int8Array: IDLType.Tags.int8array,
        Types.Uint8Array: IDLType.Tags.uint8array,
        Types.Uint8ClampedArray: IDLType.Tags.uint8clampedarray,
        Types.Int16Array: IDLType.Tags.int16array,
        Types.Uint16Array: IDLType.Tags.uint16array,
        Types.Int32Array: IDLType.Tags.int32array,
        Types.Uint32Array: IDLType.Tags.uint32array,
        Types.Float32Array: IDLType.Tags.float32array,
        Types.Float64Array: IDLType.Tags.float64array,
    }

    PrettyNames = {
        Types.byte: "byte",
        Types.octet: "octet",
        Types.short: "short",
        Types.unsigned_short: "unsigned short",
        Types.long: "long",
        Types.unsigned_long: "unsigned long",
        Types.long_long: "long long",
        Types.unsigned_long_long: "unsigned long long",
        Types.boolean: "boolean",
        Types.unrestricted_float: "unrestricted float",
        Types.float: "float",
        Types.unrestricted_double: "unrestricted double",
        Types.double: "double",
        Types.any: "any",
        Types.undefined: "undefined",
        Types.domstring: "DOMString",
        Types.bytestring: "ByteString",
        Types.usvstring: "USVString",
        Types.utf8string: "USVString",  # That's what it is in spec terms
        Types.jsstring: "USVString",  # Again, that's what it is in spec terms
        Types.object: "object",
        Types.ArrayBuffer: "ArrayBuffer",
        Types.ArrayBufferView: "ArrayBufferView",
        Types.Int8Array: "Int8Array",
        Types.Uint8Array: "Uint8Array",
        Types.Uint8ClampedArray: "Uint8ClampedArray",
        Types.Int16Array: "Int16Array",
        Types.Uint16Array: "Uint16Array",
        Types.Int32Array: "Int32Array",
        Types.Uint32Array: "Uint32Array",
        Types.Float32Array: "Float32Array",
        Types.Float64Array: "Float64Array",
    }

    __slots__ = (
        "_typeTag",
        "_clamped",
        "_rangeEnforced",
        "_withLegacyNullToEmptyString",
        "_withAllowShared",
    )

    def __init__(
        self,
        location,
        name,
        type,
        clamp=False,
        enforceRange=False,
        legacyNullToEmptyString=False,
        allowShared=False,
        attrLocation=[],
    ):
        """
        The mutually exclusive clamp/enforceRange/legacyNullToEmptyString/allowShared arguments
        are used to create instances of this type with the appropriate attributes attached. Use
        .clamped(), .rangeEnforced(), .withLegacyNullToEmptyString() and .withAllowShared().

        attrLocation is an array of source locations of these attributes for error reporting.
        """
        IDLType.__init__(self, location, name)
        self.builtin = True
        self._typeTag = type
        self._clamped = None
        self._rangeEnforced = None
        self._withLegacyNullToEmptyString = None
        self._withAllowShared = None
        if self.isInteger():
            if clamp:
                self._clamp = True
                self.name = "Clamped" + self.name
                self._extendedAttrDict["Clamp"] = True
            elif enforceRange:
                self._enforceRange = True
                self.name = "RangeEnforced" + self.name
                self._extendedAttrDict["EnforceRange"] = True
        elif clamp or enforceRange:
            raise WebIDLError(
                "Non-integer types cannot be [Clamp] or [EnforceRange]", attrLocation
            )
        if self.isDOMString() or self.isUTF8String():
            if legacyNullToEmptyString:
                self.legacyNullToEmptyString = True
                self.name = "NullIsEmpty" + self.name
                self._extendedAttrDict["LegacyNullToEmptyString"] = True
        elif legacyNullToEmptyString:
            raise WebIDLError(
                "Non-string types cannot be [LegacyNullToEmptyString]", attrLocation
            )
        if self.isBufferSource():
            if allowShared:
                self._allowShared = True
                self._extendedAttrDict["AllowShared"] = True
        elif allowShared:
            raise WebIDLError(
                "Types that are not buffer source types cannot be [AllowShared]",
                attrLocation,
            )

    def __str__(self):
        if self._allowShared:
            assert self.isBufferSource()
            return "MaybeShared" + str(self.name)
        return str(self.name)

    def prettyName(self):
        return IDLBuiltinType.PrettyNames[self._typeTag]

    def clamped(self, attrLocation):
        if not self._clamped:
            self._clamped = IDLBuiltinType(
                self.location,
                self.name,
                self._typeTag,
                clamp=True,
                attrLocation=attrLocation,
            )
        return self._clamped

    def rangeEnforced(self, attrLocation):
        if not self._rangeEnforced:
            self._rangeEnforced = IDLBuiltinType(
                self.location,
                self.name,
                self._typeTag,
                enforceRange=True,
                attrLocation=attrLocation,
            )
        return self._rangeEnforced

    def withLegacyNullToEmptyString(self, attrLocation):
        if not self._withLegacyNullToEmptyString:
            self._withLegacyNullToEmptyString = IDLBuiltinType(
                self.location,
                self.name,
                self._typeTag,
                legacyNullToEmptyString=True,
                attrLocation=attrLocation,
            )
        return self._withLegacyNullToEmptyString

    def withAllowShared(self, attrLocation):
        if not self._withAllowShared:
            self._withAllowShared = IDLBuiltinType(
                self.location,
                self.name,
                self._typeTag,
                allowShared=True,
                attrLocation=attrLocation,
            )
        return self._withAllowShared

    def isPrimitive(self):
        return self._typeTag <= IDLBuiltinType.Types.double

    def isBoolean(self):
        return self._typeTag == IDLBuiltinType.Types.boolean

    def isUndefined(self):
        return self._typeTag == IDLBuiltinType.Types.undefined

    def isNumeric(self):
        return self.isPrimitive() and not self.isBoolean()

    def isString(self):
        return (
            self._typeTag == IDLBuiltinType.Types.domstring
            or self._typeTag == IDLBuiltinType.Types.bytestring
            or self._typeTag == IDLBuiltinType.Types.usvstring
            or self._typeTag == IDLBuiltinType.Types.utf8string
            or self._typeTag == IDLBuiltinType.Types.jsstring
        )

    def isByteString(self):
        return self._typeTag == IDLBuiltinType.Types.bytestring

    def isDOMString(self):
        return self._typeTag == IDLBuiltinType.Types.domstring

    def isUSVString(self):
        return self._typeTag == IDLBuiltinType.Types.usvstring

    def isUTF8String(self):
        return self._typeTag == IDLBuiltinType.Types.utf8string

    def isJSString(self):
        return self._typeTag == IDLBuiltinType.Types.jsstring

    def isInteger(self):
        return self._typeTag <= IDLBuiltinType.Types.unsigned_long_long

    def isArrayBuffer(self):
        return self._typeTag == IDLBuiltinType.Types.ArrayBuffer

    def isArrayBufferView(self):
        return self._typeTag == IDLBuiltinType.Types.ArrayBufferView

    def isTypedArray(self):
        return (
            self._typeTag >= IDLBuiltinType.Types.Int8Array
            and self._typeTag <= IDLBuiltinType.Types.Float64Array
        )

    def isInterface(self):
        # TypedArray things are interface types per the TypedArray spec,
        # but we handle them as builtins because SpiderMonkey implements
        # all of it internally.
        return self.isArrayBuffer() or self.isArrayBufferView() or self.isTypedArray()

    def isNonCallbackInterface(self):
        # All the interfaces we can be are non-callback
        return self.isInterface()

    def isFloat(self):
        return (
            self._typeTag == IDLBuiltinType.Types.float
            or self._typeTag == IDLBuiltinType.Types.double
            or self._typeTag == IDLBuiltinType.Types.unrestricted_float
            or self._typeTag == IDLBuiltinType.Types.unrestricted_double
        )

    def isUnrestricted(self):
        assert self.isFloat()
        return (
            self._typeTag == IDLBuiltinType.Types.unrestricted_float
            or self._typeTag == IDLBuiltinType.Types.unrestricted_double
        )

    def isJSONType(self):
        return self.isPrimitive() or self.isString() or self.isObject()

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
        if self.isUndefined():
            return not (other.isUndefined() or other.isDictionaryLike())
        if self.isPrimitive():
            if (
                other.isUndefined()
                or other.isString()
                or other.isEnum()
                or other.isInterface()
                or other.isObject()
                or other.isCallback()
                or other.isDictionary()
                or other.isSequence()
                or other.isRecord()
            ):
                return True
            if self.isBoolean():
                return other.isNumeric()
            assert self.isNumeric()
            return other.isBoolean()
        if self.isString():
            return (
                other.isUndefined()
                or other.isPrimitive()
                or other.isInterface()
                or other.isObject()
                or other.isCallback()
                or other.isDictionary()
                or other.isSequence()
                or other.isRecord()
            )
        if self.isAny():
            # Can't tell "any" apart from anything
            return False
        if self.isObject():
            return (
                other.isUndefined()
                or other.isPrimitive()
                or other.isString()
                or other.isEnum()
            )
        # Not much else we could be!
        assert self.isSpiderMonkeyInterface()
        # Like interfaces, but we know we're not a callback
        return (
            other.isUndefined()
            or other.isPrimitive()
            or other.isString()
            or other.isEnum()
            or other.isCallback()
            or other.isDictionary()
            or other.isSequence()
            or other.isRecord()
            or (
                other.isInterface()
                and (
                    # ArrayBuffer is distinguishable from everything
                    # that's not an ArrayBuffer or a callback interface
                    (self.isArrayBuffer() and not other.isArrayBuffer())
                    or
                    # ArrayBufferView is distinguishable from everything
                    # that's not an ArrayBufferView or typed array.
                    (
                        self.isArrayBufferView()
                        and not other.isArrayBufferView()
                        and not other.isTypedArray()
                    )
                    or
                    # Typed arrays are distinguishable from everything
                    # except ArrayBufferView and the same type of typed
                    # array
                    (
                        self.isTypedArray()
                        and not other.isArrayBufferView()
                        and not (other.isTypedArray() and other.name == self.name)
                    )
                )
            )
        )

    def _getDependentObjects(self):
        return set()

    def withExtendedAttributes(self, attrs):
        ret = self
        for attribute in attrs:
            identifier = attribute.identifier()
            if identifier == "Clamp":
                if not attribute.noArguments():
                    raise WebIDLError(
                        "[Clamp] must take no arguments", [attribute.location]
                    )
                if ret.hasEnforceRange() or self._enforceRange:
                    raise WebIDLError(
                        "[EnforceRange] and [Clamp] are mutually exclusive",
                        [self.location, attribute.location],
                    )
                ret = self.clamped([self.location, attribute.location])
            elif identifier == "EnforceRange":
                if not attribute.noArguments():
                    raise WebIDLError(
                        "[EnforceRange] must take no arguments", [attribute.location]
                    )
                if ret.hasClamp() or self._clamp:
                    raise WebIDLError(
                        "[EnforceRange] and [Clamp] are mutually exclusive",
                        [self.location, attribute.location],
                    )
                ret = self.rangeEnforced([self.location, attribute.location])
            elif identifier == "LegacyNullToEmptyString":
                if not (self.isDOMString() or self.isUTF8String()):
                    raise WebIDLError(
                        "[LegacyNullToEmptyString] only allowed on DOMStrings and UTF8Strings",
                        [self.location, attribute.location],
                    )
                assert not self.nullable()
                if attribute.hasValue():
                    raise WebIDLError(
                        "[LegacyNullToEmptyString] must take no identifier argument",
                        [attribute.location],
                    )
                ret = self.withLegacyNullToEmptyString(
                    [self.location, attribute.location]
                )
            elif identifier == "AllowShared":
                if not attribute.noArguments():
                    raise WebIDLError(
                        "[AllowShared] must take no arguments", [attribute.location]
                    )
                if not self.isBufferSource():
                    raise WebIDLError(
                        "[AllowShared] only allowed on buffer source types",
                        [self.location, attribute.location],
                    )
                ret = self.withAllowShared([self.location, attribute.location])

            else:
                raise WebIDLError(
                    "Unhandled extended attribute on type",
                    [self.location, attribute.location],
                )
        return ret


BuiltinTypes = {
    IDLBuiltinType.Types.byte: IDLBuiltinType(
        BuiltinLocation("<builtin type>"), "Byte", IDLBuiltinType.Types.byte
    ),
    IDLBuiltinType.Types.octet: IDLBuiltinType(
        BuiltinLocation("<builtin type>"), "Octet", IDLBuiltinType.Types.octet
    ),
    IDLBuiltinType.Types.short: IDLBuiltinType(
        BuiltinLocation("<builtin type>"), "Short", IDLBuiltinType.Types.short
    ),
    IDLBuiltinType.Types.unsigned_short: IDLBuiltinType(
        BuiltinLocation("<builtin type>"),
        "UnsignedShort",
        IDLBuiltinType.Types.unsigned_short,
    ),
    IDLBuiltinType.Types.long: IDLBuiltinType(
        BuiltinLocation("<builtin type>"), "Long", IDLBuiltinType.Types.long
    ),
    IDLBuiltinType.Types.unsigned_long: IDLBuiltinType(
        BuiltinLocation("<builtin type>"),
        "UnsignedLong",
        IDLBuiltinType.Types.unsigned_long,
    ),
    IDLBuiltinType.Types.long_long: IDLBuiltinType(
        BuiltinLocation("<builtin type>"), "LongLong", IDLBuiltinType.Types.long_long
    ),
    IDLBuiltinType.Types.unsigned_long_long: IDLBuiltinType(
        BuiltinLocation("<builtin type>"),
        "UnsignedLongLong",
        IDLBuiltinType.Types.unsigned_long_long,
    ),
    IDLBuiltinType.Types.undefined: IDLBuiltinType(
        BuiltinLocation("<builtin type>"), "Undefined", IDLBuiltinType.Types.undefined
    ),
    IDLBuiltinType.Types.boolean: IDLBuiltinType(
        BuiltinLocation("<builtin type>"), "Boolean", IDLBuiltinType.Types.boolean
    ),
    IDLBuiltinType.Types.float: IDLBuiltinType(
        BuiltinLocation("<builtin type>"), "Float", IDLBuiltinType.Types.float
    ),
    IDLBuiltinType.Types.unrestricted_float: IDLBuiltinType(
        BuiltinLocation("<builtin type>"),
        "UnrestrictedFloat",
        IDLBuiltinType.Types.unrestricted_float,
    ),
    IDLBuiltinType.Types.double: IDLBuiltinType(
        BuiltinLocation("<builtin type>"), "Double", IDLBuiltinType.Types.double
    ),
    IDLBuiltinType.Types.unrestricted_double: IDLBuiltinType(
        BuiltinLocation("<builtin type>"),
        "UnrestrictedDouble",
        IDLBuiltinType.Types.unrestricted_double,
    ),
    IDLBuiltinType.Types.any: IDLBuiltinType(
        BuiltinLocation("<builtin type>"), "Any", IDLBuiltinType.Types.any
    ),
    IDLBuiltinType.Types.domstring: IDLBuiltinType(
        BuiltinLocation("<builtin type>"), "String", IDLBuiltinType.Types.domstring
    ),
    IDLBuiltinType.Types.bytestring: IDLBuiltinType(
        BuiltinLocation("<builtin type>"), "ByteString", IDLBuiltinType.Types.bytestring
    ),
    IDLBuiltinType.Types.usvstring: IDLBuiltinType(
        BuiltinLocation("<builtin type>"), "USVString", IDLBuiltinType.Types.usvstring
    ),
    IDLBuiltinType.Types.utf8string: IDLBuiltinType(
        BuiltinLocation("<builtin type>"), "UTF8String", IDLBuiltinType.Types.utf8string
    ),
    IDLBuiltinType.Types.jsstring: IDLBuiltinType(
        BuiltinLocation("<builtin type>"), "JSString", IDLBuiltinType.Types.jsstring
    ),
    IDLBuiltinType.Types.object: IDLBuiltinType(
        BuiltinLocation("<builtin type>"), "Object", IDLBuiltinType.Types.object
    ),
    IDLBuiltinType.Types.ArrayBuffer: IDLBuiltinType(
        BuiltinLocation("<builtin type>"),
        "ArrayBuffer",
        IDLBuiltinType.Types.ArrayBuffer,
    ),
    IDLBuiltinType.Types.ArrayBufferView: IDLBuiltinType(
        BuiltinLocation("<builtin type>"),
        "ArrayBufferView",
        IDLBuiltinType.Types.ArrayBufferView,
    ),
    IDLBuiltinType.Types.Int8Array: IDLBuiltinType(
        BuiltinLocation("<builtin type>"), "Int8Array", IDLBuiltinType.Types.Int8Array
    ),
    IDLBuiltinType.Types.Uint8Array: IDLBuiltinType(
        BuiltinLocation("<builtin type>"), "Uint8Array", IDLBuiltinType.Types.Uint8Array
    ),
    IDLBuiltinType.Types.Uint8ClampedArray: IDLBuiltinType(
        BuiltinLocation("<builtin type>"),
        "Uint8ClampedArray",
        IDLBuiltinType.Types.Uint8ClampedArray,
    ),
    IDLBuiltinType.Types.Int16Array: IDLBuiltinType(
        BuiltinLocation("<builtin type>"), "Int16Array", IDLBuiltinType.Types.Int16Array
    ),
    IDLBuiltinType.Types.Uint16Array: IDLBuiltinType(
        BuiltinLocation("<builtin type>"),
        "Uint16Array",
        IDLBuiltinType.Types.Uint16Array,
    ),
    IDLBuiltinType.Types.Int32Array: IDLBuiltinType(
        BuiltinLocation("<builtin type>"), "Int32Array", IDLBuiltinType.Types.Int32Array
    ),
    IDLBuiltinType.Types.Uint32Array: IDLBuiltinType(
        BuiltinLocation("<builtin type>"),
        "Uint32Array",
        IDLBuiltinType.Types.Uint32Array,
    ),
    IDLBuiltinType.Types.Float32Array: IDLBuiltinType(
        BuiltinLocation("<builtin type>"),
        "Float32Array",
        IDLBuiltinType.Types.Float32Array,
    ),
    IDLBuiltinType.Types.Float64Array: IDLBuiltinType(
        BuiltinLocation("<builtin type>"),
        "Float64Array",
        IDLBuiltinType.Types.Float64Array,
    ),
}


integerTypeSizes = {
    IDLBuiltinType.Types.byte: (-128, 127),
    IDLBuiltinType.Types.octet: (0, 255),
    IDLBuiltinType.Types.short: (-32768, 32767),
    IDLBuiltinType.Types.unsigned_short: (0, 65535),
    IDLBuiltinType.Types.long: (-2147483648, 2147483647),
    IDLBuiltinType.Types.unsigned_long: (0, 4294967295),
    IDLBuiltinType.Types.long_long: (-9223372036854775808, 9223372036854775807),
    IDLBuiltinType.Types.unsigned_long_long: (0, 18446744073709551615),
}


def matchIntegerValueToType(value):
    for type, extremes in integerTypeSizes.items():
        (min, max) = extremes
        if value <= max and value >= min:
            return BuiltinTypes[type]

    return None


class NoCoercionFoundError(WebIDLError):
    """
    A class we use to indicate generic coercion failures because none of the
    types worked out in IDLValue.coerceToType.
    """


class IDLValue(IDLObject):
    __slots__ = (
        "type",
        "value",
    )

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
                except Exception as e:
                    # Make sure to propagate out WebIDLErrors that are not the
                    # generic "hey, we could not coerce to this type at all"
                    # exception, because those are specific "coercion failed for
                    # reason X" exceptions.  Note that we want to swallow
                    # non-WebIDLErrors here, because those can just happen if
                    # "type" is not something that can have a default value at
                    # all.
                    if isinstance(e, WebIDLError) and not isinstance(
                        e, NoCoercionFoundError
                    ):
                        raise e

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
                raise WebIDLError(
                    "Value %s is out of range for type %s." % (self.value, type),
                    [location],
                )
        elif self.type.isInteger() and type.isFloat():
            # Convert an integer literal into float
            if -(2**24) <= self.value <= 2**24:
                return IDLValue(self.location, type, float(self.value))
            else:
                raise WebIDLError(
                    "Converting value %s to %s will lose precision."
                    % (self.value, type),
                    [location],
                )
        elif self.type.isString() and type.isEnum():
            # Just keep our string, but make sure it's a valid value for this enum
            enum = type.unroll().inner
            if self.value not in enum.values():
                raise WebIDLError(
                    "'%s' is not a valid default value for enum %s"
                    % (self.value, enum.identifier.name),
                    [location, enum.location],
                )
            return self
        elif self.type.isFloat() and type.isFloat():
            if not type.isUnrestricted() and (
                self.value == float("inf")
                or self.value == float("-inf")
                or math.isnan(self.value)
            ):
                raise WebIDLError(
                    "Trying to convert unrestricted value %s to non-unrestricted"
                    % self.value,
                    [location],
                )
            return IDLValue(self.location, type, self.value)
        elif self.type.isString() and type.isUSVString():
            # Allow USVStrings to use default value just like
            # DOMString.  No coercion is required in this case as Codegen.py
            # treats USVString just like DOMString, but with an
            # extra normalization step.
            assert self.type.isDOMString()
            return self
        elif self.type.isString() and (
            type.isByteString() or type.isJSString() or type.isUTF8String()
        ):
            # Allow ByteStrings, UTF8String, and JSStrings to use a default
            # value like DOMString.
            # No coercion is required as Codegen.py will handle the
            # extra steps. We want to make sure that our string contains
            # only valid characters, so we check that here.
            valid_ascii_lit = (
                " " + string.ascii_letters + string.digits + string.punctuation
            )
            for idx, c in enumerate(self.value):
                if c not in valid_ascii_lit:
                    raise WebIDLError(
                        "Coercing this string literal %s to a ByteString is not supported yet. "
                        "Coercion failed due to an unsupported byte %d at index %d."
                        % (self.value.__repr__(), ord(c), idx),
                        [location],
                    )

            return IDLValue(self.location, type, self.value)
        elif self.type.isDOMString() and type.legacyNullToEmptyString:
            # LegacyNullToEmptyString is a different type for resolution reasons,
            # however once you have a value it doesn't matter
            return self

        raise NoCoercionFoundError(
            "Cannot coerce type %s to type %s." % (self.type, type), [location]
        )

    def _getDependentObjects(self):
        return set()


class IDLNullValue(IDLObject):
    __slots__ = "type", "value"

    def __init__(self, location):
        IDLObject.__init__(self, location)
        self.type = None
        self.value = None

    def coerceToType(self, type, location):
        if (
            not isinstance(type, IDLNullableType)
            and not (type.isUnion() and type.hasNullableType)
            and not type.isAny()
        ):
            raise WebIDLError("Cannot coerce null value to type %s." % type, [location])

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
    __slots__ = "type", "value"

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
                except Exception:
                    pass

        if not type.isSequence():
            raise WebIDLError(
                "Cannot coerce empty sequence value to type %s." % type, [location]
            )

        emptySequenceValue = IDLEmptySequenceValue(self.location)
        emptySequenceValue.type = type
        return emptySequenceValue

    def _getDependentObjects(self):
        return set()


class IDLDefaultDictionaryValue(IDLObject):
    __slots__ = "type", "value"

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
                except Exception:
                    pass

        if not type.isDictionary():
            raise WebIDLError(
                "Cannot coerce default dictionary value to type %s." % type, [location]
            )

        defaultDictionaryValue = IDLDefaultDictionaryValue(self.location)
        defaultDictionaryValue.type = type
        return defaultDictionaryValue

    def _getDependentObjects(self):
        return set()


class IDLUndefinedValue(IDLObject):
    __slots__ = "type", "value"

    def __init__(self, location):
        IDLObject.__init__(self, location)
        self.type = None
        self.value = None

    def coerceToType(self, type, location):
        if not type.isAny():
            raise WebIDLError(
                "Cannot coerce undefined value to type %s." % type, [location]
            )

        undefinedValue = IDLUndefinedValue(self.location)
        undefinedValue.type = type
        return undefinedValue

    def _getDependentObjects(self):
        return set()


class IDLInterfaceMember(IDLObjectWithIdentifier, IDLExposureMixins):
    Tags = enum(
        "Const", "Attr", "Method", "MaplikeOrSetlike", "AsyncIterable", "Iterable"
    )

    Special = enum("Static", "Stringifier")

    AffectsValues = ("Nothing", "Everything")
    DependsOnValues = ("Nothing", "DOMState", "DeviceState", "Everything")

    # no slots : multiple inheritance
    def __init__(self, location, identifier, tag, extendedAttrDict=None):
        IDLObjectWithIdentifier.__init__(self, location, None, identifier)
        IDLExposureMixins.__init__(self, location)
        self.tag = tag
        if extendedAttrDict is None:
            self._extendedAttrDict = {}
        else:
            self._extendedAttrDict = extendedAttrDict

    def isMethod(self):
        return self.tag == IDLInterfaceMember.Tags.Method

    def isAttr(self):
        return self.tag == IDLInterfaceMember.Tags.Attr

    def isConst(self):
        return self.tag == IDLInterfaceMember.Tags.Const

    def isMaplikeOrSetlikeOrIterable(self):
        return (
            self.tag == IDLInterfaceMember.Tags.MaplikeOrSetlike
            or self.tag == IDLInterfaceMember.Tags.AsyncIterable
            or self.tag == IDLInterfaceMember.Tags.Iterable
        )

    def isMaplikeOrSetlike(self):
        return self.tag == IDLInterfaceMember.Tags.MaplikeOrSetlike

    def addExtendedAttributes(self, attrs):
        for attr in attrs:
            self.handleExtendedAttribute(attr)
            attrlist = attr.listValue()
            self._extendedAttrDict[attr.identifier()] = (
                attrlist if len(attrlist) else True
            )

    def handleExtendedAttribute(self, attr):
        pass

    def getExtendedAttribute(self, name):
        return self._extendedAttrDict.get(name, None)

    def finish(self, scope):
        IDLExposureMixins.finish(self, scope)

    def validate(self):
        if self.isAttr() or self.isMethod():
            if self.affects == "Everything" and self.dependsOn != "Everything":
                raise WebIDLError(
                    "Interface member is flagged as affecting "
                    "everything but not depending on everything. "
                    "That seems rather unlikely.",
                    [self.location],
                )

        if self.getExtendedAttribute("NewObject"):
            if self.dependsOn == "Nothing" or self.dependsOn == "DOMState":
                raise WebIDLError(
                    "A [NewObject] method is not idempotent, "
                    "so it has to depend on something other than DOM state.",
                    [self.location],
                )
            if self.getExtendedAttribute("Cached") or self.getExtendedAttribute(
                "StoreInSlot"
            ):
                raise WebIDLError(
                    "A [NewObject] attribute shouldnt be "
                    "[Cached] or [StoreInSlot], since the point "
                    "of those is to keep returning the same "
                    "thing across multiple calls, which is not "
                    "what [NewObject] does.",
                    [self.location],
                )

    def _setDependsOn(self, dependsOn):
        if self.dependsOn != "Everything":
            raise WebIDLError(
                "Trying to specify multiple different DependsOn, "
                "Pure, or Constant extended attributes for "
                "attribute",
                [self.location],
            )
        if dependsOn not in IDLInterfaceMember.DependsOnValues:
            raise WebIDLError(
                "Invalid [DependsOn=%s] on attribute" % dependsOn, [self.location]
            )
        self.dependsOn = dependsOn

    def _setAffects(self, affects):
        if self.affects != "Everything":
            raise WebIDLError(
                "Trying to specify multiple different Affects, "
                "Pure, or Constant extended attributes for "
                "attribute",
                [self.location],
            )
        if affects not in IDLInterfaceMember.AffectsValues:
            raise WebIDLError(
                "Invalid [Affects=%s] on attribute" % affects, [self.location]
            )
        self.affects = affects

    def _addAlias(self, alias):
        if alias in self.aliases:
            raise WebIDLError(
                "Duplicate [Alias=%s] on attribute" % alias, [self.location]
            )
        self.aliases.append(alias)

    def _addBindingAlias(self, bindingAlias):
        if bindingAlias in self.bindingAliases:
            raise WebIDLError(
                "Duplicate [BindingAlias=%s] on attribute" % bindingAlias,
                [self.location],
            )
        self.bindingAliases.append(bindingAlias)


class IDLMaplikeOrSetlikeOrIterableBase(IDLInterfaceMember):
    __slots__ = (
        "keyType",
        "valueType",
        "maplikeOrSetlikeOrIterableType",
        "disallowedMemberNames",
        "disallowedNonMethodNames",
    )

    def __init__(self, location, identifier, ifaceType, keyType, valueType, ifaceKind):
        IDLInterfaceMember.__init__(self, location, identifier, ifaceKind)
        if keyType is not None:
            assert isinstance(keyType, IDLType)
        else:
            assert valueType is not None
        assert ifaceType in ["maplike", "setlike", "iterable", "asynciterable"]
        if valueType is not None:
            assert isinstance(valueType, IDLType)
        self.keyType = keyType
        self.valueType = valueType
        self.maplikeOrSetlikeOrIterableType = ifaceType
        self.disallowedMemberNames = []
        self.disallowedNonMethodNames = []

    def isMaplike(self):
        return self.maplikeOrSetlikeOrIterableType == "maplike"

    def isSetlike(self):
        return self.maplikeOrSetlikeOrIterableType == "setlike"

    def isIterable(self):
        return self.maplikeOrSetlikeOrIterableType == "iterable"

    def isAsyncIterable(self):
        return self.maplikeOrSetlikeOrIterableType == "asynciterable"

    def hasKeyType(self):
        return self.keyType is not None

    def hasValueType(self):
        return self.valueType is not None

    def checkCollisions(self, members, isAncestor):
        for member in members:
            # Check that there are no disallowed members
            if member.identifier.name in self.disallowedMemberNames and not (
                (
                    member.isMethod()
                    and (
                        member.isStatic() or member.isMaplikeOrSetlikeOrIterableMethod()
                    )
                )
                or (member.isAttr() and member.isMaplikeOrSetlikeAttr())
            ):
                raise WebIDLError(
                    "Member '%s' conflicts "
                    "with reserved %s name."
                    % (member.identifier.name, self.maplikeOrSetlikeOrIterableType),
                    [self.location, member.location],
                )
            # Check that there are no disallowed non-method members.
            # Ancestor members are always disallowed here; own members
            # are disallowed only if they're non-methods.
            if (
                isAncestor or member.isAttr() or member.isConst()
            ) and member.identifier.name in self.disallowedNonMethodNames:
                raise WebIDLError(
                    "Member '%s' conflicts "
                    "with reserved %s method."
                    % (member.identifier.name, self.maplikeOrSetlikeOrIterableType),
                    [self.location, member.location],
                )

    def addMethod(
        self,
        name,
        members,
        allowExistingOperations,
        returnType,
        args=[],
        chromeOnly=False,
        isPure=False,
        affectsNothing=False,
        newObject=False,
        isIteratorAlias=False,
    ):
        """
        Create an IDLMethod based on the parameters passed in.

        - members is the member list to add this function to, since this is
          called during the member expansion portion of interface object
          building.

        - chromeOnly is only True for read-only js implemented classes, to
        implement underscore prefixed convenience functions which would
        otherwise not be available, unlike the case of C++ bindings.

        - isPure is only True for idempotent functions, so it is not valid for
        things like keys, values, etc. that return a new object every time.

        - affectsNothing means that nothing changes due to this method, which
          affects JIT optimization behavior

        - newObject means the method creates and returns a new object.

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
        # maplike/setlike operation.
        if allowExistingOperations:
            for m in members:
                if m.identifier.name == name and m.isMethod() and not m.isStatic():
                    return
        method = IDLMethod(
            self.location,
            IDLUnresolvedIdentifier(
                self.location, name, allowDoubleUnderscore=chromeOnly
            ),
            returnType,
            args,
            maplikeOrSetlikeOrIterable=self,
        )
        # We need to be able to throw from declaration methods
        method.addExtendedAttributes([IDLExtendedAttribute(self.location, ("Throws",))])
        if chromeOnly:
            method.addExtendedAttributes(
                [IDLExtendedAttribute(self.location, ("ChromeOnly",))]
            )
        if isPure:
            method.addExtendedAttributes(
                [IDLExtendedAttribute(self.location, ("Pure",))]
            )
        # Following attributes are used for keys/values/entries. Can't mark
        # them pure, since they return a new object each time they are run.
        if affectsNothing:
            method.addExtendedAttributes(
                [
                    IDLExtendedAttribute(self.location, ("DependsOn", "Everything")),
                    IDLExtendedAttribute(self.location, ("Affects", "Nothing")),
                ]
            )
        if newObject:
            method.addExtendedAttributes(
                [IDLExtendedAttribute(self.location, ("NewObject",))]
            )
        if isIteratorAlias:
            if not self.isAsyncIterable():
                method.addExtendedAttributes(
                    [IDLExtendedAttribute(self.location, ("Alias", "@@iterator"))]
                )
            else:
                method.addExtendedAttributes(
                    [IDLExtendedAttribute(self.location, ("Alias", "@@asyncIterator"))]
                )
        members.append(method)

    def resolve(self, parentScope):
        if self.keyType:
            self.keyType.resolveType(parentScope)
        if self.valueType:
            self.valueType.resolveType(parentScope)

    def finish(self, scope):
        IDLInterfaceMember.finish(self, scope)
        if self.keyType and not self.keyType.isComplete():
            t = self.keyType.complete(scope)

            assert not isinstance(t, IDLUnresolvedType)
            assert not isinstance(t, IDLTypedefType)
            assert not isinstance(t.name, IDLUnresolvedIdentifier)
            self.keyType = t
        if self.valueType and not self.valueType.isComplete():
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
        deps = set()
        if self.keyType:
            deps.add(self.keyType)
        if self.valueType:
            deps.add(self.valueType)
        return deps

    def getForEachArguments(self):
        return [
            IDLArgument(
                self.location,
                IDLUnresolvedIdentifier(
                    BuiltinLocation("<auto-generated-identifier>"), "callback"
                ),
                BuiltinTypes[IDLBuiltinType.Types.object],
            ),
            IDLArgument(
                self.location,
                IDLUnresolvedIdentifier(
                    BuiltinLocation("<auto-generated-identifier>"), "thisArg"
                ),
                BuiltinTypes[IDLBuiltinType.Types.any],
                optional=True,
            ),
        ]


# Iterable adds ES6 iterator style functions and traits
# (keys/values/entries/@@iterator) to an interface.
class IDLIterable(IDLMaplikeOrSetlikeOrIterableBase):
    __slots__ = ("iteratorType",)

    def __init__(self, location, identifier, keyType, valueType, scope):
        IDLMaplikeOrSetlikeOrIterableBase.__init__(
            self,
            location,
            identifier,
            "iterable",
            keyType,
            valueType,
            IDLInterfaceMember.Tags.Iterable,
        )
        self.iteratorType = None

    def __str__(self):
        return "declared iterable with key '%s' and value '%s'" % (
            self.keyType,
            self.valueType,
        )

    def expand(self, members):
        """
        In order to take advantage of all of the method machinery in Codegen,
        we generate our functions as if they were part of the interface
        specification during parsing.
        """
        # We only need to add entries/keys/values here if we're a pair iterator.
        # Value iterators just copy these from %ArrayPrototype% instead.
        if not self.isPairIterator():
            return

        # object entries()
        self.addMethod(
            "entries",
            members,
            False,
            self.iteratorType,
            affectsNothing=True,
            newObject=True,
            isIteratorAlias=True,
        )
        # object keys()
        self.addMethod(
            "keys",
            members,
            False,
            self.iteratorType,
            affectsNothing=True,
            newObject=True,
        )
        # object values()
        self.addMethod(
            "values",
            members,
            False,
            self.iteratorType,
            affectsNothing=True,
            newObject=True,
        )

        # undefined forEach(callback(valueType, keyType), optional any thisArg)
        self.addMethod(
            "forEach",
            members,
            False,
            BuiltinTypes[IDLBuiltinType.Types.undefined],
            self.getForEachArguments(),
        )

    def isValueIterator(self):
        return not self.isPairIterator()

    def isPairIterator(self):
        return self.hasKeyType()


class IDLAsyncIterable(IDLMaplikeOrSetlikeOrIterableBase):
    __slots__ = "iteratorType", "argList"

    def __init__(self, location, identifier, keyType, valueType, argList, scope):
        for arg in argList:
            if not arg.optional:
                raise WebIDLError(
                    "The arguments of the asynchronously iterable declaration on "
                    "%s must all be optional arguments." % identifier,
                    [arg.location],
                )

        IDLMaplikeOrSetlikeOrIterableBase.__init__(
            self,
            location,
            identifier,
            "asynciterable",
            keyType,
            valueType,
            IDLInterfaceMember.Tags.AsyncIterable,
        )
        self.iteratorType = None
        self.argList = argList

    def __str__(self):
        return "declared async iterable with key '%s' and value '%s'" % (
            self.keyType,
            self.valueType,
        )

    def expand(self, members):
        """
        In order to take advantage of all of the method machinery in Codegen,
        we generate our functions as if they were part of the interface
        specification during parsing.
        """
        # object values()
        self.addMethod(
            "values",
            members,
            False,
            self.iteratorType,
            self.argList,
            affectsNothing=True,
            newObject=True,
            isIteratorAlias=(not self.isPairIterator()),
        )

        # We only need to add entries/keys here if we're a pair iterator.
        if not self.isPairIterator():
            return

        # Methods can't share their IDLArguments, so we need to make copies here.
        def copyArgList(argList):
            return map(copy.copy, argList)

        # object entries()
        self.addMethod(
            "entries",
            members,
            False,
            self.iteratorType,
            copyArgList(self.argList),
            affectsNothing=True,
            newObject=True,
            isIteratorAlias=True,
        )
        # object keys()
        self.addMethod(
            "keys",
            members,
            False,
            self.iteratorType,
            copyArgList(self.argList),
            affectsNothing=True,
            newObject=True,
        )

    def isValueIterator(self):
        return not self.isPairIterator()

    def isPairIterator(self):
        return self.hasKeyType()


# MaplikeOrSetlike adds ES6 map-or-set-like traits to an interface.
class IDLMaplikeOrSetlike(IDLMaplikeOrSetlikeOrIterableBase):
    __slots__ = "readonly", "slotIndices", "prefix"

    def __init__(
        self, location, identifier, maplikeOrSetlikeType, readonly, keyType, valueType
    ):
        IDLMaplikeOrSetlikeOrIterableBase.__init__(
            self,
            location,
            identifier,
            maplikeOrSetlikeType,
            keyType,
            valueType,
            IDLInterfaceMember.Tags.MaplikeOrSetlike,
        )
        self.readonly = readonly
        self.slotIndices = None

        # When generating JSAPI access code, we need to know the backing object
        # type prefix to create the correct function. Generate here for reuse.
        if self.isMaplike():
            self.prefix = "Map"
        elif self.isSetlike():
            self.prefix = "Set"

    def __str__(self):
        return "declared '%s' with key '%s'" % (
            self.maplikeOrSetlikeOrIterableType,
            self.keyType,
        )

    def expand(self, members):
        """
        In order to take advantage of all of the method machinery in Codegen,
        we generate our functions as if they were part of the interface
        specification during parsing.
        """
        # Both maplike and setlike have a size attribute
        members.append(
            IDLAttribute(
                self.location,
                IDLUnresolvedIdentifier(
                    BuiltinLocation("<auto-generated-identifier>"), "size"
                ),
                BuiltinTypes[IDLBuiltinType.Types.unsigned_long],
                True,
                maplikeOrSetlike=self,
            )
        )
        self.reserved_ro_names = ["size"]
        self.disallowedMemberNames.append("size")

        # object entries()
        self.addMethod(
            "entries",
            members,
            False,
            BuiltinTypes[IDLBuiltinType.Types.object],
            affectsNothing=True,
            isIteratorAlias=self.isMaplike(),
        )
        # object keys()
        self.addMethod(
            "keys",
            members,
            False,
            BuiltinTypes[IDLBuiltinType.Types.object],
            affectsNothing=True,
        )
        # object values()
        self.addMethod(
            "values",
            members,
            False,
            BuiltinTypes[IDLBuiltinType.Types.object],
            affectsNothing=True,
            isIteratorAlias=self.isSetlike(),
        )

        # undefined forEach(callback(valueType, keyType), thisVal)
        self.addMethod(
            "forEach",
            members,
            False,
            BuiltinTypes[IDLBuiltinType.Types.undefined],
            self.getForEachArguments(),
        )

        def getKeyArg():
            return IDLArgument(
                self.location,
                IDLUnresolvedIdentifier(self.location, "key"),
                self.keyType,
            )

        # boolean has(keyType key)
        self.addMethod(
            "has",
            members,
            False,
            BuiltinTypes[IDLBuiltinType.Types.boolean],
            [getKeyArg()],
            isPure=True,
        )

        if not self.readonly:
            # undefined clear()
            self.addMethod(
                "clear", members, True, BuiltinTypes[IDLBuiltinType.Types.undefined], []
            )
            # boolean delete(keyType key)
            self.addMethod(
                "delete",
                members,
                True,
                BuiltinTypes[IDLBuiltinType.Types.boolean],
                [getKeyArg()],
            )

        if self.isSetlike():
            if not self.readonly:
                # Add returns the set object it just added to.
                # object add(keyType key)

                self.addMethod(
                    "add",
                    members,
                    True,
                    BuiltinTypes[IDLBuiltinType.Types.object],
                    [getKeyArg()],
                )
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
        self.addMethod(
            "get",
            members,
            False,
            BuiltinTypes[IDLBuiltinType.Types.any],
            [getKeyArg()],
            isPure=True,
        )

        def getValueArg():
            return IDLArgument(
                self.location,
                IDLUnresolvedIdentifier(self.location, "value"),
                self.valueType,
            )

        if not self.readonly:
            self.addMethod(
                "set",
                members,
                True,
                BuiltinTypes[IDLBuiltinType.Types.object],
                [getKeyArg(), getValueArg()],
            )


class IDLConst(IDLInterfaceMember):
    __slots__ = "type", "value"

    def __init__(self, location, identifier, type, value):
        IDLInterfaceMember.__init__(
            self, location, identifier, IDLInterfaceMember.Tags.Const
        )

        assert isinstance(type, IDLType)
        if type.isDictionary():
            raise WebIDLError(
                "A constant cannot be of a dictionary type", [self.location]
            )
        if type.isRecord():
            raise WebIDLError("A constant cannot be of a record type", [self.location])
        self.type = type
        self.value = value

        if identifier.name == "prototype":
            raise WebIDLError(
                "The identifier of a constant must not be 'prototype'", [location]
            )

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
                except Exception:
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
        elif (
            identifier == "Pref"
            or identifier == "ChromeOnly"
            or identifier == "Func"
            or identifier == "Trial"
            or identifier == "SecureContext"
            or identifier == "NonEnumerable"
        ):
            # Known attributes that we don't need to do anything with here
            pass
        else:
            raise WebIDLError(
                "Unknown extended attribute %s on constant" % identifier,
                [attr.location],
            )
        IDLInterfaceMember.handleExtendedAttribute(self, attr)

    def _getDependentObjects(self):
        return set([self.type, self.value])


class IDLAttribute(IDLInterfaceMember):
    __slots__ = (
        "type",
        "readonly",
        "inherit",
        "_static",
        "legacyLenientThis",
        "_legacyUnforgeable",
        "stringifier",
        "slotIndices",
        "maplikeOrSetlike",
        "dependsOn",
        "affects",
        "bindingAliases",
    )

    def __init__(
        self,
        location,
        identifier,
        type,
        readonly,
        inherit=False,
        static=False,
        stringifier=False,
        maplikeOrSetlike=None,
        extendedAttrDict=None,
    ):
        IDLInterfaceMember.__init__(
            self,
            location,
            identifier,
            IDLInterfaceMember.Tags.Attr,
            extendedAttrDict=extendedAttrDict,
        )

        assert isinstance(type, IDLType)
        self.type = type
        self.readonly = readonly
        self.inherit = inherit
        self._static = static
        self.legacyLenientThis = False
        self._legacyUnforgeable = False
        self.stringifier = stringifier
        self.slotIndices = None
        assert maplikeOrSetlike is None or isinstance(
            maplikeOrSetlike, IDLMaplikeOrSetlike
        )
        self.maplikeOrSetlike = maplikeOrSetlike
        self.dependsOn = "Everything"
        self.affects = "Everything"
        self.bindingAliases = []

        if static and identifier.name == "prototype":
            raise WebIDLError(
                "The identifier of a static attribute must not be 'prototype'",
                [location],
            )

        if readonly and inherit:
            raise WebIDLError(
                "An attribute cannot be both 'readonly' and 'inherit'", [self.location]
            )

    def isStatic(self):
        return self._static

    def forceStatic(self):
        self._static = True

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

        if self.readonly and (
            self.type.hasClamp()
            or self.type.hasEnforceRange()
            or self.type.hasAllowShared()
            or self.type.legacyNullToEmptyString
        ):
            raise WebIDLError(
                "A readonly attribute cannot be [Clamp] or [EnforceRange] or [AllowShared]",
                [self.location],
            )
        if self.type.isDictionary() and not self.getExtendedAttribute("Cached"):
            raise WebIDLError(
                "An attribute cannot be of a dictionary type", [self.location]
            )
        if self.type.isSequence() and not self.getExtendedAttribute("Cached"):
            raise WebIDLError(
                "A non-cached attribute cannot be of a sequence " "type",
                [self.location],
            )
        if self.type.isRecord() and not self.getExtendedAttribute("Cached"):
            raise WebIDLError(
                "A non-cached attribute cannot be of a record " "type", [self.location]
            )
        if self.type.isUnion():
            for f in self.type.unroll().flatMemberTypes:
                if f.isDictionary():
                    raise WebIDLError(
                        "An attribute cannot be of a union "
                        "type if one of its member types (or "
                        "one of its member types's member "
                        "types, and so on) is a dictionary "
                        "type",
                        [self.location, f.location],
                    )
                if f.isSequence():
                    raise WebIDLError(
                        "An attribute cannot be of a union "
                        "type if one of its member types (or "
                        "one of its member types's member "
                        "types, and so on) is a sequence "
                        "type",
                        [self.location, f.location],
                    )
                if f.isRecord():
                    raise WebIDLError(
                        "An attribute cannot be of a union "
                        "type if one of its member types (or "
                        "one of its member types's member "
                        "types, and so on) is a record "
                        "type",
                        [self.location, f.location],
                    )
        if not self.type.isInterface() and self.getExtendedAttribute("PutForwards"):
            raise WebIDLError(
                "An attribute with [PutForwards] must have an "
                "interface type as its type",
                [self.location],
            )

        if not self.type.isInterface() and self.getExtendedAttribute("SameObject"):
            raise WebIDLError(
                "An attribute with [SameObject] must have an "
                "interface type as its type",
                [self.location],
            )

        if self.type.isPromise() and not self.readonly:
            raise WebIDLError(
                "Promise-returning attributes must be readonly", [self.location]
            )

        if self.type.isObservableArray():
            if self.isStatic():
                raise WebIDLError(
                    "A static attribute cannot have an ObservableArray type",
                    [self.location],
                )
            if self.getExtendedAttribute("Cached") or self.getExtendedAttribute(
                "StoreInSlot"
            ):
                raise WebIDLError(
                    "[Cached] and [StoreInSlot] must not be used "
                    "on an attribute whose type is ObservableArray",
                    [self.location],
                )

    def validate(self):
        def typeContainsChromeOnlyDictionaryMember(type):
            if type.nullable() or type.isSequence() or type.isRecord():
                return typeContainsChromeOnlyDictionaryMember(type.inner)

            if type.isUnion():
                for memberType in type.flatMemberTypes:
                    (contains, location) = typeContainsChromeOnlyDictionaryMember(
                        memberType
                    )
                    if contains:
                        return (True, location)

            if type.isDictionary():
                dictionary = type.inner
                while dictionary:
                    (contains, location) = dictionaryContainsChromeOnlyMember(
                        dictionary
                    )
                    if contains:
                        return (True, location)
                    dictionary = dictionary.parent

            return (False, None)

        def dictionaryContainsChromeOnlyMember(dictionary):
            for member in dictionary.members:
                if member.getExtendedAttribute("ChromeOnly"):
                    return (True, member.location)
                (contains, location) = typeContainsChromeOnlyDictionaryMember(
                    member.type
                )
                if contains:
                    return (True, location)
            return (False, None)

        IDLInterfaceMember.validate(self)

        if self.getExtendedAttribute("Cached") or self.getExtendedAttribute(
            "StoreInSlot"
        ):
            if not self.affects == "Nothing":
                raise WebIDLError(
                    "Cached attributes and attributes stored in "
                    "slots must be Constant or Pure or "
                    "Affects=Nothing, since the getter won't always "
                    "be called.",
                    [self.location],
                )
            (contains, location) = typeContainsChromeOnlyDictionaryMember(self.type)
            if contains:
                raise WebIDLError(
                    "[Cached] and [StoreInSlot] must not be used "
                    "on an attribute whose type contains a "
                    "[ChromeOnly] dictionary member",
                    [self.location, location],
                )
        if self.getExtendedAttribute("Frozen"):
            if (
                not self.type.isSequence()
                and not self.type.isDictionary()
                and not self.type.isRecord()
            ):
                raise WebIDLError(
                    "[Frozen] is only allowed on "
                    "sequence-valued, dictionary-valued, and "
                    "record-valued attributes",
                    [self.location],
                )
        if not self.type.unroll().isExposedInAllOf(self.exposureSet):
            raise WebIDLError(
                "Attribute returns a type that is not exposed "
                "everywhere where the attribute is exposed",
                [self.location],
            )
        if self.getExtendedAttribute("CEReactions"):
            if self.readonly:
                raise WebIDLError(
                    "[CEReactions] is not allowed on " "readonly attributes",
                    [self.location],
                )

    def handleExtendedAttribute(self, attr):
        identifier = attr.identifier()
        if (
            identifier == "SetterThrows"
            or identifier == "SetterCanOOM"
            or identifier == "SetterNeedsSubjectPrincipal"
        ) and self.readonly:
            raise WebIDLError(
                "Readonly attributes must not be flagged as " "[%s]" % identifier,
                [self.location],
            )
        elif identifier == "BindingAlias":
            if not attr.hasValue():
                raise WebIDLError(
                    "[BindingAlias] takes an identifier or string", [attr.location]
                )
            self._addBindingAlias(attr.value())
        elif (
            (
                identifier == "Throws"
                or identifier == "GetterThrows"
                or identifier == "CanOOM"
                or identifier == "GetterCanOOM"
            )
            and self.getExtendedAttribute("StoreInSlot")
        ) or (
            identifier == "StoreInSlot"
            and (
                self.getExtendedAttribute("Throws")
                or self.getExtendedAttribute("GetterThrows")
                or self.getExtendedAttribute("CanOOM")
                or self.getExtendedAttribute("GetterCanOOM")
            )
        ):
            raise WebIDLError("Throwing things can't be [StoreInSlot]", [attr.location])
        elif identifier == "LegacyLenientThis":
            if not attr.noArguments():
                raise WebIDLError(
                    "[LegacyLenientThis] must take no arguments", [attr.location]
                )
            if self.isStatic():
                raise WebIDLError(
                    "[LegacyLenientThis] is only allowed on non-static " "attributes",
                    [attr.location, self.location],
                )
            if self.getExtendedAttribute("CrossOriginReadable"):
                raise WebIDLError(
                    "[LegacyLenientThis] is not allowed in combination "
                    "with [CrossOriginReadable]",
                    [attr.location, self.location],
                )
            if self.getExtendedAttribute("CrossOriginWritable"):
                raise WebIDLError(
                    "[LegacyLenientThis] is not allowed in combination "
                    "with [CrossOriginWritable]",
                    [attr.location, self.location],
                )
            self.legacyLenientThis = True
        elif identifier == "LegacyUnforgeable":
            if self.isStatic():
                raise WebIDLError(
                    "[LegacyUnforgeable] is only allowed on non-static " "attributes",
                    [attr.location, self.location],
                )
            self._legacyUnforgeable = True
        elif identifier == "SameObject" and not self.readonly:
            raise WebIDLError(
                "[SameObject] only allowed on readonly attributes",
                [attr.location, self.location],
            )
        elif identifier == "Constant" and not self.readonly:
            raise WebIDLError(
                "[Constant] only allowed on readonly attributes",
                [attr.location, self.location],
            )
        elif identifier == "PutForwards":
            if not self.readonly:
                raise WebIDLError(
                    "[PutForwards] is only allowed on readonly " "attributes",
                    [attr.location, self.location],
                )
            if self.type.isPromise():
                raise WebIDLError(
                    "[PutForwards] is not allowed on " "Promise-typed attributes",
                    [attr.location, self.location],
                )
            if self.isStatic():
                raise WebIDLError(
                    "[PutForwards] is only allowed on non-static " "attributes",
                    [attr.location, self.location],
                )
            if self.getExtendedAttribute("Replaceable") is not None:
                raise WebIDLError(
                    "[PutForwards] and [Replaceable] can't both "
                    "appear on the same attribute",
                    [attr.location, self.location],
                )
            if not attr.hasValue():
                raise WebIDLError(
                    "[PutForwards] takes an identifier", [attr.location, self.location]
                )
        elif identifier == "Replaceable":
            if not attr.noArguments():
                raise WebIDLError(
                    "[Replaceable] must take no arguments", [attr.location]
                )
            if not self.readonly:
                raise WebIDLError(
                    "[Replaceable] is only allowed on readonly " "attributes",
                    [attr.location, self.location],
                )
            if self.type.isPromise():
                raise WebIDLError(
                    "[Replaceable] is not allowed on " "Promise-typed attributes",
                    [attr.location, self.location],
                )
            if self.isStatic():
                raise WebIDLError(
                    "[Replaceable] is only allowed on non-static " "attributes",
                    [attr.location, self.location],
                )
            if self.getExtendedAttribute("PutForwards") is not None:
                raise WebIDLError(
                    "[PutForwards] and [Replaceable] can't both "
                    "appear on the same attribute",
                    [attr.location, self.location],
                )
        elif identifier == "LegacyLenientSetter":
            if not attr.noArguments():
                raise WebIDLError(
                    "[LegacyLenientSetter] must take no arguments", [attr.location]
                )
            if not self.readonly:
                raise WebIDLError(
                    "[LegacyLenientSetter] is only allowed on readonly " "attributes",
                    [attr.location, self.location],
                )
            if self.type.isPromise():
                raise WebIDLError(
                    "[LegacyLenientSetter] is not allowed on "
                    "Promise-typed attributes",
                    [attr.location, self.location],
                )
            if self.isStatic():
                raise WebIDLError(
                    "[LegacyLenientSetter] is only allowed on non-static " "attributes",
                    [attr.location, self.location],
                )
            if self.getExtendedAttribute("PutForwards") is not None:
                raise WebIDLError(
                    "[LegacyLenientSetter] and [PutForwards] can't both "
                    "appear on the same attribute",
                    [attr.location, self.location],
                )
            if self.getExtendedAttribute("Replaceable") is not None:
                raise WebIDLError(
                    "[LegacyLenientSetter] and [Replaceable] can't both "
                    "appear on the same attribute",
                    [attr.location, self.location],
                )
        elif identifier == "LenientFloat":
            if self.readonly:
                raise WebIDLError(
                    "[LenientFloat] used on a readonly attribute",
                    [attr.location, self.location],
                )
            if not self.type.includesRestrictedFloat():
                raise WebIDLError(
                    "[LenientFloat] used on an attribute with a "
                    "non-restricted-float type",
                    [attr.location, self.location],
                )
        elif identifier == "StoreInSlot":
            if self.getExtendedAttribute("Cached"):
                raise WebIDLError(
                    "[StoreInSlot] and [Cached] must not be "
                    "specified on the same attribute",
                    [attr.location, self.location],
                )
        elif identifier == "Cached":
            if self.getExtendedAttribute("StoreInSlot"):
                raise WebIDLError(
                    "[Cached] and [StoreInSlot] must not be "
                    "specified on the same attribute",
                    [attr.location, self.location],
                )
        elif identifier == "CrossOriginReadable" or identifier == "CrossOriginWritable":
            if not attr.noArguments():
                raise WebIDLError(
                    "[%s] must take no arguments" % identifier, [attr.location]
                )
            if self.isStatic():
                raise WebIDLError(
                    "[%s] is only allowed on non-static " "attributes" % identifier,
                    [attr.location, self.location],
                )
            if self.getExtendedAttribute("LegacyLenientThis"):
                raise WebIDLError(
                    "[LegacyLenientThis] is not allowed in combination "
                    "with [%s]" % identifier,
                    [attr.location, self.location],
                )
        elif identifier == "Exposed":
            convertExposedAttrToGlobalNameSet(attr, self._exposureGlobalNames)
        elif identifier == "Pure":
            if not attr.noArguments():
                raise WebIDLError("[Pure] must take no arguments", [attr.location])
            self._setDependsOn("DOMState")
            self._setAffects("Nothing")
        elif identifier == "Constant" or identifier == "SameObject":
            if not attr.noArguments():
                raise WebIDLError(
                    "[%s] must take no arguments" % identifier, [attr.location]
                )
            self._setDependsOn("Nothing")
            self._setAffects("Nothing")
        elif identifier == "Affects":
            if not attr.hasValue():
                raise WebIDLError("[Affects] takes an identifier", [attr.location])
            self._setAffects(attr.value())
        elif identifier == "DependsOn":
            if not attr.hasValue():
                raise WebIDLError("[DependsOn] takes an identifier", [attr.location])
            if (
                attr.value() != "Everything"
                and attr.value() != "DOMState"
                and not self.readonly
            ):
                raise WebIDLError(
                    "[DependsOn=%s] only allowed on "
                    "readonly attributes" % attr.value(),
                    [attr.location, self.location],
                )
            self._setDependsOn(attr.value())
        elif identifier == "UseCounter":
            if self.stringifier:
                raise WebIDLError(
                    "[UseCounter] must not be used on a " "stringifier attribute",
                    [attr.location, self.location],
                )
        elif identifier == "Unscopable":
            if not attr.noArguments():
                raise WebIDLError(
                    "[Unscopable] must take no arguments", [attr.location]
                )
            if self.isStatic():
                raise WebIDLError(
                    "[Unscopable] is only allowed on non-static "
                    "attributes and operations",
                    [attr.location, self.location],
                )
        elif identifier == "CEReactions":
            if not attr.noArguments():
                raise WebIDLError(
                    "[CEReactions] must take no arguments", [attr.location]
                )
        elif (
            identifier == "Pref"
            or identifier == "Deprecated"
            or identifier == "SetterThrows"
            or identifier == "Throws"
            or identifier == "GetterThrows"
            or identifier == "SetterCanOOM"
            or identifier == "CanOOM"
            or identifier == "GetterCanOOM"
            or identifier == "ChromeOnly"
            or identifier == "Func"
            or identifier == "Trial"
            or identifier == "SecureContext"
            or identifier == "Frozen"
            or identifier == "NewObject"
            or identifier == "NeedsSubjectPrincipal"
            or identifier == "SetterNeedsSubjectPrincipal"
            or identifier == "GetterNeedsSubjectPrincipal"
            or identifier == "NeedsCallerType"
            or identifier == "BinaryName"
            or identifier == "NonEnumerable"
            or identifier == "BindingTemplate"
            or identifier == "Unimplemented"
        ):
            # Known attributes that we don't need to do anything with here
            pass
        else:
            raise WebIDLError(
                "Unknown extended attribute %s on attribute" % identifier,
                [attr.location],
            )
        IDLInterfaceMember.handleExtendedAttribute(self, attr)

    def getExtendedAttributes(self):
        return self._extendedAttrDict

    def resolve(self, parentScope):
        assert isinstance(parentScope, IDLScope)
        self.type.resolveType(parentScope)
        IDLObjectWithIdentifier.resolve(self, parentScope)

    def hasLegacyLenientThis(self):
        return self.legacyLenientThis

    def isMaplikeOrSetlikeAttr(self):
        """
        True if this attribute was generated from an interface with
        maplike/setlike (e.g. this is the size attribute for
        maplike/setlike)
        """
        return self.maplikeOrSetlike is not None

    def isLegacyUnforgeable(self):
        return self._legacyUnforgeable

    def _getDependentObjects(self):
        return set([self.type])

    def expand(self, members):
        assert self.stringifier
        if (
            not self.type.isDOMString()
            and not self.type.isUSVString()
            and not self.type.isUTF8String()
        ):
            raise WebIDLError(
                "The type of a stringifer attribute must be "
                "either DOMString, USVString or UTF8String",
                [self.location],
            )
        identifier = IDLUnresolvedIdentifier(
            self.location, "__stringifier", allowDoubleUnderscore=True
        )
        method = IDLMethod(
            self.location,
            identifier,
            returnType=self.type,
            arguments=[],
            stringifier=True,
            underlyingAttr=self,
        )
        allowedExtAttrs = ["Throws", "NeedsSubjectPrincipal", "Pure"]
        # Safe to ignore these as they are only meaningful for attributes
        attributeOnlyExtAttrs = [
            "CEReactions",
            "CrossOriginWritable",
            "SetterThrows",
        ]
        for key, value in self._extendedAttrDict.items():
            if key in allowedExtAttrs:
                if value is not True:
                    raise WebIDLError(
                        "[%s] with a value is currently "
                        "unsupported in stringifier attributes, "
                        "please file a bug to add support" % key,
                        [self.location],
                    )
                method.addExtendedAttributes(
                    [IDLExtendedAttribute(self.location, (key,))]
                )
            elif key not in attributeOnlyExtAttrs:
                raise WebIDLError(
                    "[%s] is currently unsupported in "
                    "stringifier attributes, please file a bug "
                    "to add support" % key,
                    [self.location],
                )
        members.append(method)


class IDLArgument(IDLObjectWithIdentifier):
    __slots__ = (
        "type",
        "optional",
        "defaultValue",
        "variadic",
        "dictionaryMember",
        "_isComplete",
        "_allowTreatNonCallableAsNull",
        "_extendedAttrDict",
        "allowTypeAttributes",
    )

    def __init__(
        self,
        location,
        identifier,
        type,
        optional=False,
        defaultValue=None,
        variadic=False,
        dictionaryMember=False,
        allowTypeAttributes=False,
    ):
        IDLObjectWithIdentifier.__init__(self, location, None, identifier)

        assert isinstance(type, IDLType)
        self.type = type

        self.optional = optional
        self.defaultValue = defaultValue
        self.variadic = variadic
        self.dictionaryMember = dictionaryMember
        self._isComplete = False
        self._allowTreatNonCallableAsNull = False
        self._extendedAttrDict = {}
        self.allowTypeAttributes = allowTypeAttributes

        assert not variadic or optional
        assert not variadic or not defaultValue

    def addExtendedAttributes(self, attrs):
        for attribute in attrs:
            identifier = attribute.identifier()
            if self.allowTypeAttributes and (
                identifier == "EnforceRange"
                or identifier == "Clamp"
                or identifier == "LegacyNullToEmptyString"
                or identifier == "AllowShared"
            ):
                self.type = self.type.withExtendedAttributes([attribute])
            elif identifier == "TreatNonCallableAsNull":
                self._allowTreatNonCallableAsNull = True
            elif self.dictionaryMember and (
                identifier == "ChromeOnly"
                or identifier == "Func"
                or identifier == "Trial"
                or identifier == "Pref"
            ):
                if not self.optional:
                    raise WebIDLError(
                        "[%s] must not be used on a required "
                        "dictionary member" % identifier,
                        [attribute.location],
                    )
            elif self.dictionaryMember and identifier == "BinaryType":
                if not len(attribute.listValue()) == 1:
                    raise WebIDLError(
                        "[%s] BinaryType must take one argument" % identifier,
                        [attribute.location],
                    )
                if not self.defaultValue:
                    raise WebIDLError(
                        "[%s] BinaryType can't be used without default value"
                        % identifier,
                        [attribute.location],
                    )
            else:
                raise WebIDLError(
                    "Unhandled extended attribute on %s"
                    % (
                        "a dictionary member"
                        if self.dictionaryMember
                        else "an argument"
                    ),
                    [attribute.location],
                )
            attrlist = attribute.listValue()
            self._extendedAttrDict[identifier] = attrlist if len(attrlist) else True

    def getExtendedAttribute(self, name):
        return self._extendedAttrDict.get(name, None)

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

        if self.type.isUndefined():
            raise WebIDLError(
                "undefined must not be used as the type of an argument in any circumstance",
                [self.location],
            )

        if self.type.isAny():
            assert self.defaultValue is None or isinstance(
                self.defaultValue, IDLNullValue
            )
            # optional 'any' values always have a default value
            if self.optional and not self.defaultValue and not self.variadic:
                # Set the default value to undefined, for simplicity, so the
                # codegen doesn't have to special-case this.
                self.defaultValue = IDLUndefinedValue(self.location)

        if self.dictionaryMember and self.type.legacyNullToEmptyString:
            raise WebIDLError(
                "Dictionary members cannot be [LegacyNullToEmptyString]",
                [self.location],
            )
        if self.type.isObservableArray():
            raise WebIDLError(
                "%s cannot have an ObservableArray type"
                % ("Dictionary members" if self.dictionaryMember else "Arguments"),
                [self.location],
            )
        # Now do the coercing thing; this needs to happen after the
        # above creation of a default value.
        if self.defaultValue:
            self.defaultValue = self.defaultValue.coerceToType(self.type, self.location)
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
    __slots__ = (
        "_returnType",
        "_arguments",
        "_treatNonCallableAsNull",
        "_treatNonObjectAsNull",
        "_isRunScriptBoundary",
        "_isConstructor",
    )

    def __init__(
        self, location, parentScope, identifier, returnType, arguments, isConstructor
    ):
        assert isinstance(returnType, IDLType)

        self._returnType = returnType
        # Clone the list
        self._arguments = list(arguments)

        IDLObjectWithScope.__init__(self, location, parentScope, identifier)

        for returnType, arguments in self.signatures():
            for argument in arguments:
                argument.resolve(self)

        self._treatNonCallableAsNull = False
        self._treatNonObjectAsNull = False
        self._isRunScriptBoundary = False
        self._isConstructor = isConstructor

    def isCallback(self):
        return True

    def isConstructor(self):
        return self._isConstructor

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
        for argument in self._arguments:
            if argument.type.isUndefined():
                raise WebIDLError(
                    "undefined must not be used as the type of an argument in any circumstance",
                    [self.location],
                )

    def addExtendedAttributes(self, attrs):
        unhandledAttrs = []
        for attr in attrs:
            if attr.identifier() == "TreatNonCallableAsNull":
                self._treatNonCallableAsNull = True
            elif attr.identifier() == "LegacyTreatNonObjectAsNull":
                if self._isConstructor:
                    raise WebIDLError(
                        "[LegacyTreatNonObjectAsNull] is not supported "
                        "on constructors",
                        [self.location],
                    )
                self._treatNonObjectAsNull = True
            elif attr.identifier() == "MOZ_CAN_RUN_SCRIPT_BOUNDARY":
                if self._isConstructor:
                    raise WebIDLError(
                        "[MOZ_CAN_RUN_SCRIPT_BOUNDARY] is not "
                        "permitted on constructors",
                        [self.location],
                    )
                self._isRunScriptBoundary = True
            else:
                unhandledAttrs.append(attr)
        if self._treatNonCallableAsNull and self._treatNonObjectAsNull:
            raise WebIDLError(
                "Cannot specify both [TreatNonCallableAsNull] "
                "and [LegacyTreatNonObjectAsNull]",
                [self.location],
            )
        if len(unhandledAttrs) != 0:
            IDLType.addExtendedAttributes(self, unhandledAttrs)

    def _getDependentObjects(self):
        return set([self._returnType] + self._arguments)

    def isRunScriptBoundary(self):
        return self._isRunScriptBoundary


class IDLCallbackType(IDLType):
    __slots__ = ("callback",)

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
        # Callbacks without `LegacyTreatNonObjectAsNull` are distinguishable from Dictionary likes
        if other.isDictionaryLike():
            return not self.callback._treatNonObjectAsNull
        return (
            other.isUndefined()
            or other.isPrimitive()
            or other.isString()
            or other.isEnum()
            or other.isNonCallbackInterface()
            or other.isSequence()
        )

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

    __slots__ = "returnType", "arguments", "location"

    def __init__(self, returnType, arguments, location):
        self.returnType = returnType
        # Clone the list of arguments, just in case
        self.arguments = list(arguments)
        self.location = location

    def _getDependentObjects(self):
        deps = set(self.arguments)
        deps.add(self.returnType)
        return deps

    def includesRestrictedFloatArgument(self):
        return any(arg.type.includesRestrictedFloat() for arg in self.arguments)


class IDLMethod(IDLInterfaceMember, IDLScope):
    Special = enum(
        "Getter", "Setter", "Deleter", "LegacyCaller", base=IDLInterfaceMember.Special
    )

    NamedOrIndexed = enum("Neither", "Named", "Indexed")

    __slots__ = (
        "_hasOverloads",
        "_overloads",
        "_static",
        "_getter",
        "_setter",
        "_deleter",
        "_legacycaller",
        "_stringifier",
        "maplikeOrSetlikeOrIterable",
        "_htmlConstructor",
        "underlyingAttr",
        "_specialType",
        "_legacyUnforgeable",
        "dependsOn",
        "affects",
        "aliases",
    )

    def __init__(
        self,
        location,
        identifier,
        returnType,
        arguments,
        static=False,
        getter=False,
        setter=False,
        deleter=False,
        specialType=NamedOrIndexed.Neither,
        legacycaller=False,
        stringifier=False,
        maplikeOrSetlikeOrIterable=None,
        underlyingAttr=None,
    ):
        # REVIEW: specialType is NamedOrIndexed -- wow, this is messed up.
        IDLInterfaceMember.__init__(
            self, location, identifier, IDLInterfaceMember.Tags.Method
        )

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
        assert isinstance(deleter, bool)
        self._deleter = deleter
        assert isinstance(legacycaller, bool)
        self._legacycaller = legacycaller
        assert isinstance(stringifier, bool)
        self._stringifier = stringifier
        assert maplikeOrSetlikeOrIterable is None or isinstance(
            maplikeOrSetlikeOrIterable, IDLMaplikeOrSetlikeOrIterableBase
        )
        self.maplikeOrSetlikeOrIterable = maplikeOrSetlikeOrIterable
        self._htmlConstructor = False
        self.underlyingAttr = underlyingAttr
        self._specialType = specialType
        self._legacyUnforgeable = False
        self.dependsOn = "Everything"
        self.affects = "Everything"
        self.aliases = []

        if static and identifier.name == "prototype":
            raise WebIDLError(
                "The identifier of a static operation must not be 'prototype'",
                [location],
            )

        self.assertSignatureConstraints()

    def __str__(self):
        return "Method '%s'" % self.identifier

    def assertSignatureConstraints(self):
        if self._getter or self._deleter:
            assert len(self._overloads) == 1
            overload = self._overloads[0]
            arguments = overload.arguments
            assert len(arguments) == 1
            assert (
                arguments[0].type == BuiltinTypes[IDLBuiltinType.Types.domstring]
                or arguments[0].type == BuiltinTypes[IDLBuiltinType.Types.unsigned_long]
            )
            assert not arguments[0].optional and not arguments[0].variadic
            assert not self._getter or not overload.returnType.isUndefined()

        if self._setter:
            assert len(self._overloads) == 1
            arguments = self._overloads[0].arguments
            assert len(arguments) == 2
            assert (
                arguments[0].type == BuiltinTypes[IDLBuiltinType.Types.domstring]
                or arguments[0].type == BuiltinTypes[IDLBuiltinType.Types.unsigned_long]
            )
            assert not arguments[0].optional and not arguments[0].variadic
            assert not arguments[1].optional and not arguments[1].variadic

        if self._stringifier:
            assert len(self._overloads) == 1
            overload = self._overloads[0]
            assert len(overload.arguments) == 0
            if not self.underlyingAttr:
                assert (
                    overload.returnType == BuiltinTypes[IDLBuiltinType.Types.domstring]
                )

    def isStatic(self):
        return self._static

    def forceStatic(self):
        self._static = True

    def isGetter(self):
        return self._getter

    def isSetter(self):
        return self._setter

    def isDeleter(self):
        return self._deleter

    def isNamed(self):
        assert (
            self._specialType == IDLMethod.NamedOrIndexed.Named
            or self._specialType == IDLMethod.NamedOrIndexed.Indexed
        )
        return self._specialType == IDLMethod.NamedOrIndexed.Named

    def isIndexed(self):
        assert (
            self._specialType == IDLMethod.NamedOrIndexed.Named
            or self._specialType == IDLMethod.NamedOrIndexed.Indexed
        )
        return self._specialType == IDLMethod.NamedOrIndexed.Indexed

    def isLegacycaller(self):
        return self._legacycaller

    def isStringifier(self):
        return self._stringifier

    def isToJSON(self):
        return self.identifier.name == "toJSON"

    def isDefaultToJSON(self):
        return self.isToJSON() and self.getExtendedAttribute("Default")

    def isMaplikeOrSetlikeOrIterableMethod(self):
        """
        True if this method was generated as part of a
        maplike/setlike/etc interface (e.g. has/get methods)
        """
        return self.maplikeOrSetlikeOrIterable is not None

    def isSpecial(self):
        return (
            self.isGetter()
            or self.isSetter()
            or self.isDeleter()
            or self.isLegacycaller()
            or self.isStringifier()
        )

    def isHTMLConstructor(self):
        return self._htmlConstructor

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
        return (
            self.identifier.name[:2] == "__"
            and not self.isMaplikeOrSetlikeOrIterableMethod()
        )

    def resolve(self, parentScope):
        assert isinstance(parentScope, IDLScope)
        IDLObjectWithIdentifier.resolve(self, parentScope)
        IDLScope.__init__(self, self.location, parentScope, self.identifier)
        for returnType, arguments in self.signatures():
            for argument in arguments:
                argument.resolve(self)

    def addOverload(self, method):
        assert len(method._overloads) == 1

        if self._extendedAttrDict != method._extendedAttrDict:
            extendedAttrDiff = set(self._extendedAttrDict.keys()) ^ set(
                method._extendedAttrDict.keys()
            )

            if extendedAttrDiff == {"LenientFloat"}:
                if "LenientFloat" not in self._extendedAttrDict:
                    for overload in self._overloads:
                        if overload.includesRestrictedFloatArgument():
                            raise WebIDLError(
                                "Restricted float behavior differs on different "
                                "overloads of %s" % method.identifier,
                                [overload.location, method.location],
                            )
                    self._extendedAttrDict["LenientFloat"] = method._extendedAttrDict[
                        "LenientFloat"
                    ]
                elif method._overloads[0].includesRestrictedFloatArgument():
                    raise WebIDLError(
                        "Restricted float behavior differs on different "
                        "overloads of %s" % method.identifier,
                        [self.location, method.location],
                    )
            else:
                raise WebIDLError(
                    "Extended attributes differ on different "
                    "overloads of %s" % method.identifier,
                    [self.location, method.location],
                )

        self._overloads.extend(method._overloads)

        self._hasOverloads = True

        if self.isStatic() != method.isStatic():
            raise WebIDLError(
                "Overloaded identifier %s appears with different values of the 'static' attribute"
                % method.identifier,
                [method.location],
            )

        if self.isLegacycaller() != method.isLegacycaller():
            raise WebIDLError(
                (
                    "Overloaded identifier %s appears with different "
                    "values of the 'legacycaller' attribute" % method.identifier
                ),
                [method.location],
            )

        # Can't overload special things!
        if (
            self.isGetter()
            or method.isGetter()
            or self.isSetter()
            or method.isSetter()
            or self.isDeleter()
            or method.isDeleter()
            or self.isStringifier()
            or method.isStringifier()
        ):
            raise WebIDLError(
                ("Can't overload a special operation"),
                [self.location, method.location],
            )
        if self.isHTMLConstructor() or method.isHTMLConstructor():
            raise WebIDLError(
                (
                    "An interface must contain only a single operation annotated with HTMLConstructor, and no others"
                ),
                [self.location, method.location],
            )

        return self

    def signatures(self):
        return [
            (overload.returnType, overload.arguments) for overload in self._overloads
        ]

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
        self.allowedArgCounts = [
            i
            for i in range(self.maxArgCount + 1)
            if len(self.signaturesForArgCount(i)) != 0
        ]

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
                            "is before distinguishing index %d"
                            % (
                                self.identifier.name,
                                argCount,
                                idx,
                                distinguishingIndex,
                            ),
                            [self.location, overload.location],
                        )

        overloadWithPromiseReturnType = None
        overloadWithoutPromiseReturnType = None
        for overload in self._overloads:
            returnType = overload.returnType
            if not returnType.unroll().isExposedInAllOf(self.exposureSet):
                raise WebIDLError(
                    "Overload returns a type that is not exposed "
                    "everywhere where the method is exposed",
                    [overload.location],
                )

            variadicArgument = None

            arguments = overload.arguments
            for idx, argument in enumerate(arguments):
                assert argument.type.isComplete()

                if (
                    argument.type.isDictionary()
                    and argument.type.unroll().inner.canBeEmpty()
                ) or (
                    argument.type.isUnion()
                    and argument.type.unroll().hasPossiblyEmptyDictionaryType()
                ):
                    # Optional dictionaries and unions containing optional
                    # dictionaries at the end of the list or followed by
                    # optional arguments must be optional.
                    if not argument.optional and all(
                        arg.optional for arg in arguments[idx + 1 :]
                    ):
                        raise WebIDLError(
                            "Dictionary argument without any "
                            "required fields or union argument "
                            "containing such dictionary not "
                            "followed by a required argument "
                            "must be optional",
                            [argument.location],
                        )

                    if not argument.defaultValue and all(
                        arg.optional for arg in arguments[idx + 1 :]
                    ):
                        raise WebIDLError(
                            "Dictionary argument without any "
                            "required fields or union argument "
                            "containing such dictionary not "
                            "followed by a required argument "
                            "must have a default value",
                            [argument.location],
                        )

                # An argument cannot be a nullable dictionary or a
                # nullable union containing a dictionary.
                if argument.type.nullable() and (
                    argument.type.isDictionary()
                    or (
                        argument.type.isUnion()
                        and argument.type.unroll().hasDictionaryType()
                    )
                ):
                    raise WebIDLError(
                        "An argument cannot be a nullable "
                        "dictionary or nullable union "
                        "containing a dictionary",
                        [argument.location],
                    )

                # Only the last argument can be variadic
                if variadicArgument:
                    raise WebIDLError(
                        "Variadic argument is not last argument",
                        [variadicArgument.location],
                    )
                if argument.variadic:
                    variadicArgument = argument

            if returnType.isPromise():
                overloadWithPromiseReturnType = overload
            else:
                overloadWithoutPromiseReturnType = overload

        # Make sure either all our overloads return Promises or none do
        if overloadWithPromiseReturnType and overloadWithoutPromiseReturnType:
            raise WebIDLError(
                "We have overloads with both Promise and " "non-Promise return types",
                [
                    overloadWithPromiseReturnType.location,
                    overloadWithoutPromiseReturnType.location,
                ],
            )

        if overloadWithPromiseReturnType and self._legacycaller:
            raise WebIDLError(
                "May not have a Promise return type for a " "legacycaller.",
                [overloadWithPromiseReturnType.location],
            )

        if self.getExtendedAttribute("StaticClassOverride") and not (
            self.identifier.scope.isJSImplemented() and self.isStatic()
        ):
            raise WebIDLError(
                "StaticClassOverride can be applied to static"
                " methods on JS-implemented classes only.",
                [self.location],
            )

        # Ensure that toJSON methods satisfy the spec constraints on them.
        if self.identifier.name == "toJSON":
            if len(self.signatures()) != 1:
                raise WebIDLError(
                    "toJSON method has multiple overloads",
                    [self._overloads[0].location, self._overloads[1].location],
                )
            if len(self.signatures()[0][1]) != 0:
                raise WebIDLError("toJSON method has arguments", [self.location])
            if not self.signatures()[0][0].isJSONType():
                raise WebIDLError(
                    "toJSON method has non-JSON return type", [self.location]
                )

    def overloadsForArgCount(self, argc):
        return [
            overload
            for overload in self._overloads
            if len(overload.arguments) == argc
            or (
                len(overload.arguments) > argc
                and all(arg.optional for arg in overload.arguments[argc:])
            )
            or (
                len(overload.arguments) < argc
                and len(overload.arguments) > 0
                and overload.arguments[-1].variadic
            )
        ]

    def signaturesForArgCount(self, argc):
        return [
            (overload.returnType, overload.arguments)
            for overload in self.overloadsForArgCount(argc)
        ]

    def locationsForArgCount(self, argc):
        return [overload.location for overload in self.overloadsForArgCount(argc)]

    def distinguishingIndexForArgCount(self, argc):
        def isValidDistinguishingIndex(idx, signatures):
            for firstSigIndex, (firstRetval, firstArgs) in enumerate(signatures[:-1]):
                for secondRetval, secondArgs in signatures[firstSigIndex + 1 :]:
                    if idx < len(firstArgs):
                        firstType = firstArgs[idx].type
                    else:
                        assert firstArgs[-1].variadic
                        firstType = firstArgs[-1].type
                    if idx < len(secondArgs):
                        secondType = secondArgs[idx].type
                    else:
                        assert secondArgs[-1].variadic
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
        raise WebIDLError(
            "Signatures with %d arguments for method '%s' are not "
            "distinguishable" % (argc, self.identifier.name),
            locations,
        )

    def handleExtendedAttribute(self, attr):
        identifier = attr.identifier()
        if (
            identifier == "GetterThrows"
            or identifier == "SetterThrows"
            or identifier == "GetterCanOOM"
            or identifier == "SetterCanOOM"
            or identifier == "SetterNeedsSubjectPrincipal"
            or identifier == "GetterNeedsSubjectPrincipal"
        ):
            raise WebIDLError(
                "Methods must not be flagged as " "[%s]" % identifier,
                [attr.location, self.location],
            )
        elif identifier == "LegacyUnforgeable":
            if self.isStatic():
                raise WebIDLError(
                    "[LegacyUnforgeable] is only allowed on non-static " "methods",
                    [attr.location, self.location],
                )
            self._legacyUnforgeable = True
        elif identifier == "SameObject":
            raise WebIDLError(
                "Methods must not be flagged as [SameObject]",
                [attr.location, self.location],
            )
        elif identifier == "Constant":
            raise WebIDLError(
                "Methods must not be flagged as [Constant]",
                [attr.location, self.location],
            )
        elif identifier == "PutForwards":
            raise WebIDLError(
                "Only attributes support [PutForwards]", [attr.location, self.location]
            )
        elif identifier == "LegacyLenientSetter":
            raise WebIDLError(
                "Only attributes support [LegacyLenientSetter]",
                [attr.location, self.location],
            )
        elif identifier == "LenientFloat":
            # This is called before we've done overload resolution
            overloads = self._overloads
            assert len(overloads) == 1
            if not overloads[0].returnType.isUndefined():
                raise WebIDLError(
                    "[LenientFloat] used on a non-undefined method",
                    [attr.location, self.location],
                )
            if not overloads[0].includesRestrictedFloatArgument():
                raise WebIDLError(
                    "[LenientFloat] used on an operation with no "
                    "restricted float type arguments",
                    [attr.location, self.location],
                )
        elif identifier == "Exposed":
            convertExposedAttrToGlobalNameSet(attr, self._exposureGlobalNames)
        elif (
            identifier == "CrossOriginCallable"
            or identifier == "WebGLHandlesContextLoss"
        ):
            # Known no-argument attributes.
            if not attr.noArguments():
                raise WebIDLError(
                    "[%s] must take no arguments" % identifier, [attr.location]
                )
            if identifier == "CrossOriginCallable" and self.isStatic():
                raise WebIDLError(
                    "[CrossOriginCallable] is only allowed on non-static " "attributes",
                    [attr.location, self.location],
                )
        elif identifier == "Pure":
            if not attr.noArguments():
                raise WebIDLError("[Pure] must take no arguments", [attr.location])
            self._setDependsOn("DOMState")
            self._setAffects("Nothing")
        elif identifier == "Affects":
            if not attr.hasValue():
                raise WebIDLError("[Affects] takes an identifier", [attr.location])
            self._setAffects(attr.value())
        elif identifier == "DependsOn":
            if not attr.hasValue():
                raise WebIDLError("[DependsOn] takes an identifier", [attr.location])
            self._setDependsOn(attr.value())
        elif identifier == "Alias":
            if not attr.hasValue():
                raise WebIDLError(
                    "[Alias] takes an identifier or string", [attr.location]
                )
            self._addAlias(attr.value())
        elif identifier == "UseCounter":
            if self.isSpecial():
                raise WebIDLError(
                    "[UseCounter] must not be used on a special " "operation",
                    [attr.location, self.location],
                )
        elif identifier == "Unscopable":
            if not attr.noArguments():
                raise WebIDLError(
                    "[Unscopable] must take no arguments", [attr.location]
                )
            if self.isStatic():
                raise WebIDLError(
                    "[Unscopable] is only allowed on non-static "
                    "attributes and operations",
                    [attr.location, self.location],
                )
        elif identifier == "CEReactions":
            if not attr.noArguments():
                raise WebIDLError(
                    "[CEReactions] must take no arguments", [attr.location]
                )

            if self.isSpecial() and not self.isSetter() and not self.isDeleter():
                raise WebIDLError(
                    "[CEReactions] is only allowed on operation, "
                    "attribute, setter, and deleter",
                    [attr.location, self.location],
                )
        elif identifier == "Default":
            if not attr.noArguments():
                raise WebIDLError("[Default] must take no arguments", [attr.location])

            if not self.isToJSON():
                raise WebIDLError(
                    "[Default] is only allowed on toJSON operations",
                    [attr.location, self.location],
                )

            if self.signatures()[0][0] != BuiltinTypes[IDLBuiltinType.Types.object]:
                raise WebIDLError(
                    "The return type of the default toJSON "
                    "operation must be 'object'",
                    [attr.location, self.location],
                )
        elif (
            identifier == "Throws"
            or identifier == "CanOOM"
            or identifier == "NewObject"
            or identifier == "ChromeOnly"
            or identifier == "Pref"
            or identifier == "Deprecated"
            or identifier == "Func"
            or identifier == "Trial"
            or identifier == "SecureContext"
            or identifier == "BinaryName"
            or identifier == "NeedsSubjectPrincipal"
            or identifier == "NeedsCallerType"
            or identifier == "StaticClassOverride"
            or identifier == "NonEnumerable"
            or identifier == "Unexposed"
            or identifier == "WebExtensionStub"
            or identifier == "Unimplemented"
        ):
            # Known attributes that we don't need to do anything with here
            pass
        else:
            raise WebIDLError(
                "Unknown extended attribute %s on method" % identifier, [attr.location]
            )
        IDLInterfaceMember.handleExtendedAttribute(self, attr)

    def returnsPromise(self):
        return self._overloads[0].returnType.isPromise()

    def isLegacyUnforgeable(self):
        return self._legacyUnforgeable

    def _getDependentObjects(self):
        deps = set()
        for overload in self._overloads:
            deps.update(overload._getDependentObjects())
        return deps


class IDLConstructor(IDLMethod):
    __slots__ = (
        "_initLocation",
        "_initArgs",
        "_initName",
        "_inited",
        "_initExtendedAttrs",
    )

    def __init__(self, location, args, name):
        # We can't actually init our IDLMethod yet, because we do not know the
        # return type yet.  Just save the info we have for now and we will init
        # it later.
        self._initLocation = location
        self._initArgs = args
        self._initName = name
        self._inited = False
        self._initExtendedAttrs = []

    def addExtendedAttributes(self, attrs):
        if self._inited:
            return IDLMethod.addExtendedAttributes(self, attrs)
        self._initExtendedAttrs.extend(attrs)

    def handleExtendedAttribute(self, attr):
        identifier = attr.identifier()
        if (
            identifier == "BinaryName"
            or identifier == "ChromeOnly"
            or identifier == "NewObject"
            or identifier == "SecureContext"
            or identifier == "Throws"
            or identifier == "Func"
            or identifier == "Trial"
            or identifier == "Pref"
            or identifier == "UseCounter"
            or identifier == "Unimplemented"
        ):
            IDLMethod.handleExtendedAttribute(self, attr)
        elif identifier == "HTMLConstructor":
            if not attr.noArguments():
                raise WebIDLError(
                    "[HTMLConstructor] must take no arguments", [attr.location]
                )
            # We shouldn't end up here for legacy factory functions.
            assert self.identifier.name == "constructor"

            if any(len(sig[1]) != 0 for sig in self.signatures()):
                raise WebIDLError(
                    "[HTMLConstructor] must not be applied to a "
                    "constructor operation that has arguments.",
                    [attr.location],
                )
            self._htmlConstructor = True
        else:
            raise WebIDLError(
                "Unknown extended attribute %s on method" % identifier, [attr.location]
            )

    def reallyInit(self, parentInterface):
        name = self._initName
        location = self._initLocation
        identifier = IDLUnresolvedIdentifier(location, name, allowForbidden=True)
        retType = IDLWrapperType(parentInterface.location, parentInterface)
        IDLMethod.__init__(
            self, location, identifier, retType, self._initArgs, static=True
        )
        self._inited = True
        # Propagate through whatever extended attributes we already had
        self.addExtendedAttributes(self._initExtendedAttrs)
        self._initExtendedAttrs = []
        # Constructors are always NewObject.  Whether they throw or not is
        # indicated by [Throws] annotations in the usual way.
        self.addExtendedAttributes(
            [IDLExtendedAttribute(self.location, ("NewObject",))]
        )


class IDLIncludesStatement(IDLObject):
    __slots__ = ("interface", "mixin", "_finished")

    def __init__(self, location, interface, mixin):
        IDLObject.__init__(self, location)
        self.interface = interface
        self.mixin = mixin
        self._finished = False

    def finish(self, scope):
        if self._finished:
            return
        self._finished = True
        assert isinstance(self.interface, IDLIdentifierPlaceholder)
        assert isinstance(self.mixin, IDLIdentifierPlaceholder)
        interface = self.interface.finish(scope)
        mixin = self.mixin.finish(scope)
        # NOTE: we depend on not setting self.interface and
        # self.mixin here to keep track of the original
        # locations.
        if not isinstance(interface, IDLInterface):
            raise WebIDLError(
                "Left-hand side of 'includes' is not an " "interface",
                [self.interface.location, interface.location],
            )
        if interface.isCallback():
            raise WebIDLError(
                "Left-hand side of 'includes' is a callback " "interface",
                [self.interface.location, interface.location],
            )
        if not isinstance(mixin, IDLInterfaceMixin):
            raise WebIDLError(
                "Right-hand side of 'includes' is not an " "interface mixin",
                [self.mixin.location, mixin.location],
            )

        mixin.actualExposureGlobalNames.update(interface._exposureGlobalNames)

        interface.addIncludedMixin(mixin)
        self.interface = interface
        self.mixin = mixin

    def validate(self):
        pass

    def addExtendedAttributes(self, attrs):
        if len(attrs) != 0:
            raise WebIDLError(
                "There are no extended attributes that are "
                "allowed on includes statements",
                [attrs[0].location, self.location],
            )


class IDLExtendedAttribute(IDLObject):
    """
    A class to represent IDL extended attributes so we can give them locations
    """

    __slots__ = ("_tuple",)

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
        assert self.hasValue()
        return self._tuple[1]

    def hasArgs(self):
        return (
            len(self._tuple) == 2
            and isinstance(self._tuple[1], list)
            or len(self._tuple) == 3
        )

    def args(self):
        assert self.hasArgs()
        # Our args are our last element
        return self._tuple[-1]

    def listValue(self):
        """
        Backdoor for storing random data in _extendedAttrDict
        """
        return list(self._tuple)[1:]


# Parser


class Tokenizer(object):
    tokens = ["INTEGER", "FLOATLITERAL", "IDENTIFIER", "STRING", "COMMENTS", "OTHER"]

    def t_FLOATLITERAL(self, t):
        r"(-?(([0-9]+\.[0-9]*|[0-9]*\.[0-9]+)([Ee][+-]?[0-9]+)?|[0-9]+[Ee][+-]?[0-9]+|Infinity))|NaN"
        t.value = float(t.value)
        return t

    def t_INTEGER(self, t):
        r"-?(0([0-7]+|[Xx][0-9A-Fa-f]+)?|[1-9][0-9]*)"
        try:
            # Can't use int(), because that doesn't handle octal properly.
            t.value = parseInt(t.value)
        except Exception:
            raise WebIDLError(
                "Invalid integer literal",
                [
                    Location(
                        lexer=self.lexer,
                        lineno=self.lexer.lineno,
                        lexpos=self.lexer.lexpos,
                        filename=self._filename,
                    )
                ],
            )
        return t

    def t_IDENTIFIER(self, t):
        r"[_-]?[A-Za-z][0-9A-Z_a-z-]*"
        t.type = self.keywords.get(t.value, "IDENTIFIER")
        return t

    def t_STRING(self, t):
        r'"[^"]*"'
        t.value = t.value[1:-1]
        return t

    t_ignore = "\t\n\r "

    def t_COMMENTS(self, t):
        r"//[^\n]*|/\*(?s:.)*?\*/"
        pass

    def t_ELLIPSIS(self, t):
        r"\.\.\."
        t.type = "ELLIPSIS"
        return t

    def t_OTHER(self, t):
        r"[^0-9A-Z_a-z]"
        t.type = self.keywords.get(t.value, "OTHER")
        return t

    keywords = {
        "interface": "INTERFACE",
        "partial": "PARTIAL",
        "mixin": "MIXIN",
        "dictionary": "DICTIONARY",
        "exception": "EXCEPTION",
        "enum": "ENUM",
        "callback": "CALLBACK",
        "typedef": "TYPEDEF",
        "includes": "INCLUDES",
        "const": "CONST",
        "null": "NULL",
        "true": "TRUE",
        "false": "FALSE",
        "serializer": "SERIALIZER",
        "stringifier": "STRINGIFIER",
        "unrestricted": "UNRESTRICTED",
        "attribute": "ATTRIBUTE",
        "readonly": "READONLY",
        "inherit": "INHERIT",
        "static": "STATIC",
        "getter": "GETTER",
        "setter": "SETTER",
        "deleter": "DELETER",
        "legacycaller": "LEGACYCALLER",
        "optional": "OPTIONAL",
        "...": "ELLIPSIS",
        "::": "SCOPE",
        "DOMString": "DOMSTRING",
        "ByteString": "BYTESTRING",
        "USVString": "USVSTRING",
        "JSString": "JSSTRING",
        "UTF8String": "UTF8STRING",
        "any": "ANY",
        "boolean": "BOOLEAN",
        "byte": "BYTE",
        "double": "DOUBLE",
        "float": "FLOAT",
        "long": "LONG",
        "object": "OBJECT",
        "ObservableArray": "OBSERVABLEARRAY",
        "octet": "OCTET",
        "Promise": "PROMISE",
        "required": "REQUIRED",
        "sequence": "SEQUENCE",
        "record": "RECORD",
        "short": "SHORT",
        "unsigned": "UNSIGNED",
        "undefined": "UNDEFINED",
        ":": "COLON",
        ";": "SEMICOLON",
        "{": "LBRACE",
        "}": "RBRACE",
        "(": "LPAREN",
        ")": "RPAREN",
        "[": "LBRACKET",
        "]": "RBRACKET",
        "?": "QUESTIONMARK",
        "*": "ASTERISK",
        ",": "COMMA",
        "=": "EQUALS",
        "<": "LT",
        ">": "GT",
        "ArrayBuffer": "ARRAYBUFFER",
        "or": "OR",
        "maplike": "MAPLIKE",
        "setlike": "SETLIKE",
        "iterable": "ITERABLE",
        "namespace": "NAMESPACE",
        "constructor": "CONSTRUCTOR",
        "symbol": "SYMBOL",
        "async": "ASYNC",
    }

    tokens.extend(keywords.values())

    def t_error(self, t):
        raise WebIDLError(
            "Unrecognized Input",
            [
                Location(
                    lexer=self.lexer,
                    lineno=self.lexer.lineno,
                    lexpos=self.lexer.lexpos,
                    filename=self._filename,
                )
            ],
        )

    def __init__(self, outputdir, lexer=None):
        if lexer:
            self.lexer = lexer
        else:
            self.lexer = lex.lex(object=self)


class SqueakyCleanLogger(object):
    errorWhitelist = [
        # Web IDL defines the COMMENTS token, but doesn't actually
        # use it ... so far.
        "Token 'COMMENTS' defined, but not used",
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
        if (
            msg == "%s:%d: Rule %r defined, but not used"
            or msg == "%s:%d: Rule '%s' defined, but not used"
        ):
            # Munge things so we don't have to hardcode filenames and
            # line numbers in our whitelist.
            whitelistmsg = "Rule %r defined, but not used"
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
        Definition : CallbackOrInterfaceOrMixin
                   | Namespace
                   | Partial
                   | Dictionary
                   | Exception
                   | Enum
                   | Typedef
                   | IncludesStatement
        """
        p[0] = p[1]
        assert p[1]  # We might not have implemented something ...

    def p_CallbackOrInterfaceOrMixinCallback(self, p):
        """
        CallbackOrInterfaceOrMixin : CALLBACK CallbackRestOrInterface
        """
        if p[2].isInterface():
            assert isinstance(p[2], IDLInterface)
            p[2].setCallback(True)

        p[0] = p[2]

    def p_CallbackOrInterfaceOrMixinInterfaceOrMixin(self, p):
        """
        CallbackOrInterfaceOrMixin : INTERFACE InterfaceOrMixin
        """
        p[0] = p[2]

    def p_CallbackRestOrInterface(self, p):
        """
        CallbackRestOrInterface : CallbackRest
                                | CallbackConstructorRest
                                | CallbackInterface
        """
        assert p[1]
        p[0] = p[1]

    def handleNonPartialObject(
        self, location, identifier, constructor, constructorArgs, nonPartialArgs
    ):
        """
        This handles non-partial objects (interfaces, namespaces and
        dictionaries) by checking for an existing partial object, and promoting
        it to non-partial as needed.  The return value is the non-partial
        object.

        constructorArgs are all the args for the constructor except the last
        one: isKnownNonPartial.

        nonPartialArgs are the args for the setNonPartial call.
        """
        # The name of the class starts with "IDL", so strip that off.
        # Also, starts with a capital letter after that, so nix that
        # as well.
        prettyname = constructor.__name__[3:].lower()

        try:
            existingObj = self.globalScope()._lookupIdentifier(identifier)
            if existingObj:
                if not isinstance(existingObj, constructor):
                    raise WebIDLError(
                        "%s has the same name as "
                        "non-%s object" % (prettyname.capitalize(), prettyname),
                        [location, existingObj.location],
                    )
                existingObj.setNonPartial(*nonPartialArgs)
                return existingObj
        except Exception as ex:
            if isinstance(ex, WebIDLError):
                raise ex
            pass

        # True for isKnownNonPartial
        return constructor(*(constructorArgs + [True]))

    def p_InterfaceOrMixin(self, p):
        """
        InterfaceOrMixin : InterfaceRest
                         | MixinRest
        """
        p[0] = p[1]

    def p_CallbackInterface(self, p):
        """
        CallbackInterface : INTERFACE InterfaceRest
        """
        p[0] = p[2]

    def p_InterfaceRest(self, p):
        """
        InterfaceRest : IDENTIFIER Inheritance LBRACE InterfaceMembers RBRACE SEMICOLON
        """
        location = self.getLocation(p, 1)
        identifier = IDLUnresolvedIdentifier(location, p[1])
        members = p[4]
        parent = p[2]

        p[0] = self.handleNonPartialObject(
            location,
            identifier,
            IDLInterface,
            [location, self.globalScope(), identifier, parent, members],
            [location, parent, members],
        )

    def p_InterfaceForwardDecl(self, p):
        """
        InterfaceRest : IDENTIFIER SEMICOLON
        """
        location = self.getLocation(p, 1)
        identifier = IDLUnresolvedIdentifier(location, p[1])

        try:
            if self.globalScope()._lookupIdentifier(identifier):
                p[0] = self.globalScope()._lookupIdentifier(identifier)
                if not isinstance(p[0], IDLExternalInterface):
                    raise WebIDLError(
                        "Name collision between external "
                        "interface declaration for identifier "
                        "%s and %s" % (identifier.name, p[0]),
                        [location, p[0].location],
                    )
                return
        except Exception as ex:
            if isinstance(ex, WebIDLError):
                raise ex
            pass

        p[0] = IDLExternalInterface(location, self.globalScope(), identifier)

    def p_MixinRest(self, p):
        """
        MixinRest : MIXIN IDENTIFIER LBRACE MixinMembers RBRACE SEMICOLON
        """
        location = self.getLocation(p, 1)
        identifier = IDLUnresolvedIdentifier(self.getLocation(p, 2), p[2])
        members = p[4]

        p[0] = self.handleNonPartialObject(
            location,
            identifier,
            IDLInterfaceMixin,
            [location, self.globalScope(), identifier, members],
            [location, members],
        )

    def p_Namespace(self, p):
        """
        Namespace : NAMESPACE IDENTIFIER LBRACE InterfaceMembers RBRACE SEMICOLON
        """
        location = self.getLocation(p, 1)
        identifier = IDLUnresolvedIdentifier(self.getLocation(p, 2), p[2])
        members = p[4]

        p[0] = self.handleNonPartialObject(
            location,
            identifier,
            IDLNamespace,
            [location, self.globalScope(), identifier, members],
            [location, None, members],
        )

    def p_Partial(self, p):
        """
        Partial : PARTIAL PartialDefinition
        """
        p[0] = p[2]

    def p_PartialDefinitionInterface(self, p):
        """
        PartialDefinition : INTERFACE PartialInterfaceOrPartialMixin
        """
        p[0] = p[2]

    def p_PartialDefinition(self, p):
        """
        PartialDefinition : PartialNamespace
                          | PartialDictionary
        """
        p[0] = p[1]

    def handlePartialObject(
        self,
        location,
        identifier,
        nonPartialConstructor,
        nonPartialConstructorArgs,
        partialConstructorArgs,
    ):
        """
        This handles partial objects (interfaces, namespaces and dictionaries)
        by checking for an existing non-partial object, and adding ourselves to
        it as needed.  The return value is our partial object.  We use
        IDLPartialInterfaceOrNamespace for partial interfaces or namespaces,
        and IDLPartialDictionary for partial dictionaries.

        nonPartialConstructorArgs are all the args for the non-partial
        constructor except the last two: members and isKnownNonPartial.

        partialConstructorArgs are the arguments for the partial object
        constructor, except the last one (the non-partial object).
        """
        # The name of the class starts with "IDL", so strip that off.
        # Also, starts with a capital letter after that, so nix that
        # as well.
        prettyname = nonPartialConstructor.__name__[3:].lower()

        nonPartialObject = None
        try:
            nonPartialObject = self.globalScope()._lookupIdentifier(identifier)
            if nonPartialObject:
                if not isinstance(nonPartialObject, nonPartialConstructor):
                    raise WebIDLError(
                        "Partial %s has the same name as "
                        "non-%s object" % (prettyname, prettyname),
                        [location, nonPartialObject.location],
                    )
        except Exception as ex:
            if isinstance(ex, WebIDLError):
                raise ex
            pass

        if not nonPartialObject:
            nonPartialObject = nonPartialConstructor(
                # No members, False for isKnownNonPartial
                *(nonPartialConstructorArgs),
                members=[],
                isKnownNonPartial=False
            )

        partialObject = None
        if isinstance(nonPartialObject, IDLDictionary):
            partialObject = IDLPartialDictionary(
                *(partialConstructorArgs + [nonPartialObject])
            )
        elif isinstance(
            nonPartialObject, (IDLInterface, IDLInterfaceMixin, IDLNamespace)
        ):
            partialObject = IDLPartialInterfaceOrNamespace(
                *(partialConstructorArgs + [nonPartialObject])
            )
        else:
            raise WebIDLError(
                "Unknown partial object type %s" % type(partialObject), [location]
            )

        return partialObject

    def p_PartialInterfaceOrPartialMixin(self, p):
        """
        PartialInterfaceOrPartialMixin : PartialInterfaceRest
                                       | PartialMixinRest
        """
        p[0] = p[1]

    def p_PartialInterfaceRest(self, p):
        """
        PartialInterfaceRest : IDENTIFIER LBRACE PartialInterfaceMembers RBRACE SEMICOLON
        """
        location = self.getLocation(p, 1)
        identifier = IDLUnresolvedIdentifier(location, p[1])
        members = p[3]

        p[0] = self.handlePartialObject(
            location,
            identifier,
            IDLInterface,
            [location, self.globalScope(), identifier, None],
            [location, identifier, members],
        )

    def p_PartialMixinRest(self, p):
        """
        PartialMixinRest : MIXIN IDENTIFIER LBRACE MixinMembers RBRACE SEMICOLON
        """
        location = self.getLocation(p, 1)
        identifier = IDLUnresolvedIdentifier(self.getLocation(p, 2), p[2])
        members = p[4]

        p[0] = self.handlePartialObject(
            location,
            identifier,
            IDLInterfaceMixin,
            [location, self.globalScope(), identifier],
            [location, identifier, members],
        )

    def p_PartialNamespace(self, p):
        """
        PartialNamespace : NAMESPACE IDENTIFIER LBRACE InterfaceMembers RBRACE SEMICOLON
        """
        location = self.getLocation(p, 1)
        identifier = IDLUnresolvedIdentifier(self.getLocation(p, 2), p[2])
        members = p[4]

        p[0] = self.handlePartialObject(
            location,
            identifier,
            IDLNamespace,
            [location, self.globalScope(), identifier],
            [location, identifier, members],
        )

    def p_PartialDictionary(self, p):
        """
        PartialDictionary : DICTIONARY IDENTIFIER LBRACE DictionaryMembers RBRACE SEMICOLON
        """
        location = self.getLocation(p, 1)
        identifier = IDLUnresolvedIdentifier(self.getLocation(p, 2), p[2])
        members = p[4]

        p[0] = self.handlePartialObject(
            location,
            identifier,
            IDLDictionary,
            [location, self.globalScope(), identifier],
            [location, identifier, members],
        )

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
        p[0] = [p[2]]

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
        InterfaceMember : PartialInterfaceMember
                        | Constructor
        """
        p[0] = p[1]

    def p_Constructor(self, p):
        """
        Constructor : CONSTRUCTOR LPAREN ArgumentList RPAREN SEMICOLON
        """
        p[0] = IDLConstructor(self.getLocation(p, 1), p[3], "constructor")

    def p_PartialInterfaceMembers(self, p):
        """
        PartialInterfaceMembers : ExtendedAttributeList PartialInterfaceMember PartialInterfaceMembers
        """
        p[0] = [p[2]]

        assert not p[1] or p[2]
        p[2].addExtendedAttributes(p[1])

        p[0].extend(p[3])

    def p_PartialInterfaceMembersEmpty(self, p):
        """
        PartialInterfaceMembers :
        """
        p[0] = []

    def p_PartialInterfaceMember(self, p):
        """
        PartialInterfaceMember : Const
                               | AttributeOrOperationOrMaplikeOrSetlikeOrIterable
        """
        p[0] = p[1]

    def p_MixinMembersEmpty(self, p):
        """
        MixinMembers :
        """
        p[0] = []

    def p_MixinMembers(self, p):
        """
        MixinMembers : ExtendedAttributeList MixinMember MixinMembers
        """
        p[0] = [p[2]]

        assert not p[1] or p[2]
        p[2].addExtendedAttributes(p[1])

        p[0].extend(p[3])

    def p_MixinMember(self, p):
        """
        MixinMember : Const
                    | Attribute
                    | Operation
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
        p[2].addExtendedAttributes(p[1])
        p[0] = [p[2]]
        p[0].extend(p[3])

    def p_DictionaryMemberRequired(self, p):
        """
        DictionaryMember : REQUIRED TypeWithExtendedAttributes IDENTIFIER SEMICOLON
        """
        # These quack a lot like required arguments, so just treat them that way.
        t = p[2]
        assert isinstance(t, IDLType)
        identifier = IDLUnresolvedIdentifier(self.getLocation(p, 3), p[3])

        p[0] = IDLArgument(
            self.getLocation(p, 3),
            identifier,
            t,
            optional=False,
            defaultValue=None,
            variadic=False,
            dictionaryMember=True,
        )

    def p_DictionaryMember(self, p):
        """
        DictionaryMember : Type IDENTIFIER Default SEMICOLON
        """
        # These quack a lot like optional arguments, so just treat them that way.
        t = p[1]
        assert isinstance(t, IDLType)
        identifier = IDLUnresolvedIdentifier(self.getLocation(p, 2), p[2])
        defaultValue = p[3]

        # Any attributes that precede this may apply to the type, so
        # we configure the argument to forward type attributes down instead of producing
        # a parse error
        p[0] = IDLArgument(
            self.getLocation(p, 2),
            identifier,
            t,
            optional=True,
            defaultValue=defaultValue,
            variadic=False,
            dictionaryMember=True,
            allowTypeAttributes=True,
        )

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
                     | LBRACE RBRACE
        """
        if len(p) == 2:
            p[0] = p[1]
        else:
            assert len(p) == 3  # Must be [] or {}
            if p[1] == "[":
                p[0] = IDLEmptySequenceValue(self.getLocation(p, 1))
            else:
                assert p[1] == "{"
                p[0] = IDLDefaultDictionaryValue(self.getLocation(p, 1))

    def p_DefaultValueNull(self, p):
        """
        DefaultValue : NULL
        """
        p[0] = IDLNullValue(self.getLocation(p, 1))

    def p_DefaultValueUndefined(self, p):
        """
        DefaultValue : UNDEFINED
        """
        p[0] = IDLUndefinedValue(self.getLocation(p, 1))

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
        CallbackRest : IDENTIFIER EQUALS Type LPAREN ArgumentList RPAREN SEMICOLON
        """
        identifier = IDLUnresolvedIdentifier(self.getLocation(p, 1), p[1])
        p[0] = IDLCallback(
            self.getLocation(p, 1),
            self.globalScope(),
            identifier,
            p[3],
            p[5],
            isConstructor=False,
        )

    def p_CallbackConstructorRest(self, p):
        """
        CallbackConstructorRest : CONSTRUCTOR IDENTIFIER EQUALS Type LPAREN ArgumentList RPAREN SEMICOLON
        """
        identifier = IDLUnresolvedIdentifier(self.getLocation(p, 2), p[2])
        p[0] = IDLCallback(
            self.getLocation(p, 2),
            self.globalScope(),
            identifier,
            p[4],
            p[6],
            isConstructor=True,
        )

    def p_ExceptionMembers(self, p):
        """
        ExceptionMembers : ExtendedAttributeList ExceptionMember ExceptionMembers
                         |
        """
        pass

    def p_Typedef(self, p):
        """
        Typedef : TYPEDEF TypeWithExtendedAttributes IDENTIFIER SEMICOLON
        """
        typedef = IDLTypedef(self.getLocation(p, 1), self.globalScope(), p[2], p[3])
        p[0] = typedef

    def p_IncludesStatement(self, p):
        """
        IncludesStatement : ScopedName INCLUDES ScopedName SEMICOLON
        """
        assert p[2] == "includes"
        interface = IDLIdentifierPlaceholder(self.getLocation(p, 1), p[1])
        mixin = IDLIdentifierPlaceholder(self.getLocation(p, 3), p[3])
        p[0] = IDLIncludesStatement(self.getLocation(p, 1), interface, mixin)

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
        p[0] = IDLValue(
            location, BuiltinTypes[IDLBuiltinType.Types.unrestricted_float], p[1]
        )

    def p_ConstValueString(self, p):
        """
        ConstValue : STRING
        """
        location = self.getLocation(p, 1)
        stringType = BuiltinTypes[IDLBuiltinType.Types.domstring]
        p[0] = IDLValue(location, stringType, p[1])

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

    def p_AttributeOrOperationOrMaplikeOrSetlikeOrIterable(self, p):
        """
        AttributeOrOperationOrMaplikeOrSetlikeOrIterable : Attribute
                                                         | Maplike
                                                         | Setlike
                                                         | Iterable
                                                         | AsyncIterable
                                                         | Operation
        """
        p[0] = p[1]

    def p_Iterable(self, p):
        """
        Iterable : ITERABLE LT TypeWithExtendedAttributes GT SEMICOLON
                 | ITERABLE LT TypeWithExtendedAttributes COMMA TypeWithExtendedAttributes GT SEMICOLON
        """
        location = self.getLocation(p, 2)
        identifier = IDLUnresolvedIdentifier(
            location, "__iterable", allowDoubleUnderscore=True
        )
        if len(p) > 6:
            keyType = p[3]
            valueType = p[5]
        else:
            keyType = None
            valueType = p[3]

        p[0] = IDLIterable(location, identifier, keyType, valueType, self.globalScope())

    def p_AsyncIterable(self, p):
        """
        AsyncIterable : ASYNC ITERABLE LT TypeWithExtendedAttributes GT SEMICOLON
                      | ASYNC ITERABLE LT TypeWithExtendedAttributes COMMA TypeWithExtendedAttributes GT SEMICOLON
                      | ASYNC ITERABLE LT TypeWithExtendedAttributes GT LPAREN ArgumentList RPAREN SEMICOLON
                      | ASYNC ITERABLE LT TypeWithExtendedAttributes COMMA TypeWithExtendedAttributes GT LPAREN ArgumentList RPAREN SEMICOLON
        """
        location = self.getLocation(p, 2)
        identifier = IDLUnresolvedIdentifier(
            location, "__iterable", allowDoubleUnderscore=True
        )
        if len(p) == 12:
            keyType = p[4]
            valueType = p[6]
            argList = p[9]
        elif len(p) == 10:
            keyType = None
            valueType = p[4]
            argList = p[7]
        elif len(p) == 9:
            keyType = p[4]
            valueType = p[6]
            argList = []
        else:
            keyType = None
            valueType = p[4]
            argList = []

        p[0] = IDLAsyncIterable(
            location, identifier, keyType, valueType, argList, self.globalScope()
        )

    def p_Setlike(self, p):
        """
        Setlike : ReadOnly SETLIKE LT TypeWithExtendedAttributes GT SEMICOLON
        """
        readonly = p[1]
        maplikeOrSetlikeType = p[2]
        location = self.getLocation(p, 2)
        identifier = IDLUnresolvedIdentifier(
            location, "__setlike", allowDoubleUnderscore=True
        )
        keyType = p[4]
        valueType = keyType
        p[0] = IDLMaplikeOrSetlike(
            location, identifier, maplikeOrSetlikeType, readonly, keyType, valueType
        )

    def p_Maplike(self, p):
        """
        Maplike : ReadOnly MAPLIKE LT TypeWithExtendedAttributes COMMA TypeWithExtendedAttributes GT SEMICOLON
        """
        readonly = p[1]
        maplikeOrSetlikeType = p[2]
        location = self.getLocation(p, 2)
        identifier = IDLUnresolvedIdentifier(
            location, "__maplike", allowDoubleUnderscore=True
        )
        keyType = p[4]
        valueType = p[6]
        p[0] = IDLMaplikeOrSetlike(
            location, identifier, maplikeOrSetlikeType, readonly, keyType, valueType
        )

    def p_AttributeWithQualifier(self, p):
        """
        Attribute : Qualifier AttributeRest
        """
        static = IDLInterfaceMember.Special.Static in p[1]
        stringifier = IDLInterfaceMember.Special.Stringifier in p[1]
        (location, identifier, type, readonly) = p[2]
        p[0] = IDLAttribute(
            location, identifier, type, readonly, static=static, stringifier=stringifier
        )

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
        AttributeRest : ReadOnly ATTRIBUTE TypeWithExtendedAttributes AttributeName SEMICOLON
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
            raise WebIDLError(
                "Duplicate qualifiers are not allowed", [self.getLocation(p, 1)]
            )

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
        deleter = True if IDLMethod.Special.Deleter in p[1] else False
        legacycaller = True if IDLMethod.Special.LegacyCaller in p[1] else False

        if getter or deleter:
            if setter:
                raise WebIDLError(
                    "getter and deleter are incompatible with setter",
                    [self.getLocation(p, 1)],
                )

        (returnType, identifier, arguments) = p[2]

        assert isinstance(returnType, IDLType)

        specialType = IDLMethod.NamedOrIndexed.Neither

        if getter or deleter:
            if len(arguments) != 1:
                raise WebIDLError(
                    "%s has wrong number of arguments"
                    % ("getter" if getter else "deleter"),
                    [self.getLocation(p, 2)],
                )
            argType = arguments[0].type
            if argType == BuiltinTypes[IDLBuiltinType.Types.domstring]:
                specialType = IDLMethod.NamedOrIndexed.Named
            elif argType == BuiltinTypes[IDLBuiltinType.Types.unsigned_long]:
                specialType = IDLMethod.NamedOrIndexed.Indexed
                if deleter:
                    raise WebIDLError(
                        "There is no such thing as an indexed deleter.",
                        [self.getLocation(p, 1)],
                    )
            else:
                raise WebIDLError(
                    "%s has wrong argument type (must be DOMString or UnsignedLong)"
                    % ("getter" if getter else "deleter"),
                    [arguments[0].location],
                )
            if arguments[0].optional or arguments[0].variadic:
                raise WebIDLError(
                    "%s cannot have %s argument"
                    % (
                        "getter" if getter else "deleter",
                        "optional" if arguments[0].optional else "variadic",
                    ),
                    [arguments[0].location],
                )
        if getter:
            if returnType.isUndefined():
                raise WebIDLError(
                    "getter cannot have undefined return type", [self.getLocation(p, 2)]
                )
        if setter:
            if len(arguments) != 2:
                raise WebIDLError(
                    "setter has wrong number of arguments", [self.getLocation(p, 2)]
                )
            argType = arguments[0].type
            if argType == BuiltinTypes[IDLBuiltinType.Types.domstring]:
                specialType = IDLMethod.NamedOrIndexed.Named
            elif argType == BuiltinTypes[IDLBuiltinType.Types.unsigned_long]:
                specialType = IDLMethod.NamedOrIndexed.Indexed
            else:
                raise WebIDLError(
                    "settter has wrong argument type (must be DOMString or UnsignedLong)",
                    [arguments[0].location],
                )
            if arguments[0].optional or arguments[0].variadic:
                raise WebIDLError(
                    "setter cannot have %s argument"
                    % ("optional" if arguments[0].optional else "variadic"),
                    [arguments[0].location],
                )
            if arguments[1].optional or arguments[1].variadic:
                raise WebIDLError(
                    "setter cannot have %s argument"
                    % ("optional" if arguments[1].optional else "variadic"),
                    [arguments[1].location],
                )

        if stringifier:
            if len(arguments) != 0:
                raise WebIDLError(
                    "stringifier has wrong number of arguments",
                    [self.getLocation(p, 2)],
                )
            if not returnType.isDOMString():
                raise WebIDLError(
                    "stringifier must have DOMString return type",
                    [self.getLocation(p, 2)],
                )

        # identifier might be None.  This is only permitted for special methods.
        if not identifier:
            if (
                not getter
                and not setter
                and not deleter
                and not legacycaller
                and not stringifier
            ):
                raise WebIDLError(
                    "Identifier required for non-special methods",
                    [self.getLocation(p, 2)],
                )

            location = BuiltinLocation("<auto-generated-identifier>")
            identifier = IDLUnresolvedIdentifier(
                location,
                "__%s%s%s%s%s%s"
                % (
                    "named"
                    if specialType == IDLMethod.NamedOrIndexed.Named
                    else "indexed"
                    if specialType == IDLMethod.NamedOrIndexed.Indexed
                    else "",
                    "getter" if getter else "",
                    "setter" if setter else "",
                    "deleter" if deleter else "",
                    "legacycaller" if legacycaller else "",
                    "stringifier" if stringifier else "",
                ),
                allowDoubleUnderscore=True,
            )

        method = IDLMethod(
            self.getLocation(p, 2),
            identifier,
            returnType,
            arguments,
            static=static,
            getter=getter,
            setter=setter,
            deleter=deleter,
            specialType=specialType,
            legacycaller=legacycaller,
            stringifier=stringifier,
        )
        p[0] = method

    def p_Stringifier(self, p):
        """
        Operation : STRINGIFIER SEMICOLON
        """
        identifier = IDLUnresolvedIdentifier(
            BuiltinLocation("<auto-generated-identifier>"),
            "__stringifier",
            allowDoubleUnderscore=True,
        )
        method = IDLMethod(
            self.getLocation(p, 1),
            identifier,
            returnType=BuiltinTypes[IDLBuiltinType.Types.domstring],
            arguments=[],
            stringifier=True,
        )
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
        OperationRest : Type OptionalIdentifier LPAREN ArgumentList RPAREN SEMICOLON
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
        Argument : ExtendedAttributeList ArgumentRest
        """
        p[0] = p[2]
        p[0].addExtendedAttributes(p[1])

    def p_ArgumentRestOptional(self, p):
        """
        ArgumentRest : OPTIONAL TypeWithExtendedAttributes ArgumentName Default
        """
        t = p[2]
        assert isinstance(t, IDLType)
        # Arg names can be reserved identifiers
        identifier = IDLUnresolvedIdentifier(
            self.getLocation(p, 3), p[3], allowForbidden=True
        )

        defaultValue = p[4]

        # We can't test t.isAny() here and give it a default value as needed,
        # since at this point t is not a fully resolved type yet (e.g. it might
        # be a typedef).  We'll handle the 'any' case in IDLArgument.complete.

        p[0] = IDLArgument(
            self.getLocation(p, 3), identifier, t, True, defaultValue, False
        )

    def p_ArgumentRest(self, p):
        """
        ArgumentRest : Type Ellipsis ArgumentName
        """
        t = p[1]
        assert isinstance(t, IDLType)
        # Arg names can be reserved identifiers
        identifier = IDLUnresolvedIdentifier(
            self.getLocation(p, 3), p[3], allowForbidden=True
        )

        variadic = p[2]

        # We can't test t.isAny() here and give it a default value as needed,
        # since at this point t is not a fully resolved type yet (e.g. it might
        # be a typedef).  We'll handle the 'any' case in IDLArgument.complete.

        # variadic implies optional
        # Any attributes that precede this may apply to the type, so
        # we configure the argument to forward type attributes down instead of producing
        # a parse error
        p[0] = IDLArgument(
            self.getLocation(p, 3),
            identifier,
            t,
            variadic,
            None,
            variadic,
            allowTypeAttributes=True,
        )

    def p_ArgumentName(self, p):
        """
        ArgumentName : IDENTIFIER
                     | ArgumentNameKeyword
        """
        p[0] = p[1]

    def p_ArgumentNameKeyword(self, p):
        """
        ArgumentNameKeyword : ASYNC
                            | ATTRIBUTE
                            | CALLBACK
                            | CONST
                            | CONSTRUCTOR
                            | DELETER
                            | DICTIONARY
                            | ENUM
                            | EXCEPTION
                            | GETTER
                            | INCLUDES
                            | INHERIT
                            | INTERFACE
                            | ITERABLE
                            | LEGACYCALLER
                            | MAPLIKE
                            | MIXIN
                            | NAMESPACE
                            | PARTIAL
                            | READONLY
                            | REQUIRED
                            | SERIALIZER
                            | SETLIKE
                            | SETTER
                            | STATIC
                            | STRINGIFIER
                            | TYPEDEF
                            | UNRESTRICTED
        """
        p[0] = p[1]

    def p_AttributeName(self, p):
        """
        AttributeName : IDENTIFIER
                      | AttributeNameKeyword
        """
        p[0] = p[1]

    def p_AttributeNameKeyword(self, p):
        """
        AttributeNameKeyword : ASYNC
                             | REQUIRED
        """
        p[0] = p[1]

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
                          | ExtendedAttributeWildcard
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
              | ASTERISK
              | DOMSTRING
              | BYTESTRING
              | USVSTRING
              | UTF8STRING
              | JSSTRING
              | PROMISE
              | ANY
              | BOOLEAN
              | BYTE
              | DOUBLE
              | FALSE
              | FLOAT
              | LONG
              | NULL
              | OBJECT
              | OCTET
              | OR
              | OPTIONAL
              | RECORD
              | SEQUENCE
              | SHORT
              | SYMBOL
              | TRUE
              | UNSIGNED
              | UNDEFINED
              | ArgumentNameKeyword
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
        Type : UnionType Null
        """
        p[0] = self.handleNullable(p[1], p[2])

    def p_TypeWithExtendedAttributes(self, p):
        """
        TypeWithExtendedAttributes : ExtendedAttributeList Type
        """
        p[0] = p[2].withExtendedAttributes(p[1])

    def p_SingleTypeDistinguishableType(self, p):
        """
        SingleType : DistinguishableType
        """
        p[0] = p[1]

    def p_SingleTypeAnyType(self, p):
        """
        SingleType : ANY
        """
        p[0] = BuiltinTypes[IDLBuiltinType.Types.any]

    def p_SingleTypePromiseType(self, p):
        """
        SingleType : PROMISE LT Type GT
        """
        p[0] = IDLPromiseType(self.getLocation(p, 1), p[3])

    def p_UnionType(self, p):
        """
        UnionType : LPAREN UnionMemberType OR UnionMemberType UnionMemberTypes RPAREN
        """
        types = [p[2], p[4]]
        types.extend(p[5])
        p[0] = IDLUnionType(self.getLocation(p, 1), types)

    def p_UnionMemberTypeDistinguishableType(self, p):
        """
        UnionMemberType : ExtendedAttributeList DistinguishableType
        """
        p[0] = p[2].withExtendedAttributes(p[1])

    def p_UnionMemberType(self, p):
        """
        UnionMemberType : UnionType Null
        """
        p[0] = self.handleNullable(p[1], p[2])

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

    def p_DistinguishableType(self, p):
        """
        DistinguishableType : PrimitiveType Null
                            | ARRAYBUFFER Null
                            | OBJECT Null
                            | UNDEFINED Null
        """
        if p[1] == "object":
            type = BuiltinTypes[IDLBuiltinType.Types.object]
        elif p[1] == "ArrayBuffer":
            type = BuiltinTypes[IDLBuiltinType.Types.ArrayBuffer]
        elif p[1] == "undefined":
            type = BuiltinTypes[IDLBuiltinType.Types.undefined]
        else:
            type = BuiltinTypes[p[1]]

        p[0] = self.handleNullable(type, p[2])

    def p_DistinguishableTypeStringType(self, p):
        """
        DistinguishableType : StringType Null
        """
        p[0] = self.handleNullable(p[1], p[2])

    def p_DistinguishableTypeSequenceType(self, p):
        """
        DistinguishableType : SEQUENCE LT TypeWithExtendedAttributes GT Null
        """
        innerType = p[3]
        type = IDLSequenceType(self.getLocation(p, 1), innerType)
        p[0] = self.handleNullable(type, p[5])

    def p_DistinguishableTypeRecordType(self, p):
        """
        DistinguishableType : RECORD LT StringType COMMA TypeWithExtendedAttributes GT Null
        """
        keyType = p[3]
        valueType = p[5]
        type = IDLRecordType(self.getLocation(p, 1), keyType, valueType)
        p[0] = self.handleNullable(type, p[7])

    def p_DistinguishableTypeObservableArrayType(self, p):
        """
        DistinguishableType : OBSERVABLEARRAY LT TypeWithExtendedAttributes GT Null
        """
        innerType = p[3]
        type = IDLObservableArrayType(self.getLocation(p, 1), innerType)
        p[0] = self.handleNullable(type, p[5])

    def p_DistinguishableTypeScopedName(self, p):
        """
        DistinguishableType : ScopedName Null
        """
        assert isinstance(p[1], IDLUnresolvedIdentifier)

        if p[1].name == "Promise":
            raise WebIDLError(
                "Promise used without saying what it's " "parametrized over",
                [self.getLocation(p, 1)],
            )

        type = None

        try:
            if self.globalScope()._lookupIdentifier(p[1]):
                obj = self.globalScope()._lookupIdentifier(p[1])
                assert not obj.isType()
                if obj.isTypedef():
                    type = IDLTypedefType(
                        self.getLocation(p, 1), obj.innerType, obj.identifier.name
                    )
                elif obj.isCallback() and not obj.isInterface():
                    type = IDLCallbackType(obj.location, obj)
                else:
                    type = IDLWrapperType(self.getLocation(p, 1), p[1])
                p[0] = self.handleNullable(type, p[2])
                return
        except Exception:
            pass

        type = IDLUnresolvedType(self.getLocation(p, 1), p[1])
        p[0] = self.handleNullable(type, p[2])

    def p_ConstType(self, p):
        """
        ConstType : PrimitiveType
        """
        p[0] = BuiltinTypes[p[1]]

    def p_ConstTypeIdentifier(self, p):
        """
        ConstType : IDENTIFIER
        """
        identifier = IDLUnresolvedIdentifier(self.getLocation(p, 1), p[1])

        p[0] = IDLUnresolvedType(self.getLocation(p, 1), identifier)

    def p_PrimitiveTypeUint(self, p):
        """
        PrimitiveType : UnsignedIntegerType
        """
        p[0] = p[1]

    def p_PrimitiveTypeBoolean(self, p):
        """
        PrimitiveType : BOOLEAN
        """
        p[0] = IDLBuiltinType.Types.boolean

    def p_PrimitiveTypeByte(self, p):
        """
        PrimitiveType : BYTE
        """
        p[0] = IDLBuiltinType.Types.byte

    def p_PrimitiveTypeOctet(self, p):
        """
        PrimitiveType : OCTET
        """
        p[0] = IDLBuiltinType.Types.octet

    def p_PrimitiveTypeFloat(self, p):
        """
        PrimitiveType : FLOAT
        """
        p[0] = IDLBuiltinType.Types.float

    def p_PrimitiveTypeUnrestictedFloat(self, p):
        """
        PrimitiveType : UNRESTRICTED FLOAT
        """
        p[0] = IDLBuiltinType.Types.unrestricted_float

    def p_PrimitiveTypeDouble(self, p):
        """
        PrimitiveType : DOUBLE
        """
        p[0] = IDLBuiltinType.Types.double

    def p_PrimitiveTypeUnrestictedDouble(self, p):
        """
        PrimitiveType : UNRESTRICTED DOUBLE
        """
        p[0] = IDLBuiltinType.Types.unrestricted_double

    def p_StringType(self, p):
        """
        StringType : BuiltinStringType
        """
        p[0] = BuiltinTypes[p[1]]

    def p_BuiltinStringTypeDOMString(self, p):
        """
        BuiltinStringType : DOMSTRING
        """
        p[0] = IDLBuiltinType.Types.domstring

    def p_BuiltinStringTypeBytestring(self, p):
        """
        BuiltinStringType : BYTESTRING
        """
        p[0] = IDLBuiltinType.Types.bytestring

    def p_BuiltinStringTypeUSVString(self, p):
        """
        BuiltinStringType : USVSTRING
        """
        p[0] = IDLBuiltinType.Types.usvstring

    def p_BuiltinStringTypeUTF8String(self, p):
        """
        BuiltinStringType : UTF8STRING
        """
        p[0] = IDLBuiltinType.Types.utf8string

    def p_BuiltinStringTypeJSString(self, p):
        """
        BuiltinStringType : JSSTRING
        """
        p[0] = IDLBuiltinType.Types.jsstring

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

    def p_Null(self, p):
        """
        Null : QUESTIONMARK
             |
        """
        if len(p) > 1:
            p[0] = self.getLocation(p, 1)
        else:
            p[0] = None

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

    def p_ExtendedAttributeWildcard(self, p):
        """
        ExtendedAttributeWildcard : IDENTIFIER EQUALS ASTERISK
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
        # This is only used for identifier-list-valued extended attributes, and if
        # we're going to restrict to IDENTIFIER here we should at least allow
        # escaping with leading '_' as usual for identifiers.
        ident = p[1]
        if ident[0] == "_":
            ident = ident[1:]
        idents.insert(0, ident)
        p[0] = idents

    def p_IdentifiersList(self, p):
        """
        Identifiers : COMMA IDENTIFIER Identifiers
        """
        idents = list(p[3])
        # This is only used for identifier-list-valued extended attributes, and if
        # we're going to restrict to IDENTIFIER here we should at least allow
        # escaping with leading '_' as usual for identifiers.
        ident = p[2]
        if ident[0] == "_":
            ident = ident[1:]
        idents.insert(0, ident)
        p[0] = idents

    def p_IdentifiersEmpty(self, p):
        """
        Identifiers :
        """
        p[0] = []

    def p_error(self, p):
        if not p:
            raise WebIDLError(
                (
                    "Syntax Error at end of file. Possibly due to "
                    "missing semicolon(;), braces(}) or both"
                ),
                [self._filename],
            )
        else:
            raise WebIDLError(
                "invalid syntax",
                [Location(self.lexer, p.lineno, p.lexpos, self._filename)],
            )

    def __init__(self, outputdir="", lexer=None):
        Tokenizer.__init__(self, outputdir, lexer)

        logger = SqueakyCleanLogger()
        try:
            self.parser = yacc.yacc(
                module=self,
                outputdir=outputdir,
                errorlog=logger,
                debug=False,
                write_tables=False,
                # Pickling the grammar is a speedup in
                # some cases (older Python?) but a
                # significant slowdown in others.
                # We're not pickling for now, until it
                # becomes a speedup again.
                # , picklefile='WebIDLGrammar.pkl'
            )
        finally:
            logger.reportGrammarErrors()

        self._globalScope = IDLScope(BuiltinLocation("<Global Scope>"), None, None)
        self._installBuiltins(self._globalScope)
        self._productions = []

        self._filename = "<builtin>"
        self.lexer.input(Parser._builtins)
        self._filename = None

        self.parser.parse(lexer=self.lexer, tracking=True)

    def _installBuiltins(self, scope):
        assert isinstance(scope, IDLScope)

        # range omits the last value.
        for x in range(
            IDLBuiltinType.Types.ArrayBuffer, IDLBuiltinType.Types.Float64Array + 1
        ):
            builtin = BuiltinTypes[x]
            name = builtin.name
            IDLTypedef(BuiltinLocation("<builtin type>"), scope, builtin, name)

    @staticmethod
    def handleNullable(type, questionMarkLocation):
        if questionMarkLocation is not None:
            type = IDLNullableType(questionMarkLocation, type)

        return type

    def parse(self, t, filename=None):
        self.lexer.input(t)

        # for tok in iter(self.lexer.token, None):
        #    print tok

        self._filename = filename
        self._productions.extend(self.parser.parse(lexer=self.lexer, tracking=True))
        self._filename = None

    def finish(self):
        # If we have interfaces that are iterable, create their
        # iterator interfaces and add them to the productions array.
        interfaceStatements = []
        for p in self._productions:
            if isinstance(p, IDLInterface):
                interfaceStatements.append(p)

        for iface in interfaceStatements:
            iterable = None
            # We haven't run finish() on the interface yet, so we don't know
            # whether our interface is maplike/setlike/iterable or not. This
            # means we have to loop through the members to see if we have an
            # iterable member.
            for m in iface.members:
                if isinstance(m, (IDLIterable, IDLAsyncIterable)):
                    iterable = m
                    break
            if iterable and (iterable.isPairIterator() or iterable.isAsyncIterable()):

                def simpleExtendedAttr(str):
                    return IDLExtendedAttribute(iface.location, (str,))

                if isinstance(iterable, IDLAsyncIterable):
                    nextReturnType = IDLPromiseType(
                        iterable.location, BuiltinTypes[IDLBuiltinType.Types.any]
                    )
                else:
                    nextReturnType = BuiltinTypes[IDLBuiltinType.Types.object]
                nextMethod = IDLMethod(
                    iterable.location,
                    IDLUnresolvedIdentifier(iterable.location, "next"),
                    nextReturnType,
                    [],
                )
                nextMethod.addExtendedAttributes([simpleExtendedAttr("Throws")])

                methods = [nextMethod]

                if iterable.getExtendedAttribute("GenerateReturnMethod"):
                    assert isinstance(iterable, IDLAsyncIterable)

                    returnMethod = IDLMethod(
                        iterable.location,
                        IDLUnresolvedIdentifier(iterable.location, "return"),
                        IDLPromiseType(
                            iterable.location, BuiltinTypes[IDLBuiltinType.Types.any]
                        ),
                        [
                            IDLArgument(
                                iterable.location,
                                IDLUnresolvedIdentifier(
                                    BuiltinLocation("<auto-generated-identifier>"),
                                    "value",
                                ),
                                BuiltinTypes[IDLBuiltinType.Types.any],
                                optional=True,
                            ),
                        ],
                    )
                    returnMethod.addExtendedAttributes([simpleExtendedAttr("Throws")])
                    methods.append(returnMethod)

                if iterable.isIterable():
                    itr_suffix = "Iterator"
                else:
                    itr_suffix = "AsyncIterator"
                itr_ident = IDLUnresolvedIdentifier(
                    iface.location, iface.identifier.name + itr_suffix
                )
                if iterable.isIterable():
                    classNameOverride = iface.identifier.name + " Iterator"
                elif iterable.isAsyncIterable():
                    classNameOverride = iface.identifier.name + " AsyncIterator"
                itr_iface = IDLInterface(
                    iface.location,
                    self.globalScope(),
                    itr_ident,
                    None,
                    methods,
                    isKnownNonPartial=True,
                    classNameOverride=classNameOverride,
                )
                itr_iface.addExtendedAttributes(
                    [simpleExtendedAttr("LegacyNoInterfaceObject")]
                )
                # Make sure the exposure set for the iterator interface is the
                # same as the exposure set for the iterable interface, because
                # we're going to generate methods on the iterable that return
                # instances of the iterator.
                itr_iface._exposureGlobalNames = set(iface._exposureGlobalNames)
                # Always append generated iterable interfaces after the
                # interface they're a member of, otherwise nativeType generation
                # won't work correctly.
                if iterable.isIterable():
                    itr_iface.iterableInterface = iface
                else:
                    itr_iface.asyncIterableInterface = iface
                self._productions.append(itr_iface)
                iterable.iteratorType = IDLWrapperType(iface.location, itr_iface)
            if not iterable:
                # We haven't run finish() on the interface yet, so we don't know
                # whether our interface is maplike/setlike/iterable or not. This
                # means we have to loop through the members to see if we have an
                # iterable member.
                for m in iface.members:
                    if isinstance(m, IDLMaplikeOrSetlike):
                        iterable = m
                        break
                if iterable and (iterable.isSetlike() or iterable.isMaplike()):

                    def simpleExtendedAttr(str):
                        return IDLExtendedAttribute(iface.location, (str,))

                    if isinstance(iterable, IDLAsyncIterable):
                        nextReturnType = IDLPromiseType(
                            iterable.location, BuiltinTypes[IDLBuiltinType.Types.any]
                        )
                    else:
                        nextReturnType = BuiltinTypes[IDLBuiltinType.Types.object]
                    nextMethod = IDLMethod(
                        iterable.location,
                        IDLUnresolvedIdentifier(iterable.location, "next"),
                        nextReturnType,
                        [],
                    )
                    nextMethod.addExtendedAttributes([simpleExtendedAttr("Throws")])

                    methods = [nextMethod]

                    if iterable.isSetlike():
                        itr_suffix = "Setlike"
                    else:
                        itr_suffix = "Maplike"
                    itr_ident = IDLUnresolvedIdentifier(
                        iface.location, iface.identifier.name + itr_suffix
                    )
                    classNameOverride = iface.identifier.name + " " + itr_suffix
                    itr_iface = IDLInterface(
                        iface.location,
                        self.globalScope(),
                        itr_ident,
                        None,
                        methods,
                        isKnownNonPartial=True,
                        classNameOverride=classNameOverride,
                    )
                    itr_iface.addExtendedAttributes(
                        [simpleExtendedAttr("LegacyNoInterfaceObject")]
                    )
                    # Make sure the exposure set for the iterator interface is the
                    # same as the exposure set for the iterable interface, because
                    # we're going to generate methods on the iterable that return
                    # instances of the iterator.
                    itr_iface._exposureGlobalNames = set(iface._exposureGlobalNames)
                    # Always append generated iterable interfaces after the
                    # interface they're a member of, otherwise nativeType generation
                    # won't work correctly.
                    itr_iface.iterableInterface = iface
                    self._productions.append(itr_iface)
                    iterable.iteratorType = IDLWrapperType(iface.location, itr_iface)

        # Make sure we finish IDLIncludesStatements before we finish the
        # IDLInterfaces.
        # XXX khuey hates this bit and wants to nuke it from orbit.
        includesStatements = [
            p for p in self._productions if isinstance(p, IDLIncludesStatement)
        ]
        otherStatements = [
            p for p in self._productions if not isinstance(p, IDLIncludesStatement)
        ]
        for production in includesStatements:
            production.finish(self.globalScope())
        for production in otherStatements:
            production.finish(self.globalScope())

        # Do any post-finish validation we need to do
        for production in self._productions:
            production.validate()

        # De-duplicate self._productions, without modifying its order.
        result = dict.fromkeys(self._productions)
        return list(result.keys())

    def reset(self):
        return Parser(lexer=self.lexer)

    # Builtin IDL defined by WebIDL
    _builtins = """
        typedef (ArrayBufferView or ArrayBuffer) BufferSource;
    """


def main():
    # Parse arguments.
    from optparse import OptionParser

    usageString = "usage: %prog [options] files"
    o = OptionParser(usage=usageString)
    o.add_option(
        "--cachedir",
        dest="cachedir",
        default=None,
        help="Directory in which to cache lex/parse tables.",
    )
    o.add_option(
        "--verbose-errors",
        action="store_true",
        default=False,
        help="When an error happens, display the Python traceback.",
    )
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
            f = open(fullPath, "rb")
            lines = f.readlines()
            f.close()
            print(fullPath)
            parser.parse("".join(lines), fullPath)
        parser.finish()
    except WebIDLError as e:
        if options.verbose_errors:
            traceback.print_exc()
        else:
            print(e)


if __name__ == "__main__":
    main()
