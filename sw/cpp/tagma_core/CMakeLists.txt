cmake_minimum_required(VERSION 3.16)

project(tagma_core
    VERSION 0.1.0
    DESCRIPTION "Tagma coordinate engine C++ core"
    LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Release)
endif()

add_library(tagma_core src/coord.cpp)
target_include_directories(tagma_core PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_compile_options(tagma_core PRIVATE -Wall -Wextra)

enable_testing()
add_executable(test_coord tests/test_coord.cpp)
target_include_directories(test_coord PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(test_coord PRIVATE tagma_core)
target_compile_options(test_coord PRIVATE -Wall -Wextra)
add_test(NAME coord_test COMMAND test_coord)
