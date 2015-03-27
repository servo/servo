# Uses

widlproc can be used to validate WebIDL in W3C specifications. It serves as the basis for the [W3C Web IDL on-line checker](http://www.w3.org/2009/07/webidl-check).

widlproc's generated XML is used to generate [webinos JavaScript APIs specifications](http://dev.webinos.org/specifications/draft/).

# License

widlproc is licensed under the Apache 2 License.

# Others

See also [webidl.js](https://github.com/darobin/webidl.js), a JavaScript-based Web IDL parser used by various tools in W3C.

# Credits

Most of the work on widlproc was done by Tim Renouf and Paddy Byers. Aplix corporation owns the copyright of the code up to June 2011.

The tool is kept up to date with the changes in the spec by Dominique Hazael-Massieux, through funding from the [webinos project](http://webinos.org/) since June 2011.

# Documentation

See doc/widlproc.html in the tree.

# Build Instructions

## Windows

Install requirements
* Cygwin - must install must install libs/libxslt
* Visual Studio express 2012 or 2010 (see difference below)

makefile uses cygwin make. References are coded in the make file to detect teh current version of visual studio 

# Future work
windows build could be improved to handle multipe versions with vcvars.bat
http://stackoverflow.com/questions/62029/vs2008-command-prompt-cygwin



