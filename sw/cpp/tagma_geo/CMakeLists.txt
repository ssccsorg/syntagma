cmake_minimum_required(VERSION 3.16)

project(tagma_geo
    VERSION 0.1.0
    DESCRIPTION "Tagma spatial operations C++ port"
    LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Release)
endif()

# Buildable standalone or as part of the sw/cpp superproject.
if(NOT TARGET tagma_core)
    add_subdirectory(../tagma_core tagma_core_build)
endif()

# Header-only library mirroring the Rust tagma-geo crate.
add_library(tagma_geo INTERFACE)
target_include_directories(tagma_geo INTERFACE ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(tagma_geo INTERFACE tagma_core)

enable_testing()
add_executable(test_spatial tests/test_spatial.cpp)
target_include_directories(test_spatial PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(test_spatial PRIVATE tagma_geo tagma_core)
target_compile_options(test_spatial PRIVATE -Wall -Wextra)
add_test(NAME spatial_test COMMAND test_spatial)
