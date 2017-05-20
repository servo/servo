#!/bin/bash
# First arg is directory path to list, second arg is path to output file (minus file extension)
find $1 -type f ! -ipath '*.hg*' ! -ipath '*build-test*' ! -ipath '*selectors3*' ! -ipath '*/support/*' ! -ipath '*boland*' ! -ipath '*incoming*' > $2.txt
perl -pi -e "s#^$1/?((?:[^/]+/)*)([^/]+?)(\.[a-z]+)?\$#\$2\t\$1\$2\$3#" $2.txt
sort $2.txt -o $2.txt
echo '<!DOCTYPE html><html><title>CSS Tests by Filename</title><pre>' > $2.html
perl -pe 's#\t(.+)$#\t<a href="$1">$1</a>#' < $2.txt >> $2.html
echo '</pre>' >> $2.html
