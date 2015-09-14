#!/bin/zsh

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.


setopt extended_glob
echo \
"/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
" >>| interfaces_mod.rs

# loop all files in interfaces dir
for x in $(print interfaces/*.rs~interfaces/mod.rs)
do
    # open the use statement
    echo -n "pub use interfaces::$(basename $x .rs)::{" >>| interfaces_mod.rs
    # append all pub struct names which do not begin with '_'
    grep -E '^pub struct [^_]' $x|sed 's/.*struct \(.*\) .*/\1/'|tr '\n' ',' >>| interfaces_mod.rs
    # append all pub types
    grep -E '^pub type ' $x|sed 's/pub type \([^ ]*\) .*/\1/'|tr '\n' ',' >>| interfaces_mod.rs
    # close the use statement
    echo '};' >>| interfaces_mod.rs
done
# open use statement for manually-generated types.rs
echo -n "pub use types::{" >>| interfaces_mod.rs
# append all pub types
grep -E '^pub type ' types.rs|sed 's/pub type \([^ ]*\) .*/\1/'|uniq|tr '\n' ',' >>| interfaces_mod.rs
# append all pub enums
grep -E '^pub enum ' types.rs|sed 's/pub enum \([^ ]*\) .*/\1/'|uniq|tr '\n' ',' >>| interfaces_mod.rs
# close use statement
echo '};' >>| interfaces_mod.rs
# append all pub structs beginning with "Cef" to alias them from types:: -> interfaces::
# some generated code from cef uses interfaces:: for types that it does not generate
grep -E '^pub struct Cef' types.rs|sed 's/pub struct \([^ ]*\) .*/pub use types::\1 as \1;/'|uniq >>| interfaces_mod.rs
# newline separators
echo -e '\n\n' >>| interfaces_mod.rs
# loop all files in interfaces dir again
for x in $(print interfaces/*.rs~interfaces/mod.rs)
do
    # add mod statements for all interfaces
    echo "pub mod $(basename $x .rs);" >>| interfaces_mod.rs
done
