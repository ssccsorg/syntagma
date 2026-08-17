cmake_minimum_required(VERSION 3.16)

project(tagma_bench
    VERSION 0.1.0
    DESCRIPTION "Tagma C++ benchmark suite"
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
if(NOT TARGET tagma_geo)
    add_subdirectory(../tagma_geo tagma_geo_build)
endif()
if(NOT TARGET tagma_map)
    add_subdirectory(../tagma_map tagma_map_build)
endif()

# The suite uses CoordSpaceM (mmap); the superproject adds this
# subdirectory only on Unix, mirroring the Rust mmap feature gate.
add_executable(tagma_bench bench.cpp)
target_link_libraries(tagma_bench PRIVATE tagma_map tagma_geo tagma_core)
target_compile_options(tagma_bench PRIVATE -Wall -Wextra)
