use crate::{CapturedOperation, Command, OperationOrigin};

impl CapturedOperation {
    pub(crate) fn transaction(commands: &[Command], origin: OperationOrigin) -> Self {
        let steps = commands
            .iter()
            .map(|c| match c {
                Command::RegisterAsset { .. } => "asset.register",
                Command::RemoveAsset { .. } => "asset.remove",
                Command::RenameAsset { .. } => "asset.rename",
                Command::CreateDocument { .. } => "document.create",
                Command::RemoveDocument { .. } => "document.remove",
                Command::ResizeDocument { .. } => "document.resize",
                Command::SetDocumentVisibility { .. } => "document.visibility",
                Command::AddLayer { .. } => "layer.add",
                Command::RemoveLayer { .. } => "layer.remove",
                Command::MoveLayer { .. } => "layer.move",
                Command::SetLayerTransform { .. } => "layer.transform",
                Command::SetLayerProperties { .. } => "layer.properties",
                Command::SetLayerAsset { .. } => "layer.asset",
                Command::AddComponent { .. } => "component.add",
                Command::RemoveComponent { .. } => "component.remove",
                Command::MoveComponent { .. } => "component.move",
                Command::SetComponentTransform { .. } => "component.transform",
                Command::SetComponentMapping { .. } => "component.mapping",
                Command::SetOutputMapping { .. } => "output.set",
                Command::RemoveOutputMapping { .. } => "output.remove",
                Command::InsertConfigRecord { .. } => "config.insert",
                Command::SetConfigRecord { .. } => "config.set",
                Command::RemoveConfigRecord { .. } => "config.remove",
                Command::CreateAudioComponent { .. } => "sound.create",
                Command::RemoveAudioComponent { .. } => "sound.remove",
                Command::SetAudioAsset { .. } => "sound.asset",
                Command::SetAudioProperties { .. } => "sound.properties",
            })
            .collect();
        Self {
            version: 1,
            origin,
            steps,
        }
    }
}
