#!/usr/bin/env python

# Copyright 2026 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

"""
Generate the mapping for [`text-transform: full-width`]

> The definition of **full-width** and **half-width** forms can be found in
> [Unicode Standard Annex #11: East Asian Width][UAX11].
> The mapping to full-width form is defined by taking code points
> with the `<wide>` or the `<narrow>` tag in their `Decomposition_Mapping`
> in [Unicode Standard Annex #44: Unicode Character Database][UAX44].
> For the `<narrow>` tag, the mapping is from the code point to the decomposition
> (minus `<narrow>` tag), and for the `<wide>` tag, the mapping is from
> the decomposition (minus the `<wide>` tag) back to the original code point.

[`text-transform: full-width`]: https://drafts.csswg.org/css-text-4/#full-width
[UAX11]: https://www.unicode.org/reports/tr11/
[UAX44]: https://www.unicode.org/reports/tr44/
"""

from urllib.request import urlopen
import re

README_URL = "https://www.unicode.org/Public/UCD/latest/ReadMe.txt"
DATA_URL = "https://www.unicode.org/Public/UCD/latest/ucd/UnicodeData.txt"

response = urlopen(README_URL)
assert response.status == 200
body = response.read().decode("utf-8")
version = re.search(r"Version ([\d.]+)", body).group(1)

response = urlopen(DATA_URL)
assert response.status == 200
body = response.read().decode("utf-8")

print(
    f"""
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generated from <{DATA_URL}> version {version}
//!
//! Do not edit directly. Update it with:
//!
//! ```sh
//! etc/generate_full_width_mappings.py > components/layout/flow/inline/full_width.rs
//! ```

/// <https://drafts.csswg.org/css-text-4/#full-width>
pub(crate) static FULL_WIDTH_MAPPINGS: phf::Map<char, char> = phf::phf_map! {{
""".strip()
)
for line in body.splitlines():
    line = line.strip()
    if not line:
        continue
    fields = line.split(";")
    code_point = fields[0]
    decomposition = iter(fields[5].split(" ", 1))
    decomposition_type = next(decomposition)
    decomposition_mapping = next(decomposition, None)
    if decomposition_type == "<wide>":
        key = decomposition_mapping
        value = code_point
    elif decomposition_type == "<narrow>":
        key = code_point
        value = decomposition_mapping
    else:
        continue
    assert len(key) == 4, line
    assert len(value) == 4, line
    key_chr = chr(int(key, 16))
    value_chr = chr(int(value, 16))
    print(f"    '\\u{{{key}}}' => '\\u{{{value}}}', // '{key_chr}' → '{value_chr}'")
print("};")
