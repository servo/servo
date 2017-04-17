#!/bin/bash

cd "$(dirname "$0")"
find . -name reftest.list | sed 's,/reftest.list$,,' | while read DIRNAME
do
    cat "$DIRNAME/reftest.list" | grep -v -e "^default-preferences" -e "include " | sed 's/ #.*//;s/^#.*//;s/.* == /== /;s/.* != /!= /' | grep -v "^ *$" | while read TYPE TEST REF
    do
        REFTYPE=""
        if [ "$TYPE" == "==" ]
        then
            REFTYPE="match"
        elif [ "$TYPE" == "!=" ]
        then
            REFTYPE="mismatch"
        else
            echo "Unexpected type $TYPE for $DIRNAME/$TEST"
        fi
        if grep "rel=\(\"$REFTYPE\"\|'$REFTYPE'\)" "$DIRNAME/$TEST" | head -1 | grep -q "href=\(\"$REF\"\|'$REF'\)"
        then
            #echo "Good link for $DIRNAME/$TEST"
            echo -n
        else
            echo "Missing link for $DIRNAME/$TEST"
            #echo "<link rel=\"$REFTYPE\" href=\"$REF\">" >> "$DIRNAME/$TEST"
        fi
    done
done
