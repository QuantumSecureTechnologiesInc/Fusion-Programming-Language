cmake_minimum_required(VERSION 3.20)
project(fusionrt C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

# Source files — core native runtime + collection runtimes
set(FUSIONRT_SOURCES
    fusionrt.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../vector_runtime.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../hashmap_runtime.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../hashset_runtime.c
)

# Create static library
add_library(fusionrt STATIC ${FUSIONRT_SOURCES})

# Include directories
target_include_directories(fusionrt PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}
)

# Platform-specific libraries
if(WIN32)
    target_link_libraries(fusionrt ws2_32)
else()
    target_link_libraries(fusionrt pthread)
endif()

# Install targets
install(TARGETS fusionrt
    ARCHIVE DESTINATION lib
    LIBRARY DESTINATION lib
)

install(FILES fusionrt.h DESTINATION include)

# Build output
message(STATUS "Fusion Native Runtime Configuration:")
message(STATUS "  - Build type: ${CMAKE_BUILD_TYPE}")
message(STATUS "  - C Compiler: ${CMAKE_C_COMPILER}")
message(STATUS "  - Sources: fusionrt.c + vector/hashmap/hashset runtimes")
message(STATUS "  - Install prefix: ${CMAKE_INSTALL_PREFIX}")
