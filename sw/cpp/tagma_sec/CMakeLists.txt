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

set(TAGMA_SEC_SOURCES
    src/hash.cpp
    src/audit.cpp
    src/channel.cpp
    src/legacy.cpp
    src/delos.cpp
    src/proxy.cpp)

add_library(tagma_sec ${TAGMA_SEC_SOURCES})
target_include_directories(tagma_sec PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(tagma_sec PUBLIC tagma_core)
target_compile_options(tagma_sec PRIVATE -Wall -Wextra)

# Standard-library hardening for the test build, selected by the standard
# library in use. It exists because of the truncated-evidence case in
# test_verification.cpp: that case must be rejected by the length guard in
# MacChannel::verify_receipt, and without hardening a missing guard is not
# observable, because the guard protects an out-of-bounds read (undefined
# behavior) whose shifted fields happen to disagree, so the case passes by
# accident of memory layout instead of failing.
include(CheckCXXSourceCompiles)

check_cxx_source_compiles("
#include <vector>
#ifndef __GLIBCXX__
#error libstdc++ is not in use
#endif
int main() { return 0; }
" TAGMA_SEC_TESTS_USE_LIBSTDCXX)

check_cxx_source_compiles("
#include <vector>
#if !defined(_LIBCPP_VERSION) || _LIBCPP_VERSION < 180000
#error _LIBCPP_HARDENING_MODE needs libc++ 18 or newer
#endif
int main() { return 0; }
" TAGMA_SEC_TESTS_USE_LIBCXX_HARDENING_MODE)

if(TAGMA_SEC_TESTS_USE_LIBSTDCXX)
    # __glibcxx_requires_subscript aborts with an assertion message.
    set(TAGMA_SEC_TEST_HARDENING_DEFINITION "_GLIBCXX_ASSERTIONS")
elseif(TAGMA_SEC_TESTS_USE_LIBCXX_HARDENING_MODE)
    # The extensive mode checks std::vector::operator[] bounds.
    set(TAGMA_SEC_TEST_HARDENING_DEFINITION
        "_LIBCPP_HARDENING_MODE=_LIBCPP_HARDENING_MODE_EXTENSIVE")
else()
    set(TAGMA_SEC_TEST_HARDENING_DEFINITION "")
    message(WARNING
        "no standard-library hardening definition known for this toolchain: "
        "the truncated-evidence case in test_verification.cpp may pass by "
        "accident of memory layout")
endif()

# The tests link their own hardened build of the same sources instead of
# tagma_sec, because the guard they pin is compiled into channel.cpp: hardening
# only the test executables leaves that read in unhardened code, which was
# measured to keep passing with the guard deleted. tagma_sec itself keeps its
# flags, so the shipped artifact and its consumers are unaffected.
if(TAGMA_SEC_TEST_HARDENING_DEFINITION)
    message(STATUS "tagma_sec test hardening: ${TAGMA_SEC_TEST_HARDENING_DEFINITION}")
    add_library(tagma_sec_tests STATIC ${TAGMA_SEC_SOURCES})
    target_include_directories(tagma_sec_tests PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/include)
    target_link_libraries(tagma_sec_tests PUBLIC tagma_core)
    target_compile_options(tagma_sec_tests PRIVATE -Wall -Wextra)
    target_compile_definitions(tagma_sec_tests PRIVATE ${TAGMA_SEC_TEST_HARDENING_DEFINITION})
    set(TAGMA_SEC_TEST_LIBRARY tagma_sec_tests)
else()
    set(TAGMA_SEC_TEST_LIBRARY tagma_sec)
endif()

enable_testing()
add_executable(test_hash tests/test_hash.cpp)
target_include_directories(test_hash PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(test_hash PRIVATE ${TAGMA_SEC_TEST_LIBRARY} tagma_core)
target_compile_options(test_hash PRIVATE -Wall -Wextra)
add_test(NAME sec_hash_test COMMAND test_hash)

add_executable(test_vectors tests/test_vectors.cpp)
target_include_directories(test_vectors PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(test_vectors PRIVATE ${TAGMA_SEC_TEST_LIBRARY} tagma_core)
target_compile_options(test_vectors PRIVATE -Wall -Wextra)
add_test(NAME sec_vectors_test COMMAND test_vectors)

add_executable(test_verification tests/test_verification.cpp)
target_include_directories(test_verification PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(test_verification PRIVATE ${TAGMA_SEC_TEST_LIBRARY} tagma_core)
target_compile_options(test_verification PRIVATE -Wall -Wextra)
add_test(NAME sec_verification_test COMMAND test_verification)

add_executable(test_workflow tests/test_workflow.cpp)
target_include_directories(test_workflow PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(test_workflow PRIVATE ${TAGMA_SEC_TEST_LIBRARY} tagma_core)
target_compile_options(test_workflow PRIVATE -Wall -Wextra)
add_test(NAME sec_workflow_test COMMAND test_workflow)

add_executable(test_delos tests/test_delos.cpp)
target_include_directories(test_delos PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(test_delos PRIVATE ${TAGMA_SEC_TEST_LIBRARY} tagma_core)
target_compile_options(test_delos PRIVATE -Wall -Wextra)
add_test(NAME sec_delos_test COMMAND test_delos)

add_executable(test_scenarios tests/test_scenarios.cpp)
target_include_directories(test_scenarios PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(test_scenarios PRIVATE ${TAGMA_SEC_TEST_LIBRARY} tagma_core)
target_compile_options(test_scenarios PRIVATE -Wall -Wextra)
add_test(NAME sec_scenarios_test COMMAND test_scenarios)

# The test executables are hardened as well, so undefined behavior in the test
# code itself traps too.
if(TAGMA_SEC_TEST_HARDENING_DEFINITION)
    foreach(sec_test IN ITEMS
            test_hash test_vectors test_verification
            test_workflow test_delos test_scenarios)
        target_compile_definitions(${sec_test} PRIVATE ${TAGMA_SEC_TEST_HARDENING_DEFINITION})
    endforeach()
endif()
