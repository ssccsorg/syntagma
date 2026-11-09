#pragma once

// tagma_matrix: an owned matrix. Mirrors sw/rust/matrix/src/matrix.rs and the
// owner-dependent half of sw/rust/matrix/src/wire.rs.
//
// `R` is the row count, `C` the column count, and `O` the physical order of the
// backing bytes. The order decides where an element sits and never which
// element an address names, so two matrices in different orders answer the same
// coordinate with the same value.
//
// Both dimensions are bounded by the coordinate space. A matrix larger than the
// space fails to compile rather than truncating an address.
//
// A copy duplicates the whole buffer, so a large matrix is a value to borrow or
// to read through `MatrixRef` rather than to pass around by value.
//
// Unlike the Rust type, which carries a `PhantomData<O>`, the order is a
// template parameter that changes the address arithmetic only, so two matrices
// of different orders are different types that answer the same coordinates.

#include "tagma_matrix/elements.h"
#include "tagma_matrix/order.h"
#include "tagma_matrix/wire.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <optional>
#include <stdexcept>

namespace tagma_matrix {

// A rank-2 matrix that owns its buffer.
template <std::size_t R, std::size_t C, class O = RowMajor>
class Matrix : public Elements<Matrix<R, C, O>, R, C, O> {
 public:
  using Buffer = std::array<std::array<int8_t, C>, R>;

  // A matrix of zeros, which is what the reader fills.
  constexpr Matrix() = default;

  // Wraps a buffer whose bytes are already in `O` order; the Rust
  // `Matrix::new`.
  constexpr explicit Matrix(const Buffer& data) : data_(data) {}

  // The element at row `i`, column `j`.
  //
  // Throws std::out_of_range if `i` is at or above `R` or `j` at or above `C`.
  // The flat offset of a position outside the matrix is another position's
  // offset, so reading it unchecked answers with a value that belongs to a
  // different coordinate.
  constexpr int8_t get(std::size_t i, std::size_t j) const {
    if (i >= R || j >= C) {
      throw std::out_of_range("tagma-matrix: position outside the matrix");
    }
    return read(i, j);
  }

  // Sets the element at row `i`, column `j`.
  //
  // Throws std::out_of_range if `i` is at or above `R` or `j` at or above `C`.
  constexpr void set(std::size_t i, std::size_t j, int8_t value) {
    if (i >= R || j >= C) {
      throw std::out_of_range("tagma-matrix: position outside the matrix");
    }
    write(i, j, value);
  }

  // The element at a position already known to be inside the matrix.
  //
  // The coordinate surface and the product call this, having established the
  // range themselves.
  constexpr int8_t element(std::size_t i, std::size_t j) const {
    return read(i, j);
  }

  // Sets the element at a position already known to be inside the matrix.
  //
  // The product over two matrices calls this once per element, having
  // established the range from the destination's own dimensions, so the check
  // belongs at the public boundary rather than inside the contraction. The Rust
  // method is crate-visible; C++ has no equivalent privacy across a free
  // function template, so it is public and documented as unchecked.
  constexpr void write(std::size_t i, std::size_t j, int8_t value) {
    const std::size_t offset = O::offset(R, C, i, j);
    data_[offset / C][offset % C] = value;
  }

  // Reads a matrix from a stream, placing every value at its coordinate.
  //
  // Reading in is the one direction that needs an owner, which is why it is
  // here and not on the base class. Returns an empty optional and fills
  // `error` with the reason when the stream cannot be accounted for; on
  // success `error` is left empty. The error is an out parameter rather than a
  // returned value because C++17 has no expected type, and the reason has to
  // survive for a caller that reports it.
  static std::optional<Matrix> decode(const uint8_t* src, std::size_t len,
                                      std::optional<DecodeError>& error) {
    if (len < detail::kHeaderLen) {
      error = DecodeError::too_short();
      return std::nullopt;
    }
    if (src[0] != detail::kMagic[0] || src[1] != detail::kMagic[1] ||
        src[2] != detail::kMagic[2] || src[3] != detail::kMagic[3]) {
      error = DecodeError::bad_magic();
      return std::nullopt;
    }
    if (src[4] != detail::kVersion) {
      error = DecodeError::unsupported_version(src[4]);
      return std::nullopt;
    }

    const std::size_t rows = detail::read_le16(src + 5);
    const std::size_t cols = detail::read_le16(src + 7);
    if (rows != R || cols != C) {
      error = DecodeError::shape_mismatch(rows, cols);
      return std::nullopt;
    }

    const std::size_t needed = detail::encoded_len<R, C>();
    if (len < needed) {
      error = DecodeError::truncated();
      return std::nullopt;
    }
    if (len > needed) {
      error = DecodeError::trailing_bytes(len - needed);
      return std::nullopt;
    }

    Matrix matrix;
    std::size_t cursor = detail::kHeaderLen;
    for (std::size_t i = 0; i < R; ++i) {
      for (std::size_t j = 0; j < C; ++j) {
        const std::size_t row = detail::read_le16(src + cursor);
        const std::size_t column = detail::read_le16(src + cursor + 2);
        // Modulo 2^8 written out: the narrowing conversion of a byte above 127
        // to int8 is implementation-defined before C++20, and the wire value
        // has to be exact on every platform.
        const uint8_t raw = src[cursor + 4];
        const int8_t value =
            raw <= 127 ? static_cast<int8_t>(raw)
                       : static_cast<int8_t>(static_cast<int>(raw) - 256);
        cursor += detail::kRecordLen;

        if (row >= R || column >= C) {
          error = DecodeError::coordinate_out_of_range();
          return std::nullopt;
        }
        if (row != i || column != j) {
          error = DecodeError::coordinate_mismatch();
          return std::nullopt;
        }

        matrix.set(row, column, value);
      }
    }

    error.reset();
    return matrix;
  }

  // The Rust type derives PartialEq over the buffer; the order carries no data.
  friend bool operator==(const Matrix& left, const Matrix& right) {
    return left.data_ == right.data_;
  }
  friend bool operator!=(const Matrix& left, const Matrix& right) {
    return !(left == right);
  }

 private:
  // The element at a position already known to be inside the matrix.
  constexpr int8_t read(std::size_t i, std::size_t j) const {
    const std::size_t offset = O::offset(R, C, i, j);
    return data_[offset / C][offset % C];
  }

  Buffer data_{};
};

}  // namespace tagma_matrix
