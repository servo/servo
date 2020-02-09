import os

here = os.path.dirname(__file__)

readonly_template = """\
interface {interface}A {{
  readonly setlike<DOMString>;
  static void {method}();
}};

interface {interface}B {{
  readonly setlike<DOMString>;
  static readonly attribute long {method};
}};
"""

readwrite_template = """\
interface ReadOnly {{
  readonly setlike<DOMString>;
}};

interface ReadWrite {{
  setlike<DOMString>;
}};

interface {interface}A {{
  setlike<DOMString>;
  void {method}();
}};

interface {interface}B {{
  readonly setlike<DOMString>;
  void {method}();
}};

interface {interface}C {{
  readonly setlike<DOMString>;
  readonly attribute long {method};
}};

interface {interface}D {{
  readonly setlike<DOMString>;
  const long {method} = 0;
}};

interface {interface}E : ReadOnly {{
  void {method}();
}};

interface {interface}F : ReadOnly {{
  readonly attribute long {method};
}};

interface {interface}G : ReadOnly {{
  const long {method} = 0;
}};

interface {interface}H {{
  readonly setlike<DOMString>;
  static void {method}();
}};

interface {interface}I {{
  readonly setlike<DOMString>;
  static readonly attribute long {method};
}};

interface {interface}J1 {{
  static void {method}();
}};

interface {interface}J2 : {interface}J1 {{
  readonly setlike<DOMString>;
}};

interface {interface}K1 {{
  static readonly attribute long {method};
}};

interface {interface}K2 : {interface}K1 {{
  readonly setlike<DOMString>;
}};
"""

members_readonly = [
    "entries",
    "forEach",
    "has",
    "keys",
    "size",
    "values",
]

members_readwrite = [
    "add",
    "clear",
    "delete",
]

def transform(m):
    return m[0].upper() + m[1:]

tests = [
    (members_readonly, readonly_template),
    (members_readwrite, readwrite_template),
]

for (members, template) in tests:
    for method in members:
        path = "{here}/../valid/idl/setlike-{method}.widl".format(here=here, method=method)
        test = template.format(method=method, interface=transform(method))
        with open(path, "wb") as f:
            f.write(test.encode("utf8"))
