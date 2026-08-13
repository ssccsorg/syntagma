cmake_minimum_required(VERSION 3.16)

project(tagma_sec
    VERSION 0.1.0
    DESCRIPTION "Tagma security layer C++ port"
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

add_library(tagma_sec
    src/hash.cpp
    src/audit.cpp
    src/channel.cpp
    src/legacy.cpp
    src/delos.cpp
    src/proxy.cpp)
target_include_directories(tagma_sec PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(tagma_sec PUBLIC tagma_core)
target_compile_options(tagma_sec PRIVATE -Wall -Wextra)

enable_testing()
add_executable(test_workflow tests/test_workflow.cpp)
target_include_directories(test_workflow PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(test_workflow PRIVATE tagma_sec tagma_core)
target_compile_options(test_workflow PRIVATE -Wall -Wextra)
add_test(NAME sec_workflow_test COMMAND test_workflow)

add_executable(test_delos tests/test_delos.cpp)
target_include_directories(test_delos PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(test_delos PRIVATE tagma_sec tagma_core)
target_compile_options(test_delos PRIVATE -Wall -Wextra)
add_test(NAME sec_delos_test COMMAND test_delos)

add_executable(test_scenarios tests/test_scenarios.cpp)
target_include_directories(test_scenarios PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(test_scenarios PRIVATE tagma_sec tagma_core)
target_compile_options(test_scenarios PRIVATE -Wall -Wextra)
add_test(NAME sec_scenarios_test COMMAND test_scenarios)
