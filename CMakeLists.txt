add_custom_command(
  OUTPUT "${CMAKE_CURRENT_SOURCE_DIR}/inc/srobo2/ffi/im920.hpp"
  COMMAND /home/vscode/.cargo/bin/cbindgen --lang c++ --output inc/srobo2/ffi/im920.hpp
  WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
)

add_custom_target(im920_rs_hxx_target DEPENDS "${CMAKE_CURRENT_SOURCE_DIR}/inc/srobo2/ffi/im920.hpp")

add_library(im920_rs_hxx INTERFACE)
add_dependencies(im920_rs_hxx im920_rs_hxx_target)
target_include_directories(im920_rs_hxx INTERFACE ./inc)