cmake_minimum_required(VERSION 3.16)

project(tagma_matrix
    VERSION 0.1.0
    DESCRIPTION "Coordinate-addressed matrices C++ port"
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

# Header-only: the shapes are template parameters, so there is no translation
# unit, mirroring the Rust crate's generic types.
add_library(tagma_matrix INTERFACE)
target_include_directories(tagma_matrix INTERFACE ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(tagma_matrix INTERFACE tagma_core)

enable_testing()

add_executable(test_matrix_addressing tests/test_addressing.cpp)
target_include_directories(test_matrix_addressing PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(test_matrix_addressing PRIVATE tagma_matrix tagma_core)
target_compile_options(test_matrix_addressing PRIVATE -Wall -Wextra)
add_test(NAME matrix_addressing_test COMMAND test_matrix_addressing)

add_executable(test_matrix_borrowed tests/test_borrowed.cpp)
target_include_directories(test_matrix_borrowed PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(test_matrix_borrowed PRIVATE tagma_matrix tagma_core)
target_compile_options(test_matrix_borrowed PRIVATE -Wall -Wextra)
add_test(NAME matrix_borrowed_test COMMAND test_matrix_borrowed)

add_executable(test_matrix_matmul tests/test_matmul.cpp)
target_include_directories(test_matrix_matmul PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(test_matrix_matmul PRIVATE tagma_matrix tagma_core)
target_compile_options(test_matrix_matmul PRIVATE -Wall -Wextra)
add_test(NAME matrix_matmul_test COMMAND test_matrix_matmul)

add_executable(test_matrix_relocation tests/test_relocation.cpp)
target_include_directories(test_matrix_relocation PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(test_matrix_relocation PRIVATE tagma_matrix tagma_core)
target_compile_options(test_matrix_relocation PRIVATE -Wall -Wextra)
add_test(NAME matrix_relocation_test COMMAND test_matrix_relocation)

add_executable(test_matrix_requant tests/test_requant.cpp)
target_include_directories(test_matrix_requant PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(test_matrix_requant PRIVATE tagma_matrix tagma_core)
target_compile_options(test_matrix_requant PRIVATE -Wall -Wextra)
add_test(NAME matrix_requant_test COMMAND test_matrix_requant)

add_executable(test_matrix_scalar_ref tests/test_scalar_ref.cpp)
target_include_directories(test_matrix_scalar_ref PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(test_matrix_scalar_ref PRIVATE tagma_matrix tagma_core)
target_compile_options(test_matrix_scalar_ref PRIVATE -Wall -Wextra)
add_test(NAME matrix_scalar_ref_test COMMAND test_matrix_scalar_ref)

add_executable(test_matrix_wire tests/test_wire.cpp)
target_include_directories(test_matrix_wire PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(test_matrix_wire PRIVATE tagma_matrix tagma_core)
target_compile_options(test_matrix_wire PRIVATE -Wall -Wextra)
add_test(NAME matrix_wire_test COMMAND test_matrix_wire)

add_executable(test_matrix_parity tests/test_parity.cpp)
target_include_directories(test_matrix_parity PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(test_matrix_parity PRIVATE tagma_matrix tagma_core)
target_compile_options(test_matrix_parity PRIVATE -Wall -Wextra)
add_test(NAME matrix_parity_test COMMAND test_matrix_parity)
