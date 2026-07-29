use anyhow::{Result, bail};
use turbo_tasks::{ResolvedVc, TryJoinIterExt, Vc};
use turbopack_core::version::{VersionedContent, VersionedContentMerger, VersionedContents};

use crate::chunk_hmr::{
    content::{EcmascriptChunkPlatform, EcmascriptHmrChunkContent},
    merged_content::EcmascriptMergedChunkContent,
};

/// Merges multiple [`EcmascriptHmrChunkContent`] into a single
/// [`EcmascriptMergedChunkContent`]. This allows the chunk list to produce a
/// single `EcmascriptMergedUpdate` for multiple chunks updating at the same time.
///
/// The `platform` field is load-bearing, not informational: chunk lists group
/// chunks by merger cell identity, so a single merger shared across runtimes
/// would merge browser and node chunks into one update.
#[turbo_tasks::value]
pub struct EcmascriptChunkContentMerger {
    platform: EcmascriptChunkPlatform,
}

#[turbo_tasks::value_impl]
impl EcmascriptChunkContentMerger {
    #[turbo_tasks::function]
    pub fn new(platform: EcmascriptChunkPlatform) -> Vc<Self> {
        Self::cell(EcmascriptChunkContentMerger { platform })
    }
}

#[turbo_tasks::value_impl]
impl VersionedContentMerger for EcmascriptChunkContentMerger {
    #[turbo_tasks::function]
    async fn merge(
        &self,
        contents: Vc<VersionedContents>,
    ) -> Result<Vc<Box<dyn VersionedContent>>> {
        let contents = contents
            .await?
            .iter()
            .map(|content| async move {
                if let Some(content) =
                    ResolvedVc::try_sidecast::<Box<dyn EcmascriptHmrChunkContent>>(*content)
                {
                    Ok(content)
                } else {
                    bail!("expected Vc<Box<dyn EcmascriptHmrChunkContent>>")
                }
            })
            .try_join()
            .await?;

        Ok(Vc::upcast(EcmascriptMergedChunkContent { contents }.cell()))
    }
}
