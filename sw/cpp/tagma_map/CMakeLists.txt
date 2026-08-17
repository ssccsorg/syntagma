cmake_minimum_required(VERSION 3.16)

project(tagma_map
    VERSION 0.1.0
    DESCRIPTION "Tagma hashless string-key map C++ port"
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
# coord_cube_map.h includes tagma_geo/spatial.h and test_cube_map links it.
if(NOT TARGET tagma_geo)
    add_subdirectory(../tagma_geo tagma_geo_build)
endif()

add_library(tagma_map src/coord_gen.cpp)
target_include_directories(tagma_map PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(tagma_map PUBLIC tagma_core)
target_compile_options(tagma_map PRIVATE -Wall -Wextra)

enable_testing()
add_executable(test_map tests/test_map.cpp)
target_include_directories(test_map PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(test_map PRIVATE tagma_map tagma_core)
target_compile_options(test_map PRIVATE -Wall -Wextra)
add_test(NAME map_test COMMAND test_map)

add_executable(test_dyn_map tests/test_dyn_map.cpp)
target_include_directories(test_dyn_map PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(test_dyn_map PRIVATE tagma_map tagma_core)
target_compile_options(test_dyn_map PRIVATE -Wall -Wextra)
add_test(NAME dyn_map_test COMMAND test_dyn_map)

add_executable(test_cube_map tests/test_cube_map.cpp)
target_include_directories(test_cube_map PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(test_cube_map PRIVATE tagma_map tagma_geo tagma_core)
target_compile_options(test_cube_map PRIVATE -Wall -Wextra)
add_test(NAME cube_map_test COMMAND test_cube_map)
