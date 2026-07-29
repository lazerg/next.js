use anyhow::Result;
use turbo_rcstr::RcStr;
use turbo_tasks::{ReadRef, TryJoinIterExt, Vc};
use turbo_tasks_hash::{Xxh3Hash64Hasher, encode_base64};
use turbopack_core::version::Version;

use crate::chunk_hmr::version::EcmascriptChunkVersion;

/// The version of a [`super::content::EcmascriptMergedChunkContent`]. This is
/// essentially a composite [`EcmascriptChunkVersion`].
#[turbo_tasks::value(serialization = "skip", shared)]
pub struct EcmascriptMergedChunkVersion {
    #[turbo_tasks(trace_ignore)]
    pub versions: Vec<ReadRef<EcmascriptChunkVersion>>,
}

#[turbo_tasks::value_impl]
impl Version for EcmascriptMergedChunkVersion {
    #[turbo_tasks::function]
    async fn id(&self) -> Result<Vc<RcStr>> {
        let mut hasher = Xxh3Hash64Hasher::new();
        hasher.write_value(self.versions.len());
        let sorted_ids = {
            let mut sorted_ids = self
                .versions
                .iter()
                // This `ReadRef::cell` call is important: it ensures the id is
                // computed from a cell, so it is cached.
                .map(|version| ReadRef::cell(version.clone()).id())
                .try_join()
                .await?;
            sorted_ids.sort();
            sorted_ids
        };
        for id in sorted_ids {
            hasher.write_value(id);
        }
        let hash = hasher.finish();
        let hash = encode_base64(hash);
        Ok(Vc::cell(hash.into()))
    }
}
