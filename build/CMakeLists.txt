cmake_minimum_required(VERSION 3.16)

# Skip compiler test - for configurations only
set(CMAKE_TRY_COMPILE_CONFIGURATION Release)

project(FusionVortex VERSION 2.0.0 LANGUAGES C)

# Set C standard
set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

# Set default build type to Release
if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Release)
endif()

# Options
option(BUILD_FUSION_COMPILER "Build the Fusion compiler" ON)
option(BUILD_RUNTIME "Build the Fusion runtime library" ON)
option(BUILD_EXAMPLES "Build examples" OFF)
option(ENABLE_WERROR "Treat warnings as errors" OFF)

# Find LLVM (optional)
find_package(LLVM 20 CONFIG)
if(LLVM_FOUND)
    message(STATUS "Found LLVM ${LLVM_PACKAGE_VERSION}")
    message(STATUS "Using LLVMConfig.cmake in: ${LLVM_DIR}")
endif()

# Output directories
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)

# Include directories
include_directories(
    ${CMAKE_CURRENT_SOURCE_DIR}
    ${CMAKE_CURRENT_SOURCE_DIR}/src
    ${CMAKE_CURRENT_SOURCE_DIR}/src/runtime
    ${CMAKE_CURRENT_SOURCE_DIR}/runtime
    ${CMAKE_CURRENT_SOURCE_DIR}/runtime/native
)

# Find all C source files in runtime.
# runtime.c and runtime/native/fusionrt.c both support POSIX and Win32.
file(GLOB_RECURSE RUNTIME_SOURCES
    ${CMAKE_CURRENT_SOURCE_DIR}/runtime/*.c
    ${CMAKE_CURRENT_SOURCE_DIR}/runtime/native/*.c
)

# Runtime library
if(BUILD_RUNTIME)
    add_library(fusionrt STATIC ${RUNTIME_SOURCES})
    set_target_properties(fusionrt PROPERTIES
        OUTPUT_NAME "fusionrt"
        POSITION_INDEPENDENT_CODE ON
    )
    target_compile_definitions(fusionrt PRIVATE FUSION_RUNTIME)
    target_compile_options(fusionrt PRIVATE /W4)
    
    # Create import library for Windows
    if(WIN32)
        set_target_properties(fusionrt PROPERTIES
            OUTPUT_NAME "fusionrt"
            IMPORT_LIBRARY_OUTPUT_NAME "fusionrt"
        )
    endif()
    
    install(TARGETS fusionrt
        ARCHIVE DESTINATION lib
        LIBRARY DESTINATION lib
    )
endif()

# Examples
if(BUILD_EXAMPLES)
    add_subdirectory(examples)
endif()

# Fusion compiler (Rust/Cargo)
if(BUILD_FUSION_COMPILER)
    # The fuc compiler is built via Cargo
    # This is handled by the build system
    
    # Add a custom target to build the compiler
    add_custom_target(build_fuc
        COMMAND ${CMAKE_CURRENT_SOURCE_DIR}/install.sh
        WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
        COMMENT "Building Fusion compiler (fuc)"
    )
    
    # Copy prebuilt compiler if it exists
    if(EXISTS ${CMAKE_CURRENT_SOURCE_DIR}/bin/fuc)
        add_custom_command(OUTPUT ${CMAKE_BINARY_DIR}/bin/fuc
            COMMAND ${CMAKE_COMMAND} -E copy
                ${CMAKE_CURRENT_SOURCE_DIR}/bin/fuc
                ${CMAKE_BINARY_DIR}/bin/fuc
            DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/bin/fuc
        )
        add_custom_target(fuc_copy ALL DEPENDS ${CMAKE_BINARY_DIR}/bin/fuc)
    endif()
endif()

# Installation
install(DIRECTORY bin/
    DESTINATION bin
    FILES_MATCHING PATTERN "*"
)

install(DIRECTORY src/
    DESTINATION include/fusion
    FILES_MATCHING PATTERN "*.fu"
)

install(DIRECTORY runtime/
    DESTINATION include/fusion
    FILES_MATCHING PATTERN "*.h"
    PATTERN "*.c"
)

# Print configuration summary
message(STATUS "")
message(STATUS "=== Fusion Vortex Configuration ===")
message(STATUS "Version: ${PROJECT_VERSION}")
message(STATUS "Build Type: ${CMAKE_BUILD_TYPE}")
message(STATUS "C Compiler: ${CMAKE_C_COMPILER}")
if(LLVM_FOUND)
    message(STATUS "LLVM: ${LLVM_PACKAGE_VERSION}")
else()
    message(STATUS "LLVM: Not found (optional)")
endif()
message(STATUS "Build Runtime: ${BUILD_RUNTIME}")
message(STATUS "Build Compiler: ${BUILD_FUSION_COMPILER}")
message(STATUS "===========================")
message(STATUS "")
