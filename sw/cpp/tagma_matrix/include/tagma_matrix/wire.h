#pragma once

// tagma_matrix: the wire form of a matrix. Mirrors sw/rust/matrix/src/wire.rs.
//
// A matrix crosses to another device as its coordinates and their values. A
// receiver places each value at the coordinate it names, so the two sides may
// keep their bytes in different orders and still agree on every element.
//
// The layout and the constants describing it live here, inside `detail`: the
// Rust crate keeps the module private and re-exports only the two refusal
// types, so a consumer of this port has the same surface. The writing half is a
// method on the `Elements` base class, since reading the elements is all it
// takes; the reading half sits on `Matrix`, the one direction that needs an
// owner.
//
// This header holds no dependency on `Elements`, so `elements.h` may include
// it without a cycle.

#include <array>
#include <cstddef>
#include <cstdint>

namespace tagma_matrix {
namespace detail {

// The four bytes that open a stream.
constexpr std::array<uint8_t, 4> kMagic{static_cast<uint8_t>('T'),
                                        static_cast<uint8_t>('G'),
                                        static_cast<uint8_t>('M'),
                                        static_cast<uint8_t>('X')};

// The encoding this build writes, and the only one it reads.
constexpr uint8_t kVersion = 1;

// Magic, version, rows, and columns.
constexpr std::size_t kHeaderLen = 9;

// Two sixteen-bit coordinates and one value.
constexpr std::size_t kRecordLen = 5;

// Writes a sixteen-bit value little-endian, as the Rust `to_le_bytes` order.
inline void write_le16(uint8_t* out, uint16_t value) {
  out[0] = static_cast<uint8_t>(value & 0xFF);
  out[1] = static_cast<uint8_t>(value >> 8);
}

// Reads a sixteen-bit value little-endian.
inline uint16_t read_le16(const uint8_t* in) {
  return static_cast<uint16_t>(static_cast<uint16_t>(in[0]) |
                               (static_cast<uint16_t>(in[1]) << 8));
}

// The bytes a stream of an R by C matrix occupies.
template <std::size_t R, std::size_t C>
constexpr std::size_t encoded_len() {
  return kHeaderLen + R * C * kRecordLen;
}

}  // namespace detail

// Why a matrix could not be written.
//
// The Rust enum carries one variant, `BufferTooSmall { needed, available }`,
// so the port is the payload itself.
struct EncodeError {
  // The bytes the matrix needs.
  std::size_t needed = 0;
  // The bytes the destination offers.
  std::size_t available = 0;

  friend constexpr bool operator==(const EncodeError& left,
                                   const EncodeError& right) {
    return left.needed == right.needed && left.available == right.available;
  }
  friend constexpr bool operator!=(const EncodeError& left,
                                   const EncodeError& right) {
    return !(left == right);
  }
};

// Why a stream could not be read.
enum class DecodeErrorKind : uint8_t {
  // Fewer bytes than a header holds.
  TooShort,
  // The opening bytes belong to another format.
  BadMagic,
  // The version is not one this build reads.
  UnsupportedVersion,
  // The stream's shape is not the shape of the matrix being filled.
  ShapeMismatch,
  // The stream ends before its records do.
  Truncated,
  // The stream holds bytes the shape does not account for.
  TrailingBytes,
  // A record's coordinate names no element of the matrix.
  CoordinateOutOfRange,
  // A record's coordinate names another element than its own position.
  CoordinateMismatch,
};

// A refusal, with the payload the Rust variant carries.
class DecodeError {
 public:
  static DecodeError too_short() { return DecodeError(DecodeErrorKind::TooShort); }
  static DecodeError bad_magic() { return DecodeError(DecodeErrorKind::BadMagic); }
  static DecodeError unsupported_version(uint8_t version) {
    DecodeError error(DecodeErrorKind::UnsupportedVersion);
    error.version_ = version;
    return error;
  }
  static DecodeError shape_mismatch(std::size_t rows, std::size_t cols) {
    DecodeError error(DecodeErrorKind::ShapeMismatch);
    error.rows_ = rows;
    error.cols_ = cols;
    return error;
  }
  static DecodeError truncated() { return DecodeError(DecodeErrorKind::Truncated); }
  static DecodeError trailing_bytes(std::size_t extra) {
    DecodeError error(DecodeErrorKind::TrailingBytes);
    error.extra_ = extra;
    return error;
  }
  static DecodeError coordinate_out_of_range() {
    return DecodeError(DecodeErrorKind::CoordinateOutOfRange);
  }
  static DecodeError coordinate_mismatch() {
    return DecodeError(DecodeErrorKind::CoordinateMismatch);
  }

  DecodeErrorKind kind() const { return kind_; }

  // The version the stream declares; meaningful for UnsupportedVersion.
  uint8_t version() const { return version_; }
  // The shape the stream declares; meaningful for ShapeMismatch.
  std::size_t rows() const { return rows_; }
  std::size_t cols() const { return cols_; }
  // The bytes past the last record; meaningful for TrailingBytes.
  std::size_t extra() const { return extra_; }

  friend bool operator==(const DecodeError& left, const DecodeError& right) {
    if (left.kind_ != right.kind_) return false;
    switch (left.kind_) {
      case DecodeErrorKind::UnsupportedVersion:
        return left.version_ == right.version_;
      case DecodeErrorKind::ShapeMismatch:
        return left.rows_ == right.rows_ && left.cols_ == right.cols_;
      case DecodeErrorKind::TrailingBytes:
        return left.extra_ == right.extra_;
      default:
        return true;
    }
  }
  friend bool operator!=(const DecodeError& left, const DecodeError& right) {
    return !(left == right);
  }

 private:
  explicit DecodeError(DecodeErrorKind kind) : kind_(kind) {}

  DecodeErrorKind kind_;
  uint8_t version_ = 0;
  std::size_t rows_ = 0;
  std::size_t cols_ = 0;
  std::size_t extra_ = 0;
};

}  // namespace tagma_matrix
