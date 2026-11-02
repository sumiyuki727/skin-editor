//! Conservative logical costs. Shared trees may be charged more than once.
use crate::CaptureError;
use skin_editor_domain::{
    Asset, AudioComponent, ComponentTree, ElementDocument, OutputMapping, ProjectDefinition,
    SkinConfiguration, SkinProject,
};

#[derive(Default)]
pub(crate) struct Cost(usize);
impl Cost {
    pub fn add(&mut self, bytes: usize) -> Result<(), CaptureError> {
        self.0 = self
            .0
            .checked_add(bytes)
            .ok_or(CaptureError::ResourceLimit("metadata"))?;
        Ok(())
    }
    pub fn count(&mut self, count: usize, each: usize) -> Result<(), CaptureError> {
        self.add(
            count
                .checked_mul(each)
                .ok_or(CaptureError::ResourceLimit("metadata"))?,
        )
    }
    pub fn bytes(&self) -> usize {
        self.0
    }
    pub fn asset(&mut self, a: &Asset) -> Result<(), CaptureError> {
        self.add(128)?;
        self.add(a.name().len())
    }
    pub fn document(&mut self, d: &ElementDocument) -> Result<(), CaptureError> {
        self.add(128)?;
        self.count(d.layers().roots().len(), 16)?;
        for n in d.layers().iter() {
            self.add(256)?;
            self.count(n.children().len(), 16)?;
        }
        Ok(())
    }
    pub fn components(&mut self, tree: &ComponentTree) -> Result<(), CaptureError> {
        self.count(tree.roots().len(), 16)?;
        for n in tree.iter() {
            self.add(256)?;
            self.count(n.children().len(), 16)?;
        }
        Ok(())
    }
    pub fn sound(&mut self, a: &AudioComponent) -> Result<(), CaptureError> {
        self.add(128)?;
        self.add(a.slot().as_str().len())
    }
    pub fn configuration(&mut self, c: &SkinConfiguration) -> Result<(), CaptureError> {
        for r in c.records() {
            self.add(64)?;
            self.add(r.raw().len())?;
        }
        Ok(())
    }
    pub fn outputs(&mut self, outputs: &[OutputMapping]) -> Result<(), CaptureError> {
        for o in outputs {
            self.add(128)?;
            self.add(o.path().as_str().len())?;
            self.add(o.element_key().len())?;
        }
        Ok(())
    }
}
pub(crate) fn project(p: &SkinProject) -> Result<usize, CaptureError> {
    let mut c = Cost::default();
    c.add(256)?;
    for a in p.assets().iter() {
        c.asset(a)?;
    }
    for d in p.documents() {
        c.document(d)?;
    }
    for s in p.audio_components() {
        c.sound(s)?;
    }
    c.components(p.components())?;
    c.configuration(p.configuration())?;
    c.outputs(p.outputs())?;
    Ok(c.bytes())
}
pub(crate) fn definition(p: &ProjectDefinition) -> Result<usize, CaptureError> {
    let mut c = Cost::default();
    c.add(256)?;
    for a in &p.assets {
        c.asset(a)?;
    }
    for d in &p.documents {
        c.document(d)?;
    }
    for s in &p.audio_components {
        c.sound(s)?;
    }
    c.components(&p.components)?;
    c.configuration(&p.configuration)?;
    c.outputs(&p.outputs)?;
    Ok(c.bytes())
}
