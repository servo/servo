# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import re
import sys

if len(sys.argv) != 2:
    print "usage: ./%s PATH/TO/OBJDIR" % sys.argv[0]
objdir_path = sys.argv[1]


def line_to_atom(line):
    result = re.match('^GK_ATOM\((.+),\s*"(.*)"\)', line)
    return (result.group(1), result.group(2))


def map_atom(ident):
    if ident in {"box", "loop", "match", "mod", "ref",
                 "self", "type", "use", "where", "in"}:
        return ident + "_"
    return ident


def gnu_symbolify(ident):
    return "_ZN9nsGkAtoms" + str(len(ident)) + ident + "E"


def msvc64_symbolify(ident):
    return "?" + ident + "@nsGkAtoms@@2PEAVnsIAtom@@EA"


def msvc32_symbolify(ident):
    return "?" + ident + "@nsGkAtoms@@2PAVnsIAtom@@A"


def write_items(f, func):
    f.write("        extern {\n")
    for atom in atoms:
        f.write(TEMPLATE.format(name=map_atom(atom[0]),
                                link_name=func(atom[0])))
    f.write("        }\n")


with open(objdir_path + "/dist/include/nsGkAtomList.h") as f:
    lines = [line for line in f.readlines() if line.startswith("GK_ATOM")]
    atoms = [line_to_atom(line) for line in lines]

TEMPLATE = """
            #[link_name = "{link_name}"]
            pub static {name}: *mut nsIAtom;
"""[1:]

with open("atom_macro.rs", "wb") as f:
    f.write("use gecko_bindings::structs::nsIAtom;\n\n")
    f.write("use Atom;\n\n")
    f.write("pub fn unsafe_atom_from_static(ptr: *mut nsIAtom) -> Atom { unsafe { Atom::from_static(ptr) } }\n\n")
    f.write("cfg_if! {\n")
    f.write("    if #[cfg(not(target_env = \"msvc\"))] {\n")
    write_items(f, gnu_symbolify)
    f.write("    } else if #[cfg(target_pointer_width = \"64\")] {\n")
    write_items(f, msvc64_symbolify)
    f.write("    } else {\n")
    write_items(f, msvc32_symbolify)
    f.write("    }\n")
    f.write("}\n\n")
    f.write("#[macro_export]\n")
    f.write("macro_rules! atom {\n")
    f.writelines(['("%s") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::%s) };\n'
                 % (atom[1], map_atom(atom[0])) for atom in atoms])
    f.write("}\n")
