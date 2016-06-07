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


def symbolify(ident):
    return "_ZN9nsGkAtoms" + str(len(ident)) + ident + "E"


with open(objdir_path + "/dist/include/nsGkAtomList.h") as f:
    lines = [line for line in f.readlines() if line.startswith("GK_ATOM")]
    atoms = [line_to_atom(line) for line in lines]

with open("atom_macro.rs", "w") as f:
    f.write("use gecko_bindings::structs::nsIAtom;\n\n")
    f.write("use Atom;\n\n")
    f.write("pub fn unsafe_atom_from_static(ptr: *mut nsIAtom) -> Atom { unsafe { Atom::from_static(ptr) } }\n\n")
    for atom in atoms:
        f.write('extern { pub static %s: *mut nsIAtom; }\n' % symbolify(atom[0]))
    f.write("#[macro_export]\n")
    f.write("macro_rules! atom {\n")
    f.writelines(['("%s") => { $crate::atom_macro::unsafe_atom_from_static($crate::atom_macro::%s) };\n'
                 % (atom[1], symbolify(atom[0])) for atom in atoms])
    f.write("}\n")
