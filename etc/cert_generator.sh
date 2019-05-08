#!/usr/bin/env bash

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

set -o errexit
set -o nounset
set -o pipefail

# https://wiki.mozilla.org/CA/Included_Certificates
# 1. Mozilla's official CA database CSV file is downloaded with curl
#    and processed with awk.
# 2. Rows end with `"\n`.
# 3. Each row is split by ^" and "," into columns.
# 4. Single and double quotes are removed from column 32.
# 5. If column 13 (12 in the csv file) contains `Websites`
#    (some are Email-only), column 32 is printed, the raw certificate.
# 6. All CA certs trusted for Websites are stored into the `certs` file.

url="https://ccadb-public.secure.force.com/mozilla/IncludedCACertificateReportPEMCSV"
curl "${url}" -sSf | gawk -v RS="\"\n" -F'","|^"' \
'{gsub("\047","",$(32));gsub("\"","",$(32));if($(13)~/Websites/)print $(32)}' \
> ../resources/certs
