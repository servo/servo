from components.script_bindings.codegen.codegen import CGIndenter
from components.script_bindings.codegen.utils import stripTrailingWhitespace


class CGThing():
    """
    Abstract base class for things that spit out code.
    """
    def __init__(self):
        pass  # Nothing for now

    def define(self):
        """Produce code for a Rust file."""
        raise NotImplementedError  # Override me!


class CGWrapper(CGThing):
    """
    Generic CGThing that wraps other CGThings with pre and post text.
    """
    def __init__(self, child, pre="", post="", reindent=False):
        CGThing.__init__(self)
        self.child = child
        self.pre = pre
        self.post = post
        self.reindent = reindent

    def define(self):
        defn = self.child.define()
        if self.reindent:
            # We don't use lineStartDetector because we don't want to
            # insert whitespace at the beginning of our _first_ line.
            defn = stripTrailingWhitespace(
                defn.replace("\n", f"\n{' ' * len(self.pre)}"))
        return f"{self.pre}{defn}{self.post}"


class CGList(CGThing):
    """
    Generate code for a list of GCThings.  Just concatenates them together, with
    an optional joiner string.  "\n" is a common joiner.
    """
    def __init__(self, children, joiner=""):
        CGThing.__init__(self)
        # Make a copy of the kids into a list, because if someone passes in a
        # generator we won't be able to both declare and define ourselves, or
        # define ourselves more than once!
        self.children = list(children)
        self.joiner = joiner

    def append(self, child):
        self.children.append(child)

    def prepend(self, child):
        self.children.insert(0, child)

    def join(self, iterable):
        return self.joiner.join(s for s in iterable if len(s) > 0)

    def define(self):
        return self.join(child.define() for child in self.children if child is not None)

    def __len__(self):
        return len(self.children)


class CGGeneric(CGThing):
    """
    A class that spits out a fixed string into the codegen.  Can spit out a
    separate string for the declaration too.
    """
    def __init__(self, text):
        self.text = text

    def define(self):
        return self.text


class CGAbstractMethod(CGThing):
    """
    An abstract class for generating code for a method.  Subclasses
    should override definition_body to create the actual code.

    descriptor is the descriptor for the interface the method is associated with

    name is the name of the method as a string

    returnType is the IDLType of the return value

    args is a list of Argument objects

    inline should be True to generate an inline method, whose body is
    part of the declaration.

    alwaysInline should be True to generate an inline method annotated with
    MOZ_ALWAYS_INLINE.

    If templateArgs is not None it should be a list of strings containing
    template arguments, and the function will be templatized using those
    arguments.

    docs is None or documentation for the method in a string.

    unsafe is used to add the decorator 'unsafe' to a function, giving as a result
    an 'unsafe fn()' declaration.
    """
    def __init__(self, descriptor, name, returnType, args, inline=False,
                 alwaysInline=False, extern=False, unsafe=False, pub=False,
                 templateArgs=None, docs=None, doesNotPanic=False, extra_decorators=[]):
        CGThing.__init__(self)
        self.descriptor = descriptor
        self.name = name
        self.returnType = returnType
        self.args = args
        self.alwaysInline = alwaysInline
        self.extern = extern
        self.unsafe = extern or unsafe
        self.templateArgs = templateArgs
        self.pub = pub
        self.docs = docs
        self.catchPanic = self.extern and not doesNotPanic
        self.extra_decorators = extra_decorators

    def _argstring(self):
        return ', '.join([a.declare() for a in self.args])

    def _template(self):
        if self.templateArgs is None:
            return ''
        return f'<{", ".join(self.templateArgs)}>\n'

    def _docs(self):
        if self.docs is None:
            return ''

        lines = self.docs.splitlines()
        return ''.join(f'/// {line}\n' for line in lines)

    def _decorators(self):
        decorators = []
        if self.alwaysInline:
            decorators.append('#[inline]')

        decorators.extend(self.extra_decorators)

        if self.pub:
            decorators.append('pub')

        if self.unsafe:
            decorators.append('unsafe')

        if self.extern:
            decorators.append('extern "C"')

        if not decorators:
            return ''
        return f'{" ".join(decorators)} '

    def _returnType(self):
        return f" -> {self.returnType}" if self.returnType != "void" else ""

    def define(self):
        body = self.definition_body()

        if self.catchPanic:
            if self.returnType == "void":
                pre = "wrap_panic(&mut || {\n"
                post = "\n})"
            elif "return" not in body.define() or self.name.startswith("_constructor"):
                pre = (
                    "let mut result = false;\n"
                    "wrap_panic(&mut || result = {\n"
                )
                post = (
                    "\n});\n"
                    "result"
                )
            else:
                pre = (
                    "let mut result = false;\n"
                    "wrap_panic(&mut || result = (|| {\n"
                )
                post = (
                    "\n})());\n"
                    "result"
                )
            body = CGWrapper(CGIndenter(body), pre=pre, post=post)

        return CGWrapper(CGIndenter(body),
                         pre=self.definition_prologue(),
                         post=self.definition_epilogue()).define()

    def definition_prologue(self):
        return (f"{self._docs()}{self._decorators()}"
                f"fn {self.name}{self._template()}({self._argstring()}){self._returnType()}{{\n")

    def definition_epilogue(self):
        return "\n}\n"

    def definition_body(self):
        raise NotImplementedError  # Override me!


class CGAbstractExternMethod(CGAbstractMethod):
    """
    Abstract base class for codegen of implementation-only (no
    declaration) static methods.
    """
    def __init__(self, descriptor, name, returnType, args, doesNotPanic=False, templateArgs=None):
        CGAbstractMethod.__init__(self, descriptor, name, returnType, args,
                                  inline=False, extern=True, doesNotPanic=doesNotPanic, templateArgs=templateArgs)
