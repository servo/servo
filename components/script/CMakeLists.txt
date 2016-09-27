project(script)
cmake_minimum_required(VERSION 2.6)

set(DUMMY ${CMAKE_BUILD_TYPE})

FUNCTION(PREPEND var prefix)
   SET(listVar "")
   FOREACH(f ${ARGN})
      LIST(APPEND listVar "${prefix}/${f}")
   ENDFOREACH(f)
   SET(${var} "${listVar}" PARENT_SCOPE)
ENDFUNCTION(PREPEND)

set(bindings_src ${PROJECT_SOURCE_DIR}/dom/bindings/codegen)
set(webidls_src ${PROJECT_SOURCE_DIR}/dom/webidls)

# Without Bindings/* stuff, since we install that separately below
set(globalgen_base_src
  PrototypeList.rs
  RegisterBindings.rs
  InterfaceObjectMap.rs
  InterfaceTypes.rs
  InheritTypes.rs
  UnionTypes.rs
  )

set(globalgen_src
  ${globalgen_base_src}
  Bindings/mod.rs
  )

file(GLOB_RECURSE webidls ${webidls_src}/*.webidl)
string(REGEX REPLACE ";" "\n" webidl_filelist "${webidls}")
file(WRITE "${PROJECT_BINARY_DIR}/webidls.list" "${webidl_filelist}")
string(REGEX REPLACE "\\.webidl(;|$)" "\\1" bindings "${webidls}")
string(REGEX REPLACE "(^|;)${webidls_src}/" "\\1" bindings "${bindings}")

set(globalgen_deps
  ${bindings_src}/GlobalGen.py
  ${bindings_src}/Bindings.conf
  ${bindings_src}/Configuration.py
  ${bindings_src}/CodegenRust.py
  ${bindings_src}/parser/WebIDL.py
  )
set(bindinggen_deps
  ${bindings_src}/BindingGen.py
  ${bindings_src}/Bindings.conf
  ${bindings_src}/Configuration.py
  ${bindings_src}/CodegenRust.py
  ${bindings_src}/parser/WebIDL.py
  )
  
add_custom_command(
  OUTPUT Bindings
  COMMAND ${CMAKE_COMMAND} -E make_directory Bindings
  )
add_custom_command(
  OUTPUT _cache
  COMMAND ${CMAKE_COMMAND} -E make_directory _cache
  )

add_custom_command(
  OUTPUT ParserResults.pkl
  COMMAND python -B ${bindings_src}/pythonpath.py -I ${bindings_src}/parser -I ${bindings_src}/ply
                 ${bindings_src}/GlobalGen.py
                 --cachedir=_cache
                 --filelist=webidls.list
                 ${bindings_src}/Bindings.conf
                 .
                 ${PROJECT_SOURCE_DIR}
  DEPENDS Bindings _cache ${globalgen_deps} ${webidls}
  VERBATIM
  )

add_custom_command(
  OUTPUT apis.html
  COMMAND python -B ${bindings_src}/pythonpath.py -I ${bindings_src}/parser -I ${bindings_src}/ply
                 ${bindings_src}/GlobalGen.py
                 --cachedir=_cache
                 --filelist=webidls.list
                 --only-html
                 ${bindings_src}/Bindings.conf
                 .
                 ${PROJECT_SOURCE_DIR}
  DEPENDS _cache ${globalgen_deps} ${webidls}
  VERBATIM
  )

add_custom_target(supported-apis DEPENDS apis.html)

# We need an intermediate custom target for this, due to this misfeature:
# > If any dependency is an OUTPUT of another custom command in the same
# > directory CMake automatically brings the other custom command into the
# > target in which this command is built.
# So, depending directly on ParserResults.pkl from the add_custom_command
# below would cause GlobalGen.py to be executed each time.
add_custom_target(ParserResults ALL DEPENDS ParserResults.pkl)
add_custom_target(generate-bindings ALL)

foreach(binding IN LISTS bindings)
  add_custom_command(
    OUTPUT Bindings/${binding}Binding.rs
    COMMAND python -B ${bindings_src}/pythonpath.py -I ${bindings_src}/parser -I ${bindings_src}/ply
                   ${bindings_src}/BindingGen.py
                   ${bindings_src}/Bindings.conf
                   .
                   Bindings/${binding}Binding
                   ${webidls_src}/${binding}.webidl
    DEPENDS Bindings ${bindinggen_deps} ${webidls_src}/${binding}.webidl ParserResults
    VERBATIM
    )
  add_custom_target(${binding} DEPENDS Bindings/${binding}Binding.rs)
  add_dependencies(generate-bindings ${binding})
endforeach()

PREPEND(globalgen_out ${CMAKE_BINARY_DIR}/ ${globalgen_base_src})
install(FILES ${globalgen_out} DESTINATION .)
install(DIRECTORY ${CMAKE_BINARY_DIR}/Bindings/ DESTINATION Bindings)
