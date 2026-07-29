use std::sync::Arc;

use anyhow::Result;
use turbo_tasks::{FxIndexMap, ReadRef, ResolvedVc, Vc};
use turbopack_core::{
    chunk::ModuleId,
    code_builder::Code,
    version::{PartialUpdate, TotalUpdate, Update, Version},
};

use crate::{
    chunk_hmr::{content::EcmascriptHmrChunkContent, version::EcmascriptChunkVersion},
    chunk_list::merged_update::{
        EcmascriptMergedChunkPartial, EcmascriptMergedChunkUpdate, EcmascriptMergedUpdate,
        EcmascriptModuleEntry,
    },
};

/// The module-level difference between two versions of a single chunk.
pub enum EcmascriptChunkUpdate {
    None,
    Partial {
        /// Added modules, keyed by id, with their content hash (used by the
        /// merged-chunk path to dedup code emission across chunks) and code.
        added: FxIndexMap<ModuleId, (u128, ResolvedVc<Code>)>,
        modified: FxIndexMap<ModuleId, ResolvedVc<Code>>,
        deleted: FxIndexMap<ModuleId, u128>,
    },
}

/// Diffs two versions of a single chunk's content.
///
/// Runtime-agnostic: both the browser and node paths, and the merged-chunk path
/// on top of them, share this one implementation.
pub async fn update_ecmascript_hmr_chunk_content(
    content: Vc<Box<dyn EcmascriptHmrChunkContent>>,
    to: &ReadRef<EcmascriptChunkVersion>,
    from: &ReadRef<EcmascriptChunkVersion>,
) -> Result<EcmascriptChunkUpdate> {
    let mut added = FxIndexMap::default();
    let mut modified = FxIndexMap::default();
    let mut deleted = FxIndexMap::default();

    // Lazily resolve the entries map only when we actually need to ship code
    // bytes for an added or modified module. For chunks that only have deletions
    // (or no changes that need code beyond hashes), this avoids materializing
    // any `Vc<Code>`.
    let mut entries_ref = None;

    // Check for deleted and modified modules
    for (id, from_hash) in &from.entries_hashes {
        if let Some(to_hash) = to.entries_hashes.get(id) {
            if *to_hash != *from_hash {
                // Module was modified
                let entries = match &entries_ref {
                    Some(entries) => entries,
                    None => entries_ref.insert(content.hmr_entries().await?),
                };
                if let Some(entry) = entries.get(id) {
                    modified.insert(id.clone(), entry.code);
                }
            }
        } else {
            // Module was deleted
            deleted.insert(id.clone(), *from_hash);
        }
    }

    // Check for added modules
    for (id, hash) in &to.entries_hashes {
        if !from.entries_hashes.contains_key(id) {
            let entries = match &entries_ref {
                Some(entries) => entries,
                None => entries_ref.insert(content.hmr_entries().await?),
            };
            if let Some(entry) = entries.get(id) {
                added.insert(id.clone(), (*hash, entry.code));
            }
        }
    }

    Ok(
        if added.is_empty() && modified.is_empty() && deleted.is_empty() {
            EcmascriptChunkUpdate::None
        } else {
            EcmascriptChunkUpdate::Partial {
                added,
                modified,
                deleted,
            }
        },
    )
}

/// Computes a standalone [`Update`] for a single chunk, wrapping the module diff
/// in an [`EcmascriptMergedUpdate`] payload.
///
/// This is the shared body of [`turbopack_core::version::VersionedContent::update`]
/// for HMR-capable Ecmascript chunks.
pub async fn update_ecmascript_hmr_chunk(
    content: Vc<Box<dyn EcmascriptHmrChunkContent>>,
    from_version: ResolvedVc<Box<dyn Version>>,
) -> Result<Update> {
    let to_version = content.own_version();
    let Some(from_version) = ResolvedVc::try_downcast_type::<EcmascriptChunkVersion>(from_version)
    else {
        // It's likely `from_version` is `NotFoundVersion`.
        return Ok(Update::Total(TotalUpdate {
            to: Vc::upcast::<Box<dyn Version>>(to_version)
                .into_trait_ref()
                .await?,
        }));
    };

    let to = to_version.await?;
    let from = from_version.await?;

    // When to and from point to the same value we can skip comparing them
    if from.ptr_eq(&to) {
        return Ok(Update::None);
    }

    let chunk_path = to.chunk_path.as_str();

    let mut merged_update = EcmascriptMergedUpdate::default();

    match update_ecmascript_hmr_chunk_content(content, &to, &from).await? {
        EcmascriptChunkUpdate::None => {
            return Ok(Update::None);
        }
        EcmascriptChunkUpdate::Partial {
            added,
            modified,
            deleted,
        } => {
            let mut partial = EcmascriptMergedChunkPartial::default();

            for (module_id, (_hash, module_code)) in added {
                partial.added.insert(module_id.clone());

                let entry =
                    EcmascriptModuleEntry::from_code(&module_id, *module_code, chunk_path).await?;
                merged_update.entries.insert(module_id, entry);
            }

            partial.deleted.extend(deleted.into_keys());

            for (module_id, module_code) in modified {
                let entry =
                    EcmascriptModuleEntry::from_code(&module_id, *module_code, chunk_path).await?;
                merged_update.entries.insert(module_id, entry);
            }

            merged_update
                .chunks
                .insert(chunk_path, EcmascriptMergedChunkUpdate::Partial(partial));
        }
    }

    Ok(if merged_update.is_empty() {
        Update::None
    } else {
        // The hot-reloader wraps this in ChunkListUpdate format for the runtime.
        let instruction_value = serde_json::to_value(&merged_update)?;

        Update::Partial(PartialUpdate {
            to: Vc::upcast::<Box<dyn Version>>(to_version)
                .into_trait_ref()
                .await?,
            instruction: Arc::new(instruction_value),
        })
    })
}
