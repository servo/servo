#!/bin/bash

# Spidermonkey requires autoconf 2.13 exactly

if [ ! -z `which autoconf213` ]
then
    AUTOCONF213=autoconf213
fi

if [ ! -z `which autoconf2.13` ]
then
    AUTOCONF213=autoconf2.13
fi

if [ -z "$AUTOCONF213" ]
then
    echo "I need autoconf 2.13"
fi

(cd src/mozjs/js/src && $AUTOCONF213) || exit $?

cp -f configure.in configure
