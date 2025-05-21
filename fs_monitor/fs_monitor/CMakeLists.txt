set(BIN_NAME fs_monitor)

add_library(
    ${BIN_NAME}

    private/lib.cpp
    private/monitors.h
    private/monitors.cpp
    private/util.h
    private/crt.h
    private/crt.cpp

    public/fs_monitor.h
)

target_include_directories(
    ${BIN_NAME}
    PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/private
)
target_include_directories(
    ${BIN_NAME}
    PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/public
)

set_property(TARGET ${BIN_NAME} PROPERTY CXX_STANDARD 20)

