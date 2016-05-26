project(script)
cmake_minimum_required(VERSION 2.6)

set(DUMMY ${CMAKE_BUILD_TYPE})

set(bindings_src ${PROJECT_SOURCE_DIR}/dom/bindings/codegen)
set(webidls_src ${PROJECT_SOURCE_DIR}/dom/webidls)

set(globalgen_src
  PrototypeList.rs
  RegisterBindings.rs
  InterfaceObjectMap.rs
  InterfaceTypes.rs
  InheritTypes.rs
  Bindings/mod.rs
  UnionTypes.rs
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
  OUTPUT ParserResults.pkl ${globalgen_src}
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

add_custom_target(generate-bindings ALL DEPENDS ${globalgen_src})

foreach(binding IN LISTS bindings)
  add_custom_command(
    OUTPUT Bindings/${binding}Binding.rs
    COMMAND python -B ${bindings_src}/pythonpath.py -I ${bindings_src}/parser -I ${bindings_src}/ply
                   ${bindings_src}/BindingGen.py
                   ${bindings_src}/Bindings.conf
                   .
                   Bindings/${binding}Binding
                   ${webidls_src}/${binding}.webidl
    DEPENDS Bindings ${bindinggen_deps} ${webidls_src}/${binding}.webidl ParserResults.pkl
    VERBATIM
    )
  add_custom_target(${binding} DEPENDS Bindings/${binding}Binding.rs)
  add_dependencies(generate-bindings ${binding})
endforeach()
