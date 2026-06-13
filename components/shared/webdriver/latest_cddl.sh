#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

# @file latest_cddl.sh
# @brief Fetch the latest CDDL definition from WebDriver BiDi spec.
# @description
#    bash latest_cddl.sh > cddls/webdriver-bidi.cddl

# @description Extract CDDL definitions from index.bs
extract_cddl() {
  awk '
# preamble
BEGIN {
    print "; This Source Code Form is subject to the terms of the Mozilla Public"
    print "; License, v. 2.0. If a copy of the MPL was not distributed with this"
    print "; file, You can obtain one at https://mozilla.org/MPL/2.0/."
    print ";"
    print "; The origin of this CDDL file is:"
    print "; https://www.w3.org/TR/webdriver-bidi"
}

# only extract content in <pre> and ignore comment
/<pre class="cddl"/ { in_pre = 1; next } 
/<\/pre>/ { in_pre = 0; next }

in_pre {
    # skip empty line
    if (/^[ \t]*$/) next;

    # when a new rule is encountered
    if (/=/) {
        # manually add empty line when a new rule is encountered
        print ""
        # record the indent of each rule
        match($0, /^[ \t]+/)
        indent = substr($0, 1, RLENGTH)
    }
    
    # remove the indent of each line in the rule
    # guard in case the indent is incorrect
    if (substr($0, 1, length(indent)) == indent) {
        $0 = substr($0, length(indent) + 1);
    }
    
    print
}'
}

# @description Remove HTML comments in advance
remove_comments() {
  awk -v RS='<!--' 'NR>1 {sub(/.*?-->/, "")} {print}'
}

# @description Fetch latest index.bs
fetch_cddl() {
  local url="https://github.com/w3c/webdriver-bidi/raw/refs/heads/main/index.bs"
  curl -sSL "$url"
}

fetch_cddl | remove_comments | extract_cddl
