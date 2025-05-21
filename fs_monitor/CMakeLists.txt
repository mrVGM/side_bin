cmake_minimum_required(VERSION 3.30.3)

project(fs_monitor)

set(CMAKE_CXX_COMPILER clang)
set(CMAKE_EXPORT_COMPILE_COMMANDS on)

set(CMAKE_BUILD_TYPE Release)

set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/ready)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/ready)
set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/ready)

# add_subdirectory(test)
add_subdirectory(fs_monitor)

