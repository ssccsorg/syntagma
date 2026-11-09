#pragma once

// tagma_matrix: the physical order of a matrix's backing bytes. Mirrors
// sw/rust/matrix/src/order.rs.
//
// The order decides where a logical element sits. It never decides which
// element an address names, so two matrices in different orders answer the
// same coordinate with the same value.
//
// An order is a type with a static `offset(rows, cols, i, j)`, mirroring the
// Rust `Order` trait. The C++ port states the requirement here rather than in
// a constraint, because C++17 has no concepts.

#include <cstddef>

namespace tagma_matrix {

// Consecutive columns are adjacent in memory.
struct RowMajor {
  static constexpr std::size_t offset(std::size_t rows, std::size_t cols,
                                      std::size_t i, std::size_t j) {
    static_cast<void>(rows);
    return i * cols + j;
  }
};

// Consecutive rows are adjacent in memory.
struct ColMajor {
  static constexpr std::size_t offset(std::size_t rows, std::size_t cols,
                                      std::size_t i, std::size_t j) {
    static_cast<void>(cols);
    return j * rows + i;
  }
};

}  // namespace tagma_matrix
