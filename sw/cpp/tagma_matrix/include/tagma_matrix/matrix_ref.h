#pragma once

// tagma_matrix: a matrix that borrows its buffer. Mirrors
// sw/rust/matrix/src/view.rs.
//
// For weights that live in flash. Reading them through this type copies
// nothing, which an owned matrix cannot promise, so the weights stay where the
// linker put them. It writes them out through the same base-class method an
// owned matrix uses, so sending them costs no copy either.
//
// The view holds a pointer, as the Rust view holds a reference: the buffer must
// outlive it. Unlike the Rust borrow, nothing checks that here, so the one case
// the compiler can catch is refused: a view may not be built from a temporary
// buffer, whose lifetime ends at the end of the full expression.

#include "tagma_matrix/elements.h"
#include "tagma_matrix/order.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <stdexcept>

namespace tagma_matrix {

// A rank-2 matrix that borrows its buffer.
template <std::size_t R, std::size_t C, class O = RowMajor>
class MatrixRef : public Elements<MatrixRef<R, C, O>, R, C, O> {
 public:
  using Buffer = std::array<std::array<int8_t, C>, R>;

  // Borrows a buffer whose bytes are already in `O` order.
  constexpr explicit MatrixRef(const Buffer& data) : data_(&data) {}

  // A temporary buffer has no lifetime past this expression, and a view into it
  // would be the dangling pointer the Rust borrow checker refuses.
  MatrixRef(Buffer&&) = delete;

  // The element at row `i`, column `j`.
  //
  // Throws std::out_of_range if `i` is at or above `R` or `j` at or above `C`,
  // for the reason an owned matrix does: the offset of a position outside the
  // matrix is another position's offset.
  constexpr int8_t get(std::size_t i, std::size_t j) const {
    if (i >= R || j >= C) {
      throw std::out_of_range("tagma-matrix: position outside the matrix");
    }
    return read(i, j);
  }

  // The element at a position already known to be inside the matrix.
  constexpr int8_t element(std::size_t i, std::size_t j) const {
    return read(i, j);
  }

  // The Rust type derives PartialEq over the borrowed slice; a view compares
  // the bytes it sees, not the address it sees them at.
  friend bool operator==(const MatrixRef& left, const MatrixRef& right) {
    return *left.data_ == *right.data_;
  }
  friend bool operator!=(const MatrixRef& left, const MatrixRef& right) {
    return !(left == right);
  }

 private:
  // The element at a position already known to be inside the matrix.
  constexpr int8_t read(std::size_t i, std::size_t j) const {
    const std::size_t offset = O::offset(R, C, i, j);
    return (*data_)[offset / C][offset % C];
  }

  const Buffer* data_;
};

}  // namespace tagma_matrix
