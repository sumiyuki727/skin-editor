//! Small read-only fixture loader and byte-bounded caches for P0, not product APIs.
pub mod audio_output;
pub mod audio_transport;
pub mod composition;
pub mod composition_fixture;
pub mod composition_gpu;
pub mod composition_loading;
pub mod composition_switch;
pub mod composition_view;
pub mod desktop_view;
pub mod interaction;
pub mod interaction_view;
pub mod loading;
#[cfg(all(windows, feature = "media-proof"))]
pub mod streaming_decode;
pub mod texture_cache;
pub mod upload_fence;
pub mod view;
#[cfg(all(windows, feature = "device-audio"))]
pub mod windows_audio;
use image::{ImageFormat, ImageReader};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Cursor,
    path::{Path, PathBuf},
    sync::Arc,
};

pub const CACHE_BYTES: usize = 128 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct Asset {
    pub path: PathBuf,
    pub thumbnail: PathBuf,
    pub size: [u32; 2],
}
#[derive(Clone, Copy, Debug)]
pub struct Layer {
    pub asset: usize,
    pub x: f32,
    pub y: f32,
    pub scale: f32,
    pub opacity: f32,
}
pub struct Workload {
    pub assets: Vec<Asset>,
    pub documents: Vec<Vec<Layer>>,
}

fn number(value: &Value, key: &str) -> Result<f32, String> {
    value[key]
        .as_f64()
        .filter(|v| v.is_finite())
        .map(|v| v as f32)
        .filter(|v| v.is_finite())
        .ok_or_else(|| format!("Invalid {key}"))
}
fn resolve(root: &Path, value: &Value) -> Result<PathBuf, String> {
    let relative = value.as_str().ok_or("Missing resource path")?;
    let path = root
        .join(relative)
        .canonicalize()
        .map_err(|e| e.to_string())?;
    if !path.starts_with(root) || !path.is_file() {
        return Err("Resource outside fixture root".into());
    }
    Ok(path)
}
fn flatten(
    nodes: &Value,
    parent: [f32; 4],
    depth: usize,
    assets: usize,
    out: &mut Vec<Layer>,
) -> Result<(), String> {
    if depth > 8 {
        return Err("Group nesting limit".into());
    }
    for node in nodes.as_array().ok_or("Missing children")? {
        let local = [
            number(node, "x")?,
            number(node, "y")?,
            number(node, "scale")?,
            number(node, "opacity")?,
        ];
        if local[2] <= 0.0 || local[2] > 8.0 || !(0.0..=1.0).contains(&local[3]) {
            return Err("Invalid transform".into());
        }
        let state = [
            parent[0] + local[0] * parent[2],
            parent[1] + local[1] * parent[2],
            parent[2] * local[2],
            parent[3] * local[3],
        ];
        if state.iter().any(|v| !v.is_finite()) {
            return Err("Transform overflow".into());
        }
        if node.get("children").is_some() {
            flatten(&node["children"], state, depth + 1, assets, out)?;
        } else {
            let asset = node["asset"].as_u64().ok_or("Missing asset")? as usize;
            if asset >= assets || out.len() >= 5000 {
                return Err("Layer or reference limit".into());
            }
            out.push(Layer {
                asset,
                x: state[0],
                y: state[1],
                scale: state[2],
                opacity: state[3],
            });
        }
    }
    Ok(())
}
impl Workload {
    pub fn open(root: &Path) -> Result<Self, String> {
        let root = root.canonicalize().map_err(|e| e.to_string())?;
        let data = std::fs::read(root.join("workload.json")).map_err(|e| e.to_string())?;
        if data.len() > 2 * 1024 * 1024 {
            return Err("Manifest too large".into());
        }
        let value: Value = serde_json::from_slice(&data).map_err(|e| e.to_string())?;
        if value["schema_version"] != 1 {
            return Err("Unsupported fixture version".into());
        }
        let mut assets = Vec::new();
        for (index, entry) in value["assets"]
            .as_array()
            .ok_or("Missing assets")?
            .iter()
            .enumerate()
        {
            if index >= 1000 || entry["id"].as_u64() != Some(index as u64) {
                return Err("Invalid asset identity".into());
            }
            let path = resolve(&root, &entry["path"])?;
            let size = image::image_dimensions(&path).map_err(|e| e.to_string())?;
            if entry["width"].as_u64() != Some(u64::from(size.0))
                || entry["height"].as_u64() != Some(u64::from(size.1))
                || size.0 > 4096
                || size.1 > 4096
            {
                return Err("Unexpected PNG dimensions".into());
            }
            assets.push(Asset {
                path,
                thumbnail: resolve(&root, &entry["thumbnail"])?,
                size: [size.0, size.1],
            });
        }
        let mut documents = Vec::new();
        for document in value["documents"].as_array().ok_or("Missing documents")? {
            if documents.len() >= 100 {
                return Err("Document limit".into());
            }
            let mut layers = Vec::new();
            flatten(
                &document["children"],
                [0.0, 0.0, 1.0, 1.0],
                0,
                assets.len(),
                &mut layers,
            )?;
            documents.push(layers);
        }
        if assets.is_empty() || documents.is_empty() {
            return Err("Empty fixture".into());
        }
        Ok(Self { assets, documents })
    }
}

pub fn decode_png(path: &Path) -> Result<egui_image::ColorImage, String> {
    decode_png_with_limit(path, [4096, 4096])
}
pub fn decode_png_sized(path: &Path, size: [usize; 2]) -> Result<egui_image::ColorImage, String> {
    if size.contains(&0) || size.iter().any(|&n| n > 4096) {
        return Err("Invalid expected PNG dimensions".into());
    }
    let image = decode_png_with_limit(path, [size[0] as u32, size[1] as u32])?;
    if image.size != size {
        return Err("PNG dimensions changed since request".into());
    }
    Ok(image)
}
fn decode_png_with_limit(path: &Path, maximum: [u32; 2]) -> Result<egui_image::ColorImage, String> {
    if std::fs::metadata(path).map_err(|e| e.to_string())?.len() > 16 * 1024 * 1024 {
        return Err("Compressed PNG limit".into());
    }
    let data = std::fs::read(path).map_err(|e| e.to_string())?;
    let mut reader = ImageReader::with_format(Cursor::new(data), ImageFormat::Png);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(maximum[0]);
    limits.max_image_height = Some(maximum[1]);
    limits.max_alloc = Some(80 * 1024 * 1024);
    reader.limits(limits);
    let rgba = reader.decode().map_err(|e| e.to_string())?.into_rgba8();
    Ok(egui_image::ColorImage::from_rgba_unmultiplied(
        [rgba.width() as usize, rgba.height() as usize],
        rgba.as_raw(),
    ))
}
use eframe::egui as egui_image;

/// LRU ownership is bounded; callers must not retain evicted values indefinitely.
pub struct ByteCache<T> {
    entries: BTreeMap<usize, (T, usize, u64)>,
    pub bytes: usize,
    pub peak_bytes: usize,
    budget: usize,
    clock: u64,
}
impl<T> ByteCache<T> {
    pub fn new(budget: usize) -> Self {
        Self {
            entries: BTreeMap::new(),
            bytes: 0,
            peak_bytes: 0,
            budget,
            clock: 0,
        }
    }
    pub fn get(&mut self, id: usize) -> Option<&T> {
        self.clock += 1;
        self.entries.get_mut(&id).map(|entry| {
            entry.2 = self.clock;
            &entry.0
        })
    }
    pub fn insert(
        &mut self,
        id: usize,
        value: T,
        bytes: usize,
        pinned: &BTreeSet<usize>,
    ) -> Result<(), String> {
        if bytes > self.budget || self.entries.contains_key(&id) {
            return Err("Oversize or duplicate cache entry".into());
        }
        let reclaimable: usize = self
            .entries
            .iter()
            .filter(|(id, _)| !pinned.contains(id))
            .map(|(_, e)| e.1)
            .sum();
        if self.bytes + bytes > self.budget + reclaimable {
            return Err("Pinned entries exceed cache budget".into());
        }
        while self.bytes + bytes > self.budget {
            let victim = *self
                .entries
                .iter()
                .filter(|(id, _)| !pinned.contains(id))
                .min_by_key(|(_, e)| e.2)
                .ok_or("Pinned cache")?
                .0;
            self.bytes -= self.entries.remove(&victim).expect("existing victim").1;
        }
        self.clock += 1;
        self.entries.insert(id, (value, bytes, self.clock));
        self.bytes += bytes;
        self.peak_bytes = self.peak_bytes.max(self.bytes);
        Ok(())
    }
    pub fn clear(&mut self) {
        self.entries.clear();
        self.bytes = 0;
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
pub type Decoded = Arc<egui_image::ColorImage>;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cache_lru_sharing_pins_and_atomic_failure() {
        let mut cache = ByteCache::new(8);
        let empty = BTreeSet::new();
        cache.insert(1, Arc::new(10), 4, &empty).unwrap();
        cache.insert(2, Arc::new(20), 4, &empty).unwrap();
        let a = cache.get(1).unwrap().clone();
        assert!(Arc::ptr_eq(&a, cache.get(1).unwrap()));
        cache.insert(3, Arc::new(30), 4, &empty).unwrap();
        assert!(cache.get(2).is_none());
        let pins = BTreeSet::from([1, 3]);
        assert!(cache.insert(4, Arc::new(40), 4, &pins).is_err());
        assert_eq!(cache.bytes, 8);
        assert!(cache.get(1).is_some());
        assert!(cache.get(3).is_some());
        assert!(cache.insert(4, Arc::new(40), 9, &empty).is_err());
        cache.clear();
        assert_eq!(cache.bytes, 0);
    }
    #[test]
    fn nested_transforms_and_bad_references() {
        let nodes = serde_json::json!([{"x":4,"y":8,"scale":2,"opacity":0.5,"children":[{"asset":0,"x":3,"y":5,"scale":0.5,"opacity":0.8}]}]);
        let mut out = Vec::new();
        flatten(&nodes, [0., 0., 1., 1.], 0, 1, &mut out).unwrap();
        assert_eq!((out[0].x, out[0].y, out[0].scale), (10., 18., 1.));
        assert!((out[0].opacity - 0.4).abs() < 1e-6);
        assert!(flatten(&nodes, [0., 0., 1., 1.], 0, 0, &mut Vec::new()).is_err());
    }
    #[test]
    fn gui_loading_cache_cannot_replace_straight_pixels_or_isolated_groups() {
        use composition::{Affine, Image, Node, Sampling, compile, render};
        let layer = |asset, opacity| Node::Layer {
            asset,
            transform: Affine::IDENTITY,
            opacity,
            sampling: Sampling::Nearest,
        };
        let raw = [200, 80, 40, 128];
        let gui = egui_image::ColorImage::from_rgba_unmultiplied([1, 1], &raw);
        let sources = [Image {
            size: [1, 1],
            rgba: vec![raw],
        }];
        let gui_sources = [Image {
            size: [1, 1],
            rgba: vec![gui.pixels[0].to_array()],
        }];
        let straight = render(&compile([1, 1], &sources, &[layer(0, 1.)]).unwrap());
        let misinterpreted = render(&compile([1, 1], &gui_sources, &[layer(0, 1.)]).unwrap());
        assert_eq!(straight.rgba[0], raw);
        assert_ne!(misinterpreted.rgba[0], raw);

        let sources = [
            Image {
                size: [1, 1],
                rgba: vec![[255, 0, 0, 255]],
            },
            Image {
                size: [1, 1],
                rgba: vec![[0, 0, 255, 255]],
            },
        ];
        let isolated = [Node::Group {
            transform: Affine::IDENTITY,
            opacity: 0.5,
            children: vec![layer(0, 1.), layer(1, 1.)],
        }];
        let legacy = serde_json::json!([{"x":0,"y":0,"scale":1,"opacity":0.5,"children":[
            {"asset":0,"x":0,"y":0,"scale":1,"opacity":1},
            {"asset":1,"x":0,"y":0,"scale":1,"opacity":1}]}]);
        let mut flat = Vec::new();
        flatten(&legacy, [0., 0., 1., 1.], 0, 2, &mut flat).unwrap();
        let flat_nodes: Vec<_> = flat
            .iter()
            .map(|l| layer(l.asset, f64::from(l.opacity)))
            .collect();
        let grouped = render(&compile([1, 1], &sources, &isolated).unwrap());
        let flattened = render(&compile([1, 1], &sources, &flat_nodes).unwrap());
        assert_eq!(grouped.rgba[0], [0, 0, 255, 128]);
        assert_eq!(flattened.rgba[0][3], 191);
        assert_ne!(grouped.rgba, flattened.rgba);
    }
    #[test]
    fn png_pixels_errors_and_non_ascii_paths() {
        let dir = std::env::temp_dir().join(format!("p0-png-{}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("素材 日本語.png");
        let rgba = [255, 0, 0, 255, 0, 200, 0, 128];
        image::save_buffer(&path, &rgba, 2, 1, image::ColorType::Rgba8).unwrap();
        let pixels = decode_png(&path).unwrap();
        assert_eq!(pixels.size, [2, 1]);
        assert_eq!(pixels.pixels[0].to_array(), [255, 0, 0, 255]);
        // egui stores premultiplied pixels; alpha must not be silently discarded.
        assert_eq!(pixels.pixels[1].a(), 128);
        assert!(pixels.pixels[1].g() > 0 && pixels.pixels[1].g() < 200);
        std::fs::write(&path, b"corrupt PNG").unwrap();
        assert!(decode_png(&path).is_err());
        std::fs::remove_file(&path).unwrap();
        assert!(decode_png(&path).is_err());
        std::fs::remove_dir(&dir).unwrap();
    }
    #[test]
    fn manifest_load_validates_versions_references_and_resource_boundary() {
        let dir = std::env::temp_dir().join(format!("p0-manifest-{}", std::process::id()));
        let root = dir.join("root");
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("素材.png");
        image::save_buffer(&path, &[90, 80, 70, 255], 1, 1, image::ColorType::Rgba8).unwrap();
        let mut manifest = serde_json::json!({"schema_version":1,"assets":[{"id":0,"path":"素材.png","thumbnail":"素材.png","width":1,"height":1}],"documents":[{"children":[{"asset":0,"x":0,"y":0,"scale":1,"opacity":1}]}]});
        let save = |v: &Value| std::fs::write(root.join("workload.json"), v.to_string()).unwrap();
        save(&manifest);
        let loaded = Workload::open(&root).unwrap();
        assert_eq!(loaded.documents[0][0].asset, 0);
        let background_read = || {
            let (tx, rx) = std::sync::mpsc::channel();
            let mut task = loading::MetadataTask::new(root.clone(), move || tx.send(()).unwrap());
            rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
            task.poll().unwrap().0
        };
        assert_eq!(background_read().unwrap().documents[0][0].asset, 0);
        manifest["schema_version"] = json_value(2);
        save(&manifest);
        assert!(Workload::open(&root).is_err());
        assert!(background_read().is_err());
        manifest["schema_version"] = json_value(1);
        manifest["documents"][0]["children"][0]["asset"] = json_value(1);
        save(&manifest);
        assert!(Workload::open(&root).is_err());
        manifest["documents"][0]["children"][0]["asset"] = json_value(0);
        std::fs::copy(&path, dir.join("outside.png")).unwrap();
        manifest["assets"][0]["path"] = Value::String("../outside.png".into());
        save(&manifest);
        assert!(Workload::open(&root).is_err());
        manifest["assets"][0]["path"] = Value::String("missing.png".into());
        save(&manifest);
        assert!(Workload::open(&root).is_err());
        // Remove only explicit test-owned files, without recursive path deletion.
        for file in [path, root.join("workload.json"), dir.join("outside.png")] {
            std::fs::remove_file(file).unwrap();
        }
        std::fs::remove_dir(root).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }
    fn json_value(n: u32) -> Value {
        Value::from(n)
    }
}
