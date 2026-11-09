// The address surface: what a position outside the matrix does.
//
// A position outside the matrix has no element of its own, and its flat offset
// equals the offset of a position that is inside it. A reader that answered
// with that other element would return a value belonging to a different
// coordinate, which contradicts the one thing this library promises, so the
// readers refuse the position instead of resolving it.
//
// Mirrors sw/rust/matrix/tests/addressing.rs.

#include "common.h"
#include "harness.h"

namespace {

void a_column_one_past_the_last_refuses_to_be_read() {
  const auto a = common::row_major<2, 3>();
  harness::check_throws([&] { static_cast<void>(a.get(0, 3)); },
                        "position outside the matrix",
                        "a column one past the last is refused");
}

void a_row_one_past_the_last_refuses_to_be_read() {
  const auto a = common::row_major<2, 3>();
  harness::check_throws([&] { static_cast<void>(a.get(2, 0)); },
                        "position outside the matrix",
                        "a row one past the last is refused");
}

void a_column_one_past_the_last_refuses_to_be_read_in_column_major_order() {
  const auto a = common::col_major<2, 3>();
  harness::check_throws([&] { static_cast<void>(a.get(0, 3)); },
                        "position outside the matrix",
                        "a column one past the last is refused, column-major");
}

void a_position_outside_the_matrix_refuses_to_be_written() {
  auto a = common::row_major<2, 3>();
  harness::check_throws([&] { a.set(2, 0, 7); },
                        "position outside the matrix",
                        "a position outside the matrix is not written");
}

void a_borrowed_read_outside_the_matrix_is_refused() {
  const auto buffer = common::row_major_buffer<2, 3>();
  const tagma_matrix::MatrixRef<2, 3, tagma_matrix::RowMajor> a(buffer);
  harness::check_throws([&] { static_cast<void>(a.get(0, 3)); },
                        "position outside the matrix",
                        "a borrowed read outside the matrix is refused");
}

}  // namespace

int main() {
  a_column_one_past_the_last_refuses_to_be_read();
  a_row_one_past_the_last_refuses_to_be_read();
  a_column_one_past_the_last_refuses_to_be_read_in_column_major_order();
  a_position_outside_the_matrix_refuses_to_be_written();
  a_borrowed_read_outside_the_matrix_is_refused();
  return harness::finish("tagma_matrix addressing");
}
