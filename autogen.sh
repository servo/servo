#!/bin/bash

# Spidermonkey requires autoconf 2.13 exactly

if [ ! -z `which autoconf213` ]
then
    AUTOCONF=autoconf213
fi

if [ ! -z `which autoconf2.13` ]
then
    AUTOCONF=autoconf2.13
fi

(cd src/mozjs/js/src && $AUTOCONF)

cp -f configure.in configure
