#pragma once

// tagma_matrix: what a matrix's elements have in common. Mirrors
// sw/rust/matrix/src/elements.rs.
//
// The Rust `Elements` trait is a CRTP base here. The derived type supplies
// `element`, and the base holds the trait's default methods (`path_of`, `at`,
// `encoded_len`, `encode_into`) so they are written once for both matrix types.
//
// The base declares no virtual function and no data member, which is what keeps
// a matrix the value the Rust type is: a literal, trivially copyable type with
// no pointer in it, so a `constexpr` weight table can be a matrix and its bytes
// are its elements.
//
// A derived type must define `element`. The forwarder below would otherwise
// find itself through `derived()` and recurse.

#include "tagma_core/coord.h"
#include "tagma_core/coord_path.h"
#include "tagma_matrix/wire.h"

#include <array>
#include <cstddef>
#include <cstdint>
#include <optional>
#include <stdexcept>

namespace tagma_matrix {

// The coordinate of a zero-based index.
//
// The bound guards the cast to sixteen bits, and it is the runtime backstop for
// an implementor that declares dimensions outside the coordinate space without
// calling any method that would report it. Throws std::out_of_range where the
// Rust function panics.
inline tagma::Coord coord_of(std::size_t index) {
  if (index < static_cast<std::size_t>(tagma::Coord::kNValid)) {
    const std::optional<tagma::Coord> coord =
        tagma::Coord::from_index(static_cast<uint16_t>(index));
    if (coord.has_value()) return *coord;
  }
  throw std::out_of_range("tagma-matrix: index exceeds the coordinate space");
}

// Reads the elements of a rank-2 matrix, and writes it out.
//
// Implemented by an owned matrix and by one that borrows a buffer, so a product
// reads weights that live in flash without copying them into RAM first, and a
// flash-resident matrix is sent on without a copy as well.
//
// An implementor must bound both dimensions by the coordinate space, because an
// element's address is a pair of `tagma::Coord`: an index at or above
// `tagma::Coord::kNValid` has no coordinate to be named by. The static
// assertion below reports a violation at build time, as the Rust `const`
// assertion does.
//
// `path_of` and `at` are not `constexpr` because `tagma::CoordPath` is not, and
// that type belongs to `tagma_core`. The matrix itself is: a weight table read
// at compile time goes through `get` or `element`, which are.
template <class Derived, std::size_t R, std::size_t C, class O>
class Elements {
 public:
  static_assert(R <= static_cast<std::size_t>(tagma::Coord::kNValid) &&
                    C <= static_cast<std::size_t>(tagma::Coord::kNValid) &&
                    R > 0 && C > 0,
                "tagma-matrix: a dimension is outside the coordinate space");

  // The element at row `i`, column `j`.
  //
  // `i` must be below `R` and `j` below `C`. A position outside the matrix has
  // no element of its own: its flat offset is another position's, so an
  // unchecked read answers with a value that belongs to a different coordinate.
  // The methods below, the shipped product, and the shipped implementors'
  // public readers stay inside the range.
  constexpr int8_t element(std::size_t i, std::size_t j) const {
    return derived().element(i, j);
  }

  // The coordinate of the element at row `i`, column `j`.
  //
  // An empty optional when the position lies outside the matrix.
  std::optional<tagma::CoordPath<2>> path_of(std::size_t i,
                                             std::size_t j) const {
    if (i < R && j < C) {
      return tagma::CoordPath<2>::from_array({coord_of(i), coord_of(j)});
    }
    return std::nullopt;
  }

  // The element a coordinate names.
  //
  // An empty optional when the coordinate names no element of this matrix.
  std::optional<int8_t> at(const tagma::CoordPath<2>& path) const {
    const std::array<tagma::Coord, 2>& coords = path.coords();
    const std::size_t i = coords[0].index();
    const std::size_t j = coords[1].index();
    if (i < R && j < C) return element(i, j);
    return std::nullopt;
  }

  // The bytes a stream of this matrix occupies.
  static constexpr std::size_t encoded_len() { return detail::encoded_len<R, C>(); }

  // Writes the matrix as its coordinates and their values.
  //
  // Elements are written in logical order, so two matrices with the same
  // logical content write the same bytes whatever order their own storage uses.
  // The operation is here rather than on the owned type because reading is all
  // it takes, and a matrix whose weights live in flash must not be copied into
  // RAM to be sent.
  //
  // An empty optional on success, having written exactly `encoded_len()`
  // bytes; the refusal otherwise.
  std::optional<EncodeError> encode_into(uint8_t* out,
                                         std::size_t out_len) const {
    const std::size_t needed = encoded_len();
    if (out_len < needed) {
      return EncodeError{needed, out_len};
    }

    out[0] = detail::kMagic[0];
    out[1] = detail::kMagic[1];
    out[2] = detail::kMagic[2];
    out[3] = detail::kMagic[3];
    out[4] = detail::kVersion;
    detail::write_le16(out + 5, static_cast<uint16_t>(R));
    detail::write_le16(out + 7, static_cast<uint16_t>(C));

    std::size_t cursor = detail::kHeaderLen;
    for (std::size_t i = 0; i < R; ++i) {
      for (std::size_t j = 0; j < C; ++j) {
        detail::write_le16(out + cursor, coord_of(i).index());
        detail::write_le16(out + cursor + 2, coord_of(j).index());
        out[cursor + 4] = static_cast<uint8_t>(element(i, j));
        cursor += detail::kRecordLen;
      }
    }

    return std::nullopt;
  }

 private:
  constexpr const Derived& derived() const {
    return static_cast<const Derived&>(*this);
  }
};

}  // namespace tagma_matrix
