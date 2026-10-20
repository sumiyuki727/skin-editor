//! Fixed-shape texture ownership experiment. Asset keys and physical texture IDs
//! are distinct. Reuse never overwrites a pinned view's texture.
use crate::Decoded;
use eframe::egui;
use std::collections::{BTreeMap, BTreeSet};

struct Entry {
    handle: egui::TextureHandle,
    used: u64,
}
pub struct TextureCache {
    entries: BTreeMap<usize, Entry>,
    idle: Vec<egui::TextureHandle>,
    size: [usize; 2],
    capacity: usize,
    reuse: bool,
    clock: u64,
    pub peak_bytes: usize,
    pub created: usize,
    pub updated: usize,
}
impl TextureCache {
    pub fn new(budget: usize, size: [usize; 2], reuse: bool) -> Self {
        assert!(!size.contains(&0) && size.iter().all(|&n| n <= 4096));
        let bytes = size[0] * size[1] * 4;
        assert!(budget >= bytes);
        Self {
            entries: BTreeMap::new(),
            idle: Vec::new(),
            size,
            capacity: budget / bytes,
            reuse,
            clock: 0,
            peak_bytes: 0,
            created: 0,
            updated: 0,
        }
    }
    pub fn bytes(&self) -> usize {
        (self.entries.len() + self.idle.len()) * self.size[0] * self.size[1] * 4
    }
    pub fn idle_bytes(&self) -> usize {
        self.idle.len() * self.size[0] * self.size[1] * 4
    }
    pub fn get(&mut self, id: usize) -> Option<&egui::TextureHandle> {
        self.clock += 1;
        self.entries.get_mut(&id).map(|entry| {
            entry.used = self.clock;
            &entry.handle
        })
    }
    pub fn upload(
        &mut self,
        ctx: &egui::Context,
        id: usize,
        image: Decoded,
        pinned: &BTreeSet<usize>,
    ) -> Result<egui::TextureId, String> {
        if image.size != self.size || self.entries.contains_key(&id) {
            return Err("Texture shape changed or duplicate asset binding".into());
        }
        let mut recycled = self.idle.pop();
        if self.entries.len() == self.capacity {
            let victim = self
                .entries
                .iter()
                .filter(|(key, _)| !pinned.contains(key))
                .min_by_key(|(_, e)| e.used)
                .map(|(&key, _)| key)
                .ok_or("All texture slots pinned")?;
            let entry = self.entries.remove(&victim).unwrap();
            if self.reuse {
                recycled = Some(entry.handle);
            }
        }
        let handle = if let Some(mut handle) = recycled {
            // A full delta recreates the backend texture. A full-size partial
            // delta preserves allocation, view and bind group with these options.
            handle.set_partial([0, 0], image, egui::TextureOptions::LINEAR);
            self.updated += 1;
            handle
        } else {
            self.created += 1;
            ctx.load_texture(
                format!("original-slot-{}", self.created),
                image,
                egui::TextureOptions::LINEAR,
            )
        };
        let texture = handle.id();
        self.clock += 1;
        self.entries.insert(
            id,
            Entry {
                handle,
                used: self.clock,
            },
        );
        self.peak_bytes = self.peak_bytes.max(self.bytes());
        Ok(texture)
    }
    /// Drops logical asset bindings. In reuse mode the pool remains owned and
    /// counted within the same byte cap; closing must call clear instead.
    pub fn reset_bindings(&mut self) {
        if self.reuse {
            self.idle.extend(
                std::mem::take(&mut self.entries)
                    .into_values()
                    .map(|e| e.handle),
            );
        } else {
            self.clear();
        }
    }
    pub fn clear(&mut self) {
        self.entries.clear();
        self.idle.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    fn image(color: egui::Color32) -> Decoded {
        Arc::new(egui::ColorImage::filled([1, 1], color))
    }
    #[test]
    fn reuse_preserves_pins_and_fails_atomically_when_all_pinned() {
        let ctx = egui::Context::default();
        let mut cache = TextureCache::new(8, [1, 1], true);
        let none = BTreeSet::new();
        let a = cache
            .upload(&ctx, 0, image(egui::Color32::RED), &none)
            .unwrap();
        let b = cache
            .upload(&ctx, 1, image(egui::Color32::GREEN), &none)
            .unwrap();
        let _ = ctx.run_ui(Default::default(), |_| {});
        cache.get(0);
        let c = cache
            .upload(&ctx, 2, image(egui::Color32::BLUE), &BTreeSet::from([0]))
            .unwrap();
        assert_eq!(c, b);
        assert_eq!(cache.get(0).unwrap().id(), a);
        assert!(cache.get(1).is_none());
        assert!(
            cache
                .upload(
                    &ctx,
                    3,
                    image(egui::Color32::WHITE),
                    &BTreeSet::from([0, 2])
                )
                .is_err()
        );
        assert_eq!(cache.get(2).unwrap().id(), b);
        assert_eq!((cache.created, cache.updated, cache.bytes()), (2, 1, 8));
    }
    #[test]
    fn reset_counts_idle_slots_partial_update_shares_pixels_and_close_frees() {
        let ctx = egui::Context::default();
        let mut cache = TextureCache::new(4, [1, 1], true);
        let id = cache
            .upload(&ctx, 0, image(egui::Color32::RED), &BTreeSet::new())
            .unwrap();
        let _ = ctx.run_ui(Default::default(), |_| {});
        cache.reset_bindings();
        assert!(cache.get(0).is_none());
        assert_eq!(cache.idle_bytes(), 4);
        let pixels = image(egui::Color32::BLUE);
        assert_eq!(
            cache
                .upload(&ctx, 1, pixels.clone(), &BTreeSet::new())
                .unwrap(),
            id
        );
        let out = ctx.run_ui(Default::default(), |_| {});
        let delta = &out.textures_delta.set.get(&id).unwrap()[0];
        assert_eq!(delta.pos, Some([0, 0]));
        let egui::ImageData::Color(queued) = &delta.image;
        assert!(Arc::ptr_eq(queued, &pixels));
        let wrong = Arc::new(egui::ColorImage::filled([2, 1], egui::Color32::WHITE));
        assert!(cache.upload(&ctx, 2, wrong, &BTreeSet::new()).is_err());
        assert_eq!(cache.get(1).unwrap().id(), id);
        cache.clear();
        assert_eq!(cache.bytes(), 0);
        let out = ctx.run_ui(Default::default(), |_| {});
        assert!(out.textures_delta.free.contains(&id));
    }
    #[test]
    fn baseline_discards_evicted_textures_and_resets() {
        let ctx = egui::Context::default();
        let mut cache = TextureCache::new(4, [1, 1], false);
        let a = cache
            .upload(&ctx, 0, image(egui::Color32::RED), &BTreeSet::new())
            .unwrap();
        let _ = ctx.run_ui(Default::default(), |_| {});
        let b = cache
            .upload(&ctx, 1, image(egui::Color32::BLUE), &BTreeSet::new())
            .unwrap();
        assert_ne!(a, b);
        cache.reset_bindings();
        assert_eq!(cache.bytes(), 0);
        assert_eq!((cache.created, cache.updated), (2, 0));
    }
}
