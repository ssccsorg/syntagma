cmake_minimum_required(VERSION 3.16)

project(tagma_kv
    VERSION 0.1.0
    DESCRIPTION "Tagma hashless string-key store C++ port"
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

add_library(tagma_kv src/coord_gen.cpp)
target_include_directories(tagma_kv PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(tagma_kv PUBLIC tagma_core)
target_compile_options(tagma_kv PRIVATE -Wall -Wextra)

enable_testing()
add_executable(test_kv tests/test_kv.cpp)
target_include_directories(test_kv PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(test_kv PRIVATE tagma_kv tagma_core)
target_compile_options(test_kv PRIVATE -Wall -Wextra)
add_test(NAME kv_test COMMAND test_kv)

add_executable(test_dyn_kv tests/test_dyn_kv.cpp)
target_include_directories(test_dyn_kv PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(test_dyn_kv PRIVATE tagma_kv tagma_core)
target_compile_options(test_dyn_kv PRIVATE -Wall -Wextra)
add_test(NAME dyn_kv_test COMMAND test_dyn_kv)
