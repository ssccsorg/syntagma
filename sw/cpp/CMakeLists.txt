cmake_minimum_required(VERSION 3.16)

project(tagma_cpp
    VERSION 0.1.0
    DESCRIPTION "Tagma coordinate engine C++ implementation"
    LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Release)
endif()

enable_testing()

# The C++ port mirrors the Rust workspace crate layout: tagma_core first,
# then the modules that depend on it.
add_subdirectory(tagma_core)
add_subdirectory(base11172)
add_subdirectory(tagma_geo)
