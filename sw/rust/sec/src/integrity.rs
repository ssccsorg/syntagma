//! Integrity: tamper-evident binding of a record to its path, principal,
//! and epoch.
//!
//! The seal is a keyed commitment. It does not hide the record; it makes
//! modification evident. Implementations differ in whether the epoch and the
//! principal participate in the binding (the legacy pattern ignores them).

use crate::types::{Epoch, Path, PrincipalId, Seal};

pub trait Integrity {
    /// Binds `record` to `path`, `principal`, and `epoch`.
    fn seal(&self, record: &[u8], path: &Path, principal: PrincipalId, epoch: Epoch) -> Seal;

    /// Accepts only seals that match all bound inputs.
    fn verify(
        &self,
        record: &[u8],
        path: &Path,
        principal: PrincipalId,
        epoch: Epoch,
        seal: &Seal,
    ) -> bool;

    /// Re-binds an existing seal to a newer epoch without altering the
    /// record. Returns `None` when the input seal does not verify.
    fn refresh(
        &self,
        record: &[u8],
        path: &Path,
        principal: PrincipalId,
        from_epoch: Epoch,
        to_epoch: Epoch,
        seal: &Seal,
    ) -> Option<Seal>;
}
