#pragma once

// Channel: non-repudiation of origin and receipt. Mirrors
// sw/rust/sec/src/channel.rs.
//
// Evidence is signed with a keyed tag and verified by recomputation. A
// production deployment replaces this with a signature scheme over the module
// interface.

#include "tagma_sec/types.h"

namespace tagma_sec {

class Channel {
 public:
  virtual ~Channel() = default;

  // Signs evidence, producing a verifiable signed message.
  virtual SignedEvidence sign(const Bytes& evidence) = 0;

  // Accepts only signed evidence whose tag matches the evidence.
  virtual bool verify(const SignedEvidence& signed_evidence) = 0;

  // Produces a receipt binding `local`, `remote`, and `epoch`.
  virtual Receipt exchange(const Bytes& local, PrincipalId remote,
                           Epoch epoch) = 0;

  // Accepts only receipts whose signed evidence matches the remote and epoch
  // recorded on the receipt.
  virtual bool verify_receipt(const Receipt& receipt) = 0;
};

// Keyed-tag channel shared by the legacy and tagma-sec stacks.
class MacChannel : public Channel {
 public:
  SignedEvidence sign(const Bytes& evidence) override;
  bool verify(const SignedEvidence& signed_evidence) override;
  Receipt exchange(const Bytes& local, PrincipalId remote, Epoch epoch) override;
  bool verify_receipt(const Receipt& receipt) override;
};

}  // namespace tagma_sec
