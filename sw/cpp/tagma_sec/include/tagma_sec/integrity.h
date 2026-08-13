#pragma once

// Integrity: tamper-evident binding of a record to its path, principal,
// and epoch. Mirrors sw/rust/sec/src/integrity.rs.
//
// The seal is a keyed commitment. It does not hide the record; it makes
// modification evident. Implementations differ in whether the epoch and the
// principal participate in the binding (the legacy pattern ignores them).

#include "tagma_sec/types.h"

#include <optional>

namespace tagma_sec {

class Integrity {
 public:
  virtual ~Integrity() = default;

  // Binds `record` to `path`, `principal`, and `epoch`.
  virtual Seal seal(const Bytes& record, const Path& path,
                    PrincipalId principal, Epoch epoch) = 0;

  // Accepts only seals that match all bound inputs.
  virtual bool verify(const Bytes& record, const Path& path,
                      PrincipalId principal, Epoch epoch,
                      const Seal& seal) = 0;

  // Re-binds an existing seal to a newer epoch without altering the record.
  // Returns nullopt when the input seal does not verify.
  virtual std::optional<Seal> refresh(const Bytes& record, const Path& path,
                                      PrincipalId principal, Epoch from_epoch,
                                      Epoch to_epoch, const Seal& seal) = 0;
};

}  // namespace tagma_sec
