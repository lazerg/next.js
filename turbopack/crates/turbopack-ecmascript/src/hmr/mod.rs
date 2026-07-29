use turbo_tasks::Vc;
use turbopack_core::version::VersionedContent;

use crate::{chunk::EcmascriptChunkContentEntries, hmr::chunk_version::EcmascriptChunkVersion};

/// Diffing and merging of Ecmascript chunk contents for hot module replacement.
///
/// The `chunk_*` modules operate on a single chunk; the unprefixed modules
/// operate on a merged set of chunks, which is what a chunk list produces a
/// single update from.
pub mod chunk_update;
pub mod chunk_version;
pub mod content;
pub mod merger;
pub mod update;
pub mod version;

/// An Ecmascript chunk content that participates in HMR.
///
/// Exists so the version/diff/merge machinery can be written once against the
/// trait rather than duplicated per runtime. The turbo-tasks value types cannot
/// be generic, so the runtimes keep their own content structs and implement this
/// to expose the two things the shared machinery needs.
#[turbo_tasks::value_trait]
pub trait EcmascriptHmrChunkContent: VersionedContent {
    /// The per-module code and hashes making up this chunk.
    #[turbo_tasks::function]
    fn entries(self: Vc<Self>) -> Vc<EcmascriptChunkContentEntries>;

    /// This chunk's own version, used as the `to` side of a diff.
    #[turbo_tasks::function]
    fn own_version(self: Vc<Self>) -> Vc<EcmascriptChunkVersion>;
}
