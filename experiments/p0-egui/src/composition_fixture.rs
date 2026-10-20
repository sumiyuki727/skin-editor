//! Bounded read-only PNG fixture adapter shared by P0 proofs.
use crate::composition::{Affine, Image, Node, Sampling};
use std::{path::Path, sync::Arc};
pub fn load_document(fixture: &Path, document: usize) -> Result<(Arc<[Image]>, Vec<Node>), String> {
    let loaded = read_document(fixture, document, &|| false)?;
    Ok((loaded.sources, loaded.nodes))
}
pub fn document_spec(fixture: &Path, document: usize) -> Result<DocumentSpec, String> {
    let root = fixture.canonicalize().map_err(|e| e.to_string())?;
    let bytes = std::fs::read(root.join("workload.json")).map_err(|e| e.to_string())?;
    if bytes.len() > 2 * 1024 * 1024 {
        return Err("Fixture manifest too large".into());
    }
    let manifest: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let sources = manifest["assets"].as_array().ok_or("Missing assets")?;
    let documents = manifest["documents"]
        .as_array()
        .ok_or("Missing documents")?;
    if manifest["schema_version"] != 1 || sources.len() != 100 || documents.len() != 10 {
        return Err("Expected standard fixture v1".into());
    }
    let value = documents.get(document).ok_or("Missing document")?;
    let mut ids = std::collections::BTreeSet::new();
    let mut nodes = standard_nodes(&value["children"], &mut ids, 0, &mut 0)?;
    let ids: Vec<_> = ids.into_iter().collect();
    if ids.len() != 10 {
        return Err("Expected ten active sources".into());
    }
    let paths = ids
        .iter()
        .map(|&id| {
            if sources[id]["id"].as_u64() != Some(id as u64) {
                return Err("Fixture identity mismatch".into());
            }
            let path = root
                .join(sources[id]["path"].as_str().ok_or("Missing source path")?)
                .canonicalize()
                .map_err(|e| e.to_string())?;
            if !path.starts_with(&root) || !path.is_file() {
                return Err("Source boundary mismatch".into());
            }
            Ok(path)
        })
        .collect::<Result<Vec<_>, String>>()?;
    remap(&mut nodes, &ids);
    Ok(DocumentSpec { paths, nodes })
}

pub struct DocumentSpec {
    pub paths: Vec<std::path::PathBuf>,
    pub nodes: Vec<Node>,
}
pub struct LoadedDocument {
    pub sources: Arc<[Image]>,
    pub nodes: Vec<Node>,
}
/// Original straight RGBA; no GUI cache or opacity flattening.
pub fn decode_original(path: &Path) -> Result<Image, String> {
    use image::{ImageFormat, ImageReader};
    let mut reader = ImageReader::open(path).map_err(|e| e.to_string())?;
    reader.set_format(ImageFormat::Png);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(1024);
    limits.max_image_height = Some(1024);
    limits.max_alloc = Some(16 * 1024 * 1024);
    reader.limits(limits);
    let png = reader.decode().map_err(|e| e.to_string())?.into_rgba8();
    if png.dimensions() != (1024, 1024) {
        return Err("Source dimension mismatch".into());
    }
    Ok(Image {
        size: [1024, 1024],
        rgba: png
            .into_raw()
            .chunks_exact(4)
            .map(|p| [p[0], p[1], p[2], p[3]])
            .collect(),
    })
}
pub fn read_document(
    fixture: &Path,
    document: usize,
    cancelled: &dyn Fn() -> bool,
) -> Result<LoadedDocument, String> {
    if cancelled() {
        return Err("Cancelled document".into());
    }
    let spec = document_spec(fixture, document)?;
    let mut sources = Vec::new();
    for path in spec.paths {
        if cancelled() {
            return Err("Cancelled document".into());
        }
        sources.push(decode_original(&path)?);
        if cancelled() {
            return Err("Cancelled document".into());
        }
    }
    crate::composition::compile_with_limits(
        [1024, 1024],
        &sources,
        &spec.nodes,
        crate::composition::Limits::STANDARD,
    )?;
    Ok(LoadedDocument {
        sources: sources.into(),
        nodes: spec.nodes,
    })
}

/// Move the first leaf by a document-space delta, preserving its parent geometry.
pub fn translated_first(nodes: &[Node], delta: [f64; 2]) -> Result<Vec<Node>, String> {
    if delta.iter().any(|n| !n.is_finite()) {
        return Err("Nonfinite edit projection".into());
    }
    fn move_leaf(nodes: &mut [Node], parent: Affine, delta: [f64; 2]) -> Result<bool, String> {
        for node in nodes {
            match node {
                Node::Layer { transform, .. } => {
                    let inverse = parent.inverse()?;
                    let [a, b, c, d, _, _] = inverse.0;
                    transform.0[4] += a * delta[0] + c * delta[1];
                    transform.0[5] += b * delta[0] + d * delta[1];
                    return Ok(true);
                }
                Node::Group {
                    transform,
                    children,
                    ..
                } => {
                    if move_leaf(children, parent.then(*transform), delta)? {
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }
    let mut out = nodes.to_vec();
    if !move_leaf(&mut out, Affine::IDENTITY, delta)? {
        return Err("Missing editable leaf".into());
    }
    Ok(out)
}
pub fn standard_nodes(
    value: &serde_json::Value,
    ids: &mut std::collections::BTreeSet<usize>,
    depth: usize,
    count: &mut usize,
) -> Result<Vec<Node>, String> {
    if depth > 8 {
        return Err("Fixture group depth exceeded".into());
    }
    value
        .as_array()
        .ok_or("Missing fixture children")?
        .iter()
        .map(|v| {
            *count += 1;
            if *count > 128 {
                return Err("Fixture node limit".into());
            }
            let n = |key: &str| {
                v[key]
                    .as_f64()
                    .filter(|n| n.is_finite())
                    .ok_or_else(|| format!("Invalid fixture {key}"))
            };
            let transform = Affine([n("scale")?, 0., 0., n("scale")?, n("x")?, n("y")?]);
            let opacity = n("opacity")?;
            if let Some(children) = v.get("children") {
                Ok(Node::Group {
                    children: standard_nodes(children, ids, depth + 1, count)?,
                    transform,
                    opacity,
                })
            } else {
                let id = v["asset"]
                    .as_u64()
                    .filter(|&n| n < 100)
                    .ok_or("Invalid fixture asset")? as usize;
                ids.insert(id);
                Ok(Node::Layer {
                    asset: id,
                    transform,
                    opacity,
                    sampling: Sampling::Bilinear,
                })
            }
        })
        .collect()
}
pub fn remap(nodes: &mut [Node], ids: &[usize]) {
    for node in nodes {
        match node {
            Node::Layer { asset, .. } => *asset = ids.binary_search(asset).unwrap(),
            Node::Group { children, .. } => remap(children, ids),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::composition::{compile, render};
    #[test]
    fn original_decoder_preserves_straight_rgba_and_rejects_corruption_and_wrong_size() {
        let root = std::env::temp_dir().join(format!("p0-original-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let path = root.join("original.png");
        let pixels: Vec<_> = [[200, 80, 40, 128], [255, 10, 90, 0]]
            .into_iter()
            .cycle()
            .take(1024 * 1024)
            .flatten()
            .collect();
        image::save_buffer(&path, &pixels, 1024, 1024, image::ColorType::Rgba8).unwrap();
        let original = decode_original(&path).unwrap();
        assert_eq!(original.rgba[0], [200, 80, 40, 128]);
        assert_eq!(original.rgba[1], [255, 10, 90, 0]);
        image::save_buffer(&path, &[200, 80, 40, 128], 1, 1, image::ColorType::Rgba8).unwrap();
        assert!(decode_original(&path).is_err());
        std::fs::write(&path, b"corrupt PNG").unwrap();
        assert!(decode_original(&path).is_err());
        std::fs::remove_file(&path).unwrap();
        assert!(decode_original(&path).is_err());
        assert!(matches!(read_document(&root,0,&||true),Err(e) if e.contains("Cancelled")));
        std::fs::remove_dir(root).unwrap();
    }
    #[test]
    fn fixture_loader_rejects_missing_resource_without_rewriting_manifest() {
        let root = std::env::temp_dir().join(format!(
            "p0-composition-loader-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        assert!(load_document(&root, 0).is_err());
        let assets: Vec<_> = (0..100)
            .map(|id| serde_json::json!({"id":id,"path":format!("missing-{id}.png")}))
            .collect();
        let children: Vec<_> = (0..10)
            .map(|asset| serde_json::json!({"asset":asset,"x":0,"y":0,"scale":1,"opacity":1}))
            .collect();
        let doc = serde_json::json!({"children":children});
        let manifest = serde_json::to_vec(
            &serde_json::json!({"schema_version":1,"assets":assets,"documents":vec![doc;10]}),
        )
        .unwrap();
        std::fs::write(root.join("workload.json"), &manifest).unwrap();
        assert!(load_document(&root, 0).is_err());
        assert!(load_document(&root, 10).is_err());
        assert_eq!(std::fs::read(root.join("workload.json")).unwrap(), manifest);
        std::fs::remove_file(root.join("workload.json")).unwrap();
        std::fs::remove_dir(root).unwrap();
    }
    #[test]
    fn document_delta_respects_rotated_scaled_parent_and_leaves_other_instances_unchanged() {
        let leaf = Node::Layer {
            asset: 0,
            transform: Affine::IDENTITY,
            opacity: 1.,
            sampling: Sampling::Nearest,
        };
        let source = [Image {
            size: [1, 1],
            rgba: vec![[255; 4]],
        }];
        let nodes = vec![
            Node::Group {
                transform: Affine([0., 2., -2., 0., 4., 0.]),
                opacity: 1.,
                children: vec![leaf.clone()],
            },
            leaf,
        ];
        let before = render(&compile([8, 8], &source, &nodes).unwrap());
        let moved = translated_first(&nodes, [2., 0.]).unwrap();
        let after = render(&compile([8, 8], &source, &moved).unwrap());
        assert_eq!(before.rgba[2], [255; 4]);
        assert_eq!(after.rgba[2], [0; 4]);
        assert_eq!(after.rgba[4], [255; 4]);
        assert_eq!(before.rgba[0], after.rgba[0]);
        assert_eq!(
            render(&compile([8, 8], &source, &nodes).unwrap()).rgba,
            before.rgba
        );
        assert!(translated_first(&nodes, [f64::NAN, 0.]).is_err());
        assert!(translated_first(&[], [1., 1.]).is_err());
    }
}
