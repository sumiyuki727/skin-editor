//! Bounded control-side classification; no project, device or playback owners.
use crate::{Origin, SoundPlan};

pub(crate) fn same_content(a: SoundPlan, b: SoundPlan) -> bool {
    a.target == b.target
        && a.source == b.source
        && a.metadata == b.metadata
        && a.visible == b.visible
}
#[derive(Clone, Copy, Default)]
pub struct AuditionChange {
    pub gain_edit: bool,
    pub preserve_cancel: bool,
    pub replacement: bool,
}
#[derive(Clone, Default)]
pub struct AuditionEdits {
    last: Option<Origin>,
    plans: Vec<SoundPlan>,
    gain_draft: Option<u64>,
}
impl AuditionEdits {
    /// Call only for a validated/admitted projection. Caller capacity is MAX_BINDINGS.
    pub fn select(
        &mut self,
        origin: Origin,
        plans: &[SoundPlan],
        cancelled: &[u64],
    ) -> AuditionChange {
        let compatible_origin = self.last.is_some_and(|old| {
            old.session == origin.session && old.navigation_revision == origin.navigation_revision
        });
        let same_layout = self.plans.len() == plans.len()
            && self.plans.iter().all(|a| {
                plans
                    .iter()
                    .find(|b| a.target == b.target)
                    .is_some_and(|b| same_content(*a, *b))
            });
        let gain_changed = same_layout
            && self.plans.iter().any(|a| {
                plans
                    .iter()
                    .find(|b| a.target == b.target)
                    .is_some_and(|b| a.gain != b.gain)
            });
        let previous_gain_draft = self.gain_draft;
        let finishing = previous_gain_draft.is_some() && origin.draft.is_none();
        let keep =
            compatible_origin && same_layout && (gain_changed || previous_gain_draft.is_some());
        let preserve_cancel = keep
            && finishing
            && previous_gain_draft.is_some_and(|id| {
                cancelled.contains(&id)
                    || self.last.is_some_and(|old| {
                        old.draft.is_some_and(|d| d.0 == id)
                            && old.node == origin.node
                            && old.revision == origin.revision
                    })
            });
        let replacement = compatible_origin
            && self.plans.iter().any(|a| {
                plans
                    .iter()
                    .find(|b| a.target == b.target)
                    .is_none_or(|b| !same_content(*a, *b))
            });
        self.gain_draft = if keep {
            origin.draft.map(|d| d.0)
        } else {
            None
        };
        self.last = Some(origin);
        self.plans.clear();
        self.plans.extend_from_slice(plans);
        AuditionChange {
            gain_edit: keep,
            preserve_cancel,
            replacement,
        }
    }
}
