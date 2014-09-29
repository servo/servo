# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

# A set of simple pretty printers for gdb to make debugging Servo a bit easier.

# To load these, you need to add something like the following
# to your .gdbinit file.
#python
#import sys
#sys.path.insert(0, '/home/<path to git checkout>/servo/src/etc')
#import servo_gdb
#servo_gdb.register_printers(None)
#end

import gdb

# Print Au in both raw value and CSS pixels
class AuPrinter:
    def __init__(self, val):
        self.val = val

    def to_string(self):
        i32_type = gdb.lookup_type("i32")
        au = self.val.cast(i32_type);
        return "{0}px".format(au / 60.0)

# Print a U8 bitfield as binary
class BitFieldU8Printer:
    def __init__(self, val):
        self.val = val

    def to_string(self):
        u8_type = gdb.lookup_type("u8")
        value = self.val.cast(u8_type);
        return "[{0:#010b}]".format(int(value))

# Print a struct with fields as children
class ChildPrinter:
    def __init__(self, val):
        self.val = val

    def children(self):
        children = []
        for f in self.val.type.fields():
            children.append( (f.name, self.val[f.name]) )
        return children

    def to_string(self):
        return None

# Allow a trusted node to be dereferenced in the debugger
class TrustedNodeAddressPrinter:
    def __init__(self, val):
        self.val = val

    def children(self):
        node_type = gdb.lookup_type("struct script::dom::node::Node").pointer()
        value = self.val.cast(node_type)
        return [('Node', value)]

    def to_string(self):
        return self.val.address

# Extract a node type ID from enum
class NodeTypeIdPrinter:
    def __init__(self, val):
        self.val = val

    def to_string(self):
        u8_ptr_type = gdb.lookup_type("u8").pointer()
        enum_0 = self.val.address.cast(u8_ptr_type).dereference()
        enum_type = self.val.type.fields()[int(enum_0)].type;
        return str(enum_type).lstrip('struct ')

# Printer for std::Option<>
class OptionPrinter:
    def __init__(self, val):
        self.val = val

    def is_some(self):
        # Get size of discriminator
        d_size = self.val.type.fields()[0].type.sizeof

        if d_size > 0 and d_size <= 8:
            # Read first byte to check if None or Some
            ptr = self.val.address.cast(gdb.lookup_type("unsigned char").pointer())
            discriminator = int(ptr.dereference())
            return discriminator != 0

        raise "unhandled discriminator size"

    def children(self):
        if self.is_some():
            option_type = self.val.type

            # Get total size and size of value
            ptr = self.val.address.cast(gdb.lookup_type("unsigned char").pointer())
            t_size = option_type.sizeof
            value_type = option_type.fields()[1].type.fields()[1].type
            v_size = value_type.sizeof
            data_ptr = (ptr + t_size - v_size).cast(value_type.pointer()).dereference()
            return [('Some', data_ptr)]
        return [('None', None)]

    def to_string(self):
        return None

# Useful for debugging when type is unknown
class TestPrinter:
    def __init__(self, val):
        self.val = val

    def to_string(self):
        return "[UNKNOWN - type = {0}]".format(str(self.val.type))

type_map = [
    ('struct Au', AuPrinter),
    ('FlowFlags', BitFieldU8Printer),
    ('IntrinsicWidths', ChildPrinter),
    ('PlacementInfo', ChildPrinter),
    ('TrustedNodeAddress', TrustedNodeAddressPrinter),
    ('NodeTypeId', NodeTypeIdPrinter),
    ('Option', OptionPrinter),
]

def lookup_servo_type (val):
    val_type = str(val.type)
    for (type_name, printer) in type_map:
        if val_type == type_name or val_type.endswith("::"+type_name):
            return printer(val)
    return None
    #return TestPrinter(val)

def register_printers(obj):
    gdb.pretty_printers.append(lookup_servo_type)
