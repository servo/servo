# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import os

from WebIDL import IDLExternalInterface, IDLWrapperType, WebIDLError


class Configuration:
    """
    Represents global configuration state based on IDL parse data and
    the configuration file.
    """
    def __init__(self, filename, parseData):
        # Read the configuration file.
        glbl = {}
        execfile(filename, glbl)
        config = glbl['DOMInterfaces']

        # Build descriptors for all the interfaces we have in the parse data.
        # This allows callers to specify a subset of interfaces by filtering
        # |parseData|.
        self.descriptors = []
        self.interfaces = {}
        self.maxProtoChainLength = 0
        for thing in parseData:
            # Servo does not support external interfaces.
            if isinstance(thing, IDLExternalInterface):
                raise WebIDLError("Servo does not support external interfaces.",
                                  [thing.location])

            assert not thing.isType()

            if not thing.isInterface() and not thing.isNamespace():
                continue

            iface = thing
            self.interfaces[iface.identifier.name] = iface
            if iface.identifier.name not in config:
                # Completely skip consequential interfaces with no descriptor
                # if they have no interface object because chances are we
                # don't need to do anything interesting with them.
                if iface.isConsequential() and not iface.hasInterfaceObject():
                    continue
                entry = {}
            else:
                entry = config[iface.identifier.name]
            if not isinstance(entry, list):
                assert isinstance(entry, dict)
                entry = [entry]
            self.descriptors.extend(
                [Descriptor(self, iface, x) for x in entry])

        # Mark the descriptors for which only a single nativeType implements
        # an interface.
        for descriptor in self.descriptors:
            intefaceName = descriptor.interface.identifier.name
            otherDescriptors = [d for d in self.descriptors
                                if d.interface.identifier.name == intefaceName]
            descriptor.uniqueImplementation = len(otherDescriptors) == 1

        self.enums = [e for e in parseData if e.isEnum()]
        self.typedefs = [e for e in parseData if e.isTypedef()]
        self.dictionaries = [d for d in parseData if d.isDictionary()]
        self.callbacks = [c for c in parseData if
                          c.isCallback() and not c.isInterface()]

        # Keep the descriptor list sorted for determinism.
        self.descriptors.sort(lambda x, y: cmp(x.name, y.name))

    def getInterface(self, ifname):
        return self.interfaces[ifname]

    def getDescriptors(self, **filters):
        """Gets the descriptors that match the given filters."""
        curr = self.descriptors
        for key, val in filters.iteritems():
            if key == 'webIDLFile':
                getter = lambda x: x.interface.filename()
            elif key == 'hasInterfaceObject':
                getter = lambda x: x.interface.hasInterfaceObject()
            elif key == 'isCallback':
                getter = lambda x: x.interface.isCallback()
            elif key == 'isNamespace':
                getter = lambda x: x.interface.isNamespace()
            elif key == 'isJSImplemented':
                getter = lambda x: x.interface.isJSImplemented()
            elif key == 'isGlobal':
                getter = lambda x: x.isGlobal()
            elif key == 'isInline':
                getter = lambda x: x.interface.getExtendedAttribute('Inline') is not None
            elif key == 'isExposedConditionally':
                getter = lambda x: x.interface.isExposedConditionally()
            elif key == 'isIteratorInterface':
                getter = lambda x: x.interface.isIteratorInterface()
            else:
                getter = lambda x: getattr(x, key)
            curr = filter(lambda x: getter(x) == val, curr)
        return curr

    def getEnums(self, webIDLFile):
        return filter(lambda e: e.filename() == webIDLFile, self.enums)

    def getTypedefs(self, webIDLFile):
        return filter(lambda e: e.filename() == webIDLFile, self.typedefs)

    @staticmethod
    def _filterForFile(items, webIDLFile=""):
        """Gets the items that match the given filters."""
        if not webIDLFile:
            return items

        return filter(lambda x: x.filename() == webIDLFile, items)

    def getDictionaries(self, webIDLFile=""):
        return self._filterForFile(self.dictionaries, webIDLFile=webIDLFile)

    def getCallbacks(self, webIDLFile=""):
        return self._filterForFile(self.callbacks, webIDLFile=webIDLFile)

    def getDescriptor(self, interfaceName):
        """
        Gets the appropriate descriptor for the given interface name.
        """
        iface = self.getInterface(interfaceName)
        descriptors = self.getDescriptors(interface=iface)

        # We should have exactly one result.
        if len(descriptors) != 1:
            raise NoSuchDescriptorError("For " + interfaceName + " found " +
                                        str(len(descriptors)) + " matches")
        return descriptors[0]

    def getDescriptorProvider(self):
        """
        Gets a descriptor provider that can provide descriptors as needed.
        """
        return DescriptorProvider(self)


class NoSuchDescriptorError(TypeError):
    def __init__(self, str):
        TypeError.__init__(self, str)


class DescriptorProvider:
    """
    A way of getting descriptors for interface names
    """
    def __init__(self, config):
        self.config = config

    def getDescriptor(self, interfaceName):
        """
        Gets the appropriate descriptor for the given interface name given the
        context of the current descriptor.
        """
        return self.config.getDescriptor(interfaceName)


def MemberIsUnforgeable(member, descriptor):
    return ((member.isAttr() or member.isMethod()) and
            not member.isStatic() and
            (member.isUnforgeable() or
             bool(descriptor.interface.getExtendedAttribute("Unforgeable"))))


class Descriptor(DescriptorProvider):
    """
    Represents a single descriptor for an interface. See Bindings.conf.
    """
    def __init__(self, config, interface, desc):
        DescriptorProvider.__init__(self, config)
        self.interface = interface

        if not self.isExposedConditionally():
            if interface.parent and interface.parent.isExposedConditionally():
                raise TypeError("%s is not conditionally exposed but inherits from "
                                "%s which is" %
                                (interface.identifier.name, interface.parent.identifier.name))

        # Read the desc, and fill in the relevant defaults.
        ifaceName = self.interface.identifier.name
        nativeTypeDefault = ifaceName

        # For generated iterator interfaces for other iterable interfaces, we
        # just use IterableIterator as the native type, templated on the
        # nativeType of the iterable interface. That way we can have a
        # templated implementation for all the duplicated iterator
        # functionality.
        if self.interface.isIteratorInterface():
            itrName = self.interface.iterableInterface.identifier.name
            itrDesc = self.getDescriptor(itrName)
            nativeTypeDefault = iteratorNativeType(itrDesc)

        typeName = desc.get('nativeType', nativeTypeDefault)

        spiderMonkeyInterface = desc.get('spiderMonkeyInterface', False)

        # Callback and SpiderMonkey types do not use JS smart pointers, so we should not use the
        # built-in rooting mechanisms for them.
        if spiderMonkeyInterface:
            self.returnType = 'Rc<%s>' % typeName
            self.argumentType = '&%s' % typeName
            self.nativeType = typeName
            pathDefault = 'dom::types::%s' % typeName
        elif self.interface.isCallback():
            ty = 'dom::bindings::codegen::Bindings::%sBinding::%s' % (ifaceName, ifaceName)
            pathDefault = ty
            self.returnType = "Rc<%s>" % ty
            self.argumentType = "???"
            self.nativeType = ty
        else:
            self.returnType = "Root<%s>" % typeName
            self.argumentType = "&%s" % typeName
            self.nativeType = "*const %s" % typeName
            if self.interface.isIteratorInterface():
                pathDefault = 'dom::bindings::iterable::IterableIterator'
            else:
                pathDefault = 'dom::types::%s' % MakeNativeName(typeName)

        self.concreteType = typeName
        self.register = desc.get('register', True)
        self.path = desc.get('path', pathDefault)
        self.bindingPath = 'dom::bindings::codegen::Bindings::%s' % ('::'.join([ifaceName + 'Binding'] * 2))
        self.outerObjectHook = desc.get('outerObjectHook', 'None')
        self.proxy = False
        self.weakReferenceable = desc.get('weakReferenceable', False)

        # If we're concrete, we need to crawl our ancestor interfaces and mark
        # them as having a concrete descendant.
        self.concrete = (not self.interface.isCallback() and
                         not self.interface.isNamespace() and
                         not self.interface.getExtendedAttribute("Abstract") and
                         not self.interface.getExtendedAttribute("Inline") and
                         not spiderMonkeyInterface)
        self.hasUnforgeableMembers = (self.concrete and
                                      any(MemberIsUnforgeable(m, self) for m in
                                          self.interface.members))

        self.operations = {
            'IndexedGetter': None,
            'IndexedSetter': None,
            'IndexedDeleter': None,
            'NamedGetter': None,
            'NamedSetter': None,
            'NamedDeleter': None,
            'Stringifier': None,
        }

        def addOperation(operation, m):
            if not self.operations[operation]:
                self.operations[operation] = m

        # Since stringifiers go on the prototype, we only need to worry
        # about our own stringifier, not those of our ancestor interfaces.
        for m in self.interface.members:
            if m.isMethod() and m.isStringifier():
                addOperation('Stringifier', m)

        if self.concrete:
            iface = self.interface
            while iface:
                for m in iface.members:
                    if not m.isMethod():
                        continue

                    def addIndexedOrNamedOperation(operation, m):
                        self.proxy = True
                        if m.isIndexed():
                            operation = 'Indexed' + operation
                        else:
                            assert m.isNamed()
                            operation = 'Named' + operation
                        addOperation(operation, m)

                    if m.isGetter():
                        addIndexedOrNamedOperation('Getter', m)
                    if m.isSetter():
                        addIndexedOrNamedOperation('Setter', m)
                    if m.isCreator():
                        addIndexedOrNamedOperation('Creator', m)
                    if m.isDeleter():
                        addIndexedOrNamedOperation('Deleter', m)

                iface = iface.parent
                if iface:
                    iface.setUserData('hasConcreteDescendant', True)

            if self.proxy:
                iface = self.interface
                while iface.parent:
                    iface = iface.parent
                    iface.setUserData('hasProxyDescendant', True)

        self.name = interface.identifier.name

        # self.extendedAttributes is a dict of dicts, keyed on
        # all/getterOnly/setterOnly and then on member name. Values are an
        # array of extended attributes.
        self.extendedAttributes = {'all': {}, 'getterOnly': {}, 'setterOnly': {}}

        def addExtendedAttribute(attribute, config):
            def add(key, members, attribute):
                for member in members:
                    self.extendedAttributes[key].setdefault(member, []).append(attribute)

            if isinstance(config, dict):
                for key in ['all', 'getterOnly', 'setterOnly']:
                    add(key, config.get(key, []), attribute)
            elif isinstance(config, list):
                add('all', config, attribute)
            else:
                assert isinstance(config, str)
                if config == '*':
                    iface = self.interface
                    while iface:
                        add('all', map(lambda m: m.name, iface.members), attribute)
                        iface = iface.parent
                else:
                    add('all', [config], attribute)

        self._binaryNames = desc.get('binaryNames', {})
        self._binaryNames.setdefault('__legacycaller', 'LegacyCall')
        self._binaryNames.setdefault('__stringifier', 'Stringifier')

        self._internalNames = desc.get('internalNames', {})

        for member in self.interface.members:
            if not member.isAttr() and not member.isMethod():
                continue
            binaryName = member.getExtendedAttribute("BinaryName")
            if binaryName:
                assert isinstance(binaryName, list)
                assert len(binaryName) == 1
                self._binaryNames.setdefault(member.identifier.name,
                                             binaryName[0])
            self._internalNames.setdefault(member.identifier.name,
                                           member.identifier.name.replace('-', '_'))

        # Build the prototype chain.
        self.prototypeChain = []
        parent = interface
        while parent:
            self.prototypeChain.insert(0, parent.identifier.name)
            parent = parent.parent
        self.prototypeDepth = len(self.prototypeChain) - 1
        config.maxProtoChainLength = max(config.maxProtoChainLength,
                                         len(self.prototypeChain))

    def binaryNameFor(self, name):
        return self._binaryNames.get(name, name)

    def internalNameFor(self, name):
        return self._internalNames.get(name, name)

    def getExtendedAttributes(self, member, getter=False, setter=False):
        def maybeAppendInfallibleToAttrs(attrs, throws):
            if throws is None:
                attrs.append("infallible")
            elif throws is True:
                pass
            else:
                raise TypeError("Unknown value for 'Throws'")

        name = member.identifier.name
        if member.isMethod():
            attrs = self.extendedAttributes['all'].get(name, [])
            throws = member.getExtendedAttribute("Throws")
            maybeAppendInfallibleToAttrs(attrs, throws)
            return attrs

        assert member.isAttr()
        assert bool(getter) != bool(setter)
        key = 'getterOnly' if getter else 'setterOnly'
        attrs = self.extendedAttributes['all'].get(name, []) + self.extendedAttributes[key].get(name, [])
        throws = member.getExtendedAttribute("Throws")
        if throws is None:
            throwsAttr = "GetterThrows" if getter else "SetterThrows"
            throws = member.getExtendedAttribute(throwsAttr)
        maybeAppendInfallibleToAttrs(attrs, throws)
        return attrs

    def getParentName(self):
        parent = self.interface.parent
        while parent:
            if not parent.getExtendedAttribute("Inline"):
                return parent.identifier.name
            parent = parent.parent
        return None

    def hasDescendants(self):
        return (self.interface.getUserData("hasConcreteDescendant", False) or
                self.interface.getUserData("hasProxyDescendant", False))

    def shouldHaveGetConstructorObjectMethod(self):
        assert self.interface.hasInterfaceObject()
        if self.interface.getExtendedAttribute("Inline"):
            return False
        return (self.interface.isCallback() or self.interface.isNamespace() or
                self.hasDescendants() or self.interface.getExtendedAttribute("HTMLConstructor"))

    def shouldCacheConstructor(self):
        return self.hasDescendants() or self.interface.getExtendedAttribute("HTMLConstructor")

    def isExposedConditionally(self):
        return self.interface.isExposedConditionally()

    def isGlobal(self):
        """
        Returns true if this is the primary interface for a global object
        of some sort.
        """
        return bool(self.interface.getExtendedAttribute("Global") or
                    self.interface.getExtendedAttribute("PrimaryGlobal"))


# Some utility methods


def MakeNativeName(name):
    return name[0].upper() + name[1:]


def getModuleFromObject(object):
    return ('dom::bindings::codegen::Bindings::' +
            os.path.basename(object.location.filename()).split('.webidl')[0] + 'Binding')


def getTypesFromDescriptor(descriptor):
    """
    Get all argument and return types for all members of the descriptor
    """
    members = [m for m in descriptor.interface.members]
    if descriptor.interface.ctor():
        members.append(descriptor.interface.ctor())
    members.extend(descriptor.interface.namedConstructors)
    signatures = [s for m in members if m.isMethod() for s in m.signatures()]
    types = []
    for s in signatures:
        assert len(s) == 2
        (returnType, arguments) = s
        types.append(returnType)
        types.extend(a.type for a in arguments)

    types.extend(a.type for a in members if a.isAttr())
    return types


def getTypesFromDictionary(dictionary):
    """
    Get all member types for this dictionary
    """
    if isinstance(dictionary, IDLWrapperType):
        dictionary = dictionary.inner
    types = []
    curDict = dictionary
    while curDict:
        types.extend([m.type for m in curDict.members])
        curDict = curDict.parent
    return types


def getTypesFromCallback(callback):
    """
    Get the types this callback depends on: its return type and the
    types of its arguments.
    """
    sig = callback.signatures()[0]
    types = [sig[0]]  # Return type
    types.extend(arg.type for arg in sig[1])  # Arguments
    return types


def iteratorNativeType(descriptor, infer=False):
    assert descriptor.interface.isIterable()
    iterableDecl = descriptor.interface.maplikeOrSetlikeOrIterable
    assert iterableDecl.isPairIterator()
    return "IterableIterator%s" % ("" if infer else '<%s>' % descriptor.interface.identifier.name)
