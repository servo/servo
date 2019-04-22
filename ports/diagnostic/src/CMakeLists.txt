project(diagnostic)
cmake_minimum_required(VERSION 2.6)

set(DUMMY ${CMAKE_BUILD_TYPE})

option(BUILD_tools "build the xmlwf tool for expat library" ON)
option(BUILD_examples "build the examples for expat library" ON)
option(BUILD_tests "build the tests for expat library" ON)
option(BUILD_shared "build a shared expat library" ON)

include(CheckIncludeFile)
include(CheckIncludeFiles)
include(CheckFunctionExists)
include(CheckSymbolExists)
include(TestBigEndian)

check_include_file("dlfcn.h" HAVE_DLFCN_H)

install(FILES foo DESTINATION .)
