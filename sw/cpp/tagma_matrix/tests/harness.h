#pragma once

// The check harness the tagma_matrix tests share.
//
// The other C++ module tests define `check` and `failures` per file. Seven test
// executables would copy that seven times, so it is one header here.

#include <cstdio>
#include <cstring>
#include <exception>

namespace harness {

inline int failures = 0;

inline void check(bool condition, const char* message) {
  if (!condition) {
    std::fprintf(stderr, "FAIL: %s\n", message);
    ++failures;
  }
}

inline void fail(const char* message) {
  std::fprintf(stderr, "FAIL: %s\n", message);
  ++failures;
}

// Reports both values when an integral comparison fails, so an element that
// disagrees names itself. `check` alone carries no actual or expected value,
// which a Rust `assert_eq!` does, and an element inside a double loop is the
// case where that omission costs the most.
template <class A, class B>
void check_eq(const A& actual, const B& expected, const char* message) {
  if (actual == expected) return;
  std::fprintf(stderr, "FAIL: %s (got %lld, expected %lld)\n", message,
               static_cast<long long>(actual), static_cast<long long>(expected));
  ++failures;
}

// Runs `body` and requires it to throw a std::exception whose `what()` contains
// `expected`. This is the C++ form of a Rust `#[should_panic(expected = ...)]`.
template <class F>
void check_throws(F&& body, const char* expected, const char* message) {
  try {
    body();
  } catch (const std::exception& error) {
    if (std::strstr(error.what(), expected) == nullptr) {
      std::fprintf(stderr, "FAIL: %s (threw \"%s\", expected \"%s\")\n", message,
                   error.what(), expected);
      ++failures;
    }
    return;
  }
  fail(message);
}

inline int finish(const char* module) {
  if (failures != 0) {
    std::fprintf(stderr, "%d check(s) failed\n", failures);
    return 1;
  }
  std::printf("%s: all checks passed\n", module);
  return 0;
}

}  // namespace harness
