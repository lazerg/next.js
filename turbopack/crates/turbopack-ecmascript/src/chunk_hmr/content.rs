use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use turbo_tasks::{Vc, trace::TraceRawVcs};
use turbopack_core::version::VersionedContent;

use crate::{chunk::EcmascriptChunkContentEntries, chunk_hmr::version::EcmascriptHmrChunkVersion};

/// Which runtime a chunk was produced for.
///
/// Only used to keep otherwise-identical per-runtime cells distinct. In
/// particular the [`super::merged::merger::EcmascriptHmrChunkContentMerger`]
/// must not be a singleton across runtimes, or a chunk list containing both
/// browser and node chunks would merge them into a single update.
#[turbo_tasks::task_input]
#[derive(
    Eq, PartialEq, Debug, Clone, Copy, Hash, TraceRawVcs, Serialize, Deserialize, Encode, Decode,
)]
#[serde(rename_all = "kebab-case")]
pub enum EcmascriptChunkPlatform {
    Browser,
    NodeJs,
}

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
    fn hmr_entries(self: Vc<Self>) -> Vc<EcmascriptChunkContentEntries>;

    /// This chunk's own version, used as the `to` side of a diff.
    #[turbo_tasks::function]
    fn own_hmr_version(self: Vc<Self>) -> Vc<EcmascriptHmrChunkVersion>;
}
