cmake_minimum_required(VERSION 3.16)

project(base11172
    VERSION 0.1.0
    DESCRIPTION "Tagma base-11172 self-validating serialization"
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

add_library(base11172 src/base11172.cpp)
target_include_directories(base11172 PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(base11172 PUBLIC tagma_core)
target_compile_options(base11172 PRIVATE -Wall -Wextra)

enable_testing()
add_executable(test_base11172 tests/test_base11172.cpp)
target_include_directories(test_base11172 PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(test_base11172 PRIVATE base11172)
target_compile_options(test_base11172 PRIVATE -Wall -Wextra)
add_test(NAME base11172_test COMMAND test_base11172)
