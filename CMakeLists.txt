set(IM920_FFIInclude ${CMAKE_BINARY_DIR}/im920_rs_inc)

add_custom_command(
  OUTPUT "${IM920_FFIInclude}/srobo2/ffi/im920.hpp"
  COMMAND cbindgen --lang c++ --output ${IM920_FFIInclude}/srobo2/ffi/im920.hpp
  WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
  COMMENT "Generating im920.hpp with cbindgen"
)

add_custom_target(im920_rs_hxx_target DEPENDS "${IM920_FFIInclude}/srobo2/ffi/im920.hpp")

add_library(im920_rs_hxx INTERFACE)
add_dependencies(im920_rs_hxx im920_rs_hxx_target)
target_include_directories(im920_rs_hxx INTERFACE ./inc "${IM920_FFIInclude}")

unset(IM920_FFIInclude CACHE)