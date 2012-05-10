#!/bin/bash

(cd src/mozjs/js/src && autoconf)

cp -f configure.in configure
