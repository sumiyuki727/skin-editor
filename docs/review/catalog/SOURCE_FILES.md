# 全部源码与辅助代码索引

Rust 表是屏蔽注释／字符串后的词法定义位置；不保证泛型、宏展开或 cfg 的语义解析。Python 表来自 AST，基类和调用拼写并非动态派发证明。C／WGSL 仅记录有限词法声明，需结合原文件审阅。
P0、测试及诊断不因此成为产品模块；本表不执行代码或扫描用户目录。

## crates/application/src/address.rs

68 行；SHA256 `d6d1ba60051876c653bbb829caa243b1e166514f3256de37e9b227ccfcf07c0b`

导入：`use crate::{AddressError, BranchCoordinate, NodeId};`

| 定义 | 位置／签名 |
| --- | --- |
| const MAX_NODE_ADDRESS | [源码:3](../../../crates/application/src/address.rs#L3) |
| const MAX_BRANCH_ADDRESS | [源码:4](../../../crates/application/src/address.rs#L4) |
| fn encode | [源码:6](../../../crates/application/src/address.rs#L6) |
| fn number | [源码:26](../../../crates/application/src/address.rs#L26) |
| impl impl NodeId | [源码:39](../../../crates/application/src/address.rs#L39) |
| fn short_address | [源码:40](../../../crates/application/src/address.rs#L40) |
| fn parse_address | [源码:46](../../../crates/application/src/address.rs#L46) |
| impl impl BranchCoordinate | [源码:55](../../../crates/application/src/address.rs#L55) |
| fn short_address | [源码:56](../../../crates/application/src/address.rs#L56) |
| fn parse_address | [源码:60](../../../crates/application/src/address.rs#L60) |

## crates/application/src/audio_editor.rs

171 行；SHA256 `87247d6c14bfa4972c64ecdc87e1a2ec483bec65bda94cc0adc0ee4871524d03`

导入：`use crate::{ProjectSession, SkinAudioError, SkinAudioPort, SkinAudioSource};`, `use skin_editor_domain::{AssetId, AssetKind, AudioComponentId, AudioMetadata, BlobRef};`

| 定义 | 位置／签名 |
| --- | --- |
| const AUDIO_EDITOR_API_VERSION | [源码:4](../../../crates/application/src/audio_editor.rs#L4) |
| enum AudioEditorAction | [源码:6](../../../crates/application/src/audio_editor.rs#L6) |
| enum AudioEditorError | [源码:17](../../../crates/application/src/audio_editor.rs#L17) |
| enum AudioPreparationState | [源码:28](../../../crates/application/src/audio_editor.rs#L28) |
| struct AudioComponentState | [源码:34](../../../crates/application/src/audio_editor.rs#L34) |
| struct AssetAudition | [源码:41](../../../crates/application/src/audio_editor.rs#L41) |
| struct AssetAuditionState | [源码:48](../../../crates/application/src/audio_editor.rs#L48) |
| struct AudioOutputObservation | [源码:54](../../../crates/application/src/audio_editor.rs#L54) |
| struct AudioEditorState | [源码:66](../../../crates/application/src/audio_editor.rs#L66) |
| trait AudioEditorPort | [源码:83](../../../crates/application/src/audio_editor.rs#L83) |
| fn audition_asset | [源码:84](../../../crates/application/src/audio_editor.rs#L84) |
| fn request | [源码:87](../../../crates/application/src/audio_editor.rs#L87) |
| fn poll | [源码:88](../../../crates/application/src/audio_editor.rs#L88) |
| struct DisabledAudio | [源码:91](../../../crates/application/src/audio_editor.rs#L91) |
| impl impl SkinAudioPort for DisabledAudio | [源码:92](../../../crates/application/src/audio_editor.rs#L92) |
| fn synchronize | [源码:93](../../../crates/application/src/audio_editor.rs#L93) |
| impl impl AudioEditorPort for DisabledAudio | [源码:102](../../../crates/application/src/audio_editor.rs#L102) |
| fn request | [源码:103](../../../crates/application/src/audio_editor.rs#L103) |
| fn poll | [源码:106](../../../crates/application/src/audio_editor.rs#L106) |
| impl impl ProjectSession | [源码:110](../../../crates/application/src/audio_editor.rs#L110) |
| fn skin_audio_source | [源码:112](../../../crates/application/src/audio_editor.rs#L112) |
| fn control_audio | [源码:120](../../../crates/application/src/audio_editor.rs#L120) |

## crates/application/src/audio_port.rs

65 行；SHA256 `4ce375c8689d82f51a715ca8d0d403e134521aefddc796f0c469e57e5d18d6bc`

导入：`use crate::{DraftId, DraftState, ExpectedState, HistorySessionId, ProjectSession};`, `use skin_editor_domain::SkinProject;`

| 定义 | 位置／签名 |
| --- | --- |
| const SKIN_AUDIO_API_VERSION | [源码:5](../../../crates/application/src/audio_port.rs#L5) |
| const MAX_AUDIO_DRAFT_REFERENCES | [源码:6](../../../crates/application/src/audio_port.rs#L6) |
| struct SkinAudioSource | [源码:9](../../../crates/application/src/audio_port.rs#L9) |
| enum SkinAudioError | [源码:17](../../../crates/application/src/audio_port.rs#L17) |
| trait SkinAudioPort | [源码:22](../../../crates/application/src/audio_port.rs#L22) |
| fn retained_drafts | [源码:24](../../../crates/application/src/audio_port.rs#L24) |
| fn synchronize | [源码:28](../../../crates/application/src/audio_port.rs#L28) |
| impl impl ProjectSession | [源码:36](../../../crates/application/src/audio_port.rs#L36) |
| fn synchronize_audio | [源码:41](../../../crates/application/src/audio_port.rs#L41) |

## crates/application/src/contract.rs

428 行；SHA256 `22153f8959866c4a3e9df2c2dc37648113ba849eb11b9b53e46cb30f0b07d099`

导入：`use core::num::{NonZeroU64, NonZeroU128};`, `use skin_editor_domain::{ Affine2, Asset, AssetId, AudioComponent, AudioComponentId, ComponentId, ComponentNode, ConfigRecord, DocumentId, ElementDocument, Gain, LayerId, LayerNode, ModelError, ObjectId, Opacity, OutputId, OutputMapping, PixelSize, };`

| 定义 | 位置／签名 |
| --- | --- |
| struct HistorySessionId | [源码:12](../../../crates/application/src/contract.rs#L12) |
| impl impl HistorySessionId | [源码:13](../../../crates/application/src/contract.rs#L13) |
| fn new | [源码:14](../../../crates/application/src/contract.rs#L14) |
| fn get | [源码:20](../../../crates/application/src/contract.rs#L20) |
| struct RequestId | [源码:26](../../../crates/application/src/contract.rs#L26) |
| impl impl RequestId | [源码:27](../../../crates/application/src/contract.rs#L27) |
| fn new | [源码:28](../../../crates/application/src/contract.rs#L28) |
| fn get | [源码:34](../../../crates/application/src/contract.rs#L34) |
| struct NodeId | [源码:41](../../../crates/application/src/contract.rs#L41) |
| impl impl NodeId | [源码:42](../../../crates/application/src/contract.rs#L42) |
| fn new | [源码:43](../../../crates/application/src/contract.rs#L43) |
| fn get | [源码:49](../../../crates/application/src/contract.rs#L49) |
| struct ExpectedState | [源码:55](../../../crates/application/src/contract.rs#L55) |
| enum Command | [源码:64](../../../crates/application/src/contract.rs#L64) |
| struct CommandRequest | [源码:177](../../../crates/application/src/contract.rs#L177) |
| enum CommandStatus | [源码:186](../../../crates/application/src/contract.rs#L186) |
| struct CommandResult | [源码:192](../../../crates/application/src/contract.rs#L192) |
| enum CommandFailure | [源码:204](../../../crates/application/src/contract.rs#L204) |
| static str | [源码:211](../../../crates/application/src/contract.rs#L211) |
| impl impl From<ModelError> for CommandFailure | [源码:227](../../../crates/application/src/contract.rs#L227) |
| fn from | [源码:228](../../../crates/application/src/contract.rs#L228) |
| enum CommandError | [源码:234](../../../crates/application/src/contract.rs#L234) |
| static str | [源码:250](../../../crates/application/src/contract.rs#L250) |
| struct ValidationResult | [源码:256](../../../crates/application/src/contract.rs#L256) |
| enum HistoryQueryError | [源码:266](../../../crates/application/src/contract.rs#L266) |
| struct HistoryEntry | [源码:277](../../../crates/application/src/contract.rs#L277) |
| struct BranchCoordinate | [源码:291](../../../crates/application/src/contract.rs#L291) |
| impl impl BranchCoordinate | [源码:295](../../../crates/application/src/contract.rs#L295) |
| fn new | [源码:296](../../../crates/application/src/contract.rs#L296) |
| fn branch | [源码:302](../../../crates/application/src/contract.rs#L302) |
| fn ordinal | [源码:305](../../../crates/application/src/contract.rs#L305) |
| enum AddressError | [源码:311](../../../crates/application/src/contract.rs#L311) |
| enum HistoryAction | [源码:319](../../../crates/application/src/contract.rs#L319) |
| struct HistoryRequest | [源码:332](../../../crates/application/src/contract.rs#L332) |
| enum HistoryStatus | [源码:341](../../../crates/application/src/contract.rs#L341) |
| struct HistoryResult | [源码:347](../../../crates/application/src/contract.rs#L347) |
| enum HistoryError | [源码:359](../../../crates/application/src/contract.rs#L359) |
| static str | [源码:376](../../../crates/application/src/contract.rs#L376) |
| impl impl From<HistoryQueryError> for HistoryError | [源码:379](../../../crates/application/src/contract.rs#L379) |
| fn from | [源码:380](../../../crates/application/src/contract.rs#L380) |
| const MAX_HISTORY_PAGE | [源码:385](../../../crates/application/src/contract.rs#L385) |
| struct HistorySummary | [源码:388](../../../crates/application/src/contract.rs#L388) |
| struct HistoryChildrenPage | [源码:397](../../../crates/application/src/contract.rs#L397) |
| struct HistoryAncestorsPage | [源码:406](../../../crates/application/src/contract.rs#L406) |
| struct HistoryStatistics | [源码:416](../../../crates/application/src/contract.rs#L416) |

## crates/application/src/draft.rs

347 行；SHA256 `06a2c4c2cad35c54201ae681aba9559d41c2d8e8482581d0442f3062c59e8b7b`

导入：`use crate::{ API_VERSION, BlobLease, Command, CommandError, DraftAction, DraftError, DraftId, DraftInfo, DraftOutcome, DraftRequest, DraftResult, DraftSnapshot, DraftState, HistorySessionId, ProjectSession, Receipt, };`, `use skin_editor_domain::{BlobRef, ObjectId, SkinProject};`, `use std::collections::{BTreeMap, BTreeSet};`, `use std::sync::Arc;`, `use super::*;`, `use crate::RequestId;`

| 定义 | 位置／签名 |
| --- | --- |
| struct Draft | [源码:10](../../../crates/application/src/draft.rs#L10) |
| impl impl ProjectSession | [源码:18](../../../crates/application/src/draft.rs#L18) |
| fn draft_info | [源码:19](../../../crates/application/src/draft.rs#L19) |
| fn expected_draft | [源码:34](../../../crates/application/src/draft.rs#L34) |
| fn draft_snapshot | [源码:46](../../../crates/application/src/draft.rs#L46) |
| fn edit_draft | [源码:63](../../../crates/application/src/draft.rs#L63) |
| fn draft_once | [源码:121](../../../crates/application/src/draft.rs#L121) |
| mod tests | [源码:206](../../../crates/application/src/draft.rs#L206) |
| fn apply | [源码:209](../../../crates/application/src/draft.rs#L209) |
| fn begin | [源码:221](../../../crates/application/src/draft.rs#L221) |
| fn command | [源码:232](../../../crates/application/src/draft.rs#L232) |
| fn draft_id_and_revision_exhaustion_never_change_committed_state | [源码:243](../../../crates/application/src/draft.rs#L243) |
| fn unexpected_base_change_rejects_commit_and_still_allows_cancel | [源码:288](../../../crates/application/src/draft.rs#L288) |
| fn failed_publication_keeps_draft_for_cancel_or_new_request_retry | [源码:313](../../../crates/application/src/draft.rs#L313) |

## crates/application/src/draft_contract.rs

96 行；SHA256 `9ca0e3fba10f8b8545511ed45fa649523e5fc07d6f538ce9ccc85f5175538bd7`

导入：`use crate::{Command, CommandError, CommandResult, ExpectedState, HistorySessionId, RequestId};`, `use core::num::NonZeroU64;`, `use skin_editor_domain::SkinProject;`

| 定义 | 位置／签名 |
| --- | --- |
| struct DraftId | [源码:7](../../../crates/application/src/draft_contract.rs#L7) |
| impl impl DraftId | [源码:8](../../../crates/application/src/draft_contract.rs#L8) |
| fn new | [源码:9](../../../crates/application/src/draft_contract.rs#L9) |
| fn get | [源码:15](../../../crates/application/src/draft_contract.rs#L15) |
| struct DraftState | [源码:21](../../../crates/application/src/draft_contract.rs#L21) |
| enum DraftAction | [源码:28](../../../crates/application/src/draft_contract.rs#L28) |
| struct DraftRequest | [源码:47](../../../crates/application/src/draft_contract.rs#L47) |
| struct DraftInfo | [源码:55](../../../crates/application/src/draft_contract.rs#L55) |
| struct DraftSnapshot | [源码:63](../../../crates/application/src/draft_contract.rs#L63) |
| enum DraftOutcome | [源码:69](../../../crates/application/src/draft_contract.rs#L69) |
| struct DraftResult | [源码:76](../../../crates/application/src/draft_contract.rs#L76) |
| enum DraftError | [源码:84](../../../crates/application/src/draft_contract.rs#L84) |
| impl impl From<CommandError> for DraftError | [源码:92](../../../crates/application/src/draft_contract.rs#L92) |
| fn from | [源码:93](../../../crates/application/src/draft_contract.rs#L93) |

## crates/application/src/execute.rs

489 行；SHA256 `735cbc5a56ac1256123ad9e69d74fae321b79a398fa43b6730de63b4a4f029dc`

导入：`use std::collections::{BTreeMap, BTreeSet};`, `use skin_editor_domain::{ Affine2, Asset, AudioComponent, ComponentId, ComponentKind, ComponentNode, ComponentTree, DocumentId, ElementDocument, LayerId, LayerKind, LayerNode, LayerTree, ObjectId, OutputSource, ProjectDefinition, SkinConfiguration, SkinProject, };`, `use crate::history::definition;`, `use crate::{Command, CommandFailure};`

| 定义 | 位置／签名 |
| --- | --- |
| fn reserve | [源码:12](../../../crates/application/src/execute.rs#L12) |
| fn position | [源码:18](../../../crates/application/src/execute.rs#L18) |
| fn doc_index | [源码:24](../../../crates/application/src/execute.rs#L24) |
| type LayerNodes | [源码:32](../../../crates/application/src/execute.rs#L32) |
| fn layer_children | [源码:33](../../../crates/application/src/execute.rs#L33) |
| fn layer_node | [源码:50](../../../crates/application/src/execute.rs#L50) |
| fn layers | [源码:55](../../../crates/application/src/execute.rs#L55) |
| impl impl FnOnce(&LayerTree, &mut Vec<LayerId>, &mut LayerNodes) -> Result<(), CommandFailure>, ) -> Result<(), CommandFailure> | [源码:58](../../../crates/application/src/execute.rs#L58) |
| type ComponentNodes | [源码:71](../../../crates/application/src/execute.rs#L71) |
| fn component_children | [源码:72](../../../crates/application/src/execute.rs#L72) |
| fn component_node | [源码:89](../../../crates/application/src/execute.rs#L89) |
| fn components | [源码:97](../../../crates/application/src/execute.rs#L97) |
| impl impl FnOnce( &ComponentTree, &mut Vec<ComponentId>, &mut ComponentNodes, ) -> Result<(), CommandFailure>, ) -> Result<(), CommandFailure> | [源码:99](../../../crates/application/src/execute.rs#L99) |
| fn apply | [源码:114](../../../crates/application/src/execute.rs#L114) |

## crates/application/src/history.rs

560 行；SHA256 `44c9c53b29e6e63b30ddf6ba6dcde360f3d2885d21b40178c0a89de81e3a49cf`

导入：`use std::collections::BTreeMap;`, `use skin_editor_domain::{ Asset, AssetId, AudioComponent, AudioComponentId, ComponentTree, DocumentId, ElementDocument, ModelError, ObjectId, OutputMapping, ProjectDefinition, SkinConfiguration, SkinProject, };`, `use crate::{ API_VERSION, BranchCoordinate, CommandError, HistoryEntry, HistoryQueryError, HistorySessionId, HistorySummary, NodeId, RequestId, };`, `use super::*;`, `use skin_editor_domain::{ Affine2, AssetKind, BlobRef, ComponentId, ComponentKind, ComponentNode, ConfigInterpretation, ConfigRecord, LayerId, LayerKind, LayerNode, LayerTree, Opacity, OutputId, OutputPath, PixelSize, ProjectId, };`

| 定义 | 位置／签名 |
| --- | --- |
| fn definition | [源码:13](../../../crates/application/src/history.rs#L13) |
| struct Change | [源码:27](../../../crates/application/src/history.rs#L27) |
| fn changes | [源码:33](../../../crates/application/src/history.rs#L33) |
| struct Patch | [源码:56](../../../crates/application/src/history.rs#L56) |
| impl impl Patch | [源码:66](../../../crates/application/src/history.rs#L66) |
| fn between | [源码:67](../../../crates/application/src/history.rs#L67) |
| fn configuration_changed | [源码:135](../../../crates/application/src/history.rs#L135) |
| fn apply | [源码:140](../../../crates/application/src/history.rs#L140) |
| fn apply_definition | [源码:146](../../../crates/application/src/history.rs#L146) |
| struct Node | [源码:195](../../../crates/application/src/history.rs#L195) |
| struct History | [源码:203](../../../crates/application/src/history.rs#L203) |
| struct AppendPlan | [源码:213](../../../crates/application/src/history.rs#L213) |
| impl impl History | [源码:221](../../../crates/application/src/history.rs#L221) |
| fn new | [源码:222](../../../crates/application/src/history.rs#L222) |
| fn plan_append | [源码:248](../../../crates/application/src/history.rs#L248) |
| fn append | [源码:285](../../../crates/application/src/history.rs#L285) |
| fn summary | [源码:311](../../../crates/application/src/history.rs#L311) |
| fn node | [源码:322](../../../crates/application/src/history.rs#L322) |
| fn parent | [源码:328](../../../crates/application/src/history.rs#L328) |
| fn entry | [源码:335](../../../crates/application/src/history.rs#L335) |
| fn materialize | [源码:353](../../../crates/application/src/history.rs#L353) |
| fn transition | [源码:359](../../../crates/application/src/history.rs#L359) |
| mod tests | [源码:408](../../../crates/application/src/history.rs#L408) |
| fn invalid_restored_project_is_rejected_even_when_all_node_patches_exist | [源码:417](../../../crates/application/src/history.rs#L417) |
| fn depth_ordinal_and_branch_exhaustion_reject_before_appending | [源码:454](../../../crates/application/src/history.rs#L454) |
| fn patches_round_trip_assets_documents_components_outputs_and_configuration_together | [源码:496](../../../crates/application/src/history.rs#L496) |

## crates/application/src/history_control.rs

361 行；SHA256 `c63f63027d347bd747730eacc2c02c7037df1105cf4dfb7deabf49ce1e3b8d44`

导入：`use crate::{ API_VERSION, ExpectedState, HistoryAction, HistoryError, HistoryRequest, HistoryResult, HistoryStatus, NodeId, ProjectSession, Receipt, history::Patch, };`, `use super::*;`, `use crate::{Command, CommandRequest, HistoryQueryError, RequestId, SessionLimits, values::*};`, `use std::sync::Arc;`

| 定义 | 位置／签名 |
| --- | --- |
| impl impl ProjectSession | [源码:6](../../../crates/application/src/history_control.rs#L6) |
| fn navigate | [源码:9](../../../crates/application/src/history_control.rs#L9) |
| fn navigation_target | [源码:42](../../../crates/application/src/history_control.rs#L42) |
| fn navigate_once | [源码:79](../../../crates/application/src/history_control.rs#L79) |
| mod tests | [源码:143](../../../crates/application/src/history_control.rs#L143) |
| fn failed_restore_preserves_fork_preference_and_old_api_is_rejected | [源码:148](../../../crates/application/src/history_control.rs#L148) |
| fn edit | [源码:196](../../../crates/application/src/history_control.rs#L196) |
| fn request | [源码:213](../../../crates/application/src/history_control.rs#L213) |
| fn revision_exhaustion_preserves_active_head_but_allows_no_change | [源码:224](../../../crates/application/src/history_control.rs#L224) |
| fn failed_replay_discards_intermediate_candidate_and_caches_failure | [源码:251](../../../crates/application/src/history_control.rs#L251) |
| fn commands_and_navigation_share_receipt_capacity_without_evicting_replays | [源码:278](../../../crates/application/src/history_control.rs#L278) |
| fn missing_internal_resource_pin_prevents_restoring_head | [源码:304](../../../crates/application/src/history_control.rs#L304) |
| struct Lease | [源码:306](../../../crates/application/src/history_control.rs#L306) |
| impl impl crate::BlobLease for Lease | [源码:307](../../../crates/application/src/history_control.rs#L307) |
| fn reference | [源码:308](../../../crates/application/src/history_control.rs#L308) |
| struct Store | [源码:312](../../../crates/application/src/history_control.rs#L312) |
| impl impl crate::ResourceStore for Store | [源码:313](../../../crates/application/src/history_control.rs#L313) |
| fn pin | [源码:314](../../../crates/application/src/history_control.rs#L314) |

## crates/application/src/history_queries.rs

131 行；SHA256 `461058962269de64d73f4002065f75b6dbdfdc970abc4ad9d529078445ba0741`

导入：`use crate::{ API_VERSION, AddressError, BranchCoordinate, HistoryAncestorsPage, HistoryChildrenPage, HistoryQueryError, HistorySessionId, HistoryStatistics, HistorySummary, MAX_BRANCH_ADDRESS, MAX_HISTORY_PAGE, NodeId, ProjectSession, };`

| 定义 | 位置／签名 |
| --- | --- |
| fn page_limit | [源码:7](../../../crates/application/src/history_queries.rs#L7) |
| impl impl ProjectSession | [源码:14](../../../crates/application/src/history_queries.rs#L14) |
| fn history_scope | [源码:15](../../../crates/application/src/history_queries.rs#L15) |
| fn history_summary | [源码:22](../../../crates/application/src/history_queries.rs#L22) |
| fn resolve_history_address | [源码:31](../../../crates/application/src/history_queries.rs#L31) |
| fn history_children | [源码:56](../../../crates/application/src/history_queries.rs#L56) |
| fn history_ancestors | [源码:87](../../../crates/application/src/history_queries.rs#L87) |
| fn history_statistics | [源码:113](../../../crates/application/src/history_queries.rs#L113) |

## crates/application/src/lib.rs

651 行；SHA256 `9ee6850d37829100ec5d9fd304939f5db5f7d918eddee7b94290fdcd08082a03`

导入：`use audio_editor::*;`, `use audio_port::*;`, `use preview_port::*;`, `use std::collections::{BTreeMap, BTreeSet};`, `use std::sync::Arc;`, `use skin_editor_domain::{BlobRef, ObjectId};`, `use skin_editor_domain::{ProjectId, SkinProject};`, `use address::{MAX_BRANCH_ADDRESS, MAX_NODE_ADDRESS};`, `use contract::{ AddressError, BranchCoordinate, Command, CommandError, CommandFailure, CommandRequest, CommandResult, CommandStatus, ExpectedState, HistoryAction, HistoryAncestorsPage, HistoryChildrenPage, HistoryEntry, HistoryError, HistoryQueryError, HistoryRequest, HistoryResult, HistorySessionId, HistoryStatistics, HistoryStatus, HistorySummary, MAX_HISTORY_PAGE, NodeId, RequestId, ValidationResult, };`, `use draft_contract::{ DraftAction, DraftError, DraftId, DraftInfo, DraftOutcome, DraftRequest, DraftResult, DraftSnapshot, DraftState, };`, `use resources::{BlobLease, ResourceError, ResourceStore};`, `use skin_editor_domain::{ Affine2, Asset, AssetId, AssetKind, AudioComponent, AudioComponentId, AudioMetadata, BlobRef, Bounds, ComponentId, ComponentKind, ComponentNode, ConfigInterpretation, ConfigRecord, DocumentId, ElementDocument, Gain, LayerId, LayerKind, LayerNode, LayerTree, ModelError, ObjectId, Opacity, OutputId, OutputMapping, OutputPath, OutputSource, PixelSize, SkinProject, SoundSlotKey, };`, `use super::{API_VERSION, ProjectSession, SessionError};`, `use skin_editor_domain::{ ConfigInterpretation, ConfigRecord, DocumentId, ElementDocument, LayerTree, PixelSize, ProjectDefinition, ProjectId, SkinConfiguration, SkinProject, };`

| 定义 | 位置／签名 |
| --- | --- |
| mod address | [源码:4](../../../crates/application/src/lib.rs#L4) |
| mod audio_editor | [源码:5](../../../crates/application/src/lib.rs#L5) |
| mod audio_port | [源码:6](../../../crates/application/src/lib.rs#L6) |
| mod contract | [源码:7](../../../crates/application/src/lib.rs#L7) |
| mod draft | [源码:8](../../../crates/application/src/lib.rs#L8) |
| mod draft_contract | [源码:9](../../../crates/application/src/lib.rs#L9) |
| mod execute | [源码:10](../../../crates/application/src/lib.rs#L10) |
| mod history | [源码:11](../../../crates/application/src/lib.rs#L11) |
| mod history_control | [源码:12](../../../crates/application/src/lib.rs#L12) |
| mod history_queries | [源码:13](../../../crates/application/src/lib.rs#L13) |
| mod preview_port | [源码:14](../../../crates/application/src/lib.rs#L14) |
| mod resources | [源码:15](../../../crates/application/src/lib.rs#L15) |
| mod values | [源码:42](../../../crates/application/src/lib.rs#L42) |
| const API_VERSION | [源码:53](../../../crates/application/src/lib.rs#L53) |
| enum SessionError | [源码:57](../../../crates/application/src/lib.rs#L57) |
| struct ProjectInfo | [源码:66](../../../crates/application/src/lib.rs#L66) |
| struct ProjectSnapshot | [源码:77](../../../crates/application/src/lib.rs#L77) |
| struct HistoricalSnapshot | [源码:85](../../../crates/application/src/lib.rs#L85) |
| struct SessionLimits | [源码:95](../../../crates/application/src/lib.rs#L95) |
| impl impl Default for SessionLimits | [源码:100](../../../crates/application/src/lib.rs#L100) |
| fn default | [源码:101](../../../crates/application/src/lib.rs#L101) |
| enum Receipt | [源码:110](../../../crates/application/src/lib.rs#L110) |
| struct ProjectSession | [源码:137](../../../crates/application/src/lib.rs#L137) |
| impl impl ProjectSession | [源码:158](../../../crates/application/src/lib.rs#L158) |
| fn new | [源码:160](../../../crates/application/src/lib.rs#L160) |
| fn with_resources | [源码:174](../../../crates/application/src/lib.rs#L174) |
| fn project_info | [源码:212](../../../crates/application/src/lib.rs#L212) |
| fn snapshot | [源码:221](../../../crates/application/src/lib.rs#L221) |
| fn history_len | [源码:230](../../../crates/application/src/lib.rs#L230) |
| fn history_entry | [源码:234](../../../crates/application/src/lib.rs#L234) |
| fn history_snapshot | [源码:248](../../../crates/application/src/lib.rs#L248) |
| fn envelope | [源码:269](../../../crates/application/src/lib.rs#L269) |
| fn cached | [源码:282](../../../crates/application/src/lib.rs#L282) |
| fn prepare | [源码:296](../../../crates/application/src/lib.rs#L296) |
| fn validate | [源码:331](../../../crates/application/src/lib.rs#L331) |
| fn submit | [源码:379](../../../crates/application/src/lib.rs#L379) |
| fn commit | [源码:402](../../../crates/application/src/lib.rs#L402) |
| fn pin_candidate | [源码:407](../../../crates/application/src/lib.rs#L407) |
| fn publish | [源码:446](../../../crates/application/src/lib.rs#L446) |
| mod tests | [源码:496](../../../crates/application/src/lib.rs#L496) |
| fn rejects_an_invalid_session_identity | [源码:500](../../../crates/application/src/lib.rs#L500) |
| fn queries_keep_the_version_and_identity | [源码:508](../../../crates/application/src/lib.rs#L508) |
| fn modifying_a_query_value_cannot_modify_the_active_session | [源码:518](../../../crates/application/src/lib.rs#L518) |
| fn new_session_owns_a_valid_empty_project | [源码:529](../../../crates/application/src/lib.rs#L529) |
| fn snapshot_values_are_detached_even_for_populated_project_content | [源码:539](../../../crates/application/src/lib.rs#L539) |
| fn session_scope_and_safeguards_are_validated_at_creation | [源码:577](../../../crates/application/src/lib.rs#L577) |
| fn config_request | [源码:595](../../../crates/application/src/lib.rs#L595) |
| fn reversible_patch_restores_exact_values_without_reexecuting_commands | [源码:613](../../../crates/application/src/lib.rs#L613) |
| fn node_and_revision_exhaustion_reject_before_publishing | [源码:627](../../../crates/application/src/lib.rs#L627) |

## crates/application/src/preview_port.rs

226 行；SHA256 `9dec4f42efc8201c4dc6bbbb784548e4beba30cac7fe8de305473bbefabd21aa`

导入：`use crate::{BlobLease, DraftState, ExpectedState, HistorySessionId, ProjectSession};`, `use skin_editor_domain::{BlobRef, DocumentId, PixelSize, SkinProject};`, `use std::sync::Arc;`, `use super::*;`, `use crate::{API_VERSION, DraftAction, DraftRequest, RequestId};`

| 定义 | 位置／签名 |
| --- | --- |
| const PREVIEW_API_VERSION | [源码:6](../../../crates/application/src/preview_port.rs#L6) |
| enum PreviewTarget | [源码:8](../../../crates/application/src/preview_port.rs#L8) |
| struct PreviewContext | [源码:15](../../../crates/application/src/preview_port.rs#L15) |
| struct FrameSource | [源码:24](../../../crates/application/src/preview_port.rs#L24) |
| struct FrameKey | [源码:30](../../../crates/application/src/preview_port.rs#L30) |
| enum FrameStatus | [源码:32](../../../crates/application/src/preview_port.rs#L32) |
| struct FrameDescriptor | [源码:37](../../../crates/application/src/preview_port.rs#L37) |
| enum PreviewError | [源码:44](../../../crates/application/src/preview_port.rs#L44) |
| trait PreviewPort | [源码:51](../../../crates/application/src/preview_port.rs#L51) |
| fn request | [源码:52](../../../crates/application/src/preview_port.rs#L52) |
| impl impl ProjectSession | [源码:58](../../../crates/application/src/preview_port.rs#L58) |
| fn request_preview | [源码:59](../../../crates/application/src/preview_port.rs#L59) |
| enum ImportError | [源码:92](../../../crates/application/src/preview_port.rs#L92) |
| struct StagedRaster | [源码:98](../../../crates/application/src/preview_port.rs#L98) |
| trait RasterImportPort | [源码:104](../../../crates/application/src/preview_port.rs#L104) |
| fn stage_png | [源码:106](../../../crates/application/src/preview_port.rs#L106) |
| mod tests | [源码:110](../../../crates/application/src/preview_port.rs#L110) |
| struct Port | [源码:112](../../../crates/application/src/preview_port.rs#L112) |
| fn workspace_projection_is_read_only_and_uses_version_two | [源码:114](../../../crates/application/src/preview_port.rs#L114) |
| impl impl PreviewPort for Port | [源码:136](../../../crates/application/src/preview_port.rs#L136) |
| fn request | [源码:137](../../../crates/application/src/preview_port.rs#L137) |
| fn preview_is_scoped_read_only_and_rejects_invalid_requests_before_adapter | [源码:152](../../../crates/application/src/preview_port.rs#L152) |
| fn cancelled_draft_token_cannot_select_a_project_for_preview | [源码:178](../../../crates/application/src/preview_port.rs#L178) |

## crates/application/src/resources.rs

37 行；SHA256 `b21d2808b96457ea6fd35cdfdc3e45c89c61e4d69fe92f64477105e33c117770`

导入：`use std::sync::Arc;`, `use skin_editor_domain::{AudioMetadata, BlobRef};`

| 定义 | 位置／签名 |
| --- | --- |
| enum ResourceError | [源码:6](../../../crates/application/src/resources.rs#L6) |
| trait BlobLease | [源码:16](../../../crates/application/src/resources.rs#L16) |
| fn reference | [源码:17](../../../crates/application/src/resources.rs#L17) |
| trait ResourceStore | [源码:20](../../../crates/application/src/resources.rs#L20) |
| fn audio_metadata | [源码:22](../../../crates/application/src/resources.rs#L22) |
| fn pin | [源码:29](../../../crates/application/src/resources.rs#L29) |
| struct NoResources | [源码:32](../../../crates/application/src/resources.rs#L32) |
| impl impl ResourceStore for NoResources | [源码:33](../../../crates/application/src/resources.rs#L33) |
| fn pin | [源码:34](../../../crates/application/src/resources.rs#L34) |

## crates/application/tests/audio.rs

445 行；SHA256 `7ba8b8d28f96a445b26634323ae4d1345dfe8384b3ceda0e83a583e31bc31143`

导入：`use skin_editor_application::{values::*, *};`, `use std::sync::Arc;`

| 定义 | 位置／签名 |
| --- | --- |
| fn aid | [源码:4](../../../crates/application/tests/audio.rs#L4) |
| fn sid | [源码:7](../../../crates/application/tests/audio.rs#L7) |
| fn did | [源码:10](../../../crates/application/tests/audio.rs#L10) |
| fn metadata | [源码:13](../../../crates/application/tests/audio.rs#L13) |
| fn blob | [源码:16](../../../crates/application/tests/audio.rs#L16) |
| struct Lease | [源码:19](../../../crates/application/tests/audio.rs#L19) |
| impl impl BlobLease for Lease | [源码:20](../../../crates/application/tests/audio.rs#L20) |
| fn reference | [源码:21](../../../crates/application/tests/audio.rs#L21) |
| struct Store | [源码:25](../../../crates/application/tests/audio.rs#L25) |
| impl impl ResourceStore for Store | [源码:26](../../../crates/application/tests/audio.rs#L26) |
| fn pin | [源码:27](../../../crates/application/tests/audio.rs#L27) |
| fn audio_metadata | [源码:34](../../../crates/application/tests/audio.rs#L34) |
| struct Driver | [源码:38](../../../crates/application/tests/audio.rs#L38) |
| impl impl Driver | [源码:42](../../../crates/application/tests/audio.rs#L42) |
| fn new | [源码:43](../../../crates/application/tests/audio.rs#L43) |
| fn id | [源码:55](../../../crates/application/tests/audio.rs#L55) |
| fn request | [源码:60](../../../crates/application/tests/audio.rs#L60) |
| fn commit | [源码:69](../../../crates/application/tests/audio.rs#L69) |
| fn navigate | [源码:73](../../../crates/application/tests/audio.rs#L73) |
| fn draft | [源码:83](../../../crates/application/tests/audio.rs#L83) |
| fn scope | [源码:92](../../../crates/application/tests/audio.rs#L92) |
| fn ds | [源码:95](../../../crates/application/tests/audio.rs#L95) |
| fn sound | [源码:102](../../../crates/application/tests/audio.rs#L102) |
| fn baseline | [源码:110](../../../crates/application/tests/audio.rs#L110) |
| fn register | [源码:122](../../../crates/application/tests/audio.rs#L122) |
| fn sound | [源码:127](../../../crates/application/tests/audio.rs#L127) |
| fn properties | [源码:134](../../../crates/application/tests/audio.rs#L134) |
| fn mapping | [源码:141](../../../crates/application/tests/audio.rs#L141) |
| fn sounds_are_independent_reusable_and_report_no_image_changes | [源码:152](../../../crates/application/tests/audio.rs#L152) |
| fn referenced_asset_and_output_owner_require_explicit_unlinking | [源码:169](../../../crates/application/tests/audio.rs#L169) |
| fn typed_metadata_is_checked_even_for_already_pinned_opaque_bytes | [源码:204](../../../crates/application/tests/audio.rs#L204) |
| fn failed_batch_noop_replay_and_conflict_keep_atomic_history | [源码:248](../../../crates/application/tests/audio.rs#L248) |
| fn branch_restore_retains_shared_and_replaced_raw_resources | [源码:286](../../../crates/application/tests/audio.rs#L286) |
| fn gain_draft_three_hundred_updates_commit_once_and_busy_is_preserved | [源码:320](../../../crates/application/tests/audio.rs#L320) |
| fn cancelled_failed_and_noop_audio_drafts_leave_no_project_or_node | [源码:350](../../../crates/application/tests/audio.rs#L350) |
| fn whole_image_visibility_survives_layer_and_canvas_edits_and_history | [源码:401](../../../crates/application/tests/audio.rs#L401) |

## crates/application/tests/audio_editor.rs

217 行；SHA256 `d273cf2c870e1db39b04e23e50607bdb1548d3f88a7251f0ea563ed3ffe285d3`

导入：`use skin_editor_application::{values::*, *};`, `use std::sync::Arc;`

| 定义 | 位置／签名 |
| --- | --- |
| struct Port | [源码:3](../../../crates/application/tests/audio_editor.rs#L3) |
| static str | [源码:4](../../../crates/application/tests/audio_editor.rs#L4) |
| impl impl SkinAudioPort for Port | [源码:9](../../../crates/application/tests/audio_editor.rs#L9) |
| fn synchronize | [源码:10](../../../crates/application/tests/audio_editor.rs#L10) |
| impl impl AudioEditorPort for Port | [源码:25](../../../crates/application/tests/audio_editor.rs#L25) |
| fn audition_asset | [源码:26](../../../crates/application/tests/audio_editor.rs#L26) |
| fn request | [源码:31](../../../crates/application/tests/audio_editor.rs#L31) |
| fn poll | [源码:39](../../../crates/application/tests/audio_editor.rs#L39) |
| fn device_controls_are_read_only_and_enable_selects_source_before_playback | [源码:44](../../../crates/application/tests/audio_editor.rs#L44) |
| fn failed_sync_and_missing_component_do_not_trigger_and_disable_remains_available | [源码:61](../../../crates/application/tests/audio_editor.rs#L61) |
| fn readonly_source_query_tracks_drafts_and_history_navigation | [源码:89](../../../crates/application/tests/audio_editor.rs#L89) |
| fn monitor_and_stop_are_readonly_and_not_blocked_by_project_sync_failure | [源码:119](../../../crates/application/tests/audio_editor.rs#L119) |
| fn library_control_resolves_actual_asset_and_source_without_component_or_history_edits | [源码:151](../../../crates/application/tests/audio_editor.rs#L151) |
| struct Lease | [源码:153](../../../crates/application/tests/audio_editor.rs#L153) |
| impl impl BlobLease for Lease | [源码:154](../../../crates/application/tests/audio_editor.rs#L154) |
| fn reference | [源码:155](../../../crates/application/tests/audio_editor.rs#L155) |
| struct Store | [源码:159](../../../crates/application/tests/audio_editor.rs#L159) |
| impl impl ResourceStore for Store | [源码:160](../../../crates/application/tests/audio_editor.rs#L160) |
| fn pin | [源码:161](../../../crates/application/tests/audio_editor.rs#L161) |

## crates/application/tests/audio_port.rs

199 行；SHA256 `2409f7e0191ccf6de1de937c232ddd5eac74962effdac74c8d9c4a5f792b061e`

导入：`use skin_editor_application::{values::*, *};`

| 定义 | 位置／签名 |
| --- | --- |
| struct Capture | [源码:4](../../../crates/application/tests/audio_port.rs#L4) |
| impl impl SkinAudioPort for Capture | [源码:11](../../../crates/application/tests/audio_port.rs#L11) |
| fn retained_drafts | [源码:12](../../../crates/application/tests/audio_port.rs#L12) |
| fn synchronize | [源码:15](../../../crates/application/tests/audio_port.rs#L15) |
| struct Driver | [源码:31](../../../crates/application/tests/audio_port.rs#L31) |
| impl impl Driver | [源码:35](../../../crates/application/tests/audio_port.rs#L35) |
| fn new | [源码:36](../../../crates/application/tests/audio_port.rs#L36) |
| fn id | [源码:42](../../../crates/application/tests/audio_port.rs#L42) |
| fn commit | [源码:47](../../../crates/application/tests/audio_port.rs#L47) |
| fn nav_request | [源码:57](../../../crates/application/tests/audio_port.rs#L57) |
| fn draft | [源码:66](../../../crates/application/tests/audio_port.rs#L66) |
| fn ds | [源码:75](../../../crates/application/tests/audio_port.rs#L75) |
| fn source | [源码:82](../../../crates/application/tests/audio_port.rs#L82) |
| fn insert | [源码:88](../../../crates/application/tests/audio_port.rs#L88) |
| fn navigation_source_changes_even_when_returning_to_the_same_node_between_syncs | [源码:96](../../../crates/application/tests/audio_port.rs#L96) |
| fn noop_and_failed_navigation_do_not_invalidate_audio_selection | [源码:115](../../../crates/application/tests/audio_port.rs#L115) |
| fn port_selects_actual_draft_and_adapter_failure_never_changes_project_or_history | [源码:136](../../../crates/application/tests/audio_port.rs#L136) |
| fn cancellation_facts_survive_other_edits_but_do_not_label_committed_drafts_cancelled | [源码:167](../../../crates/application/tests/audio_port.rs#L167) |

## crates/application/tests/commands.rs

970 行；SHA256 `c2c8b2b1ac4d2773604f4e27bf0b695881863c11e11b3bb89bdf364f3041fe0c`

导入：`use std::collections::BTreeMap;`, `use std::sync::Arc;`, `use std::sync::atomic::{AtomicUsize, Ordering};`, `use skin_editor_application::*;`, `use skin_editor_domain::*;`

| 定义 | 位置／签名 |
| --- | --- |
| fn aid | [源码:8](../../../crates/application/tests/commands.rs#L8) |
| fn did | [源码:11](../../../crates/application/tests/commands.rs#L11) |
| fn lid | [源码:14](../../../crates/application/tests/commands.rs#L14) |
| fn cid | [源码:17](../../../crates/application/tests/commands.rs#L17) |
| fn oid | [源码:20](../../../crates/application/tests/commands.rs#L20) |
| fn size | [源码:23](../../../crates/application/tests/commands.rs#L23) |
| fn transform | [源码:26](../../../crates/application/tests/commands.rs#L26) |
| fn blob | [源码:29](../../../crates/application/tests/commands.rs#L29) |
| fn asset | [源码:32](../../../crates/application/tests/commands.rs#L32) |
| fn doc | [源码:41](../../../crates/application/tests/commands.rs#L41) |
| fn group | [源码:44](../../../crates/application/tests/commands.rs#L44) |
| fn image | [源码:53](../../../crates/application/tests/commands.rs#L53) |
| fn cgroup | [源码:59](../../../crates/application/tests/commands.rs#L59) |
| fn member | [源码:66](../../../crates/application/tests/commands.rs#L66) |
| fn record | [源码:75](../../../crates/application/tests/commands.rs#L75) |
| fn mapping | [源码:78](../../../crates/application/tests/commands.rs#L78) |
| struct Lease | [源码:88](../../../crates/application/tests/commands.rs#L88) |
| impl impl BlobLease for Lease | [源码:92](../../../crates/application/tests/commands.rs#L92) |
| fn reference | [源码:93](../../../crates/application/tests/commands.rs#L93) |
| struct Store | [源码:97](../../../crates/application/tests/commands.rs#L97) |
| impl impl ResourceStore for Store | [源码:101](../../../crates/application/tests/commands.rs#L101) |
| fn pin | [源码:102](../../../crates/application/tests/commands.rs#L102) |
| fn session_with | [源码:111](../../../crates/application/tests/commands.rs#L111) |
| fn session | [源码:138](../../../crates/application/tests/commands.rs#L138) |
| fn request | [源码:141](../../../crates/application/tests/commands.rs#L141) |
| fn commit | [源码:151](../../../crates/application/tests/commands.rs#L151) |
| fn baseline | [源码:155](../../../crates/application/tests/commands.rs#L155) |
| fn assert_unchanged | [源码:190](../../../crates/application/tests/commands.rs#L190) |
| fn multi_command_success_publishes_exactly_one_node_and_owned_queries | [源码:196](../../../crates/application/tests/commands.rs#L196) |
| fn every_step_is_validated_and_later_failure_rolls_back_all_prior_commands | [源码:222](../../../crates/application/tests/commands.rs#L222) |
| fn a_failure_at_the_first_step_does_not_publish_or_consume_node_ids | [源码:252](../../../crates/application/tests/commands.rs#L252) |
| fn effective_noops_and_net_zero_batches_do_not_create_history | [源码:265](../../../crates/application/tests/commands.rs#L265) |
| fn no_net_change_never_pins_transient_resources | [源码:321](../../../crates/application/tests/commands.rs#L321) |
| fn revision_and_head_conflicts_are_checked_before_candidate_work | [源码:337](../../../crates/application/tests/commands.rs#L337) |
| fn retry_returns_original_result_even_after_another_commit_without_repinning | [源码:366](../../../crates/application/tests/commands.rs#L366) |
| fn rejected_and_no_change_requests_are_also_deduplicated | [源码:393](../../../crates/application/tests/commands.rs#L393) |
| fn wrong_session_and_old_api_are_rejected_and_do_not_reserve_request_id | [源码:414](../../../crates/application/tests/commands.rs#L414) |
| fn preflight_is_read_only_and_submission_rechecks_a_changed_revision | [源码:432](../../../crates/application/tests/commands.rs#L432) |
| fn unavailable_resource_rolls_back_and_releases_already_acquired_candidate_pins | [源码:454](../../../crates/application/tests/commands.rs#L454) |
| fn history_pins_shared_content_once_and_keeps_it_after_current_deletion | [源码:484](../../../crates/application/tests/commands.rs#L484) |
| fn a_store_returning_the_wrong_lease_is_rejected_atomically | [源码:520](../../../crates/application/tests/commands.rs#L520) |
| fn layer_updates_reorder_reparent_and_subtree_delete_preserve_explicit_semantics | [源码:550](../../../crates/application/tests/commands.rs#L550) |
| fn invalid_layer_moves_cycles_indices_and_asset_deletion_never_leak_partial_work | [源码:634](../../../crates/application/tests/commands.rs#L634) |
| fn reparenting_to_a_singular_parent_rejects_the_whole_batch | [源码:683](../../../crates/application/tests/commands.rs#L683) |
| fn component_changes_preserve_member_geometry_and_independent_outputs | [源码:721](../../../crates/application/tests/commands.rs#L721) |
| fn invalid_component_membership_cycles_and_mappings_roll_back | [源码:779](../../../crates/application/tests/commands.rs#L779) |
| fn configuration_and_output_failures_preserve_unknown_bytes_and_old_node | [源码:808](../../../crates/application/tests/commands.rs#L808) |
| fn deleting_dependencies_in_explicit_order_is_atomic_and_preserves_old_history | [源码:880](../../../crates/application/tests/commands.rs#L880) |
| fn history_query_has_explicit_scope_and_never_changes_active_state | [源码:920](../../../crates/application/tests/commands.rs#L920) |
| fn explicit_caps_reject_without_eviction_or_project_publication | [源码:936](../../../crates/application/tests/commands.rs#L936) |

## crates/application/tests/draft.rs

664 行；SHA256 `b1eee3863619ebbbe7ed8530659e33ee5f7773f67b80c4f12e5c1396480716ef`

导入：`use skin_editor_application::{values::*, *};`, `use std::sync::{ Arc, atomic::{AtomicBool, AtomicUsize, Ordering}, };`

| 定义 | 位置／签名 |
| --- | --- |
| struct Driver | [源码:7](../../../crates/application/tests/draft.rs#L7) |
| impl impl Driver | [源码:11](../../../crates/application/tests/draft.rs#L11) |
| fn new | [源码:12](../../../crates/application/tests/draft.rs#L12) |
| fn request | [源码:18](../../../crates/application/tests/draft.rs#L18) |
| fn apply | [源码:28](../../../crates/application/tests/draft.rs#L28) |
| fn state | [源码:32](../../../crates/application/tests/draft.rs#L32) |
| fn begin | [源码:39](../../../crates/application/tests/draft.rs#L39) |
| fn replace | [源码:45](../../../crates/application/tests/draft.rs#L45) |
| fn scope | [源码:52](../../../crates/application/tests/draft.rs#L52) |
| fn command_request | [源码:55](../../../crates/application/tests/draft.rs#L55) |
| fn insert | [源码:68](../../../crates/application/tests/draft.rs#L68) |
| fn bytes | [源码:74](../../../crates/application/tests/draft.rs#L74) |
| fn replacements_are_relative_to_base_and_one_commit_creates_one_node | [源码:79](../../../crates/application/tests/draft.rs#L79) |
| fn cancel_keeps_content_revision_history_and_candidate_id_admission_unchanged | [源码:133](../../../crates/application/tests/draft.rs#L133) |
| fn empty_and_zero_net_change_drafts_finish_without_history_or_revision | [源码:159](../../../crates/application/tests/draft.rs#L159) |
| fn failed_replacement_keeps_previous_valid_candidate_and_does_not_publish | [源码:193](../../../crates/application/tests/draft.rs#L193) |
| fn busy_blocks_new_writes_navigation_and_second_begin_but_not_queries_or_retries | [源码:217](../../../crates/application/tests/draft.rs#L217) |
| fn stale_base_draft_revision_identity_and_wrong_scope_are_rejected | [源码:263](../../../crates/application/tests/draft.rs#L263) |
| fn shared_receipts_replay_begin_update_commit_cancel_without_repeating_side_effects | [源码:329](../../../crates/application/tests/draft.rs#L329) |
| fn identical_replacement_preserves_draft_revision_but_new_payload_advances_it | [源码:387](../../../crates/application/tests/draft.rs#L387) |
| fn reserved_receipt_allows_cancel_at_capacity_and_never_evicts_receipts | [源码:411](../../../crates/application/tests/draft.rs#L411) |
| fn missing_draft_oversized_payload_and_insufficient_start_capacity_are_safe | [源码:453](../../../crates/application/tests/draft.rs#L453) |
| struct TestStore | [源码:508](../../../crates/application/tests/draft.rs#L508) |
| struct Lease | [源码:513](../../../crates/application/tests/draft.rs#L513) |
| impl impl BlobLease for Lease | [源码:518](../../../crates/application/tests/draft.rs#L518) |
| fn reference | [源码:519](../../../crates/application/tests/draft.rs#L519) |
| impl impl Drop for Lease | [源码:523](../../../crates/application/tests/draft.rs#L523) |
| fn drop | [源码:524](../../../crates/application/tests/draft.rs#L524) |
| impl impl ResourceStore for TestStore | [源码:528](../../../crates/application/tests/draft.rs#L528) |
| fn pin | [源码:529](../../../crates/application/tests/draft.rs#L529) |
| fn asset | [源码:542](../../../crates/application/tests/draft.rs#L542) |
| fn draft_resource_replacement_cancel_and_commit_have_distinct_lifetimes | [源码:555](../../../crates/application/tests/draft.rs#L555) |
| fn resource_failure_releases_new_leases_and_keeps_previous_draft_and_project | [源码:598](../../../crates/application/tests/draft.rs#L598) |
| fn draft_commit_at_old_node_preserves_other_branch_content | [源码:633](../../../crates/application/tests/draft.rs#L633) |

## crates/application/tests/history.rs

942 行；SHA256 `788ba4921c3d8c7a3b523f00fa23d8dade8d38db3a33dd70ef8ee66de27da0eb`

导入：`use std::collections::{BTreeMap, BTreeSet};`, `use std::sync::Arc;`, `use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};`, `use skin_editor_application::{values::*, *};`

| 定义 | 位置／签名 |
| --- | --- |
| fn record | [源码:7](../../../crates/application/tests/history.rs#L7) |
| fn value | [源码:10](../../../crates/application/tests/history.rs#L10) |
| fn plain_undo_redo_remembers_each_fork_in_the_undone_path | [源码:15](../../../crates/application/tests/history.rs#L15) |
| fn failed_no_change_and_cached_navigation_do_not_replace_the_remembered_child | [源码:56](../../../crates/application/tests/history.rs#L56) |
| fn ten_thousand_linear_undo_redo_steps_need_no_fork_preference_or_ancestor_address | [源码:101](../../../crates/application/tests/history.rs#L101) |
| struct Driver | [源码:133](../../../crates/application/tests/history.rs#L133) |
| impl impl Driver | [源码:137](../../../crates/application/tests/history.rs#L137) |
| fn new | [源码:138](../../../crates/application/tests/history.rs#L138) |
| fn with_session | [源码:141](../../../crates/application/tests/history.rs#L141) |
| fn command_request | [源码:144](../../../crates/application/tests/history.rs#L144) |
| fn history_request | [源码:156](../../../crates/application/tests/history.rs#L156) |
| fn commit | [源码:168](../../../crates/application/tests/history.rs#L168) |
| fn set | [源码:172](../../../crates/application/tests/history.rs#L172) |
| fn navigate | [源码:193](../../../crates/application/tests/history.rs#L193) |
| fn checkout | [源码:197](../../../crates/application/tests/history.rs#L197) |
| fn scope | [源码:200](../../../crates/application/tests/history.rs#L200) |
| fn undo_redo_and_checkout_preserve_old_descendants_when_editing_a_branch | [源码:206](../../../crates/application/tests/history.rs#L206) |
| fn navigation_increments_revision_even_between_equal_contents_without_new_nodes | [源码:261](../../../crates/application/tests/history.rs#L261) |
| fn root_undo_leaf_redo_and_read_queries_do_not_change_head_or_revision | [源码:284](../../../crates/application/tests/history.rs#L284) |
| fn navigation_scope_expected_state_and_unknown_targets_are_validated_atomically | [源码:312](../../../crates/application/tests/history.rs#L312) |
| fn navigation_receipts_are_idempotent_and_share_ids_with_content_commands | [源码:350](../../../crates/application/tests/history.rs#L350) |
| fn failed_and_no_change_navigation_receipts_keep_the_original_outcome | [源码:391](../../../crates/application/tests/history.rs#L391) |
| fn canonical_addresses_have_fixed_upper_bounds_and_reject_noncanonical_inputs | [源码:410](../../../crates/application/tests/history.rs#L410) |
| fn branch_coordinates_resolve_stably_after_checkout_and_display_sorting | [源码:448](../../../crates/application/tests/history.rs#L448) |
| fn history_pages_enforce_limits_offsets_and_session_scope | [源码:501](../../../crates/application/tests/history.rs#L501) |
| struct Lease | [源码:533](../../../crates/application/tests/history.rs#L533) |
| impl impl BlobLease for Lease | [源码:537](../../../crates/application/tests/history.rs#L537) |
| fn reference | [源码:538](../../../crates/application/tests/history.rs#L538) |
| struct Store | [源码:542](../../../crates/application/tests/history.rs#L542) |
| impl impl ResourceStore for Store | [源码:547](../../../crates/application/tests/history.rs#L547) |
| fn pin | [源码:548](../../../crates/application/tests/history.rs#L548) |
| fn cross_branch_resources_and_document_state_survive_deletion_and_store_unavailability | [源码:562](../../../crates/application/tests/history.rs#L562) |
| fn scale_ten_thousand_deep_history_queries_and_addresses_are_bounded | [源码:710](../../../crates/application/tests/history.rs#L710) |
| fn scale_ten_thousand_siblings_keep_old_coordinates_and_stable_paging | [源码:778](../../../crates/application/tests/history.rs#L778) |
| fn seeded_branch_navigation_matches_an_independent_snapshot_model | [源码:852](../../../crates/application/tests/history.rs#L852) |
| struct Reference | [源码:853](../../../crates/application/tests/history.rs#L853) |

## crates/domain/src/assets.rs

125 行；SHA256 `6daa3625dab9205497dd9230a460fa55441da022d83a237e644ce07c35e850a5`

导入：`use std::collections::BTreeMap;`, `use crate::{AssetId, AudioMetadata, ModelError, ObjectId, PixelSize};`

| 定义 | 位置／签名 |
| --- | --- |
| struct BlobRef | [源码:7](../../../crates/domain/src/assets.rs#L7) |
| impl impl BlobRef | [源码:12](../../../crates/domain/src/assets.rs#L12) |
| fn new | [源码:13](../../../crates/domain/src/assets.rs#L13) |
| fn sha256 | [源码:17](../../../crates/domain/src/assets.rs#L17) |
| fn byte_len | [源码:21](../../../crates/domain/src/assets.rs#L21) |
| enum AssetKind | [源码:27](../../../crates/domain/src/assets.rs#L27) |
| struct Asset | [源码:36](../../../crates/domain/src/assets.rs#L36) |
| impl impl Asset | [源码:43](../../../crates/domain/src/assets.rs#L43) |
| fn new | [源码:44](../../../crates/domain/src/assets.rs#L44) |
| fn id | [源码:61](../../../crates/domain/src/assets.rs#L61) |
| fn name | [源码:65](../../../crates/domain/src/assets.rs#L65) |
| fn blob | [源码:69](../../../crates/domain/src/assets.rs#L69) |
| fn kind | [源码:73](../../../crates/domain/src/assets.rs#L73) |
| struct AssetLibrary | [源码:79](../../../crates/domain/src/assets.rs#L79) |
| impl impl AssetLibrary | [源码:83](../../../crates/domain/src/assets.rs#L83) |
| fn new | [源码:84](../../../crates/domain/src/assets.rs#L84) |
| fn get | [源码:118](../../../crates/domain/src/assets.rs#L118) |
| fn iter | [源码:122](../../../crates/domain/src/assets.rs#L122) |
| impl impl ExactSizeIterator<Item = &Asset> | [源码:122](../../../crates/domain/src/assets.rs#L122) |

## crates/domain/src/audio.rs

98 行；SHA256 `807d2cbd247406d0e7a9a6305884e6c6122518f9eab919a3d43beb44db5efb7d`

导入：`use crate::{AssetId, AudioComponentId, ModelError};`

| 定义 | 位置／签名 |
| --- | --- |
| struct AudioMetadata | [源码:5](../../../crates/domain/src/audio.rs#L5) |
| impl impl AudioMetadata | [源码:10](../../../crates/domain/src/audio.rs#L10) |
| fn new | [源码:11](../../../crates/domain/src/audio.rs#L11) |
| fn sample_rate | [源码:21](../../../crates/domain/src/audio.rs#L21) |
| fn channels | [源码:24](../../../crates/domain/src/audio.rs#L24) |
| fn frames | [源码:27](../../../crates/domain/src/audio.rs#L27) |
| struct Gain | [源码:32](../../../crates/domain/src/audio.rs#L32) |
| impl impl Gain | [源码:33](../../../crates/domain/src/audio.rs#L33) |
| const UNITY | [源码:34](../../../crates/domain/src/audio.rs#L34) |
| fn new | [源码:35](../../../crates/domain/src/audio.rs#L35) |
| fn get | [源码:41](../../../crates/domain/src/audio.rs#L41) |
| struct SoundSlotKey | [源码:46](../../../crates/domain/src/audio.rs#L46) |
| impl impl SoundSlotKey | [源码:47](../../../crates/domain/src/audio.rs#L47) |
| fn new | [源码:49](../../../crates/domain/src/audio.rs#L49) |
| fn as_str | [源码:55](../../../crates/domain/src/audio.rs#L55) |
| struct AudioComponent | [源码:60](../../../crates/domain/src/audio.rs#L60) |
| impl impl AudioComponent | [源码:67](../../../crates/domain/src/audio.rs#L67) |
| fn new | [源码:68](../../../crates/domain/src/audio.rs#L68) |
| fn with_properties | [源码:78](../../../crates/domain/src/audio.rs#L78) |
| fn id | [源码:83](../../../crates/domain/src/audio.rs#L83) |
| fn slot | [源码:86](../../../crates/domain/src/audio.rs#L86) |
| fn asset | [源码:89](../../../crates/domain/src/audio.rs#L89) |
| fn gain | [源码:92](../../../crates/domain/src/audio.rs#L92) |
| fn visible | [源码:95](../../../crates/domain/src/audio.rs#L95) |

## crates/domain/src/components.rs

139 行；SHA256 `93276309ce2ccdcc165843633d26b2f90176d67fc02a6068fd0cfbfc8529d3c6`

导入：`use std::collections::BTreeMap;`, `use std::sync::Arc;`, `use crate::forest::{ForestIndex, index_forest};`, `use crate::{Affine2, ComponentId, DocumentId, ModelError, ObjectId};`

| 定义 | 位置／签名 |
| --- | --- |
| enum ComponentKind | [源码:8](../../../crates/domain/src/components.rs#L8) |
| struct ComponentNode | [源码:21](../../../crates/domain/src/components.rs#L21) |
| impl impl ComponentNode | [源码:27](../../../crates/domain/src/components.rs#L27) |
| fn children | [源码:28](../../../crates/domain/src/components.rs#L28) |
| struct ComponentTree | [源码:37](../../../crates/domain/src/components.rs#L37) |
| struct ComponentStorage | [源码:42](../../../crates/domain/src/components.rs#L42) |
| impl impl ComponentTree | [源码:51](../../../crates/domain/src/components.rs#L51) |
| fn new | [源码:52](../../../crates/domain/src/components.rs#L52) |
| fn roots | [源码:108](../../../crates/domain/src/components.rs#L108) |
| fn node | [源码:112](../../../crates/domain/src/components.rs#L112) |
| fn parent | [源码:116](../../../crates/domain/src/components.rs#L116) |
| fn world_transform | [源码:120](../../../crates/domain/src/components.rs#L120) |
| fn member | [源码:124](../../../crates/domain/src/components.rs#L124) |
| fn document_content_transform | [源码:128](../../../crates/domain/src/components.rs#L128) |
| fn iter | [源码:132](../../../crates/domain/src/components.rs#L132) |
| impl impl ExactSizeIterator<Item = &ComponentNode> | [源码:132](../../../crates/domain/src/components.rs#L132) |

## crates/domain/src/configuration.rs

79 行；SHA256 `2472c94b2fac0fcb69ed967d361bd6a1feecc3137af31bf54138dbf6030418fa`

导入：`use core::ops::Range;`, `use crate::ModelError;`

| 定义 | 位置／签名 |
| --- | --- |
| enum ConfigInterpretation | [源码:7](../../../crates/domain/src/configuration.rs#L7) |
| struct ConfigRecord | [源码:19](../../../crates/domain/src/configuration.rs#L19) |
| impl impl ConfigRecord | [源码:24](../../../crates/domain/src/configuration.rs#L24) |
| fn new | [源码:26](../../../crates/domain/src/configuration.rs#L26) |
| fn raw | [源码:48](../../../crates/domain/src/configuration.rs#L48) |
| fn interpretation | [源码:52](../../../crates/domain/src/configuration.rs#L52) |
| struct SkinConfiguration | [源码:60](../../../crates/domain/src/configuration.rs#L60) |
| impl impl SkinConfiguration | [源码:64](../../../crates/domain/src/configuration.rs#L64) |
| fn new | [源码:65](../../../crates/domain/src/configuration.rs#L65) |
| fn records | [源码:69](../../../crates/domain/src/configuration.rs#L69) |
| fn to_bytes | [源码:73](../../../crates/domain/src/configuration.rs#L73) |

## crates/domain/src/forest.rs

61 行；SHA256 `07731db4e0302b768fd995f297dc332aeda2f821f1b29055b1355a1df3ac7827`

导入：`use std::collections::BTreeMap;`, `use crate::{ModelError, ObjectId};`

| 定义 | 位置／签名 |
| --- | --- |
| struct ForestIndex | [源码:7](../../../crates/domain/src/forest.rs#L7) |
| impl impl<I> Default for ForestIndex<I> | [源码:12](../../../crates/domain/src/forest.rs#L12) |
| fn default | [源码:13](../../../crates/domain/src/forest.rs#L13) |
| fn index_forest | [源码:21](../../../crates/domain/src/forest.rs#L21) |

## crates/domain/src/geometry.rs

158 行；SHA256 `21ce8edbfb4ad963525dfe5ecfb79c7cef3cb8a53c8fc930939bff887335f4c9`

导入：`use crate::ModelError;`

| 定义 | 位置／签名 |
| --- | --- |
| struct PixelSize | [源码:5](../../../crates/domain/src/geometry.rs#L5) |
| impl impl PixelSize | [源码:10](../../../crates/domain/src/geometry.rs#L10) |
| fn new | [源码:11](../../../crates/domain/src/geometry.rs#L11) |
| fn width | [源码:22](../../../crates/domain/src/geometry.rs#L22) |
| fn height | [源码:26](../../../crates/domain/src/geometry.rs#L26) |
| struct Opacity | [源码:32](../../../crates/domain/src/geometry.rs#L32) |
| impl impl Opacity | [源码:34](../../../crates/domain/src/geometry.rs#L34) |
| const OPAQUE | [源码:35](../../../crates/domain/src/geometry.rs#L35) |
| fn new | [源码:37](../../../crates/domain/src/geometry.rs#L37) |
| fn get | [源码:44](../../../crates/domain/src/geometry.rs#L44) |
| struct Affine2 | [源码:51](../../../crates/domain/src/geometry.rs#L51) |
| impl impl Affine2 | [源码:53](../../../crates/domain/src/geometry.rs#L53) |
| const IDENTITY | [源码:54](../../../crates/domain/src/geometry.rs#L54) |
| fn new | [源码:56](../../../crates/domain/src/geometry.rs#L56) |
| fn coefficients | [源码:63](../../../crates/domain/src/geometry.rs#L63) |
| fn compose | [源码:68](../../../crates/domain/src/geometry.rs#L68) |
| fn inverse | [源码:82](../../../crates/domain/src/geometry.rs#L82) |
| fn rebased | [源码:100](../../../crates/domain/src/geometry.rs#L100) |
| fn point | [源码:104](../../../crates/domain/src/geometry.rs#L104) |
| struct Bounds | [源码:116](../../../crates/domain/src/geometry.rs#L116) |
| impl impl Bounds | [源码:121](../../../crates/domain/src/geometry.rs#L121) |
| fn transformed | [源码:122](../../../crates/domain/src/geometry.rs#L122) |
| fn union | [源码:144](../../../crates/domain/src/geometry.rs#L144) |
| fn min | [源码:151](../../../crates/domain/src/geometry.rs#L151) |
| fn max | [源码:155](../../../crates/domain/src/geometry.rs#L155) |

## crates/domain/src/identity.rs

28 行；SHA256 `410d53bc95072343060772cac51c19c694cbb691f1cf3587aa3eaeec84e74b56`

导入：`use core::num::NonZeroU128;`

| 定义 | 位置／签名 |
| --- | --- |
| macro_rules! identity | [源码:3](../../../crates/domain/src/identity.rs#L3) |
| impl impl $name | [源码:8](../../../crates/domain/src/identity.rs#L8) |
| fn new | [源码:9](../../../crates/domain/src/identity.rs#L9) |
| fn get | [源码:16](../../../crates/domain/src/identity.rs#L16) |

## crates/domain/src/layers.rs

149 行；SHA256 `ebefe1ea28a4e079c434469a7f4ed5e112acc9b60ca34f7675c4a2592f8e87df`

导入：`use std::collections::BTreeMap;`, `use std::sync::Arc;`, `use crate::forest::{ForestIndex, index_forest};`, `use crate::{Affine2, AssetId, DocumentId, LayerId, ModelError, ObjectId, Opacity, PixelSize};`

| 定义 | 位置／签名 |
| --- | --- |
| enum LayerKind | [源码:8](../../../crates/domain/src/layers.rs#L8) |
| struct LayerNode | [源码:20](../../../crates/domain/src/layers.rs#L20) |
| impl impl LayerNode | [源码:28](../../../crates/domain/src/layers.rs#L28) |
| fn children | [源码:29](../../../crates/domain/src/layers.rs#L29) |
| struct LayerTree | [源码:44](../../../crates/domain/src/layers.rs#L44) |
| struct LayerStorage | [源码:51](../../../crates/domain/src/layers.rs#L51) |
| impl impl LayerTree | [源码:58](../../../crates/domain/src/layers.rs#L58) |
| fn new | [源码:60](../../../crates/domain/src/layers.rs#L60) |
| fn roots | [源码:88](../../../crates/domain/src/layers.rs#L88) |
| fn node | [源码:92](../../../crates/domain/src/layers.rs#L92) |
| fn parent | [源码:96](../../../crates/domain/src/layers.rs#L96) |
| fn world_transform | [源码:100](../../../crates/domain/src/layers.rs#L100) |
| fn iter | [源码:104](../../../crates/domain/src/layers.rs#L104) |
| impl impl ExactSizeIterator<Item = &LayerNode> | [源码:104](../../../crates/domain/src/layers.rs#L104) |
| struct ElementDocument | [源码:114](../../../crates/domain/src/layers.rs#L114) |
| impl impl ElementDocument | [源码:121](../../../crates/domain/src/layers.rs#L121) |
| fn new | [源码:122](../../../crates/domain/src/layers.rs#L122) |
| fn with_visibility | [源码:130](../../../crates/domain/src/layers.rs#L130) |
| fn visible | [源码:134](../../../crates/domain/src/layers.rs#L134) |
| fn id | [源码:138](../../../crates/domain/src/layers.rs#L138) |
| fn canvas | [源码:142](../../../crates/domain/src/layers.rs#L142) |
| fn layers | [源码:146](../../../crates/domain/src/layers.rs#L146) |

## crates/domain/src/lib.rs

103 行；SHA256 `886b26f31d907a7aab8156624acb9a592a7b596ff9d08bc8b6fa5ac81c2f472d`

导入：`use assets::{Asset, AssetKind, AssetLibrary, BlobRef};`, `use audio::{AudioComponent, AudioMetadata, Gain, SoundSlotKey};`, `use components::{ComponentKind, ComponentNode, ComponentTree};`, `use configuration::{ConfigInterpretation, ConfigRecord, SkinConfiguration};`, `use geometry::{Affine2, Bounds, Opacity, PixelSize};`, `use identity::{AssetId, AudioComponentId, ComponentId, DocumentId, LayerId, OutputId};`, `use layers::{ElementDocument, LayerKind, LayerNode, LayerTree};`, `use output::{OutputMapping, OutputPath, OutputSource};`, `use project::{MODEL_VERSION, ProjectDefinition, SkinProject};`, `use core::num::NonZeroU128;`, `use super::ProjectId;`

| 定义 | 位置／签名 |
| --- | --- |
| mod assets | [源码:11](../../../crates/domain/src/lib.rs#L11) |
| mod audio | [源码:12](../../../crates/domain/src/lib.rs#L12) |
| mod components | [源码:13](../../../crates/domain/src/lib.rs#L13) |
| mod configuration | [源码:14](../../../crates/domain/src/lib.rs#L14) |
| mod forest | [源码:15](../../../crates/domain/src/lib.rs#L15) |
| mod geometry | [源码:16](../../../crates/domain/src/lib.rs#L16) |
| mod identity | [源码:17](../../../crates/domain/src/lib.rs#L17) |
| mod layers | [源码:18](../../../crates/domain/src/lib.rs#L18) |
| mod output | [源码:19](../../../crates/domain/src/lib.rs#L19) |
| mod project | [源码:20](../../../crates/domain/src/lib.rs#L20) |
| enum ObjectId | [源码:35](../../../crates/domain/src/lib.rs#L35) |
| enum ModelError | [源码:45](../../../crates/domain/src/lib.rs#L45) |
| static str | [源码:46](../../../crates/domain/src/lib.rs#L46) |
| struct ProjectId | [源码:69](../../../crates/domain/src/lib.rs#L69) |
| impl impl ProjectId | [源码:71](../../../crates/domain/src/lib.rs#L71) |
| fn new | [源码:73](../../../crates/domain/src/lib.rs#L73) |
| fn get | [源码:81](../../../crates/domain/src/lib.rs#L81) |
| mod tests | [源码:87](../../../crates/domain/src/lib.rs#L87) |
| fn rejects_missing_identity | [源码:91](../../../crates/domain/src/lib.rs#L91) |
| fn preserves_full_width_resolved_identities | [源码:96](../../../crates/domain/src/lib.rs#L96) |

## crates/domain/src/output.rs

116 行；SHA256 `238e5deb808cf3b5fd4064a5684e869406075cc45409bb8173a4f341d024f95a`

导入：`use crate::{AudioComponentId, DocumentId, ModelError, ObjectId, OutputId};`

| 定义 | 位置／签名 |
| --- | --- |
| enum OutputSource | [源码:4](../../../crates/domain/src/output.rs#L4) |
| impl impl OutputSource | [源码:8](../../../crates/domain/src/output.rs#L8) |
| fn object_id | [源码:9](../../../crates/domain/src/output.rs#L9) |
| struct OutputPath | [源码:20](../../../crates/domain/src/output.rs#L20) |
| impl impl OutputPath | [源码:22](../../../crates/domain/src/output.rs#L22) |
| fn new | [源码:23](../../../crates/domain/src/output.rs#L23) |
| fn as_str | [源码:44](../../../crates/domain/src/output.rs#L44) |
| fn collision_key | [源码:48](../../../crates/domain/src/output.rs#L48) |
| struct OutputMapping | [源码:54](../../../crates/domain/src/output.rs#L54) |
| impl impl OutputMapping | [源码:61](../../../crates/domain/src/output.rs#L61) |
| fn new | [源码:62](../../../crates/domain/src/output.rs#L62) |
| fn id | [源码:79](../../../crates/domain/src/output.rs#L79) |
| fn document | [源码:83](../../../crates/domain/src/output.rs#L83) |
| fn for_audio | [源码:89](../../../crates/domain/src/output.rs#L89) |
| fn source | [源码:105](../../../crates/domain/src/output.rs#L105) |
| fn element_key | [源码:109](../../../crates/domain/src/output.rs#L109) |
| fn path | [源码:113](../../../crates/domain/src/output.rs#L113) |

## crates/domain/src/project.rs

374 行；SHA256 `bd371fad3ef3fa50939a9481ea6481ef789f5485dd7ea22cb8feed4dbbe077df`

导入：`use std::collections::{BTreeMap, BTreeSet};`, `use crate::{ Affine2, Asset, AssetKind, AssetLibrary, AudioComponent, AudioComponentId, Bounds, ComponentId, ComponentKind, ComponentTree, DocumentId, ElementDocument, LayerId, LayerKind, ModelError, ObjectId, OutputMapping, OutputSource, ProjectId, SkinConfiguration, };`

| 定义 | 位置／签名 |
| --- | --- |
| const MODEL_VERSION | [源码:10](../../../crates/domain/src/project.rs#L10) |
| struct ProjectDefinition | [源码:14](../../../crates/domain/src/project.rs#L14) |
| impl impl ProjectDefinition | [源码:25](../../../crates/domain/src/project.rs#L25) |
| fn upgrade_v1 | [源码:28](../../../crates/domain/src/project.rs#L28) |
| fn empty | [源码:49](../../../crates/domain/src/project.rs#L49) |
| struct SkinProject | [源码:71](../../../crates/domain/src/project.rs#L71) |
| impl impl SkinProject | [源码:81](../../../crates/domain/src/project.rs#L81) |
| fn empty | [源码:83](../../../crates/domain/src/project.rs#L83) |
| fn from_definition | [源码:95](../../../crates/domain/src/project.rs#L95) |
| fn id | [源码:225](../../../crates/domain/src/project.rs#L225) |
| fn model_version | [源码:229](../../../crates/domain/src/project.rs#L229) |
| fn assets | [源码:233](../../../crates/domain/src/project.rs#L233) |
| fn document | [源码:237](../../../crates/domain/src/project.rs#L237) |
| fn documents | [源码:241](../../../crates/domain/src/project.rs#L241) |
| impl impl ExactSizeIterator<Item = &ElementDocument> | [源码:241](../../../crates/domain/src/project.rs#L241) |
| fn components | [源码:245](../../../crates/domain/src/project.rs#L245) |
| fn audio_component | [源码:249](../../../crates/domain/src/project.rs#L249) |
| fn audio_components | [源码:253](../../../crates/domain/src/project.rs#L253) |
| impl impl ExactSizeIterator<Item = &AudioComponent> | [源码:253](../../../crates/domain/src/project.rs#L253) |
| fn configuration | [源码:257](../../../crates/domain/src/project.rs#L257) |
| fn outputs | [源码:261](../../../crates/domain/src/project.rs#L261) |
| fn document_content_transform | [源码:266](../../../crates/domain/src/project.rs#L266) |
| fn document_workspace_transform | [源码:278](../../../crates/domain/src/project.rs#L278) |
| fn layer_bounds | [源码:305](../../../crates/domain/src/project.rs#L305) |
| fn component_bounds | [源码:343](../../../crates/domain/src/project.rs#L343) |

## crates/domain/tests/audio.rs

204 行；SHA256 `60bd84c634c2b1f7573e334001f13ac7fc1cdd77c75f4bc7fe8e484707b96d3f`

导入：`use skin_editor_domain::*;`

| 定义 | 位置／签名 |
| --- | --- |
| fn asset | [源码:3](../../../crates/domain/tests/audio.rs#L3) |
| fn sound | [源码:12](../../../crates/domain/tests/audio.rs#L12) |
| fn input | [源码:19](../../../crates/domain/tests/audio.rs#L19) |
| fn sound_components_share_a_source_without_visual_membership_or_shared_properties | [源码:26](../../../crates/domain/tests/audio.rs#L26) |
| fn dangling_image_duplicate_slot_and_duplicate_identity_are_rejected | [源码:46](../../../crates/domain/tests/audio.rs#L46) |
| fn numeric_metadata_and_slot_constraints_do_not_depend_on_a_decoder | [源码:73](../../../crates/domain/tests/audio.rs#L73) |
| fn blob_aliases_cannot_disagree_about_media_metadata | [源码:91](../../../crates/domain/tests/audio.rs#L91) |
| fn image_and_audio_outputs_share_one_collision_namespace | [源码:113](../../../crates/domain/tests/audio.rs#L113) |
| fn v1_upgrade_is_explicit_preserves_opaque_content_and_rejects_unknown_versions | [源码:161](../../../crates/domain/tests/audio.rs#L161) |

## crates/domain/tests/model.rs

814 行；SHA256 `e2f91c50f6a34cf6c6c40502115796ffe6bf23eb5a0cea55cca061aa620feebf`

导入：`use skin_editor_domain::*;`

| 定义 | 位置／签名 |
| --- | --- |
| fn aid | [源码:3](../../../crates/domain/tests/model.rs#L3) |
| fn did | [源码:6](../../../crates/domain/tests/model.rs#L6) |
| fn lid | [源码:9](../../../crates/domain/tests/model.rs#L9) |
| fn cid | [源码:12](../../../crates/domain/tests/model.rs#L12) |
| fn oid | [源码:15](../../../crates/domain/tests/model.rs#L15) |
| fn size | [源码:18](../../../crates/domain/tests/model.rs#L18) |
| fn affine | [源码:21](../../../crates/domain/tests/model.rs#L21) |
| fn translation | [源码:24](../../../crates/domain/tests/model.rs#L24) |
| fn scale | [源码:27](../../../crates/domain/tests/model.rs#L27) |
| fn close | [源码:30](../../../crates/domain/tests/model.rs#L30) |
| fn image | [源码:35](../../../crates/domain/tests/model.rs#L35) |
| fn group | [源码:44](../../../crates/domain/tests/model.rs#L44) |
| fn cgroup | [源码:55](../../../crates/domain/tests/model.rs#L55) |
| fn member | [源码:64](../../../crates/domain/tests/model.rs#L64) |
| fn document | [源码:74](../../../crates/domain/tests/model.rs#L74) |
| fn asset | [源码:81](../../../crates/domain/tests/model.rs#L81) |
| fn definition | [源码:90](../../../crates/domain/tests/model.rs#L90) |
| fn workspace_document_query_preserves_membership_and_includes_member_mapping | [源码:98](../../../crates/domain/tests/model.rs#L98) |
| fn output | [源码:142](../../../crates/domain/tests/model.rs#L142) |
| fn object_id_types_preserve_full_width_and_reject_zero | [源码:153](../../../crates/domain/tests/model.rs#L153) |
| fn numeric_values_reject_nonfinite_invalid_extents_and_byte_overflow | [源码:164](../../../crates/domain/tests/model.rs#L164) |
| fn rebase_preserves_world_geometry_including_shear_and_reflection | [源码:182](../../../crates/domain/tests/model.rs#L182) |
| fn layer_groups_keep_order_local_parameters_and_composed_geometry | [源码:197](../../../crates/domain/tests/model.rs#L197) |
| fn layer_forests_reject_duplicate_ids_roots_and_missing_edges | [源码:233](../../../crates/domain/tests/model.rs#L233) |
| fn layer_forests_reject_multiple_parents_and_duplicate_children | [源码:258](../../../crates/domain/tests/model.rs#L258) |
| fn disconnected_and_self_cycles_are_rejected | [源码:289](../../../crates/domain/tests/model.rs#L289) |
| fn accumulated_world_overflow_is_rejected_before_project_publication | [源码:308](../../../crates/domain/tests/model.rs#L308) |
| fn deep_layer_tree_build_traversal_clone_and_drop_are_iterative | [源码:322](../../../crates/domain/tests/model.rs#L322) |
| fn library_shares_blob_content_but_keeps_asset_identity | [源码:345](../../../crates/domain/tests/model.rs#L345) |
| fn library_rejects_duplicate_identity_and_contradictory_blob_metadata | [源码:359](../../../crates/domain/tests/model.rs#L359) |
| fn shared_image_layers_have_independent_parameters_and_full_geometric_bounds | [源码:391](../../../crates/domain/tests/model.rs#L391) |
| fn empty_groups_have_no_bounds_but_keep_their_transform | [源码:423](../../../crates/domain/tests/model.rs#L423) |
| fn project_rejects_missing_image_sources_and_nonimage_layers | [源码:446](../../../crates/domain/tests/model.rs#L446) |
| fn project_rejects_duplicate_documents_and_project_wide_layer_ids | [源码:467](../../../crates/domain/tests/model.rs#L467) |
| fn changing_a_detached_definition_never_changes_the_validated_value | [源码:485](../../../crates/domain/tests/model.rs#L485) |
| fn nested_component_transform_affects_two_distinct_document_outputs | [源码:495](../../../crates/domain/tests/model.rs#L495) |
| fn component_membership_is_unique_even_in_separate_roots | [源码:541](../../../crates/domain/tests/model.rs#L541) |
| fn components_require_invertible_member_maps_but_allow_zero_scale_edits | [源码:555](../../../crates/domain/tests/model.rs#L555) |
| fn component_forests_reject_cycles_missing_children_and_multiple_parents | [源码:575](../../../crates/domain/tests/model.rs#L575) |
| fn component_member_references_must_exist_in_project | [源码:618](../../../crates/domain/tests/model.rs#L618) |
| fn ungrouped_documents_have_identity_component_transform | [源码:629](../../../crates/domain/tests/model.rs#L629) |
| fn deep_components_validate_and_drop_without_recursive_nodes | [源码:639](../../../crates/domain/tests/model.rs#L639) |
| fn composed_document_and_component_extents_cannot_overflow | [源码:662](../../../crates/domain/tests/model.rs#L662) |
| fn configuration_preserves_unknown_bytes_duplicates_and_line_endings | [源码:684](../../../crates/domain/tests/model.rs#L684) |
| fn configuration_ranges_reject_out_of_bounds_empty_keys_and_overlap | [源码:715](../../../crates/domain/tests/model.rs#L715) |
| fn output_paths_are_relative_and_do_not_guess_element_from_filename | [源码:747](../../../crates/domain/tests/model.rs#L747) |
| fn output_conflicts_include_case_and_file_directory_prefixes | [源码:760](../../../crates/domain/tests/model.rs#L760) |
| fn output_identity_and_document_references_are_checked | [源码:780](../../../crates/domain/tests/model.rs#L780) |
| fn unsupported_model_version_is_explicit_and_empty_root_is_valid | [源码:802](../../../crates/domain/tests/model.rs#L802) |

## desktop/audio/src/audition_edit.rs

82 行；SHA256 `1d3643839c0aa3fc399395f01b2364411d7c86738624ef2abd0395635edef473`

导入：`use crate::{Origin, SoundPlan};`

| 定义 | 位置／签名 |
| --- | --- |
| fn same_content | [源码:4](../../../desktop/audio/src/audition_edit.rs#L4) |
| struct AuditionChange | [源码:11](../../../desktop/audio/src/audition_edit.rs#L11) |
| struct AuditionEdits | [源码:17](../../../desktop/audio/src/audition_edit.rs#L17) |
| impl impl AuditionEdits | [源码:22](../../../desktop/audio/src/audition_edit.rs#L22) |
| fn select | [源码:24](../../../desktop/audio/src/audition_edit.rs#L24) |

## desktop/audio/src/background.rs

128 行；SHA256 `248ce86de90fa360a9ae192a82ef8329928020c5f0f7ed633b806f86f6882939`

导入：`use crate::{PcmBuffer, PcmError};`, `use super::*;`, `use crate::PcmBudget;`

| 定义 | 位置／签名 |
| --- | --- |
| struct BackgroundTrack | [源码:3](../../../desktop/audio/src/background.rs#L3) |
| impl impl BackgroundTrack | [源码:11](../../../desktop/audio/src/background.rs#L11) |
| fn new | [源码:12](../../../desktop/audio/src/background.rs#L12) |
| fn format | [源码:31](../../../desktop/audio/src/background.rs#L31) |
| fn pcm | [源码:34](../../../desktop/audio/src/background.rs#L34) |
| fn position | [源码:37](../../../desktop/audio/src/background.rs#L37) |
| fn frames | [源码:40](../../../desktop/audio/src/background.rs#L40) |
| fn is_running | [源码:43](../../../desktop/audio/src/background.rs#L43) |
| fn set_running | [源码:46](../../../desktop/audio/src/background.rs#L46) |
| fn restore_position | [源码:50](../../../desktop/audio/src/background.rs#L50) |
| fn next_frame | [源码:57](../../../desktop/audio/src/background.rs#L57) |
| mod tests | [源码:73](../../../desktop/audio/src/background.rs#L73) |
| fn pause_restore_and_eof_preserve_the_shared_reference | [源码:77](../../../desktop/audio/src/background.rs#L77) |
| fn invalid_reference_never_becomes_a_timeline | [源码:108](../../../desktop/audio/src/background.rs#L108) |

## desktop/audio/src/lib.rs

140 行；SHA256 `f4c3706c2270a71348b0bbb6722789f25337c3d4ec7b5d5bb56b240a64be47a6`

导入：`use background::BackgroundTrack;`, `use audition_edit::{AuditionChange, AuditionEdits};`, `use voice::{MonitorFormat, VoiceHandoff, VoiceRuntimeKey, VoiceValidity};`, `use pcm::*;`, `use runtime::*;`, `use skin_editor_domain::{ AssetId, AssetKind, AudioComponentId, AudioMetadata, BlobRef, Gain, SkinProject, };`

| 定义 | 位置／签名 |
| --- | --- |
| mod background | [源码:4](../../../desktop/audio/src/lib.rs#L4) |
| mod pcm | [源码:5](../../../desktop/audio/src/lib.rs#L5) |
| mod audition_edit | [源码:7](../../../desktop/audio/src/lib.rs#L7) |
| mod runtime | [源码:8](../../../desktop/audio/src/lib.rs#L8) |
| mod voice | [源码:10](../../../desktop/audio/src/lib.rs#L10) |
| mod separated_model | [源码:14](../../../desktop/audio/src/lib.rs#L14) |
| const SOUND_PLAN_VERSION | [源码:21](../../../desktop/audio/src/lib.rs#L21) |
| enum SoundTarget | [源码:24](../../../desktop/audio/src/lib.rs#L24) |
| impl impl From<AudioComponentId> for SoundTarget | [源码:28](../../../desktop/audio/src/lib.rs#L28) |
| fn from | [源码:29](../../../desktop/audio/src/lib.rs#L29) |
| struct SoundPlan | [源码:34](../../../desktop/audio/src/lib.rs#L34) |
| enum OutputAction | [源码:44](../../../desktop/audio/src/lib.rs#L44) |
| enum AudioError | [源码:53](../../../desktop/audio/src/lib.rs#L53) |
| impl impl SoundPlan | [源码:64](../../../desktop/audio/src/lib.rs#L64) |
| fn build | [源码:65](../../../desktop/audio/src/lib.rs#L65) |
| fn output_action | [源码:87](../../../desktop/audio/src/lib.rs#L87) |
| fn original_bytes | [源码:102](../../../desktop/audio/src/lib.rs#L102) |
| fn process_pcm | [源码:119](../../../desktop/audio/src/lib.rs#L119) |

## desktop/audio/src/pcm.rs

130 行；SHA256 `82506312e2dbe9b99af37cf2e0577cbd4e2dc79ab9930af3f70d762a964ef2b2`

导入：`use std::sync::{ Arc, Weak, atomic::{AtomicUsize, Ordering}, };`

| 定义 | 位置／签名 |
| --- | --- |
| struct PcmUsage | [源码:8](../../../desktop/audio/src/pcm.rs#L8) |
| enum PcmError | [源码:14](../../../desktop/audio/src/pcm.rs#L14) |
| struct Budget | [源码:19](../../../desktop/audio/src/pcm.rs#L19) |
| struct PcmBudget | [源码:26](../../../desktop/audio/src/pcm.rs#L26) |
| impl impl Default for PcmBudget | [源码:27](../../../desktop/audio/src/pcm.rs#L27) |
| fn default | [源码:28](../../../desktop/audio/src/pcm.rs#L28) |
| impl impl PcmBudget | [源码:32](../../../desktop/audio/src/pcm.rs#L32) |
| fn new | [源码:33](../../../desktop/audio/src/pcm.rs#L33) |
| fn reserve | [源码:44](../../../desktop/audio/src/pcm.rs#L44) |
| fn usage | [源码:62](../../../desktop/audio/src/pcm.rs#L62) |
| fn same_pool | [源码:69](../../../desktop/audio/src/pcm.rs#L69) |
| struct PcmReservation | [源码:74](../../../desktop/audio/src/pcm.rs#L74) |
| impl impl PcmReservation | [源码:78](../../../desktop/audio/src/pcm.rs#L78) |
| fn allocate | [源码:79](../../../desktop/audio/src/pcm.rs#L79) |
| impl impl Drop for PcmReservation | [源码:87](../../../desktop/audio/src/pcm.rs#L87) |
| fn drop | [源码:88](../../../desktop/audio/src/pcm.rs#L88) |
| struct Allocation | [源码:96](../../../desktop/audio/src/pcm.rs#L96) |
| struct PcmBuffer | [源码:101](../../../desktop/audio/src/pcm.rs#L101) |
| impl impl PcmBuffer | [源码:102](../../../desktop/audio/src/pcm.rs#L102) |
| fn samples | [源码:103](../../../desktop/audio/src/pcm.rs#L103) |
| fn samples_mut | [源码:106](../../../desktop/audio/src/pcm.rs#L106) |
| fn budget | [源码:111](../../../desktop/audio/src/pcm.rs#L111) |
| fn same_allocation | [源码:114](../../../desktop/audio/src/pcm.rs#L114) |
| fn downgrade | [源码:117](../../../desktop/audio/src/pcm.rs#L117) |
| struct PcmWeak | [源码:122](../../../desktop/audio/src/pcm.rs#L122) |
| impl impl PcmWeak | [源码:123](../../../desktop/audio/src/pcm.rs#L123) |
| fn is_alive | [源码:124](../../../desktop/audio/src/pcm.rs#L124) |
| fn upgrade | [源码:127](../../../desktop/audio/src/pcm.rs#L127) |

## desktop/audio/src/runtime.rs

1223 行；SHA256 `bf530944b85fdcb2fdd886a498bd0604990085f1b5b7fbc78237d017ca4f42b3`

导入：`use crate::{SoundPlan, SoundTarget};`, `use skin_editor_domain::AudioComponentId;`, `use std::collections::{BTreeMap, VecDeque};`

| 定义 | 位置／签名 |
| --- | --- |
| const MAX_VOICES | [源码:6](../../../desktop/audio/src/runtime.rs#L6) |
| const MAX_EVENTS | [源码:7](../../../desktop/audio/src/runtime.rs#L7) |
| const MAX_BINDINGS | [源码:8](../../../desktop/audio/src/runtime.rs#L8) |
| const MAX_SOUND_SAMPLES | [源码:9](../../../desktop/audio/src/runtime.rs#L9) |
| const MIX_CONVERSION_VERSION | [源码:11](../../../desktop/audio/src/runtime.rs#L11) |
| struct Origin | [源码:14](../../../desktop/audio/src/runtime.rs#L14) |
| enum Channel | [源码:22](../../../desktop/audio/src/runtime.rs#L22) |
| impl impl Channel | [源码:26](../../../desktop/audio/src/runtime.rs#L26) |
| fn index | [源码:27](../../../desktop/audio/src/runtime.rs#L27) |
| struct GateToken | [源码:36](../../../desktop/audio/src/runtime.rs#L36) |
| struct BuildRequest | [源码:43](../../../desktop/audio/src/runtime.rs#L43) |
| enum PrepareFailure | [源码:49](../../../desktop/audio/src/runtime.rs#L49) |
| enum RuntimeError | [源码:59](../../../desktop/audio/src/runtime.rs#L59) |
| enum Completion | [源码:75](../../../desktop/audio/src/runtime.rs#L75) |
| enum Preparation | [源码:81](../../../desktop/audio/src/runtime.rs#L81) |
| struct BindingStatus | [源码:87](../../../desktop/audio/src/runtime.rs#L87) |
| struct PreparedSound | [源码:94](../../../desktop/audio/src/runtime.rs#L94) |
| struct PreparedKey | [源码:101](../../../desktop/audio/src/runtime.rs#L101) |
| impl impl PreparedKey | [源码:110](../../../desktop/audio/src/runtime.rs#L110) |
| fn new | [源码:111](../../../desktop/audio/src/runtime.rs#L111) |
| struct PreparedWeak | [源码:125](../../../desktop/audio/src/runtime.rs#L125) |
| impl impl PreparedWeak | [源码:129](../../../desktop/audio/src/runtime.rs#L129) |
| fn is_alive | [源码:130](../../../desktop/audio/src/runtime.rs#L130) |
| fn upgrade_for | [源码:133](../../../desktop/audio/src/runtime.rs#L133) |
| impl impl PreparedSound | [源码:149](../../../desktop/audio/src/runtime.rs#L149) |
| fn from_source | [源码:152](../../../desktop/audio/src/runtime.rs#L152) |
| fn from_decoded | [源码:169](../../../desktop/audio/src/runtime.rs#L169) |
| fn samples | [源码:195](../../../desktop/audio/src/runtime.rs#L195) |
| fn pcm | [源码:198](../../../desktop/audio/src/runtime.rs#L198) |
| fn monitor_gain | [源码:201](../../../desktop/audio/src/runtime.rs#L201) |
| fn format | [源码:209](../../../desktop/audio/src/runtime.rs#L209) |
| fn cache_key | [源码:212](../../../desktop/audio/src/runtime.rs#L212) |
| fn downgrade | [源码:215](../../../desktop/audio/src/runtime.rs#L215) |
| struct Effective | [源码:223](../../../desktop/audio/src/runtime.rs#L223) |
| struct Binding | [源码:227](../../../desktop/audio/src/runtime.rs#L227) |
| struct Event | [源码:233](../../../desktop/audio/src/runtime.rs#L233) |
| struct Voice | [源码:240](../../../desktop/audio/src/runtime.rs#L240) |
| impl impl Drop for Voice | [源码:249](../../../desktop/audio/src/runtime.rs#L249) |
| fn drop | [源码:250](../../../desktop/audio/src/runtime.rs#L250) |
| struct TriggerReceipt | [源码:257](../../../desktop/audio/src/runtime.rs#L257) |
| struct VoiceInfo | [源码:265](../../../desktop/audio/src/runtime.rs#L265) |
| struct MixReport | [源码:273](../../../desktop/audio/src/runtime.rs#L273) |
| struct Runtime | [源码:279](../../../desktop/audio/src/runtime.rs#L279) |
| fn next | [源码:301](../../../desktop/audio/src/runtime.rs#L301) |
| impl impl Runtime | [源码:304](../../../desktop/audio/src/runtime.rs#L304) |
| fn new | [源码:305](../../../desktop/audio/src/runtime.rs#L305) |
| fn pcm_budget | [源码:308](../../../desktop/audio/src/runtime.rs#L308) |
| fn with_budget | [源码:311](../../../desktop/audio/src/runtime.rs#L311) |
| fn enable_minimum_conversion | [源码:344](../../../desktop/audio/src/runtime.rs#L344) |
| fn output_activity | [源码:351](../../../desktop/audio/src/runtime.rs#L351) |
| fn output_format | [源码:362](../../../desktop/audio/src/runtime.rs#L362) |
| fn synchronize | [源码:368](../../../desktop/audio/src/runtime.rs#L368) |
| fn synchronize_cancelled | [源码:376](../../../desktop/audio/src/runtime.rs#L376) |
| fn request_library | [源码:515](../../../desktop/audio/src/runtime.rs#L515) |
| fn library_sound | [源码:545](../../../desktop/audio/src/runtime.rs#L545) |
| fn library_status | [源码:554](../../../desktop/audio/src/runtime.rs#L554) |
| fn fail_library | [源码:569](../../../desktop/audio/src/runtime.rs#L569) |
| fn clear_library | [源码:578](../../../desktop/audio/src/runtime.rs#L578) |
| fn complete | [源码:593](../../../desktop/audio/src/runtime.rs#L593) |
| fn retry | [源码:644](../../../desktop/audio/src/runtime.rs#L644) |
| fn status | [源码:670](../../../desktop/audio/src/runtime.rs#L670) |
| fn retained_drafts | [源码:680](../../../desktop/audio/src/runtime.rs#L680) |
| fn invalidate_drafts | [源码:708](../../../desktop/audio/src/runtime.rs#L708) |
| fn token | [源码:732](../../../desktop/audio/src/runtime.rs#L732) |
| fn valid | [源码:740](../../../desktop/audio/src/runtime.rs#L740) |
| fn clear_lane | [源码:752](../../../desktop/audio/src/runtime.rs#L752) |
| fn set_scene_running | [源码:760](../../../desktop/audio/src/runtime.rs#L760) |
| fn stop_audition | [源码:775](../../../desktop/audio/src/runtime.rs#L775) |
| fn seek_scene | [源码:786](../../../desktop/audio/src/runtime.rs#L786) |
| fn invalidate_device | [源码:791](../../../desktop/audio/src/runtime.rs#L791) |
| fn rebuild_device | [源码:806](../../../desktop/audio/src/runtime.rs#L806) |
| fn device_epoch | [源码:816](../../../desktop/audio/src/runtime.rs#L816) |
| fn start | [源码:820](../../../desktop/audio/src/runtime.rs#L820) |
| fn trigger | [源码:851](../../../desktop/audio/src/runtime.rs#L851) |
| impl impl Into<SoundTarget>, at_mix_frame: u64, ) -> Result<TriggerReceipt, RuntimeError> | [源码:854](../../../desktop/audio/src/runtime.rs#L854) |
| fn mix | [源码:918](../../../desktop/audio/src/runtime.rs#L918) |
| fn mix_split_buffers | [源码:930](../../../desktop/audio/src/runtime.rs#L930) |
| fn mix_split_buffers_marked | [源码:939](../../../desktop/audio/src/runtime.rs#L939) |
| fn split_buffers | [源码:947](../../../desktop/audio/src/runtime.rs#L947) |
| fn mix_frames | [源码:982](../../../desktop/audio/src/runtime.rs#L982) |
| impl impl FnMut(usize, [f32; 2], [f32; 2], [u64; 2]), ) -> Result<MixReport, RuntimeError> | [源码:985](../../../desktop/audio/src/runtime.rs#L985) |
| fn mix_buffer | [源码:1065](../../../desktop/audio/src/runtime.rs#L1065) |
| fn voices | [源码:1076](../../../desktop/audio/src/runtime.rs#L1076) |
| impl impl Iterator<Item = VoiceInfo> + '_ | [源码:1076](../../../desktop/audio/src/runtime.rs#L1076) |
| fn queued | [源码:1085](../../../desktop/audio/src/runtime.rs#L1085) |
| fn mix_frame | [源码:1088](../../../desktop/audio/src/runtime.rs#L1088) |
| fn scene_frame | [源码:1091](../../../desktop/audio/src/runtime.rs#L1091) |
| fn enable_voice_handoff | [源码:1097](../../../desktop/audio/src/runtime.rs#L1097) |
| fn voice_handoff_enabled | [源码:1110](../../../desktop/audio/src/runtime.rs#L1110) |
| fn is_closed | [源码:1113](../../../desktop/audio/src/runtime.rs#L1113) |
| fn synchronize_handoff_clock | [源码:1118](../../../desktop/audio/src/runtime.rs#L1118) |
| fn voice_runtime_key | [源码:1136](../../../desktop/audio/src/runtime.rs#L1136) |
| fn scene_running | [源码:1141](../../../desktop/audio/src/runtime.rs#L1141) |
| fn handoff_waiting | [源码:1144](../../../desktop/audio/src/runtime.rs#L1144) |
| fn take_voice_handoff | [源码:1154](../../../desktop/audio/src/runtime.rs#L1154) |
| fn finish_voice_handoff | [源码:1181](../../../desktop/audio/src/runtime.rs#L1181) |
| fn retry_voice_handoff | [源码:1201](../../../desktop/audio/src/runtime.rs#L1201) |
| fn shutdown | [源码:1215](../../../desktop/audio/src/runtime.rs#L1215) |

## desktop/audio/src/separated_model.rs

958 行；SHA256 `be42cddee960a8ecc71863ee78dcbbbff7b2c79701c1b83f10cc73269665ecc6`

导入：`use crate::{BackgroundTrack, Channel, MAX_VOICES, Origin, PcmBudget, PcmBuffer, PcmReservation};`

| 定义 | 位置／签名 |
| --- | --- |
| const RESERVE | [源码:6](../../../desktop/audio/src/separated_model.rs#L6) |
| const PERIOD | [源码:7](../../../desktop/audio/src/separated_model.rs#L7) |
| const RATE | [源码:8](../../../desktop/audio/src/separated_model.rs#L8) |
| fn lane | [源码:10](../../../desktop/audio/src/separated_model.rs#L10) |
| struct Token | [源码:18](../../../desktop/audio/src/separated_model.rs#L18) |
| struct Gates | [源码:24](../../../desktop/audio/src/separated_model.rs#L24) |
| impl impl Gates | [源码:32](../../../desktop/audio/src/separated_model.rs#L32) |
| fn token | [源码:33](../../../desktop/audio/src/separated_model.rs#L33) |
| fn permits | [源码:41](../../../desktop/audio/src/separated_model.rs#L41) |
| struct Key | [源码:49](../../../desktop/audio/src/separated_model.rs#L49) |
| struct Voice | [源码:53](../../../desktop/audio/src/separated_model.rs#L53) |
| struct Pin | [源码:60](../../../desktop/audio/src/separated_model.rs#L60) |
| struct Ready | [源码:66](../../../desktop/audio/src/separated_model.rs#L66) |
| impl impl Ready | [源码:67](../../../desktop/audio/src/separated_model.rs#L67) |
| fn new | [源码:68](../../../desktop/audio/src/separated_model.rs#L68) |
| struct Frame | [源码:81](../../../desktop/audio/src/separated_model.rs#L81) |
| struct Reserve | [源码:85](../../../desktop/audio/src/separated_model.rs#L85) |
| impl impl Reserve | [源码:90](../../../desktop/audio/src/separated_model.rs#L90) |
| fn push | [源码:91](../../../desktop/audio/src/separated_model.rs#L91) |
| fn pop | [源码:96](../../../desktop/audio/src/separated_model.rs#L96) |
| fn clear | [源码:105](../../../desktop/audio/src/separated_model.rs#L105) |
| struct Bridge | [源码:114](../../../desktop/audio/src/separated_model.rs#L114) |
| struct Owner | [源码:121](../../../desktop/audio/src/separated_model.rs#L121) |
| enum Refusal | [源码:131](../../../desktop/audio/src/separated_model.rs#L131) |
| impl impl Owner | [源码:141](../../../desktop/audio/src/separated_model.rs#L141) |
| fn admit | [源码:142](../../../desktop/audio/src/separated_model.rs#L142) |
| fn collect | [源码:186](../../../desktop/audio/src/separated_model.rs#L186) |
| fn refill | [源码:201](../../../desktop/audio/src/separated_model.rs#L201) |
| fn reconcile_pause | [源码:218](../../../desktop/audio/src/separated_model.rs#L218) |
| fn invalidate_source | [源码:230](../../../desktop/audio/src/separated_model.rs#L230) |
| struct Report | [源码:238](../../../desktop/audio/src/separated_model.rs#L238) |
| struct Callback | [源码:245](../../../desktop/audio/src/separated_model.rs#L245) |
| impl impl Callback | [源码:250](../../../desktop/audio/src/separated_model.rs#L250) |
| fn output | [源码:253](../../../desktop/audio/src/separated_model.rs#L253) |
| fn retire | [源码:309](../../../desktop/audio/src/separated_model.rs#L309) |
| struct Model | [源码:316](../../../desktop/audio/src/separated_model.rs#L316) |
| impl impl Model | [源码:321](../../../desktop/audio/src/separated_model.rs#L321) |
| fn new | [源码:322](../../../desktop/audio/src/separated_model.rs#L322) |
| fn ready | [源码:377](../../../desktop/audio/src/separated_model.rs#L377) |
| fn admit | [源码:382](../../../desktop/audio/src/separated_model.rs#L382) |
| fn bridge_token | [源码:387](../../../desktop/audio/src/separated_model.rs#L387) |
| fn output | [源码:390](../../../desktop/audio/src/separated_model.rs#L390) |
| fn pause | [源码:402](../../../desktop/audio/src/separated_model.rs#L402) |
| fn resume | [源码:408](../../../desktop/audio/src/separated_model.rs#L408) |
| fn close | [源码:423](../../../desktop/audio/src/separated_model.rs#L423) |
| fn origin | [源码:429](../../../desktop/audio/src/separated_model.rs#L429) |
| fn refill_before_or_after_admission_never_places_ready_voice_behind_reserve | [源码:440](../../../desktop/audio/src/separated_model.rs#L440) |
| fn three_burst_callbacks_keep_background_and_existing_voices_without_owner_steps | [源码:466](../../../desktop/audio/src/separated_model.rs#L466) |
| fn finite_reserve_and_premix_counterexamples_remain_failures | [源码:489](../../../desktop/audio/src/separated_model.rs#L489) |
| fn pause_ack_restores_submission_anchor_and_audition_remains_independent | [源码:511](../../../desktop/audio/src/separated_model.rs#L511) |
| fn cancelled_pending_and_active_sounds_retire_without_releasing_pcm_in_callback | [源码:552](../../../desktop/audio/src/separated_model.rs#L552) |
| fn complete_source_identity_and_each_gate_reject_late_requests | [源码:596](../../../desktop/audio/src/separated_model.rs#L596) |
| fn exhausted_credits_do_not_block_stop_and_retirement_cannot_overwrite_or_reuse_keys | [源码:644](../../../desktop/audio/src/separated_model.rs#L644) |
| fn voice_admitted_inside_burst_starts_next_callback_without_dropping_existing_audio | [源码:675](../../../desktop/audio/src/separated_model.rs#L675) |
| fn stopping_audition_preserves_scene_and_pause_still_admits_a_new_explicit_audition | [源码:693](../../../desktop/audio/src/separated_model.rs#L693) |
| fn pause_before_or_after_callback_never_replays_already_submitted_voice_samples | [源码:712](../../../desktop/audio/src/separated_model.rs#L712) |
| fn resume_rejects_frozen_scene_tokens_and_obsolete_pause_ack_without_invalidating_audition | [源码:734](../../../desktop/audio/src/separated_model.rs#L734) |
| fn repeated_retirement_reuses_slots_but_never_voice_identity_or_pcm_budget | [源码:769](../../../desktop/audio/src/separated_model.rs#L769) |
| fn shared_pcm_has_one_charge_and_wrong_pool_or_invalid_prepared_audio_is_rejected | [源码:798](../../../desktop/audio/src/separated_model.rs#L798) |
| fn close_with_full_handoff_retires_every_lease_on_owner_and_stops_submission | [源码:824](../../../desktop/audio/src/separated_model.rs#L824) |
| fn scheduled_probe | [源码:851](../../../desktop/audio/src/separated_model.rs#L851) |
| fn deterministic_phase_sweep_reports_submission_bounds_without_relaxing_twenty_ms | [源码:891](../../../desktop/audio/src/separated_model.rs#L891) |
| fn admission_stall_beyond_model_envelope_is_a_latency_failure_not_a_pass | [源码:944](../../../desktop/audio/src/separated_model.rs#L944) |

## desktop/audio/src/voice.rs

229 行；SHA256 `cb73370199879dc2067ce1b87eb406793cbb752fe51c0f7c04438c3f6880e2d0`

导入：`use crate::{PreparedSound, VoiceInfo};`, `use std::sync::{ Arc, Weak, atomic::{AtomicBool, AtomicU32, Ordering}, };`

| 定义 | 位置／签名 |
| --- | --- |
| struct MonitorFormat | [源码:9](../../../desktop/audio/src/voice.rs#L9) |
| impl impl MonitorFormat | [源码:13](../../../desktop/audio/src/voice.rs#L13) |
| fn rate | [源码:14](../../../desktop/audio/src/voice.rs#L14) |
| fn new | [源码:17](../../../desktop/audio/src/voice.rs#L17) |
| impl impl PreparedSound | [源码:21](../../../desktop/audio/src/voice.rs#L21) |
| fn add_monitor_frames | [源码:25](../../../desktop/audio/src/voice.rs#L25) |
| fn add_monitor_frames_with_gain | [源码:33](../../../desktop/audio/src/voice.rs#L33) |
| fn monitor_finished | [源码:92](../../../desktop/audio/src/voice.rs#L92) |
| fn monitor_frame | [源码:109](../../../desktop/audio/src/voice.rs#L109) |
| fn monitor_frame_with_gain | [源码:113](../../../desktop/audio/src/voice.rs#L113) |
| struct VoiceRuntimeKey | [源码:167](../../../desktop/audio/src/voice.rs#L167) |
| impl impl VoiceRuntimeKey | [源码:168](../../../desktop/audio/src/voice.rs#L168) |
| fn same_runtime | [源码:169](../../../desktop/audio/src/voice.rs#L169) |
| struct VoiceControl | [源码:177](../../../desktop/audio/src/voice.rs#L177) |
| struct VoiceValidity | [源码:182](../../../desktop/audio/src/voice.rs#L182) |
| impl impl Clone for VoiceValidity | [源码:183](../../../desktop/audio/src/voice.rs#L183) |
| fn clone | [源码:184](../../../desktop/audio/src/voice.rs#L184) |
| impl impl VoiceValidity | [源码:188](../../../desktop/audio/src/voice.rs#L188) |
| fn new | [源码:189](../../../desktop/audio/src/voice.rs#L189) |
| fn revoke | [源码:195](../../../desktop/audio/src/voice.rs#L195) |
| fn set_gain | [源码:198](../../../desktop/audio/src/voice.rs#L198) |
| fn is_valid | [源码:201](../../../desktop/audio/src/voice.rs#L201) |
| fn gain | [源码:204](../../../desktop/audio/src/voice.rs#L204) |
| struct VoiceHandoff | [源码:211](../../../desktop/audio/src/voice.rs#L211) |
| impl impl VoiceHandoff | [源码:216](../../../desktop/audio/src/voice.rs#L216) |
| fn info | [源码:217](../../../desktop/audio/src/voice.rs#L217) |
| fn sound | [源码:220](../../../desktop/audio/src/voice.rs#L220) |
| fn validity | [源码:223](../../../desktop/audio/src/voice.rs#L223) |
| mod tests | [源码:229](../../../desktop/audio/src/voice.rs#L229) |

## desktop/audio/src/voice/tests.rs

325 行；SHA256 `8d0846cda38bc2a67f13458e4d662562bfe686527b085729004d4597bafa8e76`

导入：`use super::*;`, `use crate::{Channel, Origin, PcmBudget, Runtime, RuntimeError, SoundPlan};`, `use skin_editor_domain::{AudioComponentId, AudioMetadata, BlobRef, Gain};`

| 定义 | 位置／签名 |
| --- | --- |
| fn source | [源码:5](../../../desktop/audio/src/voice/tests.rs#L5) |
| fn prepared | [源码:14](../../../desktop/audio/src/voice/tests.rs#L14) |
| fn runtime | [源码:26](../../../desktop/audio/src/voice/tests.rs#L26) |
| fn trigger | [源码:40](../../../desktop/audio/src/voice/tests.rs#L40) |
| fn monitoring_sampler_has_independent_interpolation_channel_and_end_goldens | [源码:51](../../../desktop/audio/src/voice/tests.rs#L51) |
| fn integer_rate_positions_keep_interpolation_bits_zero_sign_channel_mapping_and_end | [源码:74](../../../desktop/audio/src/voice/tests.rs#L74) |
| fn block_addition_matches_scalar_bits_across_formats_offsets_layouts_and_tail | [源码:132](../../../desktop/audio/src/voice/tests.rs#L132) |
| fn invalid_block_layout_does_not_modify_output_or_allocate_pcm | [源码:186](../../../desktop/audio/src/voice/tests.rs#L186) |
| fn external_execution_is_explicit_and_never_silently_mixes_or_queues_future_events | [源码:200](../../../desktop/audio/src/voice/tests.rs#L200) |
| fn switching_an_active_owner_mixer_or_scheduled_queue_is_rejected_without_rewind | [源码:233](../../../desktop/audio/src/voice/tests.rs#L233) |
| fn foreign_and_retried_completion_cannot_retire_reused_numeric_voice_identity | [源码:254](../../../desktop/audio/src/voice/tests.rs#L254) |
| fn runtime_destruction_revokes_export_but_actual_pcm_lease_outlives_it | [源码:279](../../../desktop/audio/src/voice/tests.rs#L279) |
| fn handoff_clock_rejects_stale_updates_and_allows_backward_seek_only_when_frozen | [源码:296](../../../desktop/audio/src/voice/tests.rs#L296) |

## desktop/audio/tests/live_audition.rs

108 行；SHA256 `e2bb6144edac15f1ab342e3c786df6bbe9f99343d1f55e0eb9918e9bca3fcf15`

导入：`use skin_editor_audio::*;`, `use skin_editor_domain::{AudioComponentId, AudioMetadata, BlobRef, Gain};`

| 定义 | 位置／签名 |
| --- | --- |
| fn origin | [源码:4](../../../desktop/audio/tests/live_audition.rs#L4) |
| fn plan | [源码:13](../../../desktop/audio/tests/live_audition.rs#L13) |
| fn ready | [源码:22](../../../desktop/audio/tests/live_audition.rs#L22) |
| fn mix | [源码:30](../../../desktop/audio/tests/live_audition.rs#L30) |
| fn trigger | [源码:36](../../../desktop/audio/tests/live_audition.rs#L36) |
| fn binding_replacement_revokes_active_audition_without_replaying_new_sound | [源码:45](../../../desktop/audio/tests/live_audition.rs#L45) |
| fn gain_drag_zero_restore_and_cancel_keep_the_same_audition_cursor | [源码:56](../../../desktop/audio/tests/live_audition.rs#L56) |
| fn audition_started_at_zero_can_become_audible_without_another_decode_or_voice | [源码:74](../../../desktop/audio/tests/live_audition.rs#L74) |
| fn live_gain_does_not_exempt_history_navigation_or_late_preparation | [源码:91](../../../desktop/audio/tests/live_audition.rs#L91) |

## desktop/audio/tests/pcm.rs

171 行；SHA256 `3522dd8694c8265d81a41eee4364f47ae248c573b99114f5738695efcdb0d2f0`

导入：`use skin_editor_audio::*;`, `use skin_editor_domain::*;`

| 定义 | 位置／签名 |
| --- | --- |
| fn monitoring_conversion_refuses_unreviewed_multichannel_layouts | [源码:4](../../../desktop/audio/tests/pcm.rs#L4) |
| fn plan | [源码:16](../../../desktop/audio/tests/pcm.rs#L16) |
| fn origin | [源码:25](../../../desktop/audio/tests/pcm.rs#L25) |
| fn reservation_precedes_allocation_and_cancel_returns_the_exact_credit | [源码:35](../../../desktop/audio/tests/pcm.rs#L35) |
| fn clones_and_weak_cache_references_count_one_allocation_until_the_last_owner | [源码:51](../../../desktop/audio/tests/pcm.rs#L51) |
| fn default_pool_enforces_real_32_mib_total_and_4_mib_single_sound | [源码:65](../../../desktop/audio/tests/pcm.rs#L65) |
| fn shared_pcm_cannot_mutate_or_create_an_unaccounted_copy_on_write | [源码:80](../../../desktop/audio/tests/pcm.rs#L80) |
| fn runtime_rejects_samples_paid_for_by_an_unrelated_pool | [源码:91](../../../desktop/audio/tests/pcm.rs#L91) |
| fn started_voice_keeps_credit_after_binding_removal_until_actual_completion | [源码:105](../../../desktop/audio/tests/pcm.rs#L105) |
| fn cached_recipe_reuses_full_pool_across_components_and_checks_gain_and_pool_identity | [源码:124](../../../desktop/audio/tests/pcm.rs#L124) |
| fn output_blocks_share_the_voice_budget_and_cannot_be_modified_when_shared | [源码:143](../../../desktop/audio/tests/pcm.rs#L143) |

## desktop/audio/tests/recipe.rs

104 行；SHA256 `4f94d1bbb6d7a2ed3a1e6a852cae47d3243634ad50024bf2a412fa5c7928bc24`

导入：`use skin_editor_audio::*;`, `use skin_editor_domain::*;`

| 定义 | 位置／签名 |
| --- | --- |
| fn plan | [源码:3](../../../desktop/audio/tests/recipe.rs#L3) |
| fn known | [源码:30](../../../desktop/audio/tests/recipe.rs#L30) |
| fn undecodable_unedited_source_is_preserved_exactly_without_a_decoder | [源码:34](../../../desktop/audio/tests/recipe.rs#L34) |
| fn edited_unsupported_source_is_diagnosed_instead_of_exported_with_ignored_edits | [源码:51](../../../desktop/audio/tests/recipe.rs#L51) |
| fn hidden_unknown_audio_requests_valid_encoded_silence_and_never_original_or_omission | [源码:60](../../../desktop/audio/tests/recipe.rs#L60) |
| fn gain_matches_independent_reference_and_does_not_modify_source | [源码:69](../../../desktop/audio/tests/recipe.rs#L69) |
| fn processing_rejects_invalid_pcm_and_excess_capacity_before_allocating | [源码:84](../../../desktop/audio/tests/recipe.rs#L84) |

## desktop/audio/tests/runtime.rs

491 行；SHA256 `0413ac6799929a6c6903fe5e9aef9c17d1da65f7a8504d8acad1bdfb39c225bb`

导入：`use skin_editor_audio::*;`, `use skin_editor_domain::*;`

| 定义 | 位置／签名 |
| --- | --- |
| fn id | [源码:4](../../../desktop/audio/tests/runtime.rs#L4) |
| fn origin | [源码:7](../../../desktop/audio/tests/runtime.rs#L7) |
| fn plan | [源码:16](../../../desktop/audio/tests/runtime.rs#L16) |
| fn runtime | [源码:25](../../../desktop/audio/tests/runtime.rs#L25) |
| fn select | [源码:28](../../../desktop/audio/tests/runtime.rs#L28) |
| fn prepared | [源码:31](../../../desktop/audio/tests/runtime.rs#L31) |
| fn ready | [源码:34](../../../desktop/audio/tests/runtime.rs#L34) |
| fn fire | [源码:39](../../../desktop/audio/tests/runtime.rs#L39) |
| fn output_activity_covers_future_events_and_voices_but_not_the_independent_scene_clock | [源码:44](../../../desktop/audio/tests/runtime.rs#L44) |
| fn controlled_completion_order_rejects_late_success_failure_and_duplicate | [源码:67](../../../desktop/audio/tests/runtime.rs#L67) |
| fn activation_keeps_started_voices_but_drops_queued_old_versions | [源码:106](../../../desktop/audio/tests/runtime.rs#L106) |
| fn failure_preserves_explicit_stale_fallback_and_retry_is_serial_scoped | [源码:130](../../../desktop/audio/tests/runtime.rs#L130) |
| fn unrelated_content_edit_relabels_cached_pcm_without_repreparing_or_starting_audio | [源码:156](../../../desktop/audio/tests/runtime.rs#L156) |
| fn scene_freeze_and_resume_neither_stop_audition_nor_replay_scene_events | [源码:168](../../../desktop/audio/tests/runtime.rs#L168) |
| fn stop_audition_and_seek_have_separate_scope_and_old_controls_fail | [源码:200](../../../desktop/audio/tests/runtime.rs#L200) |
| fn cancel_draft_clears_all_its_revisions_voices_events_candidates_and_pcm_pins | [源码:228](../../../desktop/audio/tests/runtime.rs#L228) |
| fn draft_commit_can_finish_captured_voice_without_relabeling_its_origin | [源码:267](../../../desktop/audio/tests/runtime.rs#L267) |
| fn history_return_to_same_node_still_clears_both_channels_and_old_results | [源码:282](../../../desktop/audio/tests/runtime.rs#L282) |
| fn queue_and_voice_limits_report_refusal_and_scheduled_drops | [源码:312](../../../desktop/audio/tests/runtime.rs#L312) |
| fn device_rebuild_requires_fresh_controls_and_does_not_start_either_channel | [源码:339](../../../desktop/audio/tests/runtime.rs#L339) |
| fn prepared_format_and_recipe_mismatch_cannot_activate_wrong_samples | [源码:364](../../../desktop/audio/tests/runtime.rs#L364) |
| fn hidden_component_is_silent_without_preparation_or_a_voice | [源码:384](../../../desktop/audio/tests/runtime.rs#L384) |
| fn removing_binding_retains_only_started_one_shots_until_their_end | [源码:399](../../../desktop/audio/tests/runtime.rs#L399) |
| fn invalid_projection_does_not_destroy_ready_sound_and_shutdown_releases_every_pin | [源码:419](../../../desktop/audio/tests/runtime.rs#L419) |
| fn library_candidate_has_distinct_identity_and_obeys_stop_and_source_generations | [源码:448](../../../desktop/audio/tests/runtime.rs#L448) |

## desktop/host/src/audio_verification.rs

280 行；SHA256 `a5a8875300972ed936e2c11f2dddd9dba930a939e30592dd217424771d1b43aa`

导入：`use skin_editor_application::{values::*, *};`, `use skin_editor_audio::{Channel, PcmBudget};`, `use skin_editor_infrastructure::{AudioPlayback, BlobStore, NativeOutput, WavDecoder};`, `use std::time::{Duration, Instant};`, `use std::io::Write;`

| 定义 | 位置／签名 |
| --- | --- |
| fn tone | [源码:7](../../../desktop/host/src/audio_verification.rs#L7) |
| fn component | [源码:32](../../../desktop/host/src/audio_verification.rs#L32) |
| fn submit | [源码:35](../../../desktop/host/src/audio_verification.rs#L35) |
| fn phase | [源码:52](../../../desktop/host/src/audio_verification.rs#L52) |
| fn settle | [源码:67](../../../desktop/host/src/audio_verification.rs#L67) |
| fn quiet | [源码:79](../../../desktop/host/src/audio_verification.rs#L79) |
| fn run | [源码:101](../../../desktop/host/src/audio_verification.rs#L101) |

## desktop/host/src/combined_verification.rs

1582 行；SHA256 `9c1434fa4f37ab7a68ac87f8cf743697fbd46b3e941e4153ea32ff4ee3a71e85`

导入：`use crate::audio_verification::submit;`, `use crate::edit_verification::source;`, `use crate::load_verification::{outcome, phase, stop, wait, wav};`, `use crate::{Editor, fresh_session};`, `use eframe::egui;`, `use skin_editor_application::{values::*, *};`, `use skin_editor_audio::{Channel, PcmBudget, Preparation, PrepareFailure};`, `use skin_editor_infrastructure::{ AudioSupplyClient, NativeOutput, NativeStream, OutputControl, OutputStats, SupplyOutcome, write_png, };`, `use std::io::Write;`, `use std::path::PathBuf;`, `use std::sync::mpsc::{self, SyncSender};`, `use std::thread::JoinHandle;`, `use std::time::{Duration, Instant};`, `use recovery::run as recovery_run;`, `use phase::run as phase_run;`, `use phase::run_controlled as phase_controlled_run;`, `use phase::run_recovery as phase_recovery_run;`, `use skin_editor_infrastructure::VoiceSubmission;`, `use skin_editor_infrastructure::VoiceSubmission;`, `use super::*;`, `use skin_editor_infrastructure::VoiceSubmission::*;`, `use skin_editor_infrastructure::output_boundary;`

| 定义 | 位置／签名 |
| --- | --- |
| const SECONDS | [源码:19](../../../desktop/host/src/combined_verification.rs#L19) |
| const TRIGGERS | [源码:20](../../../desktop/host/src/combined_verification.rs#L20) |
| const EDITS | [源码:21](../../../desktop/host/src/combined_verification.rs#L21) |
| const SWITCHES | [源码:22](../../../desktop/host/src/combined_verification.rs#L22) |
| mod recovery | [源码:23](../../../desktop/host/src/combined_verification.rs#L23) |
| mod phase | [源码:25](../../../desktop/host/src/combined_verification.rs#L25) |
| mod cadence | [源码:29](../../../desktop/host/src/combined_verification.rs#L29) |
| fn optional_rows | [源码:30](../../../desktop/host/src/combined_verification.rs#L30) |
| const N | [源码:30](../../../desktop/host/src/combined_verification.rs#L30) |
| impl impl IntoIterator<Item = Option<[u64; N]>>) -> String | [源码:30](../../../desktop/host/src/combined_verification.rs#L30) |
| fn clock_alignment | [源码:36](../../../desktop/host/src/combined_verification.rs#L36) |
| fn scheduling_json | [源码:45](../../../desktop/host/src/combined_verification.rs#L45) |
| fn error | [源码:58](../../../desktop/host/src/combined_verification.rs#L58) |
| impl impl std::fmt::Debug) -> String | [源码:58](../../../desktop/host/src/combined_verification.rs#L58) |
| fn document | [源码:61](../../../desktop/host/src/combined_verification.rs#L61) |
| fn sound | [源码:64](../../../desktop/host/src/combined_verification.rs#L64) |
| fn layer | [源码:67](../../../desktop/host/src/combined_verification.rs#L67) |
| fn ready_all | [源码:70](../../../desktop/host/src/combined_verification.rs#L70) |
| fn play_old | [源码:108](../../../desktop/host/src/combined_verification.rs#L108) |
| fn cleared | [源码:122](../../../desktop/host/src/combined_verification.rs#L122) |
| struct Probe | [源码:141](../../../desktop/host/src/combined_verification.rs#L141) |
| struct Observer | [源码:151](../../../desktop/host/src/combined_verification.rs#L151) |
| struct ProbeObservations | [源码:155](../../../desktop/host/src/combined_verification.rs#L155) |
| struct SubmissionBracket | [源码:169](../../../desktop/host/src/combined_verification.rs#L169) |
| impl impl SubmissionBracket | [源码:173](../../../desktop/host/src/combined_verification.rs#L173) |
| fn completion | [源码:174](../../../desktop/host/src/combined_verification.rs#L174) |
| fn observe | [源码:190](../../../desktop/host/src/combined_verification.rs#L190) |
| impl impl Observer | [源码:204](../../../desktop/host/src/combined_verification.rs#L204) |
| fn new | [源码:205](../../../desktop/host/src/combined_verification.rs#L205) |
| fn with_capacity | [源码:208](../../../desktop/host/src/combined_verification.rs#L208) |
| fn finish | [源码:337](../../../desktop/host/src/combined_verification.rs#L337) |
| impl impl Drop for Observer | [源码:346](../../../desktop/host/src/combined_verification.rs#L346) |
| fn drop | [源码:347](../../../desktop/host/src/combined_verification.rs#L347) |
| fn fixture | [源码:355](../../../desktop/host/src/combined_verification.rs#L355) |
| fn visual_fixture | [源码:358](../../../desktop/host/src/combined_verification.rs#L358) |
| fn reopen_fixture | [源码:361](../../../desktop/host/src/combined_verification.rs#L361) |
| fn build_fixture | [源码:364](../../../desktop/host/src/combined_verification.rs#L364) |
| struct Combined | [源码:482](../../../desktop/host/src/combined_verification.rs#L482) |
| impl impl Combined | [源码:544](../../../desktop/host/src/combined_verification.rs#L544) |
| fn new | [源码:545](../../../desktop/host/src/combined_verification.rs#L545) |
| fn create | [源码:563](../../../desktop/host/src/combined_verification.rs#L563) |
| fn tick | [源码:670](../../../desktop/host/src/combined_verification.rs#L670) |
| fn finish | [源码:957](../../../desktop/host/src/combined_verification.rs#L957) |
| fn report | [源码:1111](../../../desktop/host/src/combined_verification.rs#L1111) |
| impl impl Combined | [源码:1241](../../../desktop/host/src/combined_verification.rs#L1241) |
| fn ns | [源码:1242](../../../desktop/host/src/combined_verification.rs#L1242) |
| fn logic_inner | [源码:1245](../../../desktop/host/src/combined_verification.rs#L1245) |
| impl impl eframe::App for Combined | [源码:1299](../../../desktop/host/src/combined_verification.rs#L1299) |
| fn logic | [源码:1300](../../../desktop/host/src/combined_verification.rs#L1300) |
| fn ui | [源码:1322](../../../desktop/host/src/combined_verification.rs#L1322) |
| fn on_exit | [源码:1370](../../../desktop/host/src/combined_verification.rs#L1370) |
| fn run | [源码:1411](../../../desktop/host/src/combined_verification.rs#L1411) |
| fn memory_run | [源码:1414](../../../desktop/host/src/combined_verification.rs#L1414) |
| fn control_run | [源码:1417](../../../desktop/host/src/combined_verification.rs#L1417) |
| fn memory_control_run | [源码:1420](../../../desktop/host/src/combined_verification.rs#L1420) |
| fn cold_run | [源码:1423](../../../desktop/host/src/combined_verification.rs#L1423) |
| fn staging_run | [源码:1426](../../../desktop/host/src/combined_verification.rs#L1426) |
| fn run_mode | [源码:1429](../../../desktop/host/src/combined_verification.rs#L1429) |
| mod tests | [源码:1475](../../../desktop/host/src/combined_verification.rs#L1475) |
| fn unknown_marker_cannot_advance_negative_or_late_telemetry_move_positive | [源码:1478](../../../desktop/host/src/combined_verification.rs#L1478) |
| fn invalid_probe_capacity_is_rejected_without_thread_or_device | [源码:1496](../../../desktop/host/src/combined_verification.rs#L1496) |
| fn submission_bounds_cover_atomic_read_races_and_delayed_observer | [源码:1506](../../../desktop/host/src/combined_verification.rs#L1506) |
| fn native_probe_close_and_overflow_are_explicit_without_sleep_or_device | [源码:1524](../../../desktop/host/src/combined_verification.rs#L1524) |
| fn standard_schedule_covers_300_seconds_without_unbounded_catch_up | [源码:1570](../../../desktop/host/src/combined_verification.rs#L1570) |

## desktop/host/src/combined_verification/cadence.rs

104 行；SHA256 `075198b815ada0a2d60624fb2519a8ede00563563a58db41adc2c9dfa8666abc`

导入：`use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| struct Cadence | [源码:2](../../../desktop/host/src/combined_verification/cadence.rs#L2) |
| impl impl Default for Cadence | [源码:9](../../../desktop/host/src/combined_verification/cadence.rs#L9) |
| fn default | [源码:10](../../../desktop/host/src/combined_verification/cadence.rs#L10) |
| impl impl Cadence | [源码:19](../../../desktop/host/src/combined_verification/cadence.rs#L19) |
| fn begin | [源码:20](../../../desktop/host/src/combined_verification/cadence.rs#L20) |
| fn tick_end | [源码:36](../../../desktop/host/src/combined_verification/cadence.rs#L36) |
| fn end | [源码:42](../../../desktop/host/src/combined_verification/cadence.rs#L42) |
| fn repaint | [源码:48](../../../desktop/host/src/combined_verification/cadence.rs#L48) |
| fn ui | [源码:51](../../../desktop/host/src/combined_verification/cadence.rs#L51) |
| mod tests | [源码:56](../../../desktop/host/src/combined_verification/cadence.rs#L56) |
| fn late_wakeup_is_distinct_from_work_and_survives_early_failure | [源码:59](../../../desktop/host/src/combined_verification/cadence.rs#L59) |
| fn on_time_and_unrequested_calls_do_not_invent_lateness | [源码:95](../../../desktop/host/src/combined_verification/cadence.rs#L95) |

## desktop/host/src/combined_verification/phase.rs

396 行；SHA256 `bf4bb995fd29529c9e2783e152a1cd6de0926b0bdbb5e6b5aa1ca68a06ff8417`

导入：`use super::*;`, `use skin_editor_infrastructure::{BlobStore, WavDecoder};`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| const BINS | [源码:4](../../../desktop/host/src/combined_verification/phase.rs#L4) |
| const REPEATS | [源码:5](../../../desktop/host/src/combined_verification/phase.rs#L5) |
| const COUNT | [源码:6](../../../desktop/host/src/combined_verification/phase.rs#L6) |
| const REFERENCE_ATTEMPTS | [源码:7](../../../desktop/host/src/combined_verification/phase.rs#L7) |
| enum RecoveryProbeMode | [源码:9](../../../desktop/host/src/combined_verification/phase.rs#L9) |
| fn reference | [源码:15](../../../desktop/host/src/combined_verification/phase.rs#L15) |
| impl impl FnMut() -> Option<[u64; 4]>, mut pause: impl FnMut() -> Result<(), String>, ) -> Result<([u64; 4], usize), String> | [源码:16](../../../desktop/host/src/combined_verification/phase.rs#L16) |
| impl impl FnMut() -> Result<(), String>, ) -> Result<([u64; 4], usize), String> | [源码:17](../../../desktop/host/src/combined_verification/phase.rs#L17) |
| fn callback_reference | [源码:29](../../../desktop/host/src/combined_verification/phase.rs#L29) |
| fn future_phase | [源码:42](../../../desktop/host/src/combined_verification/phase.rs#L42) |
| fn run | [源码:54](../../../desktop/host/src/combined_verification/phase.rs#L54) |
| fn run_recovery | [源码:61](../../../desktop/host/src/combined_verification/phase.rs#L61) |
| fn run_controlled | [源码:68](../../../desktop/host/src/combined_verification/phase.rs#L68) |
| fn run_with_policy | [源码:79](../../../desktop/host/src/combined_verification/phase.rs#L79) |
| mod tests | [源码:333](../../../desktop/host/src/combined_verification/phase.rs#L333) |
| fn incomplete_reference_is_retried_and_returns_the_actual_later_snapshot | [源码:336](../../../desktop/host/src/combined_verification/phase.rs#L336) |
| fn reference_retry_is_bounded_and_closed_output_stops_immediately | [源码:354](../../../desktop/host/src/combined_verification/phase.rs#L354) |
| fn phase_deadlines_are_future_and_cover_all_bins_without_overflow | [源码:378](../../../desktop/host/src/combined_verification/phase.rs#L378) |

## desktop/host/src/combined_verification/recovery.rs

277 行；SHA256 `0e72b10ee5560b2c11154af705af42637841ed1d128f718ba1c853cd474161a9`

导入：`use super::*;`, `use eframe::App;`, `use eframe::egui_wgpu::RenderState;`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| const CYCLES | [源码:7](../../../desktop/host/src/combined_verification/recovery.rs#L7) |
| const SETTLE | [源码:8](../../../desktop/host/src/combined_verification/recovery.rs#L8) |
| const STABLE | [源码:9](../../../desktop/host/src/combined_verification/recovery.rs#L9) |
| enum Stage | [源码:12](../../../desktop/host/src/combined_verification/recovery.rs#L12) |
| impl impl Stage | [源码:20](../../../desktop/host/src/combined_verification/recovery.rs#L20) |
| fn stable | [源码:21](../../../desktop/host/src/combined_verification/recovery.rs#L21) |
| struct Recovery | [源码:25](../../../desktop/host/src/combined_verification/recovery.rs#L25) |
| impl impl Recovery | [源码:38](../../../desktop/host/src/combined_verification/recovery.rs#L38) |
| fn open | [源码:39](../../../desktop/host/src/combined_verification/recovery.rs#L39) |
| fn begin | [源码:65](../../../desktop/host/src/combined_verification/recovery.rs#L65) |
| fn snapshot | [源码:70](../../../desktop/host/src/combined_verification/recovery.rs#L70) |
| fn close | [源码:80](../../../desktop/host/src/combined_verification/recovery.rs#L80) |
| fn tick | [源码:106](../../../desktop/host/src/combined_verification/recovery.rs#L106) |
| impl impl eframe::App for Recovery | [源码:172](../../../desktop/host/src/combined_verification/recovery.rs#L172) |
| fn logic | [源码:173](../../../desktop/host/src/combined_verification/recovery.rs#L173) |
| fn ui | [源码:183](../../../desktop/host/src/combined_verification/recovery.rs#L183) |
| fn on_exit | [源码:202](../../../desktop/host/src/combined_verification/recovery.rs#L202) |
| fn run | [源码:216](../../../desktop/host/src/combined_verification/recovery.rs#L216) |
| mod tests | [源码:260](../../../desktop/host/src/combined_verification/recovery.rs#L260) |
| fn stable_observation_excludes_settling_and_quick_cycle_gaps | [源码:263](../../../desktop/host/src/combined_verification/recovery.rs#L263) |

## desktop/host/src/edit_verification.rs

495 行；SHA256 `56f235f658eba65e001d34ec64910fda5a7291f12ba68c7a2146b39994988535`

导入：`use crate::audio_verification::{component, submit};`, `use crate::load_verification::{consume, outcome, phase, stop, wait, wav};`, `use skin_editor_application::{values::*, *};`, `use skin_editor_audio::{Channel, Origin, PcmBudget, Preparation};`, `use skin_editor_infrastructure::{ AdmissionState, AudioSupplyClient, BlobStore, NativeOutput, OutputControl, SupplyOutcome, };`, `use std::io::Write;`, `use std::time::Instant;`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| fn error | [源码:12](../../../desktop/host/src/edit_verification.rs#L12) |
| impl impl std::fmt::Debug) -> String | [源码:12](../../../desktop/host/src/edit_verification.rs#L12) |
| fn draft | [源码:15](../../../desktop/host/src/edit_verification.rs#L15) |
| fn ds | [源码:27](../../../desktop/host/src/edit_verification.rs#L27) |
| struct Capture | [源码:35](../../../desktop/host/src/edit_verification.rs#L35) |
| impl impl SkinAudioPort for Capture | [源码:36](../../../desktop/host/src/edit_verification.rs#L36) |
| fn synchronize | [源码:37](../../../desktop/host/src/edit_verification.rs#L37) |
| fn source | [源码:53](../../../desktop/host/src/edit_verification.rs#L53) |
| fn ready | [源码:58](../../../desktop/host/src/edit_verification.rs#L58) |
| fn prepare | [源码:92](../../../desktop/host/src/edit_verification.rs#L92) |
| fn trigger | [源码:140](../../../desktop/host/src/edit_verification.rs#L140) |
| fn settled_pcm | [源码:163](../../../desktop/host/src/edit_verification.rs#L163) |
| fn health | [源码:172](../../../desktop/host/src/edit_verification.rs#L172) |
| fn restored | [源码:189](../../../desktop/host/src/edit_verification.rs#L189) |
| fn run | [源码:195](../../../desktop/host/src/edit_verification.rs#L195) |
| mod tests | [源码:450](../../../desktop/host/src/edit_verification.rs#L450) |
| fn history_restoration_checks_content_and_head_but_requires_a_new_revision | [源码:453](../../../desktop/host/src/edit_verification.rs#L453) |

## desktop/host/src/editor_audio.rs

1360 行；SHA256 `b5ae03ceec5249a1d8150ef87308d1e65a77fab2aecde479c7f531d1f887ce93`

导入：`use skin_editor_application::{values::*, *};`, `use skin_editor_audio::{BackgroundTrack, Channel, Origin, PcmBudget, Preparation, TriggerReceipt};`, `use skin_editor_infrastructure::{ AudioSupplyClient, BlobStore, NativeOutput, NativeStream, SupplyError, SupplyOutcome, separated_native::{EndpointReturnError, NativeExecutor, PreparedDevice}, };`, `use super::*;`, `use super::*;`, `use skin_editor_infrastructure::{OutputConsumer, WavDecoder, output_boundary};`, `use skin_editor_ui::EditorUi;`, `use std::{cell::RefCell, rc::Rc, time::Duration};`

| 定义 | 位置／签名 |
| --- | --- |
| mod combined | [源码:8](../../../desktop/host/src/editor_audio.rs#L8) |
| mod cycles | [源码:9](../../../desktop/host/src/editor_audio.rs#L9) |
| mod faults | [源码:10](../../../desktop/host/src/editor_audio.rs#L10) |
| mod gui_verification | [源码:11](../../../desktop/host/src/editor_audio.rs#L11) |
| mod stability | [源码:12](../../../desktop/host/src/editor_audio.rs#L12) |
| mod window_recovery | [源码:13](../../../desktop/host/src/editor_audio.rs#L13) |
| mod product_tests | [源码:15](../../../desktop/host/src/editor_audio.rs#L15) |
| fn ordinary_audio_is_disabled_and_lazy_but_selects_the_platform_route | [源码:18](../../../desktop/host/src/editor_audio.rs#L18) |
| struct EditorAudio | [源码:28](../../../desktop/host/src/editor_audio.rs#L28) |
| struct Recovery | [源码:57](../../../desktop/host/src/editor_audio.rs#L57) |
| fn preparation | [源码:63](../../../desktop/host/src/editor_audio.rs#L63) |
| fn source | [源码:70](../../../desktop/host/src/editor_audio.rs#L70) |
| impl impl EditorAudio | [源码:88](../../../desktop/host/src/editor_audio.rs#L88) |
| fn receive | [源码:89](../../../desktop/host/src/editor_audio.rs#L89) |
| fn new | [源码:123](../../../desktop/host/src/editor_audio.rs#L123) |
| fn candidate | [源码:153](../../../desktop/host/src/editor_audio.rs#L153) |
| fn take_reference | [源码:158](../../../desktop/host/src/editor_audio.rs#L158) |
| fn enable_candidate | [源码:183](../../../desktop/host/src/editor_audio.rs#L183) |
| fn restore_background | [源码:230](../../../desktop/host/src/editor_audio.rs#L230) |
| fn handle_native_fault | [源码:259](../../../desktop/host/src/editor_audio.rs#L259) |
| fn close | [源码:285](../../../desktop/host/src/editor_audio.rs#L285) |
| fn transport | [源码:349](../../../desktop/host/src/editor_audio.rs#L349) |
| impl impl Drop for EditorAudio | [源码:370](../../../desktop/host/src/editor_audio.rs#L370) |
| fn drop | [源码:371](../../../desktop/host/src/editor_audio.rs#L371) |
| impl impl SkinAudioPort for EditorAudio | [源码:375](../../../desktop/host/src/editor_audio.rs#L375) |
| fn retained_drafts | [源码:376](../../../desktop/host/src/editor_audio.rs#L376) |
| fn synchronize | [源码:381](../../../desktop/host/src/editor_audio.rs#L381) |
| impl impl AudioEditorPort for EditorAudio | [源码:396](../../../desktop/host/src/editor_audio.rs#L396) |
| fn audition_asset | [源码:397](../../../desktop/host/src/editor_audio.rs#L397) |
| fn request | [源码:414](../../../desktop/host/src/editor_audio.rs#L414) |
| fn poll | [源码:521](../../../desktop/host/src/editor_audio.rs#L521) |
| mod tests | [源码:623](../../../desktop/host/src/editor_audio.rs#L623) |
| fn failed_cleanup_cannot_be_cleared_by_disable_then_reenable | [源码:626](../../../desktop/host/src/editor_audio.rs#L626) |
| struct Shared | [源码:644](../../../desktop/host/src/editor_audio.rs#L644) |
| fn transient_transport_busy_keeps_resume_goal_retryable_without_poisoning_editor_error | [源码:646](../../../desktop/host/src/editor_audio.rs#L646) |
| fn drained_receipts_keep_bounded_request_identity_for_gui_observation | [源码:660](../../../desktop/host/src/editor_audio.rs#L660) |
| fn candidate_without_song_is_explicit_eof_and_only_reserves_one_silent_frame | [源码:696](../../../desktop/host/src/editor_audio.rs#L696) |
| impl impl SkinAudioPort for Shared | [源码:707](../../../desktop/host/src/editor_audio.rs#L707) |
| fn retained_drafts | [源码:708](../../../desktop/host/src/editor_audio.rs#L708) |
| fn synchronize | [源码:711](../../../desktop/host/src/editor_audio.rs#L711) |
| impl impl AudioEditorPort for Shared | [源码:720](../../../desktop/host/src/editor_audio.rs#L720) |
| fn audition_asset | [源码:721](../../../desktop/host/src/editor_audio.rs#L721) |
| fn request | [源码:724](../../../desktop/host/src/editor_audio.rs#L724) |
| fn poll | [源码:727](../../../desktop/host/src/editor_audio.rs#L727) |
| fn barrier | [源码:731](../../../desktop/host/src/editor_audio.rs#L731) |
| fn settle | [源码:751](../../../desktop/host/src/editor_audio.rs#L751) |
| fn sound | [源码:772](../../../desktop/host/src/editor_audio.rs#L772) |
| fn flush | [源码:779](../../../desktop/host/src/editor_audio.rs#L779) |
| fn opaque_preparation_failure_is_visible_and_retry_never_autoplays_or_changes_history | [源码:783](../../../desktop/host/src/editor_audio.rs#L783) |
| fn ui_port_real_wav_hot_replace_cancel_and_history_invalidate_without_ui_frame | [源码:855](../../../desktop/host/src/editor_audio.rs#L855) |
| fn library_audition_without_component_is_frozen_readonly_and_cleaned_outside_ui_frames | [源码:978](../../../desktop/host/src/editor_audio.rs#L978) |
| struct CandidateRig | [源码:1060](../../../desktop/host/src/editor_audio.rs#L1060) |
| impl impl CandidateRig | [源码:1068](../../../desktop/host/src/editor_audio.rs#L1068) |
| fn new | [源码:1069](../../../desktop/host/src/editor_audio.rs#L1069) |
| fn drive | [源码:1116](../../../desktop/host/src/editor_audio.rs#L1116) |
| impl impl Fn(&Self) -> bool) | [源码:1116](../../../desktop/host/src/editor_audio.rs#L1116) |
| fn ready | [源码:1149](../../../desktop/host/src/editor_audio.rs#L1149) |
| fn candidate_gui_observes_preparation_freeze_and_independent_audition_without_native_device | [源码:1159](../../../desktop/host/src/editor_audio.rs#L1159) |
| fn candidate_gui_cancel_and_navigation_reject_old_samples_and_label_late_observation | [源码:1202](../../../desktop/host/src/editor_audio.rs#L1202) |
| fn candidate_gui_live_gain_cancel_and_binding_replacement_use_the_existing_worker | [源码:1279](../../../desktop/host/src/editor_audio.rs#L1279) |

## desktop/host/src/editor_audio/combined.rs

758 行；SHA256 `2623d072283d0b0d108778cd65015c6f2c21f87bdeb2cb38b12437b7485d44ce`

导入：`use super::{ gui_verification::{Observations, Shared}, *, };`, `use crate::{Editor, fresh_session, load_verification::phase};`, `use eframe::egui;`, `use skin_editor_infrastructure::{ background_stream::StreamObservation, separated_native::NativeStats, };`, `use skin_editor_ui::Presenter;`, `use std::{ cell::RefCell, path::PathBuf, rc::Rc, sync::atomic::Ordering, time::{Duration, Instant}, };`, `use plan::{COUNTS, Event, Plan, SECONDS};`, `use probes::Probes;`

| 定义 | 位置／签名 |
| --- | --- |
| mod plan | [源码:20](../../../desktop/host/src/editor_audio/combined.rs#L20) |
| mod probes | [源码:21](../../../desktop/host/src/editor_audio/combined.rs#L21) |
| mod report | [源码:22](../../../desktop/host/src/editor_audio/combined.rs#L22) |
| fn error | [源码:25](../../../desktop/host/src/editor_audio/combined.rs#L25) |
| impl impl std::fmt::Debug) -> String | [源码:25](../../../desktop/host/src/editor_audio/combined.rs#L25) |
| fn document | [源码:28](../../../desktop/host/src/editor_audio/combined.rs#L28) |
| fn layer | [源码:31](../../../desktop/host/src/editor_audio/combined.rs#L31) |
| fn sound | [源码:34](../../../desktop/host/src/editor_audio/combined.rs#L34) |
| struct VisualPending | [源码:37](../../../desktop/host/src/editor_audio/combined.rs#L37) |
| struct Run | [源码:43](../../../desktop/host/src/editor_audio/combined.rs#L43) |
| impl impl Run | [源码:88](../../../desktop/host/src/editor_audio/combined.rs#L88) |
| fn new | [源码:89](../../../desktop/host/src/editor_audio/combined.rs#L89) |
| fn advance | [源码:178](../../../desktop/host/src/editor_audio/combined.rs#L178) |
| fn usage | [源码:181](../../../desktop/host/src/editor_audio/combined.rs#L181) |
| fn action | [源码:190](../../../desktop/host/src/editor_audio/combined.rs#L190) |
| fn draft | [源码:200](../../../desktop/host/src/editor_audio/combined.rs#L200) |
| fn draft_state | [源码:206](../../../desktop/host/src/editor_audio/combined.rs#L206) |
| fn ready | [源码:215](../../../desktop/host/src/editor_audio/combined.rs#L215) |
| fn background | [源码:240](../../../desktop/host/src/editor_audio/combined.rs#L240) |
| fn trigger | [源码:248](../../../desktop/host/src/editor_audio/combined.rs#L248) |
| fn collect | [源码:278](../../../desktop/host/src/editor_audio/combined.rs#L278) |
| fn visual | [源码:300](../../../desktop/host/src/editor_audio/combined.rs#L300) |
| fn observe_visual | [源码:312](../../../desktop/host/src/editor_audio/combined.rs#L312) |
| fn event | [源码:331](../../../desktop/host/src/editor_audio/combined.rs#L331) |
| fn step | [源码:384](../../../desktop/host/src/editor_audio/combined.rs#L384) |
| impl impl eframe::App for Run | [源码:637](../../../desktop/host/src/editor_audio/combined.rs#L637) |
| fn raw_input_hook | [源码:638](../../../desktop/host/src/editor_audio/combined.rs#L638) |
| fn logic | [源码:651](../../../desktop/host/src/editor_audio/combined.rs#L651) |
| fn ui | [源码:666](../../../desktop/host/src/editor_audio/combined.rs#L666) |
| fn on_exit | [源码:719](../../../desktop/host/src/editor_audio/combined.rs#L719) |
| fn run | [源码:724](../../../desktop/host/src/editor_audio/combined.rs#L724) |
| fn run_policy | [源码:727](../../../desktop/host/src/editor_audio/combined.rs#L727) |

## desktop/host/src/editor_audio/combined/plan.rs

159 行；SHA256 `4d0a042c05d091400a23aef673d8d7464cce60eab5eb211a6579cc6e07ba7a33`

导入：`use super::*;`, `use std::time::Duration;`

| 定义 | 位置／签名 |
| --- | --- |
| const SECONDS | [源码:2](../../../desktop/host/src/editor_audio/combined/plan.rs#L2) |
| const COUNTS | [源码:3](../../../desktop/host/src/editor_audio/combined/plan.rs#L3) |
| struct StageClock | [源码:5](../../../desktop/host/src/editor_audio/combined/plan.rs#L5) |
| impl impl StageClock | [源码:9](../../../desktop/host/src/editor_audio/combined/plan.rs#L9) |
| fn enter | [源码:10](../../../desktop/host/src/editor_audio/combined/plan.rs#L10) |
| fn age | [源码:14](../../../desktop/host/src/editor_audio/combined/plan.rs#L14) |
| static str | [源码:14](../../../desktop/host/src/editor_audio/combined/plan.rs#L14) |
| fn check | [源码:17](../../../desktop/host/src/editor_audio/combined/plan.rs#L17) |
| static str | [源码:17](../../../desktop/host/src/editor_audio/combined/plan.rs#L17) |
| enum Event | [源码:29](../../../desktop/host/src/editor_audio/combined/plan.rs#L29) |
| impl impl Event | [源码:35](../../../desktop/host/src/editor_audio/combined/plan.rs#L35) |
| fn index | [源码:36](../../../desktop/host/src/editor_audio/combined/plan.rs#L36) |
| struct Plan | [源码:46](../../../desktop/host/src/editor_audio/combined/plan.rs#L46) |
| impl impl Plan | [源码:50](../../../desktop/host/src/editor_audio/combined/plan.rs#L50) |
| fn due | [源码:51](../../../desktop/host/src/editor_audio/combined/plan.rs#L51) |
| fn next | [源码:62](../../../desktop/host/src/editor_audio/combined/plan.rs#L62) |
| fn take | [源码:68](../../../desktop/host/src/editor_audio/combined/plan.rs#L68) |
| static str | [源码:68](../../../desktop/host/src/editor_audio/combined/plan.rs#L68) |
| mod tests | [源码:86](../../../desktop/host/src/editor_audio/combined/plan.rs#L86) |
| fn transition_after_a_full_playback_gets_a_fresh_control_deadline | [源码:90](../../../desktop/host/src/editor_audio/combined/plan.rs#L90) |
| fn stale_time_is_rejected_and_each_control_transition_resets_watchdog | [源码:101](../../../desktop/host/src/editor_audio/combined/plan.rs#L101) |
| fn cleanup_deadline_is_checked_before_a_ready_natural_end_can_pass | [源码:112](../../../desktop/host/src/editor_audio/combined/plan.rs#L112) |
| fn full_three_hundred_seconds_has_exact_counts_and_no_phase_collisions | [源码:122](../../../desktop/host/src/editor_audio/combined/plan.rs#L122) |
| fn stalled_host_fails_without_consuming_an_event_or_bursting | [源码:144](../../../desktop/host/src/editor_audio/combined/plan.rs#L144) |
| fn bounded_jitter_preserves_all_operations_and_records_lateness | [源码:151](../../../desktop/host/src/editor_audio/combined/plan.rs#L151) |

## desktop/host/src/editor_audio/combined/probes.rs

191 行；SHA256 `3b3b1d23aeac551c67ce4ff7b4cf227b342e7083a3da030e2184cb9bda01a089`

导入：`use skin_editor_audio::{Channel, Origin, TriggerReceipt};`, `use skin_editor_infrastructure::separated_native::FirstCopy;`, `use std::time::{Duration, Instant};`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| const LIMIT | [源码:5](../../../desktop/host/src/editor_audio/combined/probes.rs#L5) |
| struct Pending | [源码:6](../../../desktop/host/src/editor_audio/combined/probes.rs#L6) |
| struct Complete | [源码:14](../../../desktop/host/src/editor_audio/combined/probes.rs#L14) |
| struct Probes | [源码:23](../../../desktop/host/src/editor_audio/combined/probes.rs#L23) |
| impl impl Default for Probes | [源码:27](../../../desktop/host/src/editor_audio/combined/probes.rs#L27) |
| fn default | [源码:28](../../../desktop/host/src/editor_audio/combined/probes.rs#L28) |
| impl impl Probes | [源码:35](../../../desktop/host/src/editor_audio/combined/probes.rs#L35) |
| fn begin | [源码:36](../../../desktop/host/src/editor_audio/combined/probes.rs#L36) |
| fn receipt | [源码:61](../../../desktop/host/src/editor_audio/combined/probes.rs#L61) |
| fn observe | [源码:73](../../../desktop/host/src/editor_audio/combined/probes.rs#L73) |
| fn empty | [源码:107](../../../desktop/host/src/editor_audio/combined/probes.rs#L107) |
| mod tests | [源码:112](../../../desktop/host/src/editor_audio/combined/probes.rs#L112) |
| fn origin | [源码:114](../../../desktop/host/src/editor_audio/combined/probes.rs#L114) |
| fn receipt | [源码:123](../../../desktop/host/src/editor_audio/combined/probes.rs#L123) |
| fn first | [源码:132](../../../desktop/host/src/editor_audio/combined/probes.rs#L132) |
| fn exact_request_and_channel_match_even_if_copy_precedes_receipt | [源码:143](../../../desktop/host/src/editor_audio/combined/probes.rs#L143) |
| fn wrong_origin_duplicate_and_unrelated_newer_voice_are_not_completion | [源码:160](../../../desktop/host/src/editor_audio/combined/probes.rs#L160) |
| fn admission_is_bounded_before_any_receipt_and_cancelled_source_is_rejected | [源码:180](../../../desktop/host/src/editor_audio/combined/probes.rs#L180) |

## desktop/host/src/editor_audio/combined/report.rs

112 行；SHA256 `66b985dd048174117853d7ba32b921ce8cbfff23a5fe93b8fbda758349faef2c`

导入：`use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| impl impl Run | [源码:2](../../../desktop/host/src/editor_audio/combined/report.rs#L2) |
| fn report | [源码:3](../../../desktop/host/src/editor_audio/combined/report.rs#L3) |

## desktop/host/src/editor_audio/cycles.rs

825 行；SHA256 `cec41998b1ebfd17b92d94f2c0aefd8861b26c95c019037a8fd4ca8b883b329b`

导入：`use super::{ combined::probes::Probes, gui_verification::{Observations, Shared}, *, };`, `use crate::{Editor, fresh_session, load_verification::phase};`, `use eframe::egui;`, `use skin_editor_infrastructure::{ ByteObservation, background_stream::{BufferedBackground, StreamObservation}, separated_native::NativeStats, };`, `use skin_editor_ui::Presenter;`, `use std::{ cell::RefCell, fs::File, path::PathBuf, rc::Rc, sync::atomic::Ordering, time::{Duration, Instant}, };`, `use plan::Mode;`, `use std::io::Write;`

| 定义 | 位置／签名 |
| --- | --- |
| mod plan | [源码:23](../../../desktop/host/src/editor_audio/cycles.rs#L23) |
| mod report | [源码:24](../../../desktop/host/src/editor_audio/cycles.rs#L24) |
| fn error | [源码:26](../../../desktop/host/src/editor_audio/cycles.rs#L26) |
| impl impl std::fmt::Debug) -> String | [源码:26](../../../desktop/host/src/editor_audio/cycles.rs#L26) |
| fn sound | [源码:29](../../../desktop/host/src/editor_audio/cycles.rs#L29) |
| struct Counters | [源码:32](../../../desktop/host/src/editor_audio/cycles.rs#L32) |
| impl impl Counters | [源码:38](../../../desktop/host/src/editor_audio/cycles.rs#L38) |
| fn bytes | [源码:39](../../../desktop/host/src/editor_audio/cycles.rs#L39) |
| struct Round | [源码:47](../../../desktop/host/src/editor_audio/cycles.rs#L47) |
| impl impl Round | [源码:81](../../../desktop/host/src/editor_audio/cycles.rs#L81) |
| fn new | [源码:82](../../../desktop/host/src/editor_audio/cycles.rs#L82) |
| fn usage | [源码:202](../../../desktop/host/src/editor_audio/cycles.rs#L202) |
| fn observe | [源码:213](../../../desktop/host/src/editor_audio/cycles.rs#L213) |
| fn action | [源码:233](../../../desktop/host/src/editor_audio/cycles.rs#L233) |
| fn ready | [源码:243](../../../desktop/host/src/editor_audio/cycles.rs#L243) |
| fn draft_state | [源码:268](../../../desktop/host/src/editor_audio/cycles.rs#L268) |
| fn draft | [源码:277](../../../desktop/host/src/editor_audio/cycles.rs#L277) |
| fn image | [源码:283](../../../desktop/host/src/editor_audio/cycles.rs#L283) |
| fn trigger | [源码:299](../../../desktop/host/src/editor_audio/cycles.rs#L299) |
| fn collect | [源码:329](../../../desktop/host/src/editor_audio/cycles.rs#L329) |
| fn background | [源码:350](../../../desktop/host/src/editor_audio/cycles.rs#L350) |
| fn begin_close | [源码:358](../../../desktop/host/src/editor_audio/cycles.rs#L358) |
| struct Run | [源码:404](../../../desktop/host/src/editor_audio/cycles.rs#L404) |
| impl impl Run | [源码:425](../../../desktop/host/src/editor_audio/cycles.rs#L425) |
| fn new | [源码:426](../../../desktop/host/src/editor_audio/cycles.rs#L426) |
| fn publish_phase | [源码:461](../../../desktop/host/src/editor_audio/cycles.rs#L461) |
| fn advance | [源码:469](../../../desktop/host/src/editor_audio/cycles.rs#L469) |
| fn step | [源码:486](../../../desktop/host/src/editor_audio/cycles.rs#L486) |
| fn delay | [源码:684](../../../desktop/host/src/editor_audio/cycles.rs#L684) |
| impl impl eframe::App for Run | [源码:697](../../../desktop/host/src/editor_audio/cycles.rs#L697) |
| fn raw_input_hook | [源码:698](../../../desktop/host/src/editor_audio/cycles.rs#L698) |
| fn logic | [源码:709](../../../desktop/host/src/editor_audio/cycles.rs#L709) |
| fn ui | [源码:722](../../../desktop/host/src/editor_audio/cycles.rs#L722) |
| fn on_exit | [源码:785](../../../desktop/host/src/editor_audio/cycles.rs#L785) |
| fn run | [源码:792](../../../desktop/host/src/editor_audio/cycles.rs#L792) |
| fn run_policy | [源码:795](../../../desktop/host/src/editor_audio/cycles.rs#L795) |

## desktop/host/src/editor_audio/cycles/plan.rs

88 行；SHA256 `b62bad83b6acf59ab69dfacca4bcba931fc26c4ef90fc20fa7cd83e1352fb0eb`

导入：`use std::time::Duration;`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| struct Mode | [源码:3](../../../desktop/host/src/editor_audio/cycles/plan.rs#L3) |
| impl impl Mode | [源码:6](../../../desktop/host/src/editor_audio/cycles/plan.rs#L6) |
| fn rounds | [源码:7](../../../desktop/host/src/editor_audio/cycles/plan.rs#L7) |
| fn counts | [源码:10](../../../desktop/host/src/editor_audio/cycles/plan.rs#L10) |
| fn play | [源码:13](../../../desktop/host/src/editor_audio/cycles/plan.rs#L13) |
| fn settle | [源码:16](../../../desktop/host/src/editor_audio/cycles/plan.rs#L16) |
| fn baseline | [源码:19](../../../desktop/host/src/editor_audio/cycles/plan.rs#L19) |
| fn final_settle | [源码:22](../../../desktop/host/src/editor_audio/cycles/plan.rs#L22) |
| fn final_observe | [源码:25](../../../desktop/host/src/editor_audio/cycles/plan.rs#L25) |
| fn next | [源码:29](../../../desktop/host/src/editor_audio/cycles/plan.rs#L29) |
| fn released | [源码:44](../../../desktop/host/src/editor_audio/cycles/plan.rs#L44) |
| mod tests | [源码:53](../../../desktop/host/src/editor_audio/cycles/plan.rs#L53) |
| fn full_and_smoke_schedules_are_distinct_bounded_and_finish_inside_playback | [源码:56](../../../desktop/host/src/editor_audio/cycles/plan.rs#L56) |
| fn a_missing_join_or_any_live_lease_prevents_the_next_resource_cycle | [源码:72](../../../desktop/host/src/editor_audio/cycles/plan.rs#L72) |

## desktop/host/src/editor_audio/cycles/report.rs

129 行；SHA256 `9f9e9e425face993a97c17ba5b2055183c5f304c786c8ce05f0c398c4f8f9ebb`

导入：`use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| impl impl Round | [源码:2](../../../desktop/host/src/editor_audio/cycles/report.rs#L2) |
| fn record | [源码:3](../../../desktop/host/src/editor_audio/cycles/report.rs#L3) |
| fn summary | [源码:84](../../../desktop/host/src/editor_audio/cycles/report.rs#L84) |
| impl impl Run | [源码:97](../../../desktop/host/src/editor_audio/cycles/report.rs#L97) |
| fn report | [源码:98](../../../desktop/host/src/editor_audio/cycles/report.rs#L98) |

## desktop/host/src/editor_audio/faults.rs

357 行；SHA256 `e96049d64d823c4436ffa468c19ad4ed5391e22bf09dad9b2ee3b0cf46dee063`

导入：`use super::combined::probes::Probes;`, `use super::gui_verification::Observations;`, `use super::*;`, `use crate::audio_verification::{component, submit, tone};`, `use crate::load_verification::phase;`, `use skin_editor_infrastructure::{WavDecoder, background_stream::BufferedBackground};`, `use std::{ fs::File, path::Path, time::{Duration, Instant}, };`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| fn error | [源码:14](../../../desktop/host/src/editor_audio/faults.rs#L14) |
| impl impl std::fmt::Debug) -> String | [源码:14](../../../desktop/host/src/editor_audio/faults.rs#L14) |
| fn until | [源码:17](../../../desktop/host/src/editor_audio/faults.rs#L17) |
| impl impl FnMut() -> Result<bool, String>) -> Result<(), String> | [源码:17](../../../desktop/host/src/editor_audio/faults.rs#L17) |
| fn selected | [源码:30](../../../desktop/host/src/editor_audio/faults.rs#L30) |
| fn transport | [源码:46](../../../desktop/host/src/editor_audio/faults.rs#L46) |
| fn measure | [源码:57](../../../desktop/host/src/editor_audio/faults.rs#L57) |
| fn run | [源码:131](../../../desktop/host/src/editor_audio/faults.rs#L131) |
| mod tests | [源码:328](../../../desktop/host/src/editor_audio/faults.rs#L328) |
| fn recovery_requires_a_confirmed_matching_background_and_keeps_the_copy_position | [源码:331](../../../desktop/host/src/editor_audio/faults.rs#L331) |

## desktop/host/src/editor_audio/gui_verification.rs

1295 行；SHA256 `6092029d59f2ab17bac7e11ea22b1371e17b6c38f094e876b37b0f28a01d6f47`

导入：`use super::*;`, `use crate::{Editor, load_verification::phase};`, `use eframe::egui;`, `use skin_editor_infrastructure::{ WavDecoder, separated_native::{FirstCopy, FirstObserver, NativeStats}, };`, `use std::{ cell::RefCell, path::PathBuf, rc::Rc, sync::{ Arc, Mutex, atomic::{AtomicBool, Ordering}, }, thread::JoinHandle, time::{Duration, Instant}, };`, `use skin_editor_ui::Presenter;`, `use skin_editor_ui::Presenter;`, `use super::GuiWork;`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| struct Shared | [源码:22](../../../desktop/host/src/editor_audio/gui_verification.rs#L22) |
| enum OutputProbe | [源码:24](../../../desktop/host/src/editor_audio/gui_verification.rs#L24) |
| impl impl SkinAudioPort for Shared | [源码:32](../../../desktop/host/src/editor_audio/gui_verification.rs#L32) |
| fn retained_drafts | [源码:33](../../../desktop/host/src/editor_audio/gui_verification.rs#L33) |
| fn synchronize | [源码:36](../../../desktop/host/src/editor_audio/gui_verification.rs#L36) |
| impl impl AudioEditorPort for Shared | [源码:45](../../../desktop/host/src/editor_audio/gui_verification.rs#L45) |
| fn request | [源码:46](../../../desktop/host/src/editor_audio/gui_verification.rs#L46) |
| fn audition_asset | [源码:49](../../../desktop/host/src/editor_audio/gui_verification.rs#L49) |
| fn poll | [源码:52](../../../desktop/host/src/editor_audio/gui_verification.rs#L52) |
| struct Observations | [源码:56](../../../desktop/host/src/editor_audio/gui_verification.rs#L56) |
| impl impl Observations | [源码:63](../../../desktop/host/src/editor_audio/gui_verification.rs#L63) |
| fn new | [源码:64](../../../desktop/host/src/editor_audio/gui_verification.rs#L64) |
| fn with_capacity | [源码:67](../../../desktop/host/src/editor_audio/gui_verification.rs#L67) |
| fn close | [源码:103](../../../desktop/host/src/editor_audio/gui_verification.rs#L103) |
| impl impl Drop for Observations | [源码:111](../../../desktop/host/src/editor_audio/gui_verification.rs#L111) |
| fn drop | [源码:112](../../../desktop/host/src/editor_audio/gui_verification.rs#L112) |
| fn wav | [源码:116](../../../desktop/host/src/editor_audio/gui_verification.rs#L116) |
| struct Probe | [源码:136](../../../desktop/host/src/editor_audio/gui_verification.rs#L136) |
| struct GuiWork | [源码:142](../../../desktop/host/src/editor_audio/gui_verification.rs#L142) |
| impl impl GuiWork | [源码:149](../../../desktop/host/src/editor_audio/gui_verification.rs#L149) |
| fn add | [源码:150](../../../desktop/host/src/editor_audio/gui_verification.rs#L150) |
| fn previous_frame | [源码:155](../../../desktop/host/src/editor_audio/gui_verification.rs#L155) |
| struct Run | [源码:172](../../../desktop/host/src/editor_audio/gui_verification.rs#L172) |
| impl impl Run | [源码:219](../../../desktop/host/src/editor_audio/gui_verification.rs#L219) |
| fn new | [源码:220](../../../desktop/host/src/editor_audio/gui_verification.rs#L220) |
| fn component | [源码:383](../../../desktop/host/src/editor_audio/gui_verification.rs#L383) |
| fn action | [源码:386](../../../desktop/host/src/editor_audio/gui_verification.rs#L386) |
| fn advance | [源码:392](../../../desktop/host/src/editor_audio/gui_verification.rs#L392) |
| fn ready | [源码:433](../../../desktop/host/src/editor_audio/gui_verification.rs#L433) |
| fn drain | [源码:441](../../../desktop/host/src/editor_audio/gui_verification.rs#L441) |
| fn step | [源码:462](../../../desktop/host/src/editor_audio/gui_verification.rs#L462) |
| fn report | [源码:853](../../../desktop/host/src/editor_audio/gui_verification.rs#L853) |
| fn draw_control | [源码:916](../../../desktop/host/src/editor_audio/gui_verification.rs#L916) |
| impl impl eframe::App for Run | [源码:982](../../../desktop/host/src/editor_audio/gui_verification.rs#L982) |
| fn raw_input_hook | [源码:983](../../../desktop/host/src/editor_audio/gui_verification.rs#L983) |
| fn logic | [源码:994](../../../desktop/host/src/editor_audio/gui_verification.rs#L994) |
| fn ui | [源码:1024](../../../desktop/host/src/editor_audio/gui_verification.rs#L1024) |
| fn clear_color | [源码:1089](../../../desktop/host/src/editor_audio/gui_verification.rs#L1089) |
| fn on_exit | [源码:1099](../../../desktop/host/src/editor_audio/gui_verification.rs#L1099) |
| fn run | [源码:1123](../../../desktop/host/src/editor_audio/gui_verification.rs#L1123) |
| fn streamed | [源码:1126](../../../desktop/host/src/editor_audio/gui_verification.rs#L1126) |
| fn profiled | [源码:1129](../../../desktop/host/src/editor_audio/gui_verification.rs#L1129) |
| fn main_work | [源码:1132](../../../desktop/host/src/editor_audio/gui_verification.rs#L1132) |
| fn repaint_only | [源码:1135](../../../desktop/host/src/editor_audio/gui_verification.rs#L1135) |
| fn dispatch | [源码:1146](../../../desktop/host/src/editor_audio/gui_verification.rs#L1146) |
| fn baseline | [源码:1156](../../../desktop/host/src/editor_audio/gui_verification.rs#L1156) |
| fn text_work | [源码:1166](../../../desktop/host/src/editor_audio/gui_verification.rs#L1166) |
| fn mesh_replay | [源码:1169](../../../desktop/host/src/editor_audio/gui_verification.rs#L1169) |
| fn gpu_path | [源码:1172](../../../desktop/host/src/editor_audio/gui_verification.rs#L1172) |
| fn run_mode | [源码:1186](../../../desktop/host/src/editor_audio/gui_verification.rs#L1186) |
| fn run_control | [源码:1201](../../../desktop/host/src/editor_audio/gui_verification.rs#L1201) |
| mod gui_work_tests | [源码:1246](../../../desktop/host/src/editor_audio/gui_verification.rs#L1246) |
| fn previous_frame_belongs_to_submitted_stage_and_is_not_counted_twice | [源码:1250](../../../desktop/host/src/editor_audio/gui_verification.rs#L1250) |
| fn absent_first_frame_and_invalid_values_are_not_cpu_samples | [源码:1265](../../../desktop/host/src/editor_audio/gui_verification.rs#L1265) |
| mod observation_close_tests | [源码:1280](../../../desktop/host/src/editor_audio/gui_verification.rs#L1280) |
| fn observer_panic_is_a_sticky_failed_join | [源码:1283](../../../desktop/host/src/editor_audio/gui_verification.rs#L1283) |

## desktop/host/src/editor_audio/stability.rs

461 行；SHA256 `6f0ec073ad0e19343bbed01d484b6bfe7a9199c236d2112e84c62b57552dc510`

导入：`use super::{gui_verification::Shared, *};`, `use crate::{ Editor, fresh_session, stable_window::{Interval, MEASURE, SETTLE}, };`, `use eframe::egui;`, `use std::{ cell::RefCell, path::PathBuf, rc::Rc, time::{Duration, Instant}, };`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| struct Run | [源码:15](../../../desktop/host/src/editor_audio/stability.rs#L15) |
| fn error | [源码:37](../../../desktop/host/src/editor_audio/stability.rs#L37) |
| impl impl std::fmt::Debug) -> String | [源码:37](../../../desktop/host/src/editor_audio/stability.rs#L37) |
| impl impl Run | [源码:40](../../../desktop/host/src/editor_audio/stability.rs#L40) |
| fn new | [源码:41](../../../desktop/host/src/editor_audio/stability.rs#L41) |
| fn usage | [源码:79](../../../desktop/host/src/editor_audio/stability.rs#L79) |
| fn background | [源码:90](../../../desktop/host/src/editor_audio/stability.rs#L90) |
| fn shape | [源码:98](../../../desktop/host/src/editor_audio/stability.rs#L98) |
| fn begin | [源码:110](../../../desktop/host/src/editor_audio/stability.rs#L110) |
| fn action | [源码:136](../../../desktop/host/src/editor_audio/stability.rs#L136) |
| fn load | [源码:143](../../../desktop/host/src/editor_audio/stability.rs#L143) |
| fn finish_due | [源码:221](../../../desktop/host/src/editor_audio/stability.rs#L221) |
| fn after_draw | [源码:272](../../../desktop/host/src/editor_audio/stability.rs#L272) |
| fn report | [源码:327](../../../desktop/host/src/editor_audio/stability.rs#L327) |
| impl impl eframe::App for Run | [源码:354](../../../desktop/host/src/editor_audio/stability.rs#L354) |
| fn raw_input_hook | [源码:355](../../../desktop/host/src/editor_audio/stability.rs#L355) |
| fn logic | [源码:361](../../../desktop/host/src/editor_audio/stability.rs#L361) |
| fn ui | [源码:369](../../../desktop/host/src/editor_audio/stability.rs#L369) |
| fn on_exit | [源码:402](../../../desktop/host/src/editor_audio/stability.rs#L402) |
| fn physical_input | [源码:409](../../../desktop/host/src/editor_audio/stability.rs#L409) |
| fn run | [源码:415](../../../desktop/host/src/editor_audio/stability.rs#L415) |
| fn run_policy | [源码:418](../../../desktop/host/src/editor_audio/stability.rs#L418) |
| mod tests | [源码:450](../../../desktop/host/src/editor_audio/stability.rs#L450) |
| fn focus_notifications_do_not_hide_real_pointer_or_text_input | [源码:453](../../../desktop/host/src/editor_audio/stability.rs#L453) |

## desktop/host/src/editor_audio/window_recovery.rs

637 行；SHA256 `793e406179414d0893b7d43c645f4aef0b37631289ffd391fd5a5333f6808769`

导入：`use super::{ combined::probes::Probes, gui_verification::{Observations, Shared}, *, };`, `use crate::{Editor, presentation::RebuildSeed};`, `use eframe::{App, egui};`, `use skin_editor_ui::Presenter;`, `use std::{ cell::RefCell, path::{Path, PathBuf}, rc::Rc, sync::{ Arc, atomic::{AtomicU8, Ordering}, mpsc, }, time::{Duration, Instant}, };`

| 定义 | 位置／签名 |
| --- | --- |
| mod activity | [源码:21](../../../desktop/host/src/editor_audio/window_recovery.rs#L21) |
| mod backend | [源码:22](../../../desktop/host/src/editor_audio/window_recovery.rs#L22) |
| mod fault | [源码:23](../../../desktop/host/src/editor_audio/window_recovery.rs#L23) |
| mod init_failure | [源码:24](../../../desktop/host/src/editor_audio/window_recovery.rs#L24) |
| mod report | [源码:25](../../../desktop/host/src/editor_audio/window_recovery.rs#L25) |
| mod restore | [源码:26](../../../desktop/host/src/editor_audio/window_recovery.rs#L26) |
| mod sound | [源码:27](../../../desktop/host/src/editor_audio/window_recovery.rs#L27) |
| fn error | [源码:28](../../../desktop/host/src/editor_audio/window_recovery.rs#L28) |
| impl impl std::fmt::Debug) -> String | [源码:28](../../../desktop/host/src/editor_audio/window_recovery.rs#L28) |
| struct Seed | [源码:31](../../../desktop/host/src/editor_audio/window_recovery.rs#L31) |
| impl impl Seed | [源码:38](../../../desktop/host/src/editor_audio/window_recovery.rs#L38) |
| fn restore | [源码:39](../../../desktop/host/src/editor_audio/window_recovery.rs#L39) |
| struct Returned | [源码:76](../../../desktop/host/src/editor_audio/window_recovery.rs#L76) |
| struct Run | [源码:87](../../../desktop/host/src/editor_audio/window_recovery.rs#L87) |
| impl impl Run | [源码:125](../../../desktop/host/src/editor_audio/window_recovery.rs#L125) |
| fn shot | [源码:126](../../../desktop/host/src/editor_audio/window_recovery.rs#L126) |
| fn screenshots | [源码:137](../../../desktop/host/src/editor_audio/window_recovery.rs#L137) |
| fn step | [源码:176](../../../desktop/host/src/editor_audio/window_recovery.rs#L176) |
| fn size | [源码:440](../../../desktop/host/src/editor_audio/window_recovery.rs#L440) |
| impl impl App for Run | [源码:444](../../../desktop/host/src/editor_audio/window_recovery.rs#L444) |
| fn logic | [源码:445](../../../desktop/host/src/editor_audio/window_recovery.rs#L445) |
| fn ui | [源码:492](../../../desktop/host/src/editor_audio/window_recovery.rs#L492) |
| fn on_exit | [源码:562](../../../desktop/host/src/editor_audio/window_recovery.rs#L562) |
| fn run | [源码:618](../../../desktop/host/src/editor_audio/window_recovery.rs#L618) |
| fn run_fault | [源码:621](../../../desktop/host/src/editor_audio/window_recovery.rs#L621) |

## desktop/host/src/editor_audio/window_recovery/activity.rs

299 行；SHA256 `8b49782855c46608211acd713beede1c7697f8d914834ecf28b55f98a8c71cb5`

导入：`use super::*;`, `use eframe::egui_wgpu::wgpu;`, `use skin_editor_ui::Selection;`

| 定义 | 位置／签名 |
| --- | --- |
| struct ResultRecord | [源码:6](../../../desktop/host/src/editor_audio/window_recovery/activity.rs#L6) |
| fn pass | [源码:11](../../../desktop/host/src/editor_audio/window_recovery/activity.rs#L11) |
| fn ready_frame | [源码:35](../../../desktop/host/src/editor_audio/window_recovery/activity.rs#L35) |
| fn button | [源码:38](../../../desktop/host/src/editor_audio/window_recovery/activity.rs#L38) |
| fn draft | [源码:46](../../../desktop/host/src/editor_audio/window_recovery/activity.rs#L46) |
| fn exercise | [源码:53](../../../desktop/host/src/editor_audio/window_recovery/activity.rs#L53) |

## desktop/host/src/editor_audio/window_recovery/backend.rs

200 行；SHA256 `e1cc23b0f6a65dce13e2037d7b44ff3f4fd42bad65c198f79746c1c50bd14c81`

导入：`use crate::presentation::DeviceLoss;`, `use eframe::{ NativeOptions, egui_wgpu::{self, wgpu}, };`, `use std::sync::{ Arc, atomic::{AtomicBool, AtomicU8, Ordering}, };`, `use std::time::Duration;`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| struct RendererWitness | [源码:13](../../../desktop/host/src/editor_audio/window_recovery/backend.rs#L13) |
| impl impl Drop for RendererWitness | [源码:14](../../../desktop/host/src/editor_audio/window_recovery/backend.rs#L14) |
| fn drop | [源码:15](../../../desktop/host/src/editor_audio/window_recovery/backend.rs#L15) |
| struct Backend | [源码:19](../../../desktop/host/src/editor_audio/window_recovery/backend.rs#L19) |
| impl impl Backend | [源码:27](../../../desktop/host/src/editor_audio/window_recovery/backend.rs#L27) |
| fn new | [源码:28](../../../desktop/host/src/editor_audio/window_recovery/backend.rs#L28) |
| fn with_fault | [源码:31](../../../desktop/host/src/editor_audio/window_recovery/backend.rs#L31) |
| fn destroy | [源码:138](../../../desktop/host/src/editor_audio/window_recovery/backend.rs#L138) |
| fn retired | [源码:153](../../../desktop/host/src/editor_audio/window_recovery/backend.rs#L153) |
| fn surface_ids_only | [源码:167](../../../desktop/host/src/editor_audio/window_recovery/backend.rs#L167) |
| mod tests | [源码:171](../../../desktop/host/src/editor_audio/window_recovery/backend.rs#L171) |
| fn witness_reports_actual_owner_drop_and_does_not_pin_the_owner | [源码:174](../../../desktop/host/src/editor_audio/window_recovery/backend.rs#L174) |
| fn next_window_requires_every_actual_retirement_condition | [源码:182](../../../desktop/host/src/editor_audio/window_recovery/backend.rs#L182) |
| fn unassigned_surface_ids_require_exact_failure_evidence_and_no_live_object | [源码:194](../../../desktop/host/src/editor_audio/window_recovery/backend.rs#L194) |

## desktop/host/src/editor_audio/window_recovery/fault.rs

195 行；SHA256 `74267a1c2c39ea14c2d0e2b9a72838c44902e2397f88f379ae262f1c61044c57`

导入：`use super::*;`, `use eframe::egui_wgpu::{self, wgpu};`, `use skin_editor_infrastructure::separated_native::NativeFaultSignal;`, `use std::sync::{OnceLock, atomic::AtomicU64};`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| enum Plan | [源码:9](../../../desktop/host/src/editor_audio/window_recovery/fault.rs#L9) |
| impl impl Plan | [源码:23](../../../desktop/host/src/editor_audio/window_recovery/fault.rs#L23) |
| fn code | [源码:24](../../../desktop/host/src/editor_audio/window_recovery/fault.rs#L24) |
| fn active_edit | [源码:40](../../../desktop/host/src/editor_audio/window_recovery/fault.rs#L40) |
| struct Signals | [源码:45](../../../desktop/host/src/editor_audio/window_recovery/fault.rs#L45) |
| impl impl Signals | [源码:63](../../../desktop/host/src/editor_audio/window_recovery/fault.rs#L63) |
| fn bind | [源码:64](../../../desktop/host/src/editor_audio/window_recovery/fault.rs#L64) |
| fn notify | [源码:72](../../../desktop/host/src/editor_audio/window_recovery/fault.rs#L72) |
| fn record_error | [源码:81](../../../desktop/host/src/editor_audio/window_recovery/fault.rs#L81) |
| fn native_error | [源码:87](../../../desktop/host/src/editor_audio/window_recovery/fault.rs#L87) |
| fn fire | [源码:96](../../../desktop/host/src/editor_audio/window_recovery/fault.rs#L96) |
| fn json | [源码:105](../../../desktop/host/src/editor_audio/window_recovery/fault.rs#L105) |
| struct Inject | [源码:126](../../../desktop/host/src/editor_audio/window_recovery/fault.rs#L126) |
| impl impl egui_wgpu::CallbackTrait for Inject | [源码:131](../../../desktop/host/src/editor_audio/window_recovery/fault.rs#L131) |
| fn prepare | [源码:132](../../../desktop/host/src/editor_audio/window_recovery/fault.rs#L132) |
| fn paint | [源码:146](../../../desktop/host/src/editor_audio/window_recovery/fault.rs#L146) |
| fn append | [源码:158](../../../desktop/host/src/editor_audio/window_recovery/fault.rs#L158) |
| mod tests | [源码:175](../../../desktop/host/src/editor_audio/window_recovery/fault.rs#L175) |
| fn pre_injection_errors_cannot_be_reclassified_as_expected_loss | [源码:178](../../../desktop/host/src/editor_audio/window_recovery/fault.rs#L178) |
| fn repeated_notices_do_not_change_injection_identity_or_clear_failure | [源码:187](../../../desktop/host/src/editor_audio/window_recovery/fault.rs#L187) |

## desktop/host/src/editor_audio/window_recovery/init_failure.rs

564 行；SHA256 `abc71b88e015c311491113bb75df8c7eab56a1ac7168e84714f6620424f379c6`

导入：`use super::*;`, `use restore::{Attempt, Owner, Slot, State};`

| 定义 | 位置／签名 |
| --- | --- |
| mod activity_tests | [源码:5](../../../desktop/host/src/editor_audio/window_recovery/init_failure.rs#L5) |
| mod endpoint_tests | [源码:7](../../../desktop/host/src/editor_audio/window_recovery/init_failure.rs#L7) |
| struct Stamp | [源码:10](../../../desktop/host/src/editor_audio/window_recovery/init_failure.rs#L10) |
| impl impl Stamp | [源码:16](../../../desktop/host/src/editor_audio/window_recovery/init_failure.rs#L16) |
| fn same_committed | [源码:17](../../../desktop/host/src/editor_audio/window_recovery/init_failure.rs#L17) |
| fn requests | [源码:23](../../../desktop/host/src/editor_audio/window_recovery/init_failure.rs#L23) |
| fn read | [源码:26](../../../desktop/host/src/editor_audio/window_recovery/init_failure.rs#L26) |
| fn fixture | [源码:63](../../../desktop/host/src/editor_audio/window_recovery/init_failure.rs#L63) |
| fn verify_redo | [源码:88](../../../desktop/host/src/editor_audio/window_recovery/init_failure.rs#L88) |
| struct Recovered | [源码:115](../../../desktop/host/src/editor_audio/window_recovery/init_failure.rs#L115) |
| fn run | [源码:122](../../../desktop/host/src/editor_audio/window_recovery/init_failure.rs#L122) |

## desktop/host/src/editor_audio/window_recovery/init_failure/activity_tests.rs

144 行；SHA256 `23afc5da0cbf35e1b8f58edf8816d8fde616f9c79acff1ed7aa196e9ded37e79`

导入：`use super::endpoint_tests::rig_with_ui;`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| fn settle | [源码:4](../../../desktop/host/src/editor_audio/window_recovery/init_failure/activity_tests.rs#L4) |
| fn active_audio_draft_fault_or_ui_interrupt_invalidates_old_voices_without_committing | [源码:39](../../../desktop/host/src/editor_audio/window_recovery/init_failure/activity_tests.rs#L39) |

## desktop/host/src/editor_audio/window_recovery/init_failure/endpoint_tests.rs

180 行；SHA256 `7403cee2ecdfd21aaf102b85a0adb5c4dce96bbe0a319d0cef14052205d4c296`

导入：`use super::*;`, `use skin_editor_infrastructure::{ BlobStore, WavDecoder, separated_native::{ControlledCallback, ReturnGate}, };`

| 定义 | 位置／签名 |
| --- | --- |
| fn rig_with_ui | [源码:7](../../../desktop/host/src/editor_audio/window_recovery/init_failure/endpoint_tests.rs#L7) |
| fn rig | [源码:70](../../../desktop/host/src/editor_audio/window_recovery/init_failure/endpoint_tests.rs#L70) |
| fn pending_return_preserves_all_history_and_only_actual_delivery_releases_pcm | [源码:82](../../../desktop/host/src/editor_audio/window_recovery/init_failure/endpoint_tests.rs#L82) |
| fn disconnected_return_preserves_history_and_blocks_disable_enable_until_owner_exit | [源码:121](../../../desktop/host/src/editor_audio/window_recovery/init_failure/endpoint_tests.rs#L121) |
| fn wrong_returned_endpoint_is_retained_instead_of_being_dropped_on_finish_error | [源码:152](../../../desktop/host/src/editor_audio/window_recovery/init_failure/endpoint_tests.rs#L152) |

## desktop/host/src/editor_audio/window_recovery/report.rs

453 行；SHA256 `8d95a5a571130d79d6b7727e3300d32744ef71cf1f301f4d3d831753122070b5`

导入：`use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| fn window | [源码:2](../../../desktop/host/src/editor_audio/window_recovery/report.rs#L2) |
| fn run | [源码:42](../../../desktop/host/src/editor_audio/window_recovery/report.rs#L42) |
| fn reject_creation | [源码:412](../../../desktop/host/src/editor_audio/window_recovery/report.rs#L412) |

## desktop/host/src/editor_audio/window_recovery/restore.rs

544 行；SHA256 `e42d50e55afebebc6daa23bfb3c51fe278557a27d962f77db3a4cf9d62fc581c`

导入：`use super::*;`, `use eframe::egui_wgpu::wgpu;`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| struct Slot | [源码:5](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L5) |
| impl impl<T> Slot<T> | [源码:8](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L8) |
| fn new | [源码:9](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L9) |
| fn vacant | [源码:12](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L12) |
| fn take | [源码:15](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L15) |
| fn get | [源码:22](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L22) |
| struct Loan | [源码:26](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L26) |
| impl impl<T> Loan<'_, T> | [源码:30](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L30) |
| fn get_mut | [源码:31](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L31) |
| fn finish | [源码:34](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L34) |
| impl impl<T> Drop for Loan<'_, T> | [源码:38](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L38) |
| fn drop | [源码:39](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L39) |
| enum Owner | [源码:45](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L45) |
| struct State | [源码:49](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L49) |
| impl impl State | [源码:58](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L58) |
| fn seed | [源码:59](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L59) |
| fn snapshot | [源码:70](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L70) |
| fn retry_ready | [源码:77](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L77) |
| fn runtime_retired | [源码:80](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L80) |
| fn framework_retired | [源码:85](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L85) |
| fn open_backend | [源码:92](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L92) |
| fn retry_cleanup | [源码:100](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L100) |
| fn settle | [源码:153](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L153) |
| enum Failure | [源码:167](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L167) |
| static str | [源码:168](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L168) |
| static str | [源码:169](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L169) |
| impl impl Failure | [源码:171](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L171) |
| fn message | [源码:172](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L172) |
| static str | [源码:172](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L172) |
| struct Gate | [源码:179](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L179) |
| impl impl Gate | [源码:180](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L180) |
| fn terminal | [源码:181](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L181) |
| fn run | [源码:184](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L184) |
| impl impl FnOnce() -> Result<(), Failure>) -> Result<(), Failure> | [源码:184](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L184) |
| struct Cleanup | [源码:195](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L195) |
| impl impl Cleanup | [源码:202](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L202) |
| fn step | [源码:203](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L203) |
| struct Attempt | [源码:272](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L272) |
| impl impl<'a> Attempt<'a> | [源码:280](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L280) |
| fn begin | [源码:281](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L281) |
| fn fresh | [源码:312](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L312) |
| fn editor | [源码:331](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L331) |
| fn attach_audio | [源码:343](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L343) |
| fn prepared | [源码:346](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L346) |
| fn commit | [源码:349](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L349) |
| impl impl Drop for Attempt<'_> | [源码:358](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L358) |
| fn drop | [源码:359](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L359) |
| mod tests | [源码:380](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L380) |
| fn pending_cleanup_keeps_history_and_inflight_ownership_until_release_receipt | [源码:383](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L383) |
| struct Lease | [源码:384](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L384) |
| impl impl Drop for Lease | [源码:385](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L385) |
| fn drop | [源码:386](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L386) |
| const ProjectSession | [源码:399](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L399) |
| const ProjectSession | [源码:424](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L424) |
| fn failed_join_is_sticky_and_cannot_be_cleared_by_a_successful_noop_retry | [源码:430](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L430) |
| fn project | [源码:454](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L454) |
| fn failed_loan_and_retry_preserve_the_same_session_and_both_branch_contents | [源码:482](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L482) |
| const ProjectSession | [源码:484](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L484) |
| const ProjectSession | [源码:491](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L491) |
| const ProjectSession | [源码:500](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L500) |
| fn unwinding_before_publish_returns_history_without_creating_another_node | [源码:503](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L503) |
| fn cancelled_initialization_keeps_an_inflight_lease_until_its_actual_owner_exits | [源码:518](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L518) |
| struct Lease | [源码:519](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L519) |
| impl impl Drop for Lease | [源码:520](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L520) |
| fn drop | [源码:521](../../../desktop/host/src/editor_audio/window_recovery/restore.rs#L521) |

## desktop/host/src/editor_audio/window_recovery/sound.rs

174 行；SHA256 `d10194feba9e75cd636aa4e5044fde9c73e26db53bc6ac85b1c6de02ba7eeed7`

导入：`use super::*;`, `use crate::audio_verification::tone;`, `use skin_editor_infrastructure::{WavDecoder, background_stream::BufferedBackground};`, `use std::fs::File;`

| 定义 | 位置／签名 |
| --- | --- |
| fn component | [源码:6](../../../desktop/host/src/editor_audio/window_recovery/sound.rs#L6) |
| fn install | [源码:9](../../../desktop/host/src/editor_audio/window_recovery/sound.rs#L9) |
| fn reopen | [源码:75](../../../desktop/host/src/editor_audio/window_recovery/sound.rs#L75) |
| fn collect | [源码:100](../../../desktop/host/src/editor_audio/window_recovery/sound.rs#L100) |
| fn trigger | [源码:121](../../../desktop/host/src/editor_audio/window_recovery/sound.rs#L121) |
| fn rows | [源码:153](../../../desktop/host/src/editor_audio/window_recovery/sound.rs#L153) |

## desktop/host/src/frame_worker.rs

494 行；SHA256 `1d998bcc095d6203186592a4e879de77f4de1ab842f86e72704dd7c3396da3d8`

导入：`use skin_editor_application::{FrameDescriptor, PreviewError};`, `use skin_editor_infrastructure::{Images, RawBlob};`, `use skin_editor_rendering::{Images as Rasters, Plan};`, `use std::sync::{ Arc, atomic::{AtomicBool, Ordering}, mpsc::{self, Receiver, SyncSender}, };`, `use std::thread::JoinHandle;`, `use std::time::{Duration, Instant};`, `use super::*;`, `use skin_editor_application::{values::*, *};`, `use skin_editor_rendering::Target;`, `use skin_editor_application::values::{Asset, AssetId, AssetKind};`, `use skin_editor_application::{Command, DraftAction, SessionLimits};`, `use skin_editor_ui::EditorUi;`

| 定义 | 位置／签名 |
| --- | --- |
| struct Job | [源码:13](../../../desktop/host/src/frame_worker.rs#L13) |
| impl impl Job | [源码:22](../../../desktop/host/src/frame_worker.rs#L22) |
| fn is_cancelled | [源码:23](../../../desktop/host/src/frame_worker.rs#L23) |
| struct Completion | [源码:27](../../../desktop/host/src/frame_worker.rs#L27) |
| struct FrameWorker | [源码:35](../../../desktop/host/src/frame_worker.rs#L35) |
| impl impl<T: Send + 'static> FrameWorker<T> | [源码:43](../../../desktop/host/src/frame_worker.rs#L43) |
| fn new | [源码:44](../../../desktop/host/src/frame_worker.rs#L44) |
| impl impl Fn(&Job) -> Result<Option<T>, PreviewError> + Send + 'static, wake: impl Fn() + Send + 'static, ) -> Result<Self, PreviewError> | [源码:45](../../../desktop/host/src/frame_worker.rs#L45) |
| impl impl Fn() + Send + 'static, ) -> Result<Self, PreviewError> | [源码:46](../../../desktop/host/src/frame_worker.rs#L46) |
| fn current | [源码:84](../../../desktop/host/src/frame_worker.rs#L84) |
| fn cancel | [源码:87](../../../desktop/host/src/frame_worker.rs#L87) |
| fn try_close | [源码:95](../../../desktop/host/src/frame_worker.rs#L95) |
| fn submit | [源码:106](../../../desktop/host/src/frame_worker.rs#L106) |
| fn poll | [源码:119](../../../desktop/host/src/frame_worker.rs#L119) |
| fn wait | [源码:140](../../../desktop/host/src/frame_worker.rs#L140) |
| fn close | [源码:152](../../../desktop/host/src/frame_worker.rs#L152) |
| fn closed | [源码:164](../../../desktop/host/src/frame_worker.rs#L164) |
| impl impl<T> Drop for FrameWorker<T> | [源码:173](../../../desktop/host/src/frame_worker.rs#L173) |
| fn drop | [源码:174](../../../desktop/host/src/frame_worker.rs#L174) |
| fn decode | [源码:186](../../../desktop/host/src/frame_worker.rs#L186) |
| mod tests | [源码:202](../../../desktop/host/src/frame_worker.rs#L202) |
| fn fault_poll_does_not_wait_or_refund_a_gate_held_job | [源码:205](../../../desktop/host/src/frame_worker.rs#L205) |
| fn close_joins_held_work_before_success_and_detects_worker_panics | [源码:234](../../../desktop/host/src/frame_worker.rs#L234) |
| fn job | [源码:264](../../../desktop/host/src/frame_worker.rs#L264) |
| fn cancellation_keeps_credit_and_raw_lease_until_actual_completion | [源码:300](../../../desktop/host/src/frame_worker.rs#L300) |
| fn fault_cancelled_draft_keeps_inflight_raw_until_late_completion_or_actual_join | [源码:341](../../../desktop/host/src/frame_worker.rs#L341) |
| fn completed_result_and_disconnect_release_raw_ownership | [源码:441](../../../desktop/host/src/frame_worker.rs#L441) |
| fn real_png_reuse_cancellation_and_invalid_bytes_are_explicit | [源码:460](../../../desktop/host/src/frame_worker.rs#L460) |
| fn preparation_failure_releases_credit_and_next_request_can_run | [源码:481](../../../desktop/host/src/frame_worker.rs#L481) |

## desktop/host/src/framework_trace.rs

544 行；SHA256 `eeb315f916ad2f49e0a07e30eae8cdfc75911b9445119737c8786d91cd251d5f`

导入：`use std::{ fmt, sync::{ Arc, Mutex, atomic::{AtomicU8, Ordering}, }, time::{Instant, SystemTime, UNIX_EPOCH}, };`, `use tracing::{ Event, Metadata, Subscriber, field::{Field, Visit}, span::{Attributes, Id, Record}, };`, `use fmt::Write;`, `use event_loop::run_native;`, `use super::*;`, `use fmt::Write;`, `use eframe::egui;`

| 定义 | 位置／签名 |
| --- | --- |
| const LIMIT | [源码:16](../../../desktop/host/src/framework_trace.rs#L16) |
| fn candidate | [源码:21](../../../desktop/host/src/framework_trace.rs#L21) |
| struct Text | [源码:36](../../../desktop/host/src/framework_trace.rs#L36) |
| impl impl Default for Text | [源码:40](../../../desktop/host/src/framework_trace.rs#L40) |
| fn default | [源码:41](../../../desktop/host/src/framework_trace.rs#L41) |
| impl impl fmt::Write for Text | [源码:48](../../../desktop/host/src/framework_trace.rs#L48) |
| fn write_str | [源码:49](../../../desktop/host/src/framework_trace.rs#L49) |
| impl impl Text | [源码:58](../../../desktop/host/src/framework_trace.rs#L58) |
| fn as_str | [源码:59](../../../desktop/host/src/framework_trace.rs#L59) |
| struct Tags | [源码:66](../../../desktop/host/src/framework_trace.rs#L66) |
| impl impl Visit for Tags | [源码:70](../../../desktop/host/src/framework_trace.rs#L70) |
| fn record_debug | [源码:71](../../../desktop/host/src/framework_trace.rs#L71) |
| fn kind | [源码:83](../../../desktop/host/src/framework_trace.rs#L83) |
| struct Slot | [源码:117](../../../desktop/host/src/framework_trace.rs#L117) |
| struct Data | [源码:122](../../../desktop/host/src/framework_trace.rs#L122) |
| static str | [源码:128](../../../desktop/host/src/framework_trace.rs#L128) |
| struct Pass | [源码:130](../../../desktop/host/src/framework_trace.rs#L130) |
| impl impl Data | [源码:135](../../../desktop/host/src/framework_trace.rs#L135) |
| fn new | [源码:136](../../../desktop/host/src/framework_trace.rs#L136) |
| fn open | [源码:146](../../../desktop/host/src/framework_trace.rs#L146) |
| fn enter | [源码:162](../../../desktop/host/src/framework_trace.rs#L162) |
| fn exit | [源码:176](../../../desktop/host/src/framework_trace.rs#L176) |
| fn close | [源码:203](../../../desktop/host/src/framework_trace.rs#L203) |
| struct Inner | [源码:222](../../../desktop/host/src/framework_trace.rs#L222) |
| struct Trace | [源码:228](../../../desktop/host/src/framework_trace.rs#L228) |
| impl impl Trace | [源码:229](../../../desktop/host/src/framework_trace.rs#L229) |
| fn new | [源码:230](../../../desktop/host/src/framework_trace.rs#L230) |
| fn stamp | [源码:237](../../../desktop/host/src/framework_trace.rs#L237) |
| fn stage | [源码:246](../../../desktop/host/src/framework_trace.rs#L246) |
| fn logic | [源码:249](../../../desktop/host/src/framework_trace.rs#L249) |
| fn report | [源码:254](../../../desktop/host/src/framework_trace.rs#L254) |
| struct Scope | [源码:293](../../../desktop/host/src/framework_trace.rs#L293) |
| struct Passes | [源码:294](../../../desktop/host/src/framework_trace.rs#L294) |
| impl impl eframe::egui::plugin::Plugin for Passes | [源码:295](../../../desktop/host/src/framework_trace.rs#L295) |
| fn debug_name | [源码:296](../../../desktop/host/src/framework_trace.rs#L296) |
| static str | [源码:296](../../../desktop/host/src/framework_trace.rs#L296) |
| fn output_hook | [源码:299](../../../desktop/host/src/framework_trace.rs#L299) |
| impl impl Drop for Scope | [源码:345](../../../desktop/host/src/framework_trace.rs#L345) |
| fn drop | [源码:346](../../../desktop/host/src/framework_trace.rs#L346) |
| mod event_loop | [源码:353](../../../desktop/host/src/framework_trace.rs#L353) |
| impl impl Trace | [源码:355](../../../desktop/host/src/framework_trace.rs#L355) |
| fn handler | [源码:356](../../../desktop/host/src/framework_trace.rs#L356) |
| struct Recorder | [源码:362](../../../desktop/host/src/framework_trace.rs#L362) |
| impl impl Subscriber for Recorder | [源码:363](../../../desktop/host/src/framework_trace.rs#L363) |
| fn enabled | [源码:364](../../../desktop/host/src/framework_trace.rs#L364) |
| fn new_span | [源码:367](../../../desktop/host/src/framework_trace.rs#L367) |
| fn record | [源码:376](../../../desktop/host/src/framework_trace.rs#L376) |
| fn record_follows_from | [源码:377](../../../desktop/host/src/framework_trace.rs#L377) |
| fn event | [源码:378](../../../desktop/host/src/framework_trace.rs#L378) |
| fn enter | [源码:379](../../../desktop/host/src/framework_trace.rs#L379) |
| fn exit | [源码:383](../../../desktop/host/src/framework_trace.rs#L383) |
| fn clone_span | [源码:387](../../../desktop/host/src/framework_trace.rs#L387) |
| fn try_close | [源码:398](../../../desktop/host/src/framework_trace.rs#L398) |
| fn current | [源码:402](../../../desktop/host/src/framework_trace.rs#L402) |
| fn run | [源码:405](../../../desktop/host/src/framework_trace.rs#L405) |
| mod tests | [源码:420](../../../desktop/host/src/framework_trace.rs#L420) |
| fn concrete_framework_labels_and_unknown_payloads_have_no_text_output | [源码:423](../../../desktop/host/src/framework_trace.rs#L423) |
| fn invalid_duplicate_reversed_and_closed_span_edges_remain_visible | [源码:463](../../../desktop/host/src/framework_trace.rs#L463) |
| fn recording_is_bounded_and_has_distinct_monotonic_and_epoch_clocks | [源码:476](../../../desktop/host/src/framework_trace.rs#L476) |
| fn thread_scoped_official_spans_and_clones_release_all_owners | [源码:490](../../../desktop/host/src/framework_trace.rs#L490) |
| fn fixed_text_buffer_rejects_overflow_without_growth | [源码:511](../../../desktop/host/src/framework_trace.rs#L511) |
| fn discarded_layout_passes_keep_frame_identity_and_source_without_reason_text | [源码:519](../../../desktop/host/src/framework_trace.rs#L519) |

## desktop/host/src/framework_trace/event_loop.rs

128 行；SHA256 `968b3037000249bf2b567d9fc5118a7b32c7849e6f425cb51df5211897cf0fea`

导入：`use super::Trace;`, `use winit::{ application::ApplicationHandler, event::{DeviceEvent, DeviceId, StartCause, WindowEvent}, event_loop::{ActiveEventLoop, EventLoop}, window::WindowId, };`, `use winit::platform::run_on_demand::EventLoopExtRunOnDemand;`, `use super::*;`, `use std::{ sync::{Arc, Mutex}, time::Duration, };`

| 定义 | 位置／签名 |
| --- | --- |
| struct Observed | [源码:10](../../../desktop/host/src/framework_trace/event_loop.rs#L10) |
| impl impl<T: ApplicationHandler<eframe::UserEvent>> ApplicationHandler<eframe::UserEvent> for Observed<T> | [源码:14](../../../desktop/host/src/framework_trace/event_loop.rs#L14) |
| fn new_events | [源码:17](../../../desktop/host/src/framework_trace/event_loop.rs#L17) |
| fn about_to_wait | [源码:21](../../../desktop/host/src/framework_trace/event_loop.rs#L21) |
| fn window_event | [源码:25](../../../desktop/host/src/framework_trace/event_loop.rs#L25) |
| fn user_event | [源码:29](../../../desktop/host/src/framework_trace/event_loop.rs#L29) |
| fn device_event | [源码:33](../../../desktop/host/src/framework_trace/event_loop.rs#L33) |
| fn resumed | [源码:37](../../../desktop/host/src/framework_trace/event_loop.rs#L37) |
| fn suspended | [源码:41](../../../desktop/host/src/framework_trace/event_loop.rs#L41) |
| fn exiting | [源码:45](../../../desktop/host/src/framework_trace/event_loop.rs#L45) |
| fn memory_warning | [源码:49](../../../desktop/host/src/framework_trace/event_loop.rs#L49) |
| fn run_native | [源码:54](../../../desktop/host/src/framework_trace/event_loop.rs#L54) |
| mod tests | [源码:72](../../../desktop/host/src/framework_trace/event_loop.rs#L72) |
| fn equal_repaint_delays_are_coalesced_but_earlier_delays_still_send_callbacks | [源码:75](../../../desktop/host/src/framework_trace/event_loop.rs#L75) |
| fn probe | [源码:80](../../../desktop/host/src/framework_trace/event_loop.rs#L80) |
| fn handler_boundaries_are_explicit_bounded_and_released | [源码:113](../../../desktop/host/src/framework_trace/event_loop.rs#L113) |

## desktop/host/src/gpu_observation.rs

230 行；SHA256 `2b7124cebeaddf47ea5efc8ebbb1379ed4a487cd67f8815a4f9f8e93b0d44539`

导入：`use eframe::{egui, egui_wgpu::wgpu};`, `use std::time::{Duration, Instant};`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| fn configuration | [源码:6](../../../desktop/host/src/gpu_observation.rs#L6) |
| fn apply_memory_hint | [源码:25](../../../desktop/host/src/gpu_observation.rs#L25) |
| fn snapshot | [源码:33](../../../desktop/host/src/gpu_observation.rs#L33) |
| fn encode | [源码:37](../../../desktop/host/src/gpu_observation.rs#L37) |
| fn baseline | [源码:83](../../../desktop/host/src/gpu_observation.rs#L83) |
| struct Baseline | [源码:89](../../../desktop/host/src/gpu_observation.rs#L89) |
| impl impl eframe::App for Baseline | [源码:97](../../../desktop/host/src/gpu_observation.rs#L97) |
| fn ui | [源码:98](../../../desktop/host/src/gpu_observation.rs#L98) |
| mod tests | [源码:177](../../../desktop/host/src/gpu_observation.rs#L177) |
| fn memory_policy_keeps_requested_device_features_and_limits | [源码:180](../../../desktop/host/src/gpu_observation.rs#L180) |
| fn diagnostic_memory_hint_preserves_native_presentation_policy | [源码:199](../../../desktop/host/src/gpu_observation.rs#L199) |
| fn absent_and_overflowing_reports_are_not_fabricated_as_zero | [源码:205](../../../desktop/host/src/gpu_observation.rs#L205) |

## desktop/host/src/gpu_path.rs

475 行；SHA256 `e98e617aa59cd215a09c35820ff37a814a37d7985cd9b4e172ac528c8b4ffcf7`

导入：`use eframe::{ egui, egui_wgpu::{self, wgpu}, };`, `use std::{ sync::{ Arc, atomic::{AtomicU64, AtomicUsize, Ordering}, }, time::{Duration, Instant}, };`, `use super::*;`, `use egui::plugin::Plugin;`

| 定义 | 位置／签名 |
| --- | --- |
| const FRAMES | [源码:16](../../../desktop/host/src/gpu_path.rs#L16) |
| const UPLOADS | [源码:17](../../../desktop/host/src/gpu_path.rs#L17) |
| const NONE | [源码:18](../../../desktop/host/src/gpu_path.rs#L18) |
| const ORDINALS | [源码:19](../../../desktop/host/src/gpu_path.rs#L19) |
| enum Mode | [源码:21](../../../desktop/host/src/gpu_path.rs#L21) |
| struct Trace | [源码:25](../../../desktop/host/src/gpu_path.rs#L25) |
| impl impl Trace | [源码:32](../../../desktop/host/src/gpu_path.rs#L32) |
| fn new | [源码:33](../../../desktop/host/src/gpu_path.rs#L33) |
| fn now | [源码:44](../../../desktop/host/src/gpu_path.rs#L44) |
| fn set | [源码:47](../../../desktop/host/src/gpu_path.rs#L47) |
| fn stamp | [源码:58](../../../desktop/host/src/gpu_path.rs#L58) |
| fn values | [源码:61](../../../desktop/host/src/gpu_path.rs#L61) |
| struct Marker | [源码:68](../../../desktop/host/src/gpu_path.rs#L68) |
| impl impl egui_wgpu::CallbackTrait for Marker | [源码:73](../../../desktop/host/src/gpu_path.rs#L73) |
| fn prepare | [源码:74](../../../desktop/host/src/gpu_path.rs#L74) |
| fn finish_prepare | [源码:87](../../../desktop/host/src/gpu_path.rs#L87) |
| fn paint | [源码:104](../../../desktop/host/src/gpu_path.rs#L104) |
| fn bracket | [源码:113](../../../desktop/host/src/gpu_path.rs#L113) |
| fn admit | [源码:129](../../../desktop/host/src/gpu_path.rs#L129) |
| fn walk | [源码:130](../../../desktop/host/src/gpu_path.rs#L130) |
| struct Upload | [源码:144](../../../desktop/host/src/gpu_path.rs#L144) |
| struct Probe | [源码:152](../../../desktop/host/src/gpu_path.rs#L152) |
| impl impl Probe | [源码:164](../../../desktop/host/src/gpu_path.rs#L164) |
| fn new | [源码:165](../../../desktop/host/src/gpu_path.rs#L165) |
| fn after_tessellation | [源码:197](../../../desktop/host/src/gpu_path.rs#L197) |
| fn upload | [源码:203](../../../desktop/host/src/gpu_path.rs#L203) |
| fn finish | [源码:289](../../../desktop/host/src/gpu_path.rs#L289) |
| fn report | [源码:315](../../../desktop/host/src/gpu_path.rs#L315) |
| impl impl egui::plugin::Plugin for Probe | [源码:353](../../../desktop/host/src/gpu_path.rs#L353) |
| fn debug_name | [源码:354](../../../desktop/host/src/gpu_path.rs#L354) |
| static str | [源码:354](../../../desktop/host/src/gpu_path.rs#L354) |
| fn output_hook | [源码:357](../../../desktop/host/src/gpu_path.rs#L357) |
| mod tests | [源码:397](../../../desktop/host/src/gpu_path.rs#L397) |
| fn bounded_slots_and_duplicate_edges_fail_visibly | [源码:400](../../../desktop/host/src/gpu_path.rs#L400) |
| fn late_completion_keeps_original_slot_and_owner | [源码:412](../../../desktop/host/src/gpu_path.rs#L412) |
| fn marker_bracket_preserves_original_shapes_textures_and_order | [源码:427](../../../desktop/host/src/gpu_path.rs#L427) |
| fn inactive_and_released_probes_cannot_create_markers | [源码:447](../../../desktop/host/src/gpu_path.rs#L447) |
| fn source_recursion_and_primitive_count_are_bounded | [源码:461](../../../desktop/host/src/gpu_path.rs#L461) |

## desktop/host/src/graphics_faults.rs

329 行；SHA256 `fa802f4d7d69d013dfd587be9871288d04235c861e69b921726d2c5f82db1755`

导入：`use crate::{NativeHost, fresh_session, sample, verification};`, `use eframe::{egui, egui_wgpu::wgpu};`, `use skin_editor_application::*;`, `use skin_editor_infrastructure::Images;`, `use skin_editor_rendering::SourceUpload;`, `use skin_editor_ui::{EditorUi, Presenter};`, `use std::{path::Path, sync::mpsc, time::Duration};`

| 定义 | 位置／签名 |
| --- | --- |
| type StageObserver | [源码:10](../../../desktop/host/src/graphics_faults.rs#L10) |
| fn error | [源码:11](../../../desktop/host/src/graphics_faults.rs#L11) |
| impl impl std::fmt::Debug) -> String | [源码:11](../../../desktop/host/src/graphics_faults.rs#L11) |
| fn wait | [源码:14](../../../desktop/host/src/graphics_faults.rs#L14) |
| fn descriptor | [源码:23](../../../desktop/host/src/graphics_faults.rs#L23) |
| fn run | [源码:28](../../../desktop/host/src/graphics_faults.rs#L28) |
| fn run_observed | [源码:31](../../../desktop/host/src/graphics_faults.rs#L31) |
| mod tests | [源码:312](../../../desktop/host/src/graphics_faults.rs#L312) |
| fn both_incarnations_keep_defaults_except_explicit_memory_hint | [源码:314](../../../desktop/host/src/graphics_faults.rs#L314) |

## desktop/host/src/gui_baseline.rs

225 行；SHA256 `53f25c94cdf1b50f6b8f1d922866dcfcf00e86df1f3684e2170290a279bb11f1`

导入：`use eframe::egui;`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| enum Mode | [源码:6](../../../desktop/host/src/gui_baseline.rs#L6) |
| impl impl Mode | [源码:11](../../../desktop/host/src/gui_baseline.rs#L11) |
| fn from_flag | [源码:12](../../../desktop/host/src/gui_baseline.rs#L12) |
| fn name | [源码:20](../../../desktop/host/src/gui_baseline.rs#L20) |
| static str | [源码:20](../../../desktop/host/src/gui_baseline.rs#L20) |
| fn full_ui | [源码:27](../../../desktop/host/src/gui_baseline.rs#L27) |
| fn viewport | [源码:32](../../../desktop/host/src/gui_baseline.rs#L32) |
| struct Geometry | [源码:44](../../../desktop/host/src/gui_baseline.rs#L44) |
| impl impl Geometry | [源码:51](../../../desktop/host/src/gui_baseline.rs#L51) |
| fn new | [源码:52](../../../desktop/host/src/gui_baseline.rs#L52) |
| static str | [源码:58](../../../desktop/host/src/gui_baseline.rs#L58) |
| fn rect | [源码:83](../../../desktop/host/src/gui_baseline.rs#L83) |
| fn report | [源码:89](../../../desktop/host/src/gui_baseline.rs#L89) |
| struct Control | [源码:97](../../../desktop/host/src/gui_baseline.rs#L97) |
| impl impl Control | [源码:104](../../../desktop/host/src/gui_baseline.rs#L104) |
| fn new | [源码:105](../../../desktop/host/src/gui_baseline.rs#L105) |
| fn observe | [源码:113](../../../desktop/host/src/gui_baseline.rs#L113) |
| static str | [源码:113](../../../desktop/host/src/gui_baseline.rs#L113) |
| fn record | [源码:121](../../../desktop/host/src/gui_baseline.rs#L121) |
| fn report | [源码:128](../../../desktop/host/src/gui_baseline.rs#L128) |
| mod tests | [源码:141](../../../desktop/host/src/gui_baseline.rs#L141) |
| fn geometry | [源码:143](../../../desktop/host/src/gui_baseline.rs#L143) |
| fn diagnostic_modes_do_not_activate_on_normal_flags_and_prime_same_layout | [源码:154](../../../desktop/host/src/gui_baseline.rs#L154) |
| fn only_explicit_controls_request_inactive_creation_without_changing_normal_windows | [源码:163](../../../desktop/host/src/gui_baseline.rs#L163) |
| fn viewport_scale_size_and_generation_changes_are_rejected_not_normalized | [源码:174](../../../desktop/host/src/gui_baseline.rs#L174) |
| fn no_draw_editor_and_minimal_counts_remain_separate | [源码:193](../../../desktop/host/src/gui_baseline.rs#L193) |
| fn invalid_or_outside_geometry_cannot_become_a_control_baseline | [源码:202](../../../desktop/host/src/gui_baseline.rs#L202) |

## desktop/host/src/import_worker.rs

202 行；SHA256 `3de9d526bb6054b7f5509fe19acb7fc3ea480a2dc2f389450414ab3d86f2e657`

导入：`use eframe::egui;`, `use skin_editor_application::{BlobLease, RasterImportPort, values::*};`, `use skin_editor_infrastructure::Images;`, `use std::sync::Arc;`, `use std::sync::mpsc::{self, Receiver, SyncSender};`, `use std::thread::JoinHandle;`, `use super::*;`, `use std::io::Write;`

| 定义 | 位置／签名 |
| --- | --- |
| struct ImportWorker | [源码:9](../../../desktop/host/src/import_worker.rs#L9) |
| enum ImportRequest | [源码:15](../../../desktop/host/src/import_worker.rs#L15) |
| struct ImportedAsset | [源码:20](../../../desktop/host/src/import_worker.rs#L20) |
| fn stage | [源码:26](../../../desktop/host/src/import_worker.rs#L26) |
| impl impl ImportWorker | [源码:86](../../../desktop/host/src/import_worker.rs#L86) |
| fn new | [源码:87](../../../desktop/host/src/import_worker.rs#L87) |
| fn enqueue | [源码:109](../../../desktop/host/src/import_worker.rs#L109) |
| fn enqueue_request | [源码:112](../../../desktop/host/src/import_worker.rs#L112) |
| fn drain | [源码:119](../../../desktop/host/src/import_worker.rs#L119) |
| impl impl Iterator<Item = Result<ImportedAsset, String>> + '_ | [源码:119](../../../desktop/host/src/import_worker.rs#L119) |
| fn close | [源码:122](../../../desktop/host/src/import_worker.rs#L122) |
| fn closed | [源码:129](../../../desktop/host/src/import_worker.rs#L129) |
| impl impl Drop for ImportWorker | [源码:133](../../../desktop/host/src/import_worker.rs#L133) |
| fn drop | [源码:134](../../../desktop/host/src/import_worker.rs#L134) |
| mod tests | [源码:140](../../../desktop/host/src/import_worker.rs#L140) |
| fn full_result_and_request_queues_can_be_closed_without_waiting_for_a_consumer | [源码:143](../../../desktop/host/src/import_worker.rs#L143) |
| fn wav_sample_is_attested_and_shares_the_image_raw_store | [源码:158](../../../desktop/host/src/import_worker.rs#L158) |
| fn unsupported_wav_requires_explicit_original_preservation_and_does_not_change_file | [源码:169](../../../desktop/host/src/import_worker.rs#L169) |

## desktop/host/src/load_verification.rs

409 行；SHA256 `027cd28ab97469a397d00e1303edb3645866122afa34e4191b136fd6fba4eac4`

导入：`use crate::audio_verification::submit;`, `use skin_editor_application::{values::*, *};`, `use skin_editor_audio::{Channel, PcmBudget};`, `use skin_editor_infrastructure::{ AudioSupplyClient, BlobStore, NativeOutput, OutputControl, SupplyOutcome, WavDecoder, };`, `use std::io::Write;`, `use std::time::{Duration, Instant};`

| 定义 | 位置／签名 |
| --- | --- |
| fn phase | [源码:11](../../../desktop/host/src/load_verification.rs#L11) |
| fn wait | [源码:15](../../../desktop/host/src/load_verification.rs#L15) |
| impl impl FnMut() -> bool, ) -> Result<(), String> | [源码:17](../../../desktop/host/src/load_verification.rs#L17) |
| fn outcome | [源码:28](../../../desktop/host/src/load_verification.rs#L28) |
| fn control | [源码:40](../../../desktop/host/src/load_verification.rs#L40) |
| fn startup | [源码:49](../../../desktop/host/src/load_verification.rs#L49) |
| fn stop | [源码:115](../../../desktop/host/src/load_verification.rs#L115) |
| fn consume | [源码:127](../../../desktop/host/src/load_verification.rs#L127) |
| fn wav | [源码:136](../../../desktop/host/src/load_verification.rs#L136) |
| fn run | [源码:162](../../../desktop/host/src/load_verification.rs#L162) |

## desktop/host/src/main.rs

712 行；SHA256 `360fb8a8e5d7f82b482213caf2313a0e757b72ca6bf8309a679f7521047040ae`

导入：`use presentation::NativeHost;`, `use skin_editor_application::{values::*, *};`, `use skin_editor_infrastructure::Images;`, `use skin_editor_preview::{PreviewState, Scene};`, `use skin_editor_ui::EditorUi;`, `use std::time::Instant;`

| 定义 | 位置／签名 |
| --- | --- |
| mod audio_verification | [源码:2](../../../desktop/host/src/main.rs#L2) |
| mod combined_verification | [源码:3](../../../desktop/host/src/main.rs#L3) |
| mod edit_verification | [源码:4](../../../desktop/host/src/main.rs#L4) |
| mod editor_audio | [源码:5](../../../desktop/host/src/main.rs#L5) |
| mod frame_worker | [源码:6](../../../desktop/host/src/main.rs#L6) |
| mod framework_trace | [源码:8](../../../desktop/host/src/main.rs#L8) |
| mod gpu_observation | [源码:9](../../../desktop/host/src/main.rs#L9) |
| mod gpu_path | [源码:10](../../../desktop/host/src/main.rs#L10) |
| mod graphics_faults | [源码:11](../../../desktop/host/src/main.rs#L11) |
| mod gui_baseline | [源码:12](../../../desktop/host/src/main.rs#L12) |
| mod import_worker | [源码:13](../../../desktop/host/src/main.rs#L13) |
| mod load_verification | [源码:14](../../../desktop/host/src/main.rs#L14) |
| mod main_work | [源码:15](../../../desktop/host/src/main.rs#L15) |
| mod mesh_replay | [源码:16](../../../desktop/host/src/main.rs#L16) |
| mod presentation | [源码:17](../../../desktop/host/src/main.rs#L17) |
| mod product_policy | [源码:18](../../../desktop/host/src/main.rs#L18) |
| mod rebuild_stages | [源码:19](../../../desktop/host/src/main.rs#L19) |
| mod repaint_deadline | [源码:20](../../../desktop/host/src/main.rs#L20) |
| mod repaint_observation | [源码:21](../../../desktop/host/src/main.rs#L21) |
| mod separated_verification | [源码:22](../../../desktop/host/src/main.rs#L22) |
| mod stable_window | [源码:23](../../../desktop/host/src/main.rs#L23) |
| mod stream_fixture | [源码:24](../../../desktop/host/src/main.rs#L24) |
| mod supply_verification | [源码:25](../../../desktop/host/src/main.rs#L25) |
| mod text_work | [源码:26](../../../desktop/host/src/main.rs#L26) |
| mod verification | [源码:27](../../../desktop/host/src/main.rs#L27) |
| mod worker_verification | [源码:28](../../../desktop/host/src/main.rs#L28) |
| struct Editor | [源码:36](../../../desktop/host/src/main.rs#L36) |
| fn fresh_session | [源码:50](../../../desktop/host/src/main.rs#L50) |
| fn import | [源码:61](../../../desktop/host/src/main.rs#L61) |
| fn sample | [源码:78](../../../desktop/host/src/main.rs#L78) |
| impl impl Editor | [源码:88](../../../desktop/host/src/main.rs#L88) |
| fn new | [源码:89](../../../desktop/host/src/main.rs#L89) |
| fn with_upload | [源码:96](../../../desktop/host/src/main.rs#L96) |
| fn with_render_state | [源码:107](../../../desktop/host/src/main.rs#L107) |
| fn with_initial_content | [源码:115](../../../desktop/host/src/main.rs#L115) |
| fn work_edge | [源码:155](../../../desktop/host/src/main.rs#L155) |
| impl impl eframe::App for Editor | [源码:161](../../../desktop/host/src/main.rs#L161) |
| fn logic | [源码:162](../../../desktop/host/src/main.rs#L162) |
| fn ui | [源码:207](../../../desktop/host/src/main.rs#L207) |
| fn on_exit | [源码:253](../../../desktop/host/src/main.rs#L253) |
| impl impl Editor | [源码:266](../../../desktop/host/src/main.rs#L266) |
| fn interrupt_for_fault | [源码:267](../../../desktop/host/src/main.rs#L267) |
| fn complete_ui | [源码:292](../../../desktop/host/src/main.rs#L292) |
| fn main | [源码:378](../../../desktop/host/src/main.rs#L378) |

## desktop/host/src/main_work.rs

159 行；SHA256 `6804c2244f7c05523a8568c42dc1b9f571dffcb109f93a41ae4e853833609021`

导入：`use std::time::{Instant, SystemTime, UNIX_EPOCH};`, `use skin_editor_ui::UiPhase;`, `use super::*;`, `use skin_editor_ui::UiPhase;`

| 定义 | 位置／签名 |
| --- | --- |
| const FRAMEWORK | [源码:5](../../../desktop/host/src/main_work.rs#L5) |
| const LOGIC | [源码:6](../../../desktop/host/src/main_work.rs#L6) |
| const UI | [源码:7](../../../desktop/host/src/main_work.rs#L7) |
| const DIAGNOSTIC | [源码:8](../../../desktop/host/src/main_work.rs#L8) |
| const PRESENT_AND_EVENTS | [源码:9](../../../desktop/host/src/main_work.rs#L9) |
| const WIDGETS | [源码:10](../../../desktop/host/src/main_work.rs#L10) |
| const GRAPHICS_POLL | [源码:11](../../../desktop/host/src/main_work.rs#L11) |
| const PREVIEW_REQUEST | [源码:12](../../../desktop/host/src/main_work.rs#L12) |
| const UI_DISPATCH | [源码:13](../../../desktop/host/src/main_work.rs#L13) |
| const LIMIT | [源码:14](../../../desktop/host/src/main_work.rs#L14) |
| fn ui_category | [源码:16](../../../desktop/host/src/main_work.rs#L16) |
| struct MainWork | [源码:39](../../../desktop/host/src/main_work.rs#L39) |
| impl impl MainWork | [源码:45](../../../desktop/host/src/main_work.rs#L45) |
| fn new | [源码:46](../../../desktop/host/src/main_work.rs#L46) |
| fn edge | [源码:53](../../../desktop/host/src/main_work.rs#L53) |
| fn record | [源码:70](../../../desktop/host/src/main_work.rs#L70) |
| fn report | [源码:83](../../../desktop/host/src/main_work.rs#L83) |
| mod tests | [源码:91](../../../desktop/host/src/main_work.rs#L91) |
| fn boundaries_remain_ordered_and_reject_invalid_clock_or_category | [源码:94](../../../desktop/host/src/main_work.rs#L94) |
| fn detailed_boundaries_share_one_bounded_ordered_partition | [源码:105](../../../desktop/host/src/main_work.rs#L105) |
| fn ui_boundaries_cannot_alias_framework_or_host_dispatch_categories | [源码:125](../../../desktop/host/src/main_work.rs#L125) |
| fn capacity_is_fixed_and_overflow_is_visible | [源码:151](../../../desktop/host/src/main_work.rs#L151) |

## desktop/host/src/mesh_replay.rs

533 行；SHA256 `a9da791430adac35d129b9b13b69e1eca2c6cb0d086bcd1624bebb00625919c0`

导入：`use eframe::egui::{ self, epaint::{ClippedPrimitive, ClippedShape, Primitive, Shape, Vertex}, };`, `use std::{hint::black_box, mem::size_of, time::Instant};`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| const VERTICES | [源码:9](../../../desktop/host/src/mesh_replay.rs#L9) |
| const INDICES | [源码:10](../../../desktop/host/src/mesh_replay.rs#L10) |
| const NODES | [源码:11](../../../desktop/host/src/mesh_replay.rs#L11) |
| const ATLAS_BYTES | [源码:12](../../../desktop/host/src/mesh_replay.rs#L12) |
| const ORDINALS | [源码:13](../../../desktop/host/src/mesh_replay.rs#L13) |
| const PLAN | [源码:14](../../../desktop/host/src/mesh_replay.rs#L14) |
| const REPEATS | [源码:15](../../../desktop/host/src/mesh_replay.rs#L15) |
| fn admit_atlas | [源码:16](../../../desktop/host/src/mesh_replay.rs#L16) |
| struct Counts | [源码:26](../../../desktop/host/src/mesh_replay.rs#L26) |
| fn admit | [源码:32](../../../desktop/host/src/mesh_replay.rs#L32) |
| static str | [源码:32](../../../desktop/host/src/mesh_replay.rs#L32) |
| fn select | [源码:62](../../../desktop/host/src/mesh_replay.rs#L62) |
| fn partition | [源码:74](../../../desktop/host/src/mesh_replay.rs#L74) |
| static str | [源码:74](../../../desktop/host/src/mesh_replay.rs#L74) |
| fn counts | [源码:96](../../../desktop/host/src/mesh_replay.rs#L96) |
| static str | [源码:96](../../../desktop/host/src/mesh_replay.rs#L96) |
| fn equal | [源码:112](../../../desktop/host/src/mesh_replay.rs#L112) |
| struct Packed | [源码:125](../../../desktop/host/src/mesh_replay.rs#L125) |
| impl impl Packed | [源码:129](../../../desktop/host/src/mesh_replay.rs#L129) |
| fn new | [源码:130](../../../desktop/host/src/mesh_replay.rs#L130) |
| fn copy | [源码:136](../../../desktop/host/src/mesh_replay.rs#L136) |
| static str | [源码:136](../../../desktop/host/src/mesh_replay.rs#L136) |
| struct Replay | [源码:154](../../../desktop/host/src/mesh_replay.rs#L154) |
| static str | [源码:162](../../../desktop/host/src/mesh_replay.rs#L162) |
| impl impl Replay | [源码:164](../../../desktop/host/src/mesh_replay.rs#L164) |
| fn new | [源码:165](../../../desktop/host/src/mesh_replay.rs#L165) |
| fn capture | [源码:177](../../../desktop/host/src/mesh_replay.rs#L177) |
| static str | [源码:182](../../../desktop/host/src/mesh_replay.rs#L182) |
| static str | [源码:240](../../../desktop/host/src/mesh_replay.rs#L240) |
| static str | [源码:245](../../../desktop/host/src/mesh_replay.rs#L245) |
| fn report | [源码:308](../../../desktop/host/src/mesh_replay.rs#L308) |
| impl impl egui::plugin::Plugin for Replay | [源码:322](../../../desktop/host/src/mesh_replay.rs#L322) |
| fn debug_name | [源码:323](../../../desktop/host/src/mesh_replay.rs#L323) |
| static str | [源码:323](../../../desktop/host/src/mesh_replay.rs#L323) |
| fn output_hook | [源码:326](../../../desktop/host/src/mesh_replay.rs#L326) |
| mod tests | [源码:351](../../../desktop/host/src/mesh_replay.rs#L351) |
| fn frame | [源码:353](../../../desktop/host/src/mesh_replay.rs#L353) |
| fn partition_preserves_nested_clip_transform_and_relative_order | [源码:366](../../../desktop/host/src/mesh_replay.rs#L366) |
| fn same_output_replay_matches_framework_and_keeps_shapes_and_texture_updates | [源码:400](../../../desktop/host/src/mesh_replay.rs#L400) |
| fn packing_keeps_mesh_local_indices_and_reuses_only_capacity | [源码:434](../../../desktop/host/src/mesh_replay.rs#L434) |
| fn admission_rejects_deep_or_large_sources_and_atlas_before_copy | [源码:469](../../../desktop/host/src/mesh_replay.rs#L469) |
| fn callbacks_are_forwarded_without_invocation_or_retention | [源码:483](../../../desktop/host/src/mesh_replay.rs#L483) |
| fn fractional_scale_clip_opacity_and_transform_match_fresh_framework_output | [源码:506](../../../desktop/host/src/mesh_replay.rs#L506) |

## desktop/host/src/presentation.rs

935 行；SHA256 `ccc5f167c8ab723d0d651dd4f2a9ba95bfd37aae04659f0e9b358fb742d7fe3c`

导入：`use eframe::{ egui, egui_wgpu::{self, wgpu}, };`, `use skin_editor_application::{values::*, *};`, `use skin_editor_infrastructure::Images;`, `use skin_editor_rendering::{GpuComposer, GpuImage, Plan, Target};`, `use skin_editor_ui::{PresentedView, Presenter};`, `use std::collections::{BTreeMap, BTreeSet};`, `use std::sync::{ Arc, OnceLock, Weak, atomic::{AtomicBool, Ordering}, };`, `use std::time::Instant;`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| struct PreparationTrace | [源码:16](../../../desktop/host/src/presentation.rs#L16) |
| struct NativeTarget | [源码:31](../../../desktop/host/src/presentation.rs#L31) |
| struct Frame | [源码:36](../../../desktop/host/src/presentation.rs#L36) |
| struct Prepared | [源码:43](../../../desktop/host/src/presentation.rs#L43) |
| struct DeviceLoss | [源码:51](../../../desktop/host/src/presentation.rs#L51) |
| impl impl DeviceLoss | [源码:52](../../../desktop/host/src/presentation.rs#L52) |
| fn report | [源码:53](../../../desktop/host/src/presentation.rs#L53) |
| fn is_lost | [源码:56](../../../desktop/host/src/presentation.rs#L56) |
| struct NativeHost | [源码:60](../../../desktop/host/src/presentation.rs#L60) |
| struct RebuildSeed | [源码:90](../../../desktop/host/src/presentation.rs#L90) |
| impl impl RebuildSeed | [源码:96](../../../desktop/host/src/presentation.rs#L96) |
| fn build | [源码:97](../../../desktop/host/src/presentation.rs#L97) |
| fn build_retained | [源码:108](../../../desktop/host/src/presentation.rs#L108) |
| fn material_equal | [源码:122](../../../desktop/host/src/presentation.rs#L122) |
| fn candidate_current | [源码:130](../../../desktop/host/src/presentation.rs#L130) |
| fn ready_descriptor | [源码:137](../../../desktop/host/src/presentation.rs#L137) |
| impl impl NativeHost | [源码:149](../../../desktop/host/src/presentation.rs#L149) |
| fn bind_loss | [源码:152](../../../desktop/host/src/presentation.rs#L152) |
| fn with_upload | [源码:162](../../../desktop/host/src/presentation.rs#L162) |
| fn enable_trace | [源码:287](../../../desktop/host/src/presentation.rs#L287) |
| fn retire | [源码:294](../../../desktop/host/src/presentation.rs#L294) |
| fn wait_preparation | [源码:300](../../../desktop/host/src/presentation.rs#L300) |
| fn poll | [源码:303](../../../desktop/host/src/presentation.rs#L303) |
| fn poll_live | [源码:319](../../../desktop/host/src/presentation.rs#L319) |
| fn loss_signal | [源码:367](../../../desktop/host/src/presentation.rs#L367) |
| fn is_lost | [源码:370](../../../desktop/host/src/presentation.rs#L370) |
| fn invalidate_source | [源码:375](../../../desktop/host/src/presentation.rs#L375) |
| fn clean_lost_device | [源码:387](../../../desktop/host/src/presentation.rs#L387) |
| fn into_rebuild_seed | [源码:416](../../../desktop/host/src/presentation.rs#L416) |
| fn needs_work | [源码:433](../../../desktop/host/src/presentation.rs#L433) |
| fn preparation_closed | [源码:436](../../../desktop/host/src/presentation.rs#L436) |
| fn preparation_pending | [源码:439](../../../desktop/host/src/presentation.rs#L439) |
| fn shutdown_succeeded | [源码:442](../../../desktop/host/src/presentation.rs#L442) |
| fn shutdown | [源码:445](../../../desktop/host/src/presentation.rs#L445) |
| fn active_texture | [源码:473](../../../desktop/host/src/presentation.rs#L473) |
| fn source_usage | [源码:479](../../../desktop/host/src/presentation.rs#L479) |
| fn staging_usage | [源码:482](../../../desktop/host/src/presentation.rs#L482) |
| fn source_allocations | [源码:485](../../../desktop/host/src/presentation.rs#L485) |
| fn live_targets | [源码:488](../../../desktop/host/src/presentation.rs#L488) |
| impl impl PreviewPort for NativeHost | [源码:500](../../../desktop/host/src/presentation.rs#L500) |
| fn request | [源码:501](../../../desktop/host/src/presentation.rs#L501) |
| impl impl NativeHost | [源码:595](../../../desktop/host/src/presentation.rs#L595) |
| fn accept_prepared | [源码:596](../../../desktop/host/src/presentation.rs#L596) |
| struct Draw | [源码:688](../../../desktop/host/src/presentation.rs#L688) |
| impl impl egui_wgpu::CallbackTrait for Draw | [源码:695](../../../desktop/host/src/presentation.rs#L695) |
| fn prepare | [源码:696](../../../desktop/host/src/presentation.rs#L696) |
| fn paint | [源码:720](../../../desktop/host/src/presentation.rs#L720) |
| impl impl Presenter for NativeHost | [源码:734](../../../desktop/host/src/presentation.rs#L734) |
| fn current | [源码:735](../../../desktop/host/src/presentation.rs#L735) |
| fn paint | [源码:745](../../../desktop/host/src/presentation.rs#L745) |
| fn thumbnail | [源码:762](../../../desktop/host/src/presentation.rs#L762) |
| const DISPLAY | [源码:808](../../../desktop/host/src/presentation.rs#L808) |
| mod tests | [源码:823](../../../desktop/host/src/presentation.rs#L823) |
| fn timed_candidate_publication_uses_current_descriptor_without_discarding_identical_material | [源码:826](../../../desktop/host/src/presentation.rs#L826) |
| fn time_alias_cannot_hide_a_project_draft_or_viewport_change | [源码:885](../../../desktop/host/src/presentation.rs#L885) |

## desktop/host/src/product_policy.rs

67 行；SHA256 `ca8e4332dc4716557112c713951faeeda4ce9ec6276c811b81f0536a6066be7e`

导入：`use crate::editor_audio::EditorAudio;`, `use skin_editor_infrastructure::BlobStore;`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| struct Policy | [源码:6](../../../desktop/host/src/product_policy.rs#L6) |
| impl impl Policy | [源码:10](../../../desktop/host/src/product_policy.rs#L10) |
| fn for_windows | [源码:11](../../../desktop/host/src/product_policy.rs#L11) |
| const CURRENT | [源码:17](../../../desktop/host/src/product_policy.rs#L17) |
| fn audio | [源码:19](../../../desktop/host/src/product_policy.rs#L19) |
| fn graphics | [源码:26](../../../desktop/host/src/product_policy.rs#L26) |
| mod tests | [源码:31](../../../desktop/host/src/product_policy.rs#L31) |
| fn only_the_validated_platform_uses_the_separated_route | [源码:34](../../../desktop/host/src/product_policy.rs#L34) |
| fn ordinary_assembly_changes_only_the_allocator_hint | [源码:55](../../../desktop/host/src/product_policy.rs#L55) |

## desktop/host/src/rebuild_stages.rs

73 行；SHA256 `c268dd0e22e2865cc484f0e67ecc6f237e2536aeac1c9894aa64554356463f6d`

导入：`use eframe::egui_wgpu::wgpu;`, `use std::{ io::Write, path::Path, time::{Duration, Instant}, };`

| 定义 | 位置／签名 |
| --- | --- |
| fn ack_matches | [源码:9](../../../desktop/host/src/rebuild_stages.rs#L9) |
| fn run | [源码:13](../../../desktop/host/src/rebuild_stages.rs#L13) |
| mod tests | [源码:59](../../../desktop/host/src/rebuild_stages.rs#L59) |
| fn checkpoint_ack_is_exact_and_cannot_replay_another_sequence | [源码:61](../../../desktop/host/src/rebuild_stages.rs#L61) |
| fn stages_require_fresh_absolute_destination | [源码:68](../../../desktop/host/src/rebuild_stages.rs#L68) |

## desktop/host/src/repaint_deadline.rs

232 行；SHA256 `30d414e7b1f123fd722025d5328ea554aad8253a8d5768d440b46b7fdf147a2f`

导入：`use std::time::{Duration, Instant};`, `use super::*;`, `use eframe::egui;`, `use std::sync::{Arc, Mutex};`

| 定义 | 位置／签名 |
| --- | --- |
| struct RepaintPass | [源码:6](../../../desktop/host/src/repaint_deadline.rs#L6) |
| impl impl RepaintPass | [源码:10](../../../desktop/host/src/repaint_deadline.rs#L10) |
| fn begin | [源码:12](../../../desktop/host/src/repaint_deadline.rs#L12) |
| fn publish_at | [源码:16](../../../desktop/host/src/repaint_deadline.rs#L16) |
| struct RepaintCycle | [源码:30](../../../desktop/host/src/repaint_deadline.rs#L30) |
| impl impl RepaintCycle | [源码:34](../../../desktop/host/src/repaint_deadline.rs#L34) |
| fn new | [源码:35](../../../desktop/host/src/repaint_deadline.rs#L35) |
| fn after | [源码:41](../../../desktop/host/src/repaint_deadline.rs#L41) |
| fn offer_at | [源码:44](../../../desktop/host/src/repaint_deadline.rs#L44) |
| fn remaining_at | [源码:49](../../../desktop/host/src/repaint_deadline.rs#L49) |
| fn publish | [源码:52](../../../desktop/host/src/repaint_deadline.rs#L52) |
| mod tests | [源码:58](../../../desktop/host/src/repaint_deadline.rs#L58) |
| fn ms | [源码:62](../../../desktop/host/src/repaint_deadline.rs#L62) |
| fn selects_absolute_deadline_without_restarting_the_delay_at_publish | [源码:67](../../../desktop/host/src/repaint_deadline.rs#L67) |
| fn every_combination_keeps_the_original_earliest_policy_and_empty_stays_idle | [源码:78](../../../desktop/host/src/repaint_deadline.rs#L78) |
| fn mature | [源码:96](../../../desktop/host/src/repaint_deadline.rs#L96) |
| fn one_publication_reduces_actual_callbacks_without_replacing_external_callback | [源码:105](../../../desktop/host/src/repaint_deadline.rs#L105) |
| fn paused_logic_cannot_suppress_post_command_work_or_next_pass_resume | [源码:127](../../../desktop/host/src/repaint_deadline.rs#L127) |
| fn absolute_pass_guard_ignores_later_ui_demand_but_accepts_new_earlier_work | [源码:175](../../../desktop/host/src/repaint_deadline.rs#L175) |
| fn pass_advance_and_explicit_logic_restart_both_reset_old_deadlines | [源码:209](../../../desktop/host/src/repaint_deadline.rs#L209) |

## desktop/host/src/repaint_observation.rs

312 行；SHA256 `2e76ba14c4d259371fb1d1484a50de952ec3c643fb833642920fee7b264ed62e`

导入：`use eframe::egui;`, `use std::sync::{ Arc, atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering}, };`, `use std::time::{SystemTime, UNIX_EPOCH};`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| const REQUEST_LIMIT | [源码:10](../../../desktop/host/src/repaint_observation.rs#L10) |
| const PASS_LIMIT | [源码:11](../../../desktop/host/src/repaint_observation.rs#L11) |
| const SOURCE_LIMIT | [源码:12](../../../desktop/host/src/repaint_observation.rs#L12) |
| const CAUSES_PER_PASS | [源码:13](../../../desktop/host/src/repaint_observation.rs#L13) |
| fn utc | [源码:14](../../../desktop/host/src/repaint_observation.rs#L14) |
| struct Slot | [源码:21](../../../desktop/host/src/repaint_observation.rs#L21) |
| struct Requests | [源码:27](../../../desktop/host/src/repaint_observation.rs#L27) |
| impl impl Requests | [源码:31](../../../desktop/host/src/repaint_observation.rs#L31) |
| fn new | [源码:32](../../../desktop/host/src/repaint_observation.rs#L32) |
| fn push | [源码:45](../../../desktop/host/src/repaint_observation.rs#L45) |
| fn snapshot | [源码:54](../../../desktop/host/src/repaint_observation.rs#L54) |
| struct Source | [源码:72](../../../desktop/host/src/repaint_observation.rs#L72) |
| struct Pass | [源码:76](../../../desktop/host/src/repaint_observation.rs#L76) |
| struct Repaints | [源码:82](../../../desktop/host/src/repaint_observation.rs#L82) |
| fn source_name | [源码:89](../../../desktop/host/src/repaint_observation.rs#L89) |
| impl impl Repaints | [源码:101](../../../desktop/host/src/repaint_observation.rs#L101) |
| fn install | [源码:102](../../../desktop/host/src/repaint_observation.rs#L102) |
| fn source | [源码:120](../../../desktop/host/src/repaint_observation.rs#L120) |
| fn pass | [源码:136](../../../desktop/host/src/repaint_observation.rs#L136) |
| fn input | [源码:163](../../../desktop/host/src/repaint_observation.rs#L163) |
| fn report | [源码:185](../../../desktop/host/src/repaint_observation.rs#L185) |
| mod tests | [源码:217](../../../desktop/host/src/repaint_observation.rs#L217) |
| fn bounded_requests_preserve_delay_and_report_overflow | [源码:220](../../../desktop/host/src/repaint_observation.rs#L220) |
| fn unfinished_slot_is_reported_not_read_as_a_completed_request | [源码:227](../../../desktop/host/src/repaint_observation.rs#L227) |
| fn observer_preserves_framework_callback_and_observes_requests | [源码:233](../../../desktop/host/src/repaint_observation.rs#L233) |
| fn sources_are_bounded_and_remove_absolute_path_prefix | [源码:251](../../../desktop/host/src/repaint_observation.rs#L251) |
| fn input_diagnostic_counts_kinds_without_storing_input_values | [源码:265](../../../desktop/host/src/repaint_observation.rs#L265) |
| fn repeated_delayed_requests_do_not_multiply_scheduler_callbacks | [源码:279](../../../desktop/host/src/repaint_observation.rs#L279) |
| fn raw_mouse_event_requests_immediate_repaint_without_application_widgets | [源码:296](../../../desktop/host/src/repaint_observation.rs#L296) |

## desktop/host/src/separated_verification.rs

306 行；SHA256 `f74a963795c653dab876560a3c2fd271b012a2295f1de7dac1dd8e63ae26f2dc`

导入：`use skin_editor_application::values::{AudioComponentId, AudioMetadata, BlobRef, Gain};`, `use skin_editor_audio::{ BackgroundTrack, Channel, Origin, PcmBudget, PreparedSound, Runtime, SoundPlan, };`, `use skin_editor_infrastructure::{ separated_native::{NativeExecutor, PreparedDevice}, separated_output::{self, BackgroundOwner, Phase}, };`, `use std::io::Write;`, `use std::time::{Duration, Instant};`

| 定义 | 位置／签名 |
| --- | --- |
| fn origin | [源码:12](../../../desktop/host/src/separated_verification.rs#L12) |
| fn sound | [源码:21](../../../desktop/host/src/separated_verification.rs#L21) |
| fn trigger | [源码:44](../../../desktop/host/src/separated_verification.rs#L44) |
| fn tick | [源码:54](../../../desktop/host/src/separated_verification.rs#L54) |
| fn wait | [源码:69](../../../desktop/host/src/separated_verification.rs#L69) |
| impl impl FnMut(&Runtime, &BackgroundOwner) -> bool, ) -> Result<(), String> | [源码:73](../../../desktop/host/src/separated_verification.rs#L73) |
| fn duration | [源码:87](../../../desktop/host/src/separated_verification.rs#L87) |
| fn phase | [源码:96](../../../desktop/host/src/separated_verification.rs#L96) |
| fn run | [源码:100](../../../desktop/host/src/separated_verification.rs#L100) |

## desktop/host/src/stable_window.rs

104 行；SHA256 `9b86a0b221a7c2b168e14e4124e03540247a465916932a3933be532df648c1b6`

导入：`use std::time::Duration;`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| const SETTLE | [源码:4](../../../desktop/host/src/stable_window.rs#L4) |
| const MEASURE | [源码:5](../../../desktop/host/src/stable_window.rs#L5) |
| struct Interval | [源码:8](../../../desktop/host/src/stable_window.rs#L8) |
| struct Observation | [源码:15](../../../desktop/host/src/stable_window.rs#L15) |
| impl impl Interval | [源码:21](../../../desktop/host/src/stable_window.rs#L21) |
| fn begin | [源码:22](../../../desktop/host/src/stable_window.rs#L22) |
| static str | [源码:29](../../../desktop/host/src/stable_window.rs#L29) |
| fn finish | [源码:41](../../../desktop/host/src/stable_window.rs#L41) |
| static str | [源码:48](../../../desktop/host/src/stable_window.rs#L48) |
| mod tests | [源码:67](../../../desktop/host/src/stable_window.rs#L67) |
| fn seconds | [源码:69](../../../desktop/host/src/stable_window.rs#L69) |
| fn settling_is_required_and_deadline_is_inclusive | [源码:73](../../../desktop/host/src/stable_window.rs#L73) |
| fn full_window_is_required_without_cherry_picking_a_short_tail | [源码:79](../../../desktop/host/src/stable_window.rs#L79) |
| fn extra_draws_input_or_moving_clocks_remain_visible | [源码:89](../../../desktop/host/src/stable_window.rs#L89) |
| fn counter_reversals_are_not_saturated_to_zero | [源码:98](../../../desktop/host/src/stable_window.rs#L98) |

## desktop/host/src/stream_fixture.rs

61 行；SHA256 `3e98cdf75fbabd09bb75690363c8180c6ba703c9a18f33a446ffa6d9ee77839a`

导入：`use skin_editor_audio::PcmBudget;`, `use skin_editor_infrastructure::background_stream::{BufferedBackground, StreamObservation};`, `use std::{ fs::{File, OpenOptions}, io::Write, path::Path, };`

| 定义 | 位置／签名 |
| --- | --- |
| const SECONDS | [源码:9](../../../desktop/host/src/stream_fixture.rs#L9) |
| fn open | [源码:10](../../../desktop/host/src/stream_fixture.rs#L10) |
| fn write | [源码:22](../../../desktop/host/src/stream_fixture.rs#L22) |

## desktop/host/src/supply_verification.rs

291 行；SHA256 `50794c81837f82c9a80094975488948e1fb9e30b8eb99e7361168e357ddd5e15`

导入：`use crate::audio_verification::{component, submit, tone};`, `use skin_editor_application::{values::*, *};`, `use skin_editor_audio::{BackgroundTrack, Channel, PcmBudget};`, `use skin_editor_infrastructure::{ AudioSupplyClient, BlobStore, NativeOutput, SupplyOutcome, WavDecoder, };`, `use std::time::{Duration, Instant};`, `use std::io::Write;`

| 定义 | 位置／签名 |
| --- | --- |
| fn outcome | [源码:9](../../../desktop/host/src/supply_verification.rs#L9) |
| fn ready | [源码:26](../../../desktop/host/src/supply_verification.rs#L26) |
| fn trigger | [源码:58](../../../desktop/host/src/supply_verification.rs#L58) |
| fn draft | [源码:71](../../../desktop/host/src/supply_verification.rs#L71) |
| fn run | [源码:87](../../../desktop/host/src/supply_verification.rs#L87) |

## desktop/host/src/text_work.rs

304 行；SHA256 `9ed16d38ed2e3b19ce19b6d1316fd997c605d7239cbb221c458c56b2e6074c71`

导入：`use eframe::egui::{self, epaint::Shape};`, `use std::{sync::Weak, time::Instant};`, `use super::*;`, `use egui::epaint::text::{Fonts, TextOptions, ViewportKey};`, `use egui::epaint::text::{Fonts, TextOptions, ViewportKey};`

| 定义 | 位置／签名 |
| --- | --- |
| const SHAPES | [源码:6](../../../desktop/host/src/text_work.rs#L6) |
| const PASSES | [源码:7](../../../desktop/host/src/text_work.rs#L7) |
| struct Census | [源码:8](../../../desktop/host/src/text_work.rs#L8) |
| impl impl Census | [源码:15](../../../desktop/host/src/text_work.rs#L15) |
| fn new | [源码:16](../../../desktop/host/src/text_work.rs#L16) |
| fn visit | [源码:25](../../../desktop/host/src/text_work.rs#L25) |
| fn clear_identities | [源码:57](../../../desktop/host/src/text_work.rs#L57) |
| fn report | [源码:61](../../../desktop/host/src/text_work.rs#L61) |
| impl impl egui::plugin::Plugin for Census | [源码:70](../../../desktop/host/src/text_work.rs#L70) |
| fn debug_name | [源码:71](../../../desktop/host/src/text_work.rs#L71) |
| static str | [源码:71](../../../desktop/host/src/text_work.rs#L71) |
| fn output_hook | [源码:74](../../../desktop/host/src/text_work.rs#L74) |
| fn on_exit | [源码:115](../../../desktop/host/src/text_work.rs#L115) |
| mod tests | [源码:121](../../../desktop/host/src/text_work.rs#L121) |
| fn pass | [源码:123](../../../desktop/host/src/text_work.rs#L123) |
| fn framework_lookup_reuses_layout_and_rejects_changed_dpi_or_raster_options | [源码:129](../../../desktop/host/src/text_work.rs#L129) |
| fn atlas_reset_at_the_same_size_rejects_an_old_layout | [源码:155](../../../desktop/host/src/text_work.rs#L155) |
| fn survey_is_read_only_reuses_live_identities_and_releases_weak_references | [源码:207](../../../desktop/host/src/text_work.rs#L207) |
| fn survey_limits_and_weak_identity_cannot_retain_text_payload | [源码:234](../../../desktop/host/src/text_work.rs#L234) |
| fn identical_live_galley_does_not_authorize_old_geometry_or_color | [源码:264](../../../desktop/host/src/text_work.rs#L264) |
| fn replacing_font_definitions_invalidates_layout_without_a_text_change | [源码:284](../../../desktop/host/src/text_work.rs#L284) |

## desktop/host/src/verification.rs

1082 行；SHA256 `a8ba5cc9c13fe54bc0d269acd724bf2125100e72564dfb7055c854ebdb63f1dc`

导入：`use super::*;`, `use eframe::{ egui, egui_wgpu::{self, wgpu}, };`, `use skin_editor_infrastructure::write_png;`, `use skin_editor_rendering::{Plan, Target, compose};`, `use skin_editor_ui::{Presenter, Selection};`, `use std::path::Path;`, `use std::time::Duration;`, `use skin_editor_rendering::{GpuComposer, Raster};`, `use std::sync::Arc;`

| 定义 | 位置／签名 |
| --- | --- |
| fn wait | [源码:13](../../../desktop/host/src/verification.rs#L13) |
| fn frame | [源码:22](../../../desktop/host/src/verification.rs#L22) |
| fn readback | [源码:54](../../../desktop/host/src/verification.rs#L54) |
| fn readback_observed | [源码:61](../../../desktop/host/src/verification.rs#L61) |
| fn cpu_pixels | [源码:122](../../../desktop/host/src/verification.rs#L122) |
| fn compare | [源码:151](../../../desktop/host/src/verification.rs#L151) |
| fn verify_source_buffer_reuse | [源码:164](../../../desktop/host/src/verification.rs#L164) |
| fn ui_image | [源码:284](../../../desktop/host/src/verification.rs#L284) |
| fn run | [源码:398](../../../desktop/host/src/verification.rs#L398) |
| fn run_transform_cadence | [源码:405](../../../desktop/host/src/verification.rs#L405) |
| fn run_joint_cadence | [源码:408](../../../desktop/host/src/verification.rs#L408) |
| fn run_cadence | [源码:411](../../../desktop/host/src/verification.rs#L411) |
| fn pass | [源码:472](../../../desktop/host/src/verification.rs#L472) |
| fn run_memory | [源码:627](../../../desktop/host/src/verification.rs#L627) |
| fn run_markers | [源码:630](../../../desktop/host/src/verification.rs#L630) |
| fn run_with_audio | [源码:633](../../../desktop/host/src/verification.rs#L633) |
| fn run_staging | [源码:636](../../../desktop/host/src/verification.rs#L636) |
| fn run_mode | [源码:639](../../../desktop/host/src/verification.rs#L639) |

## desktop/host/src/worker_verification.rs

535 行；SHA256 `5114480b6dddfa58d244fa61c426e50503ef46c280cedbadc884b5f6f501976a`

导入：`use crate::audio_verification::{component, submit};`, `use crate::load_verification::phase;`, `use crate::supply_verification::draft;`, `use skin_editor_application::{values::*, *};`, `use skin_editor_audio::{BackgroundTrack, Channel, PcmBudget};`, `use skin_editor_infrastructure::{ AudioSupplyClient, BlobStore, SupplyOutcome, SupplyReceipt, WavDecoder, separated_native::{FirstCopy, NativeExecutor, PreparedDevice}, };`, `use std::time::{Duration, Instant};`

| 定义 | 位置／签名 |
| --- | --- |
| fn backend | [源码:12](../../../desktop/host/src/worker_verification.rs#L12) |
| impl impl std::fmt::Debug) -> String | [源码:12](../../../desktop/host/src/worker_verification.rs#L12) |
| fn wav | [源码:15](../../../desktop/host/src/worker_verification.rs#L15) |
| fn healthy | [源码:35](../../../desktop/host/src/worker_verification.rs#L35) |
| fn receipt | [源码:45](../../../desktop/host/src/worker_verification.rs#L45) |
| fn outcome | [源码:62](../../../desktop/host/src/worker_verification.rs#L62) |
| fn ready | [源码:65](../../../desktop/host/src/worker_verification.rs#L65) |
| fn barrier | [源码:86](../../../desktop/host/src/worker_verification.rs#L86) |
| fn stop | [源码:91](../../../desktop/host/src/worker_verification.rs#L91) |
| struct Probe | [源码:96](../../../desktop/host/src/worker_verification.rs#L96) |
| fn run | [源码:103](../../../desktop/host/src/worker_verification.rs#L103) |
| fn streamed | [源码:106](../../../desktop/host/src/worker_verification.rs#L106) |
| fn run_mode | [源码:109](../../../desktop/host/src/worker_verification.rs#L109) |
| fn prime | [源码:513](../../../desktop/host/src/worker_verification.rs#L513) |
| fn location_visible | [源码:523](../../../desktop/host/src/worker_verification.rs#L523) |

## desktop/infrastructure/src/audio_playback.rs

168 行；SHA256 `9850c36db4f8ac81e2235312623cd007e5825f27bcaee0ad8889b6e4bfa69b62`

导入：`use crate::{ BlobStore, BuildAdmission, OutputFence, OutputProducer, PreparationWorker, SkinAudioController, WavDecoder, };`, `use skin_editor_application::{ProjectSession, values::AudioComponentId};`, `use skin_editor_audio::{Channel, PcmBudget, TriggerReceipt};`

| 定义 | 位置／签名 |
| --- | --- |
| struct AudioPlayback | [源码:9](../../../desktop/infrastructure/src/audio_playback.rs#L9) |
| impl impl AudioPlayback | [源码:15](../../../desktop/infrastructure/src/audio_playback.rs#L15) |
| fn new | [源码:16](../../../desktop/infrastructure/src/audio_playback.rs#L16) |
| fn refresh | [源码:27](../../../desktop/infrastructure/src/audio_playback.rs#L27) |
| fn fence_if_changed | [源码:45](../../../desktop/infrastructure/src/audio_playback.rs#L45) |
| fn step | [源码:54](../../../desktop/infrastructure/src/audio_playback.rs#L54) |
| fn trigger | [源码:91](../../../desktop/infrastructure/src/audio_playback.rs#L91) |
| fn scene_running | [源码:104](../../../desktop/infrastructure/src/audio_playback.rs#L104) |
| fn stop_audition | [源码:115](../../../desktop/infrastructure/src/audio_playback.rs#L115) |
| fn status | [源码:122](../../../desktop/infrastructure/src/audio_playback.rs#L122) |
| fn wait_preparation | [源码:129](../../../desktop/infrastructure/src/audio_playback.rs#L129) |
| fn scene_frame | [源码:145](../../../desktop/infrastructure/src/audio_playback.rs#L145) |
| fn prefill | [源码:149](../../../desktop/infrastructure/src/audio_playback.rs#L149) |
| fn close | [源码:153](../../../desktop/infrastructure/src/audio_playback.rs#L153) |
| impl impl Drop for AudioPlayback | [源码:160](../../../desktop/infrastructure/src/audio_playback.rs#L160) |
| fn drop | [源码:161](../../../desktop/infrastructure/src/audio_playback.rs#L161) |

## desktop/infrastructure/src/audio_preparation.rs

375 行；SHA256 `2ab4390e787c6eb1a0e5986b75908e52aac83ba77d762e7cea6bb2bb64c745b6`

导入：`use crate::{BlobError, BlobStore, RawBlob};`, `use skin_editor_application::values::AudioMetadata;`, `use skin_editor_audio::{ BuildRequest, Origin, PcmBudget, PcmReservation, PcmUsage, PrepareFailure, PreparedKey, PreparedSound, PreparedWeak, };`, `use std::collections::{BTreeMap, VecDeque};`, `use std::sync::{ Arc, Weak, atomic::{AtomicBool, Ordering}, };`, `use std::time::{Duration, Instant};`

| 定义 | 位置／签名 |
| --- | --- |
| const MAX_PENDING_PREPARATIONS | [源码:16](../../../desktop/infrastructure/src/audio_preparation.rs#L16) |
| const MAX_PCM_CACHE_KEYS | [源码:17](../../../desktop/infrastructure/src/audio_preparation.rs#L17) |
| enum PreparationError | [源码:19](../../../desktop/infrastructure/src/audio_preparation.rs#L19) |
| trait AudioDecoder | [源码:29](../../../desktop/infrastructure/src/audio_preparation.rs#L29) |
| fn probe | [源码:31](../../../desktop/infrastructure/src/audio_preparation.rs#L31) |
| fn decode_into | [源码:33](../../../desktop/infrastructure/src/audio_preparation.rs#L33) |
| impl impl BlobStore | [源码:41](../../../desktop/infrastructure/src/audio_preparation.rs#L41) |
| fn stage_audio | [源码:44](../../../desktop/infrastructure/src/audio_preparation.rs#L44) |
| struct JobLife | [源码:57](../../../desktop/infrastructure/src/audio_preparation.rs#L57) |
| struct Cancellation | [源码:60](../../../desktop/infrastructure/src/audio_preparation.rs#L60) |
| impl impl Cancellation | [源码:61](../../../desktop/infrastructure/src/audio_preparation.rs#L61) |
| fn is_cancelled | [源码:62](../../../desktop/infrastructure/src/audio_preparation.rs#L62) |
| struct Pending | [源码:66](../../../desktop/infrastructure/src/audio_preparation.rs#L66) |
| struct Active | [源码:72](../../../desktop/infrastructure/src/audio_preparation.rs#L72) |
| struct RunningPreparation | [源码:77](../../../desktop/infrastructure/src/audio_preparation.rs#L77) |
| struct FinishedPreparation | [源码:82](../../../desktop/infrastructure/src/audio_preparation.rs#L82) |
| struct PreparationObservation | [源码:92](../../../desktop/infrastructure/src/audio_preparation.rs#L92) |
| impl impl FinishedPreparation | [源码:99](../../../desktop/infrastructure/src/audio_preparation.rs#L99) |
| fn observation | [源码:100](../../../desktop/infrastructure/src/audio_preparation.rs#L100) |
| impl impl RunningPreparation | [源码:107](../../../desktop/infrastructure/src/audio_preparation.rs#L107) |
| fn request | [源码:108](../../../desktop/infrastructure/src/audio_preparation.rs#L108) |
| fn execute | [源码:111](../../../desktop/infrastructure/src/audio_preparation.rs#L111) |
| enum Admission | [源码:160](../../../desktop/infrastructure/src/audio_preparation.rs#L160) |
| struct PreparationUsage | [源码:166](../../../desktop/infrastructure/src/audio_preparation.rs#L166) |
| type PreparationResult | [源码:172](../../../desktop/infrastructure/src/audio_preparation.rs#L172) |
| struct AudioPreparationQueue | [源码:173](../../../desktop/infrastructure/src/audio_preparation.rs#L173) |
| impl impl AudioPreparationQueue | [源码:181](../../../desktop/infrastructure/src/audio_preparation.rs#L181) |
| fn new | [源码:182](../../../desktop/infrastructure/src/audio_preparation.rs#L182) |
| fn prune | [源码:192](../../../desktop/infrastructure/src/audio_preparation.rs#L192) |
| fn usage | [源码:202](../../../desktop/infrastructure/src/audio_preparation.rs#L202) |
| fn submit | [源码:211](../../../desktop/infrastructure/src/audio_preparation.rs#L211) |
| fn start_next | [源码:272](../../../desktop/infrastructure/src/audio_preparation.rs#L272) |
| fn finish | [源码:291](../../../desktop/infrastructure/src/audio_preparation.rs#L291) |
| fn cancel_component | [源码:316](../../../desktop/infrastructure/src/audio_preparation.rs#L316) |
| impl impl Into<skin_editor_audio::SoundTarget>) | [源码:316](../../../desktop/infrastructure/src/audio_preparation.rs#L316) |
| fn cancel_target | [源码:319](../../../desktop/infrastructure/src/audio_preparation.rs#L319) |
| fn cancel_library | [源码:330](../../../desktop/infrastructure/src/audio_preparation.rs#L330) |
| fn cancel_outdated | [源码:351](../../../desktop/infrastructure/src/audio_preparation.rs#L351) |
| fn close | [源码:362](../../../desktop/infrastructure/src/audio_preparation.rs#L362) |
| impl impl Drop for AudioPreparationQueue | [源码:371](../../../desktop/infrastructure/src/audio_preparation.rs#L371) |
| fn drop | [源码:372](../../../desktop/infrastructure/src/audio_preparation.rs#L372) |

## desktop/infrastructure/src/audio_supply.rs

2438 行；SHA256 `515a2d5b665d4a26c59197d13ecbaea3b75388b8970efcabda35ab8f0894676a`

导入：`use crate::native_audio::OutputMark;`, `use crate::separated_output::{self, BackgroundOwner, StoppedOutput};`, `use crate::supply_output::SupplyOutputControl;`, `use crate::{ BlobStore, BuildAdmission, OUTPUT_AHEAD_FRAMES, OUTPUT_QUEUE_FRAMES, OutputControl, OutputProducer, PreparationWorker, SkinAudioController, WavDecoder, };`, `use crate::{background_input::BackgroundInput, background_stream::BufferedBackground};`, `use skin_editor_application::{ AssetAudition, DraftId, ProjectSession, SkinAudioError, SkinAudioPort, SkinAudioSource, values::{AssetId, AudioComponentId, Gain, SkinProject}, };`, `use skin_editor_audio::{ BackgroundTrack, Channel, MAX_BINDINGS, Origin, PcmBudget, SoundPlan, SoundTarget, TriggerReceipt, };`, `use std::collections::{BTreeMap, BTreeSet};`, `use std::sync::{ Arc, Mutex, atomic::{AtomicBool, AtomicUsize, Ordering}, mpsc::{self, Receiver, SyncSender}, };`, `use std::thread::{self, JoinHandle};`, `use super::*;`, `use crate::{OutputConsumer, RawBlob, output_boundary};`, `use skin_editor_application::{values::*, *};`, `use skin_editor_audio::TriggerReceipt;`, `use std::time::Duration;`

| 定义 | 位置／签名 |
| --- | --- |
| const MAX_SUPPLY_CONTROLS | [源码:25](../../../desktop/infrastructure/src/audio_supply.rs#L25) |
| enum SupplyError | [源码:27](../../../desktop/infrastructure/src/audio_supply.rs#L27) |
| struct Credit | [源码:37](../../../desktop/infrastructure/src/audio_supply.rs#L37) |
| impl impl Drop for Credit | [源码:38](../../../desktop/infrastructure/src/audio_supply.rs#L38) |
| fn drop | [源码:39](../../../desktop/infrastructure/src/audio_supply.rs#L39) |
| enum SupplyOutcome | [源码:44](../../../desktop/infrastructure/src/audio_supply.rs#L44) |
| struct SupplyReceipt | [源码:64](../../../desktop/infrastructure/src/audio_supply.rs#L64) |
| struct SupplyStatus | [源码:72](../../../desktop/infrastructure/src/audio_supply.rs#L72) |
| struct Projection | [源码:91](../../../desktop/infrastructure/src/audio_supply.rs#L91) |
| enum Control | [源码:97](../../../desktop/infrastructure/src/audio_supply.rs#L97) |
| struct Request | [源码:131](../../../desktop/infrastructure/src/audio_supply.rs#L131) |
| struct AudioSupplyClient | [源码:136](../../../desktop/infrastructure/src/audio_supply.rs#L136) |
| struct RetiredSupply | [源码:156](../../../desktop/infrastructure/src/audio_supply.rs#L156) |
| enum SupplyOutput | [源码:160](../../../desktop/infrastructure/src/audio_supply.rs#L160) |
| impl impl SupplyOutput | [源码:164](../../../desktop/infrastructure/src/audio_supply.rs#L164) |
| fn pump | [源码:165](../../../desktop/infrastructure/src/audio_supply.rs#L165) |
| fn backend | [源码:200](../../../desktop/infrastructure/src/audio_supply.rs#L200) |
| impl impl std::fmt::Debug) -> SupplyError | [源码:200](../../../desktop/infrastructure/src/audio_supply.rs#L200) |
| fn origin | [源码:203](../../../desktop/infrastructure/src/audio_supply.rs#L203) |
| impl impl AudioSupplyClient | [源码:212](../../../desktop/infrastructure/src/audio_supply.rs#L212) |
| fn diagnostic_recovery_probe | [源码:215](../../../desktop/infrastructure/src/audio_supply.rs#L215) |
| fn barrier | [源码:235](../../../desktop/infrastructure/src/audio_supply.rs#L235) |
| fn spawn | [源码:239](../../../desktop/infrastructure/src/audio_supply.rs#L239) |
| fn spawn_separated | [源码:259](../../../desktop/infrastructure/src/audio_supply.rs#L259) |
| fn spawn_streamed | [源码:271](../../../desktop/infrastructure/src/audio_supply.rs#L271) |
| fn spawn_owned | [源码:283](../../../desktop/infrastructure/src/audio_supply.rs#L283) |
| fn reserve | [源码:425](../../../desktop/infrastructure/src/audio_supply.rs#L425) |
| fn send | [源码:436](../../../desktop/infrastructure/src/audio_supply.rs#L436) |
| fn synchronize | [源码:460](../../../desktop/infrastructure/src/audio_supply.rs#L460) |
| fn set_background | [源码:463](../../../desktop/infrastructure/src/audio_supply.rs#L463) |
| fn scene_running | [源码:467](../../../desktop/infrastructure/src/audio_supply.rs#L467) |
| fn seek_background | [源码:480](../../../desktop/infrastructure/src/audio_supply.rs#L480) |
| fn audition_asset | [源码:491](../../../desktop/infrastructure/src/audio_supply.rs#L491) |
| fn stop_audition | [源码:516](../../../desktop/infrastructure/src/audio_supply.rs#L516) |
| fn retry_selected | [源码:522](../../../desktop/infrastructure/src/audio_supply.rs#L522) |
| fn trigger | [源码:526](../../../desktop/infrastructure/src/audio_supply.rs#L526) |
| fn trigger_selected | [源码:536](../../../desktop/infrastructure/src/audio_supply.rs#L536) |
| fn trigger_batch_selected | [源码:566](../../../desktop/infrastructure/src/audio_supply.rs#L566) |
| fn poll | [源码:596](../../../desktop/infrastructure/src/audio_supply.rs#L596) |
| fn wait_receipt | [源码:601](../../../desktop/infrastructure/src/audio_supply.rs#L601) |
| fn accept_receipt | [源码:611](../../../desktop/infrastructure/src/audio_supply.rs#L611) |
| fn status | [源码:617](../../../desktop/infrastructure/src/audio_supply.rs#L617) |
| fn pending_controls | [源码:620](../../../desktop/infrastructure/src/audio_supply.rs#L620) |
| fn output | [源码:623](../../../desktop/infrastructure/src/audio_supply.rs#L623) |
| fn close | [源码:626](../../../desktop/infrastructure/src/audio_supply.rs#L626) |
| fn finish_separated | [源码:637](../../../desktop/infrastructure/src/audio_supply.rs#L637) |
| impl impl Drop for AudioSupplyClient | [源码:651](../../../desktop/infrastructure/src/audio_supply.rs#L651) |
| fn drop | [源码:652](../../../desktop/infrastructure/src/audio_supply.rs#L652) |
| impl impl SkinAudioPort for AudioSupplyClient | [源码:656](../../../desktop/infrastructure/src/audio_supply.rs#L656) |
| fn retained_drafts | [源码:657](../../../desktop/infrastructure/src/audio_supply.rs#L657) |
| fn synchronize | [源码:675](../../../desktop/infrastructure/src/audio_supply.rs#L675) |
| struct SupplyCore | [源码:748](../../../desktop/infrastructure/src/audio_supply.rs#L748) |
| impl impl SupplyCore | [源码:769](../../../desktop/infrastructure/src/audio_supply.rs#L769) |
| fn can_idle | [源码:770](../../../desktop/infrastructure/src/audio_supply.rs#L770) |
| fn new_separated | [源码:792](../../../desktop/infrastructure/src/audio_supply.rs#L792) |
| impl impl Into<BackgroundInput>, rate: u32, channels: u16, budget: PcmBudget, ) -> Result<(Self, BackgroundOwner, Box<separated_output::OutputConsumer>), SupplyError> | [源码:793](../../../desktop/infrastructure/src/audio_supply.rs#L793) |
| fn generated_background | [源码:842](../../../desktop/infrastructure/src/audio_supply.rs#L842) |
| fn publish_progress | [源码:851](../../../desktop/infrastructure/src/audio_supply.rs#L851) |
| fn retained_drafts | [源码:858](../../../desktop/infrastructure/src/audio_supply.rs#L858) |
| fn apply_output | [源码:869](../../../desktop/infrastructure/src/audio_supply.rs#L869) |
| fn finish_acknowledgments | [源码:1016](../../../desktop/infrastructure/src/audio_supply.rs#L1016) |
| fn apply_owned | [源码:1026](../../../desktop/infrastructure/src/audio_supply.rs#L1026) |
| fn new | [源码:1069](../../../desktop/infrastructure/src/audio_supply.rs#L1069) |
| fn apply | [源码:1100](../../../desktop/infrastructure/src/audio_supply.rs#L1100) |
| fn finish_freeze | [源码:1279](../../../desktop/infrastructure/src/audio_supply.rs#L1279) |
| fn prepare | [源码:1323](../../../desktop/infrastructure/src/audio_supply.rs#L1323) |
| fn activate_library | [源码:1338](../../../desktop/infrastructure/src/audio_supply.rs#L1338) |
| fn status | [源码:1360](../../../desktop/infrastructure/src/audio_supply.rs#L1360) |
| fn voice_counts | [源码:1395](../../../desktop/infrastructure/src/audio_supply.rs#L1395) |
| impl impl Drop for SupplyCore | [源码:1403](../../../desktop/infrastructure/src/audio_supply.rs#L1403) |
| fn drop | [源码:1404](../../../desktop/infrastructure/src/audio_supply.rs#L1404) |
| mod tests | [源码:1412](../../../desktop/infrastructure/src/audio_supply.rs#L1412) |
| fn accepted_control_wakes_owner_even_if_notification_precedes_park | [源码:1414](../../../desktop/infrastructure/src/audio_supply.rs#L1414) |
| fn retirement_observation_requires_a_status_at_least_as_new_as_the_receipt | [源码:1462](../../../desktop/infrastructure/src/audio_supply.rs#L1462) |
| mod separated | [源码:1506](../../../desktop/infrastructure/src/audio_supply.rs#L1506) |
| fn owned_recovery_probe_uses_the_same_trigger_gate_and_returns_control_credit | [源码:1508](../../../desktop/infrastructure/src/audio_supply.rs#L1508) |
| fn idle_stale_and_failed_trigger_probes_cannot_leave_an_armed_recovery | [源码:1538](../../../desktop/infrastructure/src/audio_supply.rs#L1538) |
| fn controls_keep_order_credit_backpressure_and_close_release | [源码:1586](../../../desktop/infrastructure/src/audio_supply.rs#L1586) |
| fn origin | [源码:1620](../../../desktop/infrastructure/src/audio_supply.rs#L1620) |
| fn id | [源码:1629](../../../desktop/infrastructure/src/audio_supply.rs#L1629) |
| fn request | [源码:1632](../../../desktop/infrastructure/src/audio_supply.rs#L1632) |
| fn apply | [源码:1639](../../../desktop/infrastructure/src/audio_supply.rs#L1639) |
| fn reference | [源码:1642](../../../desktop/infrastructure/src/audio_supply.rs#L1642) |
| fn pump | [源码:1649](../../../desktop/infrastructure/src/audio_supply.rs#L1649) |
| fn render_reference | [源码:1659](../../../desktop/infrastructure/src/audio_supply.rs#L1659) |
| fn select | [源码:1666](../../../desktop/infrastructure/src/audio_supply.rs#L1666) |
| fn wave | [源码:1685](../../../desktop/infrastructure/src/audio_supply.rs#L1685) |
| fn raw_plan | [源码:1705](../../../desktop/infrastructure/src/audio_supply.rs#L1705) |
| fn finish | [源码:1716](../../../desktop/infrastructure/src/audio_supply.rs#L1716) |
| fn skin_replacement_navigation_and_cancel_never_reset_background | [源码:1723](../../../desktop/infrastructure/src/audio_supply.rs#L1723) |
| fn scene_hot_activation_keeps_started_one_shot_gain | [源码:1790](../../../desktop/infrastructure/src/audio_supply.rs#L1790) |
| fn freeze_waits_for_callback_rewinds_unused_ahead_and_resumes_exact_sample | [源码:1852](../../../desktop/infrastructure/src/audio_supply.rs#L1852) |
| fn desired_fence_cannot_relabel_old_runtime_samples_or_cut_background | [源码:1951](../../../desktop/infrastructure/src/audio_supply.rs#L1951) |
| fn incompatible_background_or_supply_budget_is_rejected_and_reservations_return | [源码:2021](../../../desktop/infrastructure/src/audio_supply.rs#L2021) |
| fn receive | [源码:2048](../../../desktop/infrastructure/src/audio_supply.rs#L2048) |
| fn independent_owner_supplies_without_gui_ticks_and_redundant_resume_is_harmless | [源码:2061](../../../desktop/infrastructure/src/audio_supply.rs#L2061) |
| fn control_credit_covers_unread_and_retained_replies_and_close_bypasses_full_queue | [源码:2086](../../../desktop/infrastructure/src/audio_supply.rs#L2086) |
| fn projection_uses_application_cancel_facts_even_without_status_polling | [源码:2111](../../../desktop/infrastructure/src/audio_supply.rs#L2111) |
| fn late_library_decode_after_stop_or_navigation_cannot_activate_or_retain_pcm | [源码:2207](../../../desktop/infrastructure/src/audio_supply.rs#L2207) |
| fn library_voice_limit_is_visible_and_does_not_close_supply_or_stop_scene | [源码:2258](../../../desktop/infrastructure/src/audio_supply.rs#L2258) |
| fn repeated_library_audition_reuses_pcm_without_a_component_or_another_decode | [源码:2300](../../../desktop/infrastructure/src/audio_supply.rs#L2300) |
| fn bounded_batch_starts_32_voices_at_one_frame_and_refuses_33rd_without_stealing | [源码:2352](../../../desktop/infrastructure/src/audio_supply.rs#L2352) |

## desktop/infrastructure/src/audio_supply/tests/separated.rs

481 行；SHA256 `4bf0663ed1aed07e754e7f8a147d62a518f9dcff25e9a5dfb734ebfdcbfe87d4`

导入：`use super::*;`, `use crate::separated_output;`, `use skin_editor_audio::PreparedSound;`

| 定义 | 位置／签名 |
| --- | --- |
| mod streaming | [源码:4](../../../desktop/infrastructure/src/audio_supply/tests/separated.rs#L4) |
| fn idle | [源码:5](../../../desktop/infrastructure/src/audio_supply/tests/separated.rs#L5) |
| fn idle_wait_requires_frozen_primed_settled_controls_and_no_voice_or_preparation | [源码:11](../../../desktop/infrastructure/src/audio_supply/tests/separated.rs#L11) |
| type Setup | [源码:86](../../../desktop/infrastructure/src/audio_supply/tests/separated.rs#L86) |
| fn setup | [源码:93](../../../desktop/infrastructure/src/audio_supply/tests/separated.rs#L93) |
| fn apply_output | [源码:105](../../../desktop/infrastructure/src/audio_supply/tests/separated.rs#L105) |
| fn ready | [源码:113](../../../desktop/infrastructure/src/audio_supply/tests/separated.rs#L113) |
| fn scene | [源码:133](../../../desktop/infrastructure/src/audio_supply/tests/separated.rs#L133) |
| fn start | [源码:142](../../../desktop/infrastructure/src/audio_supply/tests/separated.rs#L142) |
| fn trigger | [源码:149](../../../desktop/infrastructure/src/audio_supply/tests/separated.rs#L149) |
| fn finish | [源码:169](../../../desktop/infrastructure/src/audio_supply/tests/separated.rs#L169) |
| fn queued_skin_invalidation_mutes_old_voice_without_cutting_background | [源码:184](../../../desktop/infrastructure/src/audio_supply/tests/separated.rs#L184) |
| fn pause_request_fences_before_worker_then_acknowledges_exact_copy_anchor | [源码:216](../../../desktop/infrastructure/src/audio_supply/tests/separated.rs#L216) |
| fn stop_and_barrier_receipts_wait_for_native_copy_ack_and_do_not_drop_credit | [源码:261](../../../desktop/infrastructure/src/audio_supply/tests/separated.rs#L261) |
| fn cancelled_draft_remains_reported_until_its_transport_pcm_is_collected | [源码:287](../../../desktop/infrastructure/src/audio_supply/tests/separated.rs#L287) |
| fn history_switch_rejects_old_trigger_and_does_not_reset_background | [源码:319](../../../desktop/infrastructure/src/audio_supply/tests/separated.rs#L319) |
| fn busy_resume_preserves_pending_pause_and_a_retry_recovers_without_old_time | [源码:366](../../../desktop/infrastructure/src/audio_supply/tests/separated.rs#L366) |
| fn real_worker_full_unread_control_window_can_always_close_and_finish | [源码:385](../../../desktop/infrastructure/src/audio_supply/tests/separated.rs#L385) |
| fn foreign_stopped_endpoint_does_not_consume_either_retired_worker | [源码:407](../../../desktop/infrastructure/src/audio_supply/tests/separated.rs#L407) |
| fn cancelling_one_draft_keeps_an_unrelated_active_scene_voice | [源码:434](../../../desktop/infrastructure/src/audio_supply/tests/separated.rs#L434) |
| fn idle_status_publication_advances_copied_position_without_another_control | [源码:466](../../../desktop/infrastructure/src/audio_supply/tests/separated.rs#L466) |

## desktop/infrastructure/src/audio_supply/tests/separated/streaming.rs

226 行；SHA256 `8aaf9e92433394bd19ed9937e17c36770f4703a874894cfb4df88320d0f1bcdb`

导入：`use super::*;`, `use crate::background_stream::{BufferedBackground, tests::Generated};`, `use std::{ io::{Read, Seek, SeekFrom}, sync::mpsc::{Receiver, SyncSender, sync_channel}, };`

| 定义 | 位置／签名 |
| --- | --- |
| const GUARD | [源码:7](../../../desktop/infrastructure/src/audio_supply/tests/separated/streaming.rs#L7) |
| fn prime | [源码:8](../../../desktop/infrastructure/src/audio_supply/tests/separated/streaming.rs#L8) |
| fn streamed | [源码:19](../../../desktop/infrastructure/src/audio_supply/tests/separated/streaming.rs#L19) |
| fn seek | [源码:31](../../../desktop/infrastructure/src/audio_supply/tests/separated/streaming.rs#L31) |
| fn streamed_seek_waits_for_owner_and_copy_ack_preserves_audition_and_uses_requested_anchor | [源码:41](../../../desktop/infrastructure/src/audio_supply/tests/separated/streaming.rs#L41) |
| fn busy_second_seek_merges_fence_without_changing_first_location_or_locking_completion | [源码:85](../../../desktop/infrastructure/src/audio_supply/tests/separated/streaming.rs#L85) |
| struct Gate | [源码:108](../../../desktop/infrastructure/src/audio_supply/tests/separated/streaming.rs#L108) |
| impl impl Read for Gate | [源码:114](../../../desktop/infrastructure/src/audio_supply/tests/separated/streaming.rs#L114) |
| fn read | [源码:115](../../../desktop/infrastructure/src/audio_supply/tests/separated/streaming.rs#L115) |
| impl impl Seek for Gate | [源码:127](../../../desktop/infrastructure/src/audio_supply/tests/separated/streaming.rs#L127) |
| fn seek | [源码:128](../../../desktop/infrastructure/src/audio_supply/tests/separated/streaming.rs#L128) |
| fn blocked_io_is_not_in_callback_and_unprimed_resume_is_explicit_busy_then_retryable | [源码:133](../../../desktop/infrastructure/src/audio_supply/tests/separated/streaming.rs#L133) |
| fn streamed_hot_selection_and_history_change_do_not_seek_or_rewind_background | [源码:175](../../../desktop/infrastructure/src/audio_supply/tests/separated/streaming.rs#L175) |
| fn invalid_client_location_does_not_invalidate_any_lane_or_consume_control_credit | [源码:208](../../../desktop/infrastructure/src/audio_supply/tests/separated/streaming.rs#L208) |

## desktop/infrastructure/src/audio_worker.rs

69 行；SHA256 `f4fa422ccf7a0239d0ef634f0e1d1d75e51ebd8b6154ff04af750d8ba5aac461`

导入：`use crate::{AudioDecoder, FinishedPreparation, RunningPreparation};`, `use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};`, `use std::thread::{self, JoinHandle};`

| 定义 | 位置／签名 |
| --- | --- |
| struct PreparationWorker | [源码:6](../../../desktop/infrastructure/src/audio_worker.rs#L6) |
| impl impl PreparationWorker | [源码:12](../../../desktop/infrastructure/src/audio_worker.rs#L12) |
| fn new | [源码:13](../../../desktop/infrastructure/src/audio_worker.rs#L13) |
| impl impl AudioDecoder + Send + 'static) -> std::io::Result<Self> | [源码:13](../../../desktop/infrastructure/src/audio_worker.rs#L13) |
| fn try_submit | [源码:32](../../../desktop/infrastructure/src/audio_worker.rs#L32) |
| fn try_result | [源码:40](../../../desktop/infrastructure/src/audio_worker.rs#L40) |
| fn wait_result | [源码:44](../../../desktop/infrastructure/src/audio_worker.rs#L44) |
| fn close | [源码:48](../../../desktop/infrastructure/src/audio_worker.rs#L48) |
| fn closed | [源码:61](../../../desktop/infrastructure/src/audio_worker.rs#L61) |
| impl impl Drop for PreparationWorker | [源码:65](../../../desktop/infrastructure/src/audio_worker.rs#L65) |
| fn drop | [源码:66](../../../desktop/infrastructure/src/audio_worker.rs#L66) |

## desktop/infrastructure/src/background_input.rs

63 行；SHA256 `cbdf7b3a5c7a89b4fb8364a2257ece2c9e87557d515d9ea6b01d567946da77dd`

导入：`use crate::{background_stream::BufferedBackground, separated_output::OutputError};`, `use skin_editor_audio::{BackgroundTrack, PcmBudget};`

| 定义 | 位置／签名 |
| --- | --- |
| enum BackgroundInput | [源码:3](../../../desktop/infrastructure/src/background_input.rs#L3) |
| impl impl From<BackgroundTrack> for BackgroundInput | [源码:7](../../../desktop/infrastructure/src/background_input.rs#L7) |
| fn from | [源码:8](../../../desktop/infrastructure/src/background_input.rs#L8) |
| impl impl From<BufferedBackground> for BackgroundInput | [源码:12](../../../desktop/infrastructure/src/background_input.rs#L12) |
| fn from | [源码:13](../../../desktop/infrastructure/src/background_input.rs#L13) |
| impl impl BackgroundInput | [源码:17](../../../desktop/infrastructure/src/background_input.rs#L17) |
| fn format | [源码:18](../../../desktop/infrastructure/src/background_input.rs#L18) |
| fn frames | [源码:24](../../../desktop/infrastructure/src/background_input.rs#L24) |
| fn position | [源码:30](../../../desktop/infrastructure/src/background_input.rs#L30) |
| fn budget | [源码:36](../../../desktop/infrastructure/src/background_input.rs#L36) |
| fn restore_position | [源码:42](../../../desktop/infrastructure/src/background_input.rs#L42) |
| fn set_running | [源码:52](../../../desktop/infrastructure/src/background_input.rs#L52) |
| fn next_frame | [源码:57](../../../desktop/infrastructure/src/background_input.rs#L57) |

## desktop/infrastructure/src/background_stream.rs

440 行；SHA256 `c6f2a9c40dd736b99d05bc7c5d34acebc953b87346b252b9816552717d5aeec3`

导入：`use crate::byte_budget::{ByteBudget, ByteLease};`, `use rtrb::{Consumer, RingBuffer};`, `use skin_editor_audio::{PcmBudget, PcmReservation};`, `use std::{ io::{Read, Seek, SeekFrom}, sync::{ Arc, atomic::{AtomicBool, AtomicU64, Ordering::SeqCst}, }, thread::{self, JoinHandle}, };`

| 定义 | 位置／签名 |
| --- | --- |
| const SOURCE_RESERVE | [源码:16](../../../desktop/infrastructure/src/background_stream.rs#L16) |
| const READ_FRAMES | [源码:17](../../../desktop/infrastructure/src/background_stream.rs#L17) |
| const READ_BYTES | [源码:18](../../../desktop/infrastructure/src/background_stream.rs#L18) |
| enum StreamError | [源码:20](../../../desktop/infrastructure/src/background_stream.rs#L20) |
| trait ReadSeek | [源码:28](../../../desktop/infrastructure/src/background_stream.rs#L28) |
| impl impl<T: Read + Seek + Send> ReadSeek for T | [源码:29](../../../desktop/infrastructure/src/background_stream.rs#L29) |
| struct Wave | [源码:30](../../../desktop/infrastructure/src/background_stream.rs#L30) |
| impl impl Wave | [源码:37](../../../desktop/infrastructure/src/background_stream.rs#L37) |
| fn open | [源码:38](../../../desktop/infrastructure/src/background_stream.rs#L38) |
| struct State | [源码:116](../../../desktop/infrastructure/src/background_stream.rs#L116) |
| impl impl State | [源码:125](../../../desktop/infrastructure/src/background_stream.rs#L125) |
| fn requested | [源码:126](../../../desktop/infrastructure/src/background_stream.rs#L126) |
| struct StreamObservation | [源码:136](../../../desktop/infrastructure/src/background_stream.rs#L136) |
| struct StreamStats | [源码:141](../../../desktop/infrastructure/src/background_stream.rs#L141) |
| impl impl StreamObservation | [源码:149](../../../desktop/infrastructure/src/background_stream.rs#L149) |
| fn stats | [源码:150](../../../desktop/infrastructure/src/background_stream.rs#L150) |
| const BLOCKS | [源码:161](../../../desktop/infrastructure/src/background_stream.rs#L161) |
| struct Block | [源码:162](../../../desktop/infrastructure/src/background_stream.rs#L162) |
| struct BufferedBackground | [源码:168](../../../desktop/infrastructure/src/background_stream.rs#L168) |
| impl impl BufferedBackground | [源码:183](../../../desktop/infrastructure/src/background_stream.rs#L183) |
| fn open | [源码:186](../../../desktop/infrastructure/src/background_stream.rs#L186) |
| impl impl ReadSeek + 'static, budget: &PcmBudget) -> Result<Self, StreamError> | [源码:186](../../../desktop/infrastructure/src/background_stream.rs#L186) |
| fn spawn | [源码:189](../../../desktop/infrastructure/src/background_stream.rs#L189) |
| fn spawn_inner | [源码:197](../../../desktop/infrastructure/src/background_stream.rs#L197) |
| fn observation | [源码:324](../../../desktop/infrastructure/src/background_stream.rs#L324) |
| fn format | [源码:327](../../../desktop/infrastructure/src/background_stream.rs#L327) |
| fn frames | [源码:330](../../../desktop/infrastructure/src/background_stream.rs#L330) |
| fn position | [源码:333](../../../desktop/infrastructure/src/background_stream.rs#L333) |
| fn budget | [源码:336](../../../desktop/infrastructure/src/background_stream.rs#L336) |
| fn recycle | [源码:339](../../../desktop/infrastructure/src/background_stream.rs#L339) |
| fn restore_position | [源码:346](../../../desktop/infrastructure/src/background_stream.rs#L346) |
| fn next_frame | [源码:374](../../../desktop/infrastructure/src/background_stream.rs#L374) |
| impl impl Drop for BufferedBackground | [源码:413](../../../desktop/infrastructure/src/background_stream.rs#L413) |
| fn drop | [源码:414](../../../desktop/infrastructure/src/background_stream.rs#L414) |
| fn park_reader | [源码:423](../../../desktop/infrastructure/src/background_stream.rs#L423) |
| mod tests | [源码:440](../../../desktop/infrastructure/src/background_stream.rs#L440) |

## desktop/infrastructure/src/background_stream/tests.rs

379 行；SHA256 `3f0964fdb64d5d6e840aa082b30ca218657633f279f5b8067d55e9da50013498`

导入：`use super::*;`, `use std::io::{self, Cursor};`, `use std::sync::mpsc::{Receiver, SyncSender, sync_channel};`, `use std::time::Duration;`

| 定义 | 位置／签名 |
| --- | --- |
| struct WaitGate | [源码:6](../../../desktop/infrastructure/src/background_stream/tests.rs#L6) |
| fn gated | [源码:10](../../../desktop/infrastructure/src/background_stream/tests.rs#L10) |
| fn recycled_block_wakes_reader_even_before_it_parks_and_preserves_every_frame | [源码:29](../../../desktop/infrastructure/src/background_stream/tests.rs#L29) |
| fn eof_seek_before_park_is_not_lost_and_close_releases_the_idle_reader | [源码:47](../../../desktop/infrastructure/src/background_stream/tests.rs#L47) |
| fn exhausted_seek_version_wakes_and_releases_reader_storage_before_owner_drop | [源码:60](../../../desktop/infrastructure/src/background_stream/tests.rs#L60) |
| struct Generated | [源码:81](../../../desktop/infrastructure/src/background_stream/tests.rs#L81) |
| impl impl Generated | [源码:87](../../../desktop/infrastructure/src/background_stream/tests.rs#L87) |
| fn new | [源码:88](../../../desktop/infrastructure/src/background_stream/tests.rs#L88) |
| fn length | [源码:110](../../../desktop/infrastructure/src/background_stream/tests.rs#L110) |
| fn sample | [源码:113](../../../desktop/infrastructure/src/background_stream/tests.rs#L113) |
| impl impl Read for Generated | [源码:117](../../../desktop/infrastructure/src/background_stream/tests.rs#L117) |
| fn read | [源码:118](../../../desktop/infrastructure/src/background_stream/tests.rs#L118) |
| impl impl Seek for Generated | [源码:135](../../../desktop/infrastructure/src/background_stream/tests.rs#L135) |
| fn seek | [源码:136](../../../desktop/infrastructure/src/background_stream/tests.rs#L136) |
| fn next | [源码:149](../../../desktop/infrastructure/src/background_stream/tests.rs#L149) |
| struct Gate | [源码:162](../../../desktop/infrastructure/src/background_stream/tests.rs#L162) |
| impl impl Read for Gate | [源码:167](../../../desktop/infrastructure/src/background_stream/tests.rs#L167) |
| fn read | [源码:168](../../../desktop/infrastructure/src/background_stream/tests.rs#L168) |
| impl impl Seek for Gate | [源码:178](../../../desktop/infrastructure/src/background_stream/tests.rs#L178) |
| fn seek | [源码:179](../../../desktop/infrastructure/src/background_stream/tests.rs#L179) |
| fn ten_minute_source_consumes_across_ring_wraps_with_constant_pcm_and_raw_storage | [源码:184](../../../desktop/infrastructure/src/background_stream/tests.rs#L184) |
| fn late_read_is_discarded_after_seek_without_publishing_old_position | [源码:206](../../../desktop/infrastructure/src/background_stream/tests.rs#L206) |
| fn cancellation_waits_for_actual_reader_exit_before_returning_resource_credits | [源码:228](../../../desktop/infrastructure/src/background_stream/tests.rs#L228) |
| fn eof_and_invalid_seek_do_not_loop_or_reset_the_source | [源码:258](../../../desktop/infrastructure/src/background_stream/tests.rs#L258) |
| fn over_budget_open_does_not_start_a_reader_or_retain_partial_reservations | [源码:275](../../../desktop/infrastructure/src/background_stream/tests.rs#L275) |
| fn malformed_and_unsupported_wave_headers_are_rejected_without_worker_storage | [源码:284](../../../desktop/infrastructure/src/background_stream/tests.rs#L284) |
| fn exhausted_source_version_cannot_wrap_to_an_old_request | [源码:302](../../../desktop/infrastructure/src/background_stream/tests.rs#L302) |
| fn both_minimum_rates_and_channel_counts_produce_exact_pcm_without_full_file_storage | [源码:311](../../../desktop/infrastructure/src/background_stream/tests.rs#L311) |
| fn repeated_locations_before_read_completion_only_publish_the_latest_position | [源码:329](../../../desktop/infrastructure/src/background_stream/tests.rs#L329) |
| struct Broken | [源码:351](../../../desktop/infrastructure/src/background_stream/tests.rs#L351) |
| impl impl Read for Broken | [源码:352](../../../desktop/infrastructure/src/background_stream/tests.rs#L352) |
| fn read | [源码:353](../../../desktop/infrastructure/src/background_stream/tests.rs#L353) |
| impl impl Seek for Broken | [源码:360](../../../desktop/infrastructure/src/background_stream/tests.rs#L360) |
| fn seek | [源码:361](../../../desktop/infrastructure/src/background_stream/tests.rs#L361) |
| fn reader_failure_is_reported_instead_of_inventing_eof_or_silence_and_close_releases_all_storage | [源码:366](../../../desktop/infrastructure/src/background_stream/tests.rs#L366) |

## desktop/infrastructure/src/blobs.rs

212 行；SHA256 `e1cf32b28f77dbcc3f80a3d2ac6a8653b41d4e54b9837de3814f0a16e74c87f8`

导入：`use crate::byte_budget::{ByteBudget, ByteLease};`, `use sha2::{Digest, Sha256};`, `use skin_editor_application::{ BlobLease, ResourceError, ResourceStore, values::{AudioMetadata, BlobRef}, };`, `use std::collections::BTreeMap;`, `use std::io::{Read, Seek, SeekFrom};`, `use std::sync::{Arc, Mutex, Weak};`

| 定义 | 位置／签名 |
| --- | --- |
| enum BlobError | [源码:13](../../../desktop/infrastructure/src/blobs.rs#L13) |
| struct BlobUsage | [源码:23](../../../desktop/infrastructure/src/blobs.rs#L23) |
| struct Entry | [源码:28](../../../desktop/infrastructure/src/blobs.rs#L28) |
| impl impl BlobLease for Entry | [源码:34](../../../desktop/infrastructure/src/blobs.rs#L34) |
| fn reference | [源码:35](../../../desktop/infrastructure/src/blobs.rs#L35) |
| struct RawBlob | [源码:40](../../../desktop/infrastructure/src/blobs.rs#L40) |
| impl impl RawBlob | [源码:41](../../../desktop/infrastructure/src/blobs.rs#L41) |
| fn reference | [源码:42](../../../desktop/infrastructure/src/blobs.rs#L42) |
| fn bytes | [源码:45](../../../desktop/infrastructure/src/blobs.rs#L45) |
| fn lease | [源码:48](../../../desktop/infrastructure/src/blobs.rs#L48) |
| fn same_allocation | [源码:51](../../../desktop/infrastructure/src/blobs.rs#L51) |
| fn audio_metadata | [源码:54](../../../desktop/infrastructure/src/blobs.rs#L54) |
| fn attest_audio | [源码:57](../../../desktop/infrastructure/src/blobs.rs#L57) |
| struct BlobStore | [源码:67](../../../desktop/infrastructure/src/blobs.rs#L67) |
| impl impl Default for BlobStore | [源码:72](../../../desktop/infrastructure/src/blobs.rs#L72) |
| fn default | [源码:73](../../../desktop/infrastructure/src/blobs.rs#L73) |
| impl impl BlobStore | [源码:77](../../../desktop/infrastructure/src/blobs.rs#L77) |
| fn observe_bytes | [源码:78](../../../desktop/infrastructure/src/blobs.rs#L78) |
| fn stage_original_file | [源码:82](../../../desktop/infrastructure/src/blobs.rs#L82) |
| fn with_limits | [源码:87](../../../desktop/infrastructure/src/blobs.rs#L87) |
| fn get | [源码:97](../../../desktop/infrastructure/src/blobs.rs#L97) |
| fn usage | [源码:105](../../../desktop/infrastructure/src/blobs.rs#L105) |
| fn insert | [源码:114](../../../desktop/infrastructure/src/blobs.rs#L114) |
| fn stage_bytes | [源码:143](../../../desktop/infrastructure/src/blobs.rs#L143) |
| fn stage_reader | [源码:162](../../../desktop/infrastructure/src/blobs.rs#L162) |
| impl impl Read + Seek, byte_len: u64, ) -> Result<RawBlob, BlobError> | [源码:164](../../../desktop/infrastructure/src/blobs.rs#L164) |
| impl impl ResourceStore for BlobStore | [源码:203](../../../desktop/infrastructure/src/blobs.rs#L203) |
| fn pin | [源码:204](../../../desktop/infrastructure/src/blobs.rs#L204) |
| fn audio_metadata | [源码:209](../../../desktop/infrastructure/src/blobs.rs#L209) |

## desktop/infrastructure/src/byte_budget.rs

66 行；SHA256 `db073b247cf1a004b57c3d1878b4357aa854dcba64cdb8786b7a95a5e711ebcd`

导入：`use std::sync::{ Arc, atomic::{AtomicUsize, Ordering}, };`

| 定义 | 位置／签名 |
| --- | --- |
| struct ByteBudget | [源码:6](../../../desktop/infrastructure/src/byte_budget.rs#L6) |
| struct ByteObservation | [源码:13](../../../desktop/infrastructure/src/byte_budget.rs#L13) |
| struct ByteUsage | [源码:15](../../../desktop/infrastructure/src/byte_budget.rs#L15) |
| impl impl ByteObservation | [源码:19](../../../desktop/infrastructure/src/byte_budget.rs#L19) |
| fn usage | [源码:20](../../../desktop/infrastructure/src/byte_budget.rs#L20) |
| impl impl ByteBudget | [源码:27](../../../desktop/infrastructure/src/byte_budget.rs#L27) |
| fn observation | [源码:28](../../../desktop/infrastructure/src/byte_budget.rs#L28) |
| fn new | [源码:31](../../../desktop/infrastructure/src/byte_budget.rs#L31) |
| fn reserve | [源码:38](../../../desktop/infrastructure/src/byte_budget.rs#L38) |
| fn used | [源码:51](../../../desktop/infrastructure/src/byte_budget.rs#L51) |
| fn peak | [源码:54](../../../desktop/infrastructure/src/byte_budget.rs#L54) |
| struct ByteLease | [源码:58](../../../desktop/infrastructure/src/byte_budget.rs#L58) |
| impl impl Drop for ByteLease | [源码:62](../../../desktop/infrastructure/src/byte_budget.rs#L62) |
| fn drop | [源码:63](../../../desktop/infrastructure/src/byte_budget.rs#L63) |

## desktop/infrastructure/src/callback_audit.rs

84 行；SHA256 `5679aaa4ef92ac999436a64e7f6184c2b51f560ec126bd320696afe15cfca3aa`

导入：`use std::cell::Cell;`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| struct State | [源码:5](../../../desktop/infrastructure/src/callback_audit.rs#L5) |
| static STATE | [源码:9](../../../desktop/infrastructure/src/callback_audit.rs#L9) |
| struct Scope | [源码:10](../../../desktop/infrastructure/src/callback_audit.rs#L10) |
| fn active | [源码:14](../../../desktop/infrastructure/src/callback_audit.rs#L14) |
| fn released | [源码:17](../../../desktop/infrastructure/src/callback_audit.rs#L17) |
| impl impl Scope | [源码:26](../../../desktop/infrastructure/src/callback_audit.rs#L26) |
| fn enter | [源码:27](../../../desktop/infrastructure/src/callback_audit.rs#L27) |
| fn finish | [源码:41](../../../desktop/infrastructure/src/callback_audit.rs#L41) |
| impl impl Drop for Scope | [源码:54](../../../desktop/infrastructure/src/callback_audit.rs#L54) |
| fn drop | [源码:55](../../../desktop/infrastructure/src/callback_audit.rs#L55) |
| mod tests | [源码:62](../../../desktop/infrastructure/src/callback_audit.rs#L62) |
| fn resource_drops_are_scoped_and_other_owner_threads_do_not_create_false_callback_violations | [源码:65](../../../desktop/infrastructure/src/callback_audit.rs#L65) |
| fn unwind_restores_the_previous_thread_scope | [源码:77](../../../desktop/infrastructure/src/callback_audit.rs#L77) |

## desktop/infrastructure/src/lib.rs

423 行；SHA256 `5db755153396f192f9ff6472fca8f26fa083c8f3d3ba33e97169ebe394b3b069`

导入：`use audio_supply::{ AudioSupplyClient, MAX_SUPPLY_CONTROLS, SupplyError, SupplyOutcome, SupplyReceipt, SupplyStatus, };`, `use submission_trace::{StreamClockSample, SubmissionTrace, VoiceSubmission};`, `use audio_playback::AudioPlayback;`, `use audio_worker::PreparationWorker;`, `use native_audio::*;`, `use wav::WavDecoder;`, `use audio_preparation::{ Admission, AudioDecoder, AudioPreparationQueue, Cancellation, FinishedPreparation, MAX_PENDING_PREPARATIONS, PreparationError, PreparationObservation, PreparationUsage, RunningPreparation, };`, `use blobs::{BlobError, BlobStore, BlobUsage, RawBlob};`, `use byte_budget::{ByteBudget, ByteLease};`, `use byte_budget::{ByteObservation, ByteUsage};`, `use image::{ImageDecoder, ImageEncoder};`, `use skin_audio::{AdmissionState, BuildAdmission, SkinAudioController};`, `use skin_editor_application::{ BlobLease, ImportError, RasterImportPort, ResourceError, ResourceStore, StagedRaster, values::{AudioMetadata, BlobRef, PixelSize}, };`, `use skin_editor_rendering::{Raster, RasterPixels};`, `use std::collections::BTreeMap;`, `use std::io::Cursor;`, `use std::sync::{Arc, Mutex};`, `use super::*;`, `use sha2::{Digest, Sha256};`

| 定义 | 位置／签名 |
| --- | --- |
| mod audio_playback | [源码:3](../../../desktop/infrastructure/src/lib.rs#L3) |
| mod audio_preparation | [源码:4](../../../desktop/infrastructure/src/lib.rs#L4) |
| mod audio_supply | [源码:5](../../../desktop/infrastructure/src/lib.rs#L5) |
| mod audio_worker | [源码:6](../../../desktop/infrastructure/src/lib.rs#L6) |
| mod background_input | [源码:7](../../../desktop/infrastructure/src/lib.rs#L7) |
| mod background_stream | [源码:9](../../../desktop/infrastructure/src/lib.rs#L9) |
| mod callback_audit | [源码:10](../../../desktop/infrastructure/src/lib.rs#L10) |
| mod native_audio | [源码:14](../../../desktop/infrastructure/src/lib.rs#L14) |
| mod output_fence | [源码:15](../../../desktop/infrastructure/src/lib.rs#L15) |
| mod separated_native | [源码:17](../../../desktop/infrastructure/src/lib.rs#L17) |
| mod separated_output | [源码:19](../../../desktop/infrastructure/src/lib.rs#L19) |
| mod submission_trace | [源码:20](../../../desktop/infrastructure/src/lib.rs#L20) |
| mod supply_output | [源码:21](../../../desktop/infrastructure/src/lib.rs#L21) |
| mod voice_handoff | [源码:23](../../../desktop/infrastructure/src/lib.rs#L23) |
| mod wav | [源码:25](../../../desktop/infrastructure/src/lib.rs#L25) |
| mod blobs | [源码:30](../../../desktop/infrastructure/src/lib.rs#L30) |
| mod byte_budget | [源码:31](../../../desktop/infrastructure/src/lib.rs#L31) |
| mod skin_audio | [源码:37](../../../desktop/infrastructure/src/lib.rs#L37) |
| struct CachedRaster | [源码:52](../../../desktop/infrastructure/src/lib.rs#L52) |
| struct Images | [源码:56](../../../desktop/infrastructure/src/lib.rs#L56) |
| impl impl Default for Images | [源码:62](../../../desktop/infrastructure/src/lib.rs#L62) |
| fn default | [源码:63](../../../desktop/infrastructure/src/lib.rs#L63) |
| impl impl ResourceStore for Images | [源码:67](../../../desktop/infrastructure/src/lib.rs#L67) |
| fn pin | [源码:68](../../../desktop/infrastructure/src/lib.rs#L68) |
| fn audio_metadata | [源码:71](../../../desktop/infrastructure/src/lib.rs#L71) |
| fn blob_import | [源码:75](../../../desktop/infrastructure/src/lib.rs#L75) |
| impl impl Images | [源码:81](../../../desktop/infrastructure/src/lib.rs#L81) |
| fn observe_rgba | [源码:82](../../../desktop/infrastructure/src/lib.rs#L82) |
| fn with_store | [源码:85](../../../desktop/infrastructure/src/lib.rs#L85) |
| fn blobs | [源码:93](../../../desktop/infrastructure/src/lib.rs#L93) |
| fn prune | [源码:96](../../../desktop/infrastructure/src/lib.rs#L96) |
| fn raster | [源码:103](../../../desktop/infrastructure/src/lib.rs#L103) |
| fn rgba_bytes | [源码:108](../../../desktop/infrastructure/src/lib.rs#L108) |
| fn rgba_peak_bytes | [源码:114](../../../desktop/infrastructure/src/lib.rs#L114) |
| fn load_raster | [源码:120](../../../desktop/infrastructure/src/lib.rs#L120) |
| fn reserve_pixels | [源码:128](../../../desktop/infrastructure/src/lib.rs#L128) |
| fn stage_bytes | [源码:164](../../../desktop/infrastructure/src/lib.rs#L164) |
| fn prepare | [源码:172](../../../desktop/infrastructure/src/lib.rs#L172) |
| fn sample_png | [源码:218](../../../desktop/infrastructure/src/lib.rs#L218) |
| fn write_png | [源码:242](../../../desktop/infrastructure/src/lib.rs#L242) |
| impl impl RasterImportPort for Images | [源码:257](../../../desktop/infrastructure/src/lib.rs#L257) |
| fn stage_png | [源码:258](../../../desktop/infrastructure/src/lib.rs#L258) |
| mod tests | [源码:282](../../../desktop/infrastructure/src/lib.rs#L282) |
| fn last_borrowed_pixels_keep_rgba_charge_after_the_cache_is_dropped | [源码:286](../../../desktop/infrastructure/src/lib.rs#L286) |
| fn counter_observers_do_not_keep_raw_or_rgba_storage_alive | [源码:305](../../../desktop/infrastructure/src/lib.rs#L305) |
| fn changed_png | [源码:327](../../../desktop/infrastructure/src/lib.rs#L327) |
| fn raw_history_survives_derivative_eviction_and_exact_reconstruction | [源码:336](../../../desktop/infrastructure/src/lib.rs#L336) |
| fn borrowed_pixels_cannot_be_evicted_or_escape_the_budget | [源码:356](../../../desktop/infrastructure/src/lib.rs#L356) |
| fn cache_target_is_soft_and_borrowed_working_sets_keep_the_hard_reservation_limit | [源码:376](../../../desktop/infrastructure/src/lib.rs#L376) |
| fn sha256_png_round_trip_and_lease_lifetime_are_real | [源码:395](../../../desktop/infrastructure/src/lib.rs#L395) |
| fn malformed_and_oversized_inputs_do_not_leave_entries | [源码:414](../../../desktop/infrastructure/src/lib.rs#L414) |

## desktop/infrastructure/src/native_audio.rs

2172 行；SHA256 `6af374ab3a0f84307b5a551ed6da3a41d19ecec86b0217864af12a2f68deb0f4`

导入：`use crate::submission_trace::SubmissionCapture;`, `use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};`, `use rtrb::{Consumer, Producer, RingBuffer};`, `use skin_editor_audio::{BackgroundTrack, PcmBudget, PcmBuffer, PcmReservation, Runtime};`, `use std::sync::{ Arc, Weak, atomic::{AtomicBool, AtomicU64, Ordering}, };`, `use crate::output_fence::OutputFence;`, `use crate::output_fence::OutputMark;`, `use super::*;`, `use skin_editor_audio::Channel;`, `use skin_editor_audio::Channel;`, `use skin_editor_audio::Channel;`, `use skin_editor_audio::Channel;`, `use skin_editor_audio::Channel;`, `use skin_editor_audio::Channel;`, `use skin_editor_audio::Channel;`, `use skin_editor_audio::Channel;`, `use skin_editor_audio::Channel;`, `use skin_editor_audio::Channel;`, `use crate::VoiceSubmission;`, `use skin_editor_audio::Channel;`, `use crate::VoiceSubmission;`, `use skin_editor_audio::Channel;`, `use skin_editor_audio::Channel;`, `use skin_editor_audio::Channel;`, `use skin_editor_audio::Channel;`, `use skin_editor_audio::Channel;`, `use skin_editor_audio::Channel;`, `use skin_editor_application::values::*;`, `use skin_editor_audio::{Origin, PreparedSound, SoundPlan};`, `use skin_editor_audio::Channel;`, `use skin_editor_audio::Channel;`, `use skin_editor_audio::Channel;`

| 定义 | 位置／签名 |
| --- | --- |
| const OUTPUT_QUEUE_FRAMES | [源码:11](../../../desktop/infrastructure/src/native_audio.rs#L11) |
| const OUTPUT_AHEAD_FRAMES | [源码:12](../../../desktop/infrastructure/src/native_audio.rs#L12) |
| const OUTPUT_BLOCK_FRAMES | [源码:13](../../../desktop/infrastructure/src/native_audio.rs#L13) |
| const STARTUP_OBSERVATIONS | [源码:14](../../../desktop/infrastructure/src/native_audio.rs#L14) |
| enum SupplyPolicy | [源码:17](../../../desktop/infrastructure/src/native_audio.rs#L17) |
| struct CallbackObservation | [源码:22](../../../desktop/infrastructure/src/native_audio.rs#L22) |
| struct UnderrunObservation | [源码:23](../../../desktop/infrastructure/src/native_audio.rs#L23) |
| struct Frame | [源码:25](../../../desktop/infrastructure/src/native_audio.rs#L25) |
| struct Shared | [源码:36](../../../desktop/infrastructure/src/native_audio.rs#L36) |
| struct OutputStats | [源码:75](../../../desktop/infrastructure/src/native_audio.rs#L75) |
| struct OutputControl | [源码:104](../../../desktop/infrastructure/src/native_audio.rs#L104) |
| struct SupplyTimingSnapshot | [源码:109](../../../desktop/infrastructure/src/native_audio.rs#L109) |
| impl impl OutputControl | [源码:119](../../../desktop/infrastructure/src/native_audio.rs#L119) |
| fn capture_origin | [源码:120](../../../desktop/infrastructure/src/native_audio.rs#L120) |
| fn supply_timing | [源码:123](../../../desktop/infrastructure/src/native_audio.rs#L123) |
| fn capture_submissions | [源码:127](../../../desktop/infrastructure/src/native_audio.rs#L127) |
| fn submission_trace | [源码:133](../../../desktop/infrastructure/src/native_audio.rs#L133) |
| fn voice_submission | [源码:138](../../../desktop/infrastructure/src/native_audio.rs#L138) |
| fn callback_progress | [源码:155](../../../desktop/infrastructure/src/native_audio.rs#L155) |
| fn set_diagnostic_stage | [源码:160](../../../desktop/infrastructure/src/native_audio.rs#L160) |
| fn diagnostic_stage_counts | [源码:165](../../../desktop/infrastructure/src/native_audio.rs#L165) |
| fn underrun_contexts | [源码:172](../../../desktop/infrastructure/src/native_audio.rs#L172) |
| fn underrun_observations | [源码:180](../../../desktop/infrastructure/src/native_audio.rs#L180) |
| fn capture_startup | [源码:192](../../../desktop/infrastructure/src/native_audio.rs#L192) |
| fn startup_observations | [源码:200](../../../desktop/infrastructure/src/native_audio.rs#L200) |
| fn set_monitor_volume | [源码:211](../../../desktop/infrastructure/src/native_audio.rs#L211) |
| fn is_closed | [源码:221](../../../desktop/infrastructure/src/native_audio.rs#L221) |
| fn mark | [源码:224](../../../desktop/infrastructure/src/native_audio.rs#L224) |
| fn invalidate_skin | [源码:232](../../../desktop/infrastructure/src/native_audio.rs#L232) |
| fn invalidate_scene | [源码:246](../../../desktop/infrastructure/src/native_audio.rs#L246) |
| fn background_acknowledged | [源码:253](../../../desktop/infrastructure/src/native_audio.rs#L253) |
| fn stats | [源码:257](../../../desktop/infrastructure/src/native_audio.rs#L257) |
| fn close | [源码:260](../../../desktop/infrastructure/src/native_audio.rs#L260) |
| struct OutputProducer | [源码:264](../../../desktop/infrastructure/src/native_audio.rs#L264) |
| struct OutputConsumer | [源码:285](../../../desktop/infrastructure/src/native_audio.rs#L285) |
| struct OutputSafety | [源码:294](../../../desktop/infrastructure/src/native_audio.rs#L294) |
| impl impl OutputSafety | [源码:295](../../../desktop/infrastructure/src/native_audio.rs#L295) |
| fn is_for | [源码:296](../../../desktop/infrastructure/src/native_audio.rs#L296) |
| fn close | [源码:299](../../../desktop/infrastructure/src/native_audio.rs#L299) |
| fn output_boundary | [源码:305](../../../desktop/infrastructure/src/native_audio.rs#L305) |
| impl impl OutputProducer | [源码:399](../../../desktop/infrastructure/src/native_audio.rs#L399) |
| fn arm_diagnostic_recovery | [源码:400](../../../desktop/infrastructure/src/native_audio.rs#L400) |
| fn cancel_diagnostic_recovery | [源码:415](../../../desktop/infrastructure/src/native_audio.rs#L415) |
| fn owner_woke | [源码:418](../../../desktop/infrastructure/src/native_audio.rs#L418) |
| fn owner_sleeping | [源码:429](../../../desktop/infrastructure/src/native_audio.rs#L429) |
| fn complete_pump | [源码:434](../../../desktop/infrastructure/src/native_audio.rs#L434) |
| fn control | [源码:459](../../../desktop/infrastructure/src/native_audio.rs#L459) |
| fn safety | [源码:462](../../../desktop/infrastructure/src/native_audio.rs#L462) |
| fn fail_device | [源码:466](../../../desktop/infrastructure/src/native_audio.rs#L466) |
| fn stats | [源码:469](../../../desktop/infrastructure/src/native_audio.rs#L469) |
| fn shared_stats | [源码:473](../../../desktop/infrastructure/src/native_audio.rs#L473) |
| impl impl OutputProducer | [源码:501](../../../desktop/infrastructure/src/native_audio.rs#L501) |
| fn configure_period | [源码:504](../../../desktop/infrastructure/src/native_audio.rs#L504) |
| fn configure_supply_policy | [源码:525](../../../desktop/infrastructure/src/native_audio.rs#L525) |
| fn idle_reserve_frames | [源码:532](../../../desktop/infrastructure/src/native_audio.rs#L532) |
| fn queued_frames | [源码:535](../../../desktop/infrastructure/src/native_audio.rs#L535) |
| fn invalidate | [源码:538](../../../desktop/infrastructure/src/native_audio.rs#L538) |
| fn acknowledged | [源码:550](../../../desktop/infrastructure/src/native_audio.rs#L550) |
| fn pump | [源码:554](../../../desktop/infrastructure/src/native_audio.rs#L554) |
| fn prefill | [源码:558](../../../desktop/infrastructure/src/native_audio.rs#L558) |
| fn pump_to | [源码:561](../../../desktop/infrastructure/src/native_audio.rs#L561) |
| fn pump_timeline | [源码:564](../../../desktop/infrastructure/src/native_audio.rs#L564) |
| impl impl OutputConsumer | [源码:764](../../../desktop/infrastructure/src/native_audio.rs#L764) |
| fn render | [源码:765](../../../desktop/infrastructure/src/native_audio.rs#L765) |
| impl impl FnMut(usize, f32)) | [源码:765](../../../desktop/infrastructure/src/native_audio.rs#L765) |
| fn render_f32 | [源码:980](../../../desktop/infrastructure/src/native_audio.rs#L980) |
| fn render_i16 | [源码:984](../../../desktop/infrastructure/src/native_audio.rs#L984) |
| struct NativeOutput | [源码:990](../../../desktop/infrastructure/src/native_audio.rs#L990) |
| struct NativeStream | [源码:999](../../../desktop/infrastructure/src/native_audio.rs#L999) |
| impl impl NativeStream | [源码:1000](../../../desktop/infrastructure/src/native_audio.rs#L1000) |
| fn clock_sample | [源码:1002](../../../desktop/infrastructure/src/native_audio.rs#L1002) |
| fn play | [源码:1015](../../../desktop/infrastructure/src/native_audio.rs#L1015) |
| impl impl NativeOutput | [源码:1019](../../../desktop/infrastructure/src/native_audio.rs#L1019) |
| fn into_parts | [源码:1020](../../../desktop/infrastructure/src/native_audio.rs#L1020) |
| fn open | [源码:1023](../../../desktop/infrastructure/src/native_audio.rs#L1023) |
| fn play | [源码:1077](../../../desktop/infrastructure/src/native_audio.rs#L1077) |
| mod tests | [源码:1082](../../../desktop/infrastructure/src/native_audio.rs#L1082) |
| fn explicit_recovery_requires_capture_policy_and_a_single_pending_probe | [源码:1085](../../../desktop/infrastructure/src/native_audio.rs#L1085) |
| fn explicit_probe_counts_real_commits_separately_from_recovery_attempts | [源码:1103](../../../desktop/infrastructure/src/native_audio.rs#L1103) |
| fn full_ring_diagnostic_attempt_is_not_misreported_as_a_completed_refill | [源码:1139](../../../desktop/infrastructure/src/native_audio.rs#L1139) |
| fn late_wake_replenishes_before_a_larger_callback_even_above_normal_target | [源码:1169](../../../desktop/infrastructure/src/native_audio.rs#L1169) |
| fn fixed_policy_preserves_the_original_larger_callback_failure | [源码:1203](../../../desktop/infrastructure/src/native_audio.rs#L1203) |
| fn recovery_is_opt_in_once_per_late_wake_and_not_a_permanent_larger_target | [源码:1230](../../../desktop/infrastructure/src/native_audio.rs#L1230) |
| fn late_idle_exit_keeps_a_bounded_blank_guard_without_consuming_voice_on_backpressure | [源码:1272](../../../desktop/infrastructure/src/native_audio.rs#L1272) |
| fn recovery_keeps_background_continuous_and_queued_lane_cancellation_independent | [源码:1318](../../../desktop/infrastructure/src/native_audio.rs#L1318) |
| fn recovery_cannot_erase_a_callback_that_drained_before_the_owner_woke | [源码:1374](../../../desktop/infrastructure/src/native_audio.rs#L1374) |
| fn recovery_caps_unknown_larger_demand_at_existing_ring_capacity | [源码:1399](../../../desktop/infrastructure/src/native_audio.rs#L1399) |
| fn callback_defers_an_incomplete_cutover_guard_pair_without_waiting | [源码:1427](../../../desktop/infrastructure/src/native_audio.rs#L1427) |
| fn diagnostic_context_identifies_three_callback_burst_and_stage_crossing | [源码:1458](../../../desktop/infrastructure/src/native_audio.rs#L1458) |
| fn diagnostic_context_is_bounded_keeps_first_events_and_total_counts | [源码:1489](../../../desktop/infrastructure/src/native_audio.rs#L1489) |
| fn completion_marker_waits_for_entire_buffer_and_preserves_epoch_identity | [源码:1511](../../../desktop/infrastructure/src/native_audio.rs#L1511) |
| fn controlled_callback_owner_phases_change_latency_without_changing_supply_policy | [源码:1552](../../../desktop/infrastructure/src/native_audio.rs#L1552) |
| fn causal_capture_distinguishes_waiting_for_generation_and_native_submission | [源码:1613](../../../desktop/infrastructure/src/native_audio.rs#L1613) |
| fn invalidation_cannot_attribute_old_copy_frames_to_captured_submission | [源码:1645](../../../desktop/infrastructure/src/native_audio.rs#L1645) |
| fn empty_callback_trace_is_bounded_and_does_not_require_startup_capture | [源码:1665](../../../desktop/infrastructure/src/native_audio.rs#L1665) |
| fn native_voice_observation_identifies_a_new_contribution_among_old_voices | [源码:1682](../../../desktop/infrastructure/src/native_audio.rs#L1682) |
| fn silent_and_invalidated_frames_cannot_publish_voice_observations | [源码:1709](../../../desktop/infrastructure/src/native_audio.rs#L1709) |
| fn malformed_marker_blocks_do_not_advance_the_mixer | [源码:1743](../../../desktop/infrastructure/src/native_audio.rs#L1743) |
| fn replay_of_observed_initial_callbacks_changes_192_empty_frames_to_zero | [源码:1758](../../../desktop/infrastructure/src/native_audio.rs#L1758) |
| fn first_trigger_waits_for_space_without_consuming_voice_or_discarding_startup_prefill | [源码:1790](../../../desktop/infrastructure/src/native_audio.rs#L1790) |
| fn cancellation_during_idle_cutover_cannot_publish_the_old_voice | [源码:1812](../../../desktop/infrastructure/src/native_audio.rs#L1812) |
| fn startup_capture_is_one_fixed_window_and_cannot_be_armed_late | [源码:1836](../../../desktop/infrastructure/src/native_audio.rs#L1836) |
| fn ready_tone | [源码:1850](../../../desktop/infrastructure/src/native_audio.rs#L1850) |
| fn configured_idle_reserve_survives_the_same_two_callback_burst | [源码:1888](../../../desktop/infrastructure/src/native_audio.rs#L1888) |
| fn idle_exit_keeps_burst_guard_but_retimes_only_obsolete_blank_frames | [源码:1900](../../../desktop/infrastructure/src/native_audio.rs#L1900) |
| fn zero_valued_valid_background_time_is_never_retimed | [源码:1922](../../../desktop/infrastructure/src/native_audio.rs#L1922) |
| fn pending_event_keeps_its_scheduled_time_and_prevents_idle_reserve | [源码:1972](../../../desktop/infrastructure/src/native_audio.rs#L1972) |
| fn replacement_batch_is_invisible_until_committed_and_cutover_published | [源码:1988](../../../desktop/infrastructure/src/native_audio.rs#L1988) |
| fn unsupported_or_late_period_and_exhausted_serial_do_not_advance_mix | [源码:2017](../../../desktop/infrastructure/src/native_audio.rs#L2017) |
| fn two_adjacent_480_frame_callbacks_reproduce_192_without_supply_between_them | [源码:2033](../../../desktop/infrastructure/src/native_audio.rs#L2033) |
| fn same_callbacks_with_supply_interleaved_have_no_empty_frames | [源码:2048](../../../desktop/infrastructure/src/native_audio.rs#L2048) |
| fn empty_callback_size_is_separate_from_frame_count_and_larger_valid_callback | [源码:2059](../../../desktop/infrastructure/src/native_audio.rs#L2059) |
| fn integer_callback_quantizes_and_saturates_at_the_output_boundary | [源码:2073](../../../desktop/infrastructure/src/native_audio.rs#L2073) |
| fn monitor_volume_applies_to_queued_sum_before_clipping_without_resetting_background | [源码:2100](../../../desktop/infrastructure/src/native_audio.rs#L2100) |
| fn native_lane_observations_reject_stale_frames_and_confirm_counters_before_epoch_ack | [源码:2139](../../../desktop/infrastructure/src/native_audio.rs#L2139) |

## desktop/infrastructure/src/output_fence.rs

16 行；SHA256 `ee205d7078d8b35955fa5dc3c673f60f98c415dc1e6cb18a1bffc8e01578315b`

导入：

| 定义 | 位置／签名 |
| --- | --- |
| struct OutputFence | [源码:3](../../../desktop/infrastructure/src/output_fence.rs#L3) |
| impl impl OutputFence | [源码:4](../../../desktop/infrastructure/src/output_fence.rs#L4) |
| fn with_scene | [源码:5](../../../desktop/infrastructure/src/output_fence.rs#L5) |
| fn with_audition | [源码:8](../../../desktop/infrastructure/src/output_fence.rs#L8) |
| struct OutputMark | [源码:13](../../../desktop/infrastructure/src/output_fence.rs#L13) |

## desktop/infrastructure/src/separated_native.rs

561 行；SHA256 `e5613c5951ce4182f771d8ade02e4b710e636be62845c29142bf85af2dfa6d27`

导入：`use crate::separated_output::{BACKGROUND_RESERVE, OutputConsumer, StoppedOutput};`, `use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};`, `use rtrb::{Consumer, Producer, RingBuffer};`, `use skin_editor_audio::Channel;`, `use std::sync::{ Arc, atomic::{AtomicBool, AtomicU16, AtomicU64, Ordering}, mpsc::{Receiver, SyncSender, sync_channel}, };`

| 定义 | 位置／签名 |
| --- | --- |
| const OBSERVATIONS | [源码:13](../../../desktop/infrastructure/src/separated_native.rs#L13) |
| struct FirstCopy | [源码:15](../../../desktop/infrastructure/src/separated_native.rs#L15) |
| struct NativeStats | [源码:24](../../../desktop/infrastructure/src/separated_native.rs#L24) |
| struct WorkCounters | [源码:40](../../../desktop/infrastructure/src/separated_native.rs#L40) |
| impl impl WorkCounters | [源码:44](../../../desktop/infrastructure/src/separated_native.rs#L44) |
| fn publish | [源码:45](../../../desktop/infrastructure/src/separated_native.rs#L45) |
| fn read | [源码:52](../../../desktop/infrastructure/src/separated_native.rs#L52) |
| struct Shared | [源码:66](../../../desktop/infrastructure/src/separated_native.rs#L66) |
| struct NativeFaultSignal | [源码:88](../../../desktop/infrastructure/src/separated_native.rs#L88) |
| impl impl NativeFaultSignal | [源码:89](../../../desktop/infrastructure/src/separated_native.rs#L89) |
| fn report | [源码:90](../../../desktop/infrastructure/src/separated_native.rs#L90) |
| impl impl Default for Shared | [源码:97](../../../desktop/infrastructure/src/separated_native.rs#L97) |
| fn default | [源码:98](../../../desktop/infrastructure/src/separated_native.rs#L98) |
| struct FirstObserver | [源码:121](../../../desktop/infrastructure/src/separated_native.rs#L121) |
| impl impl FirstObserver | [源码:122](../../../desktop/infrastructure/src/separated_native.rs#L122) |
| fn next_first | [源码:123](../../../desktop/infrastructure/src/separated_native.rs#L123) |
| impl impl Shared | [源码:127](../../../desktop/infrastructure/src/separated_native.rs#L127) |
| fn stats | [源码:128](../../../desktop/infrastructure/src/separated_native.rs#L128) |
| struct Callback | [源码:145](../../../desktop/infrastructure/src/separated_native.rs#L145) |
| impl impl Drop for Callback | [源码:158](../../../desktop/infrastructure/src/separated_native.rs#L158) |
| fn drop | [源码:159](../../../desktop/infrastructure/src/separated_native.rs#L159) |
| impl impl Callback | [源码:176](../../../desktop/infrastructure/src/separated_native.rs#L176) |
| fn render | [源码:177](../../../desktop/infrastructure/src/separated_native.rs#L177) |
| fn render_inner | [源码:205](../../../desktop/infrastructure/src/separated_native.rs#L205) |
| struct PreparedDevice | [源码:284](../../../desktop/infrastructure/src/separated_native.rs#L284) |
| struct NativeExecutor | [源码:288](../../../desktop/infrastructure/src/separated_native.rs#L288) |
| fn executor | [源码:298](../../../desktop/infrastructure/src/separated_native.rs#L298) |
| fn executor_gated | [源码:302](../../../desktop/infrastructure/src/separated_native.rs#L302) |
| enum EndpointReturnError | [源码:350](../../../desktop/infrastructure/src/separated_native.rs#L350) |
| struct ReturnGate | [源码:357](../../../desktop/infrastructure/src/separated_native.rs#L357) |
| impl impl ReturnGate | [源码:363](../../../desktop/infrastructure/src/separated_native.rs#L363) |
| fn disconnect | [源码:364](../../../desktop/infrastructure/src/separated_native.rs#L364) |
| fn deliver | [源码:367](../../../desktop/infrastructure/src/separated_native.rs#L367) |
| struct ControlledCallback | [源码:396](../../../desktop/infrastructure/src/separated_native.rs#L396) |
| impl impl ControlledCallback | [源码:397](../../../desktop/infrastructure/src/separated_native.rs#L397) |
| fn render | [源码:398](../../../desktop/infrastructure/src/separated_native.rs#L398) |
| impl impl PreparedDevice | [源码:402](../../../desktop/infrastructure/src/separated_native.rs#L402) |
| fn format | [源码:403](../../../desktop/infrastructure/src/separated_native.rs#L403) |
| fn probe | [源码:406](../../../desktop/infrastructure/src/separated_native.rs#L406) |
| fn attach | [源码:429](../../../desktop/infrastructure/src/separated_native.rs#L429) |
| fn attach_with_held_return | [源码:432](../../../desktop/infrastructure/src/separated_native.rs#L432) |
| fn attach_inner | [源码:439](../../../desktop/infrastructure/src/separated_native.rs#L439) |
| impl impl NativeExecutor | [源码:479](../../../desktop/infrastructure/src/separated_native.rs#L479) |
| fn controlled_return | [源码:480](../../../desktop/infrastructure/src/separated_native.rs#L480) |
| fn fault_signal | [源码:490](../../../desktop/infrastructure/src/separated_native.rs#L490) |
| fn capture_work_cost | [源码:494](../../../desktop/infrastructure/src/separated_native.rs#L494) |
| fn work_cost | [源码:497](../../../desktop/infrastructure/src/separated_native.rs#L497) |
| fn period_frames | [源码:500](../../../desktop/infrastructure/src/separated_native.rs#L500) |
| fn play | [源码:503](../../../desktop/infrastructure/src/separated_native.rs#L503) |
| fn stats | [源码:513](../../../desktop/infrastructure/src/separated_native.rs#L513) |
| fn next_first | [源码:516](../../../desktop/infrastructure/src/separated_native.rs#L516) |
| fn take_first_observer | [源码:519](../../../desktop/infrastructure/src/separated_native.rs#L519) |
| fn set_monitor_volume | [源码:522](../../../desktop/infrastructure/src/separated_native.rs#L522) |
| fn try_stop | [源码:531](../../../desktop/infrastructure/src/separated_native.rs#L531) |
| fn stop | [源码:549](../../../desktop/infrastructure/src/separated_native.rs#L549) |
| impl impl Drop for NativeExecutor | [源码:554](../../../desktop/infrastructure/src/separated_native.rs#L554) |
| fn drop | [源码:555](../../../desktop/infrastructure/src/separated_native.rs#L555) |
| mod tests | [源码:561](../../../desktop/infrastructure/src/separated_native.rs#L561) |

## desktop/infrastructure/src/separated_native/tests.rs

445 行；SHA256 `fd6a121eae16d1c49058ef85a1d9689a9298d00664ff1a9cae4ae49fbfb9c3da`

导入：`use super::*;`, `use crate::separated_output::{self, BackgroundOwner};`, `use skin_editor_application::values::{AudioComponentId, AudioMetadata, BlobRef, Gain};`, `use skin_editor_audio::{BackgroundTrack, Origin, PcmBudget, PreparedSound, Runtime, SoundPlan};`

| 定义 | 位置／签名 |
| --- | --- |
| fn missing_return_keeps_receiver_and_endpoint_until_the_controlled_owner_exits | [源码:6](../../../desktop/infrastructure/src/separated_native/tests.rs#L6) |
| fn disconnected_return_does_not_release_the_live_callback_or_a_failed_delivery | [源码:37](../../../desktop/infrastructure/src/separated_native/tests.rs#L37) |
| fn device_error_silences_the_next_copy_without_advancing_or_freeing_the_endpoint | [源码:68](../../../desktop/infrastructure/src/separated_native/tests.rs#L68) |
| fn retired_backend_signal_cannot_fault_the_rebuilt_endpoint | [源码:85](../../../desktop/infrastructure/src/separated_native/tests.rs#L85) |
| fn setup | [源码:99](../../../desktop/infrastructure/src/separated_native/tests.rs#L99) |
| fn trigger | [源码:137](../../../desktop/infrastructure/src/separated_native/tests.rs#L137) |
| fn finish | [源码:147](../../../desktop/infrastructure/src/separated_native/tests.rs#L147) |
| fn unstarted_and_active_closures_return_the_endpoint_before_control_releases_pcm | [源码:164](../../../desktop/infrastructure/src/separated_native/tests.rs#L164) |
| fn abandoned_return_receiver_does_not_refund_live_endpoint_storage | [源码:182](../../../desktop/infrastructure/src/separated_native/tests.rs#L182) |
| fn thirty_two_exact_voice_observations_follow_buffer_copy_and_retire_on_owner | [源码:193](../../../desktop/infrastructure/src/separated_native/tests.rs#L193) |
| fn cancel_after_handoff_never_reports_or_copies_the_cancelled_voice | [源码:240](../../../desktop/infrastructure/src/separated_native/tests.rs#L240) |
| fn callback_shape_rejection_does_not_consume_the_background_or_voice | [源码:252](../../../desktop/infrastructure/src/separated_native/tests.rs#L252) |
| fn oversized_callbacks_use_bounded_chunks_and_report_the_real_missing_reserve | [源码:266](../../../desktop/infrastructure/src/separated_native/tests.rs#L266) |
| fn observation_overflow_is_counted_and_never_reinterpreted_as_no_voice | [源码:284](../../../desktop/infrastructure/src/separated_native/tests.rs#L284) |
| fn endpoint_return_from_another_executor_does_not_close_either_owner | [源码:298](../../../desktop/infrastructure/src/separated_native/tests.rs#L298) |
| fn worker_destruction_returns_without_an_extra_render_or_sleep | [源码:312](../../../desktop/infrastructure/src/separated_native/tests.rs#L312) |
| fn monitor_volume_is_bounded_applied_after_mix_and_not_a_project_edit | [源码:334](../../../desktop/infrastructure/src/separated_native/tests.rs#L334) |
| fn detached_observer_is_unique_and_cannot_pin_endpoint_pcm | [源码:353](../../../desktop/infrastructure/src/separated_native/tests.rs#L353) |
| fn thirty_two_voice_retirements_and_cancel_do_not_destroy_managed_resources_in_render | [源码:368](../../../desktop/infrastructure/src/separated_native/tests.rs#L368) |
| fn audit_detects_accidental_endpoint_destruction_inside_a_data_callback_scope | [源码:394](../../../desktop/infrastructure/src/separated_native/tests.rs#L394) |
| fn managed_drop_detector_is_live_for_a_returned_endpoint_dropped_in_the_wrong_scope | [源码:406](../../../desktop/infrastructure/src/separated_native/tests.rs#L406) |
| fn optional_work_observation_does_not_change_samples_or_hold_resources | [源码:417](../../../desktop/infrastructure/src/separated_native/tests.rs#L417) |
| fn partial_callback_work_snapshot_is_rejected_without_spinning | [源码:439](../../../desktop/infrastructure/src/separated_native/tests.rs#L439) |

## desktop/infrastructure/src/separated_output.rs

779 行；SHA256 `ec96a2edd20e44cadb114ffc5c805967bccdebfdc54fb2f2bf060858e660e214`

导入：`use crate::background_input::BackgroundInput;`, `use crate::voice_handoff::{ self, HandoffError, HandoffReport, RuntimePumpReport, VoiceConsumer, VoiceOwner, };`, `use rtrb::{Consumer, Producer, RingBuffer};`, `use skin_editor_audio::{ BackgroundTrack, Channel, PcmBuffer, PcmReservation, Runtime, VoiceRuntimeKey, };`, `use std::sync::{ Arc, atomic::{AtomicU64, Ordering::SeqCst}, };`, `use control::Control;`

| 定义 | 位置／签名 |
| --- | --- |
| mod control | [源码:15](../../../desktop/infrastructure/src/separated_output.rs#L15) |
| const BACKGROUND_RESERVE | [源码:18](../../../desktop/infrastructure/src/separated_output.rs#L18) |
| const RUNNING | [源码:19](../../../desktop/infrastructure/src/separated_output.rs#L19) |
| const CLOSED | [源码:20](../../../desktop/infrastructure/src/separated_output.rs#L20) |
| const EPOCH_MAX | [源码:21](../../../desktop/infrastructure/src/separated_output.rs#L21) |
| fn epoch | [源码:22](../../../desktop/infrastructure/src/separated_output.rs#L22) |
| enum OutputError | [源码:27](../../../desktop/infrastructure/src/separated_output.rs#L27) |
| impl impl From<HandoffError> for OutputError | [源码:43](../../../desktop/infrastructure/src/separated_output.rs#L43) |
| fn from | [源码:44](../../../desktop/infrastructure/src/separated_output.rs#L44) |
| enum Phase | [源码:49](../../../desktop/infrastructure/src/separated_output.rs#L49) |
| struct CopyClock | [源码:56](../../../desktop/infrastructure/src/separated_output.rs#L56) |
| struct Clock | [源码:64](../../../desktop/infrastructure/src/separated_output.rs#L64) |
| impl impl Clock | [源码:72](../../../desktop/infrastructure/src/separated_output.rs#L72) |
| fn new | [源码:73](../../../desktop/infrastructure/src/separated_output.rs#L73) |
| fn read | [源码:83](../../../desktop/infrastructure/src/separated_output.rs#L83) |
| fn publish | [源码:100](../../../desktop/infrastructure/src/separated_output.rs#L100) |
| struct Shared | [源码:110](../../../desktop/infrastructure/src/separated_output.rs#L110) |
| struct Frame | [源码:120](../../../desktop/infrastructure/src/separated_output.rs#L120) |
| struct BackgroundOwner | [源码:125](../../../desktop/infrastructure/src/separated_output.rs#L125) |
| struct OutputConsumer | [源码:137](../../../desktop/infrastructure/src/separated_output.rs#L137) |
| struct StoppedOutput | [源码:154](../../../desktop/infrastructure/src/separated_output.rs#L154) |
| impl impl StoppedOutput | [源码:155](../../../desktop/infrastructure/src/separated_output.rs#L155) |
| fn copy_clock | [源码:158](../../../desktop/infrastructure/src/separated_output.rs#L158) |
| impl impl std::fmt::Debug for StoppedOutput | [源码:162](../../../desktop/infrastructure/src/separated_output.rs#L162) |
| fn fmt | [源码:163](../../../desktop/infrastructure/src/separated_output.rs#L163) |
| struct Position | [源码:168](../../../desktop/infrastructure/src/separated_output.rs#L168) |
| struct PumpReport | [源码:177](../../../desktop/infrastructure/src/separated_output.rs#L177) |
| struct OutputReport | [源码:182](../../../desktop/infrastructure/src/separated_output.rs#L182) |
| fn boundary | [源码:191](../../../desktop/infrastructure/src/separated_output.rs#L191) |
| fn boundary_input | [源码:198](../../../desktop/infrastructure/src/separated_output.rs#L198) |
| impl impl BackgroundOwner | [源码:280](../../../desktop/infrastructure/src/separated_output.rs#L280) |
| fn retained_drafts | [源码:281](../../../desktop/infrastructure/src/separated_output.rs#L281) |
| impl impl Iterator<Item = u64> + '_ | [源码:281](../../../desktop/infrastructure/src/separated_output.rs#L281) |
| fn control | [源码:284](../../../desktop/infrastructure/src/separated_output.rs#L284) |
| fn apply_mark | [源码:287](../../../desktop/infrastructure/src/separated_output.rs#L287) |
| fn validate | [源码:290](../../../desktop/infrastructure/src/separated_output.rs#L290) |
| fn position | [源码:307](../../../desktop/infrastructure/src/separated_output.rs#L307) |
| fn pump | [源码:327](../../../desktop/infrastructure/src/separated_output.rs#L327) |
| fn refill | [源码:368](../../../desktop/infrastructure/src/separated_output.rs#L368) |
| fn refill_inner | [源码:371](../../../desktop/infrastructure/src/separated_output.rs#L371) |
| impl impl FnMut()) -> Result<usize, OutputError> | [源码:371](../../../desktop/infrastructure/src/separated_output.rs#L371) |
| fn freeze | [源码:404](../../../desktop/infrastructure/src/separated_output.rs#L404) |
| fn pause | [源码:429](../../../desktop/infrastructure/src/separated_output.rs#L429) |
| fn seek | [源码:434](../../../desktop/infrastructure/src/separated_output.rs#L434) |
| fn resume | [源码:437](../../../desktop/infrastructure/src/separated_output.rs#L437) |
| fn is_primed | [源码:464](../../../desktop/infrastructure/src/separated_output.rs#L464) |
| fn close | [源码:470](../../../desktop/infrastructure/src/separated_output.rs#L470) |
| fn reclaim_abandoned | [源码:478](../../../desktop/infrastructure/src/separated_output.rs#L478) |
| fn finish | [源码:502](../../../desktop/infrastructure/src/separated_output.rs#L502) |
| impl impl Drop for BackgroundOwner | [源码:531](../../../desktop/infrastructure/src/separated_output.rs#L531) |
| fn drop | [源码:532](../../../desktop/infrastructure/src/separated_output.rs#L532) |
| impl impl Drop for OutputConsumer | [源码:537](../../../desktop/infrastructure/src/separated_output.rs#L537) |
| fn drop | [源码:538](../../../desktop/infrastructure/src/separated_output.rs#L538) |
| enum Checkpoint | [源码:544](../../../desktop/infrastructure/src/separated_output.rs#L544) |
| impl impl OutputConsumer | [源码:549](../../../desktop/infrastructure/src/separated_output.rs#L549) |
| fn control | [源码:550](../../../desktop/infrastructure/src/separated_output.rs#L550) |
| fn copy_clock | [源码:553](../../../desktop/infrastructure/src/separated_output.rs#L553) |
| fn format | [源码:556](../../../desktop/infrastructure/src/separated_output.rs#L556) |
| fn stop | [源码:559](../../../desktop/infrastructure/src/separated_output.rs#L559) |
| fn observe_freeze | [源码:563](../../../desktop/infrastructure/src/separated_output.rs#L563) |
| fn render | [源码:584](../../../desktop/infrastructure/src/separated_output.rs#L584) |
| fn render_profiled | [源码:589](../../../desktop/infrastructure/src/separated_output.rs#L589) |
| fn render_inner | [源码:610](../../../desktop/infrastructure/src/separated_output.rs#L610) |
| impl impl FnMut(Checkpoint), ) -> Result<OutputReport, OutputError> | [源码:613](../../../desktop/infrastructure/src/separated_output.rs#L613) |
| mod tests | [源码:779](../../../desktop/infrastructure/src/separated_output.rs#L779) |

## desktop/infrastructure/src/separated_output/control.rs

119 行；SHA256 `273b5c16cbfa91333e6e90407633891ae1d47d985df8638d5128d5c4b986fd39`

导入：`use crate::output_fence::{OutputFence, OutputMark};`, `use std::sync::{ Arc, atomic::{AtomicBool, AtomicU64, Ordering::SeqCst}, };`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| struct State | [源码:7](../../../desktop/infrastructure/src/separated_output/control.rs#L7) |
| impl impl State | [源码:15](../../../desktop/infrastructure/src/separated_output/control.rs#L15) |
| fn new | [源码:16](../../../desktop/infrastructure/src/separated_output/control.rs#L16) |
| fn read | [源码:26](../../../desktop/infrastructure/src/separated_output/control.rs#L26) |
| fn desired | [源码:32](../../../desktop/infrastructure/src/separated_output/control.rs#L32) |
| fn applied | [源码:35](../../../desktop/infrastructure/src/separated_output/control.rs#L35) |
| fn apply | [源码:38](../../../desktop/infrastructure/src/separated_output/control.rs#L38) |
| fn acknowledge | [源码:47](../../../desktop/infrastructure/src/separated_output/control.rs#L47) |
| struct Control | [源码:60](../../../desktop/infrastructure/src/separated_output/control.rs#L60) |
| impl impl Control | [源码:61](../../../desktop/infrastructure/src/separated_output/control.rs#L61) |
| fn is_closed | [源码:62](../../../desktop/infrastructure/src/separated_output/control.rs#L62) |
| fn close | [源码:65](../../../desktop/infrastructure/src/separated_output/control.rs#L65) |
| fn mark | [源码:68](../../../desktop/infrastructure/src/separated_output/control.rs#L68) |
| fn invalidate | [源码:71](../../../desktop/infrastructure/src/separated_output/control.rs#L71) |
| fn acknowledged | [源码:92](../../../desktop/infrastructure/src/separated_output/control.rs#L92) |
| fn background_acknowledged | [源码:99](../../../desktop/infrastructure/src/separated_output/control.rs#L99) |
| fn background_next | [源码:102](../../../desktop/infrastructure/src/separated_output/control.rs#L102) |
| mod tests | [源码:107](../../../desktop/infrastructure/src/separated_output/control.rs#L107) |
| fn exhausted_control_closes_without_wrapping_or_reopening | [源码:110](../../../desktop/infrastructure/src/separated_output/control.rs#L110) |

## desktop/infrastructure/src/separated_output/tests.rs

597 行；SHA256 `fa6bda82c6de023f508cd473a2c8a810e40cc2e7f82da484accfe4114bff846b`

导入：`use super::*;`, `use skin_editor_application::values::{AudioComponentId, AudioMetadata, BlobRef, Gain};`, `use skin_editor_audio::{Origin, PcmBudget, PreparedSound, SoundPlan};`, `use std::sync::mpsc::sync_channel;`, `use std::time::Duration;`

| 定义 | 位置／签名 |
| --- | --- |
| const GUARD | [源码:6](../../../desktop/infrastructure/src/separated_output/tests.rs#L6) |
| mod resume_race | [源码:7](../../../desktop/infrastructure/src/separated_output/tests.rs#L7) |
| mod worker_fence | [源码:8](../../../desktop/infrastructure/src/separated_output/tests.rs#L8) |
| fn selective_cancel_at_the_output_copy_boundary_preserves_only_other_contributions | [源码:10](../../../desktop/infrastructure/src/separated_output/tests.rs#L10) |
| fn origin | [源码:42](../../../desktop/infrastructure/src/separated_output/tests.rs#L42) |
| fn sample | [源码:51](../../../desktop/infrastructure/src/separated_output/tests.rs#L51) |
| fn reference | [源码:54](../../../desktop/infrastructure/src/separated_output/tests.rs#L54) |
| fn runtime | [源码:69](../../../desktop/infrastructure/src/separated_output/tests.rs#L69) |
| fn setup | [源码:74](../../../desktop/infrastructure/src/separated_output/tests.rs#L74) |
| fn add_sound | [源码:81](../../../desktop/infrastructure/src/separated_output/tests.rs#L81) |
| fn trigger | [源码:97](../../../desktop/infrastructure/src/separated_output/tests.rs#L97) |
| fn assert_background | [源码:105](../../../desktop/infrastructure/src/separated_output/tests.rs#L105) |
| fn start | [源码:110](../../../desktop/infrastructure/src/separated_output/tests.rs#L110) |
| fn reserve_frontier_and_copy_clock_are_separate_and_do_not_need_gui_ticks | [源码:116](../../../desktop/infrastructure/src/separated_output/tests.rs#L116) |
| fn one_refill_turn_is_bounded_even_if_consumer_frees_every_published_slot | [源码:139](../../../desktop/infrastructure/src/separated_output/tests.rs#L139) |
| fn matching_reference_formats_keep_frame_units_and_exact_managed_storage_cost | [源码:162](../../../desktop/infrastructure/src/separated_output/tests.rs#L162) |
| fn ready_voice_bypasses_a_full_background_reserve_before_and_after_refill | [源码:191](../../../desktop/infrastructure/src/separated_output/tests.rs#L191) |
| fn full_32_voice_load_shares_pcm_and_mixes_with_background_without_queue_delay | [源码:219](../../../desktop/infrastructure/src/separated_output/tests.rs#L219) |
| fn pause_copy_interleavings_restore_only_unsent_background_and_keep_audition | [源码:252](../../../desktop/infrastructure/src/separated_output/tests.rs#L252) |
| fn seeks_forward_backward_and_latest_request_clear_old_frames_and_stay_frozen | [源码:318](../../../desktop/infrastructure/src/separated_output/tests.rs#L318) |
| fn hot_replacement_draft_cancel_and_history_never_reset_background_generation | [源码:356](../../../desktop/infrastructure/src/separated_output/tests.rs#L356) |
| fn frozen_seek_does_not_stop_explicit_audition_or_admit_scene_input | [源码:405](../../../desktop/infrastructure/src/separated_output/tests.rs#L405) |
| fn invalid_input_is_rejected_before_mutating_output_queue_or_transport | [源码:432](../../../desktop/infrastructure/src/separated_output/tests.rs#L432) |
| fn end_of_reference_is_intentional_silence_but_exhausted_reserve_is_counted | [源码:462](../../../desktop/infrastructure/src/separated_output/tests.rs#L462) |
| fn construction_failure_releases_all_reserved_storage_and_refuses_wrong_source | [源码:480](../../../desktop/infrastructure/src/separated_output/tests.rs#L480) |
| fn shutdown_without_a_later_render_releases_pending_active_and_returned_resources | [源码:507](../../../desktop/infrastructure/src/separated_output/tests.rs#L507) |
| fn owner_drop_does_not_refund_still_live_ring_and_consumer_scratch | [源码:525](../../../desktop/infrastructure/src/separated_output/tests.rs#L525) |
| fn three_hundred_threaded_blocks_keep_clock_and_shared_resources_bounded | [源码:540](../../../desktop/infrastructure/src/separated_output/tests.rs#L540) |
| fn counter_exhaustion_closes_without_wrapping_and_partial_clock_rows_are_rejected | [源码:576](../../../desktop/infrastructure/src/separated_output/tests.rs#L576) |

## desktop/infrastructure/src/separated_output/tests/resume_race.rs

209 行；SHA256 `907415fbc38d2b5cdb419cc4780c0c6782551699a194adf6e091a0357e7b3588`

导入：`use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| fn resume_then_pause_before_first_running_copy_keeps_the_explicit_seek_anchor | [源码:4](../../../desktop/infrastructure/src/separated_output/tests/resume_race.rs#L4) |
| fn frozen_callback_finishing_after_resume_cannot_restore_pre_seek_scene_position | [源码:53](../../../desktop/infrastructure/src/separated_output/tests/resume_race.rs#L53) |
| fn abandoned_consumer_reclaims_source_ring_scratch_and_runtime_on_owner | [源码:94](../../../desktop/infrastructure/src/separated_output/tests/resume_race.rs#L94) |
| fn normal_voice_return_is_not_visible_before_composed_buffer_copy | [源码:109](../../../desktop/infrastructure/src/separated_output/tests/resume_race.rs#L109) |
| fn three_hundred_pause_seek_resume_cycles_keep_pcm_budget_and_old_frame_cleanup_stable | [源码:143](../../../desktop/infrastructure/src/separated_output/tests/resume_race.rs#L143) |
| fn runtime_shutdown_before_copy_suppresses_both_lanes_background_and_first_receipts | [源码:174](../../../desktop/infrastructure/src/separated_output/tests/resume_race.rs#L174) |

## desktop/infrastructure/src/separated_output/tests/worker_fence.rs

97 行；SHA256 `5dd944287e743b6929bc5a0d6151ca49b29482e2b46c30c71a271c17d1a24f5c`

导入：`use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| fn initial_copy_anchor_is_visible_before_any_callback | [源码:3](../../../desktop/infrastructure/src/separated_output/tests/worker_fence.rs#L3) |
| fn pending_pause_skips_dequeue_until_owner_freezes_and_acknowledges_it | [源码:16](../../../desktop/infrastructure/src/separated_output/tests/worker_fence.rs#L16) |
| fn applying_new_fence_during_copy_cannot_relabel_old_mixed_samples | [源码:42](../../../desktop/infrastructure/src/separated_output/tests/worker_fence.rs#L42) |
| fn out_of_band_close_suppresses_pending_copy_and_control_handle_keeps_no_pcm | [源码:75](../../../desktop/infrastructure/src/separated_output/tests/worker_fence.rs#L75) |

## desktop/infrastructure/src/skin_audio.rs

262 行；SHA256 `6693cfa9db0da5609cee4c1a116dff5d7d8fde5e05ae9fdd8d7e318e4b905463`

导入：`use crate::{ Admission, AudioPreparationQueue, BlobStore, FinishedPreparation, PreparationError, PreparationUsage, RunningPreparation, };`, `use skin_editor_application::{ DraftId, ProjectSession, SkinAudioError, SkinAudioPort, SkinAudioSource, values::SkinProject, };`, `use skin_editor_audio::{ BuildRequest, Completion, MAX_BINDINGS, Origin, PcmBudget, PrepareFailure, PreparedSound, Runtime, RuntimeError, SoundPlan, };`

| 定义 | 位置／签名 |
| --- | --- |
| struct SkinAudioController | [源码:14](../../../desktop/infrastructure/src/skin_audio.rs#L14) |
| impl impl SkinAudioController | [源码:21](../../../desktop/infrastructure/src/skin_audio.rs#L21) |
| fn new | [源码:22](../../../desktop/infrastructure/src/skin_audio.rs#L22) |
| fn with_budget | [源码:25](../../../desktop/infrastructure/src/skin_audio.rs#L25) |
| fn runtime | [源码:37](../../../desktop/infrastructure/src/skin_audio.rs#L37) |
| fn output_invalidation | [源码:40](../../../desktop/infrastructure/src/skin_audio.rs#L40) |
| fn take_builds | [源码:44](../../../desktop/infrastructure/src/skin_audio.rs#L44) |
| fn admit_builds | [源码:48](../../../desktop/infrastructure/src/skin_audio.rs#L48) |
| fn admit_selected | [源码:56](../../../desktop/infrastructure/src/skin_audio.rs#L56) |
| fn request_library | [源码:92](../../../desktop/infrastructure/src/skin_audio.rs#L92) |
| fn cancel_library | [源码:108](../../../desktop/infrastructure/src/skin_audio.rs#L108) |
| fn start_preparation | [源码:111](../../../desktop/infrastructure/src/skin_audio.rs#L111) |
| fn retry_preparation | [源码:114](../../../desktop/infrastructure/src/skin_audio.rs#L114) |
| fn retry_selected | [源码:122](../../../desktop/infrastructure/src/skin_audio.rs#L122) |
| fn preparation_usage | [源码:133](../../../desktop/infrastructure/src/skin_audio.rs#L133) |
| fn finish_preparation | [源码:136](../../../desktop/infrastructure/src/skin_audio.rs#L136) |
| fn finish_selected | [源码:144](../../../desktop/infrastructure/src/skin_audio.rs#L144) |
| fn close | [源码:156](../../../desktop/infrastructure/src/skin_audio.rs#L156) |
| fn accept_prepared | [源码:164](../../../desktop/infrastructure/src/skin_audio.rs#L164) |
| impl impl SkinAudioController | [源码:176](../../../desktop/infrastructure/src/skin_audio.rs#L176) |
| fn select | [源码:177](../../../desktop/infrastructure/src/skin_audio.rs#L177) |
| impl impl SkinAudioPort for SkinAudioController | [源码:216](../../../desktop/infrastructure/src/skin_audio.rs#L216) |
| fn retained_drafts | [源码:217](../../../desktop/infrastructure/src/skin_audio.rs#L217) |
| fn synchronize | [源码:224](../../../desktop/infrastructure/src/skin_audio.rs#L224) |
| enum AdmissionState | [源码:253](../../../desktop/infrastructure/src/skin_audio.rs#L253) |
| struct BuildAdmission | [源码:259](../../../desktop/infrastructure/src/skin_audio.rs#L259) |

## desktop/infrastructure/src/submission_trace.rs

364 行；SHA256 `d6e6eac1212b1a379257e7bd705c697e847beac9fc7f61b136f70708780a2f96`

导入：`use std::sync::{ OnceLock, atomic::{AtomicU64, Ordering}, };`, `use std::time::Instant;`, `use super::*;`, `use std::time::Duration;`

| 定义 | 位置／签名 |
| --- | --- |
| struct Row | [源码:8](../../../desktop/infrastructure/src/submission_trace.rs#L8) |
| const N | [源码:8](../../../desktop/infrastructure/src/submission_trace.rs#L8) |
| impl impl<const N: usize> Default for Row<N> | [源码:12](../../../desktop/infrastructure/src/submission_trace.rs#L12) |
| const N | [源码:12](../../../desktop/infrastructure/src/submission_trace.rs#L12) |
| fn default | [源码:13](../../../desktop/infrastructure/src/submission_trace.rs#L13) |
| impl impl<const N: usize> Row<N> | [源码:20](../../../desktop/infrastructure/src/submission_trace.rs#L20) |
| const N | [源码:20](../../../desktop/infrastructure/src/submission_trace.rs#L20) |
| fn write | [源码:22](../../../desktop/infrastructure/src/submission_trace.rs#L22) |
| fn read | [源码:29](../../../desktop/infrastructure/src/submission_trace.rs#L29) |
| struct SubmissionCapture | [源码:44](../../../desktop/infrastructure/src/submission_trace.rs#L44) |
| enum VoiceSubmission | [源码:62](../../../desktop/infrastructure/src/submission_trace.rs#L62) |
| struct SubmissionTrace | [源码:71](../../../desktop/infrastructure/src/submission_trace.rs#L71) |
| impl impl SubmissionCapture | [源码:80](../../../desktop/infrastructure/src/submission_trace.rs#L80) |
| fn begin | [源码:81](../../../desktop/infrastructure/src/submission_trace.rs#L81) |
| fn origin | [源码:88](../../../desktop/infrastructure/src/submission_trace.rs#L88) |
| fn produced | [源码:91](../../../desktop/infrastructure/src/submission_trace.rs#L91) |
| fn submitted | [源码:94](../../../desktop/infrastructure/src/submission_trace.rs#L94) |
| fn completed_callback | [源码:97](../../../desktop/infrastructure/src/submission_trace.rs#L97) |
| fn callback_progress | [源码:100](../../../desktop/infrastructure/src/submission_trace.rs#L100) |
| fn set_stage | [源码:103](../../../desktop/infrastructure/src/submission_trace.rs#L103) |
| fn stage | [源码:110](../../../desktop/infrastructure/src/submission_trace.rs#L110) |
| fn completed_context | [源码:113](../../../desktop/infrastructure/src/submission_trace.rs#L113) |
| fn completed_supply | [源码:150](../../../desktop/infrastructure/src/submission_trace.rs#L150) |
| fn recent_supply | [源码:161](../../../desktop/infrastructure/src/submission_trace.rs#L161) |
| fn scheduling | [源码:169](../../../desktop/infrastructure/src/submission_trace.rs#L169) |
| fn stage_counts | [源码:188](../../../desktop/infrastructure/src/submission_trace.rs#L188) |
| fn underrun_contexts | [源码:191](../../../desktop/infrastructure/src/submission_trace.rs#L191) |
| fn observe | [源码:200](../../../desktop/infrastructure/src/submission_trace.rs#L200) |
| fn read | [源码:214](../../../desktop/infrastructure/src/submission_trace.rs#L214) |
| struct StreamClockSample | [源码:227](../../../desktop/infrastructure/src/submission_trace.rs#L227) |
| impl impl StreamClockSample | [源码:232](../../../desktop/infrastructure/src/submission_trace.rs#L232) |
| fn callback_bounds_ms | [源码:233](../../../desktop/infrastructure/src/submission_trace.rs#L233) |
| mod tests | [源码:252](../../../desktop/infrastructure/src/submission_trace.rs#L252) |
| fn scheduling_joins_exact_completed_pass_and_rejects_torn_or_overwritten_rows | [源码:256](../../../desktop/infrastructure/src/submission_trace.rs#L256) |
| fn scheduling_overflow_keeps_counts_and_first_event_windows | [源码:284](../../../desktop/infrastructure/src/submission_trace.rs#L284) |
| fn exact_completion_does_not_accept_newer_or_invalidated_voice_or_torn_row | [源码:300](../../../desktop/infrastructure/src/submission_trace.rs#L300) |
| fn partial_row_is_rejected_without_waiting_for_writer | [源码:320](../../../desktop/infrastructure/src/submission_trace.rs#L320) |
| fn exact_voice_epoch_serial_and_bounded_capture_are_required | [源码:332](../../../desktop/infrastructure/src/submission_trace.rs#L332) |
| fn calibration_keeps_clock_uncertainty_and_overlapping_callback | [源码:347](../../../desktop/infrastructure/src/submission_trace.rs#L347) |

## desktop/infrastructure/src/supply_output.rs

71 行；SHA256 `e18b4cb53aec6292b185a37df4b6a0f824ee7efe2bbe1089e1105b5f215e2f0f`

导入：`use crate::{OutputControl, output_fence::OutputMark, separated_output};`

| 定义 | 位置／签名 |
| --- | --- |
| enum SupplyOutputControl | [源码:4](../../../desktop/infrastructure/src/supply_output.rs#L4) |
| impl impl SupplyOutputControl | [源码:8](../../../desktop/infrastructure/src/supply_output.rs#L8) |
| fn is_separated | [源码:9](../../../desktop/infrastructure/src/supply_output.rs#L9) |
| fn premixed | [源码:12](../../../desktop/infrastructure/src/supply_output.rs#L12) |
| fn capture_origin | [源码:20](../../../desktop/infrastructure/src/supply_output.rs#L20) |
| fn mark | [源码:23](../../../desktop/infrastructure/src/supply_output.rs#L23) |
| fn invalidate_skin | [源码:29](../../../desktop/infrastructure/src/supply_output.rs#L29) |
| fn invalidate_scene | [源码:35](../../../desktop/infrastructure/src/supply_output.rs#L35) |
| fn background_acknowledged | [源码:41](../../../desktop/infrastructure/src/supply_output.rs#L41) |
| fn acknowledged | [源码:47](../../../desktop/infrastructure/src/supply_output.rs#L47) |
| fn background_next | [源码:53](../../../desktop/infrastructure/src/supply_output.rs#L53) |
| fn is_closed | [源码:59](../../../desktop/infrastructure/src/supply_output.rs#L59) |
| fn close | [源码:65](../../../desktop/infrastructure/src/supply_output.rs#L65) |

## desktop/infrastructure/src/voice_handoff.rs

761 行；SHA256 `a9fb57dbc556bc0939334abecdaafe2bf77a03cee1a1df78a1d4db159ee16a5a`

导入：`use rtrb::{Consumer, Producer, PushError, RingBuffer};`, `use skin_editor_audio::{ Channel, MAX_VOICES, MonitorFormat, Origin, PcmBudget, PcmBuffer, PreparedSound, Runtime, VoiceHandoff, VoiceRuntimeKey, VoiceValidity, };`, `use std::sync::atomic::{AtomicU64, Ordering};`, `use std::sync::{Arc, Weak};`

| 定义 | 位置／签名 |
| --- | --- |
| const MAX_HANDOFF_FRAMES | [源码:12](../../../desktop/infrastructure/src/voice_handoff.rs#L12) |
| const EPOCH_BITS | [源码:13](../../../desktop/infrastructure/src/voice_handoff.rs#L13) |
| const EPOCH_MAX | [源码:14](../../../desktop/infrastructure/src/voice_handoff.rs#L14) |
| const RUNNING | [源码:15](../../../desktop/infrastructure/src/voice_handoff.rs#L15) |
| const CLOSED | [源码:16](../../../desktop/infrastructure/src/voice_handoff.rs#L16) |
| fn index | [源码:17](../../../desktop/infrastructure/src/voice_handoff.rs#L17) |
| fn epoch | [源码:23](../../../desktop/infrastructure/src/voice_handoff.rs#L23) |
| fn permits | [源码:26](../../../desktop/infrastructure/src/voice_handoff.rs#L26) |
| fn valid_origin | [源码:31](../../../desktop/infrastructure/src/voice_handoff.rs#L31) |
| enum HandoffError | [源码:36](../../../desktop/infrastructure/src/voice_handoff.rs#L36) |
| struct VoiceToken | [源码:50](../../../desktop/infrastructure/src/voice_handoff.rs#L50) |
| struct VoiceKey | [源码:56](../../../desktop/infrastructure/src/voice_handoff.rs#L56) |
| struct Shared | [源码:61](../../../desktop/infrastructure/src/voice_handoff.rs#L61) |
| struct Packet | [源码:64](../../../desktop/infrastructure/src/voice_handoff.rs#L64) |
| impl impl Drop for Packet | [源码:73](../../../desktop/infrastructure/src/voice_handoff.rs#L73) |
| fn drop | [源码:74](../../../desktop/infrastructure/src/voice_handoff.rs#L74) |
| struct Pin | [源码:78](../../../desktop/infrastructure/src/voice_handoff.rs#L78) |
| struct VoiceOwner | [源码:84](../../../desktop/infrastructure/src/voice_handoff.rs#L84) |
| struct VoiceConsumer | [源码:95](../../../desktop/infrastructure/src/voice_handoff.rs#L95) |
| struct StoppedVoiceConsumer | [源码:106](../../../desktop/infrastructure/src/voice_handoff.rs#L106) |
| impl impl std::fmt::Debug for StoppedVoiceConsumer | [源码:107](../../../desktop/infrastructure/src/voice_handoff.rs#L107) |
| fn fmt | [源码:108](../../../desktop/infrastructure/src/voice_handoff.rs#L108) |
| struct HandoffReport | [源码:114](../../../desktop/infrastructure/src/voice_handoff.rs#L114) |
| struct RuntimePumpReport | [源码:126](../../../desktop/infrastructure/src/voice_handoff.rs#L126) |
| fn boundary | [源码:135](../../../desktop/infrastructure/src/voice_handoff.rs#L135) |
| impl impl VoiceOwner | [源码:175](../../../desktop/infrastructure/src/voice_handoff.rs#L175) |
| fn retained_drafts | [源码:176](../../../desktop/infrastructure/src/voice_handoff.rs#L176) |
| impl impl Iterator<Item = u64> + '_ | [源码:176](../../../desktop/infrastructure/src/voice_handoff.rs#L176) |
| fn token | [源码:182](../../../desktop/infrastructure/src/voice_handoff.rs#L182) |
| fn outstanding | [源码:189](../../../desktop/infrastructure/src/voice_handoff.rs#L189) |
| fn admit | [源码:192](../../../desktop/infrastructure/src/voice_handoff.rs#L192) |
| fn publish | [源码:203](../../../desktop/infrastructure/src/voice_handoff.rs#L203) |
| fn pump_runtime | [源码:269](../../../desktop/infrastructure/src/voice_handoff.rs#L269) |
| fn update | [源码:327](../../../desktop/infrastructure/src/voice_handoff.rs#L327) |
| fn set_scene_running | [源码:369](../../../desktop/infrastructure/src/voice_handoff.rs#L369) |
| fn stop_audition | [源码:379](../../../desktop/infrastructure/src/voice_handoff.rs#L379) |
| fn invalidate_source | [源码:383](../../../desktop/infrastructure/src/voice_handoff.rs#L383) |
| fn close | [源码:396](../../../desktop/infrastructure/src/voice_handoff.rs#L396) |
| fn collect | [源码:399](../../../desktop/infrastructure/src/voice_handoff.rs#L399) |
| fn collect_with | [源码:407](../../../desktop/infrastructure/src/voice_handoff.rs#L407) |
| impl impl FnMut(VoiceHandoff)) -> usize | [源码:407](../../../desktop/infrastructure/src/voice_handoff.rs#L407) |
| fn finish | [源码:443](../../../desktop/infrastructure/src/voice_handoff.rs#L443) |
| fn finish_runtime | [源码:462](../../../desktop/infrastructure/src/voice_handoff.rs#L462) |
| impl impl Drop for VoiceOwner | [源码:480](../../../desktop/infrastructure/src/voice_handoff.rs#L480) |
| fn drop | [源码:481](../../../desktop/infrastructure/src/voice_handoff.rs#L481) |
| impl impl Drop for VoiceConsumer | [源码:486](../../../desktop/infrastructure/src/voice_handoff.rs#L486) |
| fn drop | [源码:487](../../../desktop/infrastructure/src/voice_handoff.rs#L487) |
| enum Checkpoint | [源码:495](../../../desktop/infrastructure/src/voice_handoff.rs#L495) |
| impl impl VoiceConsumer | [源码:500](../../../desktop/infrastructure/src/voice_handoff.rs#L500) |
| fn format | [源码:501](../../../desktop/infrastructure/src/voice_handoff.rs#L501) |
| fn stop | [源码:504](../../../desktop/infrastructure/src/voice_handoff.rs#L504) |
| fn drain_pending | [源码:508](../../../desktop/infrastructure/src/voice_handoff.rs#L508) |
| fn return_packet | [源码:518](../../../desktop/infrastructure/src/voice_handoff.rs#L518) |
| fn retire_slot | [源码:525](../../../desktop/infrastructure/src/voice_handoff.rs#L525) |
| fn retire_all | [源码:530](../../../desktop/infrastructure/src/voice_handoff.rs#L530) |
| fn flush_deferred | [源码:535](../../../desktop/infrastructure/src/voice_handoff.rs#L535) |
| fn render_split | [源码:545](../../../desktop/infrastructure/src/voice_handoff.rs#L545) |
| fn render_inner | [源码:552](../../../desktop/infrastructure/src/voice_handoff.rs#L552) |
| impl impl FnMut(Checkpoint), ) -> Result<HandoffReport, HandoffError> | [源码:556](../../../desktop/infrastructure/src/voice_handoff.rs#L556) |
| fn render_to | [源码:562](../../../desktop/infrastructure/src/voice_handoff.rs#L562) |
| impl impl FnMut(), copy: impl FnOnce(&[f32], &[f32], &mut HandoffReport), ) -> Result<HandoffReport, HandoffError> | [源码:566](../../../desktop/infrastructure/src/voice_handoff.rs#L566) |
| impl impl FnOnce(&[f32], &[f32], &mut HandoffReport), ) -> Result<HandoffReport, HandoffError> | [源码:567](../../../desktop/infrastructure/src/voice_handoff.rs#L567) |
| fn render_committed | [源码:580](../../../desktop/infrastructure/src/voice_handoff.rs#L580) |
| impl impl FnMut(Checkpoint), copy: impl FnOnce(&[f32], &[f32], &mut HandoffReport), ) -> Result<HandoffReport, HandoffError> | [源码:584](../../../desktop/infrastructure/src/voice_handoff.rs#L584) |
| impl impl FnOnce(&[f32], &[f32], &mut HandoffReport), ) -> Result<HandoffReport, HandoffError> | [源码:585](../../../desktop/infrastructure/src/voice_handoff.rs#L585) |
| mod tests | [源码:761](../../../desktop/infrastructure/src/voice_handoff.rs#L761) |

## desktop/infrastructure/src/voice_handoff/tests.rs

653 行；SHA256 `c6e7ee2bfb9f09acdf83cdad44cadbd7b3f5c7a5313b21a9167ca7c3d1ccfed2`

导入：`use super::*;`, `use skin_editor_application::values::{AudioComponentId, AudioMetadata, BlobRef, Gain};`, `use skin_editor_audio::{Runtime, SoundPlan};`, `use std::sync::mpsc::sync_channel;`, `use std::time::Duration;`

| 定义 | 位置／签名 |
| --- | --- |
| const GUARD | [源码:7](../../../desktop/infrastructure/src/voice_handoff/tests.rs#L7) |
| mod runtime_bridge | [源码:8](../../../desktop/infrastructure/src/voice_handoff/tests.rs#L8) |
| fn origin | [源码:9](../../../desktop/infrastructure/src/voice_handoff/tests.rs#L9) |
| fn plan | [源码:18](../../../desktop/infrastructure/src/voice_handoff/tests.rs#L18) |
| fn ready | [源码:27](../../../desktop/infrastructure/src/voice_handoff/tests.rs#L27) |
| fn pair | [源码:30](../../../desktop/infrastructure/src/voice_handoff/tests.rs#L30) |
| fn admit | [源码:33](../../../desktop/infrastructure/src/voice_handoff/tests.rs#L33) |
| fn ready_handoff_and_shared_gain_match_existing_runtime_for_same_format | [源码:40](../../../desktop/infrastructure/src/voice_handoff/tests.rs#L40) |
| fn voice_completion_is_exact_at_block_end_partial_tail_and_zero_length_callbacks | [源码:81](../../../desktop/infrastructure/src/voice_handoff/tests.rs#L81) |
| fn thirty_two_voice_block_visits_preserve_each_sample_sum_bits_and_mixed_ends | [源码:118](../../../desktop/infrastructure/src/voice_handoff/tests.rs#L118) |
| fn saturated_pending_active_and_retired_credits_never_block_stop_or_reuse_early | [源码:170](../../../desktop/infrastructure/src/voice_handoff/tests.rs#L170) |
| fn full_source_and_boundary_identity_reject_stale_or_foreign_requests | [源码:214](../../../desktop/infrastructure/src/voice_handoff/tests.rs#L214) |
| fn same_pool_format_and_origin_failures_leave_no_published_voice_or_pin | [源码:246](../../../desktop/infrastructure/src/voice_handoff/tests.rs#L246) |
| fn pause_resume_reject_frozen_inputs_and_leave_audition_valid | [源码:294](../../../desktop/infrastructure/src/voice_handoff/tests.rs#L294) |
| fn latched_control_changes_reject_drained_and_already_generated_old_output | [源码:327](../../../desktop/infrastructure/src/voice_handoff/tests.rs#L327) |
| fn stopping_without_any_further_callback_reclaims_pending_active_and_returned_packets | [源码:416](../../../desktop/infrastructure/src/voice_handoff/tests.rs#L416) |
| fn unexpected_consumer_destruction_is_detected_and_orphans_are_released_by_owner | [源码:443](../../../desktop/infrastructure/src/voice_handoff/tests.rs#L443) |
| fn foreign_stop_receipt_is_returned_intact_and_does_not_release_another_owners_credit | [源码:471](../../../desktop/infrastructure/src/voice_handoff/tests.rs#L471) |
| fn invalid_output_blocks_never_consume_packets_or_advance_voice_position | [源码:488](../../../desktop/infrastructure/src/voice_handoff/tests.rs#L488) |
| fn generation_exhaustion_fails_closed_or_rejects_without_aliasing_or_allocating | [源码:519](../../../desktop/infrastructure/src/voice_handoff/tests.rs#L519) |
| fn concurrent_retirement_and_slot_reuse_do_not_alias_inflight_reports_or_new_pcm | [源码:548](../../../desktop/infrastructure/src/voice_handoff/tests.rs#L548) |
| fn three_hundred_threaded_rounds_keep_32_credits_and_release_each_shared_allocation | [源码:602](../../../desktop/infrastructure/src/voice_handoff/tests.rs#L602) |

## desktop/infrastructure/src/voice_handoff/tests/runtime_bridge.rs

696 行；SHA256 `7974d4245e0132abf66130242e9523944dfaf46d39cab84eba5150181ba3219f`

导入：`use super::*;`, `use skin_editor_audio::{Completion, PrepareFailure, RuntimeError};`

| 定义 | 位置／签名 |
| --- | --- |
| fn selected | [源码:4](../../../desktop/infrastructure/src/voice_handoff/tests/runtime_bridge.rs#L4) |
| fn trigger | [源码:21](../../../desktop/infrastructure/src/voice_handoff/tests/runtime_bridge.rs#L21) |
| fn replacement | [源码:30](../../../desktop/infrastructure/src/voice_handoff/tests/runtime_bridge.rs#L30) |
| fn live_audition_gain_including_zero_and_cancel_preserves_native_cursor_and_pcm_pin | [源码:39](../../../desktop/infrastructure/src/voice_handoff/tests/runtime_bridge.rs#L39) |
| fn scene_activation_uses_runtime_authority_and_preserves_started_old_versions | [源码:87](../../../desktop/infrastructure/src/voice_handoff/tests/runtime_bridge.rs#L87) |
| fn latched_draft_cancel_removes_only_its_contribution_even_after_short_voice_ended | [源码:139](../../../desktop/infrastructure/src/voice_handoff/tests/runtime_bridge.rs#L139) |
| fn same_node_history_navigation_revokes_both_lanes_without_another_owner_pump | [源码:204](../../../desktop/infrastructure/src/voice_handoff/tests/runtime_bridge.rs#L204) |
| fn runtime_pause_resume_keeps_audition_cursor_and_rejects_frozen_scene_input | [源码:254](../../../desktop/infrastructure/src/voice_handoff/tests/runtime_bridge.rs#L254) |
| fn retired_transport_credits_bound_new_runtime_voices_and_never_lose_busy_requests | [源码:292](../../../desktop/infrastructure/src/voice_handoff/tests/runtime_bridge.rs#L292) |
| fn bridge_rejects_mode_pool_format_and_exhaustion_before_exporting_any_voice | [源码:348](../../../desktop/infrastructure/src/voice_handoff/tests/runtime_bridge.rs#L348) |
| fn foreign_runtime_cannot_drain_returns_or_close_the_bound_runtime | [源码:383](../../../desktop/infrastructure/src/voice_handoff/tests/runtime_bridge.rs#L383) |
| fn failed_publish_can_retry_and_abandoned_consumer_does_not_leak_runtime_credits | [源码:410](../../../desktop/infrastructure/src/voice_handoff/tests/runtime_bridge.rs#L410) |
| fn binding_runtime_rejects_raw_voices_and_does_not_adopt_an_existing_raw_queue | [源码:438](../../../desktop/infrastructure/src/voice_handoff/tests/runtime_bridge.rs#L438) |
| fn runtime_lane_cancellation_at_each_fence_keeps_the_other_lane_exact | [源码:458](../../../desktop/infrastructure/src/voice_handoff/tests/runtime_bridge.rs#L458) |
| fn selective_replay_preserves_float_summation_instead_of_subtracting_old_contribution | [源码:512](../../../desktop/infrastructure/src/voice_handoff/tests/runtime_bridge.rs#L512) |
| fn stop_without_a_further_callback_releases_runtime_pending_active_and_returned_pcm | [源码:581](../../../desktop/infrastructure/src/voice_handoff/tests/runtime_bridge.rs#L581) |
| fn shared_sampler_matches_runtime_across_all_minimum_conversion_pairs | [源码:604](../../../desktop/infrastructure/src/voice_handoff/tests/runtime_bridge.rs#L604) |
| fn three_hundred_runtime_rounds_share_pcm_and_release_authoritative_voice_credits | [源码:650](../../../desktop/infrastructure/src/voice_handoff/tests/runtime_bridge.rs#L650) |

## desktop/infrastructure/src/wav.rs

123 行；SHA256 `756bc6a2d7c02f3667d8595d7a29b31801d6544d8e80a2385cd1026c498f223b`

导入：`use crate::{AudioDecoder, BlobError, BlobStore, Cancellation, RawBlob};`, `use skin_editor_application::values::AudioMetadata;`, `use skin_editor_audio::PrepareFailure;`, `use PrepareFailure::{InvalidPcm, Unsupported};`, `use PrepareFailure::{InvalidPcm, Unsupported};`

| 定义 | 位置／签名 |
| --- | --- |
| struct WavDecoder | [源码:7](../../../desktop/infrastructure/src/wav.rs#L7) |
| struct Wave | [源码:8](../../../desktop/infrastructure/src/wav.rs#L8) |
| fn u16le | [源码:12](../../../desktop/infrastructure/src/wav.rs#L12) |
| fn u32le | [源码:15](../../../desktop/infrastructure/src/wav.rs#L15) |
| fn pcm16_format | [源码:18](../../../desktop/infrastructure/src/wav.rs#L18) |
| fn parse | [源码:39](../../../desktop/infrastructure/src/wav.rs#L39) |
| impl impl AudioDecoder for WavDecoder | [源码:87](../../../desktop/infrastructure/src/wav.rs#L87) |
| fn probe | [源码:88](../../../desktop/infrastructure/src/wav.rs#L88) |
| fn decode_into | [源码:91](../../../desktop/infrastructure/src/wav.rs#L91) |
| impl impl BlobStore | [源码:111](../../../desktop/infrastructure/src/wav.rs#L111) |
| fn stage_wav | [源码:113](../../../desktop/infrastructure/src/wav.rs#L113) |

## desktop/infrastructure/tests/audio_a2.rs

823 行；SHA256 `dfdc0feb363f97361e8df46d1f7c78c3287a5ea52056b74054bd362a244226d2`

导入：`use skin_editor_application::{values::*, *};`, `use skin_editor_audio::*;`, `use skin_editor_infrastructure::*;`, `use std::sync::mpsc;`

| 定义 | 位置／签名 |
| --- | --- |
| fn wave | [源码:6](../../../desktop/infrastructure/tests/audio_a2.rs#L6) |
| fn origin | [源码:26](../../../desktop/infrastructure/tests/audio_a2.rs#L26) |
| fn id | [源码:35](../../../desktop/infrastructure/tests/audio_a2.rs#L35) |
| fn plan | [源码:38](../../../desktop/infrastructure/tests/audio_a2.rs#L38) |
| fn prepared | [源码:47](../../../desktop/infrastructure/tests/audio_a2.rs#L47) |
| fn real_wav_minimum_matrix_decodes_signed_samples_and_preserves_original_bytes | [源码:54](../../../desktop/infrastructure/tests/audio_a2.rs#L54) |
| fn corrupt_and_unsupported_wav_never_publish_partial_resources | [源码:76](../../../desktop/infrastructure/tests/audio_a2.rs#L76) |
| fn riff_unknown_padded_chunks_are_skipped_but_duplicate_data_is_rejected | [源码:97](../../../desktop/infrastructure/tests/audio_a2.rs#L97) |
| fn minimum_monitor_conversion_is_explicit_and_keeps_source_pcm_in_its_budget | [源码:109](../../../desktop/infrastructure/tests/audio_a2.rs#L109) |
| fn native_consumer_fences_scene_without_cutting_audition_and_releases_ring_budget | [源码:149](../../../desktop/infrastructure/tests/audio_a2.rs#L149) |
| fn output_backpressure_does_not_advance_mix_and_underrun_outputs_explicit_silence | [源码:204](../../../desktop/infrastructure/tests/audio_a2.rs#L204) |
| fn output_device_fault_mutes_prequeued_samples_and_requires_explicit_rebuild | [源码:221](../../../desktop/infrastructure/tests/audio_a2.rs#L221) |
| fn dropping_playback_mutes_copies_even_if_native_output_outlives_the_control_owner | [源码:248](../../../desktop/infrastructure/tests/audio_a2.rs#L248) |
| fn native_boundary_combines_unclipped_lanes_before_saturation | [源码:275](../../../desktop/infrastructure/tests/audio_a2.rs#L275) |
| struct Latched | [源码:317](../../../desktop/infrastructure/tests/audio_a2.rs#L317) |
| impl impl AudioDecoder for Latched | [源码:322](../../../desktop/infrastructure/tests/audio_a2.rs#L322) |
| fn probe | [源码:323](../../../desktop/infrastructure/tests/audio_a2.rs#L323) |
| fn decode_into | [源码:326](../../../desktop/infrastructure/tests/audio_a2.rs#L326) |
| fn background_worker_cancel_at_latch_rejects_real_wav_and_returns_all_leases | [源码:344](../../../desktop/infrastructure/tests/audio_a2.rs#L344) |
| struct PanickingDecoder | [源码:381](../../../desktop/infrastructure/tests/audio_a2.rs#L381) |
| impl impl AudioDecoder for PanickingDecoder | [源码:382](../../../desktop/infrastructure/tests/audio_a2.rs#L382) |
| fn probe | [源码:383](../../../desktop/infrastructure/tests/audio_a2.rs#L383) |
| fn decode_into | [源码:386](../../../desktop/infrastructure/tests/audio_a2.rs#L386) |
| fn decoder_panic_releases_running_allocations_but_is_never_a_successful_join | [源码:398](../../../desktop/infrastructure/tests/audio_a2.rs#L398) |
| fn a_late_real_worker_result_cannot_override_a_newer_project_recipe | [源码:426](../../../desktop/infrastructure/tests/audio_a2.rs#L426) |
| fn real_stereo_ramp_downmix_and_resampling_match_independent_reference | [源码:493](../../../desktop/infrastructure/tests/audio_a2.rs#L493) |
| fn project | [源码:519](../../../desktop/infrastructure/tests/audio_a2.rs#L519) |
| fn draft | [源码:555](../../../desktop/infrastructure/tests/audio_a2.rs#L555) |
| fn three_hundred_real_wav_draft_versions_activate_and_retire_old_events_voices_and_pcm | [源码:566](../../../desktop/infrastructure/tests/audio_a2.rs#L566) |
| fn actual_draft_cancel_and_history_switch_fence_native_copies_without_ui_refresh | [源码:659](../../../desktop/infrastructure/tests/audio_a2.rs#L659) |
| fn library_uses_shared_cache_admission_and_holds_cancelled_active_credit_until_release | [源码:754](../../../desktop/infrastructure/tests/audio_a2.rs#L754) |

## desktop/infrastructure/tests/resources.rs

706 行；SHA256 `2be297fa3f7fae2130009680cc0a48ad079ff2ca50186ebe35302fbcdf23834d`

导入：`use skin_editor_application::{values::*, *};`, `use skin_editor_audio::*;`, `use skin_editor_infrastructure::*;`, `use std::io::{Cursor, Read, Seek, SeekFrom};`, `use std::io::Write;`

| 定义 | 位置／签名 |
| --- | --- |
| struct Decoder | [源码:7](../../../desktop/infrastructure/tests/resources.rs#L7) |
| impl impl AudioDecoder for Decoder | [源码:11](../../../desktop/infrastructure/tests/resources.rs#L11) |
| fn probe | [源码:12](../../../desktop/infrastructure/tests/resources.rs#L12) |
| fn decode_into | [源码:18](../../../desktop/infrastructure/tests/resources.rs#L18) |
| fn decoder | [源码:36](../../../desktop/infrastructure/tests/resources.rs#L36) |
| fn raw | [源码:42](../../../desktop/infrastructure/tests/resources.rs#L42) |
| fn origin | [源码:47](../../../desktop/infrastructure/tests/resources.rs#L47) |
| fn jobs | [源码:56](../../../desktop/infrastructure/tests/resources.rs#L56) |
| fn original_png_and_opaque_bytes_share_one_store_and_pins_do_not_own_rgba | [源码:72](../../../desktop/infrastructure/tests/resources.rs#L72) |
| fn full_raw_pool_reuses_known_bytes_and_reader_without_reallocating | [源码:97](../../../desktop/infrastructure/tests/resources.rs#L97) |
| struct ChangedReader | [源码:115](../../../desktop/infrastructure/tests/resources.rs#L115) |
| impl impl Read for ChangedReader | [源码:119](../../../desktop/infrastructure/tests/resources.rs#L119) |
| fn read | [源码:120](../../../desktop/infrastructure/tests/resources.rs#L120) |
| impl impl Seek for ChangedReader | [源码:124](../../../desktop/infrastructure/tests/resources.rs#L124) |
| fn seek | [源码:125](../../../desktop/infrastructure/tests/resources.rs#L125) |
| fn changed_or_truncated_readers_return_reservations_without_registering_partial_bytes | [源码:134](../../../desktop/infrastructure/tests/resources.rs#L134) |
| fn adopted_capacity_counts_and_unknown_audio_is_preserved_without_false_metadata | [源码:158](../../../desktop/infrastructure/tests/resources.rs#L158) |
| fn request | [源码:176](../../../desktop/infrastructure/tests/resources.rs#L176) |
| fn history_pins_original_bytes_even_after_active_asset_removal_and_session_drop_releases | [源码:186](../../../desktop/infrastructure/tests/resources.rs#L186) |
| fn one_worker_and_four_pending_tasks_refuse_excess_before_reserving_or_decoding | [源码:211](../../../desktop/infrastructure/tests/resources.rs#L211) |
| fn pcm_over_budget_rejects_before_decoder_and_replacement_refunds_pending_candidate | [源码:238](../../../desktop/infrastructure/tests/resources.rs#L238) |
| fn running_cancel_keeps_credit_until_ack_and_never_starts_a_second_worker | [源码:261](../../../desktop/infrastructure/tests/resources.rs#L261) |
| fn decode_failure_and_discarded_result_release_pcm_and_raw_pins | [源码:286](../../../desktop/infrastructure/tests/resources.rs#L286) |
| fn ready_cache_reuses_pcm_across_components_at_full_pool_and_never_pins_it_forever | [源码:316](../../../desktop/infrastructure/tests/resources.rs#L316) |
| fn missing_metadata_cannot_enter_typed_audio_preparation_or_modify_project_on_failure | [源码:339](../../../desktop/infrastructure/tests/resources.rs#L339) |
| fn pending_cancellation_releases_raw_and_pcm_credit_immediately | [源码:374](../../../desktop/infrastructure/tests/resources.rs#L374) |
| struct CancelDuring | [源码:388](../../../desktop/infrastructure/tests/resources.rs#L388) |
| impl impl AudioDecoder for CancelDuring<'_> | [源码:392](../../../desktop/infrastructure/tests/resources.rs#L392) |
| fn probe | [源码:393](../../../desktop/infrastructure/tests/resources.rs#L393) |
| fn decode_into | [源码:396](../../../desktop/infrastructure/tests/resources.rs#L396) |
| fn cancellation_during_decode_discards_output_even_if_decoder_returns_success | [源码:410](../../../desktop/infrastructure/tests/resources.rs#L410) |
| struct PartialDecoder | [源码:432](../../../desktop/infrastructure/tests/resources.rs#L432) |
| impl impl AudioDecoder for PartialDecoder | [源码:433](../../../desktop/infrastructure/tests/resources.rs#L433) |
| fn probe | [源码:434](../../../desktop/infrastructure/tests/resources.rs#L434) |
| fn decode_into | [源码:437](../../../desktop/infrastructure/tests/resources.rs#L437) |
| fn partial_decode_success_is_rejected_and_refunds_all_resources | [源码:450](../../../desktop/infrastructure/tests/resources.rs#L450) |
| fn foreign_completion_cannot_steal_an_active_slot_with_the_same_local_sequence | [源码:467](../../../desktop/infrastructure/tests/resources.rs#L467) |
| fn project | [源码:488](../../../desktop/infrastructure/tests/resources.rs#L488) |
| fn real_controller_reports_each_admission_and_reuses_cached_pcm_on_explicit_retry | [源码:523](../../../desktop/infrastructure/tests/resources.rs#L523) |
| fn real_draft_cancellation_rejects_finished_job_without_ui_tick_and_returns_pcm | [源码:554](../../../desktop/infrastructure/tests/resources.rs#L554) |
| fn original_bytes_registered_only_in_a_cancelled_draft_are_reclaimable | [源码:609](../../../desktop/infrastructure/tests/resources.rs#L609) |
| fn png_file_import_keeps_source_unchanged_on_success_and_decode_failure | [源码:671](../../../desktop/infrastructure/tests/resources.rs#L671) |
| struct Cleanup | [源码:686](../../../desktop/infrastructure/tests/resources.rs#L686) |
| impl impl Drop for Cleanup | [源码:687](../../../desktop/infrastructure/tests/resources.rs#L687) |
| fn drop | [源码:688](../../../desktop/infrastructure/tests/resources.rs#L688) |

## desktop/infrastructure/tests/skin_audio.rs

325 行；SHA256 `4e341c2e48ae324622d610b26cc498c287aea9f8e50a731481513da503d297c1`

导入：`use skin_editor_application::{values::*, *};`, `use skin_editor_audio::*;`, `use skin_editor_infrastructure::SkinAudioController;`, `use std::sync::Arc;`

| 定义 | 位置／签名 |
| --- | --- |
| fn component | [源码:6](../../../desktop/infrastructure/tests/skin_audio.rs#L6) |
| fn metadata | [源码:9](../../../desktop/infrastructure/tests/skin_audio.rs#L9) |
| struct Lease | [源码:12](../../../desktop/infrastructure/tests/skin_audio.rs#L12) |
| impl impl BlobLease for Lease | [源码:13](../../../desktop/infrastructure/tests/skin_audio.rs#L13) |
| fn reference | [源码:14](../../../desktop/infrastructure/tests/skin_audio.rs#L14) |
| struct Store | [源码:18](../../../desktop/infrastructure/tests/skin_audio.rs#L18) |
| impl impl ResourceStore for Store | [源码:19](../../../desktop/infrastructure/tests/skin_audio.rs#L19) |
| fn pin | [源码:20](../../../desktop/infrastructure/tests/skin_audio.rs#L20) |
| fn audio_metadata | [源码:23](../../../desktop/infrastructure/tests/skin_audio.rs#L23) |
| struct Driver | [源码:27](../../../desktop/infrastructure/tests/skin_audio.rs#L27) |
| impl impl Driver | [源码:32](../../../desktop/infrastructure/tests/skin_audio.rs#L32) |
| fn new | [源码:33](../../../desktop/infrastructure/tests/skin_audio.rs#L33) |
| fn id | [源码:66](../../../desktop/infrastructure/tests/skin_audio.rs#L66) |
| fn commit | [源码:71](../../../desktop/infrastructure/tests/skin_audio.rs#L71) |
| fn nav | [源码:81](../../../desktop/infrastructure/tests/skin_audio.rs#L81) |
| fn draft | [源码:91](../../../desktop/infrastructure/tests/skin_audio.rs#L91) |
| fn ds | [源码:100](../../../desktop/infrastructure/tests/skin_audio.rs#L100) |
| fn begin | [源码:107](../../../desktop/infrastructure/tests/skin_audio.rs#L107) |
| fn sync | [源码:112](../../../desktop/infrastructure/tests/skin_audio.rs#L112) |
| fn job | [源码:117](../../../desktop/infrastructure/tests/skin_audio.rs#L117) |
| fn accept | [源码:123](../../../desktop/infrastructure/tests/skin_audio.rs#L123) |
| fn fire | [源码:134](../../../desktop/infrastructure/tests/skin_audio.rs#L134) |
| fn ready | [源码:141](../../../desktop/infrastructure/tests/skin_audio.rs#L141) |
| fn properties | [源码:147](../../../desktop/infrastructure/tests/skin_audio.rs#L147) |
| fn cancellation_is_seen_before_late_completion_without_an_intervening_ui_refresh | [源码:156](../../../desktop/infrastructure/tests/skin_audio.rs#L156) |
| fn history_change_then_new_branch_rejects_old_worker_and_restores_branch_recipe | [源码:198](../../../desktop/infrastructure/tests/skin_audio.rs#L198) |
| fn returning_to_same_node_between_syncs_still_cleans_both_channels | [源码:222](../../../desktop/infrastructure/tests/skin_audio.rs#L222) |
| fn draft_commit_keeps_old_voice_origin_but_new_trigger_uses_committed_source | [源码:244](../../../desktop/infrastructure/tests/skin_audio.rs#L244) |
| fn frozen_scene_can_update_sound_and_audition_without_advancing_scene_or_project | [源码:267](../../../desktop/infrastructure/tests/skin_audio.rs#L267) |
| fn repeated_sync_keeps_untaken_jobs_and_hidden_edit_does_not_wait_for_decoder | [源码:288](../../../desktop/infrastructure/tests/skin_audio.rs#L288) |
| fn cancel_followed_by_unrelated_commit_cannot_keep_a_cancelled_draft_voice | [源码:302](../../../desktop/infrastructure/tests/skin_audio.rs#L302) |

## desktop/preview/src/lib.rs

75 行；SHA256 `c519ed1f4e1afa8d570ad14b66ddb1b8291d0de98dca38e599ba1cc8d6e204b6`

导入：`use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| enum Scene | [源码:4](../../../desktop/preview/src/lib.rs#L4) |
| struct PreviewState | [源码:9](../../../desktop/preview/src/lib.rs#L9) |
| impl impl Default for PreviewState | [源码:17](../../../desktop/preview/src/lib.rs#L17) |
| fn default | [源码:18](../../../desktop/preview/src/lib.rs#L18) |
| impl impl PreviewState | [源码:29](../../../desktop/preview/src/lib.rs#L29) |
| fn advance | [源码:30](../../../desktop/preview/src/lib.rs#L30) |
| fn input | [源码:35](../../../desktop/preview/src/lib.rs#L35) |
| fn micros | [源码:43](../../../desktop/preview/src/lib.rs#L43) |
| fn accepted_inputs | [源码:46](../../../desktop/preview/src/lib.rs#L46) |
| mod tests | [源码:51](../../../desktop/preview/src/lib.rs#L51) |
| fn freeze_drops_input_and_resume_does_not_accumulate_elapsed_time | [源码:54](../../../desktop/preview/src/lib.rs#L54) |

## desktop/rendering/src/gpu.rs

553 行；SHA256 `faf66adcf12bd092ec6152c204077a68f80f99ed2dd1eeb4c544410d37076510`

导入：`use crate::upload::ReusableUpload;`, `use crate::{Images, Op, Plan, RenderError, SourceUpload, StagingUsage, error};`, `use std::sync::{ Arc, Mutex, OnceLock, Weak, atomic::{AtomicBool, AtomicU64, Ordering}, };`, `use std::time::Instant;`, `use wgpu::util::DeviceExt;`, `use super::*;`, `use skin_editor_domain::{Affine2, BlobRef, PixelSize};`

| 定义 | 位置／签名 |
| --- | --- |
| struct GpuPreparation | [源码:10](../../../desktop/rendering/src/gpu.rs#L10) |
| struct GpuImage | [源码:24](../../../desktop/rendering/src/gpu.rs#L24) |
| struct GpuComposer | [源码:32](../../../desktop/rendering/src/gpu.rs#L32) |
| type PixelOffsets | [源码:41](../../../desktop/rendering/src/gpu.rs#L41) |
| type SourceIdentity | [源码:42](../../../desktop/rendering/src/gpu.rs#L42) |
| struct CachedSource | [源码:43](../../../desktop/rendering/src/gpu.rs#L43) |
| fn same_sources | [源码:49](../../../desktop/rendering/src/gpu.rs#L49) |
| fn pack_pixels | [源码:59](../../../desktop/rendering/src/gpu.rs#L59) |
| impl impl GpuComposer | [源码:100](../../../desktop/rendering/src/gpu.rs#L100) |
| fn new | [源码:101](../../../desktop/rendering/src/gpu.rs#L101) |
| fn with_upload | [源码:104](../../../desktop/rendering/src/gpu.rs#L104) |
| fn staging_usage | [源码:127](../../../desktop/rendering/src/gpu.rs#L127) |
| fn clear_sources | [源码:130](../../../desktop/rendering/src/gpu.rs#L130) |
| fn source_usage | [源码:138](../../../desktop/rendering/src/gpu.rs#L138) |
| fn source_allocations | [源码:148](../../../desktop/rendering/src/gpu.rs#L148) |
| fn source_buffer | [源码:151](../../../desktop/rendering/src/gpu.rs#L151) |
| fn render | [源码:269](../../../desktop/rendering/src/gpu.rs#L269) |
| impl impl FnOnce() + Send + 'static, ) -> Result<GpuImage, RenderError> | [源码:276](../../../desktop/rendering/src/gpu.rs#L276) |
| fn floats | [源码:288](../../../desktop/rendering/src/gpu.rs#L288) |
| impl impl IntoIterator<Item = f64>, ) -> Result<(), RenderError> | [源码:290](../../../desktop/rendering/src/gpu.rs#L290) |
| const SHADER | [源码:435](../../../desktop/rendering/src/gpu.rs#L435) |
| mod tests | [源码:469](../../../desktop/rendering/src/gpu.rs#L469) |
| fn gpu_source_identity_is_weak_and_cannot_confuse_equal_blob_labels | [源码:473](../../../desktop/rendering/src/gpu.rs#L473) |
| fn repeated_sources_are_packed_once_and_over_budget_inputs_are_rejected | [源码:499](../../../desktop/rendering/src/gpu.rs#L499) |
| fn empty_and_malformed_rasters_keep_explicit_upload_bounds | [源码:528](../../../desktop/rendering/src/gpu.rs#L528) |

## desktop/rendering/src/lib.rs

324 行；SHA256 `f1cc267f1074d40905b1633fd27e46ce6d0a4ce35dd6f8967ce8994c5d33856b`

导入：`use gpu::{GpuComposer, GpuImage, GpuPreparation};`, `use pixels::RasterPixels;`, `use skin_editor_domain::{ Affine2, BlobRef, ComponentKind, DocumentId, LayerId, LayerKind, PixelSize, SkinProject, };`, `use std::collections::BTreeMap;`, `use std::sync::Arc;`, `use upload::{MAX_STAGING_BYTES, SourceUpload, StagingUsage};`

| 定义 | 位置／签名 |
| --- | --- |
| mod gpu | [源码:4](../../../desktop/rendering/src/lib.rs#L4) |
| mod pixels | [源码:5](../../../desktop/rendering/src/lib.rs#L5) |
| mod upload | [源码:6](../../../desktop/rendering/src/lib.rs#L6) |
| const MAX_DEPTH | [源码:16](../../../desktop/rendering/src/lib.rs#L16) |
| const MAX_PIXELS | [源码:17](../../../desktop/rendering/src/lib.rs#L17) |
| const MAX_OPS | [源码:18](../../../desktop/rendering/src/lib.rs#L18) |
| struct Raster | [源码:20](../../../desktop/rendering/src/lib.rs#L20) |
| type Images | [源码:24](../../../desktop/rendering/src/lib.rs#L24) |
| enum Target | [源码:26](../../../desktop/rendering/src/lib.rs#L26) |
| enum Op | [源码:32](../../../desktop/rendering/src/lib.rs#L32) |
| struct Plan | [源码:44](../../../desktop/rendering/src/lib.rs#L44) |
| struct RenderError | [源码:50](../../../desktop/rendering/src/lib.rs#L50) |
| fn error | [源码:51](../../../desktop/rendering/src/lib.rs#L51) |
| fn math | [源码:54](../../../desktop/rendering/src/lib.rs#L54) |
| impl impl Plan | [源码:58](../../../desktop/rendering/src/lib.rs#L58) |
| fn build | [源码:59](../../../desktop/rendering/src/lib.rs#L59) |
| fn document | [源码:123](../../../desktop/rendering/src/lib.rs#L123) |
| enum Visit | [源码:143](../../../desktop/rendering/src/lib.rs#L143) |
| fn sources | [源码:210](../../../desktop/rendering/src/lib.rs#L210) |
| fn linear | [源码:224](../../../desktop/rendering/src/lib.rs#L224) |
| fn gamma | [源码:231](../../../desktop/rendering/src/lib.rs#L231) |
| fn over | [源码:238](../../../desktop/rendering/src/lib.rs#L238) |
| fn inside | [源码:241](../../../desktop/rendering/src/lib.rs#L241) |
| fn compose | [源码:244](../../../desktop/rendering/src/lib.rs#L244) |

## desktop/rendering/src/pixels.rs

74 行；SHA256 `aef38cb0f676ff263eb7a40b3570264398215baba60c9668a2b0f0bcc4420b45`

导入：`use std::fmt;`, `use std::ops::Deref;`, `use std::sync::Arc;`, `use super::*;`, `use std::sync::atomic::{AtomicUsize, Ordering};`

| 定义 | 位置／签名 |
| --- | --- |
| struct Allocation | [源码:6](../../../desktop/rendering/src/pixels.rs#L6) |
| struct RasterPixels | [源码:12](../../../desktop/rendering/src/pixels.rs#L12) |
| impl impl RasterPixels | [源码:13](../../../desktop/rendering/src/pixels.rs#L13) |
| fn with_residency | [源码:14](../../../desktop/rendering/src/pixels.rs#L14) |
| impl impl Send + Sync + 'static) -> Self | [源码:14](../../../desktop/rendering/src/pixels.rs#L14) |
| fn is_shared | [源码:20](../../../desktop/rendering/src/pixels.rs#L20) |
| impl impl From<Vec<u8>> for RasterPixels | [源码:24](../../../desktop/rendering/src/pixels.rs#L24) |
| fn from | [源码:25](../../../desktop/rendering/src/pixels.rs#L25) |
| impl impl Deref for RasterPixels | [源码:32](../../../desktop/rendering/src/pixels.rs#L32) |
| type Target | [源码:33](../../../desktop/rendering/src/pixels.rs#L33) |
| fn deref | [源码:34](../../../desktop/rendering/src/pixels.rs#L34) |
| impl impl AsRef<[u8]> for RasterPixels | [源码:38](../../../desktop/rendering/src/pixels.rs#L38) |
| fn as_ref | [源码:39](../../../desktop/rendering/src/pixels.rs#L39) |
| impl impl fmt::Debug for RasterPixels | [源码:43](../../../desktop/rendering/src/pixels.rs#L43) |
| fn fmt | [源码:44](../../../desktop/rendering/src/pixels.rs#L44) |
| mod tests | [源码:51](../../../desktop/rendering/src/pixels.rs#L51) |
| struct Lease | [源码:54](../../../desktop/rendering/src/pixels.rs#L54) |
| impl impl Drop for Lease | [源码:55](../../../desktop/rendering/src/pixels.rs#L55) |
| fn drop | [源码:56](../../../desktop/rendering/src/pixels.rs#L56) |
| fn cloned_pixels_share_bytes_and_release_once_after_last_borrow | [源码:61](../../../desktop/rendering/src/pixels.rs#L61) |

## desktop/rendering/src/upload.rs

241 行；SHA256 `e6797b54b1268cf7a45874b405a65488bc1c59b8b4196e30910edea5b55dac45`

导入：`use crate::{RenderError, error};`, `use std::sync::{ Arc, atomic::{AtomicU8, AtomicU64, Ordering}, };`, `use std::time::{Duration, Instant};`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| const MAX_STAGING_BYTES | [源码:9](../../../desktop/rendering/src/upload.rs#L9) |
| enum SourceUpload | [源码:11](../../../desktop/rendering/src/upload.rs#L11) |
| struct StagingUsage | [源码:17](../../../desktop/rendering/src/upload.rs#L17) |
| struct Budget | [源码:25](../../../desktop/rendering/src/upload.rs#L25) |
| struct Reservation | [源码:32](../../../desktop/rendering/src/upload.rs#L32) |
| impl impl Budget | [源码:36](../../../desktop/rendering/src/upload.rs#L36) |
| fn reserve | [源码:37](../../../desktop/rendering/src/upload.rs#L37) |
| fn usage | [源码:54](../../../desktop/rendering/src/upload.rs#L54) |
| impl impl Drop for Reservation | [源码:64](../../../desktop/rendering/src/upload.rs#L64) |
| fn drop | [源码:65](../../../desktop/rendering/src/upload.rs#L65) |
| struct Allocation | [源码:69](../../../desktop/rendering/src/upload.rs#L69) |
| enum State | [源码:74](../../../desktop/rendering/src/upload.rs#L74) |
| struct Slot | [源码:79](../../../desktop/rendering/src/upload.rs#L79) |
| impl impl Slot | [源码:84](../../../desktop/rendering/src/upload.rs#L84) |
| fn map | [源码:85](../../../desktop/rendering/src/upload.rs#L85) |
| struct ReusableUpload | [源码:120](../../../desktop/rendering/src/upload.rs#L120) |
| struct UploadTiming | [源码:125](../../../desktop/rendering/src/upload.rs#L125) |
| impl impl ReusableUpload | [源码:130](../../../desktop/rendering/src/upload.rs#L130) |
| fn usage | [源码:131](../../../desktop/rendering/src/upload.rs#L131) |
| fn clear | [源码:134](../../../desktop/rendering/src/upload.rs#L134) |
| fn write | [源码:137](../../../desktop/rendering/src/upload.rs#L137) |
| mod tests | [源码:209](../../../desktop/rendering/src/upload.rs#L209) |
| fn shared_completion_lease_prevents_early_refund_after_clear | [源码:212](../../../desktop/rendering/src/upload.rs#L212) |
| fn invalid_and_over_limit_reservations_do_not_change_usage | [源码:228](../../../desktop/rendering/src/upload.rs#L228) |

## desktop/rendering/tests/composition.rs

361 行；SHA256 `06ede5662f3ce8d60188dd1483ecdc8ace3e3cda165e68958d466d4e73a43d0f`

导入：`use skin_editor_domain::*;`, `use skin_editor_rendering::*;`, `use std::collections::BTreeMap;`, `use std::sync::Arc;`

| 定义 | 位置／签名 |
| --- | --- |
| fn fixture | [源码:5](../../../desktop/rendering/tests/composition.rs#L5) |
| fn group_opacity_is_isolated_and_not_applied_to_each_child | [源码:67](../../../desktop/rendering/tests/composition.rs#L67) |
| fn workspace_draws_unassociated_documents_without_creating_components_or_changing_outputs | [源码:80](../../../desktop/rendering/tests/composition.rs#L80) |
| fn missing_source_and_preview_budget_fail_instead_of_rendering_partial_pixels | [源码:114](../../../desktop/rendering/tests/composition.rs#L114) |
| fn hidden_child_removes_only_its_contribution_and_hidden_group_removes_subtree | [源码:123](../../../desktop/rendering/tests/composition.rs#L123) |
| fn hidden_element_keeps_output_and_canvas_but_needs_no_source_pixels | [源码:172](../../../desktop/rendering/tests/composition.rs#L172) |
| fn component_group_translation_changes_document_pixels_but_preserves_output_clip | [源码:203](../../../desktop/rendering/tests/composition.rs#L203) |
| fn excessive_group_depth_is_explicit_error | [源码:296](../../../desktop/rendering/tests/composition.rs#L296) |
| fn excessive_operation_count_does_not_publish_a_truncated_image | [源码:332](../../../desktop/rendering/tests/composition.rs#L332) |

## desktop/ui/src/audio.rs

529 行；SHA256 `e8109da672e373df36e5e0af037a9e18716edcfab90dfb655766cd5ebe1bb6cb`

导入：`use super::*;`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| struct CounterLabels | [源码:5](../../../desktop/ui/src/audio.rs#L5) |
| impl impl CounterLabels | [源码:10](../../../desktop/ui/src/audio.rs#L10) |
| fn update | [源码:11](../../../desktop/ui/src/audio.rs#L11) |
| impl impl EditorUi | [源码:50](../../../desktop/ui/src/audio.rs#L50) |
| fn scene_is_frozen | [源码:51](../../../desktop/ui/src/audio.rs#L51) |
| fn open_audio_editor | [源码:55](../../../desktop/ui/src/audio.rs#L55) |
| fn set_audio_port | [源码:58](../../../desktop/ui/src/audio.rs#L58) |
| fn synchronize_audio | [源码:62](../../../desktop/ui/src/audio.rs#L62) |
| fn refresh_audio | [源码:69](../../../desktop/ui/src/audio.rs#L69) |
| fn audio_needs_poll | [源码:83](../../../desktop/ui/src/audio.rs#L83) |
| fn audio_command | [源码:100](../../../desktop/ui/src/audio.rs#L100) |
| fn create_audio_component | [源码:119](../../../desktop/ui/src/audio.rs#L119) |
| fn source_label | [源码:141](../../../desktop/ui/src/audio.rs#L141) |
| fn audio_panel | [源码:154](../../../desktop/ui/src/audio.rs#L154) |
| mod counter_tests | [源码:474](../../../desktop/ui/src/audio.rs#L474) |
| fn observation | [源码:476](../../../desktop/ui/src/audio.rs#L476) |
| fn passive_readouts_are_bounded_while_live_observation_keeps_advancing | [源码:490](../../../desktop/ui/src/audio.rs#L490) |
| fn underrun_freeze_reset_format_and_clock_changes_are_never_hidden_by_cache | [源码:505](../../../desktop/ui/src/audio.rs#L505) |

## desktop/ui/src/gestures.rs

360 行；SHA256 `bf77f1f30831035a2868b5e81b00dd9bf5c70b442a5839b96e0d1af3f29aeb05`

导入：`use super::*;`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| enum Handle | [源码:5](../../../desktop/ui/src/gestures.rs#L5) |
| struct Gesture | [源码:11](../../../desktop/ui/src/gestures.rs#L11) |
| impl impl Gesture | [源码:19](../../../desktop/ui/src/gestures.rs#L19) |
| fn new | [源码:20](../../../desktop/ui/src/gestures.rs#L20) |
| fn transform_at | [源码:50](../../../desktop/ui/src/gestures.rs#L50) |
| fn pointer_delta | [源码:60](../../../desktop/ui/src/gestures.rs#L60) |
| fn frame_delta_at | [源码:67](../../../desktop/ui/src/gestures.rs#L67) |
| fn transform | [源码:71](../../../desktop/ui/src/gestures.rs#L71) |
| fn transform_with_aspect | [源码:74](../../../desktop/ui/src/gestures.rs#L74) |
| fn frame_delta | [源码:87](../../../desktop/ui/src/gestures.rs#L87) |
| fn geometry | [源码:151](../../../desktop/ui/src/gestures.rs#L151) |
| struct ControlBox | [源码:206](../../../desktop/ui/src/gestures.rs#L206) |
| impl impl From<Rect> for ControlBox | [源码:210](../../../desktop/ui/src/gestures.rs#L210) |
| fn from | [源码:211](../../../desktop/ui/src/gestures.rs#L211) |
| impl impl ControlBox | [源码:222](../../../desktop/ui/src/gestures.rs#L222) |
| fn contains | [源码:223](../../../desktop/ui/src/gestures.rs#L223) |
| fn center | [源码:235](../../../desktop/ui/src/gestures.rs#L235) |
| fn top | [源码:238](../../../desktop/ui/src/gestures.rs#L238) |
| fn hit | [源码:242](../../../desktop/ui/src/gestures.rs#L242) |
| impl impl Fn(Pos2) -> [f64; 2], ) -> Handle | [源码:246](../../../desktop/ui/src/gestures.rs#L246) |
| fn rotation | [源码:261](../../../desktop/ui/src/gestures.rs#L261) |
| impl impl Into<ControlBox>, canvas: Rect) -> Pos2 | [源码:261](../../../desktop/ui/src/gestures.rs#L261) |
| mod tests | [源码:276](../../../desktop/ui/src/gestures.rs#L276) |
| fn near | [源码:278](../../../desktop/ui/src/gestures.rs#L278) |
| fn corner_scaling_keeps_the_opposite_anchor_under_a_transformed_parent | [源码:284](../../../desktop/ui/src/gestures.rs#L284) |
| fn rotation_preserves_center_and_unstable_zero_vectors_are_rejected | [源码:302](../../../desktop/ui/src/gestures.rs#L302) |
| fn proportional_scaling_preserves_local_ratio_under_reflection_and_shear | [源码:327](../../../desktop/ui/src/gestures.rs#L327) |

## desktop/ui/src/history.rs

95 行；SHA256 `d8f8d5563af26176a3cfa2c42b4f897c801be5b0987ca68de8d7812b79afc1b0`

导入：`use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| impl impl EditorUi | [源码:5](../../../desktop/ui/src/history.rs#L5) |
| fn history_panel | [源码:6](../../../desktop/ui/src/history.rs#L6) |

## desktop/ui/src/lib.rs

1121 行；SHA256 `5d72e0c1c9f0be8058e14ba372ec743ecc93485e0ab638d53b757fccc4def289`

导入：`use selection::EditGesture;`, `use shortcuts::{ DEFAULT_SHORTCUTS, EditorAction, MODIFIER_HINTS, ShortcutBinding, ShortcutScope, };`, `use observation::UiPhase;`, `use observation::phase;`, `use egui::{Color32, Pos2, Rect, Sense, Stroke, Vec2};`, `use skin_editor_application::{values::*, *};`

| 定义 | 位置／签名 |
| --- | --- |
| mod audio | [源码:2](../../../desktop/ui/src/lib.rs#L2) |
| mod gestures | [源码:3](../../../desktop/ui/src/lib.rs#L3) |
| mod history | [源码:4](../../../desktop/ui/src/lib.rs#L4) |
| mod picking | [源码:5](../../../desktop/ui/src/lib.rs#L5) |
| mod selection | [源码:6](../../../desktop/ui/src/lib.rs#L6) |
| mod shortcuts | [源码:7](../../../desktop/ui/src/lib.rs#L7) |
| mod observation | [源码:12](../../../desktop/ui/src/lib.rs#L12) |
| mod tests | [源码:16](../../../desktop/ui/src/lib.rs#L16) |
| enum Selection | [源码:21](../../../desktop/ui/src/lib.rs#L21) |
| struct PresentedView | [源码:26](../../../desktop/ui/src/lib.rs#L26) |
| trait Presenter | [源码:31](../../../desktop/ui/src/lib.rs#L31) |
| fn current | [源码:32](../../../desktop/ui/src/lib.rs#L32) |
| fn paint | [源码:33](../../../desktop/ui/src/lib.rs#L33) |
| fn thumbnail | [源码:34](../../../desktop/ui/src/lib.rs#L34) |
| struct UiOutput | [源码:37](../../../desktop/ui/src/lib.rs#L37) |
| struct EditorUi | [源码:45](../../../desktop/ui/src/lib.rs#L45) |
| impl impl Default for EditorUi | [源码:88](../../../desktop/ui/src/lib.rs#L88) |
| fn default | [源码:89](../../../desktop/ui/src/lib.rs#L89) |
| impl impl EditorUi | [源码:135](../../../desktop/ui/src/lib.rs#L135) |
| fn object_id | [源码:136](../../../desktop/ui/src/lib.rs#L136) |
| fn request_id | [源码:141](../../../desktop/ui/src/lib.rs#L141) |
| fn submit | [源码:146](../../../desktop/ui/src/lib.rs#L146) |
| fn draft | [源码:166](../../../desktop/ui/src/lib.rs#L166) |
| fn draft_state | [源码:191](../../../desktop/ui/src/lib.rs#L191) |
| fn cancel | [源码:198](../../../desktop/ui/src/lib.rs#L198) |
| fn cancel_owned | [源码:211](../../../desktop/ui/src/lib.rs#L211) |
| fn interrupt_for_fault | [源码:227](../../../desktop/ui/src/lib.rs#L227) |
| fn commit | [源码:240](../../../desktop/ui/src/lib.rs#L240) |
| fn parameter | [源码:259](../../../desktop/ui/src/lib.rs#L259) |
| fn navigation | [源码:292](../../../desktop/ui/src/lib.rs#L292) |
| fn project | [源码:311](../../../desktop/ui/src/lib.rs#L311) |
| fn reconcile | [源码:319](../../../desktop/ui/src/lib.rs#L319) |
| fn reconcile_project | [源码:323](../../../desktop/ui/src/lib.rs#L323) |
| fn add_document | [源码:345](../../../desktop/ui/src/lib.rs#L345) |
| fn add_asset_layer | [源码:362](../../../desktop/ui/src/lib.rs#L362) |
| fn transform | [源码:426](../../../desktop/ui/src/lib.rs#L426) |
| fn transform_command | [源码:432](../../../desktop/ui/src/lib.rs#L432) |
| fn show | [源码:445](../../../desktop/ui/src/lib.rs#L445) |
| fn show_observed | [源码:455](../../../desktop/ui/src/lib.rs#L455) |

## desktop/ui/src/observation.rs

43 行；SHA256 `7c3648912494b52a589d3bf95011005441785f11bc3afd894094db078b94151f`

导入：`use super::{EditorUi, ProjectSession, SkinProject};`

| 定义 | 位置／签名 |
| --- | --- |
| enum UiPhase | [源码:6](../../../desktop/ui/src/observation.rs#L6) |
| fn phase | [源码:26](../../../desktop/ui/src/observation.rs#L26) |
| impl impl EditorUi | [源码:32](../../../desktop/ui/src/observation.rs#L32) |
| fn observed_project | [源码:33](../../../desktop/ui/src/observation.rs#L33) |

## desktop/ui/src/picking.rs

99 行；SHA256 `6398124bd7ede861d4417603ef348e62c34550f03d404c00603b360df68bae9b`

导入：`use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| fn inside | [源码:4](../../../desktop/ui/src/picking.rs#L4) |
| fn document | [源码:7](../../../desktop/ui/src/picking.rs#L7) |
| fn layer_at | [源码:48](../../../desktop/ui/src/picking.rs#L48) |

## desktop/ui/src/selection.rs

325 行；SHA256 `6eb87e1b3bdd99a480a025b92339c8be49b2b6a754e3f8247e34fd46c4a566e7`

导入：`use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| const MAX_SELECTED | [源码:4](../../../desktop/ui/src/selection.rs#L4) |
| type Stamp | [源码:5](../../../desktop/ui/src/selection.rs#L5) |
| struct Geometry | [源码:13](../../../desktop/ui/src/selection.rs#L13) |
| impl impl Geometry | [源码:18](../../../desktop/ui/src/selection.rs#L18) |
| fn single | [源码:19](../../../desktop/ui/src/selection.rs#L19) |
| fn points | [源码:36](../../../desktop/ui/src/selection.rs#L36) |
| fn controls | [源码:44](../../../desktop/ui/src/selection.rs#L44) |
| fn joint | [源码:60](../../../desktop/ui/src/selection.rs#L60) |
| struct Frame | [源码:98](../../../desktop/ui/src/selection.rs#L98) |
| fn effective | [源码:105](../../../desktop/ui/src/selection.rs#L105) |
| fn parent | [源码:137](../../../desktop/ui/src/selection.rs#L137) |
| struct Member | [源码:161](../../../desktop/ui/src/selection.rs#L161) |
| struct EditGesture | [源码:168](../../../desktop/ui/src/selection.rs#L168) |
| impl impl EditGesture | [源码:174](../../../desktop/ui/src/selection.rs#L174) |
| fn new | [源码:175](../../../desktop/ui/src/selection.rs#L175) |
| fn values | [源码:209](../../../desktop/ui/src/selection.rs#L209) |
| impl impl EditorUi | [源码:237](../../../desktop/ui/src/selection.rs#L237) |
| fn selection | [源码:238](../../../desktop/ui/src/selection.rs#L238) |
| fn selections | [源码:241](../../../desktop/ui/src/selection.rs#L241) |
| fn toggle_selection | [源码:245](../../../desktop/ui/src/selection.rs#L245) |
| fn select | [源码:248](../../../desktop/ui/src/selection.rs#L248) |
| fn pick | [源码:257](../../../desktop/ui/src/selection.rs#L257) |
| fn selection_stamp | [源码:291](../../../desktop/ui/src/selection.rs#L291) |
| fn selection_geometry | [源码:300](../../../desktop/ui/src/selection.rs#L300) |
| fn remember_frame | [源码:319](../../../desktop/ui/src/selection.rs#L319) |

## desktop/ui/src/shortcuts.rs

330 行；SHA256 `1205803c298aaadb6b055532e91b9ffc27032d5169123fccd77c8f76c72fe00b`

导入：`use super::*;`, `use Chord::*;`, `use EditorAction::*;`, `use ShortcutScope::*;`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| enum EditorAction | [源码:6](../../../desktop/ui/src/shortcuts.rs#L6) |
| enum ShortcutScope | [源码:15](../../../desktop/ui/src/shortcuts.rs#L15) |
| enum Chord | [源码:20](../../../desktop/ui/src/shortcuts.rs#L20) |
| struct ShortcutBinding | [源码:27](../../../desktop/ui/src/shortcuts.rs#L27) |
| static str | [源码:30](../../../desktop/ui/src/shortcuts.rs#L30) |
| static str | [源码:31](../../../desktop/ui/src/shortcuts.rs#L31) |
| const DEFAULT_SHORTCUTS | [源码:41](../../../desktop/ui/src/shortcuts.rs#L41) |
| const MODIFIER_HINTS | [源码:99](../../../desktop/ui/src/shortcuts.rs#L99) |
| const TRANSFORM_HINT | [源码:103](../../../desktop/ui/src/shortcuts.rs#L103) |
| const SELECTION_HINT | [源码:104](../../../desktop/ui/src/shortcuts.rs#L104) |
| fn additive | [源码:107](../../../desktop/ui/src/shortcuts.rs#L107) |
| fn proportional | [源码:110](../../../desktop/ui/src/shortcuts.rs#L110) |
| impl impl Chord | [源码:113](../../../desktop/ui/src/shortcuts.rs#L113) |
| fn matches | [源码:114](../../../desktop/ui/src/shortcuts.rs#L114) |
| struct Router | [源码:131](../../../desktop/ui/src/shortcuts.rs#L131) |
| struct FrameInput | [源码:136](../../../desktop/ui/src/shortcuts.rs#L136) |
| impl impl Router | [源码:141](../../../desktop/ui/src/shortcuts.rs#L141) |
| fn begin | [源码:142](../../../desktop/ui/src/shortcuts.rs#L142) |
| fn finish | [源码:189](../../../desktop/ui/src/shortcuts.rs#L189) |
| impl impl FrameInput | [源码:193](../../../desktop/ui/src/shortcuts.rs#L193) |
| fn early_cancel | [源码:196](../../../desktop/ui/src/shortcuts.rs#L196) |
| fn consume | [源码:212](../../../desktop/ui/src/shortcuts.rs#L212) |
| fn actions | [源码:217](../../../desktop/ui/src/shortcuts.rs#L217) |
| impl impl EditorUi | [源码:246](../../../desktop/ui/src/shortcuts.rs#L246) |
| fn sync_interaction_mode | [源码:247](../../../desktop/ui/src/shortcuts.rs#L247) |
| fn perform_action | [源码:254](../../../desktop/ui/src/shortcuts.rs#L254) |
| mod tests | [源码:305](../../../desktop/ui/src/shortcuts.rs#L305) |
| fn default_bindings_never_conflict_in_the_same_input_context | [源码:308](../../../desktop/ui/src/shortcuts.rs#L308) |

## desktop/ui/src/tests.rs

826 行；SHA256 `0cef0c9ae6bd5e62d618956b8407342bc00041112c5a81d3439978d4551be7e0`

导入：`use super::*;`, `use std::sync::Arc;`, `use std::sync::Mutex;`

| 定义 | 位置／签名 |
| --- | --- |
| mod gesture_controls | [源码:2](../../../desktop/ui/src/tests.rs#L2) |
| mod history_ui | [源码:3](../../../desktop/ui/src/tests.rs#L3) |
| mod input | [源码:4](../../../desktop/ui/src/tests.rs#L4) |
| mod interaction | [源码:5](../../../desktop/ui/src/tests.rs#L5) |
| mod joint_selection | [源码:6](../../../desktop/ui/src/tests.rs#L6) |
| struct Lease | [源码:9](../../../desktop/ui/src/tests.rs#L9) |
| impl impl BlobLease for Lease | [源码:10](../../../desktop/ui/src/tests.rs#L10) |
| fn reference | [源码:11](../../../desktop/ui/src/tests.rs#L11) |
| struct Store | [源码:15](../../../desktop/ui/src/tests.rs#L15) |
| struct AudioTrace | [源码:18](../../../desktop/ui/src/tests.rs#L18) |
| struct AudioFake | [源码:26](../../../desktop/ui/src/tests.rs#L26) |
| impl impl SkinAudioPort for AudioFake | [源码:27](../../../desktop/ui/src/tests.rs#L27) |
| fn retained_drafts | [源码:28](../../../desktop/ui/src/tests.rs#L28) |
| fn synchronize | [源码:38](../../../desktop/ui/src/tests.rs#L38) |
| impl impl AudioEditorPort for AudioFake | [源码:54](../../../desktop/ui/src/tests.rs#L54) |
| fn request | [源码:55](../../../desktop/ui/src/tests.rs#L55) |
| fn poll | [源码:63](../../../desktop/ui/src/tests.rs#L63) |
| fn failed_device_freezes_preview_and_reenable_does_not_inherit_old_play_intent | [源码:79](../../../desktop/ui/src/tests.rs#L79) |
| fn audio_fixture | [源码:99](../../../desktop/ui/src/tests.rs#L99) |
| fn audio_edit_cancel_and_history_synchronize_before_return_without_a_gui_pass | [源码:129](../../../desktop/ui/src/tests.rs#L129) |
| fn busy_audio_sync_retries_without_rolling_back_a_valid_project_or_replaying_audition | [源码:182](../../../desktop/ui/src/tests.rs#L182) |
| fn three_hundred_audio_draft_updates_commit_once_and_duplicate_slot_failure_is_atomic | [源码:216](../../../desktop/ui/src/tests.rs#L216) |
| impl impl ResourceStore for Store | [源码:245](../../../desktop/ui/src/tests.rs#L245) |
| fn pin | [源码:246](../../../desktop/ui/src/tests.rs#L246) |
| struct Fake | [源码:251](../../../desktop/ui/src/tests.rs#L251) |
| impl impl Presenter for Fake | [源码:252](../../../desktop/ui/src/tests.rs#L252) |
| fn current | [源码:253](../../../desktop/ui/src/tests.rs#L253) |
| fn paint | [源码:256](../../../desktop/ui/src/tests.rs#L256) |
| fn thumbnail | [源码:257](../../../desktop/ui/src/tests.rs#L257) |
| struct Harness | [源码:259](../../../desktop/ui/src/tests.rs#L259) |
| impl impl Harness | [源码:267](../../../desktop/ui/src/tests.rs#L267) |
| fn new | [源码:268](../../../desktop/ui/src/tests.rs#L268) |
| fn pass | [源码:300](../../../desktop/ui/src/tests.rs#L300) |
| fn pass_recorded | [源码:303](../../../desktop/ui/src/tests.rs#L303) |
| fn labels | [源码:334](../../../desktop/ui/src/tests.rs#L334) |
| fn pointer | [源码:378](../../../desktop/ui/src/tests.rs#L378) |
| fn drag | [源码:390](../../../desktop/ui/src/tests.rs#L390) |
| fn parameter_pass | [源码:396](../../../desktop/ui/src/tests.rs#L396) |
| fn key | [源码:426](../../../desktop/ui/src/tests.rs#L426) |
| fn diagnostic_boundaries_preserve_project_layout_and_output | [源码:436](../../../desktop/ui/src/tests.rs#L436) |
| fn audio_detail_observation_preserves_widgets_selection_and_project | [源码:473](../../../desktop/ui/src/tests.rs#L473) |
| fn observed_escape_cancels_the_same_draft_without_a_history_node | [源码:523](../../../desktop/ui/src/tests.rs#L523) |
| fn fault_interrupt_cancels_canvas_drag_and_late_release_cannot_commit_it | [源码:542](../../../desktop/ui/src/tests.rs#L542) |
| fn fault_interrupt_keeps_numeric_drag_blocked_until_the_old_button_is_released | [源码:561](../../../desktop/ui/src/tests.rs#L561) |
| fn fault_interrupt_does_not_cancel_an_external_draft_even_with_a_stale_ui_owner | [源码:581](../../../desktop/ui/src/tests.rs#L581) |
| fn audio_backend_failure_during_cancel_does_not_keep_a_ui_draft_or_publish_a_node | [源码:605](../../../desktop/ui/src/tests.rs#L605) |
| fn continuous_canvas_drag_is_one_node_and_pause_does_not_block_edits | [源码:637](../../../desktop/ui/src/tests.rs#L637) |
| fn escape_and_focus_loss_cancel_without_committed_changes | [源码:658](../../../desktop/ui/src/tests.rs#L658) |
| fn stale_frame_blocks_new_drag_and_resize_cancels_active_drag | [源码:678](../../../desktop/ui/src/tests.rs#L678) |
| fn canvas_focus_gates_keys_and_settings_do_not_enter_history | [源码:710](../../../desktop/ui/src/tests.rs#L710) |
| fn creating_first_document_and_layer_is_atomic_on_asset_failure | [源码:729](../../../desktop/ui/src/tests.rs#L729) |
| fn continuous_numeric_parameter_drag_uses_a_single_history_transaction | [源码:739](../../../desktop/ui/src/tests.rs#L739) |
| fn cancelled_numeric_drag_does_not_restart_as_immediate_commands_while_button_is_held | [源码:768](../../../desktop/ui/src/tests.rs#L768) |
| fn focus_loss_and_resize_do_not_cancel_a_draft_created_outside_the_ui | [源码:787](../../../desktop/ui/src/tests.rs#L787) |
| fn stale_ui_ownership_cannot_cancel_a_draft_in_another_session_with_the_same_id | [源码:807](../../../desktop/ui/src/tests.rs#L807) |

## desktop/ui/src/tests/gesture_controls.rs

562 行；SHA256 `d844dcabd85daba1a7f53e7eb995036dbdb58c33fd38c519642264b387677b47`

导入：`use super::Harness;`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| fn grouped | [源码:4](../../../desktop/ui/src/tests/gesture_controls.rs#L4) |
| fn bounds | [源码:72](../../../desktop/ui/src/tests/gesture_controls.rs#L72) |
| fn corner_drag_is_one_transaction_for_layer_nested_group_and_component_group | [源码:90](../../../desktop/ui/src/tests/gesture_controls.rs#L90) |
| fn rotation_cancel_and_late_release_preserve_the_original_node | [源码:113](../../../desktop/ui/src/tests/gesture_controls.rs#L113) |
| enum Control | [源码:144](../../../desktop/ui/src/tests/gesture_controls.rs#L144) |
| fn press_point | [源码:149](../../../desktop/ui/src/tests/gesture_controls.rs#L149) |
| fn held_canvas_gestures_keep_the_candidate_and_accumulate_motion_across_frames | [源码:159](../../../desktop/ui/src/tests/gesture_controls.rs#L159) |
| fn held_canvas_return_to_press_origin_is_a_noop | [源码:248](../../../desktop/ui/src/tests/gesture_controls.rs#L248) |
| fn canvas_release_includes_the_last_pointer_motion | [源码:273](../../../desktop/ui/src/tests/gesture_controls.rs#L273) |
| fn pending_preview_coalesces_pointer_motion_then_flushes_when_ready | [源码:299](../../../desktop/ui/src/tests/gesture_controls.rs#L299) |
| fn controls | [源码:347](../../../desktop/ui/src/tests/gesture_controls.rs#L347) |
| fn rotated_layer_nested_group_and_component_controls_scale_in_their_own_axes | [源码:373](../../../desktop/ui/src/tests/gesture_controls.rs#L373) |
| fn shift_can_toggle_at_a_stationary_pointer_and_release_flushes_pending_final_pose | [源码:432](../../../desktop/ui/src/tests/gesture_controls.rs#L432) |
| fn control_geometry_unions_children_before_selected_transform_and_keeps_output_mapping | [源码:468](../../../desktop/ui/src/tests/gesture_controls.rs#L468) |
| fn held_canvas_gestures_cancel_without_a_node_or_late_release_commit | [源码:535](../../../desktop/ui/src/tests/gesture_controls.rs#L535) |

## desktop/ui/src/tests/history_ui.rs

166 行；SHA256 `58b50c12b459aba6fa6164a5cb28c4116e5779f332c4cb31bc7e687fd5b5a836`

导入：`use super::*;`, `use joint_selection::click;`

| 定义 | 位置／签名 |
| --- | --- |
| fn click_size | [源码:4](../../../desktop/ui/src/tests/history_ui.rs#L4) |
| fn branch | [源码:29](../../../desktop/ui/src/tests/history_ui.rs#L29) |
| fn manual_two_branch_sequence_shows_both_children_and_navigates_them | [源码:45](../../../desktop/ui/src/tests/history_ui.rs#L45) |
| fn child_navigation_remains_visible_in_a_short_viewport_with_deep_ancestry | [源码:89](../../../desktop/ui/src/tests/history_ui.rs#L89) |
| fn leaf_has_an_explicit_empty_state_instead_of_an_unexplained_blank | [源码:139](../../../desktop/ui/src/tests/history_ui.rs#L139) |
| fn a_held_child_pointer_cannot_click_an_ancestor_after_the_head_changes | [源码:149](../../../desktop/ui/src/tests/history_ui.rs#L149) |

## desktop/ui/src/tests/input.rs

625 行；SHA256 `4021387ccf2245178d4698f3828251c8c673f4187772a69e50fbeca5ce6a634d`

导入：`use super::*;`, `use joint_selection::{click, pair};`

| 定义 | 位置／签名 |
| --- | --- |
| fn press | [源码:4](../../../desktop/ui/src/tests/input.rs#L4) |
| fn release | [源码:18](../../../desktop/ui/src/tests/input.rs#L18) |
| fn primary | [源码:31](../../../desktop/ui/src/tests/input.rs#L31) |
| fn point | [源码:38](../../../desktop/ui/src/tests/input.rs#L38) |
| fn select_at | [源码:48](../../../desktop/ui/src/tests/input.rs#L48) |
| fn move_layer | [源码:53](../../../desktop/ui/src/tests/input.rs#L53) |
| fn delete_is_atomic_keeps_assets_and_uses_the_button_action_with_undo_redo | [源码:65](../../../desktop/ui/src/tests/input.rs#L65) |
| fn delete_limit_failure_and_busy_leave_selection_project_and_history_intact | [源码:102](../../../desktop/ui/src/tests/input.rs#L102) |
| fn ancestor_delete_is_once_and_undo_restores_its_subtree | [源码:129](../../../desktop/ui/src/tests/input.rs#L129) |
| fn repeated_delete_empty_selection_and_modified_delete_cannot_create_edits | [源码:165](../../../desktop/ui/src/tests/input.rs#L165) |
| fn ime_composition_commit_and_focus_loss_are_quarantined | [源码:195](../../../desktop/ui/src/tests/input.rs#L195) |
| fn text_widget_owns_delete_undo_and_escape_but_a_focused_button_does_not | [源码:226](../../../desktop/ui/src/tests/input.rs#L226) |
| fn switching_to_interaction_clears_joint_selection_and_keeps_committed_edits | [源码:289](../../../desktop/ui/src/tests/input.rs#L289) |
| fn mode_change_cancels_owned_gesture_and_late_release_but_not_foreign_draft | [源码:311](../../../desktop/ui/src/tests/input.rs#L311) |
| fn topmost_canvas_pick_obeys_paint_order_and_does_not_edit_history | [源码:340](../../../desktop/ui/src/tests/input.rs#L340) |
| fn hidden_layers_groups_zero_opacity_and_document_clip_do_not_pick | [源码:355](../../../desktop/ui/src/tests/input.rs#L355) |
| fn canvas_ctrl_multi_pick_keeps_workspace_and_its_common_frame | [源码:405](../../../desktop/ui/src/tests/input.rs#L405) |
| fn stale_preview_cannot_select_and_game_keys_cannot_be_history_shortcuts | [源码:423](../../../desktop/ui/src/tests/input.rs#L423) |
| fn escape_repeat_and_simultaneous_release_cancel_once_without_clearing_selection | [源码:449](../../../desktop/ui/src/tests/input.rs#L449) |
| fn multiple_egui_passes_execute_delete_once_and_command_redo_aliases_work | [源码:485](../../../desktop/ui/src/tests/input.rs#L485) |
| fn picking_applies_member_mapping_rotation_and_visibility_without_changing_target | [源码:522](../../../desktop/ui/src/tests/input.rs#L522) |

## desktop/ui/src/tests/interaction.rs

183 行；SHA256 `adbfb1ed02a6c43bb10522110443e5f6268c81b40b923db70a5e96c03569e565`

导入：`use super::*;`, `use joint_selection::{click, pair};`

| 定义 | 位置／签名 |
| --- | --- |
| fn interaction_tool_blocks_layer_panel_selection_and_edit_tool_reenables_it | [源码:5](../../../desktop/ui/src/tests/interaction.rs#L5) |
| fn interaction_tool_blocks_nested_group_and_component_rows | [源码:52](../../../desktop/ui/src/tests/interaction.rs#L52) |
| fn interaction_selection_apis_are_noops_and_do_not_cancel_an_owned_draft | [源码:88](../../../desktop/ui/src/tests/interaction.rs#L88) |
| fn interaction_never_exposes_transform_geometry_even_before_the_next_ui_pass | [源码:118](../../../desktop/ui/src/tests/interaction.rs#L118) |
| fn preview_scene_and_operation_tool_have_independent_selection_semantics | [源码:129](../../../desktop/ui/src/tests/interaction.rs#L129) |
| fn adding_an_image_while_interacting_does_not_reopen_visual_selection | [源码:152](../../../desktop/ui/src/tests/interaction.rs#L152) |

## desktop/ui/src/tests/joint_selection.rs

646 行；SHA256 `4eae139148411aa9b136b514c525d2511de85e260567708185d4b7dc2ac71def`

导入：`use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| fn pair | [源码:3](../../../desktop/ui/src/tests/joint_selection.rs#L3) |
| fn joint | [源码:89](../../../desktop/ui/src/tests/joint_selection.rs#L89) |
| fn controls | [源码:95](../../../desktop/ui/src/tests/joint_selection.rs#L95) |
| fn click | [源码:102](../../../desktop/ui/src/tests/joint_selection.rs#L102) |
| fn near | [源码:113](../../../desktop/ui/src/tests/joint_selection.rs#L113) |
| fn ctrl_selection_and_idle_escape_are_editor_state_without_component_nodes | [源码:120](../../../desktop/ui/src/tests/joint_selection.rs#L120) |
| fn joint_frame_survives_repeated_edits_and_clear_keeps_committed_transforms | [源码:143](../../../desktop/ui/src/tests/joint_selection.rs#L143) |
| fn active_escape_keeps_joint_selection_and_late_release_cannot_commit | [源码:212](../../../desktop/ui/src/tests/joint_selection.rs#L212) |
| fn ancestor_selection_transforms_descendants_only_once | [源码:239](../../../desktop/ui/src/tests/joint_selection.rs#L239) |
| fn joint_replace_limit_failure_is_atomic_and_does_not_advance_frame | [源码:272](../../../desktop/ui/src/tests/joint_selection.rs#L272) |
| fn clearing_selection_does_not_cancel_a_foreign_application_draft | [源码:292](../../../desktop/ui/src/tests/joint_selection.rs#L292) |
| fn component_panel_distinguishes_group_and_member_without_global_creation | [源码:315](../../../desktop/ui/src/tests/joint_selection.rs#L315) |
| fn mapped_component_document_and_loose_layer_share_workspace_delta | [源码:347](../../../desktop/ui/src/tests/joint_selection.rs#L347) |
| fn singular_parent_rejects_joint_gesture_before_begin_without_partial_changes | [源码:407](../../../desktop/ui/src/tests/joint_selection.rs#L407) |
| fn fault_cancels_joint_draft_and_late_release_preserves_the_project | [源码:454](../../../desktop/ui/src/tests/joint_selection.rs#L454) |
| fn selection_is_bounded_and_blank_click_releases_only_the_temporary_frame | [源码:472](../../../desktop/ui/src/tests/joint_selection.rs#L472) |
| fn ambiguous_layer_import_does_not_create_a_document_or_component | [源码:528](../../../desktop/ui/src/tests/joint_selection.rs#L528) |
| fn old_preview_descriptor_cannot_start_a_joint_edit | [源码:541](../../../desktop/ui/src/tests/joint_selection.rs#L541) |
| fn empty_group_has_no_bounds_but_remains_in_a_joint_transform | [源码:558](../../../desktop/ui/src/tests/joint_selection.rs#L558) |
| fn replaced_application_draft_is_not_modified_by_old_joint_input_or_commit | [源码:610](../../../desktop/ui/src/tests/joint_selection.rs#L610) |

## experiments/p0-egui/src/audio_output.rs

18 行；SHA256 `abaf12abc9c2f0029886fc0be520964cf0bd3ac95ce281eb762239084b5963c8`

导入：`use crate::audio_transport::{AudioObservation, Intent, Relocate};`

| 定义 | 位置／签名 |
| --- | --- |
| enum OutputError | [源码:5](../../../experiments/p0-egui/src/audio_output.rs#L5) |
| trait Output | [源码:12](../../../experiments/p0-egui/src/audio_output.rs#L12) |
| fn snapshot | [源码:13](../../../experiments/p0-egui/src/audio_output.rs#L13) |
| fn control | [源码:14](../../../experiments/p0-egui/src/audio_output.rs#L14) |
| fn relocate | [源码:17](../../../experiments/p0-egui/src/audio_output.rs#L17) |

## experiments/p0-egui/src/audio_transport.rs

491 行；SHA256 `3dd45ce6ae3f1876a686ffaf683dc0b5c662c527ec706f53203f563d9abb2062`

导入：`use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| enum Phase | [源码:3](../../../experiments/p0-egui/src/audio_transport.rs#L3) |
| enum Fault | [源码:11](../../../experiments/p0-egui/src/audio_transport.rs#L11) |
| struct Intent | [源码:19](../../../experiments/p0-egui/src/audio_transport.rs#L19) |
| struct Relocate | [源码:25](../../../experiments/p0-egui/src/audio_transport.rs#L25) |
| struct AudioObservation | [源码:30](../../../experiments/p0-egui/src/audio_transport.rs#L30) |
| struct State | [源码:39](../../../experiments/p0-egui/src/audio_transport.rs#L39) |
| struct Transport | [源码:48](../../../experiments/p0-egui/src/audio_transport.rs#L48) |
| impl impl Default for Transport | [源码:58](../../../experiments/p0-egui/src/audio_transport.rs#L58) |
| fn default | [源码:59](../../../experiments/p0-egui/src/audio_transport.rs#L59) |
| impl impl Transport | [源码:80](../../../experiments/p0-egui/src/audio_transport.rs#L80) |
| fn state | [源码:81](../../../experiments/p0-egui/src/audio_transport.rs#L81) |
| fn desired_paused | [源码:84](../../../experiments/p0-egui/src/audio_transport.rs#L84) |
| fn intent | [源码:87](../../../experiments/p0-egui/src/audio_transport.rs#L87) |
| fn fail | [源码:105](../../../experiments/p0-egui/src/audio_transport.rs#L105) |
| fn recover | [源码:111](../../../experiments/p0-egui/src/audio_transport.rs#L111) |
| fn seek | [源码:132](../../../experiments/p0-egui/src/audio_transport.rs#L132) |
| fn observe | [源码:147](../../../experiments/p0-egui/src/audio_transport.rs#L147) |
| fn focus | [源码:196](../../../experiments/p0-egui/src/audio_transport.rs#L196) |
| fn tool | [源码:202](../../../experiments/p0-egui/src/audio_transport.rs#L202) |
| fn key | [源码:205](../../../experiments/p0-egui/src/audio_transport.rs#L205) |
| mod tests | [源码:231](../../../experiments/p0-egui/src/audio_transport.rs#L231) |
| fn event | [源码:233](../../../experiments/p0-egui/src/audio_transport.rs#L233) |
| fn only_fresh_acknowledged_device_samples_advance_simulation | [源码:244](../../../experiments/p0-egui/src/audio_transport.rs#L244) |
| fn superseded_pause_resume_ack_cannot_reopen_input_gate | [源码:274](../../../experiments/p0-egui/src/audio_transport.rs#L274) |
| fn matching_request_with_wrong_state_is_a_fault_not_a_resume | [源码:286](../../../experiments/p0-egui/src/audio_transport.rs#L286) |
| fn editor_focus_tool_and_key_repeats_are_isolated | [源码:293](../../../experiments/p0-egui/src/audio_transport.rs#L293) |
| fn clock_regression_pause_drift_and_backend_failure_close_input | [源码:312](../../../experiments/p0-egui/src/audio_transport.rs#L312) |
| fn recovery_preserves_time_rejects_old_stream_and_requires_explicit_resume | [源码:337](../../../experiments/p0-egui/src/audio_transport.rs#L337) |
| fn failed_recovery_can_retry_without_advancing_or_accepting_stale_faults | [源码:382](../../../experiments/p0-egui/src/audio_transport.rs#L382) |
| fn recovery_rejects_playing_first_ack_and_checked_clock_overflow | [源码:412](../../../experiments/p0-egui/src/audio_transport.rs#L412) |
| fn explicit_paused_seek_can_rewind_only_after_new_epoch_ack_without_replaying_held_keys | [源码:436](../../../experiments/p0-egui/src/audio_transport.rs#L436) |
| fn failed_seek_keeps_preseek_time_and_recovery_anchors_that_confirmed_position | [源码:473](../../../experiments/p0-egui/src/audio_transport.rs#L473) |

## experiments/p0-egui/src/bin/combined-soak.rs

516 行；SHA256 `3f22a5165908ba3cd1882cfba17032a6f7e441cc0d8114234a10177b337a5dbf`

导入：`use eframe::{egui, wgpu};`, `use p0_egui::{ audio_output::Output, audio_transport::{Phase, Transport}, composition_switch::DocumentSwitch, interaction::{Command, Outcome, Owner}, streaming_decode::{DecoderWorker, DiagnosticGate}, windows_audio::WindowsOutput, };`, `use p0_wasapi::{Snapshot, buffered};`, `use serde_json::{Value, json};`, `use std::{ fs::{self, File}, io::{BufWriter, Write}, path::PathBuf, sync::Arc, time::{Duration, Instant}, };`

| 定义 | 位置／签名 |
| --- | --- |
| mod native | [源码:3](../../../experiments/p0-egui/src/bin/combined-soak.rs#L3) |
| fn stats | [源码:23](../../../experiments/p0-egui/src/bin/combined-soak.rs#L23) |
| struct App | [源码:32](../../../experiments/p0-egui/src/bin/combined-soak.rs#L32) |
| static str | [源码:37](../../../experiments/p0-egui/src/bin/combined-soak.rs#L37) |
| impl impl App | [源码:72](../../../experiments/p0-egui/src/bin/combined-soak.rs#L72) |
| fn new | [源码:73](../../../experiments/p0-egui/src/bin/combined-soak.rs#L73) |
| fn mark | [源码:145](../../../experiments/p0-egui/src/bin/combined-soak.rs#L145) |
| static str | [源码:145](../../../experiments/p0-egui/src/bin/combined-soak.rs#L145) |
| fn control | [源码:153](../../../experiments/p0-egui/src/bin/combined-soak.rs#L153) |
| fn drain | [源码:161](../../../experiments/p0-egui/src/bin/combined-soak.rs#L161) |
| fn commit | [源码:184](../../../experiments/p0-egui/src/bin/combined-soak.rs#L184) |
| fn edit | [源码:192](../../../experiments/p0-egui/src/bin/combined-soak.rs#L192) |
| fn poll | [源码:214](../../../experiments/p0-egui/src/bin/combined-soak.rs#L214) |
| fn finish | [源码:356](../../../experiments/p0-egui/src/bin/combined-soak.rs#L356) |
| impl impl eframe::App for App | [源码:391](../../../experiments/p0-egui/src/bin/combined-soak.rs#L391) |
| fn raw_input_hook | [源码:392](../../../experiments/p0-egui/src/bin/combined-soak.rs#L392) |
| fn logic | [源码:395](../../../experiments/p0-egui/src/bin/combined-soak.rs#L395) |
| fn ui | [源码:421](../../../experiments/p0-egui/src/bin/combined-soak.rs#L421) |
| fn run | [源码:453](../../../experiments/p0-egui/src/bin/combined-soak.rs#L453) |
| fn main | [源码:509](../../../experiments/p0-egui/src/bin/combined-soak.rs#L509) |
| fn main | [源码:513](../../../experiments/p0-egui/src/bin/combined-soak.rs#L513) |

## experiments/p0-egui/src/bin/composition-proof.rs

419 行；SHA256 `93bb2491887c5b830f6c5b2dbe8509dbc5a1347acc2fef2c122fb76106e32983`

导入：`use eframe::wgpu;`, `use p0_egui::composition::{ self, Affine, Image, Limits, Node, Sampling, compile, compile_with_limits, render, };`, `use p0_egui::composition_fixture::{remap, standard_nodes};`, `use p0_egui::composition_gpu::GpuPlan;`, `use serde_json::json;`, `use std::{path::PathBuf, time::Instant};`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| fn layer | [源码:11](../../../experiments/p0-egui/src/bin/composition-proof.rs#L11) |
| fn group | [源码:19](../../../experiments/p0-egui/src/bin/composition-proof.rs#L19) |
| fn fixtures | [源码:26](../../../experiments/p0-egui/src/bin/composition-proof.rs#L26) |
| static str | [源码:26](../../../experiments/p0-egui/src/bin/composition-proof.rs#L26) |
| fn save | [源码:126](../../../experiments/p0-egui/src/bin/composition-proof.rs#L126) |
| fn scale | [源码:142](../../../experiments/p0-egui/src/bin/composition-proof.rs#L142) |
| fn main | [源码:256](../../../experiments/p0-egui/src/bin/composition-proof.rs#L256) |
| mod fixture_tests | [源码:386](../../../experiments/p0-egui/src/bin/composition-proof.rs#L386) |
| fn fixture_keeps_isolated_groups_remaps_shared_sources_and_rejects_bad_references | [源码:389](../../../experiments/p0-egui/src/bin/composition-proof.rs#L389) |

## experiments/p0-egui/src/bin/compressed-output-proof.rs

271 行；SHA256 `be8dc698ca87e998b63f8b44cc1331490c4ab25cd32cb75d76ad6dc16d950f9e`

导入：`use p0_egui::{ audio_output::Output, audio_transport::{Phase, Transport}, streaming_decode::{DecoderWorker, DiagnosticGate}, windows_audio::WindowsOutput, };`, `use p0_wasapi::{ Snapshot, buffered::{self, Config, Input}, };`, `use serde_json::{Value, json};`, `use std::{ fs::{self, File}, path::PathBuf, thread, time::{Duration, Instant}, };`

| 定义 | 位置／签名 |
| --- | --- |
| mod native | [源码:2](../../../experiments/p0-egui/src/bin/compressed-output-proof.rs#L2) |
| fn stats | [源码:20](../../../experiments/p0-egui/src/bin/compressed-output-proof.rs#L20) |
| struct Run | [源码:28](../../../experiments/p0-egui/src/bin/compressed-output-proof.rs#L28) |
| impl impl Run | [源码:40](../../../experiments/p0-egui/src/bin/compressed-output-proof.rs#L40) |
| fn poll | [源码:41](../../../experiments/p0-egui/src/bin/compressed-output-proof.rs#L41) |
| fn wait | [源码:64](../../../experiments/p0-egui/src/bin/compressed-output-proof.rs#L64) |
| impl impl FnMut(&Self) -> bool, ) -> Result<(), String> | [源码:67](../../../experiments/p0-egui/src/bin/compressed-output-proof.rs#L67) |
| fn control | [源码:81](../../../experiments/p0-egui/src/bin/compressed-output-proof.rs#L81) |
| fn freeze | [源码:96](../../../experiments/p0-egui/src/bin/compressed-output-proof.rs#L96) |
| fn seek | [源码:117](../../../experiments/p0-egui/src/bin/compressed-output-proof.rs#L117) |
| fn execute | [源码:136](../../../experiments/p0-egui/src/bin/compressed-output-proof.rs#L136) |
| fn run | [源码:209](../../../experiments/p0-egui/src/bin/compressed-output-proof.rs#L209) |
| fn main | [源码:260](../../../experiments/p0-egui/src/bin/compressed-output-proof.rs#L260) |

## experiments/p0-egui/src/bin/desktop-proof.rs

296 行；SHA256 `965d3b6e22705f949594f3df4a9f85f3cc94507d5861e250d3d68dfd58e0e24d`

导入：`use eframe::{egui, wgpu};`, `use p0_egui::desktop_view::{DesktopView, Layout};`, `use serde_json::json;`, `use std::{ fs::{self, OpenOptions}, io::{Cursor, Write}, path::{Path, PathBuf}, sync::{Arc, mpsc}, time::{Duration, Instant}, };`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| const FIXTURE | [源码:13](../../../experiments/p0-egui/src/bin/desktop-proof.rs#L13) |
| fn fixture_image | [源码:14](../../../experiments/p0-egui/src/bin/desktop-proof.rs#L14) |
| fn write_new | [源码:24](../../../experiments/p0-egui/src/bin/desktop-proof.rs#L24) |
| fn accept_drop | [源码:34](../../../experiments/p0-egui/src/bin/desktop-proof.rs#L34) |
| struct App | [源码:66](../../../experiments/p0-egui/src/bin/desktop-proof.rs#L66) |
| impl impl App | [源码:80](../../../experiments/p0-egui/src/bin/desktop-proof.rs#L80) |
| fn new | [源码:81](../../../experiments/p0-egui/src/bin/desktop-proof.rs#L81) |
| fn save | [源码:100](../../../experiments/p0-egui/src/bin/desktop-proof.rs#L100) |
| impl impl eframe::App for App | [源码:137](../../../experiments/p0-egui/src/bin/desktop-proof.rs#L137) |
| fn raw_input_hook | [源码:138](../../../experiments/p0-egui/src/bin/desktop-proof.rs#L138) |
| fn logic | [源码:157](../../../experiments/p0-egui/src/bin/desktop-proof.rs#L157) |
| fn ui | [源码:177](../../../experiments/p0-egui/src/bin/desktop-proof.rs#L177) |
| fn on_exit | [源码:214](../../../experiments/p0-egui/src/bin/desktop-proof.rs#L214) |
| fn main | [源码:220](../../../experiments/p0-egui/src/bin/desktop-proof.rs#L220) |
| mod tests | [源码:267](../../../experiments/p0-egui/src/bin/desktop-proof.rs#L267) |
| fn drop_gate_only_reads_exact_owned_fixture_and_never_imports_or_writes_foreign_files | [源码:270](../../../experiments/p0-egui/src/bin/desktop-proof.rs#L270) |
| fn checkpoints_do_not_overwrite_existing_evidence | [源码:288](../../../experiments/p0-egui/src/bin/desktop-proof.rs#L288) |

## experiments/p0-egui/src/bin/format-device-proof.rs

442 行；SHA256 `78f3fd8dc60844af54e0e6635ed1c04d39aba21b16d9f233f39aab530a4849b6`

导入：`use p0_egui::{ audio_output::Output, audio_transport::{Phase, Transport}, streaming_decode::{DecoderWorker, DiagnosticGate}, windows_audio::WindowsOutput, };`, `use p0_wasapi::{ Snapshot, buffered::{self, Config, Input}, };`, `use serde_json::{Value, json};`, `use std::{ fs::{self, File}, io::{BufWriter, Write}, path::PathBuf, thread, time::{Duration, Instant}, };`

| 定义 | 位置／签名 |
| --- | --- |
| mod native | [源码:2](../../../experiments/p0-egui/src/bin/format-device-proof.rs#L2) |
| fn stats | [源码:22](../../../experiments/p0-egui/src/bin/format-device-proof.rs#L22) |
| struct Run | [源码:30](../../../experiments/p0-egui/src/bin/format-device-proof.rs#L30) |
| impl impl Run | [源码:49](../../../experiments/p0-egui/src/bin/format-device-proof.rs#L49) |
| fn stage | [源码:50](../../../experiments/p0-egui/src/bin/format-device-proof.rs#L50) |
| fn drain | [源码:57](../../../experiments/p0-egui/src/bin/format-device-proof.rs#L57) |
| fn poll | [源码:69](../../../experiments/p0-egui/src/bin/format-device-proof.rs#L69) |
| fn wait | [源码:95](../../../experiments/p0-egui/src/bin/format-device-proof.rs#L95) |
| impl impl FnMut(&Self) -> bool, ) -> Result<(), String> | [源码:98](../../../experiments/p0-egui/src/bin/format-device-proof.rs#L98) |
| fn checkpoint | [源码:112](../../../experiments/p0-egui/src/bin/format-device-proof.rs#L112) |
| fn control | [源码:115](../../../experiments/p0-egui/src/bin/format-device-proof.rs#L115) |
| fn freeze | [源码:130](../../../experiments/p0-egui/src/bin/format-device-proof.rs#L130) |
| fn play_for | [源码:152](../../../experiments/p0-egui/src/bin/format-device-proof.rs#L152) |
| fn recover | [源码:159](../../../experiments/p0-egui/src/bin/format-device-proof.rs#L159) |
| fn execute | [源码:189](../../../experiments/p0-egui/src/bin/format-device-proof.rs#L189) |
| fn run | [源码:362](../../../experiments/p0-egui/src/bin/format-device-proof.rs#L362) |
| fn main | [源码:431](../../../experiments/p0-egui/src/bin/format-device-proof.rs#L431) |

## experiments/p0-egui/src/bin/interaction-proof.rs

408 行；SHA256 `4df2b97cb921957cbe805a42fc322ace901cf9d1e4ed79a856b4bffe8a0149dc`

导入：`use eframe::egui;`, `use p0_egui::{composition_view::LiveComposition, interaction_view::GestureView};`, `use serde_json::json;`, `use std::{ path::PathBuf, sync::{Arc, mpsc}, time::Instant, };`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| struct App | [源码:11](../../../experiments/p0-egui/src/bin/interaction-proof.rs#L11) |
| impl impl App | [源码:33](../../../experiments/p0-egui/src/bin/interaction-proof.rs#L33) |
| fn new | [源码:34](../../../experiments/p0-egui/src/bin/interaction-proof.rs#L34) |
| fn capture | [源码:61](../../../experiments/p0-egui/src/bin/interaction-proof.rs#L61) |
| fn advance_test | [源码:70](../../../experiments/p0-egui/src/bin/interaction-proof.rs#L70) |
| fn button | [源码:97](../../../experiments/p0-egui/src/bin/interaction-proof.rs#L97) |
| fn escape | [源码:105](../../../experiments/p0-egui/src/bin/interaction-proof.rs#L105) |
| impl impl eframe::App for App | [源码:114](../../../experiments/p0-egui/src/bin/interaction-proof.rs#L114) |
| fn raw_input_hook | [源码:115](../../../experiments/p0-egui/src/bin/interaction-proof.rs#L115) |
| fn logic | [源码:161](../../../experiments/p0-egui/src/bin/interaction-proof.rs#L161) |
| fn ui | [源码:267](../../../experiments/p0-egui/src/bin/interaction-proof.rs#L267) |
| fn main | [源码:297](../../../experiments/p0-egui/src/bin/interaction-proof.rs#L297) |
| mod tests | [源码:379](../../../experiments/p0-egui/src/bin/interaction-proof.rs#L379) |
| fn screenshot_completion_between_input_and_ui_cannot_skip_the_next_press | [源码:382](../../../experiments/p0-egui/src/bin/interaction-proof.rs#L382) |

## experiments/p0-egui/src/bin/media-proof.rs

362 行；SHA256 `d19622721130506f7ebf2b0e2dccf116bc15705d7ba07e2d577e9d7d141fdeb0`

导入：`use p0_egui::{ audio_output::Output, audio_transport::{Phase, Transport}, windows_audio::WindowsOutput, };`, `use p0_media::{ audio, chart::{self, Frame, Timeline, Window}, };`, `use p0_wasapi::{ Snapshot, pcm::{Clip, Packet, Stats}, };`, `use serde_json::{Value, json};`, `use std::{ collections::BTreeMap, fs::{self, File, OpenOptions}, io::{Read, Write}, path::{Path, PathBuf}, sync::Arc, thread, time::{Duration, Instant}, };`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| mod native | [源码:3](../../../experiments/p0-egui/src/bin/media-proof.rs#L3) |
| fn load | [源码:28](../../../experiments/p0-egui/src/bin/media-proof.rs#L28) |
| fn packet | [源码:44](../../../experiments/p0-egui/src/bin/media-proof.rs#L44) |
| fn stats | [源码:47](../../../experiments/p0-egui/src/bin/media-proof.rs#L47) |
| fn frame | [源码:50](../../../experiments/p0-egui/src/bin/media-proof.rs#L50) |
| struct Session | [源码:53](../../../experiments/p0-egui/src/bin/media-proof.rs#L53) |
| impl impl Session | [源码:64](../../../experiments/p0-egui/src/bin/media-proof.rs#L64) |
| fn poll | [源码:65](../../../experiments/p0-egui/src/bin/media-proof.rs#L65) |
| fn wait | [源码:108](../../../experiments/p0-egui/src/bin/media-proof.rs#L108) |
| impl impl Fn(&Transport) -> bool) -> Result<(), String> | [源码:108](../../../experiments/p0-egui/src/bin/media-proof.rs#L108) |
| fn control | [源码:121](../../../experiments/p0-egui/src/bin/media-proof.rs#L121) |
| fn freeze | [源码:133](../../../experiments/p0-egui/src/bin/media-proof.rs#L133) |
| fn seek | [源码:161](../../../experiments/p0-egui/src/bin/media-proof.rs#L161) |
| fn execute | [源码:183](../../../experiments/p0-egui/src/bin/media-proof.rs#L183) |
| fn run | [源码:260](../../../experiments/p0-egui/src/bin/media-proof.rs#L260) |
| mod tests | [源码:322](../../../experiments/p0-egui/src/bin/media-proof.rs#L322) |
| fn fixture_reader_is_bounded_rejects_missing_or_escaped_resources_and_keeps_inputs | [源码:325](../../../experiments/p0-egui/src/bin/media-proof.rs#L325) |
| fn main | [源码:351](../../../experiments/p0-egui/src/bin/media-proof.rs#L351) |

## experiments/p0-egui/src/bin/pcm-proof.rs

485 行；SHA256 `b88dd7710d8dd314965b08841d09031500d8478410c97a52d0091fa74a84f1fb`

导入：`use eframe::{egui, wgpu};`, `use p0_egui::{ audio_output::Output, audio_transport::{AudioObservation, Fault, Phase, Transport}, composition_view::LiveComposition, interaction_view::GestureView, windows_audio::WindowsOutput, };`, `use p0_preview_contract::Tool;`, `use p0_wasapi::pcm::{Packet, Stats};`, `use serde_json::{Value, json};`, `use std::{ collections::BTreeMap, path::PathBuf, sync::Arc, time::{Duration, Instant}, };`

| 定义 | 位置／签名 |
| --- | --- |
| mod native | [源码:3](../../../experiments/p0-egui/src/bin/pcm-proof.rs#L3) |
| struct App | [源码:22](../../../experiments/p0-egui/src/bin/pcm-proof.rs#L22) |
| fn packet | [源码:50](../../../experiments/p0-egui/src/bin/pcm-proof.rs#L50) |
| fn pcm | [源码:57](../../../experiments/p0-egui/src/bin/pcm-proof.rs#L57) |
| fn key | [源码:62](../../../experiments/p0-egui/src/bin/pcm-proof.rs#L62) |
| fn pointer | [源码:71](../../../experiments/p0-egui/src/bin/pcm-proof.rs#L71) |
| impl impl App | [源码:79](../../../experiments/p0-egui/src/bin/pcm-proof.rs#L79) |
| fn new | [源码:80](../../../experiments/p0-egui/src/bin/pcm-proof.rs#L80) |
| fn control | [源码:120](../../../experiments/p0-egui/src/bin/pcm-proof.rs#L120) |
| fn relocate | [源码:126](../../../experiments/p0-egui/src/bin/pcm-proof.rs#L126) |
| fn paused_samples | [源码:151](../../../experiments/p0-egui/src/bin/pcm-proof.rs#L151) |
| fn proof | [源码:159](../../../experiments/p0-egui/src/bin/pcm-proof.rs#L159) |
| fn advance | [源码:172](../../../experiments/p0-egui/src/bin/pcm-proof.rs#L172) |
| fn finish | [源码:301](../../../experiments/p0-egui/src/bin/pcm-proof.rs#L301) |
| impl impl eframe::App for App | [源码:343](../../../experiments/p0-egui/src/bin/pcm-proof.rs#L343) |
| fn raw_input_hook | [源码:344](../../../experiments/p0-egui/src/bin/pcm-proof.rs#L344) |
| fn logic | [源码:377](../../../experiments/p0-egui/src/bin/pcm-proof.rs#L377) |
| fn ui | [源码:414](../../../experiments/p0-egui/src/bin/pcm-proof.rs#L414) |
| fn run | [源码:431](../../../experiments/p0-egui/src/bin/pcm-proof.rs#L431) |
| fn main | [源码:478](../../../experiments/p0-egui/src/bin/pcm-proof.rs#L478) |
| fn main | [源码:482](../../../experiments/p0-egui/src/bin/pcm-proof.rs#L482) |

## experiments/p0-egui/src/bin/recovery-proof.rs

437 行；SHA256 `0d6ed63513fe736e331eee1023135c63a29384d289f4a134eea84ac4650dd7f6`

导入：`use eframe::{egui, wgpu};`, `use p0_egui::{ audio_output::Output, audio_transport::{AudioObservation, Fault, Phase, Transport}, composition_view::LiveComposition, interaction_view::GestureView, windows_audio::WindowsOutput, };`, `use p0_preview_contract::Tool;`, `use serde_json::{Value, json};`, `use std::{ collections::BTreeMap, path::PathBuf, sync::Arc, time::{Duration, Instant}, };`

| 定义 | 位置／签名 |
| --- | --- |
| mod native | [源码:3](../../../experiments/p0-egui/src/bin/recovery-proof.rs#L3) |
| struct App | [源码:21](../../../experiments/p0-egui/src/bin/recovery-proof.rs#L21) |
| fn key | [源码:46](../../../experiments/p0-egui/src/bin/recovery-proof.rs#L46) |
| fn pointer | [源码:55](../../../experiments/p0-egui/src/bin/recovery-proof.rs#L55) |
| impl impl App | [源码:63](../../../experiments/p0-egui/src/bin/recovery-proof.rs#L63) |
| fn new | [源码:64](../../../experiments/p0-egui/src/bin/recovery-proof.rs#L64) |
| fn control | [源码:101](../../../experiments/p0-egui/src/bin/recovery-proof.rs#L101) |
| fn invalidate | [源码:107](../../../experiments/p0-egui/src/bin/recovery-proof.rs#L107) |
| fn recover | [源码:112](../../../experiments/p0-egui/src/bin/recovery-proof.rs#L112) |
| fn proof | [源码:134](../../../experiments/p0-egui/src/bin/recovery-proof.rs#L134) |
| fn paused_samples | [源码:147](../../../experiments/p0-egui/src/bin/recovery-proof.rs#L147) |
| fn advance | [源码:155](../../../experiments/p0-egui/src/bin/recovery-proof.rs#L155) |
| fn finish | [源码:243](../../../experiments/p0-egui/src/bin/recovery-proof.rs#L243) |
| impl impl eframe::App for App | [源码:286](../../../experiments/p0-egui/src/bin/recovery-proof.rs#L286) |
| fn raw_input_hook | [源码:287](../../../experiments/p0-egui/src/bin/recovery-proof.rs#L287) |
| fn logic | [源码:320](../../../experiments/p0-egui/src/bin/recovery-proof.rs#L320) |
| fn ui | [源码:362](../../../experiments/p0-egui/src/bin/recovery-proof.rs#L362) |
| fn run | [源码:383](../../../experiments/p0-egui/src/bin/recovery-proof.rs#L383) |
| fn main | [源码:430](../../../experiments/p0-egui/src/bin/recovery-proof.rs#L430) |
| fn main | [源码:434](../../../experiments/p0-egui/src/bin/recovery-proof.rs#L434) |

## experiments/p0-egui/src/bin/resource-cycle-proof.rs

606 行；SHA256 `b6f2dbde79bc5225f1f8c87de2ee4d357d38137eaac314ce256b2c49b6e0741d`

导入：`use eframe::{egui, wgpu};`, `use p0_egui::{ audio_output::Output, audio_transport::{Phase, Transport}, composition_switch::DocumentSwitch, interaction::{Command, Outcome, Owner}, streaming_decode::{DecoderWorker, DiagnosticGate}, windows_audio::WindowsOutput, };`, `use p0_wasapi::{Snapshot, buffered};`, `use serde_json::{Value, json};`, `use std::{ fs::{self, File}, io::{BufWriter, Write}, path::{Path, PathBuf}, sync::Arc, time::{Duration, Instant}, };`

| 定义 | 位置／签名 |
| --- | --- |
| mod native | [源码:3](../../../experiments/p0-egui/src/bin/resource-cycle-proof.rs#L3) |
| fn stats | [源码:22](../../../experiments/p0-egui/src/bin/resource-cycle-proof.rs#L22) |
| fn observation | [源码:30](../../../experiments/p0-egui/src/bin/resource-cycle-proof.rs#L30) |
| struct Session | [源码:33](../../../experiments/p0-egui/src/bin/resource-cycle-proof.rs#L33) |
| impl impl Session | [源码:45](../../../experiments/p0-egui/src/bin/resource-cycle-proof.rs#L45) |
| fn new | [源码:46](../../../experiments/p0-egui/src/bin/resource-cycle-proof.rs#L46) |
| fn poll | [源码:75](../../../experiments/p0-egui/src/bin/resource-cycle-proof.rs#L75) |
| fn control | [源码:99](../../../experiments/p0-egui/src/bin/resource-cycle-proof.rs#L99) |
| fn drain | [源码:107](../../../experiments/p0-egui/src/bin/resource-cycle-proof.rs#L107) |
| fn close | [源码:126](../../../experiments/p0-egui/src/bin/resource-cycle-proof.rs#L126) |
| impl impl Drop for Session | [源码:149](../../../experiments/p0-egui/src/bin/resource-cycle-proof.rs#L149) |
| fn drop | [源码:150](../../../experiments/p0-egui/src/bin/resource-cycle-proof.rs#L150) |
| struct App | [源码:155](../../../experiments/p0-egui/src/bin/resource-cycle-proof.rs#L155) |
| static str | [源码:163](../../../experiments/p0-egui/src/bin/resource-cycle-proof.rs#L163) |
| impl impl App | [源码:191](../../../experiments/p0-egui/src/bin/resource-cycle-proof.rs#L191) |
| fn new | [源码:192](../../../experiments/p0-egui/src/bin/resource-cycle-proof.rs#L192) |
| fn mark | [源码:242](../../../experiments/p0-egui/src/bin/resource-cycle-proof.rs#L242) |
| static str | [源码:242](../../../experiments/p0-egui/src/bin/resource-cycle-proof.rs#L242) |
| fn open | [源码:253](../../../experiments/p0-egui/src/bin/resource-cycle-proof.rs#L253) |
| fn edit | [源码:279](../../../experiments/p0-egui/src/bin/resource-cycle-proof.rs#L279) |
| fn commit | [源码:301](../../../experiments/p0-egui/src/bin/resource-cycle-proof.rs#L301) |
| fn poll | [源码:309](../../../experiments/p0-egui/src/bin/resource-cycle-proof.rs#L309) |
| fn finish | [源码:458](../../../experiments/p0-egui/src/bin/resource-cycle-proof.rs#L458) |
| impl impl eframe::App for App | [源码:487](../../../experiments/p0-egui/src/bin/resource-cycle-proof.rs#L487) |
| fn raw_input_hook | [源码:488](../../../experiments/p0-egui/src/bin/resource-cycle-proof.rs#L488) |
| fn logic | [源码:491](../../../experiments/p0-egui/src/bin/resource-cycle-proof.rs#L491) |
| fn ui | [源码:517](../../../experiments/p0-egui/src/bin/resource-cycle-proof.rs#L517) |
| fn run | [源码:546](../../../experiments/p0-egui/src/bin/resource-cycle-proof.rs#L546) |
| fn main | [源码:599](../../../experiments/p0-egui/src/bin/resource-cycle-proof.rs#L599) |
| fn main | [源码:603](../../../experiments/p0-egui/src/bin/resource-cycle-proof.rs#L603) |

## experiments/p0-egui/src/bin/standard-load.rs

1430 行；SHA256 `382da2cf50deb102ffaee4efe8c9a9373e894b5547484eae7016e66fd6a9589a`

导入：`use eframe::egui;`, `use p0_egui::loading::{ DecodeJob, DecodeQueue, MAX_IMAGE_BYTES, MetadataTask, Ready, UPLOAD_BYTES_PER_FRAME, UPLOADS_PER_FRAME, UploadBudget, };`, `use p0_egui::texture_cache::TextureCache;`, `use p0_egui::upload_fence::UploadFence;`, `use p0_egui::view::{show_resource_list, wake_loading};`, `use p0_egui::{ByteCache, CACHE_BYTES, Decoded, Workload, decode_png};`, `use p0_preview_contract::Probe;`, `use serde_json::{Value, json};`, `use std::{ collections::{BTreeMap, BTreeSet}, fs::File, io::Write, path::PathBuf, sync::{Arc, mpsc}, time::{Duration, Instant}, };`

| 定义 | 位置／签名 |
| --- | --- |
| static GLOBAL | [源码:24](../../../experiments/p0-egui/src/bin/standard-load.rs#L24) |
| fn heap_snapshot | [源码:26](../../../experiments/p0-egui/src/bin/standard-load.rs#L26) |
| const PERIOD | [源码:42](../../../experiments/p0-egui/src/bin/standard-load.rs#L42) |
| const EDITS | [源码:43](../../../experiments/p0-egui/src/bin/standard-load.rs#L43) |
| enum Phase | [源码:45](../../../experiments/p0-egui/src/bin/standard-load.rs#L45) |
| struct PendingLoad | [源码:60](../../../experiments/p0-egui/src/bin/standard-load.rs#L60) |
| struct Readback | [源码:67](../../../experiments/p0-egui/src/bin/standard-load.rs#L67) |
| struct Bench | [源码:72](../../../experiments/p0-egui/src/bin/standard-load.rs#L72) |
| fn stats | [源码:165](../../../experiments/p0-egui/src/bin/standard-load.rs#L165) |
| impl impl Bench | [源码:173](../../../experiments/p0-egui/src/bin/standard-load.rs#L173) |
| fn new | [源码:174](../../../experiments/p0-egui/src/bin/standard-load.rs#L174) |
| fn lifetime_event | [源码:315](../../../experiments/p0-egui/src/bin/standard-load.rs#L315) |
| fn progress_uploads | [源码:324](../../../experiments/p0-egui/src/bin/standard-load.rs#L324) |
| fn event | [源码:354](../../../experiments/p0-egui/src/bin/standard-load.rs#L354) |
| fn enter | [源码:383](../../../experiments/p0-egui/src/bin/standard-load.rs#L383) |
| fn trace_memory | [源码:394](../../../experiments/p0-egui/src/bin/standard-load.rs#L394) |
| fn load | [源码:423](../../../experiments/p0-egui/src/bin/standard-load.rs#L423) |
| fn begin_async | [源码:463](../../../experiments/p0-egui/src/bin/standard-load.rs#L463) |
| fn pump_loading | [源码:505](../../../experiments/p0-egui/src/bin/standard-load.rs#L505) |
| fn complete_cycle | [源码:643](../../../experiments/p0-egui/src/bin/standard-load.rs#L643) |
| fn capture | [源码:656](../../../experiments/p0-egui/src/bin/standard-load.rs#L656) |
| fn finish | [源码:670](../../../experiments/p0-egui/src/bin/standard-load.rs#L670) |
| fn readbacks | [源码:748](../../../experiments/p0-egui/src/bin/standard-load.rs#L748) |
| fn step | [源码:857](../../../experiments/p0-egui/src/bin/standard-load.rs#L857) |
| fn install_workload | [源码:1001](../../../experiments/p0-egui/src/bin/standard-load.rs#L1001) |
| fn empty_duration | [源码:1031](../../../experiments/p0-egui/src/bin/standard-load.rs#L1031) |
| fn closed_duration | [源码:1038](../../../experiments/p0-egui/src/bin/standard-load.rs#L1038) |
| fn apply_override | [源码:1048](../../../experiments/p0-egui/src/bin/standard-load.rs#L1048) |
| fn warm_duration | [源码:1068](../../../experiments/p0-egui/src/bin/standard-load.rs#L1068) |
| fn idle_duration | [源码:1075](../../../experiments/p0-egui/src/bin/standard-load.rs#L1075) |
| fn edit_count | [源码:1082](../../../experiments/p0-egui/src/bin/standard-load.rs#L1082) |
| fn cycle_count | [源码:1089](../../../experiments/p0-egui/src/bin/standard-load.rs#L1089) |
| impl impl eframe::App for Bench | [源码:1093](../../../experiments/p0-egui/src/bin/standard-load.rs#L1093) |
| fn logic | [源码:1094](../../../experiments/p0-egui/src/bin/standard-load.rs#L1094) |
| fn ui | [源码:1128](../../../experiments/p0-egui/src/bin/standard-load.rs#L1128) |
| fn main | [源码:1296](../../../experiments/p0-egui/src/bin/standard-load.rs#L1296) |

## experiments/p0-egui/src/bin/stream-output-proof.rs

363 行；SHA256 `8cf59acee91881c352f9d9d8c55a443e694183039c788bdbb59640eaaed5c0bd`

导入：`use p0_egui::{ audio_output::Output, audio_transport::{Phase, Transport}, streaming_decode::{DecoderWorker, DiagnosticGate}, windows_audio::WindowsOutput, };`, `use p0_wasapi::{ Snapshot, buffered::{self, Config, Input, Kind}, };`, `use serde_json::{Value, json};`, `use std::{ fs::{self, File}, path::PathBuf, thread, time::{Duration, Instant}, };`

| 定义 | 位置／签名 |
| --- | --- |
| mod native | [源码:2](../../../experiments/p0-egui/src/bin/stream-output-proof.rs#L2) |
| fn packet | [源码:21](../../../experiments/p0-egui/src/bin/stream-output-proof.rs#L21) |
| fn stats | [源码:24](../../../experiments/p0-egui/src/bin/stream-output-proof.rs#L24) |
| struct Session | [源码:31](../../../experiments/p0-egui/src/bin/stream-output-proof.rs#L31) |
| impl impl Session | [源码:40](../../../experiments/p0-egui/src/bin/stream-output-proof.rs#L40) |
| fn poll | [源码:41](../../../experiments/p0-egui/src/bin/stream-output-proof.rs#L41) |
| fn wait | [源码:57](../../../experiments/p0-egui/src/bin/stream-output-proof.rs#L57) |
| impl impl FnMut(&Self) -> bool, ) -> Result<(), String> | [源码:61](../../../experiments/p0-egui/src/bin/stream-output-proof.rs#L61) |
| fn control | [源码:78](../../../experiments/p0-egui/src/bin/stream-output-proof.rs#L78) |
| fn freeze | [源码:90](../../../experiments/p0-egui/src/bin/stream-output-proof.rs#L90) |
| fn seek | [源码:118](../../../experiments/p0-egui/src/bin/stream-output-proof.rs#L118) |
| fn execute | [源码:134](../../../experiments/p0-egui/src/bin/stream-output-proof.rs#L134) |
| fn run | [源码:292](../../../experiments/p0-egui/src/bin/stream-output-proof.rs#L292) |
| fn main | [源码:352](../../../experiments/p0-egui/src/bin/stream-output-proof.rs#L352) |

## experiments/p0-egui/src/bin/switch-audio-proof.rs

491 行；SHA256 `4555d1839663dd7507e2c3e5e606823ad7a4bc76f5c10ab9e756c70dd4a911fd`

导入：`use eframe::{egui, wgpu};`, `use p0_egui::{ audio_transport::{AudioObservation, Fault, Phase, Transport}, composition_switch::DocumentSwitch, interaction_view::GestureView, };`, `use p0_preview_contract::Tool;`, `use p0_wasapi::{Device, Phase as DevicePhase};`, `use serde_json::{Value, json};`, `use std::{ path::PathBuf, sync::Arc, time::{Duration, Instant}, };`

| 定义 | 位置／签名 |
| --- | --- |
| mod native | [源码:3](../../../experiments/p0-egui/src/bin/switch-audio-proof.rs#L3) |
| struct App | [源码:18](../../../experiments/p0-egui/src/bin/switch-audio-proof.rs#L18) |
| fn key | [源码:50](../../../experiments/p0-egui/src/bin/switch-audio-proof.rs#L50) |
| fn pointer | [源码:59](../../../experiments/p0-egui/src/bin/switch-audio-proof.rs#L59) |
| impl impl App | [源码:67](../../../experiments/p0-egui/src/bin/switch-audio-proof.rs#L67) |
| fn new | [源码:68](../../../experiments/p0-egui/src/bin/switch-audio-proof.rs#L68) |
| fn control | [源码:117](../../../experiments/p0-egui/src/bin/switch-audio-proof.rs#L117) |
| fn request | [源码:124](../../../experiments/p0-egui/src/bin/switch-audio-proof.rs#L124) |
| fn proof | [源码:137](../../../experiments/p0-egui/src/bin/switch-audio-proof.rs#L137) |
| fn finish | [源码:154](../../../experiments/p0-egui/src/bin/switch-audio-proof.rs#L154) |
| fn advance | [源码:197](../../../experiments/p0-egui/src/bin/switch-audio-proof.rs#L197) |
| impl impl eframe::App for App | [源码:314](../../../experiments/p0-egui/src/bin/switch-audio-proof.rs#L314) |
| fn raw_input_hook | [源码:315](../../../experiments/p0-egui/src/bin/switch-audio-proof.rs#L315) |
| fn logic | [源码:371](../../../experiments/p0-egui/src/bin/switch-audio-proof.rs#L371) |
| fn ui | [源码:412](../../../experiments/p0-egui/src/bin/switch-audio-proof.rs#L412) |
| fn run | [源码:437](../../../experiments/p0-egui/src/bin/switch-audio-proof.rs#L437) |
| fn main | [源码:484](../../../experiments/p0-egui/src/bin/switch-audio-proof.rs#L484) |
| fn main | [源码:488](../../../experiments/p0-egui/src/bin/switch-audio-proof.rs#L488) |

## experiments/p0-egui/src/bin/switch-proof.rs

360 行；SHA256 `181a168a668a173ea75c6722fe518a2d49a84063208267f4d32a8bc43b8ca224`

导入：`use eframe::{egui, wgpu};`, `use p0_egui::{composition_switch::DocumentSwitch, interaction_view::GestureView};`, `use serde_json::{Value, json};`, `use std::{ path::PathBuf, sync::Arc, time::{Duration, Instant}, };`

| 定义 | 位置／签名 |
| --- | --- |
| struct App | [源码:10](../../../experiments/p0-egui/src/bin/switch-proof.rs#L10) |
| impl impl App | [源码:37](../../../experiments/p0-egui/src/bin/switch-proof.rs#L37) |
| fn new | [源码:38](../../../experiments/p0-egui/src/bin/switch-proof.rs#L38) |
| fn phase | [源码:84](../../../experiments/p0-egui/src/bin/switch-proof.rs#L84) |
| fn request | [源码:92](../../../experiments/p0-egui/src/bin/switch-proof.rs#L92) |
| fn check_pixels | [源码:108](../../../experiments/p0-egui/src/bin/switch-proof.rs#L108) |
| fn finish | [源码:121](../../../experiments/p0-egui/src/bin/switch-proof.rs#L121) |
| impl impl eframe::App for App | [源码:162](../../../experiments/p0-egui/src/bin/switch-proof.rs#L162) |
| fn logic | [源码:163](../../../experiments/p0-egui/src/bin/switch-proof.rs#L163) |
| fn ui | [源码:274](../../../experiments/p0-egui/src/bin/switch-proof.rs#L274) |
| fn main | [源码:302](../../../experiments/p0-egui/src/bin/switch-proof.rs#L302) |

## experiments/p0-egui/src/bin/transport-proof.rs

452 行；SHA256 `3ea4aafe2141951cc215caa1c4c0e9e7c3a9db5ba70bd0c7f842f46ca5fe9975`

导入：`use eframe::egui;`, `use p0_egui::{ audio_transport::{AudioObservation, Phase}, composition_view::LiveComposition, interaction_view::GestureView, };`, `use p0_preview_contract::Tool;`, `use p0_wasapi::{Device, Phase as DevicePhase};`, `use serde_json::{Value, json};`, `use std::{ path::PathBuf, sync::{Arc, mpsc}, time::{Duration, Instant}, };`

| 定义 | 位置／签名 |
| --- | --- |
| mod native | [源码:3](../../../experiments/p0-egui/src/bin/transport-proof.rs#L3) |
| struct App | [源码:18](../../../experiments/p0-egui/src/bin/transport-proof.rs#L18) |
| fn key | [源码:46](../../../experiments/p0-egui/src/bin/transport-proof.rs#L46) |
| fn pointer | [源码:55](../../../experiments/p0-egui/src/bin/transport-proof.rs#L55) |
| impl impl App | [源码:63](../../../experiments/p0-egui/src/bin/transport-proof.rs#L63) |
| fn new | [源码:64](../../../experiments/p0-egui/src/bin/transport-proof.rs#L64) |
| fn capture | [源码:106](../../../experiments/p0-egui/src/bin/transport-proof.rs#L106) |
| fn finish | [源码:114](../../../experiments/p0-egui/src/bin/transport-proof.rs#L114) |
| fn advance | [源码:173](../../../experiments/p0-egui/src/bin/transport-proof.rs#L173) |
| impl impl eframe::App for App | [源码:249](../../../experiments/p0-egui/src/bin/transport-proof.rs#L249) |
| fn raw_input_hook | [源码:250](../../../experiments/p0-egui/src/bin/transport-proof.rs#L250) |
| fn logic | [源码:307](../../../experiments/p0-egui/src/bin/transport-proof.rs#L307) |
| fn ui | [源码:377](../../../experiments/p0-egui/src/bin/transport-proof.rs#L377) |
| fn run | [源码:394](../../../experiments/p0-egui/src/bin/transport-proof.rs#L394) |
| fn main | [源码:445](../../../experiments/p0-egui/src/bin/transport-proof.rs#L445) |
| fn main | [源码:449](../../../experiments/p0-egui/src/bin/transport-proof.rs#L449) |

## experiments/p0-egui/src/composition.rs

492 行；SHA256 `68828405baf1006fd430a1b7c85dd67b447823fa338ed1d4d342d6d33c36fa57`

导入：`use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| struct Image | [源码:7](../../../experiments/p0-egui/src/composition.rs#L7) |
| struct Affine | [源码:12](../../../experiments/p0-egui/src/composition.rs#L12) |
| impl impl Affine | [源码:13](../../../experiments/p0-egui/src/composition.rs#L13) |
| const IDENTITY | [源码:14](../../../experiments/p0-egui/src/composition.rs#L14) |
| fn then | [源码:16](../../../experiments/p0-egui/src/composition.rs#L16) |
| fn inverse | [源码:28](../../../experiments/p0-egui/src/composition.rs#L28) |
| fn point | [源码:47](../../../experiments/p0-egui/src/composition.rs#L47) |
| enum Sampling | [源码:53](../../../experiments/p0-egui/src/composition.rs#L53) |
| enum Node | [源码:58](../../../experiments/p0-egui/src/composition.rs#L58) |
| enum Op | [源码:72](../../../experiments/p0-egui/src/composition.rs#L72) |
| struct Plan | [源码:83](../../../experiments/p0-egui/src/composition.rs#L83) |
| struct Limits | [源码:90](../../../experiments/p0-egui/src/composition.rs#L90) |
| impl impl Limits | [源码:94](../../../experiments/p0-egui/src/composition.rs#L94) |
| const SMALL | [源码:95](../../../experiments/p0-egui/src/composition.rs#L95) |
| const STANDARD | [源码:99](../../../experiments/p0-egui/src/composition.rs#L99) |
| impl impl Plan<'_> | [源码:104](../../../experiments/p0-egui/src/composition.rs#L104) |
| fn size | [源码:105](../../../experiments/p0-egui/src/composition.rs#L105) |
| fn ops | [源码:108](../../../experiments/p0-egui/src/composition.rs#L108) |
| fn assets | [源码:111](../../../experiments/p0-egui/src/composition.rs#L111) |
| fn compile | [源码:115](../../../experiments/p0-egui/src/composition.rs#L115) |
| fn compile_with_limits | [源码:122](../../../experiments/p0-egui/src/composition.rs#L122) |
| fn walk | [源码:146](../../../experiments/p0-egui/src/composition.rs#L146) |
| fn linear | [源码:203](../../../experiments/p0-egui/src/composition.rs#L203) |
| fn srgb | [源码:210](../../../experiments/p0-egui/src/composition.rs#L210) |
| fn pixel | [源码:217](../../../experiments/p0-egui/src/composition.rs#L217) |
| fn sample | [源码:229](../../../experiments/p0-egui/src/composition.rs#L229) |
| fn over | [源码:257](../../../experiments/p0-egui/src/composition.rs#L257) |
| fn encode | [源码:260](../../../experiments/p0-egui/src/composition.rs#L260) |
| fn render | [源码:272](../../../experiments/p0-egui/src/composition.rs#L272) |
| mod tests | [源码:316](../../../experiments/p0-egui/src/composition.rs#L316) |
| fn solid | [源码:318](../../../experiments/p0-egui/src/composition.rs#L318) |
| fn layer | [源码:324](../../../experiments/p0-egui/src/composition.rs#L324) |
| fn image | [源码:332](../../../experiments/p0-egui/src/composition.rs#L332) |
| fn group_opacity_is_applied_after_overlap | [源码:336](../../../experiments/p0-egui/src/composition.rs#L336) |
| fn nested_groups_multiply_alpha_once_and_preserve_order | [源码:346](../../../experiments/p0-egui/src/composition.rs#L346) |
| fn linear_light_source_over_has_known_result | [源码:361](../../../experiments/p0-egui/src/composition.rs#L361) |
| fn rotation_nonuniform_scale_and_clipping | [源码:367](../../../experiments/p0-egui/src/composition.rs#L367) |
| fn transparent_colored_pixels_do_not_create_bilinear_halos | [源码:398](../../../experiments/p0-egui/src/composition.rs#L398) |
| fn affine_roundtrip_preserves_shear_and_reflection | [源码:415](../../../experiments/p0-egui/src/composition.rs#L415) |
| fn invalid_plans_fail_without_modifying_sources | [源码:424](../../../experiments/p0-egui/src/composition.rs#L424) |
| fn bounded_plan_rejects_excessive_depth_nodes_and_malformed_pixels | [源码:444](../../../experiments/p0-egui/src/composition.rs#L444) |
| fn near_integer_boundary_uses_shared_half_open_tie_rule | [源码:465](../../../experiments/p0-egui/src/composition.rs#L465) |
| fn scale_limits_are_explicit_and_bound_total_source_bytes | [源码:479](../../../experiments/p0-egui/src/composition.rs#L479) |

## experiments/p0-egui/src/composition.wgsl

60 行；SHA256 `e82e7cce77a53f69de74c13be6cb17d99581bfaa30888cbb1dac9082fe6024af`

| 定义 | 位置／签名 |
| --- | --- |
| struct Op | [源码:1](../../../experiments/p0-egui/src/composition.wgsl#L1) |
| function decode_channel | [源码:6](../../../experiments/p0-egui/src/composition.wgsl#L6) |
| function encode_channel | [源码:10](../../../experiments/p0-egui/src/composition.wgsl#L10) |
| function texel | [源码:14](../../../experiments/p0-egui/src/composition.wgsl#L14) |
| function sample_image | [源码:20](../../../experiments/p0-egui/src/composition.wgsl#L20) |
| function over | [源码:31](../../../experiments/p0-egui/src/composition.wgsl#L31) |
| function pack | [源码:32](../../../experiments/p0-egui/src/composition.wgsl#L32) |
| function main | [源码:42](../../../experiments/p0-egui/src/composition.wgsl#L42) |

## experiments/p0-egui/src/composition_fixture.rs

294 行；SHA256 `4096a347b90284f45abebf15d0f264c30cb1fcbd4c6978640a72cbf9fcc977a5`

导入：`use crate::composition::{Affine, Image, Node, Sampling};`, `use std::{path::Path, sync::Arc};`, `use image::{ImageFormat, ImageReader};`, `use super::*;`, `use crate::composition::{compile, render};`

| 定义 | 位置／签名 |
| --- | --- |
| fn load_document | [源码:4](../../../experiments/p0-egui/src/composition_fixture.rs#L4) |
| fn document_spec | [源码:8](../../../experiments/p0-egui/src/composition_fixture.rs#L8) |
| struct DocumentSpec | [源码:49](../../../experiments/p0-egui/src/composition_fixture.rs#L49) |
| struct LoadedDocument | [源码:53](../../../experiments/p0-egui/src/composition_fixture.rs#L53) |
| fn decode_original | [源码:58](../../../experiments/p0-egui/src/composition_fixture.rs#L58) |
| fn read_document | [源码:80](../../../experiments/p0-egui/src/composition_fixture.rs#L80) |
| fn translated_first | [源码:112](../../../experiments/p0-egui/src/composition_fixture.rs#L112) |
| fn move_leaf | [源码:116](../../../experiments/p0-egui/src/composition_fixture.rs#L116) |
| fn standard_nodes | [源码:145](../../../experiments/p0-egui/src/composition_fixture.rs#L145) |
| fn remap | [源码:193](../../../experiments/p0-egui/src/composition_fixture.rs#L193) |
| mod tests | [源码:203](../../../experiments/p0-egui/src/composition_fixture.rs#L203) |
| fn original_decoder_preserves_straight_rgba_and_rejects_corruption_and_wrong_size | [源码:207](../../../experiments/p0-egui/src/composition_fixture.rs#L207) |
| fn fixture_loader_rejects_missing_resource_without_rewriting_manifest | [源码:231](../../../experiments/p0-egui/src/composition_fixture.rs#L231) |
| fn document_delta_respects_rotated_scaled_parent_and_leaves_other_instances_unchanged | [源码:261](../../../experiments/p0-egui/src/composition_fixture.rs#L261) |

## experiments/p0-egui/src/composition_gpu.rs

456 行；SHA256 `bce644f5794344e3f6a44ce5800964551f3f243b3b5b39cccec351bb746e16b5`

导入：`use super::composition::{Image, Op, Plan, Sampling};`, `use eframe::wgpu;`, `use std::{ sync::{Arc, mpsc}, time::{Duration, Instant}, };`, `use wgpu::util::DeviceExt;`, `use super::*;`, `use crate::composition::{Affine, Node, Sampling, compile};`

| 定义 | 位置／签名 |
| --- | --- |
| struct GpuPlan | [源码:10](../../../experiments/p0-egui/src/composition_gpu.rs#L10) |
| struct Frame | [源码:23](../../../experiments/p0-egui/src/composition_gpu.rs#L23) |
| fn encode_ops | [源码:28](../../../experiments/p0-egui/src/composition_gpu.rs#L28) |
| fn check_update | [源码:72](../../../experiments/p0-egui/src/composition_gpu.rs#L72) |
| struct PreparedPlan | [源码:90](../../../experiments/p0-egui/src/composition_gpu.rs#L90) |
| impl impl PreparedPlan | [源码:94](../../../experiments/p0-egui/src/composition_gpu.rs#L94) |
| fn upload | [源码:95](../../../experiments/p0-egui/src/composition_gpu.rs#L95) |
| fn complete | [源码:117](../../../experiments/p0-egui/src/composition_gpu.rs#L117) |
| fn finish | [源码:120](../../../experiments/p0-egui/src/composition_gpu.rs#L120) |
| impl impl GpuPlan | [源码:127](../../../experiments/p0-egui/src/composition_gpu.rs#L127) |
| fn new | [源码:128](../../../experiments/p0-egui/src/composition_gpu.rs#L128) |
| fn prepare | [源码:136](../../../experiments/p0-egui/src/composition_gpu.rs#L136) |
| fn build | [源码:147](../../../experiments/p0-egui/src/composition_gpu.rs#L147) |
| fn update | [源码:280](../../../experiments/p0-egui/src/composition_gpu.rs#L280) |
| fn present | [源码:286](../../../experiments/p0-egui/src/composition_gpu.rs#L286) |
| fn run | [源码:329](../../../experiments/p0-egui/src/composition_gpu.rs#L329) |
| mod tests | [源码:422](../../../experiments/p0-egui/src/composition_gpu.rs#L422) |
| fn updates_require_the_same_owned_sources_and_shape | [源码:426](../../../experiments/p0-egui/src/composition_gpu.rs#L426) |

## experiments/p0-egui/src/composition_loading.rs

260 行；SHA256 `ae1d390c775c574e9d7a0211dc094dadf68cdcbfc5450df1a32978bbb9f212a8`

导入：`use crate::composition_fixture::{LoadedDocument, read_document};`, `use std::{ path::PathBuf, sync::{ Arc, Condvar, Mutex, atomic::{AtomicU64, Ordering}, mpsc, }, thread::{self, JoinHandle}, };`, `use super::*;`, `use crate::composition::Image;`, `use std::time::{Duration, Instant};`

| 定义 | 位置／签名 |
| --- | --- |
| struct Request | [源码:12](../../../experiments/p0-egui/src/composition_loading.rs#L12) |
| struct Mailbox | [源码:18](../../../experiments/p0-egui/src/composition_loading.rs#L18) |
| struct Reply | [源码:23](../../../experiments/p0-egui/src/composition_loading.rs#L23) |
| enum Event | [源码:27](../../../experiments/p0-egui/src/composition_loading.rs#L27) |
| struct DocumentLoader | [源码:36](../../../experiments/p0-egui/src/composition_loading.rs#L36) |
| impl impl DocumentLoader | [源码:42](../../../experiments/p0-egui/src/composition_loading.rs#L42) |
| fn new | [源码:43](../../../experiments/p0-egui/src/composition_loading.rs#L43) |
| impl impl Fn() + Send + 'static) -> Self | [源码:43](../../../experiments/p0-egui/src/composition_loading.rs#L43) |
| fn with_reader | [源码:46](../../../experiments/p0-egui/src/composition_loading.rs#L46) |
| impl impl Fn() + Send + 'static, read: impl Fn(&std::path::Path, usize, &dyn Fn() -> bool) -> Result<LoadedDocument, String> + Send + 'static, ) -> Self | [源码:47](../../../experiments/p0-egui/src/composition_loading.rs#L47) |
| impl impl Fn(&std::path::Path, usize, &dyn Fn() -> bool) -> Result<LoadedDocument, String> + Send + 'static, ) -> Self | [源码:48](../../../experiments/p0-egui/src/composition_loading.rs#L48) |
| fn request | [源码:105](../../../experiments/p0-egui/src/composition_loading.rs#L105) |
| fn cancel | [源码:118](../../../experiments/p0-egui/src/composition_loading.rs#L118) |
| fn is_idle | [源码:130](../../../experiments/p0-egui/src/composition_loading.rs#L130) |
| fn poll | [源码:134](../../../experiments/p0-egui/src/composition_loading.rs#L134) |
| impl impl Drop for DocumentLoader | [源码:148](../../../experiments/p0-egui/src/composition_loading.rs#L148) |
| fn drop | [源码:149](../../../experiments/p0-egui/src/composition_loading.rs#L149) |
| mod tests | [源码:160](../../../experiments/p0-egui/src/composition_loading.rs#L160) |
| fn loaded | [源码:164](../../../experiments/p0-egui/src/composition_loading.rs#L164) |
| fn event | [源码:174](../../../experiments/p0-egui/src/composition_loading.rs#L174) |
| fn newest_request_wins_and_invalid_request_does_not_cancel_it | [源码:185](../../../experiments/p0-egui/src/composition_loading.rs#L185) |
| fn cancel_discards_pending_result_and_errors_are_not_success | [源码:215](../../../experiments/p0-egui/src/composition_loading.rs#L215) |
| fn result_backpressure_and_drop_release_worker_without_a_detached_thread | [源码:240](../../../experiments/p0-egui/src/composition_loading.rs#L240) |

## experiments/p0-egui/src/composition_present.wgsl

21 行；SHA256 `d5b73ba3fd96fcaeaeee5c5532736ca1dcdefcf1bde1324e8998153a277b8548`

| 定义 | 位置／签名 |
| --- | --- |
| function decode | [源码:4](../../../experiments/p0-egui/src/composition_present.wgsl#L4) |
| function encode | [源码:7](../../../experiments/p0-egui/src/composition_present.wgsl#L7) |
| function main | [源码:11](../../../experiments/p0-egui/src/composition_present.wgsl#L11) |

## experiments/p0-egui/src/composition_switch.rs

187 行；SHA256 `8d9b1b9cac487127a747134a4f5bb0d33b89e26243f94d6198c9a8445fc4bced`

导入：`use crate::{ composition_loading::{DocumentLoader, Event}, composition_view::{LiveComposition, PendingComposition, PipelineTask, Pipelines}, };`, `use eframe::egui;`, `use std::path::PathBuf;`

| 定义 | 位置／签名 |
| --- | --- |
| struct DocumentSwitch | [源码:8](../../../experiments/p0-egui/src/composition_switch.rs#L8) |
| impl impl DocumentSwitch | [源码:26](../../../experiments/p0-egui/src/composition_switch.rs#L26) |
| fn new | [源码:27](../../../experiments/p0-egui/src/composition_switch.rs#L27) |
| fn warm_pipelines | [源码:48](../../../experiments/p0-egui/src/composition_switch.rs#L48) |
| fn request | [源码:60](../../../experiments/p0-egui/src/composition_switch.rs#L60) |
| fn cancel | [源码:69](../../../experiments/p0-egui/src/composition_switch.rs#L69) |
| fn close | [源码:74](../../../experiments/p0-egui/src/composition_switch.rs#L74) |
| fn active | [源码:79](../../../experiments/p0-egui/src/composition_switch.rs#L79) |
| fn active_mut | [源码:82](../../../experiments/p0-egui/src/composition_switch.rs#L82) |
| fn pending | [源码:85](../../../experiments/p0-egui/src/composition_switch.rs#L85) |
| fn preparing | [源码:91](../../../experiments/p0-egui/src/composition_switch.rs#L91) |
| fn poll | [源码:94](../../../experiments/p0-egui/src/composition_switch.rs#L94) |

## experiments/p0-egui/src/composition_view.rs

469 行；SHA256 `94babe1f58d2bc357f706986df59102322312a52a247010719e865b437fa0cb5`

导入：`use crate::{ composition::{Image, Limits, Node, compile_with_limits, render}, composition_fixture::{load_document, translated_first}, composition_gpu::GpuPlan, };`, `use eframe::{egui, wgpu};`, `use std::{path::Path, sync::Arc, time::Instant};`, `use std::sync::atomic::Ordering;`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| fn opaque_reference | [源码:9](../../../experiments/p0-egui/src/composition_view.rs#L9) |
| fn pipelines | [源码:25](../../../experiments/p0-egui/src/composition_view.rs#L25) |
| static str | [源码:26](../../../experiments/p0-egui/src/composition_view.rs#L26) |
| struct Pipelines | [源码:49](../../../experiments/p0-egui/src/composition_view.rs#L49) |
| struct PipelineTask | [源码:51](../../../experiments/p0-egui/src/composition_view.rs#L51) |
| impl impl PipelineTask | [源码:55](../../../experiments/p0-egui/src/composition_view.rs#L55) |
| fn start | [源码:56](../../../experiments/p0-egui/src/composition_view.rs#L56) |
| impl impl FnOnce() + Send + 'static) -> Self | [源码:56](../../../experiments/p0-egui/src/composition_view.rs#L56) |
| fn build | [源码:65](../../../experiments/p0-egui/src/composition_view.rs#L65) |
| impl impl FnOnce() -> Result<Pipelines, String> + Send + 'static, wake: impl FnOnce() + Send + 'static, ) -> Self | [源码:66](../../../experiments/p0-egui/src/composition_view.rs#L66) |
| impl impl FnOnce() + Send + 'static, ) -> Self | [源码:67](../../../experiments/p0-egui/src/composition_view.rs#L67) |
| fn poll | [源码:81](../../../experiments/p0-egui/src/composition_view.rs#L81) |
| impl impl Drop for PipelineTask | [源码:91](../../../experiments/p0-egui/src/composition_view.rs#L91) |
| fn drop | [源码:92](../../../experiments/p0-egui/src/composition_view.rs#L92) |
| struct LiveComposition | [源码:100](../../../experiments/p0-egui/src/composition_view.rs#L100) |
| impl impl LiveComposition | [源码:114](../../../experiments/p0-egui/src/composition_view.rs#L114) |
| fn new | [源码:115](../../../experiments/p0-egui/src/composition_view.rs#L115) |
| fn assemble | [源码:123](../../../experiments/p0-egui/src/composition_view.rs#L123) |
| fn refresh | [源码:169](../../../experiments/p0-egui/src/composition_view.rs#L169) |
| fn verify | [源码:187](../../../experiments/p0-egui/src/composition_view.rs#L187) |
| fn read_display | [源码:246](../../../experiments/p0-egui/src/composition_view.rs#L246) |
| fn metrics | [源码:294](../../../experiments/p0-egui/src/composition_view.rs#L294) |
| impl impl Drop for LiveComposition | [源码:300](../../../experiments/p0-egui/src/composition_view.rs#L300) |
| fn drop | [源码:301](../../../experiments/p0-egui/src/composition_view.rs#L301) |
| struct PendingComposition | [源码:307](../../../experiments/p0-egui/src/composition_view.rs#L307) |
| impl impl PendingComposition | [源码:321](../../../experiments/p0-egui/src/composition_view.rs#L321) |
| fn new | [源码:322](../../../experiments/p0-egui/src/composition_view.rs#L322) |
| fn profiled | [源码:328](../../../experiments/p0-egui/src/composition_view.rs#L328) |
| fn prepared | [源码:335](../../../experiments/p0-egui/src/composition_view.rs#L335) |
| fn advance | [源码:370](../../../experiments/p0-egui/src/composition_view.rs#L370) |
| fn metrics | [源码:422](../../../experiments/p0-egui/src/composition_view.rs#L422) |
| fn finish | [源码:426](../../../experiments/p0-egui/src/composition_view.rs#L426) |
| mod tests | [源码:435](../../../experiments/p0-egui/src/composition_view.rs#L435) |
| fn pipeline_task_reports_errors_and_joins_even_if_result_is_unconsumed | [源码:438](../../../experiments/p0-egui/src/composition_view.rs#L438) |
| fn presentation_uses_linear_light_and_keeps_the_export_pixel_unchanged | [源码:462](../../../experiments/p0-egui/src/composition_view.rs#L462) |

## experiments/p0-egui/src/desktop_view.rs

551 行；SHA256 `d975d2d045a3163bd7f44d6f7b2764d1db732c9a1cb4296f01d7e21efc16743b`

导入：`use crate::{ audio_transport::{AudioObservation, Intent, Phase, Transport}, interaction_view::GestureView, };`, `use eframe::egui;`, `use p0_preview_contract::{Probe, Tool};`, `use serde_json::{Value, json};`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| const PHRASE | [源码:10](../../../experiments/p0-egui/src/desktop_view.rs#L10) |
| const TEXT_ID | [源码:11](../../../experiments/p0-egui/src/desktop_view.rs#L11) |
| enum Layout | [源码:13](../../../experiments/p0-egui/src/desktop_view.rs#L13) |
| struct Clock | [源码:17](../../../experiments/p0-egui/src/desktop_view.rs#L17) |
| impl impl Default for Clock | [源码:22](../../../experiments/p0-egui/src/desktop_view.rs#L22) |
| fn default | [源码:23](../../../experiments/p0-egui/src/desktop_view.rs#L23) |
| impl impl Clock | [源码:33](../../../experiments/p0-egui/src/desktop_view.rs#L33) |
| fn control | [源码:34](../../../experiments/p0-egui/src/desktop_view.rs#L34) |
| fn poll | [源码:40](../../../experiments/p0-egui/src/desktop_view.rs#L40) |
| struct DesktopView | [源码:54](../../../experiments/p0-egui/src/desktop_view.rs#L54) |
| impl impl Default for DesktopView | [源码:96](../../../experiments/p0-egui/src/desktop_view.rs#L96) |
| fn default | [源码:97](../../../experiments/p0-egui/src/desktop_view.rs#L97) |
| impl impl DesktopView | [源码:142](../../../experiments/p0-egui/src/desktop_view.rs#L142) |
| fn set_layout | [源码:143](../../../experiments/p0-egui/src/desktop_view.rs#L143) |
| fn drop_result | [源码:150](../../../experiments/p0-egui/src/desktop_view.rs#L150) |
| fn error | [源码:157](../../../experiments/p0-egui/src/desktop_view.rs#L157) |
| fn control | [源码:162](../../../experiments/p0-egui/src/desktop_view.rs#L162) |
| fn show | [源码:166](../../../experiments/p0-egui/src/desktop_view.rs#L166) |
| fn report | [源码:349](../../../experiments/p0-egui/src/desktop_view.rs#L349) |
| mod tests | [源码:370](../../../experiments/p0-egui/src/desktop_view.rs#L370) |
| fn raw | [源码:372](../../../experiments/p0-egui/src/desktop_view.rs#L372) |
| fn draw | [源码:383](../../../experiments/p0-egui/src/desktop_view.rs#L383) |
| fn key | [源码:386](../../../experiments/p0-egui/src/desktop_view.rs#L386) |
| fn paused_clock_never_catches_up_and_report_does_not_persist_typed_text | [源码:396](../../../experiments/p0-egui/src/desktop_view.rs#L396) |
| fn ime_and_editor_keys_do_not_reach_playing_preview_or_exit_fullscreen | [源码:418](../../../experiments/p0-egui/src/desktop_view.rs#L418) |
| fn focus_loss_and_layout_change_cancel_drafts_without_history | [源码:475](../../../experiments/p0-egui/src/desktop_view.rs#L475) |
| fn fullscreen_requires_native_ack_and_scale_report_keeps_native_and_ui_separate | [源码:510](../../../experiments/p0-egui/src/desktop_view.rs#L510) |

## experiments/p0-egui/src/interaction.rs

260 行；SHA256 `02d2f8f7a511b87ec058192e02d5fd88cef3d2abdc76d6854363a0dbdef8b3aa`

导入：`use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| struct Snapshot | [源码:4](../../../experiments/p0-egui/src/interaction.rs#L4) |
| enum Command | [源码:10](../../../experiments/p0-egui/src/interaction.rs#L10) |
| enum Outcome | [源码:18](../../../experiments/p0-egui/src/interaction.rs#L18) |
| enum Error | [源码:27](../../../experiments/p0-egui/src/interaction.rs#L27) |
| struct Draft | [源码:35](../../../experiments/p0-egui/src/interaction.rs#L35) |
| struct Owner | [源码:39](../../../experiments/p0-egui/src/interaction.rs#L39) |
| impl impl Default for Owner | [源码:45](../../../experiments/p0-egui/src/interaction.rs#L45) |
| fn default | [源码:46](../../../experiments/p0-egui/src/interaction.rs#L46) |
| impl impl Owner | [源码:59](../../../experiments/p0-egui/src/interaction.rs#L59) |
| fn committed | [源码:60](../../../experiments/p0-egui/src/interaction.rs#L60) |
| fn visible_position | [源码:63](../../../experiments/p0-egui/src/interaction.rs#L63) |
| fn snapshots | [源码:66](../../../experiments/p0-egui/src/interaction.rs#L66) |
| fn execute | [源码:69](../../../experiments/p0-egui/src/interaction.rs#L69) |
| mod tests | [源码:138](../../../experiments/p0-egui/src/interaction.rs#L138) |
| fn begin | [源码:140](../../../experiments/p0-egui/src/interaction.rs#L140) |
| fn many_draft_updates_commit_once_without_changing_published_snapshots | [源码:152](../../../experiments/p0-egui/src/interaction.rs#L152) |
| fn invalid_update_rolls_back_and_cancel_or_noop_leave_no_nodes | [源码:177](../../../experiments/p0-egui/src/interaction.rs#L177) |
| fn stale_and_busy_requests_cannot_modify_a_different_session | [源码:206](../../../experiments/p0-egui/src/interaction.rs#L206) |
| fn checkout_and_branch_preserve_original_descendants | [源码:234](../../../experiments/p0-egui/src/interaction.rs#L234) |

## experiments/p0-egui/src/interaction_view.rs

760 行；SHA256 `e3e71e203be304f1d6ebd0c43f19b90e9f6624c136c8abf3df54c332ab6fce8b`

导入：`use crate::audio_transport::{AudioObservation, Fault, Intent, Phase, State, Transport};`, `use crate::interaction::{Command, Outcome, Owner, Snapshot};`, `use eframe::egui;`, `use p0_preview_contract::{Observation, Probe, Tool};`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| struct Gesture | [源码:7](../../../experiments/p0-egui/src/interaction_view.rs#L7) |
| struct GestureView | [源码:12](../../../experiments/p0-egui/src/interaction_view.rs#L12) |
| impl impl Default for GestureView | [源码:23](../../../experiments/p0-egui/src/interaction_view.rs#L23) |
| fn default | [源码:24](../../../experiments/p0-egui/src/interaction_view.rs#L24) |
| impl impl GestureView | [源码:38](../../../experiments/p0-egui/src/interaction_view.rs#L38) |
| fn tool | [源码:39](../../../experiments/p0-egui/src/interaction_view.rs#L39) |
| fn snapshot | [源码:42](../../../experiments/p0-egui/src/interaction_view.rs#L42) |
| fn node_count | [源码:45](../../../experiments/p0-egui/src/interaction_view.rs#L45) |
| fn position | [源码:48](../../../experiments/p0-egui/src/interaction_view.rs#L48) |
| fn target | [源码:51](../../../experiments/p0-egui/src/interaction_view.rs#L51) |
| fn canvas | [源码:54](../../../experiments/p0-egui/src/interaction_view.rs#L54) |
| fn simulation | [源码:57](../../../experiments/p0-egui/src/interaction_view.rs#L57) |
| fn simulation_for | [源码:60](../../../experiments/p0-egui/src/interaction_view.rs#L60) |
| fn enable_audio | [源码:72](../../../experiments/p0-egui/src/interaction_view.rs#L72) |
| fn observe_audio | [源码:77](../../../experiments/p0-egui/src/interaction_view.rs#L77) |
| fn audio_state | [源码:80](../../../experiments/p0-egui/src/interaction_view.rs#L80) |
| fn take_audio_intent | [源码:83](../../../experiments/p0-egui/src/interaction_view.rs#L83) |
| fn fail_audio | [源码:86](../../../experiments/p0-egui/src/interaction_view.rs#L86) |
| fn cancel | [源码:91](../../../experiments/p0-egui/src/interaction_view.rs#L91) |
| fn set_tool | [源码:98](../../../experiments/p0-egui/src/interaction_view.rs#L98) |
| fn set_zoom | [源码:106](../../../experiments/p0-egui/src/interaction_view.rs#L106) |
| fn pause | [源码:112](../../../experiments/p0-egui/src/interaction_view.rs#L112) |
| fn show | [源码:119](../../../experiments/p0-egui/src/interaction_view.rs#L119) |
| fn show_composition | [源码:122](../../../experiments/p0-egui/src/interaction_view.rs#L122) |
| fn show_impl | [源码:130](../../../experiments/p0-egui/src/interaction_view.rs#L130) |
| fn show_transport | [源码:143](../../../experiments/p0-egui/src/interaction_view.rs#L143) |
| fn show_inner | [源码:156](../../../experiments/p0-egui/src/interaction_view.rs#L156) |
| mod tests | [源码:332](../../../experiments/p0-egui/src/interaction_view.rs#L332) |
| fn focused_non_text_control_does_not_steal_playing_preview_keys | [源码:335](../../../experiments/p0-egui/src/interaction_view.rs#L335) |
| fn document_views_borrow_one_transport_without_resetting_clock_keys_or_faults | [源码:375](../../../experiments/p0-egui/src/interaction_view.rs#L375) |
| fn acknowledged_audio_clock_replaces_wall_time_and_routes_real_widget_keys | [源码:450](../../../experiments/p0-egui/src/interaction_view.rs#L450) |
| fn run | [源码:509](../../../experiments/p0-egui/src/interaction_view.rs#L509) |
| fn button | [源码:530](../../../experiments/p0-egui/src/interaction_view.rs#L530) |
| fn start | [源码:538](../../../experiments/p0-egui/src/interaction_view.rs#L538) |
| fn egui_pointer_route_updates_paused_draft_then_commits_release_position_once | [源码:553](../../../experiments/p0-egui/src/interaction_view.rs#L553) |
| fn escape_focus_tool_and_zoom_cancellation_never_commit_on_later_release | [源码:582](../../../experiments/p0-egui/src/interaction_view.rs#L582) |
| fn interact_clicks_are_discarded_during_pause_and_do_not_leak_into_edit_history | [源码:618](../../../experiments/p0-egui/src/interaction_view.rs#L618) |
| fn repeated_widget_pass_does_not_replay_keyboard_release_press_pair | [源码:643](../../../experiments/p0-egui/src/interaction_view.rs#L643) |
| fn repeated_egui_pass_does_not_restart_a_gesture_or_toggle_pause_twice | [源码:692](../../../experiments/p0-egui/src/interaction_view.rs#L692) |

## experiments/p0-egui/src/lib.rs

429 行；SHA256 `4aac2a1ac7dc71d3b534bb9115778f52698f7c90527f52564b4acbf1edcb58f4`

导入：`use image::{ImageFormat, ImageReader};`, `use serde_json::Value;`, `use std::{ collections::{BTreeMap, BTreeSet}, io::Cursor, path::{Path, PathBuf}, sync::Arc, };`, `use eframe::egui as egui_image;`, `use super::*;`, `use composition::{Affine, Image, Node, Sampling, compile, render};`

| 定义 | 位置／签名 |
| --- | --- |
| mod audio_output | [源码:2](../../../experiments/p0-egui/src/lib.rs#L2) |
| mod audio_transport | [源码:3](../../../experiments/p0-egui/src/lib.rs#L3) |
| mod composition | [源码:4](../../../experiments/p0-egui/src/lib.rs#L4) |
| mod composition_fixture | [源码:5](../../../experiments/p0-egui/src/lib.rs#L5) |
| mod composition_gpu | [源码:6](../../../experiments/p0-egui/src/lib.rs#L6) |
| mod composition_loading | [源码:7](../../../experiments/p0-egui/src/lib.rs#L7) |
| mod composition_switch | [源码:8](../../../experiments/p0-egui/src/lib.rs#L8) |
| mod composition_view | [源码:9](../../../experiments/p0-egui/src/lib.rs#L9) |
| mod desktop_view | [源码:10](../../../experiments/p0-egui/src/lib.rs#L10) |
| mod interaction | [源码:11](../../../experiments/p0-egui/src/lib.rs#L11) |
| mod interaction_view | [源码:12](../../../experiments/p0-egui/src/lib.rs#L12) |
| mod loading | [源码:13](../../../experiments/p0-egui/src/lib.rs#L13) |
| mod streaming_decode | [源码:15](../../../experiments/p0-egui/src/lib.rs#L15) |
| mod texture_cache | [源码:16](../../../experiments/p0-egui/src/lib.rs#L16) |
| mod upload_fence | [源码:17](../../../experiments/p0-egui/src/lib.rs#L17) |
| mod view | [源码:18](../../../experiments/p0-egui/src/lib.rs#L18) |
| mod windows_audio | [源码:20](../../../experiments/p0-egui/src/lib.rs#L20) |
| const CACHE_BYTES | [源码:30](../../../experiments/p0-egui/src/lib.rs#L30) |
| struct Asset | [源码:33](../../../experiments/p0-egui/src/lib.rs#L33) |
| struct Layer | [源码:39](../../../experiments/p0-egui/src/lib.rs#L39) |
| struct Workload | [源码:46](../../../experiments/p0-egui/src/lib.rs#L46) |
| fn number | [源码:51](../../../experiments/p0-egui/src/lib.rs#L51) |
| fn resolve | [源码:59](../../../experiments/p0-egui/src/lib.rs#L59) |
| fn flatten | [源码:70](../../../experiments/p0-egui/src/lib.rs#L70) |
| impl impl Workload | [源码:117](../../../experiments/p0-egui/src/lib.rs#L117) |
| fn open | [源码:118](../../../experiments/p0-egui/src/lib.rs#L118) |
| fn decode_png | [源码:175](../../../experiments/p0-egui/src/lib.rs#L175) |
| fn decode_png_sized | [源码:178](../../../experiments/p0-egui/src/lib.rs#L178) |
| fn decode_png_with_limit | [源码:188](../../../experiments/p0-egui/src/lib.rs#L188) |
| struct ByteCache | [源码:208](../../../experiments/p0-egui/src/lib.rs#L208) |
| impl impl<T> ByteCache<T> | [源码:215](../../../experiments/p0-egui/src/lib.rs#L215) |
| fn new | [源码:216](../../../experiments/p0-egui/src/lib.rs#L216) |
| fn get | [源码:225](../../../experiments/p0-egui/src/lib.rs#L225) |
| fn insert | [源码:232](../../../experiments/p0-egui/src/lib.rs#L232) |
| fn clear | [源码:267](../../../experiments/p0-egui/src/lib.rs#L267) |
| fn len | [源码:271](../../../experiments/p0-egui/src/lib.rs#L271) |
| fn is_empty | [源码:274](../../../experiments/p0-egui/src/lib.rs#L274) |
| type Decoded | [源码:278](../../../experiments/p0-egui/src/lib.rs#L278) |
| mod tests | [源码:281](../../../experiments/p0-egui/src/lib.rs#L281) |
| fn cache_lru_sharing_pins_and_atomic_failure | [源码:284](../../../experiments/p0-egui/src/lib.rs#L284) |
| fn nested_transforms_and_bad_references | [源码:303](../../../experiments/p0-egui/src/lib.rs#L303) |
| fn gui_loading_cache_cannot_replace_straight_pixels_or_isolated_groups | [源码:312](../../../experiments/p0-egui/src/lib.rs#L312) |
| fn png_pixels_errors_and_non_ascii_paths | [源码:366](../../../experiments/p0-egui/src/lib.rs#L366) |
| fn manifest_load_validates_versions_references_and_resource_boundary | [源码:385](../../../experiments/p0-egui/src/lib.rs#L385) |
| fn json_value | [源码:426](../../../experiments/p0-egui/src/lib.rs#L426) |

## experiments/p0-egui/src/loading.rs

509 行；SHA256 `dbd7eebf93e51ce339de6674b39c85e40df3fc04483832208cbbd673d9980642`

导入：`use crate::Decoded;`, `use crate::Workload;`, `use std::{ collections::{BTreeSet, VecDeque}, path::PathBuf, sync::{ Arc, atomic::{AtomicU64, Ordering}, mpsc, }, thread::{self, JoinHandle}, time::Instant, };`, `use super::*;`, `use std::time::Duration;`

| 定义 | 位置／签名 |
| --- | --- |
| const MAX_IMAGE_BYTES | [源码:19](../../../experiments/p0-egui/src/loading.rs#L19) |
| const UPLOAD_BYTES_PER_FRAME | [源码:20](../../../experiments/p0-egui/src/loading.rs#L20) |
| const UPLOADS_PER_FRAME | [源码:21](../../../experiments/p0-egui/src/loading.rs#L21) |
| struct MetadataTask | [源码:26](../../../experiments/p0-egui/src/loading.rs#L26) |
| impl impl MetadataTask | [源码:30](../../../experiments/p0-egui/src/loading.rs#L30) |
| fn new | [源码:31](../../../experiments/p0-egui/src/loading.rs#L31) |
| impl impl FnOnce() + Send + 'static) -> Self | [源码:31](../../../experiments/p0-egui/src/loading.rs#L31) |
| fn poll | [源码:48](../../../experiments/p0-egui/src/loading.rs#L48) |
| impl impl Drop for MetadataTask | [源码:52](../../../experiments/p0-egui/src/loading.rs#L52) |
| fn drop | [源码:53](../../../experiments/p0-egui/src/loading.rs#L53) |
| struct DecodeJob | [源码:62](../../../experiments/p0-egui/src/loading.rs#L62) |
| struct Ready | [源码:67](../../../experiments/p0-egui/src/loading.rs#L67) |
| struct DecodeStats | [源码:72](../../../experiments/p0-egui/src/loading.rs#L72) |
| struct Request | [源码:80](../../../experiments/p0-egui/src/loading.rs#L80) |
| struct Reply | [源码:84](../../../experiments/p0-egui/src/loading.rs#L84) |
| struct DecodeQueue | [源码:90](../../../experiments/p0-egui/src/loading.rs#L90) |
| impl impl DecodeQueue | [源码:100](../../../experiments/p0-egui/src/loading.rs#L100) |
| fn new | [源码:101](../../../experiments/p0-egui/src/loading.rs#L101) |
| impl impl Fn() + Send + 'static) -> Self | [源码:101](../../../experiments/p0-egui/src/loading.rs#L101) |
| fn prefetched | [源码:104](../../../experiments/p0-egui/src/loading.rs#L104) |
| impl impl Fn() + Send + 'static) -> Self | [源码:104](../../../experiments/p0-egui/src/loading.rs#L104) |
| fn png_worker | [源码:107](../../../experiments/p0-egui/src/loading.rs#L107) |
| impl impl Fn() + Send + 'static, limit: usize) -> Self | [源码:107](../../../experiments/p0-egui/src/loading.rs#L107) |
| fn with_decoder | [源码:119](../../../experiments/p0-egui/src/loading.rs#L119) |
| impl impl Fn() + Send + 'static, decode: impl Fn(&DecodeJob) -> Result<Decoded, String> + Send + 'static, ) -> Self | [源码:120](../../../experiments/p0-egui/src/loading.rs#L120) |
| impl impl Fn(&DecodeJob) -> Result<Decoded, String> + Send + 'static, ) -> Self | [源码:121](../../../experiments/p0-egui/src/loading.rs#L121) |
| fn with_limit | [源码:125](../../../experiments/p0-egui/src/loading.rs#L125) |
| impl impl Fn() + Send + 'static, decode: impl Fn(&DecodeJob) -> Result<Decoded, String> + Send + 'static, limit: usize, ) -> Self | [源码:126](../../../experiments/p0-egui/src/loading.rs#L126) |
| impl impl Fn(&DecodeJob) -> Result<Decoded, String> + Send + 'static, limit: usize, ) -> Self | [源码:127](../../../experiments/p0-egui/src/loading.rs#L127) |
| fn replace | [源码:184](../../../experiments/p0-egui/src/loading.rs#L184) |
| fn dispatch | [源码:207](../../../experiments/p0-egui/src/loading.rs#L207) |
| fn poll | [源码:224](../../../experiments/p0-egui/src/loading.rs#L224) |
| fn is_idle | [源码:249](../../../experiments/p0-egui/src/loading.rs#L249) |
| fn outstanding_limit | [源码:252](../../../experiments/p0-egui/src/loading.rs#L252) |
| fn outstanding | [源码:255](../../../experiments/p0-egui/src/loading.rs#L255) |
| impl impl Drop for DecodeQueue | [源码:259](../../../experiments/p0-egui/src/loading.rs#L259) |
| fn drop | [源码:260](../../../experiments/p0-egui/src/loading.rs#L260) |
| struct UploadBudget | [源码:271](../../../experiments/p0-egui/src/loading.rs#L271) |
| impl impl UploadBudget | [源码:278](../../../experiments/p0-egui/src/loading.rs#L278) |
| fn begin | [源码:279](../../../experiments/p0-egui/src/loading.rs#L279) |
| fn take | [源码:286](../../../experiments/p0-egui/src/loading.rs#L286) |
| mod tests | [源码:299](../../../experiments/p0-egui/src/loading.rs#L299) |
| fn job | [源码:302](../../../experiments/p0-egui/src/loading.rs#L302) |
| fn image | [源码:309](../../../experiments/p0-egui/src/loading.rs#L309) |
| fn poll_after_wake | [源码:315](../../../experiments/p0-egui/src/loading.rs#L315) |
| fn replacement_discards_running_result_and_queued_jobs | [源码:323](../../../experiments/p0-egui/src/loading.rs#L323) |
| fn backpressure_does_not_decode_the_next_image_until_consumed | [源码:356](../../../experiments/p0-egui/src/loading.rs#L356) |
| fn prefetch_stops_at_two_and_rejects_both_obsolete_replies | [源码:377](../../../experiments/p0-egui/src/loading.rs#L377) |
| fn prefetched_queue_drop_with_full_result_slots_joins | [源码:404](../../../experiments/p0-egui/src/loading.rs#L404) |
| fn errors_invalid_plans_and_cancel_are_observable | [源码:420](../../../experiments/p0-egui/src/loading.rs#L420) |
| fn drop_with_unconsumed_result_joins_without_deadlock | [源码:443](../../../experiments/p0-egui/src/loading.rs#L443) |
| fn upload_limit_is_shared_by_all_passes_in_a_frame | [源码:457](../../../experiments/p0-egui/src/loading.rs#L457) |
| fn native_worker_reports_missing_and_corrupt_png | [源码:473](../../../experiments/p0-egui/src/loading.rs#L473) |

## experiments/p0-egui/src/main.rs

352 行；SHA256 `7fa519b9ef852393d1b25332406eb190712a3d073a7b42699be0c827fe59049f`

导入：`use eframe::egui;`, `use p0_preview_contract::{Probe, Scene, Tool};`, `use serde_json::json;`, `use std::{ path::PathBuf, sync::mpsc, time::{Duration, Instant}, };`

| 定义 | 位置／签名 |
| --- | --- |
| struct Candidate | [源码:13](../../../experiments/p0-egui/src/main.rs#L13) |
| impl impl Candidate | [源码:33](../../../experiments/p0-egui/src/main.rs#L33) |
| fn new | [源码:34](../../../experiments/p0-egui/src/main.rs#L34) |
| fn capture | [源码:62](../../../experiments/p0-egui/src/main.rs#L62) |
| fn finish | [源码:71](../../../experiments/p0-egui/src/main.rs#L71) |
| fn receive_captures | [源码:105](../../../experiments/p0-egui/src/main.rs#L105) |
| impl impl eframe::App for Candidate | [源码:167](../../../experiments/p0-egui/src/main.rs#L167) |
| fn logic | [源码:168](../../../experiments/p0-egui/src/main.rs#L168) |
| fn ui | [源码:191](../../../experiments/p0-egui/src/main.rs#L191) |
| fn main | [源码:316](../../../experiments/p0-egui/src/main.rs#L316) |

## experiments/p0-egui/src/streaming_decode.rs

415 行；SHA256 `c2dc8af188694fe330614e7ca18901a31cdbb80dc156306cc74d85671448a070`

导入：`use p0_media::compressed::Stream;`, `use p0_media::stream::Stream;`, `use p0_wasapi::buffered::Input;`, `use std::{ io::{Read, Seek}, sync::{ Arc, Condvar, Mutex, atomic::{AtomicBool, Ordering}, }, thread::{self, JoinHandle}, time::Duration, };`, `use super::*;`, `use p0_wasapi::buffered::{self, Config, Receiver};`, `use std::{io::Cursor, sync::atomic::AtomicUsize, time::Instant};`

| 定义 | 位置／签名 |
| --- | --- |
| struct GateState | [源码:18](../../../experiments/p0-egui/src/streaming_decode.rs#L18) |
| struct DiagnosticGate | [源码:24](../../../experiments/p0-egui/src/streaming_decode.rs#L24) |
| impl impl DiagnosticGate | [源码:25](../../../experiments/p0-egui/src/streaming_decode.rs#L25) |
| fn hold | [源码:26](../../../experiments/p0-egui/src/streaming_decode.rs#L26) |
| fn release | [源码:31](../../../experiments/p0-egui/src/streaming_decode.rs#L31) |
| fn held | [源码:35](../../../experiments/p0-egui/src/streaming_decode.rs#L35) |
| fn wait | [源码:38](../../../experiments/p0-egui/src/streaming_decode.rs#L38) |
| struct DecoderWorker | [源码:53](../../../experiments/p0-egui/src/streaming_decode.rs#L53) |
| impl impl DecoderWorker | [源码:59](../../../experiments/p0-egui/src/streaming_decode.rs#L59) |
| fn spawn | [源码:62](../../../experiments/p0-egui/src/streaming_decode.rs#L62) |
| impl impl Drop for DecoderWorker | [源码:149](../../../experiments/p0-egui/src/streaming_decode.rs#L149) |
| fn drop | [源码:150](../../../experiments/p0-egui/src/streaming_decode.rs#L150) |
| mod tests | [源码:162](../../../experiments/p0-egui/src/streaming_decode.rs#L162) |
| fn wav | [源码:166](../../../experiments/p0-egui/src/streaming_decode.rs#L166) |
| fn channel | [源码:187](../../../experiments/p0-egui/src/streaming_decode.rs#L187) |
| fn wait | [源码:196](../../../experiments/p0-egui/src/streaming_decode.rs#L196) |
| impl impl FnMut() -> bool) | [源码:196](../../../experiments/p0-egui/src/streaming_decode.rs#L196) |
| fn decoder_thread_backpressure_reuses_pool_and_shutdown_releases_waiter | [源码:204](../../../experiments/p0-egui/src/streaming_decode.rs#L204) |
| fn pause_and_seek_reject_stalled_completions_and_new_source_frame_is_exact | [源码:225](../../../experiments/p0-egui/src/streaming_decode.rs#L225) |
| fn source_open_failure_remains_faulted_until_new_generation_and_can_reopen | [源码:258](../../../experiments/p0-egui/src/streaming_decode.rs#L258) |
| struct ReadGate | [源码:285](../../../experiments/p0-egui/src/streaming_decode.rs#L285) |
| struct GatedMp3 | [源码:291](../../../experiments/p0-egui/src/streaming_decode.rs#L291) |
| impl impl Read for GatedMp3 | [源码:296](../../../experiments/p0-egui/src/streaming_decode.rs#L296) |
| fn read | [源码:297](../../../experiments/p0-egui/src/streaming_decode.rs#L297) |
| impl impl Seek for GatedMp3 | [源码:319](../../../experiments/p0-egui/src/streaming_decode.rs#L319) |
| fn seek | [源码:320](../../../experiments/p0-egui/src/streaming_decode.rs#L320) |
| fn cancel_prefix | [源码:325](../../../experiments/p0-egui/src/streaming_decode.rs#L325) |
| fn replacement_cancels_mp3_prefix_and_only_latest_request_is_published | [源码:402](../../../experiments/p0-egui/src/streaming_decode.rs#L402) |
| fn pause_cancels_mp3_prefix_without_faulting_or_publishing_old_pcm | [源码:407](../../../experiments/p0-egui/src/streaming_decode.rs#L407) |
| fn shutdown_cancels_mp3_prefix_and_returns_all_pool_leases | [源码:412](../../../experiments/p0-egui/src/streaming_decode.rs#L412) |

## experiments/p0-egui/src/texture_cache.rs

207 行；SHA256 `6d357c3955bae9ac769cccd6458b0163a249971c26ae048ed09d1a6275efc050`

导入：`use crate::Decoded;`, `use eframe::egui;`, `use std::collections::{BTreeMap, BTreeSet};`, `use super::*;`, `use std::sync::Arc;`

| 定义 | 位置／签名 |
| --- | --- |
| struct Entry | [源码:7](../../../experiments/p0-egui/src/texture_cache.rs#L7) |
| struct TextureCache | [源码:11](../../../experiments/p0-egui/src/texture_cache.rs#L11) |
| impl impl TextureCache | [源码:22](../../../experiments/p0-egui/src/texture_cache.rs#L22) |
| fn new | [源码:23](../../../experiments/p0-egui/src/texture_cache.rs#L23) |
| fn bytes | [源码:39](../../../experiments/p0-egui/src/texture_cache.rs#L39) |
| fn idle_bytes | [源码:42](../../../experiments/p0-egui/src/texture_cache.rs#L42) |
| fn get | [源码:45](../../../experiments/p0-egui/src/texture_cache.rs#L45) |
| fn upload | [源码:52](../../../experiments/p0-egui/src/texture_cache.rs#L52) |
| fn reset_bindings | [源码:104](../../../experiments/p0-egui/src/texture_cache.rs#L104) |
| fn clear | [源码:115](../../../experiments/p0-egui/src/texture_cache.rs#L115) |
| mod tests | [源码:122](../../../experiments/p0-egui/src/texture_cache.rs#L122) |
| fn image | [源码:125](../../../experiments/p0-egui/src/texture_cache.rs#L125) |
| fn reuse_preserves_pins_and_fails_atomically_when_all_pinned | [源码:129](../../../experiments/p0-egui/src/texture_cache.rs#L129) |
| fn reset_counts_idle_slots_partial_update_shares_pixels_and_close_frees | [源码:161](../../../experiments/p0-egui/src/texture_cache.rs#L161) |
| fn baseline_discards_evicted_textures_and_resets | [源码:192](../../../experiments/p0-egui/src/texture_cache.rs#L192) |

## experiments/p0-egui/src/upload_fence.rs

89 行；SHA256 `c0b488550013e15eb403043664f6782209f3eb7a23b3bf2fdd5a3c83faab4551`

导入：`use std::collections::BTreeMap;`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| struct UploadFence | [源码:4](../../../experiments/p0-egui/src/upload_fence.rs#L4) |
| impl impl UploadFence | [源码:10](../../../experiments/p0-egui/src/upload_fence.rs#L10) |
| fn bytes | [源码:11](../../../experiments/p0-egui/src/upload_fence.rs#L11) |
| fn allows | [源码:14](../../../experiments/p0-egui/src/upload_fence.rs#L14) |
| fn queued | [源码:17](../../../experiments/p0-egui/src/upload_fence.rs#L17) |
| fn arm_before | [源码:35](../../../experiments/p0-egui/src/upload_fence.rs#L35) |
| fn complete | [源码:48](../../../experiments/p0-egui/src/upload_fence.rs#L48) |
| static str | [源码:48](../../../experiments/p0-egui/src/upload_fence.rs#L48) |
| mod tests | [源码:58](../../../experiments/p0-egui/src/upload_fence.rs#L58) |
| fn blocks_later_frames_until_completion | [源码:61](../../../experiments/p0-egui/src/upload_fence.rs#L61) |
| fn cancellation_cannot_erase_physical_receipts | [源码:77](../../../experiments/p0-egui/src/upload_fence.rs#L77) |

## experiments/p0-egui/src/view.rs

149 行；SHA256 `07175368f9eac318b1a2ee899a6774643c459dd6e91f8b5bbdd9966f9843fe87`

导入：`use eframe::egui;`, `use std::time::Duration;`, `use super::*;`, `use std::time::Duration;`, `use std::sync::Arc;`

| 定义 | 位置／签名 |
| --- | --- |
| const LOADING_PERIOD | [源码:5](../../../experiments/p0-egui/src/view.rs#L5) |
| fn wake_loading | [源码:6](../../../experiments/p0-egui/src/view.rs#L6) |
| fn show_resource_list | [源码:11](../../../experiments/p0-egui/src/view.rs#L11) |
| impl impl FnOnce(&mut egui::Ui) -> R, ) -> Option<egui::scroll_area::ScrollAreaOutput<R>> | [源码:15](../../../experiments/p0-egui/src/view.rs#L15) |
| mod tests | [源码:25](../../../experiments/p0-egui/src/view.rs#L25) |
| fn completion_burst_does_not_request_immediate_or_settling_frames | [源码:30](../../../experiments/p0-egui/src/view.rs#L30) |
| fn removing_list_contents_stops_requesting_animation_frames | [源码:49](../../../experiments/p0-egui/src/view.rs#L49) |
| fn queued_texture_upload_shares_decoded_pixels | [源码:125](../../../experiments/p0-egui/src/view.rs#L125) |

## experiments/p0-egui/src/windows_audio.rs

60 行；SHA256 `ad79ea57baab103523495df300806b2ca052e4f081abf99f9c933ed44d867d5d`

导入：`use crate::{ audio_output::{Output, OutputError}, audio_transport::{AudioObservation, Intent, Relocate}, };`, `use p0_wasapi::{Device, Error, Phase, Snapshot};`

| 定义 | 位置／签名 |
| --- | --- |
| struct WindowsOutput | [源码:8](../../../experiments/p0-egui/src/windows_audio.rs#L8) |
| impl impl WindowsOutput | [源码:9](../../../experiments/p0-egui/src/windows_audio.rs#L9) |
| fn spawn | [源码:10](../../../experiments/p0-egui/src/windows_audio.rs#L10) |
| impl impl Fn() + Send + Sync + 'static) -> Result<Self, Error> | [源码:10](../../../experiments/p0-egui/src/windows_audio.rs#L10) |
| fn spawn_pcm | [源码:13](../../../experiments/p0-egui/src/windows_audio.rs#L13) |
| impl impl Fn() + Send + Sync + 'static) -> Result<Self, Error> | [源码:13](../../../experiments/p0-egui/src/windows_audio.rs#L13) |
| fn spawn_clip | [源码:16](../../../experiments/p0-egui/src/windows_audio.rs#L16) |
| impl impl Fn() + Send + Sync + 'static, ) -> Result<Self, Error> | [源码:18](../../../experiments/p0-egui/src/windows_audio.rs#L18) |
| fn spawn_buffered | [源码:22](../../../experiments/p0-egui/src/windows_audio.rs#L22) |
| impl impl Fn() + Send + Sync + 'static, ) -> Result<Self, Error> | [源码:24](../../../experiments/p0-egui/src/windows_audio.rs#L24) |
| fn spawn_buffered_converting | [源码:28](../../../experiments/p0-egui/src/windows_audio.rs#L28) |
| impl impl Fn() + Send + Sync + 'static, ) -> Result<Self, Error> | [源码:30](../../../experiments/p0-egui/src/windows_audio.rs#L30) |
| fn observation | [源码:35](../../../experiments/p0-egui/src/windows_audio.rs#L35) |
| impl impl Output for WindowsOutput | [源码:46](../../../experiments/p0-egui/src/windows_audio.rs#L46) |
| fn snapshot | [源码:47](../../../experiments/p0-egui/src/windows_audio.rs#L47) |
| fn control | [源码:50](../../../experiments/p0-egui/src/windows_audio.rs#L50) |
| fn relocate | [源码:55](../../../experiments/p0-egui/src/windows_audio.rs#L55) |

## experiments/p0-media/src/audio.rs

203 行；SHA256 `d5fda43254fc16e397e1a8b26ba2d05c10c59a640f2a312efca6060a9a38fe95`

导入：`use std::{io::Cursor, sync::Arc};`, `use symphonia::core::{ codecs::audio::AudioDecoderOptions, common::Limit, formats::{FormatOptions, TrackType, probe::Hint}, io::MediaSourceStream, meta::MetadataOptions, };`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| const MAX_ENCODED_BYTES | [源码:11](../../../experiments/p0-media/src/audio.rs#L11) |
| const MAX_DECODED_BYTES | [源码:12](../../../experiments/p0-media/src/audio.rs#L12) |
| const MAX_DURATION_US | [源码:13](../../../experiments/p0-media/src/audio.rs#L13) |
| enum DecodeError | [源码:15](../../../experiments/p0-media/src/audio.rs#L15) |
| struct Clip | [源码:23](../../../experiments/p0-media/src/audio.rs#L23) |
| impl impl Clip | [源码:28](../../../experiments/p0-media/src/audio.rs#L28) |
| fn rate | [源码:29](../../../experiments/p0-media/src/audio.rs#L29) |
| fn channels | [源码:32](../../../experiments/p0-media/src/audio.rs#L32) |
| fn into_samples | [源码:35](../../../experiments/p0-media/src/audio.rs#L35) |
| fn frames | [源码:38](../../../experiments/p0-media/src/audio.rs#L38) |
| fn duration_us | [源码:41](../../../experiments/p0-media/src/audio.rs#L41) |
| fn decode_wav | [源码:47](../../../experiments/p0-media/src/audio.rs#L47) |
| mod tests | [源码:140](../../../experiments/p0-media/src/audio.rs#L140) |
| fn wav | [源码:142](../../../experiments/p0-media/src/audio.rs#L142) |
| fn pcm16_preserves_channel_order_normalization_and_source_bytes | [源码:163](../../../experiments/p0-media/src/audio.rs#L163) |
| fn corrupt_truncated_empty_and_unsupported_shapes_never_publish_partial_pcm | [源码:172](../../../experiments/p0-media/src/audio.rs#L172) |
| fn limits_reject_oversized_input_and_duration_before_pcm_collection | [源码:193](../../../experiments/p0-media/src/audio.rs#L193) |

## experiments/p0-media/src/bin/compressed-proof.rs

100 行；SHA256 `1fc5e7069e101e66b55ade5201ededab216158bd7f67ae3ea694d442f1d6e23e`

导入：`use p0_media::{compressed::Stream, stream::MAX_BLOCK_FRAMES};`, `use std::{ fs::File, io::Write, path::PathBuf, time::{Duration, Instant}, };`

| 定义 | 位置／签名 |
| --- | --- |
| fn main | [源码:2](../../../experiments/p0-media/src/bin/compressed-proof.rs#L2) |
| fn main | [源码:97](../../../experiments/p0-media/src/bin/compressed-proof.rs#L97) |

## experiments/p0-media/src/bin/stream-proof.rs

171 行；SHA256 `85ad4a8057022843e9366577dbf343af0def9e477c6433d865bdded9f1f398e5`

导入：`use p0_media::stream::{CACHE_BYTES, MAX_BLOCK_FRAMES, SOURCE_BUFFER_BYTES, Stream};`, `use std::{ fs::File, io::{Read, Seek, SeekFrom, Write}, path::PathBuf, sync::{ Arc, atomic::{AtomicU64, Ordering}, }, time::{Duration, Instant}, };`

| 定义 | 位置／签名 |
| --- | --- |
| struct Counted | [源码:14](../../../experiments/p0-media/src/bin/stream-proof.rs#L14) |
| impl impl Read for Counted | [源码:18](../../../experiments/p0-media/src/bin/stream-proof.rs#L18) |
| fn read | [源码:19](../../../experiments/p0-media/src/bin/stream-proof.rs#L19) |
| impl impl Seek for Counted | [源码:25](../../../experiments/p0-media/src/bin/stream-proof.rs#L25) |
| fn seek | [源码:26](../../../experiments/p0-media/src/bin/stream-proof.rs#L26) |
| fn hash | [源码:30](../../../experiments/p0-media/src/bin/stream-proof.rs#L30) |
| fn run | [源码:38](../../../experiments/p0-media/src/bin/stream-proof.rs#L38) |
| fn main | [源码:166](../../../experiments/p0-media/src/bin/stream-proof.rs#L166) |

## experiments/p0-media/src/chart.rs

321 行；SHA256 `b8d42456c0605d5f681f79f91b6e9c2bd1eb4f2e0de0bb3a8ef09e2de75fc331`

导入：`use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| const MAX_TEXT_BYTES | [源码:2](../../../experiments/p0-media/src/chart.rs#L2) |
| const MAX_OBJECTS | [源码:3](../../../experiments/p0-media/src/chart.rs#L3) |
| const MAX_TIME_US | [源码:4](../../../experiments/p0-media/src/chart.rs#L4) |
| enum ErrorKind | [源码:6](../../../experiments/p0-media/src/chart.rs#L6) |
| struct Error | [源码:14](../../../experiments/p0-media/src/chart.rs#L14) |
| struct ObjectId | [源码:19](../../../experiments/p0-media/src/chart.rs#L19) |
| struct Circle | [源码:21](../../../experiments/p0-media/src/chart.rs#L21) |
| struct Chart | [源码:27](../../../experiments/p0-media/src/chart.rs#L27) |
| impl impl Chart | [源码:31](../../../experiments/p0-media/src/chart.rs#L31) |
| fn audio_file | [源码:32](../../../experiments/p0-media/src/chart.rs#L32) |
| fn objects | [源码:35](../../../experiments/p0-media/src/chart.rs#L35) |
| fn failure | [源码:39](../../../experiments/p0-media/src/chart.rs#L39) |
| fn parse | [源码:42](../../../experiments/p0-media/src/chart.rs#L42) |
| struct Window | [源码:152](../../../experiments/p0-media/src/chart.rs#L152) |
| enum Stage | [源码:157](../../../experiments/p0-media/src/chart.rs#L157) |
| struct Frame | [源码:163](../../../experiments/p0-media/src/chart.rs#L163) |
| struct Timeline | [源码:167](../../../experiments/p0-media/src/chart.rs#L167) |
| impl impl Timeline | [源码:173](../../../experiments/p0-media/src/chart.rs#L173) |
| fn new | [源码:174](../../../experiments/p0-media/src/chart.rs#L174) |
| fn frame | [源码:185](../../../experiments/p0-media/src/chart.rs#L185) |
| fn advance | [源码:205](../../../experiments/p0-media/src/chart.rs#L205) |
| fn seek | [源码:225](../../../experiments/p0-media/src/chart.rs#L225) |
| mod tests | [源码:239](../../../experiments/p0-media/src/chart.rs#L239) |
| fn source | [源码:241](../../../experiments/p0-media/src/chart.rs#L241) |
| fn public_v14_fields_keep_unicode_names_object_identity_and_equal_timestamps | [源码:247](../../../experiments/p0-media/src/chart.rs#L247) |
| fn invalid_and_unsupported_input_fails_without_silently_discarding_content | [源码:264](../../../experiments/p0-media/src/chart.rs#L264) |
| fn device_positions_emit_once_pause_is_idempotent_and_seek_does_not_replay_skipped_events | [源码:283](../../../experiments/p0-media/src/chart.rs#L283) |

## experiments/p0-media/src/compressed.rs

567 行；SHA256 `36aa0b7ce7229259603cff99e7993516633e49ed3a299d2797fd92d96777fa3d`

导入：`use crate::stream::{self, Block, Error, MAX_BLOCK_FRAMES, MAX_FILE_BYTES, MAX_SECONDS, Stats};`, `use std::io::{Read, Seek, SeekFrom};`, `use symphonia::core::{ codecs::audio::{AudioDecoder, AudioDecoderOptions}, common::Limit, formats::{FormatOptions, FormatReader, SeekMode, SeekTo, TrackType, probe::Hint}, io::{MediaSource, MediaSourceStream, MediaSourceStreamOptions}, meta::MetadataOptions, units::Timestamp, };`, `use super::*;`, `use std::io::Cursor;`

| 定义 | 位置／签名 |
| --- | --- |
| const MAX_PACKET_FRAMES | [源码:12](../../../experiments/p0-media/src/compressed.rs#L12) |
| const MAX_PACKET_BYTES | [源码:13](../../../experiments/p0-media/src/compressed.rs#L13) |
| const MAX_PACKETS | [源码:14](../../../experiments/p0-media/src/compressed.rs#L14) |
| const CACHE_BYTES | [源码:15](../../../experiments/p0-media/src/compressed.rs#L15) |
| struct Source | [源码:16](../../../experiments/p0-media/src/compressed.rs#L16) |
| impl impl<S: Read> Read for Source<S> | [源码:20](../../../experiments/p0-media/src/compressed.rs#L20) |
| fn read | [源码:21](../../../experiments/p0-media/src/compressed.rs#L21) |
| impl impl<S: Seek> Seek for Source<S> | [源码:25](../../../experiments/p0-media/src/compressed.rs#L25) |
| fn seek | [源码:26](../../../experiments/p0-media/src/compressed.rs#L26) |
| impl impl<S: Read + Seek + Send + Sync> MediaSource for Source<S> | [源码:30](../../../experiments/p0-media/src/compressed.rs#L30) |
| fn is_seekable | [源码:31](../../../experiments/p0-media/src/compressed.rs#L31) |
| fn byte_len | [源码:34](../../../experiments/p0-media/src/compressed.rs#L34) |
| struct Stream | [源码:39](../../../experiments/p0-media/src/compressed.rs#L39) |
| enum Inner | [源码:40](../../../experiments/p0-media/src/compressed.rs#L40) |
| struct Decoded | [源码:44](../../../experiments/p0-media/src/compressed.rs#L44) |
| impl impl Stream | [源码:62](../../../experiments/p0-media/src/compressed.rs#L62) |
| fn open | [源码:63](../../../experiments/p0-media/src/compressed.rs#L63) |
| fn rate | [源码:154](../../../experiments/p0-media/src/compressed.rs#L154) |
| fn channels | [源码:160](../../../experiments/p0-media/src/compressed.rs#L160) |
| fn total_frames | [源码:166](../../../experiments/p0-media/src/compressed.rs#L166) |
| fn position | [源码:172](../../../experiments/p0-media/src/compressed.rs#L172) |
| fn generation | [源码:178](../../../experiments/p0-media/src/compressed.rs#L178) |
| fn stats | [源码:184](../../../experiments/p0-media/src/compressed.rs#L184) |
| fn read | [源码:190](../../../experiments/p0-media/src/compressed.rs#L190) |
| fn seek | [源码:198](../../../experiments/p0-media/src/compressed.rs#L198) |
| fn seek_step | [源码:207](../../../experiments/p0-media/src/compressed.rs#L207) |
| impl impl Decoded | [源码:215](../../../experiments/p0-media/src/compressed.rs#L215) |
| fn seek | [源码:216](../../../experiments/p0-media/src/compressed.rs#L216) |
| fn read | [源码:276](../../../experiments/p0-media/src/compressed.rs#L276) |
| fn refill | [源码:319](../../../experiments/p0-media/src/compressed.rs#L319) |
| mod tests | [源码:373](../../../experiments/p0-media/src/compressed.rs#L373) |
| const MP3 | [源码:376](../../../experiments/p0-media/src/compressed.rs#L376) |
| const OGG | [源码:377](../../../experiments/p0-media/src/compressed.rs#L377) |
| fn decode | [源码:378](../../../experiments/p0-media/src/compressed.rs#L378) |
| fn mp3_gapless_samples_match_independent_ffmpeg_reference | [源码:394](../../../experiments/p0-media/src/compressed.rs#L394) |
| fn short_vorbis_including_tail_matches_independent_libsndfile | [源码:403](../../../experiments/p0-media/src/compressed.rs#L403) |
| fn vorbis_multipage_random_seeks_are_exact_without_decoding_the_prefix | [源码:413](../../../experiments/p0-media/src/compressed.rs#L413) |
| fn both_codecs_seek_after_eof_forward_and_backward_match_linear_decode | [源码:436](../../../experiments/p0-media/src/compressed.rs#L436) |
| fn invalid_requests_preserve_stream_position_and_do_not_clear_caller_data | [源码:458](../../../experiments/p0-media/src/compressed.rs#L458) |
| fn mp3_seek_steps_are_bounded_and_preserve_exact_samples | [源码:470](../../../experiments/p0-media/src/compressed.rs#L470) |
| fn partial_mp3_seek_can_be_replaced_and_invalid_target_is_non_mutating | [源码:495](../../../experiments/p0-media/src/compressed.rs#L495) |
| fn partial_mp3_seek_failure_stays_poisoned_and_cannot_complete_later | [源码:513](../../../experiments/p0-media/src/compressed.rs#L513) |
| fn truncated_packets_fail_without_delivering_a_partial_block_and_stay_poisoned | [源码:533](../../../experiments/p0-media/src/compressed.rs#L533) |
| fn malformed_and_oversized_sources_are_rejected | [源码:552](../../../experiments/p0-media/src/compressed.rs#L552) |
| struct Huge | [源码:554](../../../experiments/p0-media/src/compressed.rs#L554) |
| impl impl Read for Huge | [源码:555](../../../experiments/p0-media/src/compressed.rs#L555) |
| fn read | [源码:556](../../../experiments/p0-media/src/compressed.rs#L556) |
| impl impl Seek for Huge | [源码:560](../../../experiments/p0-media/src/compressed.rs#L560) |
| fn seek | [源码:561](../../../experiments/p0-media/src/compressed.rs#L561) |

## experiments/p0-media/src/lib.rs

6 行；SHA256 `cca5506f125ac02e641a269ab702ad412c2d8228f752c86ae4fd67c5ce4190c5`

导入：

| 定义 | 位置／签名 |
| --- | --- |
| mod audio | [源码:2](../../../experiments/p0-media/src/lib.rs#L2) |
| mod chart | [源码:3](../../../experiments/p0-media/src/lib.rs#L3) |
| mod compressed | [源码:5](../../../experiments/p0-media/src/lib.rs#L5) |
| mod stream | [源码:6](../../../experiments/p0-media/src/lib.rs#L6) |

## experiments/p0-media/src/stream.rs

546 行；SHA256 `904ee701d3b648ad39cfe95b5d80e4d66405c562b96945a7553d9634a4ed3013`

导入：`use std::io::{Read, Seek, SeekFrom};`, `use symphonia::core::{ codecs::audio::{AudioDecoder, AudioDecoderOptions}, common::Limit, formats::{FormatOptions, FormatReader, SeekMode, SeekTo, TrackType, probe::Hint}, io::{MediaSource, MediaSourceStream, MediaSourceStreamOptions}, meta::MetadataOptions, units::Timestamp, };`, `use super::*;`, `use std::{ io::Cursor, sync::{ Arc, atomic::{AtomicBool, AtomicU64, Ordering}, }, };`

| 定义 | 位置／签名 |
| --- | --- |
| const MAX_BLOCK_FRAMES | [源码:12](../../../experiments/p0-media/src/stream.rs#L12) |
| const MAX_FILE_BYTES | [源码:13](../../../experiments/p0-media/src/stream.rs#L13) |
| const MAX_SECONDS | [源码:14](../../../experiments/p0-media/src/stream.rs#L14) |
| const SOURCE_BUFFER_BYTES | [源码:15](../../../experiments/p0-media/src/stream.rs#L15) |
| const CACHE_BYTES | [源码:16](../../../experiments/p0-media/src/stream.rs#L16) |
| enum Error | [源码:19](../../../experiments/p0-media/src/stream.rs#L19) |
| struct Block | [源码:28](../../../experiments/p0-media/src/stream.rs#L28) |
| struct Stats | [源码:35](../../../experiments/p0-media/src/stream.rs#L35) |
| struct Source | [源码:40](../../../experiments/p0-media/src/stream.rs#L40) |
| impl impl<S: Read> Read for Source<S> | [源码:44](../../../experiments/p0-media/src/stream.rs#L44) |
| fn read | [源码:45](../../../experiments/p0-media/src/stream.rs#L45) |
| impl impl<S: Seek> Seek for Source<S> | [源码:49](../../../experiments/p0-media/src/stream.rs#L49) |
| fn seek | [源码:50](../../../experiments/p0-media/src/stream.rs#L50) |
| impl impl<S: Read + Seek + Send + Sync> MediaSource for Source<S> | [源码:54](../../../experiments/p0-media/src/stream.rs#L54) |
| fn is_seekable | [源码:55](../../../experiments/p0-media/src/stream.rs#L55) |
| fn byte_len | [源码:58](../../../experiments/p0-media/src/stream.rs#L58) |
| struct Stream | [源码:63](../../../experiments/p0-media/src/stream.rs#L63) |
| impl impl Stream | [源码:79](../../../experiments/p0-media/src/stream.rs#L79) |
| fn open | [源码:82](../../../experiments/p0-media/src/stream.rs#L82) |
| fn rate | [源码:166](../../../experiments/p0-media/src/stream.rs#L166) |
| fn channels | [源码:169](../../../experiments/p0-media/src/stream.rs#L169) |
| fn total_frames | [源码:172](../../../experiments/p0-media/src/stream.rs#L172) |
| fn position | [源码:175](../../../experiments/p0-media/src/stream.rs#L175) |
| fn generation | [源码:178](../../../experiments/p0-media/src/stream.rs#L178) |
| fn stats | [源码:181](../../../experiments/p0-media/src/stream.rs#L181) |
| fn seek | [源码:186](../../../experiments/p0-media/src/stream.rs#L186) |
| fn read | [源码:236](../../../experiments/p0-media/src/stream.rs#L236) |
| fn read_inner | [源码:262](../../../experiments/p0-media/src/stream.rs#L262) |
| fn refill | [源码:281](../../../experiments/p0-media/src/stream.rs#L281) |
| mod tests | [源码:333](../../../experiments/p0-media/src/stream.rs#L333) |
| fn wav | [源码:342](../../../experiments/p0-media/src/stream.rs#L342) |
| fn expected | [源码:362](../../../experiments/p0-media/src/stream.rs#L362) |
| fn arbitrary_pull_sizes_preserve_every_frame_channel_and_zero_pad_eof | [源码:366](../../../experiments/p0-media/src/stream.rs#L366) |
| fn unaligned_forward_backward_and_end_seeks_never_reuse_old_cached_pcm | [源码:392](../../../experiments/p0-media/src/stream.rs#L392) |
| fn invalid_requests_leave_position_generation_cache_and_output_unchanged | [源码:409](../../../experiments/p0-media/src/stream.rs#L409) |
| struct Watched | [源码:426](../../../experiments/p0-media/src/stream.rs#L426) |
| impl impl Read for Watched | [源码:431](../../../experiments/p0-media/src/stream.rs#L431) |
| fn read | [源码:432](../../../experiments/p0-media/src/stream.rs#L432) |
| impl impl Seek for Watched | [源码:440](../../../experiments/p0-media/src/stream.rs#L440) |
| fn seek | [源码:441](../../../experiments/p0-media/src/stream.rs#L441) |
| fn consumer_backpressure_has_no_background_work_and_reads_only_a_bounded_prefix | [源码:446](../../../experiments/p0-media/src/stream.rs#L446) |
| fn source_failure_after_partial_work_clears_output_and_requires_reopen | [源码:466](../../../experiments/p0-media/src/stream.rs#L466) |
| fn strict_layout_truncation_size_duration_and_format_limits_are_explicit | [源码:487](../../../experiments/p0-media/src/stream.rs#L487) |
| struct Huge | [源码:501](../../../experiments/p0-media/src/stream.rs#L501) |
| impl impl Read for Huge | [源码:502](../../../experiments/p0-media/src/stream.rs#L502) |
| fn read | [源码:503](../../../experiments/p0-media/src/stream.rs#L503) |
| impl impl Seek for Huge | [源码:507](../../../experiments/p0-media/src/stream.rs#L507) |
| fn seek | [源码:508](../../../experiments/p0-media/src/stream.rs#L508) |
| struct HeaderOnly | [源码:515](../../../experiments/p0-media/src/stream.rs#L515) |
| impl impl Read for HeaderOnly | [源码:519](../../../experiments/p0-media/src/stream.rs#L519) |
| fn read | [源码:520](../../../experiments/p0-media/src/stream.rs#L520) |
| impl impl Seek for HeaderOnly | [源码:524](../../../experiments/p0-media/src/stream.rs#L524) |
| fn seek | [源码:525](../../../experiments/p0-media/src/stream.rs#L525) |

## experiments/p0-preview-contract/src/lib.rs

203 行；SHA256 `7e7164428f5f534b7c8a44b47aa57e79529f928fad7ea75a1f23f827a85250b0`

导入：

| 定义 | 位置／签名 |
| --- | --- |
| enum Tool | [源码:5](../../../experiments/p0-preview-contract/src/lib.rs#L5) |
| enum Scene | [源码:11](../../../experiments/p0-preview-contract/src/lib.rs#L11) |
| enum Error | [源码:17](../../../experiments/p0-preview-contract/src/lib.rs#L17) |
| struct Observation | [源码:23](../../../experiments/p0-preview-contract/src/lib.rs#L23) |
| struct Stamp | [源码:34](../../../experiments/p0-preview-contract/src/lib.rs#L34) |
| struct RenderRequest | [源码:41](../../../experiments/p0-preview-contract/src/lib.rs#L41) |
| struct Frame | [源码:46](../../../experiments/p0-preview-contract/src/lib.rs#L46) |
| impl impl Frame | [源码:51](../../../experiments/p0-preview-contract/src/lib.rs#L51) |
| fn rgb | [源码:52](../../../experiments/p0-preview-contract/src/lib.rs#L52) |
| impl impl RenderRequest | [源码:57](../../../experiments/p0-preview-contract/src/lib.rs#L57) |
| fn render | [源码:59](../../../experiments/p0-preview-contract/src/lib.rs#L59) |
| struct Probe | [源码:76](../../../experiments/p0-preview-contract/src/lib.rs#L76) |
| impl impl Probe | [源码:86](../../../experiments/p0-preview-contract/src/lib.rs#L86) |
| fn new | [源码:87](../../../experiments/p0-preview-contract/src/lib.rs#L87) |
| fn observe | [源码:106](../../../experiments/p0-preview-contract/src/lib.rs#L106) |
| fn tick | [源码:110](../../../experiments/p0-preview-contract/src/lib.rs#L110) |
| fn pause | [源码:123](../../../experiments/p0-preview-contract/src/lib.rs#L123) |
| fn replace_material | [源码:134](../../../experiments/p0-preview-contract/src/lib.rs#L134) |
| fn select_scene | [源码:151](../../../experiments/p0-preview-contract/src/lib.rs#L151) |
| fn select_tool | [源码:158](../../../experiments/p0-preview-contract/src/lib.rs#L158) |
| fn set_focus | [源码:162](../../../experiments/p0-preview-contract/src/lib.rs#L162) |
| fn game_press | [源码:167](../../../experiments/p0-preview-contract/src/lib.rs#L167) |
| fn stamp | [源码:175](../../../experiments/p0-preview-contract/src/lib.rs#L175) |
| fn render_request | [源码:184](../../../experiments/p0-preview-contract/src/lib.rs#L184) |
| fn present | [源码:196](../../../experiments/p0-preview-contract/src/lib.rs#L196) |

## experiments/p0-preview-contract/src/main.rs

29 行；SHA256 `1edf475299f1e9a664b3f3dc1057288781799e64038567619c6a99f4e41fc296`

导入：`use p0_preview_contract::{Probe, Tool};`

| 定义 | 位置／签名 |
| --- | --- |
| fn main | [源码:3](../../../experiments/p0-preview-contract/src/main.rs#L3) |

## experiments/p0-preview-contract/tests/contract.rs

126 行；SHA256 `8949852b827c8b873f22b70d056e742c42a0165c08e04418abfadbc0a6e0aafb`

导入：`use p0_preview_contract::{Error, Probe, Scene, Tool};`

| 定义 | 位置／签名 |
| --- | --- |
| fn paused_material_edit_changes_pixels_without_advancing_simulation | [源码:4](../../../experiments/p0-preview-contract/tests/contract.rs#L4) |
| fn resume_without_any_paused_ticks_does_not_catch_up | [源码:22](../../../experiments/p0-preview-contract/tests/contract.rs#L22) |
| fn paused_inputs_are_discarded_instead_of_replayed_on_resume | [源码:32](../../../experiments/p0-preview-contract/tests/contract.rs#L32) |
| fn edit_tool_and_focus_prevent_game_interaction | [源码:47](../../../experiments/p0-preview-contract/tests/contract.rs#L47) |
| fn outdated_render_cannot_overwrite_a_new_material_or_scene | [源码:60](../../../experiments/p0-preview-contract/tests/contract.rs#L60) |
| fn paused_idle_does_not_request_repeated_frames | [源码:73](../../../experiments/p0-preview-contract/tests/contract.rs#L73) |
| fn scene_control_works_while_paused_without_a_content_edit | [源码:86](../../../experiments/p0-preview-contract/tests/contract.rs#L86) |
| fn stale_content_command_is_rejected_without_state_or_frame_change | [源码:99](../../../experiments/p0-preview-contract/tests/contract.rs#L99) |
| fn no_change_does_not_invalidate_a_presented_frame | [源码:110](../../../experiments/p0-preview-contract/tests/contract.rs#L110) |
| fn backwards_wall_time_is_rejected_atomically | [源码:120](../../../experiments/p0-preview-contract/tests/contract.rs#L120) |

## experiments/p0-wasapi/src/buffered.rs

789 行；SHA256 `fbaba83c2f4bc9d13726296cff9d10d05fe0f897ddde096296045432de03d224`

导入：`use crate::{Error, pcm};`, `use std::{ collections::VecDeque, sync::{Arc, Condvar, Mutex}, time::Duration, };`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| const BLOCK_FRAMES | [源码:9](../../../experiments/p0-wasapi/src/buffered.rs#L9) |
| const QUEUE_BLOCKS | [源码:10](../../../experiments/p0-wasapi/src/buffered.rs#L10) |
| const POOL_BLOCKS | [源码:11](../../../experiments/p0-wasapi/src/buffered.rs#L11) |
| const TRACE_LIMIT | [源码:12](../../../experiments/p0-wasapi/src/buffered.rs#L12) |
| struct Config | [源码:15](../../../experiments/p0-wasapi/src/buffered.rs#L15) |
| enum Kind | [源码:22](../../../experiments/p0-wasapi/src/buffered.rs#L22) |
| struct Event | [源码:31](../../../experiments/p0-wasapi/src/buffered.rs#L31) |
| struct Stats | [源码:40](../../../experiments/p0-wasapi/src/buffered.rs#L40) |
| struct Work | [源码:58](../../../experiments/p0-wasapi/src/buffered.rs#L58) |
| impl impl Work | [源码:63](../../../experiments/p0-wasapi/src/buffered.rs#L63) |
| fn generation | [源码:64](../../../experiments/p0-wasapi/src/buffered.rs#L64) |
| fn first_frame | [源码:67](../../../experiments/p0-wasapi/src/buffered.rs#L67) |
| fn samples_mut | [源码:70](../../../experiments/p0-wasapi/src/buffered.rs#L70) |
| enum Completion | [源码:75](../../../experiments/p0-wasapi/src/buffered.rs#L75) |
| struct Prepared | [源码:80](../../../experiments/p0-wasapi/src/buffered.rs#L80) |
| struct Queued | [源码:84](../../../experiments/p0-wasapi/src/buffered.rs#L84) |
| struct State | [源码:90](../../../experiments/p0-wasapi/src/buffered.rs#L90) |
| struct Shared | [源码:100](../../../experiments/p0-wasapi/src/buffered.rs#L100) |
| struct Input | [源码:106](../../../experiments/p0-wasapi/src/buffered.rs#L106) |
| struct Receiver | [源码:108](../../../experiments/p0-wasapi/src/buffered.rs#L108) |
| fn channel | [源码:109](../../../experiments/p0-wasapi/src/buffered.rs#L109) |
| impl impl Receiver | [源码:113](../../../experiments/p0-wasapi/src/buffered.rs#L113) |
| fn config | [源码:114](../../../experiments/p0-wasapi/src/buffered.rs#L114) |
| fn duration_us | [源码:117](../../../experiments/p0-wasapi/src/buffered.rs#L117) |
| fn activate | [源码:120](../../../experiments/p0-wasapi/src/buffered.rs#L120) |
| fn pause_at | [源码:123](../../../experiments/p0-wasapi/src/buffered.rs#L123) |
| fn ready | [源码:126](../../../experiments/p0-wasapi/src/buffered.rs#L126) |
| fn prepare | [源码:129](../../../experiments/p0-wasapi/src/buffered.rs#L129) |
| fn commit | [源码:132](../../../experiments/p0-wasapi/src/buffered.rs#L132) |
| fn invalid | [源码:136](../../../experiments/p0-wasapi/src/buffered.rs#L136) |
| static str | [源码:136](../../../experiments/p0-wasapi/src/buffered.rs#L136) |
| impl impl Input | [源码:142](../../../experiments/p0-wasapi/src/buffered.rs#L142) |
| fn new | [源码:143](../../../experiments/p0-wasapi/src/buffered.rs#L143) |
| fn config | [源码:171](../../../experiments/p0-wasapi/src/buffered.rs#L171) |
| fn duration_us | [源码:174](../../../experiments/p0-wasapi/src/buffered.rs#L174) |
| fn stats | [源码:177](../../../experiments/p0-wasapi/src/buffered.rs#L177) |
| fn trace | [源码:186](../../../experiments/p0-wasapi/src/buffered.rs#L186) |
| fn drain_trace | [源码:196](../../../experiments/p0-wasapi/src/buffered.rs#L196) |
| fn is_closed | [源码:201](../../../experiments/p0-wasapi/src/buffered.rs#L201) |
| fn is_current | [源码:210](../../../experiments/p0-wasapi/src/buffered.rs#L210) |
| fn wake | [源码:214](../../../experiments/p0-wasapi/src/buffered.rs#L214) |
| fn close | [源码:217](../../../experiments/p0-wasapi/src/buffered.rs#L217) |
| fn work | [源码:224](../../../experiments/p0-wasapi/src/buffered.rs#L224) |
| fn complete | [源码:253](../../../experiments/p0-wasapi/src/buffered.rs#L253) |
| fn fail | [源码:302](../../../experiments/p0-wasapi/src/buffered.rs#L302) |
| fn record | [源码:329](../../../experiments/p0-wasapi/src/buffered.rs#L329) |
| fn discard | [源码:336](../../../experiments/p0-wasapi/src/buffered.rs#L336) |
| fn restart | [源码:343](../../../experiments/p0-wasapi/src/buffered.rs#L343) |
| fn retire | [源码:389](../../../experiments/p0-wasapi/src/buffered.rs#L389) |
| fn ready | [源码:409](../../../experiments/p0-wasapi/src/buffered.rs#L409) |
| fn underrun | [源码:422](../../../experiments/p0-wasapi/src/buffered.rs#L422) |
| fn prepare | [源码:441](../../../experiments/p0-wasapi/src/buffered.rs#L441) |
| fn commit | [源码:498](../../../experiments/p0-wasapi/src/buffered.rs#L498) |
| mod tests | [源码:550](../../../experiments/p0-wasapi/src/buffered.rs#L550) |
| fn input | [源码:552](../../../experiments/p0-wasapi/src/buffered.rs#L552) |
| fn fill | [源码:561](../../../experiments/p0-wasapi/src/buffered.rs#L561) |
| fn pool_queue_and_inflight_are_bounded_and_backpressure_releases_on_commit | [源码:571](../../../experiments/p0-wasapi/src/buffered.rs#L571) |
| fn current_generation_check_rejects_replacement_retirement_and_close | [源码:591](../../../experiments/p0-wasapi/src/buffered.rs#L591) |
| fn draining_long_trace_preserves_every_packet_and_pool_bounds | [源码:611](../../../experiments/p0-wasapi/src/buffered.rs#L611) |
| fn draining_cannot_hide_a_preexisting_trace_overflow | [源码:644](../../../experiments/p0-wasapi/src/buffered.rs#L644) |
| fn prepare_is_retryable_and_only_successful_native_commit_consumes_frames | [源码:655](../../../experiments/p0-wasapi/src/buffered.rs#L655) |
| fn pause_and_seek_drop_queued_and_inflight_old_generations_and_restart_at_exact_origin | [源码:677](../../../experiments/p0-wasapi/src/buffered.rs#L677) |
| fn eof_padding_is_distinct_from_starvation_and_does_not_repeat_source | [源码:697](../../../experiments/p0-wasapi/src/buffered.rs#L697) |
| fn corrupt_completions_fail_closed_and_explicit_restart_is_required | [源码:713](../../../experiments/p0-wasapi/src/buffered.rs#L713) |
| fn invalid_config_origin_and_packet_do_not_modify_valid_queue | [源码:729](../../../experiments/p0-wasapi/src/buffered.rs#L729) |
| fn retirement_and_close_reject_late_completions_and_recycle_every_buffer | [源码:757](../../../experiments/p0-wasapi/src/buffered.rs#L757) |
| fn same_origin_new_generation_rejects_old_prepared_packet_and_old_decode_error | [源码:774](../../../experiments/p0-wasapi/src/buffered.rs#L774) |

## experiments/p0-wasapi/src/lib.rs

878 行；SHA256 `dbdeaf137af17cdac2176ef816775f588166b448f597566c2cfc5a3a91e76dea`

导入：`use std::{ sync::{ Arc, Mutex, atomic::{AtomicBool, Ordering}, mpsc, }, thread::{self, JoinHandle}, time::{Duration, Instant}, };`, `use super::*;`, `use std::sync::atomic::AtomicUsize;`

| 定义 | 位置／签名 |
| --- | --- |
| mod buffered | [源码:2](../../../experiments/p0-wasapi/src/lib.rs#L2) |
| mod pcm | [源码:3](../../../experiments/p0-wasapi/src/lib.rs#L3) |
| enum Phase | [源码:15](../../../experiments/p0-wasapi/src/lib.rs#L15) |
| struct Error | [源码:22](../../../experiments/p0-wasapi/src/lib.rs#L22) |
| static str | [源码:23](../../../experiments/p0-wasapi/src/lib.rs#L23) |
| struct Snapshot | [源码:27](../../../experiments/p0-wasapi/src/lib.rs#L27) |
| struct Format | [源码:40](../../../experiments/p0-wasapi/src/lib.rs#L40) |
| impl impl Default for Snapshot | [源码:51](../../../experiments/p0-wasapi/src/lib.rs#L51) |
| fn default | [源码:52](../../../experiments/p0-wasapi/src/lib.rs#L52) |
| enum Control | [源码:67](../../../experiments/p0-wasapi/src/lib.rs#L67) |
| enum Content | [源码:81](../../../experiments/p0-wasapi/src/lib.rs#L81) |
| impl impl Content | [源码:90](../../../experiments/p0-wasapi/src/lib.rs#L90) |
| fn duration_us | [源码:91](../../../experiments/p0-wasapi/src/lib.rs#L91) |
| struct Device | [源码:99](../../../experiments/p0-wasapi/src/lib.rs#L99) |
| impl impl Device | [源码:106](../../../experiments/p0-wasapi/src/lib.rs#L106) |
| fn spawn | [源码:107](../../../experiments/p0-wasapi/src/lib.rs#L107) |
| impl impl Fn() + Send + Sync + 'static) -> Result<Self, Error> | [源码:107](../../../experiments/p0-wasapi/src/lib.rs#L107) |
| fn spawn_pcm | [源码:111](../../../experiments/p0-wasapi/src/lib.rs#L111) |
| impl impl Fn() + Send + Sync + 'static) -> Result<Self, Error> | [源码:111](../../../experiments/p0-wasapi/src/lib.rs#L111) |
| fn spawn_clip | [源码:114](../../../experiments/p0-wasapi/src/lib.rs#L114) |
| impl impl Fn() + Send + Sync + 'static, ) -> Result<Self, Error> | [源码:116](../../../experiments/p0-wasapi/src/lib.rs#L116) |
| fn spawn_buffered | [源码:120](../../../experiments/p0-wasapi/src/lib.rs#L120) |
| impl impl Fn() + Send + Sync + 'static, ) -> Result<Self, Error> | [源码:122](../../../experiments/p0-wasapi/src/lib.rs#L122) |
| fn spawn_buffered_converting | [源码:134](../../../experiments/p0-wasapi/src/lib.rs#L134) |
| impl impl Fn() + Send + Sync + 'static, ) -> Result<Self, Error> | [源码:136](../../../experiments/p0-wasapi/src/lib.rs#L136) |
| fn spawn_mode | [源码:146](../../../experiments/p0-wasapi/src/lib.rs#L146) |
| impl impl Fn() + Send + Sync + 'static, content: Content, ) -> Result<Self, Error> | [源码:147](../../../experiments/p0-wasapi/src/lib.rs#L147) |
| fn request | [源码:194](../../../experiments/p0-wasapi/src/lib.rs#L194) |
| fn request_in | [源码:197](../../../experiments/p0-wasapi/src/lib.rs#L197) |
| fn rebuild | [源码:205](../../../experiments/p0-wasapi/src/lib.rs#L205) |
| fn relocate | [源码:208](../../../experiments/p0-wasapi/src/lib.rs#L208) |
| fn invalidate | [源码:219](../../../experiments/p0-wasapi/src/lib.rs#L219) |
| fn send | [源码:222](../../../experiments/p0-wasapi/src/lib.rs#L222) |
| fn snapshot | [源码:228](../../../experiments/p0-wasapi/src/lib.rs#L228) |
| impl impl Drop for Device | [源码:232](../../../experiments/p0-wasapi/src/lib.rs#L232) |
| fn drop | [源码:233](../../../experiments/p0-wasapi/src/lib.rs#L233) |
| fn worker | [源码:241](../../../experiments/p0-wasapi/src/lib.rs#L241) |
| impl impl Fn(Snapshot), content: Content, ) -> Result<(), Error> | [源码:244](../../../experiments/p0-wasapi/src/lib.rs#L244) |
| trait StreamPort | [源码:257](../../../experiments/p0-wasapi/src/lib.rs#L257) |
| fn format | [源码:258](../../../experiments/p0-wasapi/src/lib.rs#L258) |
| fn pause | [源码:259](../../../experiments/p0-wasapi/src/lib.rs#L259) |
| fn feed | [源码:260](../../../experiments/p0-wasapi/src/lib.rs#L260) |
| fn position | [源码:261](../../../experiments/p0-wasapi/src/lib.rs#L261) |
| fn check_device | [源码:262](../../../experiments/p0-wasapi/src/lib.rs#L262) |
| fn ready | [源码:265](../../../experiments/p0-wasapi/src/lib.rs#L265) |
| fn buffered | [源码:268](../../../experiments/p0-wasapi/src/lib.rs#L268) |
| fn pcm | [源码:271](../../../experiments/p0-wasapi/src/lib.rs#L271) |
| impl impl StreamPort for platform::Stream | [源码:276](../../../experiments/p0-wasapi/src/lib.rs#L276) |
| fn check_device | [源码:277](../../../experiments/p0-wasapi/src/lib.rs#L277) |
| fn format | [源码:280](../../../experiments/p0-wasapi/src/lib.rs#L280) |
| fn pause | [源码:283](../../../experiments/p0-wasapi/src/lib.rs#L283) |
| fn feed | [源码:286](../../../experiments/p0-wasapi/src/lib.rs#L286) |
| fn position | [源码:289](../../../experiments/p0-wasapi/src/lib.rs#L289) |
| fn pcm | [源码:292](../../../experiments/p0-wasapi/src/lib.rs#L292) |
| fn ready | [源码:295](../../../experiments/p0-wasapi/src/lib.rs#L295) |
| fn buffered | [源码:298](../../../experiments/p0-wasapi/src/lib.rs#L298) |
| fn run_worker | [源码:304](../../../experiments/p0-wasapi/src/lib.rs#L304) |
| impl impl Fn(Snapshot), mut open: impl FnMut(u64) -> Result<S, Error>, ) | [源码:307](../../../experiments/p0-wasapi/src/lib.rs#L307) |
| impl impl FnMut(u64) -> Result<S, Error>, ) | [源码:308](../../../experiments/p0-wasapi/src/lib.rs#L308) |
| fn worker | [源码:424](../../../experiments/p0-wasapi/src/lib.rs#L424) |
| impl impl Fn(Snapshot), _: Content, ) -> Result<(), Error> | [源码:427](../../../experiments/p0-wasapi/src/lib.rs#L427) |
| mod platform | [源码:437](../../../experiments/p0-wasapi/src/lib.rs#L437) |
| mod tests | [源码:440](../../../experiments/p0-wasapi/src/lib.rs#L440) |
| struct Fake | [源码:443](../../../experiments/p0-wasapi/src/lib.rs#L443) |
| impl impl Drop for Fake | [源码:449](../../../experiments/p0-wasapi/src/lib.rs#L449) |
| fn drop | [源码:450](../../../experiments/p0-wasapi/src/lib.rs#L450) |
| impl impl StreamPort for Fake | [源码:457](../../../experiments/p0-wasapi/src/lib.rs#L457) |
| fn format | [源码:458](../../../experiments/p0-wasapi/src/lib.rs#L458) |
| fn pause | [源码:470](../../../experiments/p0-wasapi/src/lib.rs#L470) |
| fn feed | [源码:474](../../../experiments/p0-wasapi/src/lib.rs#L474) |
| fn position | [源码:484](../../../experiments/p0-wasapi/src/lib.rs#L484) |
| fn receive | [源码:488](../../../experiments/p0-wasapi/src/lib.rs#L488) |
| impl impl Fn(&Snapshot) -> bool) -> Snapshot | [源码:488](../../../experiments/p0-wasapi/src/lib.rs#L488) |
| fn changed_device_faults_during_pause_or_play_and_failed_reopen_never_resumes | [源码:500](../../../experiments/p0-wasapi/src/lib.rs#L500) |
| struct Monitored | [源码:501](../../../experiments/p0-wasapi/src/lib.rs#L501) |
| impl impl StreamPort for Monitored | [源码:506](../../../experiments/p0-wasapi/src/lib.rs#L506) |
| fn format | [源码:507](../../../experiments/p0-wasapi/src/lib.rs#L507) |
| fn pause | [源码:515](../../../experiments/p0-wasapi/src/lib.rs#L515) |
| fn feed | [源码:518](../../../experiments/p0-wasapi/src/lib.rs#L518) |
| fn position | [源码:521](../../../experiments/p0-wasapi/src/lib.rs#L521) |
| fn check_device | [源码:524](../../../experiments/p0-wasapi/src/lib.rs#L524) |
| fn rebuilding_releases_stream_before_open_and_rejects_old_epoch_controls | [源码:633](../../../experiments/p0-wasapi/src/lib.rs#L633) |
| fn failed_open_and_runtime_failure_stay_faulted_until_explicit_retry | [源码:697](../../../experiments/p0-wasapi/src/lib.rs#L697) |
| fn fault_is_observable_while_old_native_stream_cleanup_is_blocked | [源码:754](../../../experiments/p0-wasapi/src/lib.rs#L754) |
| fn play_waits_for_prefill_and_newer_pause_cancels_pending_start | [源码:794](../../../experiments/p0-wasapi/src/lib.rs#L794) |
| struct Waiting | [源码:795](../../../experiments/p0-wasapi/src/lib.rs#L795) |
| impl impl StreamPort for Waiting | [源码:799](../../../experiments/p0-wasapi/src/lib.rs#L799) |
| fn format | [源码:800](../../../experiments/p0-wasapi/src/lib.rs#L800) |
| fn pause | [源码:812](../../../experiments/p0-wasapi/src/lib.rs#L812) |
| fn ready | [源码:816](../../../experiments/p0-wasapi/src/lib.rs#L816) |
| fn feed | [源码:819](../../../experiments/p0-wasapi/src/lib.rs#L819) |
| fn position | [源码:822](../../../experiments/p0-wasapi/src/lib.rs#L822) |

## experiments/p0-wasapi/src/pcm.rs

341 行；SHA256 `f157d28b289c3bc37cdb544d2e904fac3f8669f7e095591e9cc2bef698995ae3`

导入：`use crate::Error;`, `use std::sync::Arc;`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| const DURATION_US | [源码:5](../../../experiments/p0-wasapi/src/pcm.rs#L5) |
| const AMPLITUDE | [源码:6](../../../experiments/p0-wasapi/src/pcm.rs#L6) |
| const MAX_PACKET_BYTES | [源码:7](../../../experiments/p0-wasapi/src/pcm.rs#L7) |
| struct Clip | [源码:10](../../../experiments/p0-wasapi/src/pcm.rs#L10) |
| impl impl Clip | [源码:16](../../../experiments/p0-wasapi/src/pcm.rs#L16) |
| fn new | [源码:17](../../../experiments/p0-wasapi/src/pcm.rs#L17) |
| fn duration_us | [源码:37](../../../experiments/p0-wasapi/src/pcm.rs#L37) |
| struct Packet | [源码:43](../../../experiments/p0-wasapi/src/pcm.rs#L43) |
| struct Stats | [源码:52](../../../experiments/p0-wasapi/src/pcm.rs#L52) |
| struct Cursor | [源码:61](../../../experiments/p0-wasapi/src/pcm.rs#L61) |
| fn invalid | [源码:67](../../../experiments/p0-wasapi/src/pcm.rs#L67) |
| impl impl Cursor | [源码:73](../../../experiments/p0-wasapi/src/pcm.rs#L73) |
| fn new | [源码:74](../../../experiments/p0-wasapi/src/pcm.rs#L74) |
| fn from_clip | [源码:94](../../../experiments/p0-wasapi/src/pcm.rs#L94) |
| fn stats | [源码:107](../../../experiments/p0-wasapi/src/pcm.rs#L107) |
| fn check_playhead | [源码:112](../../../experiments/p0-wasapi/src/pcm.rs#L112) |
| fn packet_bytes | [源码:122](../../../experiments/p0-wasapi/src/pcm.rs#L122) |
| fn prepare | [源码:133](../../../experiments/p0-wasapi/src/pcm.rs#L133) |
| fn commit | [源码:185](../../../experiments/p0-wasapi/src/pcm.rs#L185) |
| fn sample | [源码:215](../../../experiments/p0-wasapi/src/pcm.rs#L215) |
| mod tests | [源码:229](../../../experiments/p0-wasapi/src/pcm.rs#L229) |
| fn decoded_clip_preserves_stereo_gain_seek_eof_and_failed_submission | [源码:232](../../../experiments/p0-wasapi/src/pcm.rs#L232) |
| fn decoded_clip_rejects_nonfinite_shape_gain_and_duration | [源码:259](../../../experiments/p0-wasapi/src/pcm.rs#L259) |
| fn known_sample_channel_order_and_quiet_regions | [源码:274](../../../experiments/p0-wasapi/src/pcm.rs#L274) |
| fn accepted_packets_are_contiguous_failed_submission_does_not_skip_and_seek_is_sample_quantized | [源码:293](../../../experiments/p0-wasapi/src/pcm.rs#L293) |
| fn rejects_bad_shape_rate_position_and_allocation_before_writing | [源码:314](../../../experiments/p0-wasapi/src/pcm.rs#L314) |
| fn submission_frontier_is_not_playhead_and_underflow_requires_recovery | [源码:331](../../../experiments/p0-wasapi/src/pcm.rs#L331) |

## experiments/p0-wasapi/src/platform.rs

580 行；SHA256 `df89f7660e92128909e7a05e0e359064427b7ddeaf243c370d4f97097e84fc03`

导入：`use super::{Content, Error, Format, buffered, pcm};`, `use std::time::{Duration, Instant};`, `use windows::{ Win32::{Foundation::S_OK, Media::Audio::*, System::Com::*}, core::Interface, };`, `use super::*;`

| 定义 | 位置／签名 |
| --- | --- |
| fn error | [源码:8](../../../experiments/p0-wasapi/src/platform.rs#L8) |
| static str | [源码:8](../../../experiments/p0-wasapi/src/platform.rs#L8) |
| struct Apartment | [源码:14](../../../experiments/p0-wasapi/src/platform.rs#L14) |
| impl impl Apartment | [源码:15](../../../experiments/p0-wasapi/src/platform.rs#L15) |
| fn new | [源码:16](../../../experiments/p0-wasapi/src/platform.rs#L16) |
| impl impl Drop for Apartment | [源码:23](../../../experiments/p0-wasapi/src/platform.rs#L23) |
| fn drop | [源码:24](../../../experiments/p0-wasapi/src/platform.rs#L24) |
| struct Mix | [源码:28](../../../experiments/p0-wasapi/src/platform.rs#L28) |
| impl impl Drop for Mix | [源码:29](../../../experiments/p0-wasapi/src/platform.rs#L29) |
| fn drop | [源码:30](../../../experiments/p0-wasapi/src/platform.rs#L30) |
| struct Stream | [源码:34](../../../experiments/p0-wasapi/src/platform.rs#L34) |
| impl impl Stream | [源码:49](../../../experiments/p0-wasapi/src/platform.rs#L49) |
| fn open_content | [源码:50](../../../experiments/p0-wasapi/src/platform.rs#L50) |
| const WAVEFORMATEX | [源码:137](../../../experiments/p0-wasapi/src/platform.rs#L137) |
| fn check_device | [源码:213](../../../experiments/p0-wasapi/src/platform.rs#L213) |
| fn feed | [源码:233](../../../experiments/p0-wasapi/src/platform.rs#L233) |
| fn pcm_stats | [源码:299](../../../experiments/p0-wasapi/src/platform.rs#L299) |
| fn buffered_stats | [源码:302](../../../experiments/p0-wasapi/src/platform.rs#L302) |
| fn ready | [源码:305](../../../experiments/p0-wasapi/src/platform.rs#L305) |
| fn feed_buffered | [源码:310](../../../experiments/p0-wasapi/src/platform.rs#L310) |
| fn pause | [源码:359](../../../experiments/p0-wasapi/src/platform.rs#L359) |
| fn position | [源码:392](../../../experiments/p0-wasapi/src/platform.rs#L392) |
| fn raw_position | [源码:400](../../../experiments/p0-wasapi/src/platform.rs#L400) |
| fn endpoint_id | [源码:416](../../../experiments/p0-wasapi/src/platform.rs#L416) |
| fn float_format | [源码:430](../../../experiments/p0-wasapi/src/platform.rs#L430) |
| fn validate_float | [源码:448](../../../experiments/p0-wasapi/src/platform.rs#L448) |
| fn precise | [源码:471](../../../experiments/p0-wasapi/src/platform.rs#L471) |
| fn micros | [源码:481](../../../experiments/p0-wasapi/src/platform.rs#L481) |
| impl impl Drop for Stream | [源码:495](../../../experiments/p0-wasapi/src/platform.rs#L495) |
| fn drop | [源码:496](../../../experiments/p0-wasapi/src/platform.rs#L496) |
| mod tests | [源码:507](../../../experiments/p0-wasapi/src/platform.rs#L507) |
| fn conversion_describes_source_frames_without_reinterpreting_rate_or_channels | [源码:510](../../../experiments/p0-wasapi/src/platform.rs#L510) |
| fn clock_units_need_not_be_sample_frames | [源码:527](../../../experiments/p0-wasapi/src/platform.rs#L527) |
| fn pcm_format_requires_float_identity_valid_bits_and_exact_frame_layout | [源码:541](../../../experiments/p0-wasapi/src/platform.rs#L541) |

## tools/check_core.py

112 行；SHA256 `c7cd91465685a852498d7753db9a90fd7322799d167eb0d6ffb816ac6619e9cb`

导入：`argparse`, `check_core_architecture`, `json`, `os`, `pathlib`, `shutil`, `subprocess`, `sys`

| 定义 | 位置／签名 |
| --- | --- |
| function tool_environment | [源码:18](../../../tools/check_core.py#L18) · `def tool_environment(directory=None)` |
| function linker_arguments | [源码:53](../../../tools/check_core.py#L53) · `def linker_arguments(environment)` |
| function main | [源码:78](../../../tools/check_core.py#L78) · `def main()` |

## tools/check_core_architecture.py

81 行；SHA256 `6e4fa6ee969655f97a230f148396685a0a3905e8ca567cc4a79f03d2233a221f`

导入：`pathlib`, `re`

| 定义 | 位置／签名 |
| --- | --- |
| function validate_metadata | [源码:21](../../../tools/check_core_architecture.py#L21) · `def validate_metadata(metadata, root)` |
| function source_issues | [源码:64](../../../tools/check_core_architecture.py#L64) · `def source_issues(text)` |
| function validate_sources | [源码:70](../../../tools/check_core_architecture.py#L70) · `def validate_sources(root)` |

## tools/check_core_cost.py

225 行；SHA256 `32c83b988fefbde7cf2c3384252a9d5e52cc51a458d2e8341cf7b12780075cf0`

导入：`argparse`, `check_core`, `ctypes`, `hashlib`, `json`, `os`, `pathlib`, `platform`, `queue`, `subprocess`, `threading`, `time`

| 定义 | 位置／签名 |
| --- | --- |
| function validate_phase | [源码:25](../../../tools/check_core_cost.py#L25) · `def validate_phase(value, expected)` |
| function safe_output | [源码:42](../../../tools/check_core_cost.py#L42) · `def safe_output(value)` |
| function built_executable | [源码:52](../../../tools/check_core_cost.py#L52) · `def built_executable(output)` |
| class MemoryCounters | [源码:63](../../../tools/check_core_cost.py#L63) · `ctypes.Structure` |
| class WindowsProcess | [源码:72](../../../tools/check_core_cost.py#L72) |
| function WindowsProcess.__init__ | [源码:73](../../../tools/check_core_cost.py#L73) · `def __init__(self, pid)` |
| function WindowsProcess.sample | [源码:88](../../../tools/check_core_cost.py#L88) · `def sample(self)` |
| function WindowsProcess.close | [源码:101](../../../tools/check_core_cost.py#L101) · `def close(self)` |
| function run_probe | [源码:107](../../../tools/check_core_cost.py#L107) · `def run_probe(executable, profile, repetition, output)` |
| function run_probe.read_lines | [源码:115](../../../tools/check_core_cost.py#L115) · `def read_lines()` |
| function main | [源码:185](../../../tools/check_core_cost.py#L185) · `def main()` |

## tools/check_desktop.py

98 行；SHA256 `f767655c446d875252ee5ad96e86ff94ceaf3b2b0022906d735304613f3b55aa`

导入：`argparse`, `check_core`, `check_desktop_architecture`, `json`, `os`, `pathlib`, `subprocess`, `sys`

| 定义 | 位置／签名 |
| --- | --- |
| function main | [源码:11](../../../tools/check_desktop.py#L11) · `def main()` |

## tools/check_desktop_architecture.py

85 行；SHA256 `c84ac8a30b32c5fd445345bdbe064423abc8e46cce6e4bedd4a19da1df22e0e4`

导入：`pathlib`, `re`

| 定义 | 位置／签名 |
| --- | --- |
| function validate_metadata | [源码:21](../../../tools/check_desktop_architecture.py#L21) · `def validate_metadata(metadata, root)` |
| function source_issues | [源码:62](../../../tools/check_desktop_architecture.py#L62) · `def source_issues(module, source)` |
| function validate_sources | [源码:76](../../../tools/check_desktop_architecture.py#L76) · `def validate_sources(root)` |

## tools/check_p0.py

93 行；SHA256 `371d68565eeaf7959c2062aae48b39565678dcd4c4efc7fff43d719f449216cc`

导入：`json`, `os`, `pathlib`, `re`, `shutil`, `subprocess`, `sys`

| 定义 | 位置／签名 |
| --- | --- |
| function run | [源码:16](../../../tools/check_p0.py#L16) · `def run(arguments, environment)` |
| function windows_linker | [源码:20](../../../tools/check_p0.py#L20) · `def windows_linker(environment)` |
| function check_isolation | [源码:52](../../../tools/check_p0.py#L52) · `def check_isolation(cargo, environment)` |
| function main | [源码:74](../../../tools/check_p0.py#L74) · `def main()` |

## tools/check_p0_audio_adaptation.py

267 行；SHA256 `39b26ef5964c436f9b9aef4c18ba475efaf4fa982da1d78b882d0f1da22aa1b0`

导入：`argparse`, `array`, `check_p0`, `check_p0_compressed`, `check_p0_switch`, `ctypes`, `hashlib`, `json`, `math`, `p0_process_metrics`, `pathlib`, `shutil`, `struct`, `subprocess`, `sys`, `time`, `uuid`, `wave`

| 定义 | 位置／签名 |
| --- | --- |
| function digest | [源码:26](../../../tools/check_p0_audio_adaptation.py#L26) · `def digest(path)` |
| function owned | [源码:33](../../../tools/check_p0_audio_adaptation.py#L33) · `def owned(path, exists=True)` |
| function compare | [源码:39](../../../tools/check_p0_audio_adaptation.py#L39) · `def compare(actual, reference, channels)` |
| function pull | [源码:55](../../../tools/check_p0_audio_adaptation.py#L55) · `def pull(source, reference, rate, channels)` |
| function prepare | [源码:75](../../../tools/check_p0_audio_adaptation.py#L75) · `def prepare(target)` |
| function load_fixture | [源码:112](../../../tools/check_p0_audio_adaptation.py#L112) · `def load_fixture(path)` |
| class Reference | [源码:134](../../../tools/check_p0_audio_adaptation.py#L134) |
| function Reference.__init__ | [源码:135](../../../tools/check_p0_audio_adaptation.py#L135) · `def __init__(self, path, rate, channels, frames)` |
| function Reference.check | [源码:138](../../../tools/check_p0_audio_adaptation.py#L138) · `def check(self, p)` |
| function validate | [源码:154](../../../tools/check_p0_audio_adaptation.py#L154) · `def validate(r, trace, reference)` |
| function summarize | [源码:186](../../../tools/check_p0_audio_adaptation.py#L186) · `def summarize(samples)` |
| function run | [源码:195](../../../tools/check_p0_audio_adaptation.py#L195) · `def run(source, row, convert, idle=3, manual=False)` |
| function main | [源码:239](../../../tools/check_p0_audio_adaptation.py#L239) · `def main()` |

## tools/check_p0_combined_soak.py

302 行；SHA256 `aab795ccfde5837ed87725d9d3d114a9a201ec5b73999026552923cb08de157a`

导入：`argparse`, `check_p0`, `check_p0_compressed`, `check_p0_edit_composition`, `check_p0_switch`, `ctypes`, `hashlib`, `json`, `math`, `p0_process_metrics`, `pathlib`, `shutil`, `struct`, `subprocess`, `sys`, `time`, `uuid`, `wave`

| 定义 | 位置／签名 |
| --- | --- |
| function digest | [源码:23](../../../tools/check_p0_combined_soak.py#L23) · `def digest(path)` |
| function make_audio | [源码:30](../../../tools/check_p0_combined_soak.py#L30) · `def make_audio(output)` |
| class Reference | [源码:46](../../../tools/check_p0_combined_soak.py#L46) |
| function Reference.__init__ | [源码:47](../../../tools/check_p0_combined_soak.py#L47) · `def __init__(self, path)` |
| function Reference.check | [源码:64](../../../tools/check_p0_combined_soak.py#L64) · `def check(self, p)` |
| function validate_packets | [源码:86](../../../tools/check_p0_combined_soak.py#L86) · `def validate_packets(path, reference)` |
| function timing | [源码:117](../../../tools/check_p0_combined_soak.py#L117) · `def timing(values)` |
| function validate | [源码:124](../../../tools/check_p0_combined_soak.py#L124) · `def validate(result, packets, smoke)` |
| function summarize_phases | [源码:155](../../../tools/check_p0_combined_soak.py#L155) · `def summarize_phases(samples)` |
| function budgets | [源码:169](../../../tools/check_p0_combined_soak.py#L169) · `def budgets(result, phases, smoke)` |
| function validate_decoder_mode | [源码:183](../../../tools/check_p0_combined_soak.py#L183) · `def validate_decoder_mode(result, compressed)` |
| function run | [源码:188](../../../tools/check_p0_combined_soak.py#L188) · `def run(exe, png, audio, smoke, reference, *, source_name='stream.wav', codec='wav', compressed=False)` |
| function main | [源码:284](../../../tools/check_p0_combined_soak.py#L284) · `def main()` |

## tools/check_p0_composition.py

145 行；SHA256 `277358084e319fb70afe1f406ba3365c500275dcfe6e18e6ea038badbe5a15af`

导入：`argparse`, `check_p0`, `hashlib`, `json`, `os`, `p0_process_metrics`, `pathlib`, `re`, `shutil`, `subprocess`, `sys`, `time`, `uuid`

| 定义 | 位置／签名 |
| --- | --- |
| function validate_scale | [源码:18](../../../tools/check_p0_composition.py#L18) · `def validate_scale(result)` |
| function source_paths | [源码:32](../../../tools/check_p0_composition.py#L32) · `def source_paths()` |
| function run_scale | [源码:38](../../../tools/check_p0_composition.py#L38) · `def run_scale(exe, fixture, repetitions)` |
| function check_cpu_boundary | [源码:100](../../../tools/check_p0_composition.py#L100) · `def check_cpu_boundary(source)` |
| function main | [源码:108](../../../tools/check_p0_composition.py#L108) · `def main()` |

## tools/check_p0_compressed.py

250 行；SHA256 `874a7710c6c6149ae7d4ab4854abe2fd180e7be3ff3352a2eb0e98b9d6218472`

导入：`argparse`, `array`, `check_p0`, `ctypes`, `hashlib`, `json`, `math`, `os`, `p0_compressed_fixture`, `p0_process_metrics`, `pathlib`, `shutil`, `struct`, `subprocess`, `sys`, `time`, `uuid`, `wave`

| 定义 | 位置／签名 |
| --- | --- |
| function compare_float_files | [源码:22](../../../tools/check_p0_compressed.py#L22) · `def compare_float_files(actual, reference, tolerance=1e-05)` |
| function validate_pull | [源码:43](../../../tools/check_p0_compressed.py#L43) · `def validate_pull(result, directory, reference)` |
| class NativeReference | [源码:61](../../../tools/check_p0_compressed.py#L61) |
| function NativeReference.__init__ | [源码:62](../../../tools/check_p0_compressed.py#L62) · `def __init__(self, path, total_frames=8640000)` |
| function NativeReference.check | [源码:70](../../../tools/check_p0_compressed.py#L70) · `def check(self, packet)` |
| function validate_native | [源码:89](../../../tools/check_p0_compressed.py#L89) · `def validate_native(result, reference)` |
| function identity_paths | [源码:132](../../../tools/check_p0_compressed.py#L132) · `def identity_paths()` |
| function run | [源码:139](../../../tools/check_p0_compressed.py#L139) · `def run(exe, source, mode, reference, codec)` |
| function check_compressed | [源码:190](../../../tools/check_p0_compressed.py#L190) · `def check_compressed()` |
| function check_compressed_features | [源码:202](../../../tools/check_p0_compressed.py#L202) · `def check_compressed_features(features)` |
| function main | [源码:207](../../../tools/check_p0_compressed.py#L207) · `def main()` |

## tools/check_p0_compressed_soak.py

173 行；SHA256 `20155196ec6f20d579e412f701ebdbbaa11311b256ca045cb307beca1a3f125b`

导入：`argparse`, `array`, `check_p0`, `check_p0_combined_soak`, `check_p0_compressed`, `check_p0_edit_composition`, `json`, `p0_compressed_fixture`, `pathlib`, `subprocess`, `sys`, `uuid`, `wave`

| 定义 | 位置／签名 |
| --- | --- |
| function decoder_identities | [源码:25](../../../tools/check_p0_compressed_soak.py#L25) · `def decoder_identities()` |
| function artifact_path | [源码:30](../../../tools/check_p0_compressed_soak.py#L30) · `def artifact_path(path)` |
| function prepare_audio | [源码:37](../../../tools/check_p0_compressed_soak.py#L37) · `def prepare_audio(output)` |
| function load_audio | [源码:75](../../../tools/check_p0_compressed_soak.py#L75) · `def load_audio(path, seconds=600)` |
| function result_status | [源码:109](../../../tools/check_p0_compressed_soak.py#L109) · `def result_status(rows, repetitions, smoke)` |
| function main | [源码:124](../../../tools/check_p0_compressed_soak.py#L124) · `def main()` |
| function main.save | [源码:145](../../../tools/check_p0_compressed_soak.py#L145) · `def save()` |

## tools/check_p0_desktop.py

147 行；SHA256 `de2093b6d5f0882b338d3ec41e7c0dad2023018647bef84947fc7907249318b2`

导入：`argparse`, `check_p0`, `check_p0_transport`, `json`, `math`, `os`, `p0_process_metrics`, `pathlib`, `shutil`, `subprocess`, `sys`, `time`, `uuid`

| 定义 | 位置／签名 |
| --- | --- |
| function validate | [源码:27](../../../tools/check_p0_desktop.py#L27) · `def validate(result, source)` |
| function assess | [源码:57](../../../tools/check_p0_desktop.py#L57) · `def assess(result)` |
| function run | [源码:89](../../../tools/check_p0_desktop.py#L89) · `def run(exe, interactive)` |
| function main | [源码:133](../../../tools/check_p0_desktop.py#L133) · `def main()` |

## tools/check_p0_edit_composition.py

124 行；SHA256 `0bdb32f65e6828a0da95572c5cf639e8517c5030e070a189e826c5fc6bbf5936`

导入：`argparse`, `check_p0`, `check_p0_interaction`, `hashlib`, `json`, `math`, `os`, `p0_process_metrics`, `pathlib`, `shutil`, `subprocess`, `sys`, `time`, `uuid`

| 定义 | 位置／签名 |
| --- | --- |
| function validate | [源码:19](../../../tools/check_p0_edit_composition.py#L19) · `def validate(result, readback)` |
| function fixture_check | [源码:39](../../../tools/check_p0_edit_composition.py#L39) · `def fixture_check(fixture)` |
| function digest | [源码:53](../../../tools/check_p0_edit_composition.py#L53) · `def digest(path)` |
| function sources | [源码:57](../../../tools/check_p0_edit_composition.py#L57) · `def sources()` |
| function main | [源码:66](../../../tools/check_p0_edit_composition.py#L66) · `def main()` |

## tools/check_p0_gpu_memory.py

86 行；SHA256 `c15fe78c3412b20dc6e02bc2c4523aa8448a8d48c4fd9759607f19b8428bd1af`

导入：`argparse`, `check_p0`, `check_p0_gui`, `json`, `os`, `pathlib`, `shutil`, `subprocess`, `uuid`

| 定义 | 位置／签名 |
| --- | --- |
| function main | [源码:14](../../../tools/check_p0_gpu_memory.py#L14) · `def main()` |

## tools/check_p0_gui.py

366 行；SHA256 `030fac0ecdc08237297e151f8229302b94ea868701d95fdf8133ca9aa1ad1764`

导入：`argparse`, `check_p0`, `hashlib`, `json`, `os`, `p0_modules`, `p0_process_metrics`, `p0_workload`, `pathlib`, `re`, `shutil`, `subprocess`, `sys`, `time`, `uuid`

| 定义 | 位置／签名 |
| --- | --- |
| function check_native_ui_features | [源码:23](../../../tools/check_p0_gui.py#L23) · `def check_native_ui_features(dependency)` |
| function check_audio_port_boundary | [源码:29](../../../tools/check_p0_gui.py#L29) · `def check_audio_port_boundary(sources)` |
| function check_audio_adapter_boundary | [源码:36](../../../tools/check_p0_gui.py#L36) · `def check_audio_adapter_boundary(dependencies, sources)` |
| function main | [源码:51](../../../tools/check_p0_gui.py#L51) · `def main()` |
| function evaluate_budgets | [源码:170](../../../tools/check_p0_gui.py#L170) · `def evaluate_budgets(result, summary)` |
| function run_standard | [源码:212](../../../tools/check_p0_gui.py#L212) · `def run_standard(environment, fixture=None, flags=(), label='standard', executable=None, observe_modules=False)` |
| function run_comparison | [源码:313](../../../tools/check_p0_gui.py#L313) · `def run_comparison(environment, loading=False)` |
| function run_loader_tuning | [源码:338](../../../tools/check_p0_gui.py#L338) · `def run_loader_tuning(environment)` |

## tools/check_p0_interaction.py

67 行；SHA256 `927504336ebc4b2187d9adc8ccf6320127ab68af9446cb4f7606cde9bb64a02b`

导入：`argparse`, `check_p0`, `check_p0_composition`, `hashlib`, `json`, `math`, `os`, `pathlib`, `shutil`, `subprocess`, `sys`, `uuid`

| 定义 | 位置／签名 |
| --- | --- |
| function validate | [源码:17](../../../tools/check_p0_interaction.py#L17) · `def validate(result, position_tolerance=0)` |
| function main | [源码:32](../../../tools/check_p0_interaction.py#L32) · `def main()` |

## tools/check_p0_lifetime.py

65 行；SHA256 `5d6d08b91ceffcf10cd973df2380fd50f39968e944087ec325c20872d951fa2f`

导入：`argparse`, `check_p0`, `check_p0_gui`, `hashlib`, `json`, `os`, `p0_gpu_repeats`, `pathlib`, `shutil`, `subprocess`, `sys`, `uuid`

| 定义 | 位置／签名 |
| --- | --- |
| function main | [源码:17](../../../tools/check_p0_lifetime.py#L17) · `def main()` |

## tools/check_p0_media.py

214 行；SHA256 `de300b5143a5406c4755efbd6f3358169a7252085514dd2f7248358342410c34`

导入：`argparse`, `check_p0`, `check_p0_transport`, `ctypes`, `hashlib`, `json`, `math`, `os`, `p0_process_metrics`, `pathlib`, `re`, `shutil`, `struct`, `subprocess`, `sys`, `uuid`, `wave`

| 定义 | 位置／签名 |
| --- | --- |
| function digest | [源码:27](../../../tools/check_p0_media.py#L27) · `def digest(path)` |
| function check_boundary | [源码:31](../../../tools/check_p0_media.py#L31) · `def check_boundary(dependencies, code)` |
| function make_fixture | [源码:44](../../../tools/check_p0_media.py#L44) · `def make_fixture(output)` |
| class Reference | [源码:63](../../../tools/check_p0_media.py#L63) |
| function Reference.__init__ | [源码:64](../../../tools/check_p0_media.py#L64) · `def __init__(self, fixture)` |
| function Reference.packet | [源码:71](../../../tools/check_p0_media.py#L71) · `def packet(self, start, frames)` |
| function Reference.check_packet | [源码:86](../../../tools/check_p0_media.py#L86) · `def check_packet(self, p)` |
| function validate | [源码:97](../../../tools/check_p0_media.py#L97) · `def validate(result, fixture)` |
| function check_core | [源码:145](../../../tools/check_p0_media.py#L145) · `def check_core(environment, cargo, linker)` |
| function run_native | [源码:157](../../../tools/check_p0_media.py#L157) · `def run_native(exe, fixture)` |
| function main | [源码:188](../../../tools/check_p0_media.py#L188) · `def main()` |

## tools/check_p0_pcm.py

168 行；SHA256 `c9bfbca3c53fab2c0000ace59bfca47af676836db203d917eccdf8921b632e01`

导入：`argparse`, `check_p0`, `check_p0_edit_composition`, `check_p0_transport`, `json`, `math`, `os`, `p0_process_metrics`, `pathlib`, `shutil`, `subprocess`, `sys`, `time`, `uuid`

| 定义 | 位置／签名 |
| --- | --- |
| function reference_sample | [源码:23](../../../tools/check_p0_pcm.py#L23) · `def reference_sample(frame, rate)` |
| function validate | [源码:32](../../../tools/check_p0_pcm.py#L32) · `def validate(result)` |
| function main | [源码:106](../../../tools/check_p0_pcm.py#L106) · `def main()` |

## tools/check_p0_recovery.py

128 行；SHA256 `28306d06d06b4f7600bb6ef1ecfff118d60edcb136eb107688837e199301d211`

导入：`argparse`, `check_p0`, `check_p0_edit_composition`, `check_p0_transport`, `json`, `os`, `p0_process_metrics`, `pathlib`, `shutil`, `subprocess`, `sys`, `time`, `uuid`

| 定义 | 位置／签名 |
| --- | --- |
| function validate | [源码:21](../../../tools/check_p0_recovery.py#L21) · `def validate(result)` |
| function main | [源码:65](../../../tools/check_p0_recovery.py#L65) · `def main()` |

## tools/check_p0_resource_cycles.py

196 行；SHA256 `49cb900d16f2486ab826ae4d05e1b6e607ee7beaa342b873455830877bf576c1`

导入：`argparse`, `check_p0`, `check_p0_combined_soak`, `check_p0_edit_composition`, `check_p0_switch`, `json`, `p0_process_metrics`, `pathlib`, `shutil`, `statistics`, `subprocess`, `sys`, `time`, `uuid`

| 定义 | 位置／签名 |
| --- | --- |
| function validate | [源码:21](../../../tools/check_p0_resource_cycles.py#L21) · `def validate(result, packets, smoke)` |
| function phase_key | [源码:54](../../../tools/check_p0_resource_cycles.py#L54) · `def phase_key(row)` |
| function summarize | [源码:59](../../../tools/check_p0_resource_cycles.py#L59) · `def summarize(samples)` |
| function summaries | [源码:65](../../../tools/check_p0_resource_cycles.py#L65) · `def summaries(samples, count)` |
| function trends | [源码:85](../../../tools/check_p0_resource_cycles.py#L85) · `def trends(rows)` |
| function budgets | [源码:102](../../../tools/check_p0_resource_cycles.py#L102) · `def budgets(result, summaries_value, peaks, smoke)` |
| function read_phase | [源码:114](../../../tools/check_p0_resource_cycles.py#L114) · `def read_phase(path)` |
| function run | [源码:119](../../../tools/check_p0_resource_cycles.py#L119) · `def run(exe, png, audio, reference, smoke)` |
| function main | [源码:180](../../../tools/check_p0_resource_cycles.py#L180) · `def main()` |

## tools/check_p0_stream_output.py

202 行；SHA256 `0e9eb58b66898d2422a2243deb1afb214565c063d2ca260d4f9b6a65d274d7c9`

导入：`argparse`, `check_p0`, `check_p0_transport`, `ctypes`, `hashlib`, `json`, `math`, `p0_process_metrics`, `pathlib`, `re`, `shutil`, `struct`, `subprocess`, `sys`, `time`, `uuid`, `wave`

| 定义 | 位置／签名 |
| --- | --- |
| function digest | [源码:25](../../../tools/check_p0_stream_output.py#L25) · `def digest(path)` |
| function check_decoder_boundary | [源码:28](../../../tools/check_p0_stream_output.py#L28) · `def check_decoder_boundary(source)` |
| function make_fixture | [源码:34](../../../tools/check_p0_stream_output.py#L34) · `def make_fixture(output)` |
| class Reference | [源码:47](../../../tools/check_p0_stream_output.py#L47) |
| function Reference.__init__ | [源码:48](../../../tools/check_p0_stream_output.py#L48) · `def __init__(self, fixture)` |
| function Reference.packet | [源码:54](../../../tools/check_p0_stream_output.py#L54) · `def packet(self, start, count)` |
| function Reference.check | [源码:68](../../../tools/check_p0_stream_output.py#L68) · `def check(self, p)` |
| function validate | [源码:77](../../../tools/check_p0_stream_output.py#L77) · `def validate(r, reference)` |
| function run_native | [源码:151](../../../tools/check_p0_stream_output.py#L151) · `def run_native(exe, fixture, reference)` |
| function main | [源码:192](../../../tools/check_p0_stream_output.py#L192) · `def main()` |

## tools/check_p0_streaming.py

208 行；SHA256 `6e031e3a2e1de901ca5e7d2a9bddb7e41a659e47fb901755c0f58bcde8320912`

导入：`argparse`, `array`, `check_p0`, `check_p0_media`, `check_p0_transport`, `hashlib`, `json`, `math`, `os`, `p0_process_metrics`, `pathlib`, `shutil`, `struct`, `subprocess`, `sys`, `time`, `uuid`, `wave`

| 定义 | 位置／签名 |
| --- | --- |
| function digest | [源码:23](../../../tools/check_p0_streaming.py#L23) · `def digest(path)` |
| function make_fixture | [源码:30](../../../tools/check_p0_streaming.py#L30) · `def make_fixture(output, durations=(3, 180))` |
| function fnv_pcm | [源码:48](../../../tools/check_p0_streaming.py#L48) · `def fnv_pcm(pcm)` |
| function fnv | [源码:55](../../../tools/check_p0_streaming.py#L55) · `def fnv(data, state=14695981039346656037)` |
| function reference | [源码:60](../../../tools/check_p0_streaming.py#L60) · `def reference(path)` |
| function validate | [源码:76](../../../tools/check_p0_streaming.py#L76) · `def validate(result, expected)` |
| function summarize_samples | [源码:105](../../../tools/check_p0_streaming.py#L105) · `def summarize_samples(samples)` |
| function run_probe | [源码:116](../../../tools/check_p0_streaming.py#L116) · `def run_probe(exe, fixture, name, mode, expected)` |
| function compare | [源码:165](../../../tools/check_p0_streaming.py#L165) · `def compare(rows)` |
| function main | [源码:179](../../../tools/check_p0_streaming.py#L179) · `def main()` |

## tools/check_p0_switch.py

184 行；SHA256 `c7ea7be526ada9ab8f3ce342e5cf420ad133f01d36d7b2574c5d312ac8138ed1`

导入：`argparse`, `check_p0`, `check_p0_edit_composition`, `hashlib`, `json`, `math`, `os`, `p0_frame_attribution`, `p0_process_metrics`, `pathlib`, `shutil`, `subprocess`, `sys`, `time`, `uuid`

| 定义 | 位置／签名 |
| --- | --- |
| function validate | [源码:23](../../../tools/check_p0_switch.py#L23) · `def validate(result, pixels, observe)` |
| function statistics | [源码:53](../../../tools/check_p0_switch.py#L53) · `def statistics(values)` |
| function phase_summary | [源码:61](../../../tools/check_p0_switch.py#L61) · `def phase_summary(samples)` |
| function digest | [源码:79](../../../tools/check_p0_switch.py#L79) · `def digest(path)` |
| function main | [源码:83](../../../tools/check_p0_switch.py#L83) · `def main()` |

## tools/check_p0_switch_audio.py

132 行；SHA256 `4c5dca5d6fd0a7ccbd6040857515b0a0298558ce4f9b0abf652d10ed8de54d89`

导入：`argparse`, `check_p0`, `check_p0_edit_composition`, `check_p0_transport`, `json`, `math`, `os`, `p0_process_metrics`, `pathlib`, `shutil`, `subprocess`, `sys`, `time`, `uuid`

| 定义 | 位置／签名 |
| --- | --- |
| function validate | [源码:25](../../../tools/check_p0_switch_audio.py#L25) · `def validate(result)` |
| function main | [源码:67](../../../tools/check_p0_switch_audio.py#L67) · `def main()` |

## tools/check_p0_transport.py

130 行；SHA256 `2ef7ffba6e87dafac947a38f11f53ece2c93e811814859e133ef551385045fc8`

导入：`argparse`, `check_p0`, `check_p0_composition`, `check_p0_edit_composition`, `hashlib`, `json`, `math`, `os`, `p0_process_metrics`, `pathlib`, `shutil`, `subprocess`, `sys`, `time`, `uuid`

| 定义 | 位置／签名 |
| --- | --- |
| function validate | [源码:25](../../../tools/check_p0_transport.py#L25) · `def validate(result)` |
| function digest | [源码:60](../../../tools/check_p0_transport.py#L60) · `def digest(path)` |
| function sources | [源码:64](../../../tools/check_p0_transport.py#L64) · `def sources()` |
| function main | [源码:74](../../../tools/check_p0_transport.py#L74) · `def main()` |

## tools/check_repository.py

117 行；SHA256 `d939f80eb8b55bc69a3de2521d0492ae7aac17635e9158922eac7f5cf304e62c`

导入：`argparse`, `pathlib`, `re`, `subprocess`, `sys`

| 定义 | 位置／签名 |
| --- | --- |
| function path_issues | [源码:31](../../../tools/check_repository.py#L31) · `def path_issues(relative)` |
| function content_issues | [源码:49](../../../tools/check_repository.py#L49) · `def content_issues(data)` |
| function git | [源码:61](../../../tools/check_repository.py#L61) · `def git(*arguments)` |
| function inspect | [源码:65](../../../tools/check_repository.py#L65) · `def inspect(staged=False)` |
| function main | [源码:97](../../../tools/check_repository.py#L97) · `def main()` |

## tools/core-cost/src/main.rs

339 行；SHA256 `55b9d0d3374ace851740fb8572e6039fa6932f67dc5ac46708fda86f462127c5`

导入：`use skin_editor_application::{values::*, *};`, `use std::io::{self, Write};`, `use std::sync::Arc;`, `use std::time::Instant;`

| 定义 | 位置／签名 |
| --- | --- |
| struct MetadataLease | [源码:8](../../../tools/core-cost/src/main.rs#L8) |
| impl impl BlobLease for MetadataLease | [源码:9](../../../tools/core-cost/src/main.rs#L9) |
| fn reference | [源码:10](../../../tools/core-cost/src/main.rs#L10) |
| struct MetadataStore | [源码:14](../../../tools/core-cost/src/main.rs#L14) |
| impl impl ResourceStore for MetadataStore | [源码:15](../../../tools/core-cost/src/main.rs#L15) |
| fn pin | [源码:16](../../../tools/core-cost/src/main.rs#L16) |
| struct Driver | [源码:20](../../../tools/core-cost/src/main.rs#L20) |
| impl impl Driver | [源码:24](../../../tools/core-cost/src/main.rs#L24) |
| fn submit | [源码:25](../../../tools/core-cost/src/main.rs#L25) |
| fn checkout | [源码:39](../../../tools/core-cost/src/main.rs#L39) |
| fn draft | [源码:53](../../../tools/core-cost/src/main.rs#L53) |
| fn draft_state | [源码:65](../../../tools/core-cost/src/main.rs#L65) |
| fn record | [源码:73](../../../tools/core-cost/src/main.rs#L73) |
| fn edits | [源码:76](../../../tools/core-cost/src/main.rs#L76) |
| fn initialize | [源码:90](../../../tools/core-cost/src/main.rs#L90) |
| fn checkpoint | [源码:187](../../../tools/core-cost/src/main.rs#L187) |
| fn stats | [源码:217](../../../tools/core-cost/src/main.rs#L217) |
| fn main | [源码:224](../../../tools/core-cost/src/main.rs#L224) |

## tools/measure_audio_edit.py

73 行；SHA256 `3a8853fdfec9ebe16757cd23d9524b15d067a88ac0ae187843555c0325b296cd`

导入：`argparse`, `hashlib`, `json`, `measure_audio_load`, `os`, `pathlib`, `platform`

| 定义 | 位置／签名 |
| --- | --- |
| function analyze | [源码:15](../../../tools/measure_audio_edit.py#L15) · `def analyze(report, samples)` |
| function main | [源码:48](../../../tools/measure_audio_edit.py#L48) · `def main()` |

## tools/measure_audio_load.py

181 行；SHA256 `10e59df96fdc055f2747828e645d11d1728cdf9eb3d6b7d286ee10a0adbef0b1`

导入：`argparse`, `ctypes`, `hashlib`, `json`, `math`, `os`, `pathlib`, `platform`, `queue`, `subprocess`, `threading`, `time`

| 定义 | 位置／签名 |
| --- | --- |
| function distribution | [源码:20](../../../tools/measure_audio_load.py#L20) · `def distribution(values)` |
| function summarize | [源码:29](../../../tools/measure_audio_load.py#L29) · `def summarize(samples, phases=PHASES, minimum_spans=None)` |
| function checks | [源码:53](../../../tools/measure_audio_load.py#L53) · `def checks(report, metrics)` |
| class Memory | [源码:67](../../../tools/measure_audio_load.py#L67) · `c.Structure` |
| class WindowsSampler | [源码:72](../../../tools/measure_audio_load.py#L72) |
| function WindowsSampler.__init__ | [源码:73](../../../tools/measure_audio_load.py#L73) · `def __init__(self, child)` |
| function WindowsSampler.sample | [源码:84](../../../tools/measure_audio_load.py#L84) · `def sample(self, phase)` |
| function observe | [源码:95](../../../tools/measure_audio_load.py#L95) · `def observe(binary, directory, argument, timeout_seconds=90, sampler_factory=WindowsSampler)` |
| function observe.read | [源码:103](../../../tools/measure_audio_load.py#L103) · `def read()` |
| function run | [源码:147](../../../tools/measure_audio_load.py#L147) · `def run(binary, directory)` |
| function main | [源码:155](../../../tools/measure_audio_load.py#L155) · `def main()` |

## tools/measure_audio_startup.py

45 行；SHA256 `a116b8131c9b2b4d7479624e26879d311ed2ac027c4968a5c9a7d530f4e8d41e`

导入：`argparse`, `hashlib`, `json`, `measure_audio_load`, `os`, `pathlib`

| 定义 | 位置／签名 |
| --- | --- |
| function analyze | [源码:10](../../../tools/measure_audio_startup.py#L10) · `def analyze(report, samples)` |
| function main | [源码:27](../../../tools/measure_audio_startup.py#L27) · `def main()` |

## tools/measure_automatic_closeout.py

180 行；SHA256 `ea0abdf53cd5395760622e7eab89c0c531c0e5de823c84df98db393e6203d621`

导入：`argparse`, `hashlib`, `json`, `measure_audio_load`, `measure_background_stream`, `measure_combined`, `measure_cpu_threads`, `measure_fault_rebuild`, `measure_gui_candidate`, `measure_main_work`, `measure_stable_window`, `os`, `pathlib`, `subprocess`, `sys`

| 定义 | 位置／签名 |
| --- | --- |
| function process_boundary | [源码:23](../../../tools/measure_automatic_closeout.py#L23) · `def process_boundary(sample, before, after)` |
| class LifecycleSampler | [源码:27](../../../tools/measure_automatic_closeout.py#L27) · `GpuSampler` |
| function LifecycleSampler.__init__ | [源码:28](../../../tools/measure_automatic_closeout.py#L28) · `def __init__(self, child)` |
| function LifecycleSampler.sample | [源码:32](../../../tools/measure_automatic_closeout.py#L32) · `def sample(self, phase)` |
| function gpu_budget | [源码:38](../../../tools/measure_automatic_closeout.py#L38) · `def gpu_budget(samples)` |
| function stable_result | [源码:65](../../../tools/measure_automatic_closeout.py#L65) · `def stable_result(report, samples, resources)` |
| function main | [源码:83](../../../tools/measure_automatic_closeout.py#L83) · `def main()` |
| function main.save | [源码:101](../../../tools/measure_automatic_closeout.py#L101) · `def save()` |
| function main.unchanged | [源码:104](../../../tools/measure_automatic_closeout.py#L104) · `def unchanged()` |
| function main.command | [源码:107](../../../tools/measure_automatic_closeout.py#L107) · `def command(name, command, timeout)` |

## tools/measure_background_stream.py

78 行；SHA256 `4a6fa5407111bf6265e6e9f56a3497d93d5433ca9aaf7d0a90ca06d78cc775b5`

导入：`argparse`, `hashlib`, `json`, `math`, `measure_audio_load`, `measure_combined`, `measure_gui_candidate`, `measure_worker_native`, `os`, `pathlib`, `platform`

| 定义 | 位置／签名 |
| --- | --- |
| function stream_checks | [源码:21](../../../tools/measure_background_stream.py#L21) · `def stream_checks(report, worker)` |
| function audit_checks | [源码:36](../../../tools/measure_background_stream.py#L36) · `def audit_checks(report)` |
| function main | [源码:41](../../../tools/measure_background_stream.py#L41) · `def main()` |

## tools/measure_budget_review.py

158 行；SHA256 `e33f44be4e444fa19793db83e9ce8bafcc29f525d6ed29734a25dab8da612f20`

导入：`argparse`, `copy`, `hashlib`, `json`, `math`, `measure_audio_load`, `measure_cpu_threads`, `measure_independent_cpu`, `measure_main_work`, `measure_repaint`, `os`, `pathlib`, `subprocess`

| 定义 | 位置／签名 |
| --- | --- |
| function review | [源码:35](../../../tools/measure_budget_review.py#L35) · `def review(original, costs, meter_valid, peaks)` |
| function release_receipt | [源码:60](../../../tools/measure_budget_review.py#L60) · `def release_receipt(receipt, binary)` |
| function main | [源码:70](../../../tools/measure_budget_review.py#L70) · `def main()` |

## tools/measure_combined.py

399 行；SHA256 `63f92e8d9971467da98049c8d3b65cceb68ae0b7f885e66290644e53aaf61bb4`

导入：`argparse`, `ctypes`, `hashlib`, `json`, `math`, `measure_audio_load`, `os`, `pathlib`, `platform`, `subprocess`

| 定义 | 位置／签名 |
| --- | --- |
| function source_identity | [源码:15](../../../tools/measure_combined.py#L15) · `def source_identity()` |
| function identity_matches | [源码:23](../../../tools/measure_combined.py#L23) · `def identity_matches(snapshot)` |
| class Value | [源码:27](../../../tools/measure_combined.py#L27) · `c.Union` |
| class Formatted | [源码:31](../../../tools/measure_combined.py#L31) · `c.Structure` |
| class Item | [源码:35](../../../tools/measure_combined.py#L35) · `c.Structure` |
| function pid_total | [源码:39](../../../tools/measure_combined.py#L39) · `def pid_total(rows, pid)` |
| class GpuSampler | [源码:46](../../../tools/measure_combined.py#L46) · `WindowsSampler` |
| function GpuSampler.__init__ | [源码:47](../../../tools/measure_combined.py#L47) · `def __init__(self, child)` |
| function GpuSampler.sample | [源码:71](../../../tools/measure_combined.py#L71) · `def sample(self, phase)` |
| function GpuSampler.close | [源码:89](../../../tools/measure_combined.py#L89) · `def close(self)` |
| function submission_analysis | [源码:95](../../../tools/measure_combined.py#L95) · `def submission_analysis(report)` |
| function causal_analysis | [源码:118](../../../tools/measure_combined.py#L118) · `def causal_analysis(report, *, count=64)` |
| function preparation_analysis | [源码:151](../../../tools/measure_combined.py#L151) · `def preparation_analysis(report)` |
| function allocator_analysis | [源码:175](../../../tools/measure_combined.py#L175) · `def allocator_analysis(snapshots)` |
| function compare_pixel_reports | [源码:212](../../../tools/measure_combined.py#L212) · `def compare_pixel_reports(reports, traces, *, staging=False)` |
| function staging_analysis | [源码:235](../../../tools/measure_combined.py#L235) · `def staging_analysis(report, requested)` |
| function analyze | [源码:249](../../../tools/measure_combined.py#L249) · `def analyze(report, samples, *, memory_usage=False, staging=False)` |
| function main | [源码:308](../../../tools/measure_combined.py#L308) · `def main()` |

## tools/measure_controlled_recovery.py

94 行；SHA256 `ea0cc721a6b20c76c754a4750a58f51360c0d0cb07d5ff961a8bb95a2b7935ca`

导入：`argparse`, `hashlib`, `json`, `measure_audio_load`, `measure_combined`, `measure_supply_recovery`, `os`, `pathlib`, `platform`

| 定义 | 位置／签名 |
| --- | --- |
| function evaluate_controlled | [源码:18](../../../tools/measure_controlled_recovery.py#L18) · `def evaluate_controlled(report, samples, policy, mode)` |
| function main | [源码:58](../../../tools/measure_controlled_recovery.py#L58) · `def main()` |

## tools/measure_cpu_accounting.py

231 行；SHA256 `4c9ceb02a4307d60d8d6a465ba923068fd5b9faa4586be2c86be5cb9e7adf47f`

导入：`argparse`, `ctypes`, `hashlib`, `json`, `measure_audio_load`, `measure_combined`, `measure_gui_cpu`, `os`, `pathlib`, `platform`, `subprocess`, `sys`, `time`

| 定义 | 位置／签名 |
| --- | --- |
| function next_deadline | [源码:25](../../../tools/measure_cpu_accounting.py#L25) · `def next_deadline(previous, now, period=PERIOD_NS)` |
| function reference_cost | [源码:32](../../../tools/measure_cpu_accounting.py#L32) · `def reference_cost(row)` |
| function sampled_tick_model | [源码:49](../../../tools/measure_cpu_accounting.py#L49) · `def sampled_tick_model(span_ns, offset_ns, period_ns=PERIOD_NS, busy_ns=BUSY_NS, quantum_ns=15625000)` |
| function reference_checks | [源码:64](../../../tools/measure_cpu_accounting.py#L64) · `def reference_checks(report, expected_pid, phases)` |
| class NativeReference | [源码:83](../../../tools/measure_cpu_accounting.py#L83) |
| function NativeReference.__init__ | [源码:84](../../../tools/measure_cpu_accounting.py#L84) · `def __init__(self)` |
| function NativeReference.counters | [源码:102](../../../tools/measure_cpu_accounting.py#L102) · `def counters(self)` |
| function NativeReference.wait_until | [源码:112](../../../tools/measure_cpu_accounting.py#L112) · `def wait_until(self, deadline)` |
| function NativeReference.close | [源码:122](../../../tools/measure_cpu_accounting.py#L122) · `def close(self)` |
| function run_phase | [源码:131](../../../tools/measure_cpu_accounting.py#L131) · `def run_phase(api, phase, clock=time.perf_counter_ns)` |
| function output_directory | [源码:170](../../../tools/measure_cpu_accounting.py#L170) · `def output_directory(value)` |
| function child | [源码:178](../../../tools/measure_cpu_accounting.py#L178) · `def child(output, reverse)` |
| function main | [源码:193](../../../tools/measure_cpu_accounting.py#L193) · `def main()` |

## tools/measure_cpu_threads.py

208 行；SHA256 `b3d2933de64599d6e88925df69b7f5ad9c84f163b0efc8bce80954779a021870`

导入：`argparse`, `ctypes`, `hashlib`, `json`, `measure_audio_load`, `measure_background_stream`, `measure_gui_candidate`, `measure_gui_cpu`, `measure_independent_cpu`, `os`, `pathlib`, `platform`, `struct`, `subprocess`, `sys`, `time`

| 定义 | 位置／签名 |
| --- | --- |
| function token_info | [源码:29](../../../tools/measure_cpu_threads.py#L29) · `def token_info(handle=None)` |
| function permission_checks | [源码:49](../../../tools/measure_cpu_threads.py#L49) · `def permission_checks(trace, report, limited)` |
| function pe_entry_rva | [源码:57](../../../tools/measure_cpu_threads.py#L57) · `def pe_entry_rva(path)` |
| class PhaseIdentities | [源码:70](../../../tools/measure_cpu_threads.py#L70) |
| function PhaseIdentities.__init__ | [源码:72](../../../tools/measure_cpu_threads.py#L72) · `def __init__(self)` |
| function PhaseIdentities.sample | [源码:73](../../../tools/measure_cpu_threads.py#L73) · `def sample(self, phase, identify, ordinary, clock)` |
| class IdentitySampler | [源码:82](../../../tools/measure_cpu_threads.py#L82) · `AttributedThreads` |
| function IdentitySampler.__init__ | [源码:83](../../../tools/measure_cpu_threads.py#L83) · `def __init__(self, child)` |
| function IdentitySampler.sample | [源码:85](../../../tools/measure_cpu_threads.py#L85) · `def sample(self, phase)` |
| function IdentitySampler.close | [源码:88](../../../tools/measure_cpu_threads.py#L88) · `def close(self)` |
| function thread_costs | [源码:94](../../../tools/measure_cpu_threads.py#L94) · `def thread_costs(trace, report, phase='voices_32')` |
| function product_child | [源码:126](../../../tools/measure_cpu_threads.py#L126) · `def product_child(output, attributed)` |
| class product_child.Sampler | [源码:128](../../../tools/measure_cpu_threads.py#L128) · `IdentitySampler if attributed else WindowsSampler` |
| function product_child.Sampler.__init__ | [源码:129](../../../tools/measure_cpu_threads.py#L129) · `def __init__(self, child)` |
| function product_child.Sampler.sample | [源码:131](../../../tools/measure_cpu_threads.py#L131) · `def sample(self, phase)` |
| function child | [源码:143](../../../tools/measure_cpu_threads.py#L143) · `def child(output, kind, reverse, attributed)` |
| function collect | [源码:156](../../../tools/measure_cpu_threads.py#L156) · `def collect(collector, output, kind, limited, reverse=False, attributed=True)` |
| function main | [源码:176](../../../tools/measure_cpu_threads.py#L176) · `def main()` |

## tools/measure_cpu_trace.c

360 行；SHA256 `7d12e458132bb54ef370a2cd2c0b6ff7fc77d24a36b9b354ba79b7f937d6e8aa`

| 定义 | 位置／签名 |
| --- | --- |
| function own_process | [源码:43](../../../tools/measure_cpu_trace.c#L43) |
| function own_thread | [源码:49](../../../tools/measure_cpu_trace.c#L49) |
| function thread_event | [源码:54](../../../tools/measure_cpu_trace.c#L54) |
| function property | [源码:74](../../../tools/measure_cpu_trace.c#L74) |
| function switch_schema | [源码:91](../../../tools/measure_cpu_trace.c#L91) |
| function emit_switch | [源码:106](../../../tools/measure_cpu_trace.c#L106) |
| function collect_record | [源码:115](../../../tools/measure_cpu_trace.c#L115) |
| function record | [源码:155](../../../tools/measure_cpu_trace.c#L155) |
| function buffer | [源码:160](../../../tools/measure_cpu_trace.c#L160) |
| function consume | [源码:163](../../../tools/measure_cpu_trace.c#L163) |
| function enable_privilege | [源码:166](../../../tools/measure_cpu_trace.c#L166) |
| function token_summary | [源码:176](../../../tools/measure_cpu_trace.c#L176) |
| function limited_token | [源码:185](../../../tools/measure_cpu_trace.c#L185) |
| function same_user | [源码:208](../../../tools/measure_cpu_trace.c#L208) |
| function created_at | [源码:216](../../../tools/measure_cpu_trace.c#L216) |
| function lifetime_self_test | [源码:221](../../../tools/measure_cpu_trace.c#L221) |
| function wmain | [源码:237](../../../tools/measure_cpu_trace.c#L237) |

## tools/measure_current_combined.py

174 行；SHA256 `280c88fddeb51a89314be4a1a3c1bef6cb534badd8076f6bfc9776d8fc8f0cd6`

导入：`argparse`, `hashlib`, `json`, `math`, `measure_audio_load`, `measure_combined`, `measure_cpu_threads`, `measure_main_work`, `os`, `pathlib`, `platform`

| 定义 | 位置／签名 |
| --- | --- |
| function finite | [源码:20](../../../tools/measure_current_combined.py#L20) · `def finite(values)` |
| function analyze | [源码:24](../../../tools/measure_current_combined.py#L24) · `def analyze(report, samples, product=False)` |
| function main | [源码:130](../../../tools/measure_current_combined.py#L130) · `def main()` |

## tools/measure_event_loop_ui.py

203 行；SHA256 `f5e651348d6329c1d5d0eee5bff1f899e6de98cc5a0232257b49607df6018555`

导入：`argparse`, `collections`, `hashlib`, `json`, `measure_audio_load`, `measure_cpu_threads`, `measure_frame_dispatch`, `measure_gui_base`, `measure_independent_cpu`, `measure_main_work`, `measure_painter_cpu`, `measure_repaint`, `os`, `pathlib`, `subprocess`

| 定义 | 位置／签名 |
| --- | --- |
| function handler_wall | [源码:22](../../../tools/measure_event_loop_ui.py#L22) · `def handler_wall(rows)` |
| function handler_gaps | [源码:42](../../../tools/measure_event_loop_ui.py#L42) · `def handler_gaps(rows, pid, tid, begin, end, frequency, offset, intervals, bound_ns)` |
| function loop_costs | [源码:68](../../../tools/measure_event_loop_ui.py#L68) · `def loop_costs(trace, report, base, raw, intervals)` |
| function delivery_counts | [源码:86](../../../tools/measure_event_loop_ui.py#L86) · `def delivery_counts(values)` |
| function stage_delivery_counts | [源码:102](../../../tools/measure_event_loop_ui.py#L102) · `def stage_delivery_counts(raw, ui_calls, span_seconds)` |
| function main | [源码:129](../../../tools/measure_event_loop_ui.py#L129) · `def main()` |

## tools/measure_fault_rebuild.py

110 行；SHA256 `5a64b5f8827ba7fc9ebf2228e85ce83f7eea8235ad07b21368453989873f6433`

导入：`argparse`, `hashlib`, `json`, `math`, `measure_audio_load`, `measure_combined`, `measure_cpu_threads`, `measure_main_work`, `os`, `pathlib`

| 定义 | 位置／签名 |
| --- | --- |
| function analyze | [源码:14](../../../tools/measure_fault_rebuild.py#L14) · `def analyze(report, samples, graphics, *, memory_usage=None)` |
| function main | [源码:88](../../../tools/measure_fault_rebuild.py#L88) · `def main()` |

## tools/measure_frame_dispatch.py

206 行；SHA256 `a431502401e4095952752dab0f620928f62bcbdf6e08f6c99e2df8fc99b10656`

导入：`argparse`, `collections`, `hashlib`, `json`, `measure_audio_load`, `measure_background_stream`, `measure_cpu_threads`, `measure_gui_candidate`, `measure_independent_cpu`, `measure_main_work`, `measure_repaint`, `measure_ui_wall`, `os`, `pathlib`, `subprocess`

| 定义 | 位置／签名 |
| --- | --- |
| function validate_spans | [源码:24](../../../tools/measure_frame_dispatch.py#L24) · `def validate_spans(data)` |
| function cost_regions | [源码:41](../../../tools/measure_frame_dispatch.py#L41) · `def cost_regions(rows, pid, tid, a, b, frequency, offset, intervals, bound, extended=False, handlers=False)` |
| function cost_regions.qpc | [源码:42](../../../tools/measure_frame_dispatch.py#L42) · `def qpc(epoch)` |
| function span_costs | [源码:63](../../../tools/measure_frame_dispatch.py#L63) · `def span_costs(trace, report, base, raw, intervals)` |
| function painter_partition | [源码:73](../../../tools/measure_frame_dispatch.py#L73) · `def painter_partition(rows)` |
| function run_without_uac | [源码:97](../../../tools/measure_frame_dispatch.py#L97) · `def run_without_uac(output, feature, initial, feature_hash)` |
| function layout_passes | [源码:124](../../../tools/measure_frame_dispatch.py#L124) · `def layout_passes(raw, ui_calls)` |
| function main | [源码:152](../../../tools/measure_frame_dispatch.py#L152) · `def main()` |

## tools/measure_framework_baseline.py

132 行；SHA256 `5a7113c08d08a46604db13ae944f111a5ff291a654c5ce7d128941235657f2de`

导入：`argparse`, `hashlib`, `json`, `math`, `measure_audio_load`, `measure_cpu_threads`, `measure_independent_cpu`, `measure_main_work`, `measure_repaint`, `os`, `subprocess`

| 定义 | 位置／签名 |
| --- | --- |
| function control_checks | [源码:23](../../../tools/measure_framework_baseline.py#L23) · `def control_checks(report, role)` |
| function comparison_checks | [源码:54](../../../tools/measure_framework_baseline.py#L54) · `def comparison_checks(values, reports)` |
| function editor_passed | [源码:73](../../../tools/measure_framework_baseline.py#L73) · `def editor_passed(values)` |
| function main | [源码:82](../../../tools/measure_framework_baseline.py#L82) · `def main()` |

## tools/measure_gpu_path.py

105 行；SHA256 `d689efc91d146c91f450d1ba679a096c5d7292576d005c8d5354631ab4b125bf`

导入：`argparse`, `json`, `measure_audio_load`, `measure_background_stream`, `measure_cpu_threads`, `measure_gui_candidate`, `measure_independent_cpu`, `measure_main_work`, `measure_ui_wall`, `os`

| 定义 | 位置／签名 |
| --- | --- |
| function integer_row | [源码:22](../../../tools/measure_gpu_path.py#L22) · `def integer_row(row, length)` |
| function path | [源码:26](../../../tools/measure_gpu_path.py#L26) · `def path(data, shutdown, mode, minimum_frames=97)` |
| function main | [源码:75](../../../tools/measure_gpu_path.py#L75) · `def main()` |

## tools/measure_gui_base.py

104 行；SHA256 `f591b5a79d53eb2fc08cadfec6fd4e30a01a5179da74b39f2a207f018954cc9d`

导入：`argparse`, `hashlib`, `json`, `math`, `measure_audio_load`, `measure_cpu_threads`, `measure_independent_cpu`, `measure_main_work`, `measure_repaint`, `os`, `pathlib`, `subprocess`

| 定义 | 位置／签名 |
| --- | --- |
| function digest | [源码:18](../../../tools/measure_gui_base.py#L18) · `def digest(path)` |
| function candidate_passed | [源码:19](../../../tools/measure_gui_base.py#L19) · `def candidate_passed(rows)` |
| function comparison_checks | [源码:24](../../../tools/measure_gui_base.py#L24) · `def comparison_checks(values, reports)` |
| function main | [源码:51](../../../tools/measure_gui_base.py#L51) · `def main()` |

## tools/measure_gui_candidate.py

100 行；SHA256 `995f3f66a523c3e44cf74c0798bf148b168d23cbadb678c1ae829ad4e44a5899`

导入：`argparse`, `hashlib`, `json`, `math`, `measure_audio_load`, `measure_combined`, `measure_worker_native`, `os`, `pathlib`, `platform`

| 定义 | 位置／签名 |
| --- | --- |
| function evaluate_gui | [源码:17](../../../tools/measure_gui_candidate.py#L17) · `def evaluate_gui(report, metrics)` |
| function main | [源码:66](../../../tools/measure_gui_candidate.py#L66) · `def main()` |

## tools/measure_gui_cpu.py

257 行；SHA256 `24c24d74f18b888d5d02248ad1b0d6df524fbd28475bd558a13ad4cbe41fc682`

导入：`argparse`, `contextlib`, `ctypes`, `json`, `measure_audio_load`, `measure_gui_candidate`, `measure_thread_sampling`, `measure_voice_cpu`, `os`, `platform`, `re`, `time`

| 定义 | 位置／签名 |
| --- | --- |
| class ModuleEntry | [源码:21](../../../tools/measure_gui_cpu.py#L21) · `c.Structure` |
| function origin | [源码:27](../../../tools/measure_gui_cpu.py#L27) · `def origin(address, modules, status)` |
| function configure_cycles | [源码:36](../../../tools/measure_gui_cpu.py#L36) · `def configure_cycles(kernel)` |
| function read_cycles | [源码:42](../../../tools/measure_gui_cpu.py#L42) · `def read_cycles(function, handle)` |
| class CycleProcess | [源码:49](../../../tools/measure_gui_cpu.py#L49) · `WindowsSampler` |
| function CycleProcess.__init__ | [源码:50](../../../tools/measure_gui_cpu.py#L50) · `def __init__(self, child)` |
| function CycleProcess.sample | [源码:54](../../../tools/measure_gui_cpu.py#L54) · `def sample(self, phase)` |
| class AttributedThreads | [源码:61](../../../tools/measure_gui_cpu.py#L61) · `CachedThreads` |
| function AttributedThreads.__init__ | [源码:62](../../../tools/measure_gui_cpu.py#L62) · `def __init__(self, child)` |
| function AttributedThreads.module_snapshot | [源码:78](../../../tools/measure_gui_cpu.py#L78) · `def module_snapshot(self)` |
| function AttributedThreads.open_thread | [源码:100](../../../tools/measure_gui_cpu.py#L100) · `def open_thread(self, tid)` |
| function AttributedThreads.times | [源码:105](../../../tools/measure_gui_cpu.py#L105) · `def times(self, handle)` |
| function AttributedThreads.description | [源码:114](../../../tools/measure_gui_cpu.py#L114) · `def description(self, handle)` |
| function AttributedThreads.close_thread | [源码:129](../../../tools/measure_gui_cpu.py#L129) · `def close_thread(self, handle)` |
| function AttributedThreads.sample | [源码:134](../../../tools/measure_gui_cpu.py#L134) · `def sample(self, phase)` |
| class AttributionAudit | [源码:149](../../../tools/measure_gui_cpu.py#L149) · `ObserverAudit` |
| function AttributionAudit.__init__ | [源码:150](../../../tools/measure_gui_cpu.py#L150) · `def __init__(self, child, mode)` |
| function cycle_cost | [源码:156](../../../tools/measure_gui_cpu.py#L156) · `def cycle_cost(samples, phase='voices_32')` |
| function gui_cost | [源码:178](../../../tools/measure_gui_cpu.py#L178) · `def gui_cost(report)` |
| function adapter_info | [源码:194](../../../tools/measure_gui_cpu.py#L194) · `def adapter_info(report)` |
| function backend_override | [源码:205](../../../tools/measure_gui_cpu.py#L205) · `def backend_override(value)` |
| function main | [源码:214](../../../tools/measure_gui_cpu.py#L214) · `def main()` |
| function main.factory | [源码:226](../../../tools/measure_gui_cpu.py#L226) · `def factory(child)` |

## tools/measure_independent_cpu.py

259 行；SHA256 `3e8da64bda3c9cdd2a2d78674b227f186430b4e97318ca34348c98db85dfec42`

导入：`argparse`, `check_core`, `ctypes`, `hashlib`, `json`, `measure_audio_load`, `measure_background_stream`, `measure_combined`, `measure_cpu_accounting`, `measure_gui_candidate`, `os`, `pathlib`, `platform`, `subprocess`, `sys`, `time`

| 定义 | 位置／签名 |
| --- | --- |
| function stamp_api | [源码:27](../../../tools/measure_independent_cpu.py#L27) · `def stamp_api()` |
| function stamp_api.read | [源码:31](../../../tools/measure_independent_cpu.py#L31) · `def read(name='QueryPerformanceCounter')` |
| function scheduler_intervals | [源码:38](../../../tools/measure_independent_cpu.py#L38) · `def scheduler_intervals(trace)` |
| function overlap_diagnostic | [源码:82](../../../tools/measure_independent_cpu.py#L82) · `def overlap_diagnostic(intervals, frequency)` |
| function scheduled_cost | [源码:96](../../../tools/measure_independent_cpu.py#L96) · `def scheduled_cost(intervals, pid, begin, end, frequency)` |
| function calibration | [源码:105](../../../tools/measure_independent_cpu.py#L105) · `def calibration(trace, report)` |
| function product_cost | [源码:132](../../../tools/measure_independent_cpu.py#L132) · `def product_cost(trace, report)` |
| function fingerprint | [源码:150](../../../tools/measure_independent_cpu.py#L150) · `def fingerprint()` |
| function build_collector | [源码:155](../../../tools/measure_independent_cpu.py#L155) · `def build_collector(output)` |
| function reference_child | [源码:177](../../../tools/measure_independent_cpu.py#L177) · `def reference_child(output, reverse)` |
| function product_child | [源码:191](../../../tools/measure_independent_cpu.py#L191) · `def product_child(output)` |
| class product_child.Sampler | [源码:193](../../../tools/measure_independent_cpu.py#L193) · `WindowsSampler` |
| function product_child.Sampler.__init__ | [源码:194](../../../tools/measure_independent_cpu.py#L194) · `def __init__(self, child)` |
| function product_child.Sampler.sample | [源码:195](../../../tools/measure_independent_cpu.py#L195) · `def sample(self, phase)` |
| function collect | [源码:211](../../../tools/measure_independent_cpu.py#L211) · `def collect(collector, output, mode, reverse=False)` |
| function main | [源码:223](../../../tools/measure_independent_cpu.py#L223) · `def main()` |

## tools/measure_main_work.py

359 行；SHA256 `9997475e60d6c883a9c7e3e69e48fb903c9f2ace3f242c8b80e91054a4f88107`

导入：`argparse`, `ctypes`, `hashlib`, `json`, `measure_audio_load`, `measure_background_stream`, `measure_cpu_accounting`, `measure_cpu_threads`, `measure_gui_candidate`, `measure_independent_cpu`, `measure_stable_window`, `os`, `pathlib`, `subprocess`, `sys`, `time`, `uuid`

| 定义 | 位置／签名 |
| --- | --- |
| function categories_for_schema | [源码:36](../../../tools/measure_main_work.py#L36) · `def categories_for_schema(schema)` |
| function row_limit | [源码:40](../../../tools/measure_main_work.py#L40) · `def row_limit(schema)` |
| function ordinary | [源码:45](../../../tools/measure_main_work.py#L45) · `def ordinary(token)` |
| function reference_workload_valid | [源码:49](../../../tools/measure_main_work.py#L49) · `def reference_workload_valid(report)` |
| function evaluate_reports | [源码:57](../../../tools/measure_main_work.py#L57) · `def evaluate_reports(trace, references, products)` |
| function permission_checks | [源码:75](../../../tools/measure_main_work.py#L75) · `def permission_checks(trace, runner, reports, created)` |
| function precise_pair | [源码:85](../../../tools/measure_main_work.py#L85) · `def precise_pair(stamp, kernel)` |
| function clock_mapping | [源码:90](../../../tools/measure_main_work.py#L90) · `def clock_mapping(pairs, frequency)` |
| function main_costs | [源码:111](../../../tools/measure_main_work.py#L111) · `def main_costs(trace, report, base)` |
| function product_argument | [源码:155](../../../tools/measure_main_work.py#L155) · `def product_argument(profile, ui_mode=None)` |
| function product | [源码:167](../../../tools/measure_main_work.py#L167) · `def product(output, profile, binary=BINARY, ui_mode=None)` |
| class product.Sampler | [源码:178](../../../tools/measure_main_work.py#L178) · `IdentitySampler` |
| function product.Sampler.__init__ | [源码:179](../../../tools/measure_main_work.py#L179) · `def __init__(self, child)` |
| function product.Sampler.sample | [源码:181](../../../tools/measure_main_work.py#L181) · `def sample(self, phase)` |
| class Collector | [源码:192](../../../tools/measure_main_work.py#L192) |
| function Collector.__init__ | [源码:194](../../../tools/measure_main_work.py#L194) · `def __init__(self, binary, output)` |
| class Collector.__init__.Execute | [源码:214](../../../tools/measure_main_work.py#L214) · `c.Structure` |
| function Collector.close | [源码:230](../../../tools/measure_main_work.py#L230) · `def close(self)` |
| function run_child | [源码:242](../../../tools/measure_main_work.py#L242) · `def run_child(output, kind, profile=False, reverse=False, binary=None, ui_mode=None)` |
| class run_child.Basic | [源码:261](../../../tools/measure_main_work.py#L261) · `c.Structure` |
| class run_child.Limits | [源码:264](../../../tools/measure_main_work.py#L264) · `c.Structure` |
| function launch_owned | [源码:282](../../../tools/measure_main_work.py#L282) · `def launch_owned(k, gate, job, args, output, timeout=55)` |
| function main | [源码:303](../../../tools/measure_main_work.py#L303) · `def main()` |

## tools/measure_mesh_replay.py

96 行；SHA256 `ea5efa814e2841b7fb4bc27b7e91a315140133cd2600f4cf31f3382600f7068d`

导入：`argparse`, `json`, `measure_audio_load`, `measure_background_stream`, `measure_cpu_threads`, `measure_gui_candidate`, `measure_independent_cpu`, `measure_main_work`, `measure_ui_wall`, `os`, `struct`

| 定义 | 位置／签名 |
| --- | --- |
| function replay | [源码:23](../../../tools/measure_mesh_replay.py#L23) · `def replay(data, shutdown)` |
| function main | [源码:66](../../../tools/measure_mesh_replay.py#L66) · `def main()` |

## tools/measure_painter_cpu.py

134 行；SHA256 `acebe7480df8d3b2454d98f2ce9808378d6c10c1d8d1def41b8fe9df08f5b62f`

导入：`argparse`, `hashlib`, `json`, `measure_audio_load`, `measure_frame_dispatch`, `measure_independent_cpu`, `measure_main_work`, `measure_repaint`, `pathlib`

| 定义 | 位置／签名 |
| --- | --- |
| function exclusive_cpu | [源码:16](../../../tools/measure_painter_cpu.py#L16) · `def exclusive_cpu(rows, pid, tid, begin, end, frequency, offset, intervals, bound_ns)` |
| function exclusive_cpu.qpc | [源码:25](../../../tools/measure_painter_cpu.py#L25) · `def qpc(epoch)` |
| function analyze | [源码:84](../../../tools/measure_painter_cpu.py#L84) · `def analyze(capture)` |
| function main | [源码:122](../../../tools/measure_painter_cpu.py#L122) · `def main()` |

## tools/measure_present_fault.py

45 行；SHA256 `3c51c7744bed387b0c4dc581c61d609a3867375c9832b2cfb040341630aac140`

导入：`argparse`, `hashlib`, `json`, `measure_audio_load`, `measure_combined`, `pathlib`, `subprocess`, `sys`

| 定义 | 位置／签名 |
| --- | --- |
| function main | [源码:17](../../../tools/measure_present_fault.py#L17) · `def main()` |

## tools/measure_rebuild_attribution.py

222 行；SHA256 `11e7481b5680e0cb2505cf8f06ed407594ff858341b9ac3d6f0fc79dda1cf5db`

导入：`argparse`, `ctypes`, `hashlib`, `json`, `measure_audio_load`, `measure_combined`, `measure_cpu_accounting`, `measure_cpu_threads`, `measure_main_work`, `os`, `pathlib`, `queue`, `subprocess`, `threading`, `time`

| 定义 | 位置／签名 |
| --- | --- |
| function validate_checkpoint | [源码:31](../../../tools/measure_rebuild_attribution.py#L31) · `def validate_checkpoint(message, count)` |
| function region_summary | [源码:45](../../../tools/measure_rebuild_attribution.py#L45) · `def region_summary(rows)` |
| function region_delta | [源码:53](../../../tools/measure_rebuild_attribution.py#L53) · `def region_delta(before, after)` |
| function region_delta.identity | [源码:54](../../../tools/measure_rebuild_attribution.py#L54) · `def identity(r)` |
| function timeline_interval | [源码:61](../../../tools/measure_rebuild_attribution.py#L61) · `def timeline_interval(rows, phase, read, clock)` |
| class MBI | [源码:70](../../../tools/measure_rebuild_attribution.py#L70) · `c.Structure` |
| class Regions | [源码:75](../../../tools/measure_rebuild_attribution.py#L75) · `GpuSampler` |
| function Regions.__init__ | [源码:76](../../../tools/measure_rebuild_attribution.py#L76) · `def __init__(self, child)` |
| function Regions.snapshot | [源码:85](../../../tools/measure_rebuild_attribution.py#L85) · `def snapshot(self)` |
| function run | [源码:113](../../../tools/measure_rebuild_attribution.py#L113) · `def run(binary, directory, mode, timeline=False, memory_usage=False)` |
| function run.read | [源码:126](../../../tools/measure_rebuild_attribution.py#L126) · `def read()` |
| function run.set_stage | [源码:134](../../../tools/measure_rebuild_attribution.py#L134) · `def set_stage(value)` |
| function run.get_stage | [源码:136](../../../tools/measure_rebuild_attribution.py#L136) · `def get_stage()` |
| function run.collect | [源码:143](../../../tools/measure_rebuild_attribution.py#L143) · `def collect()` |
| function main | [源码:196](../../../tools/measure_rebuild_attribution.py#L196) · `def main()` |

## tools/measure_rebuild_policy.py

120 行；SHA256 `3d40a5b6ecf61763d5718c34626f16f11a002784385ff606c9e7fc1794701e3c`

导入：`argparse`, `hashlib`, `json`, `measure_audio_load`, `measure_combined`, `measure_cpu_threads`, `measure_fault_rebuild`, `measure_main_work`, `measure_rebuild_attribution`, `os`, `pathlib`, `subprocess`

| 定义 | 位置／签名 |
| --- | --- |
| function pixels | [源码:21](../../../tools/measure_rebuild_policy.py#L21) · `def pixels(directory)` |
| function pool_analysis | [源码:27](../../../tools/measure_rebuild_policy.py#L27) · `def pool_analysis(run, memory_usage)` |
| function compare_pools | [源码:57](../../../tools/measure_rebuild_policy.py#L57) · `def compare_pools(runs)` |
| function main | [源码:69](../../../tools/measure_rebuild_policy.py#L69) · `def main()` |

## tools/measure_repaint.py

145 行；SHA256 `19b4cc56192f706893a7ab6b5283a9ed3344b9e6623cbf9f7846e6f15022144b`

导入：`argparse`, `collections`, `hashlib`, `json`, `measure_audio_load`, `measure_cpu_threads`, `measure_independent_cpu`, `measure_main_work`, `os`, `pathlib`, `platform`, `subprocess`

| 定义 | 位置／签名 |
| --- | --- |
| function permission | [源码:21](../../../tools/measure_repaint.py#L21) · `def permission(trace, runner, created, reports, expected)` |
| function require_references | [源码:31](../../../tools/measure_repaint.py#L31) · `def require_references(references)` |
| function repaint_costs | [源码:36](../../../tools/measure_repaint.py#L36) · `def repaint_costs(trace, report, base)` |
| function repaint_costs.inside | [源码:46](../../../tools/measure_repaint.py#L46) · `def inside(epoch)` |
| function close_session | [源码:80](../../../tools/measure_repaint.py#L80) · `def close_session(collector, path, runner, reports, expected)` |
| function main | [源码:88](../../../tools/measure_repaint.py#L88) · `def main()` |

## tools/measure_resource_cycles.py

206 行；SHA256 `261a69b4a16b83b8106560f02ff6e63d8f5114b54eaa1549755cee04cb06e458`

导入：`argparse`, `hashlib`, `json`, `math`, `measure_audio_load`, `measure_combined`, `measure_cpu_threads`, `measure_main_work`, `os`, `pathlib`, `re`, `statistics`

| 定义 | 位置／签名 |
| --- | --- |
| function read_phase | [源码:20](../../../tools/measure_resource_cycles.py#L20) · `def read_phase(path)` |
| function label_sample | [源码:32](../../../tools/measure_resource_cycles.py#L32) · `def label_sample(sample, before, after)` |
| class CycleSampler | [源码:39](../../../tools/measure_resource_cycles.py#L39) · `GpuSampler` |
| function CycleSampler.__init__ | [源码:40](../../../tools/measure_resource_cycles.py#L40) · `def __init__(self, child, directory)` |
| function CycleSampler.sample | [源码:42](../../../tools/measure_resource_cycles.py#L42) · `def sample(self, phase)` |
| function numeric | [源码:48](../../../tools/measure_resource_cycles.py#L48) · `def numeric(values)` |
| function cycle_checks | [源码:52](../../../tools/measure_resource_cycles.py#L52) · `def cycle_checks(r, index, smoke, rate)` |
| function slope | [源码:98](../../../tools/measure_resource_cycles.py#L98) · `def slope(values)` |
| function trend | [源码:104](../../../tools/measure_resource_cycles.py#L104) · `def trend(values)` |
| function analyze | [源码:115](../../../tools/measure_resource_cycles.py#L115) · `def analyze(report, cycles, samples, smoke=False, product=False)` |
| function main | [源码:184](../../../tools/measure_resource_cycles.py#L184) · `def main()` |

## tools/measure_scheduling.py

93 行；SHA256 `e864943979989328d7e673ad82e18963238198bad44203a18463b7ca28c8996b`

导入：

| 定义 | 位置／签名 |
| --- | --- |
| function parts | [源码:4](../../../tools/measure_scheduling.py#L4) · `def parts(r)` |
| function analyze_timing | [源码:12](../../../tools/measure_scheduling.py#L12) · `def analyze_timing(s)` |
| function analyze_timing.valid | [源码:18](../../../tools/measure_scheduling.py#L18) · `def valid(r, limit)` |
| function analyze_host | [源码:71](../../../tools/measure_scheduling.py#L71) · `def analyze_host(h)` |

## tools/measure_separated_native.py

85 行；SHA256 `11fbe962e8ec747f66d300ce86279dd019e6269005c4d8a39abbece10dcc611c`

导入：`argparse`, `hashlib`, `json`, `math`, `measure_audio_load`, `measure_combined`, `os`, `pathlib`, `platform`

| 定义 | 位置／签名 |
| --- | --- |
| function evaluate | [源码:13](../../../tools/measure_separated_native.py#L13) · `def evaluate(report, metrics)` |
| function main | [源码:48](../../../tools/measure_separated_native.py#L48) · `def main()` |

## tools/measure_stable_window.py

179 行；SHA256 `6bed1717e8999d0959978cccfd27d03d96a6aecc07304fc25e2df28d71cd9c72`

导入：`argparse`, `hashlib`, `json`, `math`, `measure_audio_load`, `measure_combined`, `measure_cpu_threads`, `measure_independent_cpu`, `measure_main_work`, `measure_repaint`, `os`, `pathlib`, `subprocess`

| 定义 | 位置／签名 |
| --- | --- |
| function phases_for | [源码:17](../../../tools/measure_stable_window.py#L17) · `def phases_for(mode)` |
| function launch_mode | [源码:23](../../../tools/measure_stable_window.py#L23) · `def launch_mode(mode, product)` |
| function report_checks | [源码:29](../../../tools/measure_stable_window.py#L29) · `def report_checks(report, mode, product=None)` |
| function observe_product | [源码:70](../../../tools/measure_stable_window.py#L70) · `def observe_product(output, binary, mode, product=False)` |
| class observe_product.Sampler | [源码:73](../../../tools/measure_stable_window.py#L73) · `GpuSampler` |
| function observe_product.Sampler.__init__ | [源码:74](../../../tools/measure_stable_window.py#L74) · `def __init__(self, child)` |
| function observe_product.Sampler.sample | [源码:78](../../../tools/measure_stable_window.py#L78) · `def sample(self, phase)` |
| function analyze | [源码:91](../../../tools/measure_stable_window.py#L91) · `def analyze(trace, record, calibrated, product=None)` |
| function main | [源码:123](../../../tools/measure_stable_window.py#L123) · `def main()` |

## tools/measure_staging_recovery.py

149 行；SHA256 `9b22cf72dacf4e59542c51e6ceeb2c1b943ef8ea07e6e7579d85b71b6a248155`

导入：`argparse`, `hashlib`, `json`, `math`, `measure_audio_load`, `measure_combined`, `os`, `pathlib`, `statistics`

| 定义 | 位置／签名 |
| --- | --- |
| class RecoverySampler | [源码:14](../../../tools/measure_staging_recovery.py#L14) · `GpuSampler` |
| function RecoverySampler.sample | [源码:15](../../../tools/measure_staging_recovery.py#L15) · `def sample(self, phase)` |
| function trend | [源码:24](../../../tools/measure_staging_recovery.py#L24) · `def trend(values)` |
| function gpu_summary | [源码:36](../../../tools/measure_staging_recovery.py#L36) · `def gpu_summary(samples)` |
| function analyze_recovery | [源码:45](../../../tools/measure_staging_recovery.py#L45) · `def analyze_recovery(report, cycles, samples)` |
| function main | [源码:119](../../../tools/measure_staging_recovery.py#L119) · `def main()` |

## tools/measure_supply_recovery.py

82 行；SHA256 `afebd6c334a562991c101db6dc62bd837342efd0a83f356161e5bde985b7b0c0`

导入：`argparse`, `hashlib`, `json`, `measure_audio_load`, `measure_combined`, `measure_scheduling`, `measure_voice_phase`, `os`, `pathlib`, `platform`

| 定义 | 位置／签名 |
| --- | --- |
| function evaluate | [源码:16](../../../tools/measure_supply_recovery.py#L16) · `def evaluate(report, samples, expected)` |
| function main | [源码:42](../../../tools/measure_supply_recovery.py#L42) · `def main()` |

## tools/measure_text_work.py

82 行；SHA256 `aeee1730317f5311c4328e7f99294fe1aa8c824a0281198e0b11bbf86eb132f5`

导入：`argparse`, `json`, `measure_audio_load`, `measure_background_stream`, `measure_cpu_threads`, `measure_gui_candidate`, `measure_independent_cpu`, `measure_main_work`, `measure_ui_wall`, `os`, `pathlib`

| 定义 | 位置／签名 |
| --- | --- |
| function census | [源码:18](../../../tools/measure_text_work.py#L18) · `def census(data, shutdown)` |
| function census.summary | [源码:34](../../../tools/measure_text_work.py#L34) · `def summary(selected)` |
| function main | [源码:48](../../../tools/measure_text_work.py#L48) · `def main()` |

## tools/measure_thread_sampling.py

230 行；SHA256 `47fe7460838e72c92330f6be767152590f8588a46f259de2a95acd1f8175a701`

导入：`argparse`, `hashlib`, `json`, `measure_audio_load`, `measure_background_stream`, `measure_combined`, `measure_gui_candidate`, `measure_voice_cpu`, `os`, `pathlib`, `platform`, `time`

| 定义 | 位置／签名 |
| --- | --- |
| class OwnedThreadSet | [源码:20](../../../tools/measure_thread_sampling.py#L20) |
| function OwnedThreadSet.__init__ | [源码:22](../../../tools/measure_thread_sampling.py#L22) · `def __init__(self, api, limit=LIMIT)` |
| function OwnedThreadSet.refresh | [源码:32](../../../tools/measure_thread_sampling.py#L32) · `def refresh(self)` |
| function OwnedThreadSet.rows | [源码:54](../../../tools/measure_thread_sampling.py#L54) · `def rows(self, phase)` |
| function OwnedThreadSet.close | [源码:69](../../../tools/measure_thread_sampling.py#L69) · `def close(self)` |
| class AuditedThreads | [源码:82](../../../tools/measure_thread_sampling.py#L82) · `ThreadSampler` |
| function AuditedThreads.__init__ | [源码:83](../../../tools/measure_thread_sampling.py#L83) · `def __init__(self, child)` |
| function AuditedThreads.live_ids | [源码:90](../../../tools/measure_thread_sampling.py#L90) · `def live_ids(self)` |
| function AuditedThreads.thread_rows | [源码:99](../../../tools/measure_thread_sampling.py#L99) · `def thread_rows(self, live)` |
| class CachedThreads | [源码:105](../../../tools/measure_thread_sampling.py#L105) · `AuditedThreads` |
| function CachedThreads.__init__ | [源码:106](../../../tools/measure_thread_sampling.py#L106) · `def __init__(self, child)` |
| function CachedThreads.sample | [源码:110](../../../tools/measure_thread_sampling.py#L110) · `def sample(self, phase)` |
| function CachedThreads.close | [源码:120](../../../tools/measure_thread_sampling.py#L120) · `def close(self)` |
| class ObserverAudit | [源码:124](../../../tools/measure_thread_sampling.py#L124) |
| function ObserverAudit.__init__ | [源码:125](../../../tools/measure_thread_sampling.py#L125) · `def __init__(self, child, mode)` |
| function ObserverAudit.sample | [源码:131](../../../tools/measure_thread_sampling.py#L131) · `def sample(self, phase)` |
| function ObserverAudit.close | [源码:144](../../../tools/measure_thread_sampling.py#L144) · `def close(self)` |
| function observer_cost | [源码:151](../../../tools/measure_thread_sampling.py#L151) · `def observer_cost(samples, phase)` |
| function identity | [源码:174](../../../tools/measure_thread_sampling.py#L174) · `def identity(binary)` |
| function additional_checks | [源码:178](../../../tools/measure_thread_sampling.py#L178) · `def additional_checks(report, samples, close_record)` |
| function main | [源码:187](../../../tools/measure_thread_sampling.py#L187) · `def main()` |
| function main.factory | [源码:201](../../../tools/measure_thread_sampling.py#L201) · `def factory(child)` |

## tools/measure_ui_wall.py

114 行；SHA256 `fbde4286b833dc4991a150b6556d8a12629639622cccd5545a5a7ca33efeacf0`

导入：`argparse`, `hashlib`, `json`, `measure_audio_load`, `measure_background_stream`, `measure_cpu_threads`, `measure_gui_candidate`, `measure_independent_cpu`, `measure_main_work`, `os`, `pathlib`

| 定义 | 位置／签名 |
| --- | --- |
| function widget_wall | [源码:20](../../../tools/measure_ui_wall.py#L20) · `def widget_wall(work, minimum_calls=1)` |
| function diagnostic_checks | [源码:67](../../../tools/measure_ui_wall.py#L67) · `def diagnostic_checks(checks)` |
| function main | [源码:73](../../../tools/measure_ui_wall.py#L73) · `def main()` |

## tools/measure_voice_cpu.py

174 行；SHA256 `6c9b974cac8009d6f3e205f186ef5feecafc0e6febca616d7201e1273475c632`

导入：`argparse`, `ctypes`, `hashlib`, `json`, `measure_audio_load`, `measure_combined`, `measure_gui_candidate`, `os`, `pathlib`

| 定义 | 位置／签名 |
| --- | --- |
| function work_cost | [源码:16](../../../tools/measure_voice_cpu.py#L16) · `def work_cost(report)` |
| class ThreadEntry | [源码:33](../../../tools/measure_voice_cpu.py#L33) · `c.Structure` |
| function group | [源码:37](../../../tools/measure_voice_cpu.py#L37) · `def group(name, main)` |
| function thread_cost | [源码:44](../../../tools/measure_voice_cpu.py#L44) · `def thread_cost(samples, phase='voices_32')` |
| class ThreadSampler | [源码:68](../../../tools/measure_voice_cpu.py#L68) · `WindowsSampler` |
| function ThreadSampler.__init__ | [源码:69](../../../tools/measure_voice_cpu.py#L69) · `def __init__(self, child)` |
| function ThreadSampler.times | [源码:85](../../../tools/measure_voice_cpu.py#L85) · `def times(self, handle)` |
| function ThreadSampler.live_ids | [源码:92](../../../tools/measure_voice_cpu.py#L92) · `def live_ids(self)` |
| function ThreadSampler.open_thread | [源码:107](../../../tools/measure_voice_cpu.py#L107) · `def open_thread(self, tid)` |
| function ThreadSampler.owner | [源码:110](../../../tools/measure_voice_cpu.py#L110) · `def owner(self, handle)` |
| function ThreadSampler.close_thread | [源码:113](../../../tools/measure_voice_cpu.py#L113) · `def close_thread(self, handle)` |
| function ThreadSampler.description | [源码:117](../../../tools/measure_voice_cpu.py#L117) · `def description(self, handle)` |
| function ThreadSampler.thread_rows | [源码:125](../../../tools/measure_voice_cpu.py#L125) · `def thread_rows(self, live)` |
| function ThreadSampler.sample | [源码:141](../../../tools/measure_voice_cpu.py#L141) · `def sample(self, phase)` |
| function main | [源码:149](../../../tools/measure_voice_cpu.py#L149) · `def main()` |

## tools/measure_voice_phase.py

203 行；SHA256 `4ad378ef4eb8263f5a2bf1befe89fd5cf10e13845e659da2f15518982c1e07d7`

导入：`argparse`, `collections`, `hashlib`, `json`, `math`, `measure_audio_load`, `measure_combined`, `os`, `pathlib`, `platform`

| 定义 | 位置／签名 |
| --- | --- |
| function underrun_analysis | [源码:16](../../../tools/measure_voice_phase.py#L16) · `def underrun_analysis(report)` |
| function exact_analysis | [源码:75](../../../tools/measure_voice_phase.py#L75) · `def exact_analysis(report, count=64)` |
| function phase_analysis | [源码:102](../../../tools/measure_voice_phase.py#L102) · `def phase_analysis(report)` |
| function request_phase_analysis | [源码:147](../../../tools/measure_voice_phase.py#L147) · `def request_phase_analysis(report, count=64)` |
| function main | [源码:177](../../../tools/measure_voice_phase.py#L177) · `def main()` |

## tools/measure_window_recovery.py

282 行；SHA256 `640553dda937dbf754d1c53d88f82e9efb25d3a1d8b3078fabe886b6bfc59445`

导入：`argparse`, `ctypes`, `hashlib`, `json`, `math`, `measure_audio_load`, `measure_combined`, `measure_cpu_threads`, `measure_main_work`, `os`, `pathlib`, `queue`, `subprocess`, `threading`, `time`

| 定义 | 位置／签名 |
| --- | --- |
| function analyze | [源码:23](../../../tools/measure_window_recovery.py#L23) · `def analyze(report, samples, actions, *, retired_instance=False)` |
| function analyze_fault | [源码:83](../../../tools/measure_window_recovery.py#L83) · `def analyze_fault(report, samples, actions, plan, initialization=None)` |
| class Windows | [源码:151](../../../tools/measure_window_recovery.py#L151) |
| function Windows.__init__ | [源码:152](../../../tools/measure_window_recovery.py#L152) · `def __init__(self, pid)` |
| function Windows.owned | [源码:164](../../../tools/measure_window_recovery.py#L164) · `def owned(self, hwnd)` |
| function Windows.find | [源码:168](../../../tools/measure_window_recovery.py#L168) · `def find(self, index)` |
| function Windows.find.visit | [源码:170](../../../tools/measure_window_recovery.py#L170) · `def visit(hwnd, _)` |
| function Windows.move_monitor | [源码:182](../../../tools/measure_window_recovery.py#L182) · `def move_monitor(self, hwnd)` |
| function Windows.move_monitor.monitor | [源码:187](../../../tools/measure_window_recovery.py#L187) · `def monitor(_, __, r, ___)` |
| function Windows.restore | [源码:200](../../../tools/measure_window_recovery.py#L200) · `def restore(self, hwnd, sampler, samples)` |
| function acknowledge | [源码:219](../../../tools/measure_window_recovery.py#L219) · `def acknowledge(path, value)` |
| function main | [源码:226](../../../tools/measure_window_recovery.py#L226) · `def main()` |
| function main.read | [源码:241](../../../tools/measure_window_recovery.py#L241) · `def read()` |

## tools/measure_worker_native.py

88 行；SHA256 `5b6120cfb1d2f843265c5fcc848b8ce51db7cd466c69bfc0729e4916ecd167b6`

导入：`argparse`, `hashlib`, `json`, `math`, `measure_audio_load`, `measure_combined`, `measure_separated_native`, `os`, `pathlib`, `platform`

| 定义 | 位置／签名 |
| --- | --- |
| function evaluate_worker | [源码:17](../../../tools/measure_worker_native.py#L17) · `def evaluate_worker(report, metrics)` |
| function main | [源码:50](../../../tools/measure_worker_native.py#L50) · `def main()` |

## tools/p0_audio_device.py

200 行；SHA256 `a177c4443cbca80acefa66ff6d54be25dd4ec71a2d3a1ae3dea5aaa631725a64`

导入：`argparse`, `check_p0`, `ctypes`, `hashlib`, `json`, `math`, `os`, `pathlib`, `time`, `uuid`

| 定义 | 位置／签名 |
| --- | --- |
| function analyze | [源码:15](../../../tools/p0_audio_device.py#L15) · `def analyze(samples)` |
| class Guid | [源码:41](../../../tools/p0_audio_device.py#L41) · `c.Structure` |
| function Guid.__init__ | [源码:44](../../../tools/p0_audio_device.py#L44) · `def __init__(self, value)` |
| class WaveFormat | [源码:48](../../../tools/p0_audio_device.py#L48) · `c.Structure` |
| function checked | [源码:55](../../../tools/p0_audio_device.py#L55) · `def checked(hr, operation)` |
| function invoke | [源码:61](../../../tools/p0_audio_device.py#L61) · `def invoke(pointer, index, args, *values)` |
| function probe | [源码:67](../../../tools/p0_audio_device.py#L67) · `def probe()` |
| function probe.obtain | [源码:85](../../../tools/p0_audio_device.py#L85) · `def obtain(parent, index, args, *values)` |
| function probe.feed | [源码:118](../../../tools/p0_audio_device.py#L118) · `def feed()` |
| function probe.observe | [源码:130](../../../tools/p0_audio_device.py#L130) · `def observe(phase, duration, fill)` |
| function main | [源码:177](../../../tools/p0_audio_device.py#L177) · `def main()` |

## tools/p0_compressed_fixture.py

72 行；SHA256 `68f91b21c9f3742914e0ad21522d958922ded7326fc4cad20329d42ad0eddc8e`

导入：`argparse`, `check_p0`, `hashlib`, `json`, `math`, `pathlib`, `struct`, `subprocess`, `wave`

| 定义 | 位置／签名 |
| --- | --- |
| function digest | [源码:13](../../../tools/p0_compressed_fixture.py#L13) · `def digest(path)` |
| function encoder_path | [源码:20](../../../tools/p0_compressed_fixture.py#L20) · `def encoder_path()` |
| function make_fixture | [源码:26](../../../tools/p0_compressed_fixture.py#L26) · `def make_fixture(output, durations=(0.37, 3, 180), encoder=None)` |

## tools/p0_evidence.py

251 行；SHA256 `8f35c106f64618fe8bf46b9e7eb3dd92ae811e7ed658088b63581d6dfdf530b2`

导入：`argparse`, `datetime`, `hashlib`, `json`, `pathlib`, `re`, `struct`, `tempfile`, `wave`, `zlib`

| 定义 | 位置／签名 |
| --- | --- |
| function sha | [源码:20](../../../tools/p0_evidence.py#L20) · `def sha(data)` |
| function output_path | [源码:24](../../../tools/p0_evidence.py#L24) · `def output_path(value)` |
| function write_json | [源码:33](../../../tools/p0_evidence.py#L33) · `def write_json(path, value)` |
| function inspect_client | [源码:40](../../../tools/p0_evidence.py#L40) · `def inspect_client(directory)` |
| function chunk | [源码:87](../../../tools/p0_evidence.py#L87) · `def chunk(kind, data)` |
| function png | [源码:91](../../../tools/p0_evidence.py#L91) · `def png(width, height, pixel)` |
| function image_metadata | [源码:97](../../../tools/p0_evidence.py#L97) · `def image_metadata(data)` |
| function make_fixture | [源码:137](../../../tools/p0_evidence.py#L137) · `def make_fixture(destination)` |
| function make_fixture.ring | [源码:147](../../../tools/p0_evidence.py#L147) · `def ring(x, y)` |
| function make_fixture.disk | [源码:152](../../../tools/p0_evidence.py#L152) · `def disk(x, y)` |
| function make_fixture.numeral | [源码:162](../../../tools/p0_evidence.py#L162) · `def numeral(x, y, bits=bits)` |
| function register_screenshot | [源码:193](../../../tools/p0_evidence.py#L193) · `def register_screenshot(image, client_report, destination, case, release=None, fixture=None)` |
| function main | [源码:224](../../../tools/p0_evidence.py#L224) · `def main()` |

## tools/p0_frame_attribution.py

26 行；SHA256 `d6c258f95a68e044b9b37227507083e6c973844d35546ff923679cbee2dc8d49`

导入：`math`

| 定义 | 位置／签名 |
| --- | --- |
| function attribute | [源码:5](../../../tools/p0_frame_attribution.py#L5) · `def attribute(trace)` |

## tools/p0_gpu_repeats.py

107 行；SHA256 `d0465cec8d6fe4fbef4820b572e07e798995b9f25e404026fd7b72762a80d23b`

导入：`argparse`, `check_p0`, `check_p0_gui`, `hashlib`, `json`, `math`, `os`, `pathlib`, `shutil`, `statistics`, `subprocess`, `sys`, `uuid`

| 定义 | 位置／签名 |
| --- | --- |
| function distribution | [源码:22](../../../tools/p0_gpu_repeats.py#L22) · `def distribution(values)` |
| function case_metrics | [源码:28](../../../tools/p0_gpu_repeats.py#L28) · `def case_metrics(result, record, timings)` |
| function case_metrics.peak | [源码:35](../../../tools/p0_gpu_repeats.py#L35) · `def peak(key)` |
| function aggregate | [源码:47](../../../tools/p0_gpu_repeats.py#L47) · `def aggregate(cases)` |
| function fingerprint | [源码:54](../../../tools/p0_gpu_repeats.py#L54) · `def fingerprint(paths)` |
| function main | [源码:58](../../../tools/p0_gpu_repeats.py#L58) · `def main()` |

## tools/p0_heap_analysis.py

128 行；SHA256 `b85366a1b5f91066585e35aeb29ecb0c8f9391ddeb9bba7da18337eaeaa92a65`

导入：`collections`, `csv`

| 定义 | 位置／签名 |
| --- | --- |
| function summarize_heap_lines | [源码:10](../../../tools/p0_heap_analysis.py#L10) · `def summarize_heap_lines(lines, modules=(), executable='normal.exe')` |
| function summarize_heap_lines.origin_for | [源码:22](../../../tools/p0_heap_analysis.py#L22) · `def origin_for(address)` |
| function summarize_heap_lines.remove | [源码:35](../../../tools/p0_heap_analysis.py#L35) · `def remove(key)` |
| function summarize_heap_lines.flush | [源码:45](../../../tools/p0_heap_analysis.py#L45) · `def flush()` |

## tools/p0_lifetime_analysis.py

39 行；SHA256 `d1834decb5d015d62c5aa44d89d0b15b9a8bce6c39db9f571526bc6bd635fcbd`

导入：`bisect`

| 定义 | 位置／签名 |
| --- | --- |
| function release_samples | [源码:5](../../../tools/p0_lifetime_analysis.py#L5) · `def release_samples(samples, released_at, offsets=(0, 0.1, 0.5, 1, 3, 10), maximum_lag=0.25)` |
| function upload_receipts | [源码:20](../../../tools/p0_lifetime_analysis.py#L20) · `def upload_receipts(events)` |

## tools/p0_modules.py

61 行；SHA256 `1949a3e5450c6ca8bd45ee541b111add569f6124687c4dd5f68e669ecc8f7929`

导入：`ctypes`, `os`, `time`

| 定义 | 位置／签名 |
| --- | --- |
| function graphic_modules | [源码:8](../../../tools/p0_modules.py#L8) · `def graphic_modules(snapshot)` |
| function module_snapshot | [源码:15](../../../tools/p0_modules.py#L15) · `def module_snapshot(process)` |
| class module_snapshot.Info | [源码:24](../../../tools/p0_modules.py#L24) · `ctypes.Structure` |

## tools/p0_process_metrics.py

210 行；SHA256 `a736bbb1b9739bfa27242ac4d0c8eb1cab81d795f6f3ca2d6ba3a9b4e2b0fba3`

导入：`ctypes`, `os`, `platform`, `time`, `winreg`

| 定义 | 位置／签名 |
| --- | --- |
| class MemoryCounters | [源码:10](../../../tools/p0_process_metrics.py#L10) · `ctypes.Structure` |
| class CounterUnion | [源码:17](../../../tools/p0_process_metrics.py#L17) · `ctypes.Union` |
| class CounterValue | [源码:21](../../../tools/p0_process_metrics.py#L21) · `ctypes.Structure` |
| class CounterItem | [源码:25](../../../tools/p0_process_metrics.py#L25) · `ctypes.Structure` |
| class MemoryRegion | [源码:29](../../../tools/p0_process_metrics.py#L29) · `ctypes.Structure` |
| function summarize_regions | [源码:36](../../../tools/p0_process_metrics.py#L36) · `def summarize_regions(regions)` |
| class WindowsProcessMetrics | [源码:53](../../../tools/p0_process_metrics.py#L53) |
| function WindowsProcessMetrics.__init__ | [源码:54](../../../tools/p0_process_metrics.py#L54) · `def __init__(self, process)` |
| function WindowsProcessMetrics.sample | [源码:84](../../../tools/p0_process_metrics.py#L84) · `def sample(self)` |
| function WindowsProcessMetrics.virtual_memory_snapshot | [源码:114](../../../tools/p0_process_metrics.py#L114) · `def virtual_memory_snapshot(self)` |
| function WindowsProcessMetrics.close | [源码:150](../../../tools/p0_process_metrics.py#L150) · `def close(self)` |
| function environment_info | [源码:155](../../../tools/p0_process_metrics.py#L155) · `def environment_info()` |
| class environment_info.MemoryStatus | [源码:161](../../../tools/p0_process_metrics.py#L161) · `ctypes.Structure` |
| function summarize | [源码:171](../../../tools/p0_process_metrics.py#L171) · `def summarize(samples, phases)` |
| function cycle_statistics | [源码:192](../../../tools/p0_process_metrics.py#L192) · `def cycle_statistics(samples, phases)` |

## tools/p0_stable_batch.py

123 行；SHA256 `e7f0b6fabefb434ee6e8e31c9af40f7abf0ce3769f97f2fdf09aa318ac30c63c`

导入：`argparse`, `check_p0`, `hashlib`, `json`, `p0_evidence`, `pathlib`, `shutil`, `tempfile`, `wave`, `zipfile`

| 定义 | 位置／签名 |
| --- | --- |
| function digest | [源码:14](../../../tools/p0_stable_batch.py#L14) · `def digest(path)` |
| function prepare | [源码:17](../../../tools/p0_stable_batch.py#L17) · `def prepare(destination, client=None)` |
| function prepare.circle | [源码:29](../../../tools/p0_stable_batch.py#L29) · `def circle(x, y)` |
| function prepare.changed_ring | [源码:53](../../../tools/p0_stable_batch.py#L53) · `def changed_ring(x, y)` |

## tools/p0_vorbis_tail.py

196 行；SHA256 `8d92d76e08b8a3953838f1262e3d6eb91bf8afe985fc895532d4b44e220c1e68`

导入：`argparse`, `check_p0`, `check_p0_compressed`, `ctypes`, `hashlib`, `json`, `p0_compressed_fixture`, `pathlib`, `shutil`, `struct`, `subprocess`, `sys`, `time`, `uuid`, `wave`

| 定义 | 位置／签名 |
| --- | --- |
| function ogg_crc | [源码:20](../../../tools/p0_vorbis_tail.py#L20) · `def ogg_crc(data)` |
| function inspect_ogg | [源码:29](../../../tools/p0_vorbis_tail.py#L29) · `def inspect_ogg(data)` |
| class Info | [源码:69](../../../tools/p0_vorbis_tail.py#L69) · `ctypes.Structure` |
| function reference_library | [源码:74](../../../tools/p0_vorbis_tail.py#L74) · `def reference_library(path)` |
| function decode_reference | [源码:88](../../../tools/p0_vorbis_tail.py#L88) · `def decode_reference(lib, source, output)` |
| function run_pull | [源码:110](../../../tools/p0_vorbis_tail.py#L110) · `def run_pull(source)` |
| function checked_paths | [源码:128](../../../tools/p0_vorbis_tail.py#L128) · `def checked_paths(fixture, output)` |
| function audit | [源码:136](../../../tools/p0_vorbis_tail.py#L136) · `def audit(fixture, output)` |

## tools/p0_vulkan_memory.py

470 行；SHA256 `c91da53f767dd7d7171765e9191736863470e62ab3ce9463eda88a73095d875f`

导入：`argparse`, `check_p0`, `ctypes`, `hashlib`, `json`, `os`, `p0_modules`, `p0_process_metrics`, `pathlib`, `struct`, `subprocess`, `sys`, `time`, `uuid`

| 定义 | 位置／签名 |
| --- | --- |
| class AppInfo | [源码:27](../../../tools/p0_vulkan_memory.py#L27) · `c.Structure` |
| class InstanceInfo | [源码:32](../../../tools/p0_vulkan_memory.py#L32) · `c.Structure` |
| class QueueInfo | [源码:37](../../../tools/p0_vulkan_memory.py#L37) · `c.Structure` |
| class DeviceInfo | [源码:42](../../../tools/p0_vulkan_memory.py#L42) · `c.Structure` |
| class MemoryType | [源码:48](../../../tools/p0_vulkan_memory.py#L48) · `c.Structure` |
| class MemoryHeap | [源码:52](../../../tools/p0_vulkan_memory.py#L52) · `c.Structure` |
| class MemoryProperties | [源码:56](../../../tools/p0_vulkan_memory.py#L56) · `c.Structure` |
| class Family | [源码:60](../../../tools/p0_vulkan_memory.py#L60) · `c.Structure` |
| class AllocateInfo | [源码:64](../../../tools/p0_vulkan_memory.py#L64) · `c.Structure` |
| class Extent | [源码:68](../../../tools/p0_vulkan_memory.py#L68) · `c.Structure` |
| class ImageInfo | [源码:72](../../../tools/p0_vulkan_memory.py#L72) · `c.Structure` |
| class BufferInfo | [源码:78](../../../tools/p0_vulkan_memory.py#L78) · `c.Structure` |
| class Requirements | [源码:83](../../../tools/p0_vulkan_memory.py#L83) · `c.Structure` |
| function select_type | [源码:87](../../../tools/p0_vulkan_memory.py#L87) · `def select_type(types, bits, required, forbidden=0)` |
| function validate_abi | [源码:94](../../../tools/p0_vulkan_memory.py#L94) · `def validate_abi()` |
| class Vulkan | [源码:103](../../../tools/p0_vulkan_memory.py#L103) |
| function Vulkan.__init__ | [源码:104](../../../tools/p0_vulkan_memory.py#L104) · `def __init__(self)` |
| function Vulkan.bind | [源码:135](../../../tools/p0_vulkan_memory.py#L135) · `def bind(self, name, restype, arguments)` |
| function Vulkan.call | [源码:140](../../../tools/p0_vulkan_memory.py#L140) · `def call(self, name, *args)` |
| function Vulkan.create | [源码:145](../../../tools/p0_vulkan_memory.py#L145) · `def create(self, checkpoint)` |
| function Vulkan.allocate | [源码:189](../../../tools/p0_vulkan_memory.py#L189) · `def allocate(self, size, index)` |
| function Vulkan.add_memory | [源码:194](../../../tools/p0_vulkan_memory.py#L194) · `def add_memory(self, images, count)` |
| function Vulkan.release_memory | [源码:214](../../../tools/p0_vulkan_memory.py#L214) · `def release_memory(self)` |
| function Vulkan.host_buffer | [源码:224](../../../tools/p0_vulkan_memory.py#L224) · `def host_buffer(self)` |
| function Vulkan.map_host | [源码:234](../../../tools/p0_vulkan_memory.py#L234) · `def map_host(self)` |
| function Vulkan.release_host | [源码:241](../../../tools/p0_vulkan_memory.py#L241) · `def release_host(self)` |
| function Vulkan.close | [源码:252](../../../tools/p0_vulkan_memory.py#L252) · `def close(self)` |
| function child | [源码:263](../../../tools/p0_vulkan_memory.py#L263) · `def child(kind, settle_seconds=0)` |
| function child.mark | [源码:265](../../../tools/p0_vulkan_memory.py#L265) · `def mark(stage, **payload)` |
| function region_map | [源码:317](../../../tools/p0_vulkan_memory.py#L317) · `def region_map(process, metrics)` |
| function committed_totals | [源码:338](../../../tools/p0_vulkan_memory.py#L338) · `def committed_totals(snapshot)` |
| function committed_overlap | [源码:350](../../../tools/p0_vulkan_memory.py#L350) · `def committed_overlap(snapshot, address, size)` |
| function checkpoint_summary | [源码:365](../../../tools/p0_vulkan_memory.py#L365) · `def checkpoint_summary(checkpoints)` |
| function run_case | [源码:391](../../../tools/p0_vulkan_memory.py#L391) · `def run_case(kind, implicit_off, output, settle_seconds=0)` |
| function main | [源码:439](../../../tools/p0_vulkan_memory.py#L439) · `def main()` |

## tools/p0_workload.py

84 行；SHA256 `fc9f03661781b5964c5bba90c3ea752a85e92a40a41cbfcbd6e7f8e6898e7a5f`

导入：`argparse`, `hashlib`, `json`, `p0_evidence`, `pathlib`, `struct`, `tempfile`, `zlib`

| 定义 | 位置／签名 |
| --- | --- |
| function png | [源码:14](../../../tools/p0_workload.py#L14) · `def png(side, identity)` |
| function png.chunk | [源码:15](../../../tools/p0_workload.py#L15) · `def chunk(kind, data)` |
| function make_workload | [源码:34](../../../tools/p0_workload.py#L34) · `def make_workload(destination, asset_count=100, document_count=10, side=1024)` |

## tools/review_inventory.py

351 行；SHA256 `ed1c761572bdebc9a3c412e8e52bc8909d418dfbac9819a6821c53ca9a3aaa3b`

导入：`argparse`, `ast`, `collections`, `hashlib`, `html.parser`, `json`, `pathlib`, `re`, `subprocess`, `urllib.parse`

| 定义 | 位置／签名 |
| --- | --- |
| class Node | [源码:31](../../../tools/review_inventory.py#L31) |
| function Node.__init__ | [源码:32](../../../tools/review_inventory.py#L32) · `def __init__(self, tag='', attrs=(), parent=None)` |
| function Node.walk | [源码:34](../../../tools/review_inventory.py#L34) · `def walk(self)` |
| function Node.text | [源码:39](../../../tools/review_inventory.py#L39) · `def text(self)` |
| function Node.has_class | [源码:42](../../../tools/review_inventory.py#L42) · `def has_class(self, name)` |
| class Document | [源码:46](../../../tools/review_inventory.py#L46) · `HTMLParser` |
| function Document.__init__ | [源码:47](../../../tools/review_inventory.py#L47) · `def __init__(self, text)` |
| function Document.handle_starttag | [源码:50](../../../tools/review_inventory.py#L50) · `def handle_starttag(self, tag, attrs)` |
| function Document.handle_startendtag | [源码:53](../../../tools/review_inventory.py#L53) · `def handle_startendtag(self, tag, attrs)` |
| function Document.handle_endtag | [源码:55](../../../tools/review_inventory.py#L55) · `def handle_endtag(self, tag)` |
| function Document.handle_data | [源码:61](../../../tools/review_inventory.py#L61) · `def handle_data(self, data)` |
| function clean | [源码:64](../../../tools/review_inventory.py#L64) · `def clean(text)` |
| function source_link | [源码:67](../../../tools/review_inventory.py#L67) · `def source_link(node)` |
| function header | [源码:76](../../../tools/review_inventory.py#L76) · `def header(node)` |
| function references | [源码:80](../../../tools/review_inventory.py#L80) · `def references(node)` |
| function implementation_context | [源码:90](../../../tools/review_inventory.py#L90) · `def implementation_context(node)` |
| function unique_members | [源码:101](../../../tools/review_inventory.py#L101) · `def unique_members(rows)` |
| function rustdoc_item | [源码:113](../../../tools/review_inventory.py#L113) · `def rustdoc_item(text, crate, relative)` |
| function python_symbols | [源码:145](../../../tools/review_inventory.py#L145) · `def python_symbols(text)` |
| class python_symbols.Visitor | [源码:147](../../../tools/review_inventory.py#L147) · `ast.NodeVisitor` |
| function python_symbols.Visitor.__init__ | [源码:148](../../../tools/review_inventory.py#L148) · `def __init__(self)` |
| function python_symbols.Visitor.visit_ClassDef | [源码:149](../../../tools/review_inventory.py#L149) · `def visit_ClassDef(self, node)` |
| function python_symbols.Visitor.visit_FunctionDef | [源码:153](../../../tools/review_inventory.py#L153) · `def visit_FunctionDef(self, node)` |
| class python_symbols.Visitor.visit_FunctionDef.Calls | [源码:155](../../../tools/review_inventory.py#L155) · `ast.NodeVisitor` |
| function python_symbols.Visitor.visit_FunctionDef.Calls.visit_Call | [源码:156](../../../tools/review_inventory.py#L156) · `def visit_Call(self, child)` |
| function python_symbols.Visitor.visit_FunctionDef.Calls.visit_FunctionDef | [源码:158](../../../tools/review_inventory.py#L158) · `def visit_FunctionDef(self, child)` |
| function python_symbols.Visitor.visit_FunctionDef.Calls.visit_ClassDef | [源码:160](../../../tools/review_inventory.py#L160) · `def visit_ClassDef(self, child)` |
| function python_symbols.Visitor.visit_Import | [源码:170](../../../tools/review_inventory.py#L170) · `def visit_Import(self, node)` |
| function python_symbols.Visitor.visit_ImportFrom | [源码:171](../../../tools/review_inventory.py#L171) · `def visit_ImportFrom(self, node)` |
| function mask_rust | [源码:176](../../../tools/review_inventory.py#L176) · `def mask_rust(text)` |
| function mask_rust.blank | [源码:179](../../../tools/review_inventory.py#L179) · `def blank(a, b)` |
| function rust_lexical | [源码:205](../../../tools/review_inventory.py#L205) · `def rust_lexical(text)` |
| function merge_items | [源码:218](../../../tools/review_inventory.py#L218) · `def merge_items(items)` |
| function native_lexical | [源码:234](../../../tools/review_inventory.py#L234) · `def native_lexical(text, suffix)` |
| function link | [源码:248](../../../tools/review_inventory.py#L248) · `def link(source, label='源码')` |
| function cell | [源码:253](../../../tools/review_inventory.py#L253) · `def cell(text)` |
| function write_text | [源码:256](../../../tools/review_inventory.py#L256) · `def write_text(path, text)` |
| function load_metadata | [源码:260](../../../tools/review_inventory.py#L260) · `def load_metadata(path)` |
| function generate | [源码:267](../../../tools/review_inventory.py#L267) · `def generate(output, metadata)` |

## tools/run_desktop.py

27 行；SHA256 `fbb29a68bee5943160c0f171d5efc452ef8ed934de11991b6aeab3b65acb6b41`

导入：`argparse`, `check_core`, `subprocess`, `sys`

| 定义 | 位置／签名 |
| --- | --- |
| function main | [源码:7](../../../tools/run_desktop.py#L7) · `def main()` |

## tools/test_check_audio_edit.py

43 行；SHA256 `b6baa91860f1bdf18314493da960d4c399199205104eb34bedef098f4e4303cd`

导入：`copy`, `measure_audio_edit`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| function fixture | [源码:6](../../../tools/test_check_audio_edit.py#L6) · `def fixture()` |
| class AudioEditChecks | [源码:17](../../../tools/test_check_audio_edit.py#L17) · `unittest.TestCase` |
| function AudioEditChecks.test_warm_read_is_separate_and_added_for_budget | [源码:18](../../../tools/test_check_audio_edit.py#L18) · `def test_warm_read_is_separate_and_added_for_budget(self)` |
| function AudioEditChecks.test_missing_samples_nonfinite_and_reverse_clocks_are_rejected | [源码:23](../../../tools/test_check_audio_edit.py#L23) · `def test_missing_samples_nonfinite_and_reverse_clocks_are_rejected(self)` |
| function AudioEditChecks.test_latency_cache_cleanup_and_memory_fail_closed | [源码:30](../../../tools/test_check_audio_edit.py#L30) · `def test_latency_cache_cleanup_and_memory_fail_closed(self)` |

## tools/test_check_audio_load.py

54 行；SHA256 `db23c219092ae1bdfd78d2c6efb5e2002b7e0eb9c3aa44dcfb5bcc6cc8da05df`

导入：`copy`, `measure_audio_load`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| function samples | [源码:6](../../../tools/test_check_audio_load.py#L6) · `def samples()` |
| class AudioLoadChecks | [源码:12](../../../tools/test_check_audio_load.py#L12) · `unittest.TestCase` |
| function AudioLoadChecks.test_phase_cpu_never_includes_other_phase_transitions | [源码:13](../../../tools/test_check_audio_load.py#L13) · `def test_phase_cpu_never_includes_other_phase_transitions(self)` |
| function AudioLoadChecks.test_missing_counters_phases_short_spans_and_negative_cpu_fail | [源码:19](../../../tools/test_check_audio_load.py#L19) · `def test_missing_counters_phases_short_spans_and_negative_cpu_fail(self)` |
| function AudioLoadChecks.test_percentile_uses_nearest_rank_and_invalid_samples_fail | [源码:29](../../../tools/test_check_audio_load.py#L29) · `def test_percentile_uses_nearest_rank_and_invalid_samples_fail(self)` |
| function AudioLoadChecks.test_latency_underflow_and_memory_limits_are_not_silently_waived | [源码:35](../../../tools/test_check_audio_load.py#L35) · `def test_latency_underflow_and_memory_limits_are_not_silently_waived(self)` |

## tools/test_check_audio_startup.py

34 行；SHA256 `5f2cbdd09841597faf574c240b33131f9d4c1bae9c079847fddc9fd05f8d4a28`

导入：`measure_audio_startup`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| function fixture | [源码:5](../../../tools/test_check_audio_startup.py#L5) · `def fixture()` |
| class StartupChecks | [源码:13](../../../tools/test_check_audio_startup.py#L13) · `unittest.TestCase` |
| function StartupChecks.test_all_instances_and_windows_are_required | [源码:14](../../../tools/test_check_audio_startup.py#L14) · `def test_all_instances_and_windows_are_required(self)` |
| function StartupChecks.test_underflow_release_capacity_and_memory_fail | [源码:20](../../../tools/test_check_audio_startup.py#L20) · `def test_underflow_release_capacity_and_memory_fail(self)` |
| function StartupChecks.test_inconsistent_or_unbounded_trace_data_is_rejected | [源码:27](../../../tools/test_check_audio_startup.py#L27) · `def test_inconsistent_or_unbounded_trace_data_is_rejected(self)` |

## tools/test_check_automatic_closeout.py

41 行；SHA256 `56ad8dd62a08d84703ff3b579eedb1d68f8750caabd90fbbfc636f707b2f2174`

导入：`measure_automatic_closeout`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| class CompleteGpuBudget | [源码:5](../../../tools/test_check_automatic_closeout.py#L5) · `unittest.TestCase` |
| function CompleteGpuBudget.test_unknown_missing_and_all_lifecycle_samples_are_not_waived | [源码:6](../../../tools/test_check_automatic_closeout.py#L6) · `def test_unknown_missing_and_all_lifecycle_samples_are_not_waived(self)` |
| function CompleteGpuBudget.test_exit_boundary_is_recorded_unknown_and_never_excuses_live_unknown_or_known_overrun | [源码:13](../../../tools/test_check_automatic_closeout.py#L13) · `def test_exit_boundary_is_recorded_unknown_and_never_excuses_live_unknown_or_known_overrun(self)` |
| function CompleteGpuBudget.test_only_a_contiguous_initial_loading_prefix_can_be_reported_separately | [源码:27](../../../tools/test_check_automatic_closeout.py#L27) · `def test_only_a_contiguous_initial_loading_prefix_can_be_reported_separately(self)` |

## tools/test_check_background_stream.py

36 行；SHA256 `6b1b0cf0b181ee306ea0c721c0761a91d9d43c9ec83eadf18fa06b5a2c485e2e`

导入：`measure_background_stream`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| function report | [源码:5](../../../tools/test_check_background_stream.py#L5) · `def report()` |
| class BackgroundStreamChecks | [源码:13](../../../tools/test_check_background_stream.py#L13) · `unittest.TestCase` |
| function BackgroundStreamChecks.test_fixed_plan_and_long_source_not_whole_pcm | [源码:14](../../../tools/test_check_background_stream.py#L14) · `def test_fixed_plan_and_long_source_not_whole_pcm(self)` |
| function BackgroundStreamChecks.test_fault_unreleased_raw_and_failed_seek_cannot_pass | [源码:20](../../../tools/test_check_background_stream.py#L20) · `def test_fault_unreleased_raw_and_failed_seek_cannot_pass(self)` |
| function BackgroundStreamChecks.test_missing_stream_identity_or_invalid_duration_is_rejected | [源码:25](../../../tools/test_check_background_stream.py#L25) · `def test_missing_stream_identity_or_invalid_duration_is_rejected(self)` |
| function BackgroundStreamChecks.test_each_callback_ownership_violation_fails | [源码:30](../../../tools/test_check_background_stream.py#L30) · `def test_each_callback_ownership_violation_fails(self)` |

## tools/test_check_budget_review.py

90 行；SHA256 `a878cdc44af5c5a2244d383b79f3f0bf1d4467f35b30e526dbdd0a04ef20b584`

导入：`copy`, `hashlib`, `math`, `measure_budget_review`, `pathlib`, `tempfile`, `unittest`, `unittest.mock`

| 定义 | 位置／签名 |
| --- | --- |
| class BudgetReviewTests | [源码:11](../../../tools/test_check_budget_review.py#L11) · `unittest.TestCase` |
| function BudgetReviewTests.fixture | [源码:12](../../../tools/test_check_budget_review.py#L12) · `def fixture(self)` |
| function BudgetReviewTests.test_cpu_over_goal_is_visible_without_rewriting_legacy_or_granting_p2 | [源码:17](../../../tools/test_check_budget_review.py#L17) · `def test_cpu_over_goal_is_visible_without_rewriting_legacy_or_granting_p2(self)` |
| function BudgetReviewTests.test_each_existing_hard_failure_still_blocks | [源码:29](../../../tools/test_check_budget_review.py#L29) · `def test_each_existing_hard_failure_still_blocks(self)` |
| function BudgetReviewTests.test_unknown_cpu_named_predicate_is_not_silently_waived | [源码:38](../../../tools/test_check_budget_review.py#L38) · `def test_unknown_cpu_named_predicate_is_not_silently_waived(self)` |
| function BudgetReviewTests.test_missing_checks_phases_or_invalid_values_are_rejected | [源码:43](../../../tools/test_check_budget_review.py#L43) · `def test_missing_checks_phases_or_invalid_values_are_rejected(self)` |
| function BudgetReviewTests.test_invalid_meter_never_turns_low_counters_into_cpu_success | [源码:58](../../../tools/test_check_budget_review.py#L58) · `def test_invalid_meter_never_turns_low_counters_into_cpu_success(self)` |
| function BudgetReviewTests.test_lifecycle_peak_is_hard_and_short_idle_is_not_B05 | [源码:65](../../../tools/test_check_budget_review.py#L65) · `def test_lifecycle_peak_is_hard_and_short_idle_is_not_B05(self)` |
| function BudgetReviewTests.test_feature_or_mismatched_release_receipt_is_rejected | [源码:73](../../../tools/test_check_budget_review.py#L73) · `def test_feature_or_mismatched_release_receipt_is_rejected(self)` |

## tools/test_check_combined.py

205 行；SHA256 `34c0173a098916f3fd52998b76c2cd471a2bddcc86e4e97fac9c61f741ec7672`

导入：`check_desktop`, `copy`, `io`, `json`, `measure_audio_load`, `measure_combined`, `pathlib`, `queue`, `tempfile`, `unittest`, `unittest.mock`

| 定义 | 位置／签名 |
| --- | --- |
| function fixture | [源码:13](../../../tools/test_check_combined.py#L13) · `def fixture()` |
| class CombinedChecks | [源码:32](../../../tools/test_check_combined.py#L32) · `unittest.TestCase` |
| function CombinedChecks.test_unscoped_staging_modifier_stops_before_opening_a_process | [源码:33](../../../tools/test_check_combined.py#L33) · `def test_unscoped_staging_modifier_stops_before_opening_a_process(self)` |
| function CombinedChecks.test_staging_candidate_requires_exact_strategy_and_independent_budget | [源码:38](../../../tools/test_check_combined.py#L38) · `def test_staging_candidate_requires_exact_strategy_and_independent_budget(self)` |
| function CombinedChecks.test_staging_unknown_counts_and_reference_leases_are_not_zero | [源码:50](../../../tools/test_check_combined.py#L50) · `def test_staging_unknown_counts_and_reference_leases_are_not_zero(self)` |
| function CombinedChecks.test_callback_entry_and_commit_do_not_erase_observation_gate_or_overlap | [源码:57](../../../tools/test_check_combined.py#L57) · `def test_callback_entry_and_commit_do_not_erase_observation_gate_or_overlap(self)` |
| function CombinedChecks.test_preparation_lifecycle_separates_worker_and_publish_delay | [源码:73](../../../tools/test_check_combined.py#L73) · `def test_preparation_lifecycle_separates_worker_and_publish_delay(self)` |
| function CombinedChecks.test_memory_policy_cannot_silently_run_the_original_configuration | [源码:86](../../../tools/test_check_combined.py#L86) · `def test_memory_policy_cannot_silently_run_the_original_configuration(self)` |
| function CombinedChecks.test_pixel_comparison_rejects_missing_frames_wrong_policy_and_pixel_drift | [源码:95](../../../tools/test_check_combined.py#L95) · `def test_pixel_comparison_rejects_missing_frames_wrong_policy_and_pixel_drift(self)` |
| function CombinedChecks.test_desktop_rejects_unused_memory_policy_before_toolchain_or_device_access | [源码:109](../../../tools/test_check_combined.py#L109) · `def test_desktop_rejects_unused_memory_policy_before_toolchain_or_device_access(self)` |
| function CombinedChecks.test_timeout_reaps_only_the_owned_child_and_preserves_partial_samples | [源码:114](../../../tools/test_check_combined.py#L114) · `def test_timeout_reaps_only_the_owned_child_and_preserves_partial_samples(self)` |
| class CombinedChecks.test_timeout_reaps_only_the_owned_child_and_preserves_partial_samples.Child | [源码:115](../../../tools/test_check_combined.py#L115) |
| function CombinedChecks.test_timeout_reaps_only_the_owned_child_and_preserves_partial_samples.Child.__init__ | [源码:117](../../../tools/test_check_combined.py#L117) · `def __init__(self, *args, **kwargs)` |
| function CombinedChecks.test_timeout_reaps_only_the_owned_child_and_preserves_partial_samples.Child.poll | [源码:118](../../../tools/test_check_combined.py#L118) · `def poll(self)` |
| function CombinedChecks.test_timeout_reaps_only_the_owned_child_and_preserves_partial_samples.Child.kill | [源码:119](../../../tools/test_check_combined.py#L119) · `def kill(self)` |
| function CombinedChecks.test_timeout_reaps_only_the_owned_child_and_preserves_partial_samples.Child.wait | [源码:120](../../../tools/test_check_combined.py#L120) · `def wait(self, timeout)` |
| class CombinedChecks.test_timeout_reaps_only_the_owned_child_and_preserves_partial_samples.Sampler | [源码:121](../../../tools/test_check_combined.py#L121) |
| function CombinedChecks.test_timeout_reaps_only_the_owned_child_and_preserves_partial_samples.Sampler.__init__ | [源码:122](../../../tools/test_check_combined.py#L122) · `def __init__(self, child)` |
| function CombinedChecks.test_timeout_reaps_only_the_owned_child_and_preserves_partial_samples.Sampler.sample | [源码:123](../../../tools/test_check_combined.py#L123) · `def sample(self, phase)` |
| class CombinedChecks.test_timeout_reaps_only_the_owned_child_and_preserves_partial_samples.EmptyQueue | [源码:124](../../../tools/test_check_combined.py#L124) |
| function CombinedChecks.test_timeout_reaps_only_the_owned_child_and_preserves_partial_samples.EmptyQueue.__init__ | [源码:125](../../../tools/test_check_combined.py#L125) · `def __init__(self, *args, **kwargs)` |
| function CombinedChecks.test_timeout_reaps_only_the_owned_child_and_preserves_partial_samples.EmptyQueue.get | [源码:126](../../../tools/test_check_combined.py#L126) · `def get(self, timeout)` |
| function CombinedChecks.test_recorded_source_identity_rejects_a_changed_or_missing_source | [源码:134](../../../tools/test_check_combined.py#L134) · `def test_recorded_source_identity_rejects_a_changed_or_missing_source(self)` |
| function CombinedChecks.test_complete_standard_run_uses_separate_process_and_gpu_budgets | [源码:139](../../../tools/test_check_combined.py#L139) · `def test_complete_standard_run_uses_separate_process_and_gpu_budgets(self)` |
| function CombinedChecks.test_missing_gpu_is_unknown_and_other_pids_are_excluded | [源码:144](../../../tools/test_check_combined.py#L144) · `def test_missing_gpu_is_unknown_and_other_pids_are_excluded(self)` |
| function CombinedChecks.test_budget_latency_underflow_and_release_failures_are_preserved | [源码:150](../../../tools/test_check_combined.py#L150) · `def test_budget_latency_underflow_and_release_failures_are_preserved(self)` |
| function CombinedChecks.test_short_schedules_traces_and_clock_spans_are_rejected | [源码:159](../../../tools/test_check_combined.py#L159) · `def test_short_schedules_traces_and_clock_spans_are_rejected(self)` |
| function CombinedChecks.test_preparation_overflow_missing_completion_and_long_frame_cannot_be_hidden | [源码:166](../../../tools/test_check_combined.py#L166) · `def test_preparation_overflow_missing_completion_and_long_frame_cannot_be_hidden(self)` |
| function CombinedChecks.test_submission_brackets_distinguish_queue_delay_from_uncertainty_without_passing_borderline | [源码:178](../../../tools/test_check_combined.py#L178) · `def test_submission_brackets_distinguish_queue_delay_from_uncertainty_without_passing_borderline(self)` |
| function CombinedChecks.test_allocator_capacity_is_distinct_from_live_allocation_and_unknown_is_explicit | [源码:193](../../../tools/test_check_combined.py#L193) · `def test_allocator_capacity_is_distinct_from_live_allocation_and_unknown_is_explicit(self)` |

## tools/test_check_controlled_recovery.py

47 行；SHA256 `cc16833fdfec0a0d9a443da5b01b31fdf765feb17a051269df2978c42b2b360c`

导入：`copy`, `measure_controlled_recovery`, `test_check_supply_recovery`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| function controlled | [源码:7](../../../tools/test_check_controlled_recovery.py#L7) · `def controlled()` |
| class ControlledRecoveryChecks | [源码:18](../../../tools/test_check_controlled_recovery.py#L18) · `unittest.TestCase` |
| function ControlledRecoveryChecks.test_real_refill_rows_and_frozen_plan_are_required | [源码:19](../../../tools/test_check_controlled_recovery.py#L19) · `def test_real_refill_rows_and_frozen_plan_are_required(self)` |
| function ControlledRecoveryChecks.test_counter_discontinuity_missing_mode_and_fake_commit_are_rejected | [源码:26](../../../tools/test_check_controlled_recovery.py#L26) · `def test_counter_discontinuity_missing_mode_and_fake_commit_are_rejected(self)` |
| function ControlledRecoveryChecks.test_latency_failure_is_not_hidden_by_successful_recovery | [源码:38](../../../tools/test_check_controlled_recovery.py#L38) · `def test_latency_failure_is_not_hidden_by_successful_recovery(self)` |

## tools/test_check_core_architecture.py

93 行；SHA256 `fb8226852e9fbaaf925a7e9fd17b8ec8fecc46f64b98e219b42031851eab7a32`

导入：`check_core_architecture`, `json`, `pathlib`, `shutil`, `subprocess`, `tempfile`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| function metadata_fixture | [源码:14](../../../tools/test_check_core_architecture.py#L14) · `def metadata_fixture()` |
| class ArchitectureChecks | [源码:26](../../../tools/test_check_core_architecture.py#L26) · `unittest.TestCase` |
| function ArchitectureChecks.test_reviewed_graph_is_allowed | [源码:27](../../../tools/test_check_core_architecture.py#L27) · `def test_reviewed_graph_is_allowed(self)` |
| function ArchitectureChecks.test_reverse_dependency_is_rejected | [源码:30](../../../tools/test_check_core_architecture.py#L30) · `def test_reverse_dependency_is_rejected(self)` |
| function ArchitectureChecks.test_external_and_p0_packages_are_rejected | [源码:35](../../../tools/test_check_core_architecture.py#L35) · `def test_external_and_p0_packages_are_rejected(self)` |
| function ArchitectureChecks.test_domain_name_cannot_hide_an_external_path | [源码:41](../../../tools/test_check_core_architecture.py#L41) · `def test_domain_name_cannot_hide_an_external_path(self)` |
| function ArchitectureChecks.test_optional_and_platform_dependencies_require_review | [源码:46](../../../tools/test_check_core_architecture.py#L46) · `def test_optional_and_platform_dependencies_require_review(self)` |
| function ArchitectureChecks.test_build_scripts_and_features_are_rejected | [源码:52](../../../tools/test_check_core_architecture.py#L52) · `def test_build_scripts_and_features_are_rejected(self)` |
| function ArchitectureChecks.test_io_includes_and_obscured_imports_are_rejected | [源码:61](../../../tools/test_check_core_architecture.py#L61) · `def test_io_includes_and_obscured_imports_are_rejected(self)` |
| function ArchitectureChecks.test_pure_value_sources_are_allowed | [源码:66](../../../tools/test_check_core_architecture.py#L66) · `def test_pure_value_sources_are_allowed(self)` |
| function ArchitectureChecks.test_real_cargo_graph_with_an_injected_ui_crate_is_rejected | [源码:69](../../../tools/test_check_core_architecture.py#L69) · `def test_real_cargo_graph_with_an_injected_ui_crate_is_rejected(self)` |

## tools/test_check_core_cost.py

48 行；SHA256 `dbe75725a39a53476f81936f7f4b12d5032d672f2e329befcfc823f3b681aa57`

导入：`check_core_cost`, `json`, `pathlib`, `tempfile`, `unittest`, `unittest.mock`

| 定义 | 位置／签名 |
| --- | --- |
| class CoreCostChecks | [源码:9](../../../tools/test_check_core_cost.py#L9) · `unittest.TestCase` |
| function CoreCostChecks.phase | [源码:10](../../../tools/test_check_core_cost.py#L10) · `def phase(self, name='transactions_a', operations=5000)` |
| function CoreCostChecks.test_phase_order_counts_and_numeric_values_are_required | [源码:14](../../../tools/test_check_core_cost.py#L14) · `def test_phase_order_counts_and_numeric_values_are_required(self)` |
| function CoreCostChecks.test_branch_and_draft_workloads_cannot_be_shortened | [源码:22](../../../tools/test_check_core_cost.py#L22) · `def test_branch_and_draft_workloads_cannot_be_shortened(self)` |
| function CoreCostChecks.test_output_rejects_existing_and_external_directories | [源码:28](../../../tools/test_check_core_cost.py#L28) · `def test_output_rejects_existing_and_external_directories(self)` |
| function CoreCostChecks.test_metadata_profiles_keep_standard_and_pressure_distinct | [源码:37](../../../tools/test_check_core_cost.py#L37) · `def test_metadata_profiles_keep_standard_and_pressure_distinct(self)` |
| function CoreCostChecks.test_actual_cargo_artifact_is_required_instead_of_a_stale_default_path | [源码:42](../../../tools/test_check_core_cost.py#L42) · `def test_actual_cargo_artifact_is_required_instead_of_a_stale_default_path(self)` |

## tools/test_check_core_environment.py

33 行；SHA256 `df4d661b681b27d21a1a02071c01e25c4d7f3577d830b48d269fa34a77499d61`

导入：`check_core`, `pathlib`, `unittest`, `unittest.mock`

| 定义 | 位置／签名 |
| --- | --- |
| class CoreEnvironmentChecks | [源码:8](../../../tools/test_check_core_environment.py#L8) · `unittest.TestCase` |
| function CoreEnvironmentChecks.test_missing_registered_toolchain_fails_without_downloading | [源码:9](../../../tools/test_check_core_environment.py#L9) · `def test_missing_registered_toolchain_fails_without_downloading(self)` |
| function CoreEnvironmentChecks.test_wrong_compiler_is_rejected_without_changing_settings | [源码:17](../../../tools/test_check_core_environment.py#L17) · `def test_wrong_compiler_is_rejected_without_changing_settings(self)` |
| function CoreEnvironmentChecks.test_inherited_compiler_overrides_are_rejected_without_echoing_values | [源码:23](../../../tools/test_check_core_environment.py#L23) · `def test_inherited_compiler_overrides_are_rejected_without_echoing_values(self)` |

## tools/test_check_cpu_accounting.py

111 行；SHA256 `79aa2e5bd416f90419518abd19947a1ae2c89f2975f90fe4865d38cb74f6ae84`

导入：`measure_cpu_accounting`, `types`, `unittest`, `unittest.mock`

| 定义 | 位置／签名 |
| --- | --- |
| class ReferenceChecks | [源码:7](../../../tools/test_check_cpu_accounting.py#L7) · `unittest.TestCase` |
| function ReferenceChecks.test_periodic_work_skips_missed_pulses_without_catchup | [源码:8](../../../tools/test_check_cpu_accounting.py#L8) · `def test_periodic_work_skips_missed_pulses_without_catchup(self)` |
| function ReferenceChecks.row | [源码:14](../../../tools/test_check_cpu_accounting.py#L14) · `def row(self)` |
| function ReferenceChecks.test_awake_wall_cycles_and_charged_cpu_are_separate_not_discounted | [源码:18](../../../tools/test_check_cpu_accounting.py#L18) · `def test_awake_wall_cycles_and_charged_cpu_are_separate_not_discounted(self)` |
| function ReferenceChecks.test_invalid_layout_negative_or_unbounded_reference_rejects | [源码:25](../../../tools/test_check_cpu_accounting.py#L25) · `def test_invalid_layout_negative_or_unbounded_reference_rejects(self)` |
| function ReferenceChecks.test_timer_close_is_idempotent_and_closes_even_if_cancel_fails | [源码:31](../../../tools/test_check_cpu_accounting.py#L31) · `def test_timer_close_is_idempotent_and_closes_even_if_cancel_fails(self)` |
| function ReferenceChecks.test_timer_uses_relative_one_shot_and_bounded_failure_guard | [源码:38](../../../tools/test_check_cpu_accounting.py#L38) · `def test_timer_uses_relative_one_shot_and_bounded_failure_guard(self)` |
| function ReferenceChecks.test_idle_fake_has_no_pulses_and_keeps_raw_counter_difference | [源码:47](../../../tools/test_check_cpu_accounting.py#L47) · `def test_idle_fake_has_no_pulses_and_keeps_raw_counter_difference(self)` |
| function ReferenceChecks.test_fixed_reference_has_short_and_long_windows_without_product_acceptance | [源码:54](../../../tools/test_check_cpu_accounting.py#L54) · `def test_fixed_reference_has_short_and_long_windows_without_product_acceptance(self)` |
| function ReferenceChecks.test_periodic_fake_uses_bounded_measured_work_without_real_sleep | [源码:58](../../../tools/test_check_cpu_accounting.py#L58) · `def test_periodic_fake_uses_bounded_measured_work_without_real_sleep(self)` |
| function ReferenceChecks.test_periodic_fake_uses_bounded_measured_work_without_real_sleep.clock | [源码:60](../../../tools/test_check_cpu_accounting.py#L60) · `def clock()` |
| function ReferenceChecks.test_periodic_fake_uses_bounded_measured_work_without_real_sleep.counters | [源码:62](../../../tools/test_check_cpu_accounting.py#L62) · `def counters()` |
| function ReferenceChecks.test_dense_fake_counts_executed_work_not_waits | [源码:71](../../../tools/test_check_cpu_accounting.py#L71) · `def test_dense_fake_counts_executed_work_not_waits(self)` |
| function ReferenceChecks.test_dense_fake_counts_executed_work_not_waits.clock | [源码:73](../../../tools/test_check_cpu_accounting.py#L73) · `def clock()` |
| function ReferenceChecks.test_dense_fake_counts_executed_work_not_waits.counters | [源码:74](../../../tools/test_check_cpu_accounting.py#L74) · `def counters()` |
| function ReferenceChecks.test_dense_fake_counts_executed_work_not_waits.wait | [源码:75](../../../tools/test_check_cpu_accounting.py#L75) · `def wait(deadline)` |
| function ReferenceChecks.test_periodic_sampling_model_can_overcount_or_miss_identical_work | [源码:80](../../../tools/test_check_cpu_accounting.py#L80) · `def test_periodic_sampling_model_can_overcount_or_miss_identical_work(self)` |
| function ReferenceChecks.test_longer_model_window_does_not_remove_phase_bias | [源码:88](../../../tools/test_check_cpu_accounting.py#L88) · `def test_longer_model_window_does_not_remove_phase_bias(self)` |
| function ReferenceChecks.test_model_capacity_and_layout_are_bounded | [源码:95](../../../tools/test_check_cpu_accounting.py#L95) · `def test_model_capacity_and_layout_are_bounded(self)` |
| function ReferenceChecks.test_report_flags_cannot_replace_work_or_owner_evidence | [源码:99](../../../tools/test_check_cpu_accounting.py#L99) · `def test_report_flags_cannot_replace_work_or_owner_evidence(self)` |

## tools/test_check_cpu_threads.py

114 行；SHA256 `87d77c04adb8a2186ab9418863fe9d74459f75e3b6a6a1b3c13204acf5f53444`

导入：`copy`, `measure_cpu_threads`, `measure_independent_cpu`, `pathlib`, `struct`, `tempfile`, `test_check_independent_cpu`, `unittest`, `unittest.mock`

| 定义 | 位置／签名 |
| --- | --- |
| class CpuThreadChecks | [源码:12](../../../tools/test_check_cpu_threads.py#L12) · `unittest.TestCase` |
| function CpuThreadChecks.test_fixed_control_brackets_token_comparison_with_limited_plain_observers | [源码:13](../../../tools/test_check_cpu_threads.py#L13) · `def test_fixed_control_brackets_token_comparison_with_limited_plain_observers(self)` |
| function CpuThreadChecks.test_permission_is_verified_by_native_runner_and_product_not_a_requested_label | [源码:18](../../../tools/test_check_cpu_threads.py#L18) · `def test_permission_is_verified_by_native_runner_and_product_not_a_requested_label(self)` |
| function CpuThreadChecks.test_permission_mismatch_cannot_fallback_to_elevated_as_normal | [源码:28](../../../tools/test_check_cpu_threads.py#L28) · `def test_permission_mismatch_cannot_fallback_to_elevated_as_normal(self)` |
| function CpuThreadChecks.test_expensive_identity_query_only_on_phase_boundaries | [源码:34](../../../tools/test_check_cpu_threads.py#L34) · `def test_expensive_identity_query_only_on_phase_boundaries(self)` |
| function CpuThreadChecks.test_expensive_identity_query_only_on_phase_boundaries.identify | [源码:36](../../../tools/test_check_cpu_threads.py#L36) · `def identify(p)` |
| function CpuThreadChecks.test_identity_phase_admission_is_bounded_before_query | [源码:42](../../../tools/test_check_cpu_threads.py#L42) · `def test_identity_phase_admission_is_bounded_before_query(self)` |
| function CpuThreadChecks.test_identity_failure_does_not_cache_an_incomplete_observation | [源码:48](../../../tools/test_check_cpu_threads.py#L48) · `def test_identity_failure_does_not_cache_an_incomplete_observation(self)` |
| function CpuThreadChecks.test_identity_failure_does_not_cache_an_incomplete_observation.fail | [源码:50](../../../tools/test_check_cpu_threads.py#L50) · `def fail(p)` |
| function CpuThreadChecks.report | [源码:54](../../../tools/test_check_cpu_threads.py#L54) · `def report(self)` |
| function CpuThreadChecks.test_thread_sum_keeps_unknown_threads_and_does_not_repair_original_failure | [源码:61](../../../tools/test_check_cpu_threads.py#L61) · `def test_thread_sum_keeps_unknown_threads_and_does_not_repair_original_failure(self)` |
| function CpuThreadChecks.test_v2_reused_tid_cost_is_preserved_without_inheriting_a_name_or_entry | [源码:70](../../../tools/test_check_cpu_threads.py#L70) · `def test_v2_reused_tid_cost_is_preserved_without_inheriting_a_name_or_entry(self)` |
| function CpuThreadChecks.test_module_start_is_not_entry_when_rva_differs | [源码:78](../../../tools/test_check_cpu_threads.py#L78) · `def test_module_start_is_not_entry_when_rva_differs(self)` |
| function CpuThreadChecks.test_owned_baseline_filename_does_not_lose_entry_identity | [源码:83](../../../tools/test_check_cpu_threads.py#L83) · `def test_owned_baseline_filename_does_not_lose_entry_identity(self)` |
| function CpuThreadChecks.test_phase_module_identity_is_not_overwritten_by_later_unload | [源码:88](../../../tools/test_check_cpu_threads.py#L88) · `def test_phase_module_identity_is_not_overwritten_by_later_unload(self)` |
| function CpuThreadChecks.test_reused_tid_with_another_creation_identity_rejects_join | [源码:95](../../../tools/test_check_cpu_threads.py#L95) · `def test_reused_tid_with_another_creation_identity_rejects_join(self)` |
| function CpuThreadChecks.test_open_observer_handles_fail_cleanup_gate | [源码:99](../../../tools/test_check_cpu_threads.py#L99) · `def test_open_observer_handles_fail_cleanup_gate(self)` |
| function CpuThreadChecks.test_pe_entry_uses_bounded_product_header_and_rejects_other_formats | [源码:103](../../../tools/test_check_cpu_threads.py#L103) · `def test_pe_entry_uses_bounded_product_header_and_rejects_other_formats(self)` |

## tools/test_check_current_combined.py

96 行；SHA256 `28f9de11b2aba1999356112417b0f8744d810f74350bc680e67a51ed7af86ba5`

导入：`measure_current_combined`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| function fixture | [源码:5](../../../tools/test_check_current_combined.py#L5) · `def fixture()` |
| class CurrentCombinedChecks | [源码:28](../../../tools/test_check_current_combined.py#L28) · `unittest.TestCase` |
| function CurrentCombinedChecks.test_cpu_target_is_nonblocking_but_not_inferred_from_raw_counts | [源码:29](../../../tools/test_check_current_combined.py#L29) · `def test_cpu_target_is_nonblocking_but_not_inferred_from_raw_counts(self)` |
| function CurrentCombinedChecks.test_refuses_legacy_route_incomplete_schedule_and_early_completion | [源码:37](../../../tools/test_check_current_combined.py#L37) · `def test_refuses_legacy_route_incomplete_schedule_and_early_completion(self)` |
| function CurrentCombinedChecks.test_exact_probes_reject_duplicates_wrong_channels_and_invalid_times | [源码:42](../../../tools/test_check_current_combined.py#L42) · `def test_exact_probes_reject_duplicates_wrong_channels_and_invalid_times(self)` |
| function CurrentCombinedChecks.test_zero_underrun_latency_and_callback_ownership_are_hard | [源码:50](../../../tools/test_check_current_combined.py#L50) · `def test_zero_underrun_latency_and_callback_ownership_are_hard(self)` |
| function CurrentCombinedChecks.test_actual_resource_release_and_all_lifecycle_memory_cannot_be_waived | [源码:59](../../../tools/test_check_current_combined.py#L59) · `def test_actual_resource_release_and_all_lifecycle_memory_cannot_be_waived(self)` |
| function CurrentCombinedChecks.test_unknown_gpu_and_above_budget_are_not_zero | [源码:67](../../../tools/test_check_current_combined.py#L67) · `def test_unknown_gpu_and_above_budget_are_not_zero(self)` |
| function CurrentCombinedChecks.test_clock_continuity_and_publication_completeness | [源码:72](../../../tools/test_check_current_combined.py#L72) · `def test_clock_continuity_and_publication_completeness(self)` |
| class ProductIdentity | [源码:85](../../../tools/test_check_current_combined.py#L85) · `unittest.TestCase` |
| function ProductIdentity.test_product_policy_is_explicit_and_keeps_all_hard_checks | [源码:86](../../../tools/test_check_current_combined.py#L86) · `def test_product_policy_is_explicit_and_keeps_all_hard_checks(self)` |

## tools/test_check_desktop_architecture.py

80 行；SHA256 `82c9423c1774c412647d20228161f5151c493c1ec3c755c688d1d36b0d10e53f`

导入：`check_desktop_architecture`, `copy`, `pathlib`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| function fixture | [源码:7](../../../tools/test_check_desktop_architecture.py#L7) · `def fixture()` |
| class Boundaries | [源码:20](../../../tools/test_check_desktop_architecture.py#L20) · `unittest.TestCase` |
| function Boundaries.test_framework_trace_is_explicit_host_only_and_default_off | [源码:21](../../../tools/test_check_desktop_architecture.py#L21) · `def test_framework_trace_is_explicit_host_only_and_default_off(self)` |
| function Boundaries.test_native_audio_dependencies_remain_in_infrastructure | [源码:30](../../../tools/test_check_desktop_architecture.py#L30) · `def test_native_audio_dependencies_remain_in_infrastructure(self)` |
| function Boundaries.test_native_audio_versions_defaults_and_source_escapes_fail | [源码:37](../../../tools/test_check_desktop_architecture.py#L37) · `def test_native_audio_versions_defaults_and_source_escapes_fail(self)` |
| function Boundaries.test_reviewed_graph | [源码:46](../../../tools/test_check_desktop_architecture.py#L46) · `def test_reviewed_graph(self)` |
| function Boundaries.test_reverse_gpu_alias_and_optional_edges_fail | [源码:48](../../../tools/test_check_desktop_architecture.py#L48) · `def test_reverse_gpu_alias_and_optional_edges_fail(self)` |
| function Boundaries.test_versions_features_and_unregistered_packages_fail | [源码:54](../../../tools/test_check_desktop_architecture.py#L54) · `def test_versions_features_and_unregistered_packages_fail(self)` |
| function Boundaries.test_source_includes_io_and_runtime_escape_fail | [源码:62](../../../tools/test_check_desktop_architecture.py#L62) · `def test_source_includes_io_and_runtime_escape_fail(self)` |
| function Boundaries.test_audio_keeps_io_devices_and_visual_adapters_out | [源码:67](../../../tools/test_check_desktop_architecture.py#L67) · `def test_audio_keeps_io_devices_and_visual_adapters_out(self)` |
| function Boundaries.test_audio_edges_cannot_bypass_application_or_import_platforms | [源码:72](../../../tools/test_check_desktop_architecture.py#L72) · `def test_audio_edges_cannot_bypass_application_or_import_platforms(self)` |

## tools/test_check_event_loop.py

89 行；SHA256 `4cb2d44974263d73479f29357da97f4a4a812a178cfc82315624c5febfa2baf3`

导入：`copy`, `measure_event_loop_ui`, `measure_frame_dispatch`, `measure_gui_base`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| function row | [源码:8](../../../tools/test_check_event_loop.py#L8) · `def row(kind, a, b, identity)` |
| class EventLoopTests | [源码:12](../../../tools/test_check_event_loop.py#L12) · `unittest.TestCase` |
| function EventLoopTests.test_discrete_stage_counts_cannot_requalify_a_failed_clock_or_missing_record | [源码:13](../../../tools/test_check_event_loop.py#L13) · `def test_discrete_stage_counts_cannot_requalify_a_failed_clock_or_missing_record(self)` |
| function EventLoopTests.test_actual_delivery_counts_use_same_window_and_reject_missing_or_partial_data | [源码:33](../../../tools/test_check_event_loop.py#L33) · `def test_actual_delivery_counts_use_same_window_and_reject_missing_or_partial_data(self)` |
| function EventLoopTests.test_ordinary_wall_excludes_cross_stage_gaps_and_keeps_cpu_unknown | [源码:45](../../../tools/test_check_event_loop.py#L45) · `def test_ordinary_wall_excludes_cross_stage_gaps_and_keeps_cpu_unknown(self)` |
| function EventLoopTests.test_gaps_are_same_thread_intersections_not_wait_wall_or_parent_sums | [源码:60](../../../tools/test_check_event_loop.py#L60) · `def test_gaps_are_same_thread_intersections_not_wait_wall_or_parent_sums(self)` |
| function EventLoopTests.test_edge_clipping_and_missing_or_reentrant_handlers_fail | [源码:70](../../../tools/test_check_event_loop.py#L70) · `def test_edge_clipping_and_missing_or_reentrant_handlers_fail(self)` |
| function EventLoopTests.test_schema_3_handlers_do_not_reinterpret_older_capture_versions | [源码:79](../../../tools/test_check_event_loop.py#L79) · `def test_schema_3_handlers_do_not_reinterpret_older_capture_versions(self)` |

## tools/test_check_fault_rebuild.py

82 行；SHA256 `538c7e60405e0863573e453f5517187833fa7591b654186e606b63145055b155`

导入：`copy`, `measure_fault_rebuild`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| function samples | [源码:6](../../../tools/test_check_fault_rebuild.py#L6) · `def samples()` |
| function graphics | [源码:12](../../../tools/test_check_fault_rebuild.py#L12) · `def graphics()` |
| function audio | [源码:19](../../../tools/test_check_fault_rebuild.py#L19) · `def audio()` |
| class FaultRebuild | [源码:29](../../../tools/test_check_fault_rebuild.py#L29) · `unittest.TestCase` |
| function FaultRebuild.test_explicit_policy_requires_both_devices_and_cannot_reinterpret_legacy | [源码:30](../../../tools/test_check_fault_rebuild.py#L30) · `def test_explicit_policy_requires_both_devices_and_cannot_reinterpret_legacy(self)` |
| function FaultRebuild.test_component_success_cannot_accept_full_window_c4_or_cpu_goal | [源码:39](../../../tools/test_check_fault_rebuild.py#L39) · `def test_component_success_cannot_accept_full_window_c4_or_cpu_goal(self)` |
| function FaultRebuild.test_missing_fault_scope_and_physical_claim_are_rejected | [源码:45](../../../tools/test_check_fault_rebuild.py#L45) · `def test_missing_fault_scope_and_physical_claim_are_rejected(self)` |
| function FaultRebuild.test_lost_draw_owner_key_reuse_and_failure_to_join_fail | [源码:55](../../../tools/test_check_fault_rebuild.py#L55) · `def test_lost_draw_owner_key_reuse_and_failure_to_join_fail(self)` |
| function FaultRebuild.test_unknown_gpu_and_outside_phase_memory_peak_are_not_hidden | [源码:59](../../../tools/test_check_fault_rebuild.py#L59) · `def test_unknown_gpu_and_outside_phase_memory_peak_are_not_hidden(self)` |
| function FaultRebuild.test_each_rebuild_channel_latency_and_missing_frames_remain_hard | [源码:65](../../../tools/test_check_fault_rebuild.py#L65) · `def test_each_rebuild_channel_latency_and_missing_frames_remain_hard(self)` |
| function FaultRebuild.test_endpoint_release_anchor_and_exact_voice_evidence_cannot_be_substituted | [源码:73](../../../tools/test_check_fault_rebuild.py#L73) · `def test_endpoint_release_anchor_and_exact_voice_evidence_cannot_be_substituted(self)` |

## tools/test_check_frame_dispatch.py

97 行；SHA256 `68d1f504010004c644bde95a1687fd6f67181d01dd0c2879f671d75a25d2b157`

导入：`copy`, `hashlib`, `json`, `measure_frame_dispatch`, `measure_main_work`, `pathlib`, `tempfile`, `unittest`, `unittest.mock`

| 定义 | 位置／签名 |
| --- | --- |
| function fixture | [源码:7](../../../tools/test_check_frame_dispatch.py#L7) · `def fixture()` |
| function painter_fixture | [源码:15](../../../tools/test_check_frame_dispatch.py#L15) · `def painter_fixture()` |
| class FrameDispatchTests | [源码:24](../../../tools/test_check_frame_dispatch.py#L24) · `unittest.TestCase` |
| function FrameDispatchTests.test_schema_2_adds_painter_scopes_without_reinterpreting_schema_1 | [源码:25](../../../tools/test_check_frame_dispatch.py#L25) · `def test_schema_2_adds_painter_scopes_without_reinterpreting_schema_1(self)` |
| function FrameDispatchTests.test_painter_partition_checks_containment_before_exclusive_wall | [源码:29](../../../tools/test_check_frame_dispatch.py#L29) · `def test_painter_partition_checks_containment_before_exclusive_wall(self)` |
| function FrameDispatchTests.test_no_uac_mode_never_constructs_or_builds_collector_and_has_no_cpu_acceptance | [源码:39](../../../tools/test_check_frame_dispatch.py#L39) · `def test_no_uac_mode_never_constructs_or_builds_collector_and_has_no_cpu_acceptance(self)` |
| function FrameDispatchTests.test_no_uac_mode_never_constructs_or_builds_collector_and_has_no_cpu_acceptance.observed | [源码:45](../../../tools/test_check_frame_dispatch.py#L45) · `def observed(binary, path, *args, **kwargs)` |
| function FrameDispatchTests.test_old_flags_unchanged_and_new_modes_are_explicit | [源码:55](../../../tools/test_check_frame_dispatch.py#L55) · `def test_old_flags_unchanged_and_new_modes_are_explicit(self)` |
| function FrameDispatchTests.test_actual_api_regions_match_only_owned_main_and_keep_uncertainty | [源码:63](../../../tools/test_check_frame_dispatch.py#L63) · `def test_actual_api_regions_match_only_owned_main_and_keep_uncertainty(self)` |
| function FrameDispatchTests.test_missing_cleanup_reversed_duplicate_unknown_and_clock_drift_fail | [源码:71](../../../tools/test_check_frame_dispatch.py#L71) · `def test_missing_cleanup_reversed_duplicate_unknown_and_clock_drift_fail(self)` |
| function FrameDispatchTests.test_missing_normal_draw_or_overlapping_same_api_cannot_pass | [源码:78](../../../tools/test_check_frame_dispatch.py#L78) · `def test_missing_normal_draw_or_overlapping_same_api_cannot_pass(self)` |
| function FrameDispatchTests.test_partial_spans_are_clipped_and_parent_totals_are_not_subtracted | [源码:83](../../../tools/test_check_frame_dispatch.py#L83) · `def test_partial_spans_are_clipped_and_parent_totals_are_not_subtracted(self)` |
| function FrameDispatchTests.test_multiple_ui_passes_are_distinct_from_frames_and_sources_are_bounded | [源码:88](../../../tools/test_check_frame_dispatch.py#L88) · `def test_multiple_ui_passes_are_distinct_from_frames_and_sources_are_bounded(self)` |

## tools/test_check_framework_baseline.py

82 行；SHA256 `dec05e52ed95121c17039d65985cf913041c32f3d69807e834b337276e72c0c2`

导入：`measure_framework_baseline`, `measure_main_work`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| function fixture | [源码:6](../../../tools/test_check_framework_baseline.py#L6) · `def fixture(role)` |
| function plan | [源码:15](../../../tools/test_check_framework_baseline.py#L15) · `def plan()` |
| class FrameworkBaselineTests | [源码:24](../../../tools/test_check_framework_baseline.py#L24) · `unittest.TestCase` |
| function FrameworkBaselineTests.test_fixed_symmetric_plan_and_explicit_flags_leave_existing_defaults | [源码:25](../../../tools/test_check_framework_baseline.py#L25) · `def test_fixed_symmetric_plan_and_explicit_flags_leave_existing_defaults(self)` |
| function FrameworkBaselineTests.test_all_modes_keep_bootstrap_and_exercise_only_intended_measured_draw | [源码:33](../../../tools/test_check_framework_baseline.py#L33) · `def test_all_modes_keep_bootstrap_and_exercise_only_intended_measured_draw(self)` |
| function FrameworkBaselineTests.test_geometry_generation_count_and_policy_errors_are_not_normalized | [源码:41](../../../tools/test_check_framework_baseline.py#L41) · `def test_geometry_generation_count_and_policy_errors_are_not_normalized(self)` |
| function FrameworkBaselineTests.test_comparison_checks_device_geometry_audio_rate_focus_and_input | [源码:53](../../../tools/test_check_framework_baseline.py#L53) · `def test_comparison_checks_device_geometry_audio_rate_focus_and_input(self)` |
| function FrameworkBaselineTests.test_control_success_cannot_cover_editor_failure_or_incomplete_plan | [源码:66](../../../tools/test_check_framework_baseline.py#L66) · `def test_control_success_cannot_cover_editor_failure_or_incomplete_plan(self)` |
| function FrameworkBaselineTests.test_missing_metadata_or_missing_acceptance_predicates_cannot_pass | [源码:74](../../../tools/test_check_framework_baseline.py#L74) · `def test_missing_metadata_or_missing_acceptance_predicates_cannot_pass(self)` |

## tools/test_check_gpu_path.py

46 行；SHA256 `5917eec5307d4a3d00ae86cc958c2a63337e6f55230e1d277d9eb4a764e3918e`

导入：`copy`, `measure_gpu_path`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| function markers | [源码:5](../../../tools/test_check_gpu_path.py#L5) · `def markers()` |
| function upload | [源码:11](../../../tools/test_check_gpu_path.py#L11) · `def upload()` |
| class GpuPathTests | [源码:19](../../../tools/test_check_gpu_path.py#L19) · `unittest.TestCase` |
| function GpuPathTests.test_actual_regions_are_named_without_submit_or_present_attribution | [源码:20](../../../tools/test_check_gpu_path.py#L20) · `def test_actual_regions_are_named_without_submit_or_present_attribution(self)` |
| function GpuPathTests.test_missing_late_reversed_duplicate_and_overflow_edges_fail | [源码:24](../../../tools/test_check_gpu_path.py#L24) · `def test_missing_late_reversed_duplicate_and_overflow_edges_fail(self)` |
| function GpuPathTests.test_upload_completion_can_precede_submit_return_but_not_begin | [源码:31](../../../tools/test_check_gpu_path.py#L31) · `def test_upload_completion_can_precede_submit_return_but_not_begin(self)` |
| function GpuPathTests.test_upload_plan_and_resource_release_cannot_be_silently_dropped | [源码:36](../../../tools/test_check_gpu_path.py#L36) · `def test_upload_plan_and_resource_release_cannot_be_silently_dropped(self)` |

## tools/test_check_gui_base.py

50 行；SHA256 `f09aab3f605bdf1acb10ff7b5fcea03fb9063ba666f19acfa7bea3a839ea8eb2`

导入：`copy`, `measure_gui_base`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| class GuiBaseTests | [源码:5](../../../tools/test_check_gui_base.py#L5) · `unittest.TestCase` |
| function GuiBaseTests.rows | [源码:6](../../../tools/test_check_gui_base.py#L6) · `def rows(self)` |
| function GuiBaseTests.test_fixed_abba_retains_failed_baselines_without_using_them_as_candidate_acceptance | [源码:9](../../../tools/test_check_gui_base.py#L9) · `def test_fixed_abba_retains_failed_baselines_without_using_them_as_candidate_acceptance(self)` |
| function GuiBaseTests.test_candidate_failure_or_missing_result_cannot_pass | [源码:11](../../../tools/test_check_gui_base.py#L11) · `def test_candidate_failure_or_missing_result_cannot_pass(self)` |
| function GuiBaseTests.test_reordered_or_extra_results_are_not_cherry_picked | [源码:14](../../../tools/test_check_gui_base.py#L14) · `def test_reordered_or_extra_results_are_not_cherry_picked(self)` |
| function GuiBaseTests.test_missing_predicates_do_not_authorize_candidate | [源码:17](../../../tools/test_check_gui_base.py#L17) · `def test_missing_predicates_do_not_authorize_candidate(self)` |
| function GuiBaseTests.test_rejected_boundary_clock_remains_a_failed_comparison_without_partial_selection | [源码:19](../../../tools/test_check_gui_base.py#L19) · `def test_rejected_boundary_clock_remains_a_failed_comparison_without_partial_selection(self)` |
| function GuiBaseTests.plan | [源码:23](../../../tools/test_check_gui_base.py#L23) · `def plan(self)` |
| function GuiBaseTests.test_ordinary_comparison_rejects_geometry_device_rate_input_and_focus_mismatch | [源码:29](../../../tools/test_check_gui_base.py#L29) · `def test_ordinary_comparison_rejects_geometry_device_rate_input_and_focus_mismatch(self)` |
| function GuiBaseTests.test_incomplete_or_reordered_comparison_is_rejected | [源码:40](../../../tools/test_check_gui_base.py#L40) · `def test_incomplete_or_reordered_comparison_is_rejected(self)` |

## tools/test_check_gui_candidate.py

65 行；SHA256 `acd1e66821308799d31f41dd01c6dee4bd8dfb341e08187db61e8f72b4f1060e`

导入：`measure_gui_candidate`, `test_check_worker_native`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| function report | [源码:6](../../../tools/test_check_gui_candidate.py#L6) · `def report()` |
| class GuiCandidateChecks | [源码:18](../../../tools/test_check_gui_candidate.py#L18) · `unittest.TestCase` |
| function GuiCandidateChecks.test_fixed_plan_and_distinct_native_vs_gui_observation | [源码:19](../../../tools/test_check_gui_candidate.py#L19) · `def test_fixed_plan_and_distinct_native_vs_gui_observation(self)` |
| function GuiCandidateChecks.test_wrong_scope_physical_claim_or_missing_gui_rejected | [源码:24](../../../tools/test_check_gui_candidate.py#L24) · `def test_wrong_scope_physical_claim_or_missing_gui_rejected(self)` |
| function GuiCandidateChecks.test_identity_missing_or_invalid_timing_rejected | [源码:29](../../../tools/test_check_gui_candidate.py#L29) · `def test_identity_missing_or_invalid_timing_rejected(self)` |
| function GuiCandidateChecks.test_original_latency_underrun_resource_and_cleanup_limits_kept | [源码:41](../../../tools/test_check_gui_candidate.py#L41) · `def test_original_latency_underrun_resource_and_cleanup_limits_kept(self)` |
| function GuiCandidateChecks.test_lifecycle_cannot_be_replaced_by_success_flags | [源码:52](../../../tools/test_check_gui_candidate.py#L52) · `def test_lifecycle_cannot_be_replaced_by_success_flags(self)` |

## tools/test_check_gui_cpu.py

137 行；SHA256 `1448cff978416498ba9114aa7eb446063e5ab5a95c2ff11d74f02c475039336f`

导入：`ctypes`, `measure_gui_cpu`, `os`, `types`, `unittest`, `unittest.mock`

| 定义 | 位置／签名 |
| --- | --- |
| function report | [源码:9](../../../tools/test_check_gui_cpu.py#L9) · `def report()` |
| function sample | [源码:16](../../../tools/test_check_gui_cpu.py#L16) · `def sample(at, cycles, threads)` |
| class GuiCpuChecks | [源码:23](../../../tools/test_check_gui_cpu.py#L23) · `unittest.TestCase` |
| function GuiCpuChecks.test_start_mapping_uses_half_open_ranges_and_does_not_guess_ambiguous_origins | [源码:24](../../../tools/test_check_gui_cpu.py#L24) · `def test_start_mapping_uses_half_open_ranges_and_does_not_guess_ambiguous_origins(self)` |
| function GuiCpuChecks.test_cycles_stay_cycles_and_counter_gap_is_not_deducted_or_clamped | [源码:32](../../../tools/test_check_gui_cpu.py#L32) · `def test_cycles_stay_cycles_and_counter_gap_is_not_deducted_or_clamped(self)` |
| function GuiCpuChecks.test_reversed_cycles_times_or_group_changes_reject | [源码:41](../../../tools/test_check_gui_cpu.py#L41) · `def test_reversed_cycles_times_or_group_changes_reject(self)` |
| function GuiCpuChecks.test_gui_framework_and_app_wall_remain_separate_from_cpu | [源码:51](../../../tools/test_check_gui_cpu.py#L51) · `def test_gui_framework_and_app_wall_remain_separate_from_cpu(self)` |
| function GuiCpuChecks.test_gui_missing_scope_invalid_or_insufficient_samples_reject | [源码:57](../../../tools/test_check_gui_cpu.py#L57) · `def test_gui_missing_scope_invalid_or_insufficient_samples_reject(self)` |
| function GuiCpuChecks.test_cycle_api_failure_does_not_become_zero_measurement | [源码:68](../../../tools/test_check_gui_cpu.py#L68) · `def test_cycle_api_failure_does_not_become_zero_measurement(self)` |
| function GuiCpuChecks.test_cycle_api_failure_does_not_become_zero_measurement.fill | [源码:71](../../../tools/test_check_gui_cpu.py#L71) · `def fill(handle, ptr)` |
| function GuiCpuChecks.module_api | [源码:74](../../../tools/test_check_gui_cpu.py#L74) · `def module_api(self, rows, error=18)` |
| function GuiCpuChecks.module_api.fill | [源码:76](../../../tools/test_check_gui_cpu.py#L76) · `def fill(handle, ptr)` |
| function GuiCpuChecks.test_module_snapshot_only_records_owned_basenames_and_closes | [源码:87](../../../tools/test_check_gui_cpu.py#L87) · `def test_module_snapshot_only_records_owned_basenames_and_closes(self)` |
| function GuiCpuChecks.test_module_pid_race_limit_and_enumeration_error_do_not_claim_mapping | [源码:94](../../../tools/test_check_gui_cpu.py#L94) · `def test_module_pid_race_limit_and_enumeration_error_do_not_claim_mapping(self)` |
| function GuiCpuChecks.test_optional_start_api_is_unknown_when_absent_not_product_failure | [源码:104](../../../tools/test_check_gui_cpu.py#L104) · `def test_optional_start_api_is_unknown_when_absent_not_product_failure(self)` |
| function GuiCpuChecks.test_optional_start_api_is_unknown_when_absent_not_product_failure.description | [源码:107](../../../tools/test_check_gui_cpu.py#L107) · `def description(handle, ptr)` |
| function GuiCpuChecks.test_optional_start_api_is_unknown_when_absent_not_product_failure.times | [源码:108](../../../tools/test_check_gui_cpu.py#L108) · `def times(handle, created, exited, kernel, user)` |
| function GuiCpuChecks.test_fixed_same_profiled_binary_control_plan_does_not_retry | [源码:114](../../../tools/test_check_gui_cpu.py#L114) · `def test_fixed_same_profiled_binary_control_plan_does_not_retry(self)` |
| function GuiCpuChecks.test_backend_override_restores_local_environment_even_after_failure | [源码:118](../../../tools/test_check_gui_cpu.py#L118) · `def test_backend_override_restores_local_environment_even_after_failure(self)` |
| function GuiCpuChecks.test_actual_adapter_is_required_instead_of_assuming_requested_backend | [源码:130](../../../tools/test_check_gui_cpu.py#L130) · `def test_actual_adapter_is_required_instead_of_assuming_requested_backend(self)` |

## tools/test_check_independent_cpu.py

136 行；SHA256 `fc3da03c5a70262e1b729dfe2c65fcfdb2809af12a54576ed0defc2ec13c8769`

导入：`copy`, `measure_cpu_accounting`, `measure_independent_cpu`, `unittest`, `unittest.mock`

| 定义 | 位置／签名 |
| --- | --- |
| function trace | [源码:8](../../../tools/test_check_independent_cpu.py#L8) · `def trace(events)` |
| class IndependentCpuTests | [源码:14](../../../tools/test_check_independent_cpu.py#L14) · `unittest.TestCase` |
| function IndependentCpuTests.test_v2_reused_tid_has_independent_switch_pairs | [源码:15](../../../tools/test_check_independent_cpu.py#L15) · `def test_v2_reused_tid_has_independent_switch_pairs(self)` |
| function IndependentCpuTests.test_v2_lifetime_cannot_close_another_lifetime_or_change_identity | [源码:20](../../../tools/test_check_independent_cpu.py#L20) · `def test_v2_lifetime_cannot_close_another_lifetime_or_change_identity(self)` |
| function IndependentCpuTests.test_v2_missing_or_duplicate_metadata_is_rejected | [源码:25](../../../tools/test_check_independent_cpu.py#L25) · `def test_v2_missing_or_duplicate_metadata_is_rejected(self)` |
| function IndependentCpuTests.test_exact_switches_clip_to_requested_window | [源码:30](../../../tools/test_check_independent_cpu.py#L30) · `def test_exact_switches_clip_to_requested_window(self)` |
| function IndependentCpuTests.test_parallel_threads_sum_without_dividing_by_machine_core_count | [源码:36](../../../tools/test_check_independent_cpu.py#L36) · `def test_parallel_threads_sum_without_dividing_by_machine_core_count(self)` |
| function IndependentCpuTests.test_pid_filter_does_not_add_another_owned_process | [源码:40](../../../tools/test_check_independent_cpu.py#L40) · `def test_pid_filter_does_not_add_another_owned_process(self)` |
| function IndependentCpuTests.test_equal_timestamp_migration_is_order_independent_across_cpus | [源码:44](../../../tools/test_check_independent_cpu.py#L44) · `def test_equal_timestamp_migration_is_order_independent_across_cpus(self)` |
| function IndependentCpuTests.test_small_cross_cpu_switch_overlap_is_preserved_not_discounted | [源码:48](../../../tools/test_check_independent_cpu.py#L48) · `def test_small_cross_cpu_switch_overlap_is_preserved_not_discounted(self)` |
| function IndependentCpuTests.test_unmatched_or_unclosed_switches_fail_closed | [源码:56](../../../tools/test_check_independent_cpu.py#L56) · `def test_unmatched_or_unclosed_switches_fail_closed(self)` |
| function IndependentCpuTests.test_switch_out_must_match_cpu_and_identity_without_duplicate_same_cpu_entry | [源码:60](../../../tools/test_check_independent_cpu.py#L60) · `def test_switch_out_must_match_cpu_and_identity_without_duplicate_same_cpu_entry(self)` |
| function IndependentCpuTests.test_same_cpu_reversal_is_rejected_not_sorted_away | [源码:64](../../../tools/test_check_independent_cpu.py#L64) · `def test_same_cpu_reversal_is_rejected_not_sorted_away(self)` |
| function IndependentCpuTests.test_malformed_or_unbounded_input_rejected | [源码:67](../../../tools/test_check_independent_cpu.py#L67) · `def test_malformed_or_unbounded_input_rejected(self)` |
| function IndependentCpuTests.test_all_loss_timeout_permission_and_capacity_errors_invalidate_meter | [源码:74](../../../tools/test_check_independent_cpu.py#L74) · `def test_all_loss_timeout_permission_and_capacity_errors_invalidate_meter(self)` |
| function IndependentCpuTests.test_invalid_window_cannot_be_used_as_zero_cpu | [源码:81](../../../tools/test_check_independent_cpu.py#L81) · `def test_invalid_window_cannot_be_used_as_zero_cpu(self)` |
| function IndependentCpuTests.reference | [源码:85](../../../tools/test_check_independent_cpu.py#L85) · `def reference(self)` |
| function IndependentCpuTests.test_continuous_periodic_and_phase_reference_accept_independent_time_without_repairing_raw_zero | [源码:100](../../../tools/test_check_independent_cpu.py#L100) · `def test_continuous_periodic_and_phase_reference_accept_independent_time_without_repairing_raw_zero(self)` |
| function IndependentCpuTests.test_aliased_periodic_measurement_cannot_pass_reference | [源码:107](../../../tools/test_check_independent_cpu.py#L107) · `def test_aliased_periodic_measurement_cannot_pass_reference(self)` |
| function IndependentCpuTests.test_phase_identity_timer_and_clock_are_checked | [源码:111](../../../tools/test_check_independent_cpu.py#L111) · `def test_phase_identity_timer_and_clock_are_checked(self)` |
| function IndependentCpuTests.test_missing_or_mismatched_pid_is_not_a_zero_cost_pass | [源码:119](../../../tools/test_check_independent_cpu.py#L119) · `def test_missing_or_mismatched_pid_is_not_a_zero_cost_pass(self)` |
| function IndependentCpuTests.test_product_retains_original_failure_and_independent_15_percent_gate | [源码:125](../../../tools/test_check_independent_cpu.py#L125) · `def test_product_retains_original_failure_and_independent_15_percent_gate(self)` |

## tools/test_check_main_work.py

197 行；SHA256 `0fc9374246dd38ae5791b33254bd6abad72157c17e1ee805c3881ebaaa1ca9a3`

导入：`copy`, `measure_independent_cpu`, `measure_main_work`, `os`, `pathlib`, `subprocess`, `tempfile`, `test_check_independent_cpu`, `unittest`, `unittest.mock`

| 定义 | 位置／签名 |
| --- | --- |
| class MainWorkTests | [源码:10](../../../tools/test_check_main_work.py#L10) · `unittest.TestCase` |
| function MainWorkTests.test_fixed_plan_brackets_timing_instrumentation | [源码:11](../../../tools/test_check_main_work.py#L11) · `def test_fixed_plan_brackets_timing_instrumentation(self)` |
| function MainWorkTests.test_missed_reference_pulse_blocks_later_admission | [源码:14](../../../tools/test_check_main_work.py#L14) · `def test_missed_reference_pulse_blocks_later_admission(self)` |
| function MainWorkTests.test_failed_calibration_retains_all_diagnostic_rows_and_cannot_pass | [源码:20](../../../tools/test_check_main_work.py#L20) · `def test_failed_calibration_retains_all_diagnostic_rows_and_cannot_pass(self)` |
| function MainWorkTests.test_incomplete_or_extra_report_plan_is_rejected | [源码:28](../../../tools/test_check_main_work.py#L28) · `def test_incomplete_or_extra_report_plan_is_rejected(self)` |
| function MainWorkTests.test_failed_attribution_does_not_drop_remaining_captured_rows | [源码:32](../../../tools/test_check_main_work.py#L32) · `def test_failed_attribution_does_not_drop_remaining_captured_rows(self)` |
| function MainWorkTests.test_both_real_ordinary_token_types_are_accepted_not_elevated | [源码:40](../../../tools/test_check_main_work.py#L40) · `def test_both_real_ordinary_token_types_are_accepted_not_elevated(self)` |
| function MainWorkTests.permission_fixture | [源码:44](../../../tools/test_check_main_work.py#L44) · `def permission_fixture(self)` |
| function MainWorkTests.test_permission_requires_native_and_parent_and_each_actual_target | [源码:50](../../../tools/test_check_main_work.py#L50) · `def test_permission_requires_native_and_parent_and_each_actual_target(self)` |
| function MainWorkTests.test_missing_or_pid_mismatched_target_is_not_permission_success | [源码:57](../../../tools/test_check_main_work.py#L57) · `def test_missing_or_pid_mismatched_target_is_not_permission_success(self)` |
| function MainWorkTests.test_reused_process_pid_is_not_silently_merged | [源码:61](../../../tools/test_check_main_work.py#L61) · `def test_reused_process_pid_is_not_silently_merged(self)` |
| function MainWorkTests.test_same_tid_in_different_owned_processes_has_separate_intervals | [源码:65](../../../tools/test_check_main_work.py#L65) · `def test_same_tid_in_different_owned_processes_has_separate_intervals(self)` |
| function MainWorkTests.fixture | [源码:71](../../../tools/test_check_main_work.py#L71) · `def fixture(self)` |
| function MainWorkTests.costs | [源码:78](../../../tools/test_check_main_work.py#L78) · `def costs(self, t, r, b)` |
| function MainWorkTests.test_exact_partition_uses_cpu_intersections_not_wall_subtraction | [源码:81](../../../tools/test_check_main_work.py#L81) · `def test_exact_partition_uses_cpu_intersections_not_wall_subtraction(self)` |
| function MainWorkTests.detailed_fixture | [源码:87](../../../tools/test_check_main_work.py#L87) · `def detailed_fixture(self)` |
| function MainWorkTests.test_detailed_cpu_partition_rolls_up_without_double_counting | [源码:94](../../../tools/test_check_main_work.py#L94) · `def test_detailed_cpu_partition_rolls_up_without_double_counting(self)` |
| function MainWorkTests.test_poll_wait_is_not_counted_as_cpu_and_parent_windows_stay_intact | [源码:105](../../../tools/test_check_main_work.py#L105) · `def test_poll_wait_is_not_counted_as_cpu_and_parent_windows_stay_intact(self)` |
| function MainWorkTests.test_schema_extension_does_not_reinterpret_legacy_records | [源码:112](../../../tools/test_check_main_work.py#L112) · `def test_schema_extension_does_not_reinterpret_legacy_records(self)` |
| function MainWorkTests.test_panel_boundaries_roll_up_to_ui_with_no_new_cpu_allowance | [源码:121](../../../tools/test_check_main_work.py#L121) · `def test_panel_boundaries_roll_up_to_ui_with_no_new_cpu_allowance(self)` |
| function MainWorkTests.test_boundary_unknown_or_capacity_or_clock_reversal_fails | [源码:134](../../../tools/test_check_main_work.py#L134) · `def test_boundary_unknown_or_capacity_or_clock_reversal_fails(self)` |
| function MainWorkTests.test_text_detail_schema_has_explicit_capacity_and_conserves_ui_cpu | [源码:139](../../../tools/test_check_main_work.py#L139) · `def test_text_detail_schema_has_explicit_capacity_and_conserves_ui_cpu(self)` |
| function MainWorkTests.test_clock_pair_delay_and_time_adjustment_are_not_cpu_corrections | [源码:151](../../../tools/test_check_main_work.py#L151) · `def test_clock_pair_delay_and_time_adjustment_are_not_cpu_corrections(self)` |
| function MainWorkTests.test_large_epoch_mapping_keeps_integer_precision | [源码:155](../../../tools/test_check_main_work.py#L155) · `def test_large_epoch_mapping_keeps_integer_precision(self)` |
| function MainWorkTests.test_all_clock_brackets_constrain_offset_including_preempted_pair | [源码:159](../../../tools/test_check_main_work.py#L159) · `def test_all_clock_brackets_constrain_offset_including_preempted_pair(self)` |
| function MainWorkTests.test_monotonic_disagreement_rejects_system_clock_fallback | [源码:165](../../../tools/test_check_main_work.py#L165) · `def test_monotonic_disagreement_rejects_system_clock_fallback(self)` |
| function MainWorkTests.test_incomplete_window_and_ambiguous_main_cannot_be_attributed | [源码:169](../../../tools/test_check_main_work.py#L169) · `def test_incomplete_window_and_ambiguous_main_cannot_be_attributed(self)` |
| function MainWorkTests.test_partition_mismatch_does_not_hide_unaccounted_main_cost | [源码:175](../../../tools/test_check_main_work.py#L175) · `def test_partition_mismatch_does_not_hide_unaccounted_main_cost(self)` |
| function MainWorkTests.test_failed_admission_does_not_release_target_and_kills_owned_child | [源码:179](../../../tools/test_check_main_work.py#L179) · `def test_failed_admission_does_not_release_target_and_kills_owned_child(self)` |
| function MainWorkTests.test_timeout_cleans_owned_target_without_releasing_another | [源码:187](../../../tools/test_check_main_work.py#L187) · `def test_timeout_cleans_owned_target_without_releasing_another(self)` |

## tools/test_check_mesh_replay.py

47 行；SHA256 `c2eaaab1afb3fea7098ee9182ca25d6061b2779b458bbea23b5b82c910237040`

导入：`copy`, `measure_mesh_replay`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| function fixture | [源码:5](../../../tools/test_check_mesh_replay.py#L5) · `def fixture()` |
| class MeshReplayTests | [源码:16](../../../tools/test_check_mesh_replay.py#L16) · `unittest.TestCase` |
| function MeshReplayTests.test_fixed_fresh_samples_balanced_cohorts_and_separate_raw_timings | [源码:17](../../../tools/test_check_mesh_replay.py#L17) · `def test_fixed_fresh_samples_balanced_cohorts_and_separate_raw_timings(self)` |
| function MeshReplayTests.test_reordered_or_incomplete_fresh_output_plan_cannot_pass | [源码:25](../../../tools/test_check_mesh_replay.py#L25) · `def test_reordered_or_incomplete_fresh_output_plan_cannot_pass(self)` |
| function MeshReplayTests.test_failed_exact_checks_or_uncleared_scratch_or_mismatched_shutdown_rejected | [源码:30](../../../tools/test_check_mesh_replay.py#L30) · `def test_failed_exact_checks_or_uncleared_scratch_or_mismatched_shutdown_rejected(self)` |
| function MeshReplayTests.test_mesh_and_atlas_bounds_empty_and_copy_sizes_checked | [源码:36](../../../tools/test_check_mesh_replay.py#L36) · `def test_mesh_and_atlas_bounds_empty_and_copy_sizes_checked(self)` |
| function MeshReplayTests.test_changed_bracket_or_invalid_time_partition_is_not_normalized | [源码:42](../../../tools/test_check_mesh_replay.py#L42) · `def test_changed_bracket_or_invalid_time_partition_is_not_normalized(self)` |

## tools/test_check_painter_cpu.py

86 行；SHA256 `206e66d3cb1bd37be96850e298d8cc34828dec14fa8e3db89b0101fce21975f8`

导入：`copy`, `json`, `measure_main_work`, `measure_painter_cpu`, `pathlib`, `tempfile`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| function row | [源码:10](../../../tools/test_check_painter_cpu.py#L10) · `def row(kind, a, b)` |
| function fixture | [源码:14](../../../tools/test_check_painter_cpu.py#L14) · `def fixture()` |
| function compute | [源码:20](../../../tools/test_check_painter_cpu.py#L20) · `def compute(rows=None, intervals=None, begin=50, end=1000)` |
| class PainterCpuTests | [源码:25](../../../tools/test_check_painter_cpu.py#L25) · `unittest.TestCase` |
| function PainterCpuTests.test_incomplete_uncalibrated_or_unowned_capture_is_not_analyzed | [源码:26](../../../tools/test_check_painter_cpu.py#L26) · `def test_incomplete_uncalibrated_or_unowned_capture_is_not_analyzed(self)` |
| function PainterCpuTests.test_nested_cpu_is_exclusive_not_sum_of_parent_and_child | [源码:40](../../../tools/test_check_painter_cpu.py#L40) · `def test_nested_cpu_is_exclusive_not_sum_of_parent_and_child(self)` |
| function PainterCpuTests.test_clipped_edge_scopes_keep_exact_window_and_wait_is_not_cpu | [源码:51](../../../tools/test_check_painter_cpu.py#L51) · `def test_clipped_edge_scopes_keep_exact_window_and_wait_is_not_cpu(self)` |
| function PainterCpuTests.test_wrong_pid_tid_ignored_and_conservative_overlap_never_deducted | [源码:58](../../../tools/test_check_painter_cpu.py#L58) · `def test_wrong_pid_tid_ignored_and_conservative_overlap_never_deducted(self)` |
| function PainterCpuTests.test_crossing_same_kind_and_wrong_parent_fail | [源码:64](../../../tools/test_check_painter_cpu.py#L64) · `def test_crossing_same_kind_and_wrong_parent_fail(self)` |
| function PainterCpuTests.test_zero_duration_and_reordered_input_do_not_steal_ownership | [源码:69](../../../tools/test_check_painter_cpu.py#L69) · `def test_zero_duration_and_reordered_input_do_not_steal_ownership(self)` |
| function PainterCpuTests.test_invalid_window_and_unknown_kind_fail | [源码:76](../../../tools/test_check_painter_cpu.py#L76) · `def test_invalid_window_and_unknown_kind_fail(self)` |

## tools/test_check_rebuild_attribution.py

42 行；SHA256 `17bc35c61bf8fe339a27a09a324263bbe9811e695652cc576f06066351643eb3`

导入：`measure_rebuild_attribution`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| class RebuildAttribution | [源码:5](../../../tools/test_check_rebuild_attribution.py#L5) · `unittest.TestCase` |
| function RebuildAttribution.test_fixed_plan_cannot_select_just_low_result | [源码:6](../../../tools/test_check_rebuild_attribution.py#L6) · `def test_fixed_plan_cannot_select_just_low_result(self)` |
| function RebuildAttribution.test_checkpoint_rejects_late_duplicate_and_wrong_stage | [源码:10](../../../tools/test_check_rebuild_attribution.py#L10) · `def test_checkpoint_rejects_late_duplicate_and_wrong_stage(self)` |
| function RebuildAttribution.test_unavailable_allocator_does_not_become_zero | [源码:16](../../../tools/test_check_rebuild_attribution.py#L16) · `def test_unavailable_allocator_does_not_become_zero(self)` |
| function RebuildAttribution.test_retention_requires_exact_region_identity_not_just_same_address | [源码:22](../../../tools/test_check_rebuild_attribution.py#L22) · `def test_retention_requires_exact_region_identity_not_just_same_address(self)` |
| function RebuildAttribution.test_timeline_phase_change_during_query_keeps_both_owners_and_clock_bracket | [源码:29](../../../tools/test_check_rebuild_attribution.py#L29) · `def test_timeline_phase_change_during_query_keeps_both_owners_and_clock_bracket(self)` |
| function RebuildAttribution.test_timeline_unknown_memory_reverse_clock_and_overflow_cannot_be_accepted | [源码:35](../../../tools/test_check_rebuild_attribution.py#L35) · `def test_timeline_unknown_memory_reverse_clock_and_overflow_cannot_be_accepted(self)` |

## tools/test_check_rebuild_policy.py

58 行；SHA256 `0598fa88861ce49659af49867c09589ab6b43b697d8262fe849a44c28ae8de12`

导入：`copy`, `measure_rebuild_attribution`, `measure_rebuild_policy`, `pathlib`, `tempfile`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| function fixture | [源码:9](../../../tools/test_check_rebuild_policy.py#L9) · `def fixture(enabled)` |
| function fixture.allocator | [源码:11](../../../tools/test_check_rebuild_policy.py#L11) · `def allocator(reserved, live=False)` |
| class RebuildPolicy | [源码:23](../../../tools/test_check_rebuild_policy.py#L23) · `unittest.TestCase` |
| function RebuildPolicy.test_both_incarnations_are_compared_and_all_fixed_controls_required | [源码:24](../../../tools/test_check_rebuild_policy.py#L24) · `def test_both_incarnations_are_compared_and_all_fixed_controls_required(self)` |
| function RebuildPolicy.test_unavailable_pool_wrong_policy_and_unreleased_owner_are_rejected | [源码:31](../../../tools/test_check_rebuild_policy.py#L31) · `def test_unavailable_pool_wrong_policy_and_unreleased_owner_are_rejected(self)` |
| function RebuildPolicy.test_empty_pool_retention_does_not_mean_readback_object_is_alive | [源码:41](../../../tools/test_check_rebuild_policy.py#L41) · `def test_empty_pool_retention_does_not_mean_readback_object_is_alive(self)` |
| function RebuildPolicy.test_pixels_must_be_complete_equal_and_sha_is_content_based | [源码:47](../../../tools/test_check_rebuild_policy.py#L47) · `def test_pixels_must_be_complete_equal_and_sha_is_content_based(self)` |

## tools/test_check_repaint.py

51 行；SHA256 `a268a9204592961450c83fcfb419ed61e0d7a8c9879b1c04560e92b5cdc6ef0d`

导入：`measure_repaint`, `os`, `test_check_main_work`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| class RepaintTests | [源码:5](../../../tools/test_check_repaint.py#L5) · `unittest.TestCase` |
| function RepaintTests.test_no_product_admission_for_missing_or_failed_strict_reference | [源码:6](../../../tools/test_check_repaint.py#L6) · `def test_no_product_admission_for_missing_or_failed_strict_reference(self)` |
| function RepaintTests.fixture | [源码:11](../../../tools/test_check_repaint.py#L11) · `def fixture(self)` |
| function RepaintTests.test_requests_causes_and_input_counters_stay_distinct_and_in_original_window | [源码:20](../../../tools/test_check_repaint.py#L20) · `def test_requests_causes_and_input_counters_stay_distinct_and_in_original_window(self)` |
| function RepaintTests.test_missing_overflow_or_pending_cannot_be_called_complete | [源码:27](../../../tools/test_check_repaint.py#L27) · `def test_missing_overflow_or_pending_cannot_be_called_complete(self)` |
| function RepaintTests.test_duplicate_pass_and_unknown_source_are_rejected | [源码:32](../../../tools/test_check_repaint.py#L32) · `def test_duplicate_pass_and_unknown_source_are_rejected(self)` |
| function RepaintTests.test_private_absolute_source_path_is_rejected | [源码:38](../../../tools/test_check_repaint.py#L38) · `def test_private_absolute_source_path_is_rejected(self)` |
| function RepaintTests.test_permission_rejects_elevated_unowned_or_wrong_session | [源码:42](../../../tools/test_check_repaint.py#L42) · `def test_permission_rejects_elevated_unowned_or_wrong_session(self)` |

## tools/test_check_repository.py

70 行；SHA256 `6918c19beecc9bce89fbf0fe989d8ebc1869d29db6b35cebaff0a8be47947e88`

导入：`check_repository`, `contextlib`, `io`, `unittest`, `unittest.mock`

| 定义 | 位置／签名 |
| --- | --- |
| class RepositoryChecks | [源码:9](../../../tools/test_check_repository.py#L9) · `unittest.TestCase` |
| function RepositoryChecks.test_private_paths_are_rejected_even_if_already_tracked | [源码:10](../../../tools/test_check_repository.py#L10) · `def test_private_paths_are_rejected_even_if_already_tracked(self)` |
| function RepositoryChecks.test_source_locks_and_self_authored_fixtures_are_allowed | [源码:16](../../../tools/test_check_repository.py#L16) · `def test_source_locks_and_self_authored_fixtures_are_allowed(self)` |
| function RepositoryChecks.test_common_secrets_report_rule_and_line_without_values | [源码:22](../../../tools/test_check_repository.py#L22) · `def test_common_secrets_report_rule_and_line_without_values(self)` |
| function RepositoryChecks.test_placeholders_and_evidence_hashes_do_not_trigger | [源码:32](../../../tools/test_check_repository.py#L32) · `def test_placeholders_and_evidence_hashes_do_not_trigger(self)` |
| function RepositoryChecks.test_staged_scan_reads_index_bytes_not_clean_worktree | [源码:35](../../../tools/test_check_repository.py#L35) · `def test_staged_scan_reads_index_bytes_not_clean_worktree(self)` |
| function RepositoryChecks.test_staged_scan_reads_index_bytes_not_clean_worktree.fake_git | [源码:36](../../../tools/test_check_repository.py#L36) · `def fake_git(*args)` |
| function RepositoryChecks.test_staged_deletions_are_not_read | [源码:49](../../../tools/test_check_repository.py#L49) · `def test_staged_deletions_are_not_read(self)` |
| function RepositoryChecks.test_invalid_path_is_rejected_without_reading_outside_repository | [源码:53](../../../tools/test_check_repository.py#L53) · `def test_invalid_path_is_rejected_without_reading_outside_repository(self)` |
| function RepositoryChecks.test_symlink_is_rejected_without_following_target | [源码:57](../../../tools/test_check_repository.py#L57) · `def test_symlink_is_rejected_without_following_target(self)` |
| function RepositoryChecks.test_failure_output_redacts_secret | [源码:61](../../../tools/test_check_repository.py#L61) · `def test_failure_output_redacts_secret(self)` |

## tools/test_check_resource_cycles.py

109 行；SHA256 `a690673cb5cc9c5ef0101987a9319fbf7d89d34d157863261a2ff613d90f3121`

导入：`copy`, `json`, `measure_resource_cycles`, `pathlib`, `tempfile`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| function fixture | [源码:8](../../../tools/test_check_resource_cycles.py#L8) · `def fixture()` |
| class ResourceCycles | [源码:37](../../../tools/test_check_resource_cycles.py#L37) · `unittest.TestCase` |
| function ResourceCycles.test_production_policy_requires_its_own_identity_and_unchanged_budgets | [源码:38](../../../tools/test_check_resource_cycles.py#L38) · `def test_production_policy_requires_its_own_identity_and_unchanged_budgets(self)` |
| function ResourceCycles.test_full_plan_keeps_cpu_goal_and_trend_acceptance_separate | [源码:47](../../../tools/test_check_resource_cycles.py#L47) · `def test_full_plan_keeps_cpu_goal_and_trend_acceptance_separate(self)` |
| function ResourceCycles.test_missing_or_reused_project_cycle_is_not_twenty_full_reopens | [源码:53](../../../tools/test_check_resource_cycles.py#L53) · `def test_missing_or_reused_project_cycle_is_not_twenty_full_reopens(self)` |
| function ResourceCycles.test_no_live_bytes_pending_work_or_missing_join_may_claim_release | [源码:61](../../../tools/test_check_resource_cycles.py#L61) · `def test_no_live_bytes_pending_work_or_missing_join_may_claim_release(self)` |
| function ResourceCycles.test_zero_missing_and_latency_are_still_hard_per_cycle | [源码:69](../../../tools/test_check_resource_cycles.py#L69) · `def test_zero_missing_and_latency_are_still_hard_per_cycle(self)` |
| function ResourceCycles.test_unknown_gpu_and_lifecycle_outliers_cannot_be_hidden_by_phase_changes | [源码:76](../../../tools/test_check_resource_cycles.py#L76) · `def test_unknown_gpu_and_lifecycle_outliers_cannot_be_hidden_by_phase_changes(self)` |
| function ResourceCycles.test_phase_epochs_reject_boundary_mislabelling_and_reverse | [源码:82](../../../tools/test_check_resource_cycles.py#L82) · `def test_phase_epochs_reject_boundary_mislabelling_and_reverse(self)` |
| function ResourceCycles.test_fixed_baseline_span_and_unknown_closed_counters_are_rejected | [源码:88](../../../tools/test_check_resource_cycles.py#L88) · `def test_fixed_baseline_span_and_unknown_closed_counters_are_rejected(self)` |
| function ResourceCycles.test_positive_closed_trend_is_reported_without_a_leak_allowance | [源码:96](../../../tools/test_check_resource_cycles.py#L96) · `def test_positive_closed_trend_is_reported_without_a_leak_allowance(self)` |
| function ResourceCycles.test_phase_file_is_bounded_and_does_not_accept_partial_or_foreign_data | [源码:101](../../../tools/test_check_resource_cycles.py#L101) · `def test_phase_file_is_bounded_and_does_not_accept_partial_or_foreign_data(self)` |

## tools/test_check_review_inventory.py

117 行；SHA256 `aef291b52c41ad14448b018787cdf3a45eb337c138d850bed1250846ca4e5c23`

导入：`pathlib`, `review_inventory`, `tempfile`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| class ReviewInventoryTests | [源码:7](../../../tools/test_check_review_inventory.py#L7) · `unittest.TestCase` |
| function ReviewInventoryTests.test_generated_inventory_uses_lf_even_on_windows | [源码:8](../../../tools/test_check_review_inventory.py#L8) · `def test_generated_inventory_uses_lf_even_on_windows(self)` |
| function ReviewInventoryTests.test_redirect_is_not_a_duplicate_definition | [源码:13](../../../tools/test_check_review_inventory.py#L13) · `def test_redirect_is_not_a_duplicate_definition(self)` |
| function ReviewInventoryTests.fixture | [源码:17](../../../tools/test_check_review_inventory.py#L17) · `def fixture(self)` |
| function ReviewInventoryTests.test_own_methods_fields_and_impls_have_compiler_source | [源码:24](../../../tools/test_check_review_inventory.py#L24) · `def test_own_methods_fields_and_impls_have_compiler_source(self)` |
| function ReviewInventoryTests.test_required_trait_method_and_nested_module_are_not_lost | [源码:32](../../../tools/test_check_review_inventory.py#L32) · `def test_required_trait_method_and_nested_module_are_not_lost(self)` |
| function ReviewInventoryTests.test_expand_button_text_does_not_corrupt_declared_fields | [源码:38](../../../tools/test_check_review_inventory.py#L38) · `def test_expand_button_text_does_not_corrupt_declared_fields(self)` |
| function ReviewInventoryTests.test_inline_script_and_external_method_do_not_create_fake_items | [源码:44](../../../tools/test_check_review_inventory.py#L44) · `def test_inline_script_and_external_method_do_not_create_fake_items(self)` |
| function ReviewInventoryTests.test_mask_preserves_lines_and_drops_nested_comments_and_raw_literals | [源码:48](../../../tools/test_check_review_inventory.py#L48) · `def test_mask_preserves_lines_and_drops_nested_comments_and_raw_literals(self)` |
| function ReviewInventoryTests.test_const_function_and_lifetime_are_not_misread_as_string_or_constant | [源码:56](../../../tools/test_check_review_inventory.py#L56) · `def test_const_function_and_lifetime_are_not_misread_as_string_or_constant(self)` |
| function ReviewInventoryTests.test_test_only_trait_impl_has_a_lexical_location_without_claiming_resolution | [源码:60](../../../tools/test_check_review_inventory.py#L60) · `def test_test_only_trait_impl_has_a_lexical_location_without_claiming_resolution(self)` |
| function ReviewInventoryTests.test_python_class_bases_methods_nested_functions_and_imports | [源码:65](../../../tools/test_check_review_inventory.py#L65) · `def test_python_class_bases_methods_nested_functions_and_imports(self)` |
| function ReviewInventoryTests.test_reexport_alias_does_not_duplicate_its_defining_object | [源码:74](../../../tools/test_check_review_inventory.py#L74) · `def test_reexport_alias_does_not_duplicate_its_defining_object(self)` |
| function ReviewInventoryTests.test_reexport_collapsed_span_and_resolved_self_are_one_definition | [源码:83](../../../tools/test_check_review_inventory.py#L83) · `def test_reexport_collapsed_span_and_resolved_self_are_one_definition(self)` |
| function ReviewInventoryTests.test_methods_on_same_derive_line_keep_distinct_trait_implementations | [源码:97](../../../tools/test_check_review_inventory.py#L97) · `def test_methods_on_same_derive_line_keep_distinct_trait_implementations(self)` |
| function ReviewInventoryTests.test_native_collector_and_shader_functions_are_only_lexical_entries | [源码:110](../../../tools/test_check_review_inventory.py#L110) · `def test_native_collector_and_shader_functions_are_only_lexical_entries(self)` |

## tools/test_check_scheduling.py

59 行；SHA256 `30fe35ec1bb8ee045d109c15fa015652ca5799acdd2c3803c20414668df2a2ad`

导入：`copy`, `measure_scheduling`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| function timing | [源码:6](../../../tools/test_check_scheduling.py#L6) · `def timing()` |
| class SchedulingChecks | [源码:15](../../../tools/test_check_scheduling.py#L15) · `unittest.TestCase` |
| function SchedulingChecks.test_same_event_links_supply_sleep_work_and_callback_clock_bracket | [源码:16](../../../tools/test_check_scheduling.py#L16) · `def test_same_event_links_supply_sleep_work_and_callback_clock_bracket(self)` |
| function SchedulingChecks.test_controls_and_pump_cost_are_not_mislabelled_sleep | [源码:25](../../../tools/test_check_scheduling.py#L25) · `def test_controls_and_pump_cost_are_not_mislabelled_sleep(self)` |
| function SchedulingChecks.test_partial_overwritten_or_truncated_data_never_proves_completeness | [源码:30](../../../tools/test_check_scheduling.py#L30) · `def test_partial_overwritten_or_truncated_data_never_proves_completeness(self)` |
| function SchedulingChecks.test_host_wakeup_lateness_is_distinct_from_previous_app_work | [源码:41](../../../tools/test_check_scheduling.py#L41) · `def test_host_wakeup_lateness_is_distinct_from_previous_app_work(self)` |
| function SchedulingChecks.test_missing_clock_or_wrong_host_snapshot_is_rejected | [源码:52](../../../tools/test_check_scheduling.py#L52) · `def test_missing_clock_or_wrong_host_snapshot_is_rejected(self)` |

## tools/test_check_separated_native.py

57 行；SHA256 `12c35a4dafb9d61d6c9582b00809f03843328c3d35ab3e53b0d517a272ee2b24`

导入：`copy`, `measure_separated_native`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| function report | [源码:6](../../../tools/test_check_separated_native.py#L6) · `def report()` |
| function metrics | [源码:15](../../../tools/test_check_separated_native.py#L15) · `def metrics()` |
| class SeparatedNativeChecks | [源码:19](../../../tools/test_check_separated_native.py#L19) · `unittest.TestCase` |
| function SeparatedNativeChecks.test_complete_exact_plan_passes | [源码:20](../../../tools/test_check_separated_native.py#L20) · `def test_complete_exact_plan_passes(self)` |
| function SeparatedNativeChecks.test_missing_duplicate_foreign_and_invalid_probes_rejected | [源码:26](../../../tools/test_check_separated_native.py#L26) · `def test_missing_duplicate_foreign_and_invalid_probes_rejected(self)` |
| function SeparatedNativeChecks.test_underrun_and_latency_failure_are_retained | [源码:38](../../../tools/test_check_separated_native.py#L38) · `def test_underrun_and_latency_failure_are_retained(self)` |
| function SeparatedNativeChecks.test_lifecycle_and_native_failure_are_not_silenced | [源码:45](../../../tools/test_check_separated_native.py#L45) · `def test_lifecycle_and_native_failure_are_not_silenced(self)` |
| function SeparatedNativeChecks.test_budgets_keep_original_limits | [源码:50](../../../tools/test_check_separated_native.py#L50) · `def test_budgets_keep_original_limits(self)` |

## tools/test_check_stable_window.py

117 行；SHA256 `4b006ac5bc13acfdf718fd6f0a26694fcf53576a9fa16d8f94f5d1d251032dd5`

导入：`copy`, `measure_main_work`, `measure_stable_window`, `pathlib`, `tempfile`, `unittest`, `unittest.mock`

| 定义 | 位置／签名 |
| --- | --- |
| function row | [源码:10](../../../tools/test_check_stable_window.py#L10) · `def row(phase)` |
| class StableWindowTests | [源码:14](../../../tools/test_check_stable_window.py#L14) · `unittest.TestCase` |
| function StableWindowTests.empty | [源码:15](../../../tools/test_check_stable_window.py#L15) · `def empty(self)` |
| function StableWindowTests.test_full_empty_contract_and_modes | [源码:17](../../../tools/test_check_stable_window.py#L17) · `def test_full_empty_contract_and_modes(self)` |
| function StableWindowTests.test_product_routes_use_ordinary_assembly_without_enabling_profiling | [源码:23](../../../tools/test_check_stable_window.py#L23) · `def test_product_routes_use_ordinary_assembly_without_enabling_profiling(self)` |
| function StableWindowTests.test_product_child_keeps_stable_protocol_and_requests_product_policy | [源码:28](../../../tools/test_check_stable_window.py#L28) · `def test_product_child_keeps_stable_protocol_and_requests_product_policy(self)` |
| function StableWindowTests.test_product_policy_requires_both_explicit_assembly_markers | [源码:33](../../../tools/test_check_stable_window.py#L33) · `def test_product_policy_requires_both_explicit_assembly_markers(self)` |
| function StableWindowTests.test_legacy_control_is_retained_but_cannot_prove_product_policy | [源码:41](../../../tools/test_check_stable_window.py#L41) · `def test_legacy_control_is_retained_but_cannot_prove_product_policy(self)` |
| function StableWindowTests.test_observer_uses_product_entry_and_retains_actual_policy | [源码:45](../../../tools/test_check_stable_window.py#L45) · `def test_observer_uses_product_entry_and_retains_actual_policy(self)` |
| function StableWindowTests.test_observer_uses_product_entry_and_retains_actual_policy.fake_observe | [源码:48](../../../tools/test_check_stable_window.py#L48) · `def fake_observe(binary, output, argument, **kw)` |
| function StableWindowTests.test_observer_uses_product_entry_and_retains_actual_policy.fake_sampler | [源码:51](../../../tools/test_check_stable_window.py#L51) · `def fake_sampler(sampler, child)` |
| function StableWindowTests.test_product_child_preserves_bounded_long_window_timeouts | [源码:62](../../../tools/test_check_stable_window.py#L62) · `def test_product_child_preserves_bounded_long_window_timeouts(self)` |
| function StableWindowTests.test_short_tail_input_extra_draw_and_moving_clock_remain_failures | [源码:71](../../../tools/test_check_stable_window.py#L71) · `def test_short_tail_input_extra_draw_and_moving_clock_remain_failures(self)` |
| function StableWindowTests.test_sample_or_retained_resource_prevents_empty_acceptance | [源码:75](../../../tools/test_check_stable_window.py#L75) · `def test_sample_or_retained_resource_prevents_empty_acceptance(self)` |
| function StableWindowTests.test_missing_or_extra_window_is_rejected | [源码:79](../../../tools/test_check_stable_window.py#L79) · `def test_missing_or_extra_window_is_rejected(self)` |
| function StableWindowTests.test_retained_pause_and_real_closed_native_audit | [源码:83](../../../tools/test_check_stable_window.py#L83) · `def test_retained_pause_and_real_closed_native_audit(self)` |
| function StableWindowTests.test_stable_child_timeout_cannot_be_unbounded | [源码:92](../../../tools/test_check_stable_window.py#L92) · `def test_stable_child_timeout_cannot_be_unbounded(self)` |
| function StableWindowTests.test_independent_two_percent_gate_calibration_and_missing_gpu_fail_closed | [源码:95](../../../tools/test_check_stable_window.py#L95) · `def test_independent_two_percent_gate_calibration_and_missing_gpu_fail_closed(self)` |

## tools/test_check_staging_recovery.py

90 行；SHA256 `bed24773a8bb4f88607f474fefd811c295698635bff3a7bf872a528f42f6f24d`

导入：`copy`, `measure_audio_load`, `measure_staging_recovery`, `pathlib`, `test_check_combined`, `unittest`, `unittest.mock`

| 定义 | 位置／签名 |
| --- | --- |
| function recovery_fixture | [源码:10](../../../tools/test_check_staging_recovery.py#L10) · `def recovery_fixture()` |
| class RecoveryChecks | [源码:29](../../../tools/test_check_staging_recovery.py#L29) · `unittest.TestCase` |
| function RecoveryChecks.test_lifecycle_timeout_remains_bounded_before_launch | [源码:30](../../../tools/test_check_staging_recovery.py#L30) · `def test_lifecycle_timeout_remains_bounded_before_launch(self)` |
| function RecoveryChecks.test_allocator_checkpoints_cannot_be_missing_or_unbounded | [源码:34](../../../tools/test_check_staging_recovery.py#L34) · `def test_allocator_checkpoints_cannot_be_missing_or_unbounded(self)` |
| function RecoveryChecks.test_short_cycles_cannot_replace_300_seconds_or_missing_lifetimes | [源码:39](../../../tools/test_check_staging_recovery.py#L39) · `def test_short_cycles_cannot_replace_300_seconds_or_missing_lifetimes(self)` |
| function RecoveryChecks.test_each_cycle_releases_and_obeys_existing_budget | [源码:46](../../../tools/test_check_staging_recovery.py#L46) · `def test_each_cycle_releases_and_obeys_existing_budget(self)` |
| function RecoveryChecks.test_stable_cpu_frames_and_short_observation_do_not_get_waived | [源码:54](../../../tools/test_check_staging_recovery.py#L54) · `def test_stable_cpu_frames_and_short_observation_do_not_get_waived(self)` |
| function RecoveryChecks.test_unknown_gpu_data_excess_memory_and_slow_submission_are_failures | [源码:64](../../../tools/test_check_staging_recovery.py#L64) · `def test_unknown_gpu_data_excess_memory_and_slow_submission_are_failures(self)` |
| function RecoveryChecks.test_trend_preserves_retention_without_an_allowed_leak_cutoff | [源码:72](../../../tools/test_check_staging_recovery.py#L72) · `def test_trend_preserves_retention_without_an_allowed_leak_cutoff(self)` |
| function RecoveryChecks.test_cycle_trend_does_not_substitute_a_long_idle_baseline | [源码:80](../../../tools/test_check_staging_recovery.py#L80) · `def test_cycle_trend_does_not_substitute_a_long_idle_baseline(self)` |

## tools/test_check_supply_recovery.py

53 行；SHA256 `6bb7baccbf52113dc914ec8f1c4c81e2392bb6f4967f579ad8d513c1802820be`

导入：`copy`, `measure_supply_recovery`, `test_check_scheduling`, `test_check_voice_phase`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| function inputs | [源码:8](../../../tools/test_check_supply_recovery.py#L8) · `def inputs()` |
| class SupplyRecoveryChecks | [源码:21](../../../tools/test_check_supply_recovery.py#L21) · `unittest.TestCase` |
| function SupplyRecoveryChecks.test_interleaved_plan_and_exact_existing_acceptance | [源码:22](../../../tools/test_check_supply_recovery.py#L22) · `def test_interleaved_plan_and_exact_existing_acceptance(self)` |
| function SupplyRecoveryChecks.test_policy_identity_and_bounded_counters_are_required | [源码:27](../../../tools/test_check_supply_recovery.py#L27) · `def test_policy_identity_and_bounded_counters_are_required(self)` |
| function SupplyRecoveryChecks.test_empty_frames_and_latency_straddling_are_not_waived | [源码:37](../../../tools/test_check_supply_recovery.py#L37) · `def test_empty_frames_and_latency_straddling_are_not_waived(self)` |
| function SupplyRecoveryChecks.test_cpu_memory_and_incomplete_capture_fail_without_relabelling | [源码:45](../../../tools/test_check_supply_recovery.py#L45) · `def test_cpu_memory_and_incomplete_capture_fail_without_relabelling(self)` |

## tools/test_check_text_work.py

30 行；SHA256 `98265bd26303b45a15bfa01998f414e01e642f3daedbf095ca6c060a94568cc1`

导入：`copy`, `measure_text_work`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| function fixture | [源码:5](../../../tools/test_check_text_work.py#L5) · `def fixture()` |
| class TextWorkTests | [源码:9](../../../tools/test_check_text_work.py#L9) · `unittest.TestCase` |
| function TextWorkTests.test_counts_are_weak_identity_reuse_not_cache_miss_or_cpu_estimates | [源码:10](../../../tools/test_check_text_work.py#L10) · `def test_counts_are_weak_identity_reuse_not_cache_miss_or_cpu_estimates(self)` |
| function TextWorkTests.test_missing_cleanup_or_census_overflow_or_mismatched_cleanup_is_rejected | [源码:16](../../../tools/test_check_text_work.py#L16) · `def test_missing_cleanup_or_census_overflow_or_mismatched_cleanup_is_rejected(self)` |
| function TextWorkTests.test_incomplete_stable_phase_and_reversed_pass_or_counts_are_rejected | [源码:21](../../../tools/test_check_text_work.py#L21) · `def test_incomplete_stable_phase_and_reversed_pass_or_counts_are_rejected(self)` |

## tools/test_check_thread_sampling.py

191 行；SHA256 `2fc6c89afef67a0d7b9d85250c00d7d8367c31f4b02ab50d24461d3a8ce1be91`

导入：`measure_thread_sampling`, `measure_voice_cpu`, `test_check_background_stream`, `types`, `unittest`, `unittest.mock`

| 定义 | 位置／签名 |
| --- | --- |
| class FakeThreads | [源码:8](../../../tools/test_check_thread_sampling.py#L8) |
| function FakeThreads.__init__ | [源码:11](../../../tools/test_check_thread_sampling.py#L11) · `def __init__(self)` |
| function FakeThreads.live_ids | [源码:20](../../../tools/test_check_thread_sampling.py#L20) · `def live_ids(self)` |
| function FakeThreads.open_thread | [源码:24](../../../tools/test_check_thread_sampling.py#L24) · `def open_thread(self, tid)` |
| function FakeThreads.owner | [源码:31](../../../tools/test_check_thread_sampling.py#L31) · `def owner(self, handle)` |
| function FakeThreads.times | [源码:33](../../../tools/test_check_thread_sampling.py#L33) · `def times(self, handle)` |
| function FakeThreads.description | [源码:39](../../../tools/test_check_thread_sampling.py#L39) · `def description(self, handle)` |
| function FakeThreads.close_thread | [源码:43](../../../tools/test_check_thread_sampling.py#L43) · `def close_thread(self, handle)` |
| class ThreadSamplingChecks | [源码:48](../../../tools/test_check_thread_sampling.py#L48) · `unittest.TestCase` |
| function ThreadSamplingChecks.test_capacity_cannot_disable_the_lifetime_bound | [源码:49](../../../tools/test_check_thread_sampling.py#L49) · `def test_capacity_cannot_disable_the_lifetime_bound(self)` |
| function ThreadSamplingChecks.test_reuses_owned_handle_and_refreshes_only_on_phase_transition | [源码:54](../../../tools/test_check_thread_sampling.py#L54) · `def test_reuses_owned_handle_and_refreshes_only_on_phase_transition(self)` |
| function ThreadSamplingChecks.test_retains_exited_identity_and_does_not_retarget_reused_tid | [源码:66](../../../tools/test_check_thread_sampling.py#L66) · `def test_retains_exited_identity_and_does_not_retarget_reused_tid(self)` |
| function ThreadSamplingChecks.test_foreign_tid_race_and_unopenable_exited_thread_are_not_sampled | [源码:78](../../../tools/test_check_thread_sampling.py#L78) · `def test_foreign_tid_race_and_unopenable_exited_thread_are_not_sampled(self)` |
| function ThreadSamplingChecks.test_limit_refusal_closes_current_and_preserves_bounded_prior_handles | [源码:85](../../../tools/test_check_thread_sampling.py#L85) · `def test_limit_refusal_closes_current_and_preserves_bounded_prior_handles(self)` |
| function ThreadSamplingChecks.test_api_failures_release_unretained_and_prior_handles | [源码:93](../../../tools/test_check_thread_sampling.py#L93) · `def test_api_failures_release_unretained_and_prior_handles(self)` |
| function ThreadSamplingChecks.test_corrupted_or_reversed_retained_counter_rejects | [源码:102](../../../tools/test_check_thread_sampling.py#L102) · `def test_corrupted_or_reversed_retained_counter_rejects(self)` |
| function ThreadSamplingChecks.test_close_attempts_every_handle_even_after_error | [源码:109](../../../tools/test_check_thread_sampling.py#L109) · `def test_close_attempts_every_handle_even_after_error(self)` |
| function ThreadSamplingChecks.test_close_attempts_every_handle_even_after_error.close | [源码:113](../../../tools/test_check_thread_sampling.py#L113) · `def close(handle)` |
| function ThreadSamplingChecks.test_enumerating_reference_still_closes_per_query_and_filters_owner | [源码:121](../../../tools/test_check_thread_sampling.py#L121) · `def test_enumerating_reference_still_closes_per_query_and_filters_owner(self)` |
| class ThreadSamplingChecks.test_enumerating_reference_still_closes_per_query_and_filters_owner.Reference | [源码:126](../../../tools/test_check_thread_sampling.py#L126) |
| function ThreadSamplingChecks.test_native_close_failure_is_not_reported_as_released | [源码:138](../../../tools/test_check_thread_sampling.py#L138) · `def test_native_close_failure_is_not_reported_as_released(self)` |
| function ThreadSamplingChecks.test_observer_cost_keeps_raw_child_budget_and_excludes_first_query_from_sums | [源码:145](../../../tools/test_check_thread_sampling.py#L145) · `def test_observer_cost_keeps_raw_child_budget_and_excludes_first_query_from_sums(self)` |
| function ThreadSamplingChecks.test_audit_closes_cached_resources_on_early_exit | [源码:159](../../../tools/test_check_thread_sampling.py#L159) · `def test_audit_closes_cached_resources_on_early_exit(self)` |
| class ThreadSamplingChecks.test_audit_closes_cached_resources_on_early_exit.Backend | [源码:160](../../../tools/test_check_thread_sampling.py#L160) |
| function ThreadSamplingChecks.test_audit_closes_cached_resources_on_early_exit.Backend.__init__ | [源码:161](../../../tools/test_check_thread_sampling.py#L161) · `def __init__(self, child)` |
| function ThreadSamplingChecks.test_audit_closes_cached_resources_on_early_exit.Backend.sample | [源码:163](../../../tools/test_check_thread_sampling.py#L163) · `def sample(self, phase)` |
| function ThreadSamplingChecks.test_audit_closes_cached_resources_on_early_exit.Backend.close | [源码:165](../../../tools/test_check_thread_sampling.py#L165) · `def close(self)` |
| function ThreadSamplingChecks.test_fixed_counterbalanced_plan_has_no_retry_or_failure_selection | [源码:172](../../../tools/test_check_thread_sampling.py#L172) · `def test_fixed_counterbalanced_plan_has_no_retry_or_failure_selection(self)` |
| function ThreadSamplingChecks.test_additional_stream_callback_and_observer_checks_reject_each_leak | [源码:175](../../../tools/test_check_thread_sampling.py#L175) · `def test_additional_stream_callback_and_observer_checks_reject_each_leak(self)` |

## tools/test_check_ui_wall.py

59 行；SHA256 `ec12cce81c16e11dfd66bdb52d171ccbf7725191a96c913b024a7602dcb3a159`

导入：`measure_main_work`, `measure_ui_wall`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| function fixture | [源码:6](../../../tools/test_check_ui_wall.py#L6) · `def fixture()` |
| class UiWallTests | [源码:12](../../../tools/test_check_ui_wall.py#L12) · `unittest.TestCase` |
| function UiWallTests.test_complete_call_has_exact_exclusive_partition | [源码:13](../../../tools/test_check_ui_wall.py#L13) · `def test_complete_call_has_exact_exclusive_partition(self)` |
| function UiWallTests.test_multiple_calls_and_no_most_favorable_call_selection | [源码:20](../../../tools/test_check_ui_wall.py#L20) · `def test_multiple_calls_and_no_most_favorable_call_selection(self)` |
| function UiWallTests.test_incomplete_or_outside_or_host_interleaving_is_rejected | [源码:26](../../../tools/test_check_ui_wall.py#L26) · `def test_incomplete_or_outside_or_host_interleaving_is_rejected(self)` |
| function UiWallTests.test_capacity_clock_and_unknown_schema_fail_closed | [源码:33](../../../tools/test_check_ui_wall.py#L33) · `def test_capacity_clock_and_unknown_schema_fail_closed(self)` |
| function UiWallTests.test_cpu_raw_predicate_is_retained_but_cannot_authorize_acceptance | [源码:42](../../../tools/test_check_ui_wall.py#L42) · `def test_cpu_raw_predicate_is_retained_but_cannot_authorize_acceptance(self)` |
| function UiWallTests.test_new_details_are_exclusive_and_roll_up_without_reinterpreting_old_format | [源码:48](../../../tools/test_check_ui_wall.py#L48) · `def test_new_details_are_exclusive_and_roll_up_without_reinterpreting_old_format(self)` |

## tools/test_check_voice_cpu.py

52 行；SHA256 `92ea736245c8c189c540ce71e02c5dbd4caafd433b67c1757de0903513906be5`

导入：`measure_voice_cpu`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| function sample | [源码:5](../../../tools/test_check_voice_cpu.py#L5) · `def sample(time, cpu, rows)` |
| class VoiceCpuChecks | [源码:9](../../../tools/test_check_voice_cpu.py#L9) · `unittest.TestCase` |
| function VoiceCpuChecks.test_callback_work_is_wall_time_and_not_subtracted_from_process_cpu | [源码:10](../../../tools/test_check_voice_cpu.py#L10) · `def test_callback_work_is_wall_time_and_not_subtracted_from_process_cpu(self)` |
| function VoiceCpuChecks.test_partial_or_reversed_callback_work_does_not_produce_cost | [源码:17](../../../tools/test_check_voice_cpu.py#L17) · `def test_partial_or_reversed_callback_work_does_not_produce_cost(self)` |
| function VoiceCpuChecks.test_unprofiled_rows_cannot_be_relabelled_as_instrumented_work | [源码:21](../../../tools/test_check_voice_cpu.py#L21) · `def test_unprofiled_rows_cannot_be_relabelled_as_instrumented_work(self)` |
| function VoiceCpuChecks.test_owned_thread_categories_and_unknowns_not_guessed_as_gpu | [源码:24](../../../tools/test_check_voice_cpu.py#L24) · `def test_owned_thread_categories_and_unknowns_not_guessed_as_gpu(self)` |
| function VoiceCpuChecks.test_cpu_attribution_preserves_process_counter_and_quantisation_gap | [源码:31](../../../tools/test_check_voice_cpu.py#L31) · `def test_cpu_attribution_preserves_process_counter_and_quantisation_gap(self)` |
| function VoiceCpuChecks.test_lifetime_change_is_visible_and_not_invented_as_zero_cost | [源码:39](../../../tools/test_check_voice_cpu.py#L39) · `def test_lifetime_change_is_visible_and_not_invented_as_zero_cost(self)` |
| function VoiceCpuChecks.test_missing_phase_or_reversed_counters_reject | [源码:45](../../../tools/test_check_voice_cpu.py#L45) · `def test_missing_phase_or_reversed_counters_reject(self)` |

## tools/test_check_voice_phase.py

144 行；SHA256 `b0e93006b4461b0687f2993262618df32fd7c57002fffb221ea045fd5f054044`

导入：`copy`, `measure_voice_phase`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| function fixture | [源码:6](../../../tools/test_check_voice_phase.py#L6) · `def fixture(count=64)` |
| function with_underrun | [源码:33](../../../tools/test_check_voice_phase.py#L33) · `def with_underrun(r)` |
| class VoicePhaseChecks | [源码:42](../../../tools/test_check_voice_phase.py#L42) · `unittest.TestCase` |
| function VoicePhaseChecks.test_standard_phase_uses_prior_callback_and_does_not_fold_old_reference | [源码:43](../../../tools/test_check_voice_phase.py#L43) · `def test_standard_phase_uses_prior_callback_and_does_not_fold_old_reference(self)` |
| function VoicePhaseChecks.test_exact_voice_marker_requires_same_epoch_serial_callback_and_requested_identity | [源码:50](../../../tools/test_check_voice_phase.py#L50) · `def test_exact_voice_marker_requires_same_epoch_serial_callback_and_requested_identity(self)` |
| function VoicePhaseChecks.test_completion_is_an_interval_and_entry_stamp_cannot_replace_its_upper_bound | [源码:60](../../../tools/test_check_voice_phase.py#L60) · `def test_completion_is_an_interval_and_entry_stamp_cannot_replace_its_upper_bound(self)` |
| function VoicePhaseChecks.test_balanced_requests_do_not_prove_achieved_phase_coverage | [源码:69](../../../tools/test_check_voice_phase.py#L69) · `def test_balanced_requests_do_not_prove_achieved_phase_coverage(self)` |
| function VoicePhaseChecks.test_exact_marker_does_not_waive_old_gate_or_resource_failures | [源码:79](../../../tools/test_check_voice_phase.py#L79) · `def test_exact_marker_does_not_waive_old_gate_or_resource_failures(self)` |
| function VoicePhaseChecks.test_missing_order_reference_and_variable_callback_are_visible | [源码:92](../../../tools/test_check_voice_phase.py#L92) · `def test_missing_order_reference_and_variable_callback_are_visible(self)` |
| function VoicePhaseChecks.test_event_context_distinguishes_consecutive_drain_from_one_large_callback | [源码:98](../../../tools/test_check_voice_phase.py#L98) · `def test_event_context_distinguishes_consecutive_drain_from_one_large_callback(self)` |
| function VoicePhaseChecks.test_partial_context_and_missing_inconsistent_stage_counts_are_rejected | [源码:112](../../../tools/test_check_voice_phase.py#L112) · `def test_partial_context_and_missing_inconsistent_stage_counts_are_rejected(self)` |
| function VoicePhaseChecks.test_in_progress_supply_or_retiming_cannot_be_labelled_uninterrupted_drain | [源码:123](../../../tools/test_check_voice_phase.py#L123) · `def test_in_progress_supply_or_retiming_cannot_be_labelled_uninterrupted_drain(self)` |
| function VoicePhaseChecks.test_trace_overflow_retains_totals_and_fails_completeness | [源码:131](../../../tools/test_check_voice_phase.py#L131) · `def test_trace_overflow_retains_totals_and_fails_completeness(self)` |

## tools/test_check_window_recovery.py

158 行；SHA256 `ca66cc111aa6ed6059ad09edf72bde3ef95237c7cfd2dab2e641c1b8a6951c6f`

导入：`copy`, `measure_window_recovery`, `pathlib`, `tempfile`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| function fixture | [源码:8](../../../tools/test_check_window_recovery.py#L8) · `def fixture()` |
| function initialization_fixture | [源码:24](../../../tools/test_check_window_recovery.py#L24) · `def initialization_fixture(plan='init-audio')` |
| class WindowRecovery | [源码:37](../../../tools/test_check_window_recovery.py#L37) · `unittest.TestCase` |
| function WindowRecovery.test_active_edit_requires_real_input_cancel_and_each_declared_fault_order | [源码:38](../../../tools/test_check_window_recovery.py#L38) · `def test_active_edit_requires_real_input_cancel_and_each_declared_fault_order(self)` |
| function WindowRecovery.test_endpoint_failure_requires_actual_retained_owners_gating_and_late_delivery | [源码:52](../../../tools/test_check_window_recovery.py#L52) · `def test_endpoint_failure_requires_actual_retained_owners_gating_and_late_delivery(self)` |
| function WindowRecovery.test_cleanup_cannot_unlock_retry_until_real_retained_owners_are_gone | [源码:65](../../../tools/test_check_window_recovery.py#L65) · `def test_cleanup_cannot_unlock_retry_until_real_retained_owners_are_gone(self)` |
| function WindowRecovery.test_post_consumption_requires_exact_history_and_each_cleanup_owner | [源码:77](../../../tools/test_check_window_recovery.py#L77) · `def test_post_consumption_requires_exact_history_and_each_cleanup_owner(self)` |
| function WindowRecovery.test_initialization_stage_copy_anchor_and_redo_path_are_independent_requirements | [源码:83](../../../tools/test_check_window_recovery.py#L83) · `def test_initialization_stage_copy_anchor_and_redo_path_are_independent_requirements(self)` |
| function WindowRecovery.test_injection_requires_actual_boundary_notice_and_pre_gui_audio_gate | [源码:91](../../../tools/test_check_window_recovery.py#L91) · `def test_injection_requires_actual_boundary_notice_and_pre_gui_audio_gate(self)` |
| function WindowRecovery.test_failed_creation_and_late_draw_cannot_be_inferred_from_regular_reopen | [源码:103](../../../tools/test_check_window_recovery.py#L103) · `def test_failed_creation_and_late_draw_cannot_be_inferred_from_regular_reopen(self)` |
| function WindowRecovery.test_surface_id_slots_are_not_live_textures_or_a_blanket_retirement_exception | [源码:112](../../../tools/test_check_window_recovery.py#L112) · `def test_surface_id_slots_are_not_live_textures_or_a_blanket_retirement_exception(self)` |
| function WindowRecovery.test_full_component_success_does_not_accept_physical_dpi_or_in_place_reset | [源码:123](../../../tools/test_check_window_recovery.py#L123) · `def test_full_component_success_does_not_accept_physical_dpi_or_in_place_reset(self)` |
| function WindowRecovery.test_each_retirement_owner_and_final_resource_is_required | [源码:127](../../../tools/test_check_window_recovery.py#L127) · `def test_each_retirement_owner_and_final_resource_is_required(self)` |
| function WindowRecovery.test_latency_underrun_memory_and_unknown_gpu_remain_hard | [源码:133](../../../tools/test_check_window_recovery.py#L133) · `def test_latency_underrun_memory_and_unknown_gpu_remain_hard(self)` |
| function WindowRecovery.test_old_receipts_duplicate_actions_or_unproven_dpi_do_not_pass | [源码:144](../../../tools/test_check_window_recovery.py#L144) · `def test_old_receipts_duplicate_actions_or_unproven_dpi_do_not_pass(self)` |
| function WindowRecovery.test_ack_creation_is_atomic_and_never_overwrites_existing_ack | [源码:150](../../../tools/test_check_window_recovery.py#L150) · `def test_ack_creation_is_atomic_and_never_overwrites_existing_ack(self)` |

## tools/test_check_worker_native.py

57 行；SHA256 `bf02e577797f2db7826843d3057f216e3f649d8b6e099a5cae894e6396da3ebb`

导入：`measure_worker_native`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| function report | [源码:5](../../../tools/test_check_worker_native.py#L5) · `def report()` |
| function metrics | [源码:14](../../../tools/test_check_worker_native.py#L14) · `def metrics()` |
| class WorkerNativeChecks | [源码:18](../../../tools/test_check_worker_native.py#L18) · `unittest.TestCase` |
| function WorkerNativeChecks.test_fixed_plan_and_complete_requests | [源码:19](../../../tools/test_check_worker_native.py#L19) · `def test_fixed_plan_and_complete_requests(self)` |
| function WorkerNativeChecks.test_scope_duplicates_missing_and_invalid_timing_rejected | [源码:24](../../../tools/test_check_worker_native.py#L24) · `def test_scope_duplicates_missing_and_invalid_timing_rejected(self)` |
| function WorkerNativeChecks.test_underrun_or_latency_cannot_pass | [源码:37](../../../tools/test_check_worker_native.py#L37) · `def test_underrun_or_latency_cannot_pass(self)` |
| function WorkerNativeChecks.test_cleanup_pause_and_runtime_errors_fail_independently | [源码:43](../../../tools/test_check_worker_native.py#L43) · `def test_cleanup_pause_and_runtime_errors_fail_independently(self)` |
| function WorkerNativeChecks.test_original_cpu_memory_and_pcm_limits_kept | [源码:48](../../../tools/test_check_worker_native.py#L48) · `def test_original_cpu_memory_and_pcm_limits_kept(self)` |

## tools/tests/test_p0_audio_adaptation.py

109 行；SHA256 `35f60d0dc7627644b6a3ceb3b178e044d81ed4a0afa901ae9c512f6b6b26cdb1`

导入：`check_p0_audio_adaptation`, `copy`, `json`, `math`, `pathlib`, `struct`, `sys`, `tempfile`, `types`, `unittest`, `unittest.mock`

| 定义 | 位置／签名 |
| --- | --- |
| class AdaptationTests | [源码:15](../../../tools/tests/test_p0_audio_adaptation.py#L15) · `unittest.TestCase` |
| function AdaptationTests.test_phase_costs_do_not_mix_active_and_paused_or_closed_samples | [源码:16](../../../tools/tests/test_p0_audio_adaptation.py#L16) · `def test_phase_costs_do_not_mix_active_and_paused_or_closed_samples(self)` |
| function AdaptationTests.test_phase_costs_do_not_mix_active_and_paused_or_closed_samples.sample | [源码:17](../../../tools/tests/test_p0_audio_adaptation.py#L17) · `def sample(t, cpu, phase)` |
| function AdaptationTests.test_mono_reference_keeps_source_rate_shape_gain_and_native_hash | [源码:25](../../../tools/tests/test_p0_audio_adaptation.py#L25) · `def test_mono_reference_keeps_source_rate_shape_gain_and_native_hash(self)` |
| function AdaptationTests.test_full_comparison_has_no_trimmed_tail_and_checks_finite_mono_samples | [源码:38](../../../tools/tests/test_p0_audio_adaptation.py#L38) · `def test_full_comparison_has_no_trimmed_tail_and_checks_finite_mono_samples(self)` |
| function AdaptationTests.test_fixture_preparation_refuses_escape_or_existing_without_encoder | [源码:47](../../../tools/tests/test_p0_audio_adaptation.py#L47) · `def test_fixture_preparation_refuses_escape_or_existing_without_encoder(self)` |
| function AdaptationTests.test_fixture_requires_current_decoder_complete_references_and_finite_proof | [源码:54](../../../tools/tests/test_p0_audio_adaptation.py#L54) · `def test_fixture_requires_current_decoder_complete_references_and_finite_proof(self)` |
| function AdaptationTests.proof | [源码:76](../../../tools/tests/test_p0_audio_adaptation.py#L76) · `def proof(self)` |
| function AdaptationTests.proof.e | [源码:83](../../../tools/tests/test_p0_audio_adaptation.py#L83) · `def e(kind, g, first=0, frames=0)` |
| function AdaptationTests.test_trace_requires_generation_order_continuity_pause_tail_and_old_rejection | [源码:90](../../../tools/tests/test_p0_audio_adaptation.py#L90) · `def test_trace_requires_generation_order_continuity_pause_tail_and_old_rejection(self)` |
| function AdaptationTests.test_proof_rejects_unfrozen_recovery_wrong_format_or_missing_pool | [源码:97](../../../tools/tests/test_p0_audio_adaptation.py#L97) · `def test_proof_rejects_unfrozen_recovery_wrong_format_or_missing_pool(self)` |

## tools/tests/test_p0_audio_device.py

51 行；SHA256 `0be9b31adc1849653ac4ff103a671c3fc76ba4905b1ba1a619f508e8a2c12abf`

导入：`copy`, `ctypes`, `p0_audio_device`, `pathlib`, `sys`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| function samples | [源码:11](../../../tools/tests/test_p0_audio_device.py#L11) · `def samples()` |
| class AudioDeviceTests | [源码:17](../../../tools/tests/test_p0_audio_device.py#L17) · `unittest.TestCase` |
| function AudioDeviceTests.test_clock_analysis_rejects_pause_drift_and_resume_catchup | [源码:18](../../../tools/tests/test_p0_audio_device.py#L18) · `def test_clock_analysis_rejects_pause_drift_and_resume_catchup(self)` |
| function AudioDeviceTests.test_missing_inaccurate_and_nonfinite_samples_cannot_pass | [源码:30](../../../tools/tests/test_p0_audio_device.py#L30) · `def test_missing_inaccurate_and_nonfinite_samples_cannot_pass(self)` |
| function AudioDeviceTests.test_public_abi_layout_uses_fixed_width_fields | [源码:46](../../../tools/tests/test_p0_audio_device.py#L46) · `def test_public_abi_layout_uses_fixed_width_fields(self)` |

## tools/tests/test_p0_benchmark.py

103 行；SHA256 `7f6c78963b68652c73c0ce19f160edd29100ab84a49751a9a3938c115e8ccf69`

导入：`check_p0_gui`, `copy`, `p0_process_metrics`, `pathlib`, `sys`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| class BenchmarkResultTests | [源码:11](../../../tools/tests/test_p0_benchmark.py#L11) · `unittest.TestCase` |
| function BenchmarkResultTests.test_cycle_cpu_counts_work_and_wall_time_without_idle_or_round_boundary_loss | [源码:12](../../../tools/tests/test_p0_benchmark.py#L12) · `def test_cycle_cpu_counts_work_and_wall_time_without_idle_or_round_boundary_loss(self)` |
| function BenchmarkResultTests.test_page_summary_excludes_reserved_and_does_not_claim_commit_ownership | [源码:24](../../../tools/tests/test_p0_benchmark.py#L24) · `def test_page_summary_excludes_reserved_and_does_not_claim_commit_ownership(self)` |
| function BenchmarkResultTests.inputs | [源码:34](../../../tools/tests/test_p0_benchmark.py#L34) · `def inputs(self)` |
| function BenchmarkResultTests.test_no_readback_never_claims_pixel_or_latency_verification | [源码:51](../../../tools/tests/test_p0_benchmark.py#L51) · `def test_no_readback_never_claims_pixel_or_latency_verification(self)` |
| function BenchmarkResultTests.test_live_refs_and_continuing_idle_frames_fail_separately | [源码:61](../../../tools/tests/test_p0_benchmark.py#L61) · `def test_live_refs_and_continuing_idle_frames_fail_separately(self)` |
| function BenchmarkResultTests.test_missing_or_over_budget_metrics_do_not_pass | [源码:72](../../../tools/tests/test_p0_benchmark.py#L72) · `def test_missing_or_over_budget_metrics_do_not_pass(self)` |
| function BenchmarkResultTests.test_async_limits_and_stale_results_require_evidence | [源码:84](../../../tools/tests/test_p0_benchmark.py#L84) · `def test_async_limits_and_stale_results_require_evidence(self)` |

## tools/tests/test_p0_combined_soak.py

74 行；SHA256 `958e1f603ea657ed4e195e41c849482a4a549a9934f6865f17aaa3158ae11c5f`

导入：`check_p0_combined_soak`, `json`, `pathlib`, `sys`, `tempfile`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| class Reference | [源码:11](../../../tools/tests/test_p0_combined_soak.py#L11) |
| function Reference.check | [源码:12](../../../tools/tests/test_p0_combined_soak.py#L12) · `def check(self, packet)` |
| class CombinedSoakTests | [源码:16](../../../tools/tests/test_p0_combined_soak.py#L16) · `unittest.TestCase` |
| function CombinedSoakTests.trace | [源码:17](../../../tools/tests/test_p0_combined_soak.py#L17) · `def trace(self, events)` |
| function CombinedSoakTests.events | [源码:23](../../../tools/tests/test_p0_combined_soak.py#L23) · `def events(self)` |
| function CombinedSoakTests.test_complete_native_lifetime_accounts_for_unread_tail | [源码:30](../../../tools/tests/test_p0_combined_soak.py#L30) · `def test_complete_native_lifetime_accounts_for_unread_tail(self)` |
| function CombinedSoakTests.test_discontinuity_or_wrong_samples_are_rejected | [源码:33](../../../tools/tests/test_p0_combined_soak.py#L33) · `def test_discontinuity_or_wrong_samples_are_rejected(self)` |
| function CombinedSoakTests.test_paused_native_submission_and_unaccounted_tail_are_rejected | [源码:40](../../../tools/tests/test_p0_combined_soak.py#L40) · `def test_paused_native_submission_and_unaccounted_tail_are_rejected(self)` |
| function CombinedSoakTests.test_incomplete_retirement_is_rejected | [源码:46](../../../tools/tests/test_p0_combined_soak.py#L46) · `def test_incomplete_retirement_is_rejected(self)` |
| function CombinedSoakTests.test_nonfinite_timing_is_rejected_and_percentile_uses_rank | [源码:49](../../../tools/tests/test_p0_combined_soak.py#L49) · `def test_nonfinite_timing_is_rejected_and_percentile_uses_rank(self)` |
| function CombinedSoakTests.test_phase_transition_guard_excludes_early_samples | [源码:53](../../../tools/tests/test_p0_combined_soak.py#L53) · `def test_phase_transition_guard_excludes_early_samples(self)` |
| function CombinedSoakTests.test_resident_budget_is_not_private_commit_budget | [源码:63](../../../tools/tests/test_p0_combined_soak.py#L63) · `def test_resident_budget_is_not_private_commit_budget(self)` |

## tools/tests/test_p0_composition_boundary.py

20 行；SHA256 `a810c172d1321d136a7990db7638e60617a38c9e7b1b628a0d72db833800a317`

导入：`check_p0_composition`, `pathlib`, `sys`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| class CompositionBoundaryTests | [源码:9](../../../tools/tests/test_p0_composition_boundary.py#L9) · `unittest.TestCase` |
| function CompositionBoundaryTests.test_accepts_pure_math_and_detects_reverse_or_platform_imports | [源码:10](../../../tools/tests/test_p0_composition_boundary.py#L10) · `def test_accepts_pure_math_and_detects_reverse_or_platform_imports(self)` |

## tools/tests/test_p0_compressed.py

86 行；SHA256 `3d21937c5e6af6dcad07984d322902b33ecea409cb2521fc8673ce01a41bd867`

导入：`array`, `check_p0_compressed`, `copy`, `json`, `p0_compressed_fixture`, `p0_stable_batch`, `pathlib`, `sys`, `tempfile`, `unittest`, `unittest.mock`, `zipfile`

| 定义 | 位置／签名 |
| --- | --- |
| class CompressedEvidenceTests | [源码:17](../../../tools/tests/test_p0_compressed.py#L17) · `unittest.TestCase` |
| function CompressedEvidenceTests.test_independent_tail_disagreement_is_not_silently_trimmed | [源码:18](../../../tools/tests/test_p0_compressed.py#L18) · `def test_independent_tail_disagreement_is_not_silently_trimmed(self)` |
| function CompressedEvidenceTests.test_nonfinite_and_corrupt_float_data_are_rejected | [源码:25](../../../tools/tests/test_p0_compressed.py#L25) · `def test_nonfinite_and_corrupt_float_data_are_rejected(self)` |
| function CompressedEvidenceTests.test_only_reviewed_optional_codec_features_are_allowed | [源码:33](../../../tools/tests/test_p0_compressed.py#L33) · `def test_only_reviewed_optional_codec_features_are_allowed(self)` |
| function CompressedEvidenceTests.test_both_codec_references_have_distinct_files_and_generation_never_overwrites | [源码:38](../../../tools/tests/test_p0_compressed.py#L38) · `def test_both_codec_references_have_distinct_files_and_generation_never_overwrites(self)` |
| function CompressedEvidenceTests.test_both_codec_references_have_distinct_files_and_generation_never_overwrites.run | [源码:41](../../../tools/tests/test_p0_compressed.py#L41) · `def run(command, stdout=None, **kwargs)` |
| class StableBatchTests | [源码:51](../../../tools/tests/test_p0_compressed.py#L51) · `unittest.TestCase` |
| function StableBatchTests.context | [源码:52](../../../tools/tests/test_p0_compressed.py#L52) · `def context(self, root)` |
| function StableBatchTests.test_packages_have_matching_baselines_and_remain_unexecuted | [源码:56](../../../tools/tests/test_p0_compressed.py#L56) · `def test_packages_have_matching_baselines_and_remain_unexecuted(self)` |
| function StableBatchTests.test_output_escape_overwrite_and_failed_preparation_preserve_original | [源码:75](../../../tools/tests/test_p0_compressed.py#L75) · `def test_output_escape_overwrite_and_failed_preparation_preserve_original(self)` |

## tools/tests/test_p0_compressed_soak.py

134 行；SHA256 `8f84ce893c36ae4ef9f903997d60dbb181737673c2eaba8f1374a4b2b56705bf`

导入：`array`, `check_p0_combined_soak`, `check_p0_compressed`, `check_p0_compressed_soak`, `copy`, `io`, `json`, `pathlib`, `sys`, `tempfile`, `types`, `unittest`, `unittest.mock`

| 定义 | 位置／签名 |
| --- | --- |
| class CompressedSoakTests | [源码:18](../../../tools/tests/test_p0_compressed_soak.py#L18) · `unittest.TestCase` |
| function CompressedSoakTests.matrix | [源码:19](../../../tools/tests/test_p0_compressed_soak.py#L19) · `def matrix(self)` |
| function CompressedSoakTests.test_matrix_requires_all_distinct_repetitions_and_all_budget_fields | [源码:23](../../../tools/tests/test_p0_compressed_soak.py#L23) · `def test_matrix_requires_all_distinct_repetitions_and_all_budget_fields(self)` |
| function CompressedSoakTests.test_smoke_has_no_budget_claim_and_feature_mode_cannot_be_a_wav_stub | [源码:36](../../../tools/tests/test_p0_compressed_soak.py#L36) · `def test_smoke_has_no_budget_claim_and_feature_mode_cannot_be_a_wav_stub(self)` |
| function CompressedSoakTests.test_reference_can_check_late_frames_without_retaining_the_full_file | [源码:48](../../../tools/tests/test_p0_compressed_soak.py#L48) · `def test_reference_can_check_late_frames_without_retaining_the_full_file(self)` |
| class CompressedSoakTests.test_reference_can_check_late_frames_without_retaining_the_full_file.Zeros | [源码:50](../../../tools/tests/test_p0_compressed_soak.py#L50) · `io.BytesIO` |
| function CompressedSoakTests.test_reference_can_check_late_frames_without_retaining_the_full_file.Zeros.seek | [源码:51](../../../tools/tests/test_p0_compressed_soak.py#L51) · `def seek(self, pos)` |
| function CompressedSoakTests.test_reference_can_check_late_frames_without_retaining_the_full_file.Zeros.read | [源码:52](../../../tools/tests/test_p0_compressed_soak.py#L52) · `def read(self, count)` |
| class CompressedSoakTests.test_reference_can_check_late_frames_without_retaining_the_full_file.Sparse | [源码:53](../../../tools/tests/test_p0_compressed_soak.py#L53) |
| function CompressedSoakTests.test_reference_can_check_late_frames_without_retaining_the_full_file.Sparse.stat | [源码:54](../../../tools/tests/test_p0_compressed_soak.py#L54) · `def stat(self)` |
| function CompressedSoakTests.test_reference_can_check_late_frames_without_retaining_the_full_file.Sparse.open | [源码:55](../../../tools/tests/test_p0_compressed_soak.py#L55) · `def open(self, mode)` |
| function CompressedSoakTests.test_reference_rejects_wrong_length_and_unbounded_duration | [源码:64](../../../tools/tests/test_p0_compressed_soak.py#L64) · `def test_reference_rejects_wrong_length_and_unbounded_duration(self)` |
| function CompressedSoakTests.fixture | [源码:71](../../../tools/tests/test_p0_compressed_soak.py#L71) · `def fixture(self, root)` |
| function CompressedSoakTests.test_sources_and_independent_full_pcm_proofs_are_required_before_device_work | [源码:93](../../../tools/tests/test_p0_compressed_soak.py#L93) · `def test_sources_and_independent_full_pcm_proofs_are_required_before_device_work(self)` |
| function CompressedSoakTests.test_path_escape_mismatched_decoder_and_trimmed_reference_cannot_pass | [源码:107](../../../tools/tests/test_p0_compressed_soak.py#L107) · `def test_path_escape_mismatched_decoder_and_trimmed_reference_cannot_pass(self)` |
| function CompressedSoakTests.test_preparation_refuses_escape_and_existing_directory_without_starting_encoder | [源码:123](../../../tools/tests/test_p0_compressed_soak.py#L123) · `def test_preparation_refuses_escape_and_existing_directory_without_starting_encoder(self)` |

## tools/tests/test_p0_desktop.py

99 行；SHA256 `2899cbf3a9109b389c425f39444ab7fa242f415c3bc383ae208aed6839500179`

导入：`check_p0_desktop`, `check_p0_gui`, `pathlib`, `sys`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| function fixture | [源码:10](../../../tools/tests/test_p0_desktop.py#L10) · `def fixture()` |
| class DesktopTests | [源码:23](../../../tools/tests/test_p0_desktop.py#L23) · `unittest.TestCase` |
| function DesktopTests.test_native_clipboard_cannot_silently_fall_back_to_application_only | [源码:24](../../../tools/tests/test_p0_desktop.py#L24) · `def test_native_clipboard_cannot_silently_fall_back_to_application_only(self)` |
| function DesktopTests.test_manual_coverage_requires_events_as_well_as_claims | [源码:33](../../../tools/tests/test_p0_desktop.py#L33) · `def test_manual_coverage_requires_events_as_well_as_claims(self)` |
| function DesktopTests.test_smoke_or_synthetic_input_never_certifies_manual_acceptance | [源码:48](../../../tools/tests/test_p0_desktop.py#L48) · `def test_smoke_or_synthetic_input_never_certifies_manual_acceptance(self)` |
| function DesktopTests.test_ui_zoom_does_not_prove_native_dpi_transition | [源码:62](../../../tools/tests/test_p0_desktop.py#L62) · `def test_ui_zoom_does_not_prove_native_dpi_transition(self)` |
| function DesktopTests.test_text_payload_extra_fields_missing_events_and_source_mismatch_rejected | [源码:74](../../../tools/tests/test_p0_desktop.py#L74) · `def test_text_payload_extra_fields_missing_events_and_source_mismatch_rejected(self)` |
| function DesktopTests.test_input_leak_and_unavailable_environment_cannot_pass | [源码:85](../../../tools/tests/test_p0_desktop.py#L85) · `def test_input_leak_and_unavailable_environment_cannot_pass(self)` |

## tools/tests/test_p0_edit_composition.py

48 行；SHA256 `d5cb5e164ebb98ce0868976f2f61da5707fc23e35bb18f88257d0e00bdf8f022`

导入：`check_p0_edit_composition`, `copy`, `pathlib`, `sys`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| class EditCompositionTests | [源码:10](../../../tools/tests/test_p0_edit_composition.py#L10) · `unittest.TestCase` |
| function EditCompositionTests.test_requires_real_edit_coverage_source_reuse_and_both_pixel_proofs | [源码:11](../../../tools/tests/test_p0_edit_composition.py#L11) · `def test_requires_real_edit_coverage_source_reuse_and_both_pixel_proofs(self)` |

## tools/tests/test_p0_evidence.py

97 行；SHA256 `9b7ad82afb16d1c7f98ebf536b57a0c22ae86887baa7d41900554483c8836e04`

导入：`json`, `p0_evidence`, `pathlib`, `sys`, `tempfile`, `unittest`, `wave`

| 定义 | 位置／签名 |
| --- | --- |
| class EvidenceTests | [源码:12](../../../tools/tests/test_p0_evidence.py#L12) · `unittest.TestCase` |
| function EvidenceTests.setUp | [源码:13](../../../tools/tests/test_p0_evidence.py#L13) · `def setUp(self)` |
| function EvidenceTests.tearDown | [源码:19](../../../tools/tests/test_p0_evidence.py#L19) · `def tearDown(self)` |
| function EvidenceTests.client | [源码:23](../../../tools/tests/test_p0_evidence.py#L23) · `def client(self)` |
| function EvidenceTests.test_inspection_is_read_only_and_excludes_sensitive_sources | [源码:33](../../../tools/tests/test_p0_evidence.py#L33) · `def test_inspection_is_read_only_and_excludes_sensitive_sources(self)` |
| function EvidenceTests.test_historical_log_tokens_are_not_current_release_proof | [源码:43](../../../tools/tests/test_p0_evidence.py#L43) · `def test_historical_log_tokens_are_not_current_release_proof(self)` |
| function EvidenceTests.test_fixtures_are_deterministic_self_contained_and_unverified | [源码:50](../../../tools/tests/test_p0_evidence.py#L50) · `def test_fixtures_are_deterministic_self_contained_and_unverified(self)` |
| function EvidenceTests.test_registration_preserves_image_and_cannot_pass_a_case | [源码:63](../../../tools/tests/test_p0_evidence.py#L63) · `def test_registration_preserves_image_and_cannot_pass_a_case(self)` |
| function EvidenceTests.test_existing_outputs_and_escape_paths_are_rejected | [源码:76](../../../tools/tests/test_p0_evidence.py#L76) · `def test_existing_outputs_and_escape_paths_are_rejected(self)` |
| function EvidenceTests.test_invalid_or_corrupted_images_are_rejected | [源码:87](../../../tools/tests/test_p0_evidence.py#L87) · `def test_invalid_or_corrupted_images_are_rejected(self)` |

## tools/tests/test_p0_frame_attribution.py

29 行；SHA256 `a3561f7d87f1b9dfce3e7f79bddffb82b86d7fd3ea743533b7bd5fdb716e1698`

导入：`copy`, `p0_frame_attribution`, `pathlib`, `sys`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| class FrameAttributionTests | [源码:10](../../../tools/tests/test_p0_frame_attribution.py#L10) · `unittest.TestCase` |
| function FrameAttributionTests.test_previous_cpu_joins_previous_frame_and_ambiguous_samples_fail | [源码:11](../../../tools/tests/test_p0_frame_attribution.py#L11) · `def test_previous_cpu_joins_previous_frame_and_ambiguous_samples_fail(self)` |

## tools/tests/test_p0_heap_analysis.py

70 行；SHA256 `3f86d19a73dbb57b71eb09b15ccfb8aeccf44dac65832497510f26c9b55bd381`

导入：`p0_heap_analysis`, `pathlib`, `sys`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| class HeapAnalysisTests | [源码:9](../../../tools/tests/test_p0_heap_analysis.py#L9) · `unittest.TestCase` |
| function HeapAnalysisTests.test_moving_realloc_completion_keeps_internal_allocation_stack | [源码:10](../../../tools/tests/test_p0_heap_analysis.py#L10) · `def test_moving_realloc_completion_keeps_internal_allocation_stack(self)` |
| function HeapAnalysisTests.test_reallocation_free_and_unknown_pretrace_blocks | [源码:24](../../../tools/tests/test_p0_heap_analysis.py#L24) · `def test_reallocation_free_and_unknown_pretrace_blocks(self)` |
| function HeapAnalysisTests.test_module_origin_skips_system_and_rejects_other_thread_stack | [源码:34](../../../tools/tests/test_p0_heap_analysis.py#L34) · `def test_module_origin_skips_system_and_rejects_other_thread_stack(self)` |
| function HeapAnalysisTests.test_failed_realloc_preserves_old_block_and_destroy_releases_it | [源码:44](../../../tools/tests/test_p0_heap_analysis.py#L44) · `def test_failed_realloc_preserves_old_block_and_destroy_releases_it(self)` |
| function HeapAnalysisTests.test_nvidia_overlay_is_not_classified_as_driver | [源码:53](../../../tools/tests/test_p0_heap_analysis.py#L53) · `def test_nvidia_overlay_is_not_classified_as_driver(self)` |
| function HeapAnalysisTests.test_reported_segments_are_separate_from_requested_allocations | [源码:59](../../../tools/tests/test_p0_heap_analysis.py#L59) · `def test_reported_segments_are_separate_from_requested_allocations(self)` |

## tools/tests/test_p0_interaction.py

44 行；SHA256 `8bf937fd0b36a98b82a30d40b57b97c56525a4ed925bace6361aec979056804c`

导入：`check_p0_composition`, `check_p0_interaction`, `copy`, `pathlib`, `sys`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| class InteractionEvidenceTests | [源码:11](../../../tools/tests/test_p0_interaction.py#L11) · `unittest.TestCase` |
| function InteractionEvidenceTests.test_gesture_proof_rejects_partial_cancel_extra_commits_and_clock_or_input_leaks | [源码:12](../../../tools/tests/test_p0_interaction.py#L12) · `def test_gesture_proof_rejects_partial_cancel_extra_commits_and_clock_or_input_leaks(self)` |
| function InteractionEvidenceTests.test_scale_proof_rejects_missing_documents_insufficient_samples_and_pixel_errors | [源码:29](../../../tools/tests/test_p0_interaction.py#L29) · `def test_scale_proof_rejects_missing_documents_insufficient_samples_and_pixel_errors(self)` |

## tools/tests/test_p0_lifetime.py

35 行；SHA256 `20d751702ba31b4d9da9a577a1d7acbbc0bbfc48a19b49c5fac0dfec3f549b7c`

导入：`p0_lifetime_analysis`, `pathlib`, `sys`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| class LifetimeTests | [源码:9](../../../tools/tests/test_p0_lifetime.py#L9) · `unittest.TestCase` |
| function LifetimeTests.test_release_alignment_does_not_use_pre_release_or_distant_samples | [源码:10](../../../tools/tests/test_p0_lifetime.py#L10) · `def test_release_alignment_does_not_use_pre_release_or_distant_samples(self)` |
| function LifetimeTests.test_multiple_images_share_receipt_and_payload_is_not_double_counted | [源码:19](../../../tools/tests/test_p0_lifetime.py#L19) · `def test_multiple_images_share_receipt_and_payload_is_not_double_counted(self)` |
| function LifetimeTests.test_missing_completion_remains_outstanding | [源码:29](../../../tools/tests/test_p0_lifetime.py#L29) · `def test_missing_completion_remains_outstanding(self)` |

## tools/tests/test_p0_media.py

114 行；SHA256 `80390b148a9c0c90b4092fe89ef744444ceb96347cf13afd904adf7b27717021`

导入：`check_p0`, `check_p0_media`, `check_p0_transport`, `copy`, `ctypes`, `json`, `pathlib`, `struct`, `sys`, `tempfile`, `unittest`, `wave`

| 定义 | 位置／签名 |
| --- | --- |
| function record | [源码:17](../../../tools/tests/test_p0_media.py#L17) · `def record(fixture)` |
| function record.stats | [源码:20](../../../tools/tests/test_p0_media.py#L20) · `def stats(base, submitted)` |
| function record.add | [源码:25](../../../tools/tests/test_p0_media.py#L25) · `def add(epoch, base, device, phase, submitted, count=1)` |
| function record.frame | [源码:29](../../../tools/tests/test_p0_media.py#L29) · `def frame(position)` |
| class MediaTests | [源码:50](../../../tools/tests/test_p0_media.py#L50) · `unittest.TestCase` |
| function MediaTests.test_evidence_inventory_keeps_optional_local_media_package_reproducible | [源码:51](../../../tools/tests/test_p0_media.py#L51) · `def test_evidence_inventory_keeps_optional_local_media_package_reproducible(self)` |
| function MediaTests.test_boundary_rejects_native_ui_files_clocks_and_unreviewed_decoder_features | [源码:57](../../../tools/tests/test_p0_media.py#L57) · `def test_boundary_rejects_native_ui_files_clocks_and_unreviewed_decoder_features(self)` |
| function MediaTests.test_fixture_has_exact_wave_shape_unicode_reference_and_never_overwrites | [源码:72](../../../tools/tests/test_p0_media.py#L72) · `def test_fixture_has_exact_wave_shape_unicode_reference_and_never_overwrites(self)` |
| function MediaTests.test_reference_distinguishes_stereo_gain_source_position_and_eof | [源码:84](../../../tools/tests/test_p0_media.py#L84) · `def test_reference_distinguishes_stereo_gain_source_position_and_eof(self)` |
| function MediaTests.test_native_evidence_rejects_wrong_clock_content_pause_seek_and_event_replay | [源码:94](../../../tools/tests/test_p0_media.py#L94) · `def test_native_evidence_rejects_wrong_clock_content_pause_seek_and_event_replay(self)` |

## tools/tests/test_p0_pcm.py

82 行；SHA256 `93c6678710efd2281094b1725356ea399203f8acaa06858171fcefeea5c79e27`

导入：`check_p0_pcm`, `copy`, `math`, `pathlib`, `sys`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| function fixture | [源码:11](../../../tools/tests/test_p0_pcm.py#L11) · `def fixture()` |
| function fixture.packet | [源码:12](../../../tools/tests/test_p0_pcm.py#L12) · `def packet(start)` |
| function fixture.stats | [源码:17](../../../tools/tests/test_p0_pcm.py#L17) · `def stats(base, submitted)` |
| function fixture.add | [源码:25](../../../tools/tests/test_p0_pcm.py#L25) · `def add(epoch, base, request, phase, device, submitted, count=1)` |
| class PcmTests | [源码:49](../../../tools/tests/test_p0_pcm.py#L49) · `unittest.TestCase` |
| function PcmTests.test_independent_pcm_reference_has_known_tone_peak_zero_and_quiet_tail | [源码:50](../../../tools/tests/test_p0_pcm.py#L50) · `def test_independent_pcm_reference_has_known_tone_peak_zero_and_quiet_tail(self)` |
| function PcmTests.test_evidence_rejects_wrong_content_clock_seek_and_hearing_claim | [源码:56](../../../tools/tests/test_p0_pcm.py#L56) · `def test_evidence_rejects_wrong_content_clock_seek_and_hearing_claim(self)` |

## tools/tests/test_p0_recovery.py

54 行；SHA256 `57994772fca9521fa436edfafcc39a8d05b184c43c0c1b0b7d6c49dfd1a38c34`

导入：`check_p0_gui`, `check_p0_recovery`, `copy`, `pathlib`, `sys`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| class RecoveryTests | [源码:11](../../../tools/tests/test_p0_recovery.py#L11) · `unittest.TestCase` |
| function RecoveryTests.test_audio_port_rejects_native_ui_io_and_wall_clock_imports | [源码:12](../../../tools/tests/test_p0_recovery.py#L12) · `def test_audio_port_rejects_native_ui_io_and_wall_clock_imports(self)` |
| function RecoveryTests.test_recovery_evidence_requires_anchored_epochs_frozen_edits_and_fresh_inputs | [源码:19](../../../tools/tests/test_p0_recovery.py#L19) · `def test_recovery_evidence_requires_anchored_epochs_frozen_edits_and_fresh_inputs(self)` |

## tools/tests/test_p0_repeats.py

49 行；SHA256 `71b47625ccc5f012a968c382c709796dbe84e249b2cfb213ae14484c44156225`

导入：`p0_gpu_repeats`, `p0_modules`, `pathlib`, `sys`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| class RepetitionTests | [源码:10](../../../tools/tests/test_p0_repeats.py#L10) · `unittest.TestCase` |
| function RepetitionTests.test_orders_balance_each_position_and_condition | [源码:11](../../../tools/tests/test_p0_repeats.py#L11) · `def test_orders_balance_each_position_and_condition(self)` |
| function RepetitionTests.test_ranges_expose_missing_values_and_do_not_pool_trials | [源码:17](../../../tools/tests/test_p0_repeats.py#L17) · `def test_ranges_expose_missing_values_and_do_not_pool_trials(self)` |
| function RepetitionTests.test_cycle_latency_excludes_initial_completions_and_requires_uninstrumented_runs | [源码:25](../../../tools/tests/test_p0_repeats.py#L25) · `def test_cycle_latency_excludes_initial_completions_and_requires_uninstrumented_runs(self)` |
| function RepetitionTests.test_failed_module_snapshot_never_means_clean_graphics_environment | [源码:38](../../../tools/tests/test_p0_repeats.py#L38) · `def test_failed_module_snapshot_never_means_clean_graphics_environment(self)` |

## tools/tests/test_p0_resource_cycles.py

100 行；SHA256 `a2b1217694083db4ecd22bc03b18e7d1993618150701a6c2da9c519569d95c5f`

导入：`check_p0_resource_cycles`, `copy`, `pathlib`, `sys`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| function fixture | [源码:10](../../../tools/tests/test_p0_resource_cycles.py#L10) · `def fixture()` |
| class ResourceCycleTests | [源码:30](../../../tools/tests/test_p0_resource_cycles.py#L30) · `unittest.TestCase` |
| function ResourceCycleTests.test_complete_independent_cycles_are_accepted | [源码:31](../../../tools/tests/test_p0_resource_cycles.py#L31) · `def test_complete_independent_cycles_are_accepted(self)` |
| function ResourceCycleTests.test_missing_or_reused_cycle_cannot_claim_twenty_cycles | [源码:34](../../../tools/tests/test_p0_resource_cycles.py#L34) · `def test_missing_or_reused_cycle_cannot_claim_twenty_cycles(self)` |
| function ResourceCycleTests.test_wrong_document_and_trace_counts_are_rejected | [源码:40](../../../tools/tests/test_p0_resource_cycles.py#L40) · `def test_wrong_document_and_trace_counts_are_rejected(self)` |
| function ResourceCycleTests.test_unretired_blocks_faults_and_pending_graphics_are_rejected | [源码:46](../../../tools/tests/test_p0_resource_cycles.py#L46) · `def test_unretired_blocks_faults_and_pending_graphics_are_rejected(self)` |
| function ResourceCycleTests.test_fresh_paused_device_sequence_and_position_are_both_required | [源码:53](../../../tools/tests/test_p0_resource_cycles.py#L53) · `def test_fresh_paused_device_sequence_and_position_are_both_required(self)` |
| function ResourceCycleTests.test_final_pixel_proof_cannot_be_omitted | [源码:58](../../../tools/tests/test_p0_resource_cycles.py#L58) · `def test_final_pixel_proof_cannot_be_omitted(self)` |
| function ResourceCycleTests.test_repeated_phase_names_keep_cycle_and_transition_identity | [源码:62](../../../tools/tests/test_p0_resource_cycles.py#L62) · `def test_repeated_phase_names_keep_cycle_and_transition_identity(self)` |
| function ResourceCycleTests.test_active_cpu_does_not_include_intervening_closed_time | [源码:67](../../../tools/tests/test_p0_resource_cycles.py#L67) · `def test_active_cpu_does_not_include_intervening_closed_time(self)` |
| function ResourceCycleTests.test_memory_trends_distinguish_flat_and_accumulating_baselines | [源码:78](../../../tools/tests/test_p0_resource_cycles.py#L78) · `def test_memory_trends_distinguish_flat_and_accumulating_baselines(self)` |
| function ResourceCycleTests.test_unavailable_gpu_counter_remains_unknown | [源码:86](../../../tools/tests/test_p0_resource_cycles.py#L86) · `def test_unavailable_gpu_counter_remains_unknown(self)` |
| function ResourceCycleTests.test_short_observation_does_not_pass_resource_gates | [源码:90](../../../tools/tests/test_p0_resource_cycles.py#L90) · `def test_short_observation_does_not_pass_resource_gates(self)` |

## tools/tests/test_p0_stream_output.py

105 行；SHA256 `2eb2e3649e4eb1a1bb783c17a97188d1247eee3f4c33bbe5c4d949fd36935168`

导入：`check_p0_stream_output`, `copy`, `pathlib`, `sys`, `tempfile`, `unittest`, `wave`

| 定义 | 位置／签名 |
| --- | --- |
| function stats | [源码:12](../../../tools/tests/test_p0_stream_output.py#L12) · `def stats(generation, origin=0, submitted=0)` |
| function record | [源码:19](../../../tools/tests/test_p0_stream_output.py#L19) · `def record(ref)` |
| function record.event | [源码:21](../../../tools/tests/test_p0_stream_output.py#L21) · `def event(kind, gen, frame, count=0, drop=0, packet=None)` |
| function record.submit | [源码:23](../../../tools/tests/test_p0_stream_output.py#L23) · `def submit(gen, frame, count)` |
| function record.row | [源码:24](../../../tools/tests/test_p0_stream_output.py#L24) · `def row(epoch, device, simulation, phase, q)` |
| function record.pause | [源码:27](../../../tools/tests/test_p0_stream_output.py#L27) · `def pause(name, gen, epoch, position, device, origin)` |
| class StreamOutputTools | [源码:60](../../../tools/tests/test_p0_stream_output.py#L60) · `unittest.TestCase` |
| function StreamOutputTools.setUpClass | [源码:62](../../../tools/tests/test_p0_stream_output.py#L62) · `def setUpClass(cls)` |
| function StreamOutputTools.tearDownClass | [源码:66](../../../tools/tests/test_p0_stream_output.py#L66) · `def tearDownClass(cls)` |
| function StreamOutputTools.test_fixture_is_self_authored_exact_length_and_never_overwrites | [源码:68](../../../tools/tests/test_p0_stream_output.py#L68) · `def test_fixture_is_self_authored_exact_length_and_never_overwrites(self)` |
| function StreamOutputTools.test_decoder_boundary_keeps_native_ui_files_and_wall_clock_out | [源码:75](../../../tools/tests/test_p0_stream_output.py#L75) · `def test_decoder_boundary_keeps_native_ui_files_and_wall_clock_out(self)` |
| function StreamOutputTools.test_independent_reference_checks_all_channel_values_and_gain | [源码:80](../../../tools/tests/test_p0_stream_output.py#L80) · `def test_independent_reference_checks_all_channel_values_and_gain(self)` |
| function StreamOutputTools.test_native_trace_rejects_old_generation_discontinuity_wrong_content_and_unrejected_holds | [源码:85](../../../tools/tests/test_p0_stream_output.py#L85) · `def test_native_trace_rejects_old_generation_discontinuity_wrong_content_and_unrejected_holds(self)` |
| function StreamOutputTools.test_pause_reset_clock_pool_and_recovery_evidence_are_independently_required | [源码:94](../../../tools/tests/test_p0_stream_output.py#L94) · `def test_pause_reset_clock_pool_and_recovery_evidence_are_independently_required(self)` |

## tools/tests/test_p0_streaming.py

82 行；SHA256 `b08ad8a17bcfab6cd3173280b15c264f5494082ee94097a450760fcd86d0b111`

导入：`check_p0_streaming`, `copy`, `math`, `pathlib`, `struct`, `sys`, `tempfile`, `unittest`, `wave`

| 定义 | 位置／签名 |
| --- | --- |
| function record | [源码:14](../../../tools/tests/test_p0_streaming.py#L14) · `def record(ref, mode='stream')` |
| class StreamingTools | [源码:26](../../../tools/tests/test_p0_streaming.py#L26) · `unittest.TestCase` |
| function StreamingTools.test_generator_is_bounded_self_authored_and_preserves_existing_inputs | [源码:27](../../../tools/tests/test_p0_streaming.py#L27) · `def test_generator_is_bounded_self_authored_and_preserves_existing_inputs(self)` |
| function StreamingTools.test_independent_pcm_reference_preserves_signed_channels_and_float_bits | [源码:37](../../../tools/tests/test_p0_streaming.py#L37) · `def test_independent_pcm_reference_preserves_signed_channels_and_float_bits(self)` |
| function StreamingTools.test_evidence_rejects_full_read_ahead_stale_pcm_wrong_identity_and_false_device_claims | [源码:42](../../../tools/tests/test_p0_streaming.py#L42) · `def test_evidence_rejects_full_read_ahead_stale_pcm_wrong_identity_and_false_device_claims(self)` |
| function StreamingTools.test_metrics_require_fresh_baseline_and_separate_sampled_and_process_peaks | [源码:55](../../../tools/tests/test_p0_streaming.py#L55) · `def test_metrics_require_fresh_baseline_and_separate_sampled_and_process_peaks(self)` |
| function StreamingTools.test_delayed_metrics_setup_uses_explicit_idle_baseline_not_process_start_time | [源码:63](../../../tools/tests/test_p0_streaming.py#L63) · `def test_delayed_metrics_setup_uses_explicit_idle_baseline_not_process_start_time(self)` |
| function StreamingTools.test_comparison_fails_when_growth_is_unbounded_or_baseline_cost_is_missing | [源码:69](../../../tools/tests/test_p0_streaming.py#L69) · `def test_comparison_fails_when_growth_is_unbounded_or_baseline_cost_is_missing(self)` |

## tools/tests/test_p0_switch.py

49 行；SHA256 `45e768fccbfacc0557fcd9708eafb4f62b5dd3eed26ef3b3f5bd640205abdba3`

导入：`check_p0_switch`, `copy`, `pathlib`, `sys`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| class SwitchTests | [源码:10](../../../tools/tests/test_p0_switch.py#L10) · `unittest.TestCase` |
| function SwitchTests.test_requires_retention_generation_uploads_and_matching_document_pixels | [源码:11](../../../tools/tests/test_p0_switch.py#L11) · `def test_requires_retention_generation_uploads_and_matching_document_pixels(self)` |
| function SwitchTests.test_phase_cpu_does_not_include_loading_or_close_transition | [源码:38](../../../tools/tests/test_p0_switch.py#L38) · `def test_phase_cpu_does_not_include_loading_or_close_transition(self)` |
| function SwitchTests.test_phase_cpu_does_not_include_loading_or_close_transition.sample | [源码:39](../../../tools/tests/test_p0_switch.py#L39) · `def sample(phase, seconds, cpu)` |

## tools/tests/test_p0_switch_audio.py

39 行；SHA256 `21954c23f973a933a48e707caa674a3813b239523d6d55aad580ba5df7255e9e`

导入：`check_p0_switch_audio`, `copy`, `pathlib`, `sys`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| class SharedClockEvidenceTests | [源码:10](../../../tools/tests/test_p0_switch_audio.py#L10) · `unittest.TestCase` |
| function SharedClockEvidenceTests.test_switch_evidence_requires_one_clock_fresh_input_and_unchanged_old_documents | [源码:11](../../../tools/tests/test_p0_switch_audio.py#L11) · `def test_switch_evidence_requires_one_clock_fresh_input_and_unchanged_old_documents(self)` |

## tools/tests/test_p0_transport.py

70 行；SHA256 `b87d8fdaa47e6e58ee97aa861a65c618f3a4bc13ea1b11b9a9ea2641630a7882`

导入：`check_p0_gui`, `check_p0_transport`, `copy`, `pathlib`, `sys`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| class TransportTests | [源码:11](../../../tools/tests/test_p0_transport.py#L11) · `unittest.TestCase` |
| function TransportTests.test_audio_boundary_rejects_ui_files_native_leaks_and_extra_apis | [源码:12](../../../tools/tests/test_p0_transport.py#L12) · `def test_audio_boundary_rejects_ui_files_native_leaks_and_extra_apis(self)` |
| function TransportTests.test_transport_proof_requires_fresh_device_clock_input_history_and_pixels | [源码:32](../../../tools/tests/test_p0_transport.py#L32) · `def test_transport_proof_requires_fresh_device_clock_input_history_and_pixels(self)` |

## tools/tests/test_p0_vorbis_tail.py

46 行；SHA256 `8cfba700623ad5347470837ee3a9321cb6b5f9f357c487694b1e606b1bc3c080`

导入：`p0_vorbis_tail`, `pathlib`, `struct`, `sys`, `tempfile`, `unittest`, `unittest.mock`

| 定义 | 位置／签名 |
| --- | --- |
| class VorbisTailTests | [源码:12](../../../tools/tests/test_p0_vorbis_tail.py#L12) · `unittest.TestCase` |
| function VorbisTailTests.test_original_fixture_crc_packet_identity_and_end_granule | [源码:13](../../../tools/tests/test_p0_vorbis_tail.py#L13) · `def test_original_fixture_crc_packet_identity_and_end_granule(self)` |
| function VorbisTailTests.test_corrupt_crc_truncated_page_and_partial_stream_are_rejected | [源码:21](../../../tools/tests/test_p0_vorbis_tail.py#L21) · `def test_corrupt_crc_truncated_page_and_partial_stream_are_rejected(self)` |
| function VorbisTailTests.test_recomputed_crc_does_not_hide_serial_sequence_or_continuation_errors | [源码:27](../../../tools/tests/test_p0_vorbis_tail.py#L27) · `def test_recomputed_crc_does_not_hide_serial_sequence_or_continuation_errors(self)` |
| function VorbisTailTests.test_output_escape_and_existing_output_are_rejected_before_oracle_load | [源码:35](../../../tools/tests/test_p0_vorbis_tail.py#L35) · `def test_output_escape_and_existing_output_are_rejected_before_oracle_load(self)` |

## tools/tests/test_p0_vulkan_memory.py

36 行；SHA256 `fc6f704b72fddf47b9daf2f1fdbb10c89ecf2638d2dae72e686a0177b1f42a6f`

导入：`p0_vulkan_memory`, `pathlib`, `sys`, `unittest`

| 定义 | 位置／签名 |
| --- | --- |
| class AllocationDiagnosticTests | [源码:9](../../../tools/tests/test_p0_vulkan_memory.py#L9) · `unittest.TestCase` |
| function AllocationDiagnosticTests.test_memory_class_requires_allowed_bit_and_exact_properties | [源码:10](../../../tools/tests/test_p0_vulkan_memory.py#L10) · `def test_memory_class_requires_allowed_bit_and_exact_properties(self)` |
| function AllocationDiagnosticTests.test_page_classes_exclude_reserved_and_preserve_unknown_scan | [源码:21](../../../tools/tests/test_p0_vulkan_memory.py#L21) · `def test_page_classes_exclude_reserved_and_preserve_unknown_scan(self)` |

## tools/tests/test_p0_workload.py

66 行；SHA256 `4631a75382d892272f546f2a7983384b7ce20c25022ab280c9dafe444808151a`

导入：`hashlib`, `json`, `p0_evidence`, `p0_process_metrics`, `p0_workload`, `pathlib`, `struct`, `sys`, `tempfile`, `unittest`, `zlib`

| 定义 | 位置／签名 |
| --- | --- |
| class WorkloadTests | [源码:16](../../../tools/tests/test_p0_workload.py#L16) · `unittest.TestCase` |
| function WorkloadTests.test_original_png_rgba_dimensions_and_determinism | [源码:17](../../../tools/tests/test_p0_workload.py#L17) · `def test_original_png_rgba_dimensions_and_determinism(self)` |
| function WorkloadTests.test_fixture_references_inventory_and_preservation | [源码:34](../../../tools/tests/test_p0_workload.py#L34) · `def test_fixture_references_inventory_and_preservation(self)` |
| function WorkloadTests.test_cpu_summary_never_spans_phase_boundaries | [源码:53](../../../tools/tests/test_p0_workload.py#L53) · `def test_cpu_summary_never_spans_phase_boundaries(self)` |

