#![feature(arbitrary_self_types_pointers)]
#![allow(clippy::needless_return)] // tokio macro-generated code doesn't respect this
#![cfg(test)]

use turbo_tasks::{ResolvedVc, Vc};
use turbo_tasks_testing::{Registration, register, run_once};
use turbopack_core::version::VersionedContentMerger;
use turbopack_ecmascript::chunk_hmr::{
    content::EcmascriptChunkPlatform, merged::merger::EcmascriptHmrChunkContentMerger,
};

static REGISTRATION: Registration = register!();

#[turbo_tasks::value(transparent)]
struct Mergers(Vec<ResolvedVc<Box<dyn VersionedContentMerger>>>);

/// Resolves the merger for both platforms, plus the browser one twice, so the
/// test can compare cell identity.
#[turbo_tasks::function(operation, root)]
async fn resolve_mergers() -> anyhow::Result<Vc<Mergers>> {
    let browser = ResolvedVc::upcast(
        EcmascriptHmrChunkContentMerger::new(EcmascriptChunkPlatform::Browser)
            .to_resolved()
            .await?,
    );
    let node = ResolvedVc::upcast(
        EcmascriptHmrChunkContentMerger::new(EcmascriptChunkPlatform::NodeJs)
            .to_resolved()
            .await?,
    );
    let browser_again = ResolvedVc::upcast(
        EcmascriptHmrChunkContentMerger::new(EcmascriptChunkPlatform::Browser)
            .to_resolved()
            .await?,
    );
    Ok(Vc::cell(vec![browser, node, browser_again]))
}

/// Chunk lists group chunks by `VersionedContentMerger` cell identity (see
/// `chunk_list::version::compute_chunk_list_version`). If the browser and node
/// mergers ever resolved to the same cell, a chunk list containing both would
/// merge them into a single `EcmascriptMergedUpdate`, shipping node module code
/// to the browser runtime and vice versa.
///
/// No end-to-end test covers this, because no fixture mixes both runtimes in one
/// chunk list. So assert the distinction directly.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn mergers_are_distinct_per_platform() {
    run_once(&REGISTRATION, || async {
        let mergers = resolve_mergers().read_strongly_consistent().await?;
        let (browser, node, browser_again) = (mergers[0], mergers[1], mergers[2]);

        assert_ne!(
            browser, node,
            "browser and node chunk content mergers must be distinct cells, otherwise chunk lists \
             would merge chunks across runtimes"
        );

        // Same platform must still dedup to one cell, so chunks of the same
        // runtime do get merged into a single update.
        assert_eq!(
            browser, browser_again,
            "mergers for the same platform must share one cell"
        );

        anyhow::Ok(())
    })
    .await
    .unwrap()
}
