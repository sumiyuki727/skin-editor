//! Headless release diagnostic: only public application API, no image bytes.
#![forbid(unsafe_code)]
use skin_editor_application::{values::*, *};
use std::io::{self, Write};
use std::sync::Arc;
use std::time::Instant;

struct MetadataLease(BlobRef);
impl BlobLease for MetadataLease {
    fn reference(&self) -> BlobRef {
        self.0
    }
}
struct MetadataStore;
impl ResourceStore for MetadataStore {
    fn pin(&self, blob: BlobRef) -> Result<Arc<dyn BlobLease>, ResourceError> {
        Ok(Arc::new(MetadataLease(blob)))
    }
}
struct Driver {
    session: ProjectSession,
    next: u128,
}
impl Driver {
    fn submit(&mut self, commands: Vec<Command>) -> CommandResult {
        let info = self.session.project_info();
        let id = self.next;
        self.next += 1;
        self.session
            .submit(CommandRequest {
                api_version: API_VERSION,
                session_id: info.session_id,
                request_id: RequestId::new(id).unwrap(),
                expected: info.state,
                commands,
            })
            .unwrap()
    }
    fn checkout(&mut self, node: NodeId) {
        let info = self.session.project_info();
        let id = self.next;
        self.next += 1;
        self.session
            .navigate(HistoryRequest {
                api_version: API_VERSION,
                session_id: info.session_id,
                request_id: RequestId::new(id).unwrap(),
                expected: info.state,
                action: HistoryAction::Checkout { node },
            })
            .unwrap();
    }
    fn draft(&mut self, action: DraftAction) {
        let id = self.next;
        self.next += 1;
        self.session
            .edit_draft(DraftRequest {
                api_version: API_VERSION,
                session_id: self.session.project_info().session_id,
                request_id: RequestId::new(id).unwrap(),
                action,
            })
            .unwrap();
    }
    fn draft_state(&self) -> DraftState {
        self.session
            .draft_info(self.session.project_info().session_id)
            .unwrap()
            .unwrap()
            .state
    }
}
fn record(text: &str) -> ConfigRecord {
    ConfigRecord::new(text.as_bytes().to_vec(), ConfigInterpretation::Opaque).unwrap()
}
fn edits(step: usize, docs: usize) -> Vec<Command> {
    if docs == 0 {
        return vec![Command::SetConfigRecord {
            index: 0,
            record: record(&format!("value-{step}")),
        }];
    }
    let document = step % docs + 1;
    vec![Command::SetLayerTransform {
        document: DocumentId::new(document as u128).unwrap(),
        layer: LayerId::new((document * 100 + 3) as u128).unwrap(),
        transform: Affine2::new([1.0, 0.0, 0.0, 1.0, step as f64, 0.0]).unwrap(),
    }]
}
fn initialize(d: &mut Driver, assets: usize, docs: usize) {
    if assets == 0 {
        d.submit(vec![Command::InsertConfigRecord {
            index: 0,
            record: record("initial"),
        }]);
        return;
    }
    let commands = (1..=assets)
        .map(|i| {
            let mut digest = [0u8; 32];
            digest[..8].copy_from_slice(&(i as u64).to_le_bytes());
            Command::RegisterAsset {
                asset: Asset::new(
                    AssetId::new(i as u128).unwrap(),
                    format!("metadata {i}"),
                    BlobRef::new(digest, 1024 * 1024 * 4),
                    AssetKind::RasterImage(PixelSize::new(1024, 1024).unwrap()),
                )
                .unwrap(),
            }
        })
        .collect();
    d.submit(commands);
    let mut commands = vec![Command::AddComponent {
        node: ComponentNode {
            id: ComponentId::new(1).unwrap(),
            local: Affine2::IDENTITY,
            kind: ComponentKind::Group { children: vec![] },
        },
        parent: None,
        index: 0,
    }];
    for doc in 1..=docs {
        let id = |i| LayerId::new((doc * 100 + i) as u128).unwrap();
        let mut nodes = vec![
            LayerNode {
                id: id(1),
                local: Affine2::IDENTITY,
                opacity: Opacity::OPAQUE,
                visible: true,
                kind: LayerKind::Group {
                    children: vec![id(2)],
                },
            },
            LayerNode {
                id: id(2),
                local: Affine2::IDENTITY,
                opacity: Opacity::OPAQUE,
                visible: true,
                kind: LayerKind::Group {
                    children: (3..=50).map(id).collect(),
                },
            },
        ];
        nodes.extend((3..=50).map(|i| LayerNode {
            id: id(i),
            local: Affine2::IDENTITY,
            opacity: Opacity::OPAQUE,
            visible: true,
            kind: LayerKind::Image {
                asset:
                    AssetId::new((((doc - 1) * 10 + (i - 3) % 10) % assets + 1) as u128).unwrap(),
            },
        }));
        let document = DocumentId::new(doc as u128).unwrap();
        commands.push(Command::CreateDocument {
            document: ElementDocument::new(
                document,
                PixelSize::new(1024, 1024).unwrap(),
                LayerTree::new(vec![id(1)], nodes).unwrap(),
            ),
        });
        commands.push(Command::AddComponent {
            node: ComponentNode {
                id: ComponentId::new(doc as u128 + 1).unwrap(),
                local: Affine2::IDENTITY,
                kind: ComponentKind::Element {
                    document,
                    document_to_component: Affine2::IDENTITY,
                },
            },
            parent: Some(ComponentId::new(1).unwrap()),
            index: doc - 1,
        });
        commands.push(Command::SetOutputMapping {
            mapping: OutputMapping::new(
                OutputId::new(doc as u128).unwrap(),
                document,
                "diagnostic".into(),
                OutputPath::new(format!("element-{doc}.png")).unwrap(),
            )
            .unwrap(),
        });
    }
    d.submit(commands);
}
fn checkpoint(
    phase: &str,
    elapsed: u128,
    latencies: &mut [u128],
    stats: Option<HistoryStatistics>,
) {
    latencies.sort_unstable();
    let quantile = |percent: usize| {
        if latencies.is_empty() {
            0
        } else {
            latencies[(latencies.len() * percent).div_ceil(100).saturating_sub(1)]
        }
    };
    let (nodes, blobs, receipts) = stats.map_or((0, 0, 0), |s| {
        (s.nodes, s.retained_blobs, s.retained_requests)
    });
    println!(
        "{{\"phase\":\"{phase}\",\"wall_ns\":{elapsed},\"operations\":{},\"p50_ns\":{},\"p95_ns\":{},\"p99_ns\":{},\"max_ns\":{},\"nodes\":{nodes},\"retained_blobs\":{blobs},\"retained_requests\":{receipts}}}",
        latencies.len(),
        quantile(50),
        quantile(95),
        quantile(99),
        latencies.last().copied().unwrap_or(0)
    );
    io::stdout().flush().unwrap();
    let mut ack = String::new();
    io::stdin().read_line(&mut ack).unwrap();
    assert_eq!(ack.trim(), "continue");
}
fn stats(d: &Driver) -> Option<HistoryStatistics> {
    Some(
        d.session
            .history_statistics(d.session.project_info().session_id)
            .unwrap(),
    )
}
fn main() {
    let profile = std::env::args().nth(1).expect("profile");
    let (assets, docs) = match profile.as_str() {
        "minimal" => (0, 0),
        "standard" => (100, 10),
        "pressure" => (1000, 100),
        _ => panic!("unknown profile"),
    };
    checkpoint("ready", 0, &mut [], None);
    let mut d = Driver {
        session: ProjectSession::with_resources(
            7,
            70,
            Box::new(MetadataStore),
            SessionLimits::default(),
        )
        .unwrap(),
        next: 1,
    };
    let start = Instant::now();
    initialize(&mut d, assets, docs);
    checkpoint(
        "initialized",
        start.elapsed().as_nanos(),
        &mut [],
        stats(&d),
    );
    let mut fork = None;
    let mut fork_project = None;
    let mut times = Vec::with_capacity(5000);
    let start = Instant::now();
    for step in 1..=5000 {
        let op = Instant::now();
        let result = d.submit(edits(step, docs));
        assert_eq!(result.status, CommandStatus::Committed);
        times.push(op.elapsed().as_nanos());
        if step == 2500 {
            fork = Some(result.state.head);
            fork_project = Some(d.session.snapshot().project);
        }
    }
    let a = d.session.snapshot();
    checkpoint(
        "transactions_a",
        start.elapsed().as_nanos(),
        &mut times,
        stats(&d),
    );
    let start = Instant::now();
    d.checkout(fork.unwrap());
    assert_eq!(d.session.snapshot().project, fork_project.take().unwrap());
    checkpoint(
        "fork_checkout",
        start.elapsed().as_nanos(),
        &mut [],
        stats(&d),
    );
    times.clear();
    let start = Instant::now();
    for step in 5001..=10000 {
        let op = Instant::now();
        let result = d.submit(edits(step, docs));
        assert_eq!(result.status, CommandStatus::Committed);
        times.push(op.elapsed().as_nanos());
    }
    let b = d.session.snapshot();
    checkpoint(
        "transactions_b",
        start.elapsed().as_nanos(),
        &mut times,
        stats(&d),
    );
    let count = d.session.history_len();
    let mut restore = Vec::new();
    let start = Instant::now();
    for snapshot in [&a, &b] {
        let op = Instant::now();
        d.checkout(snapshot.state.head);
        assert_eq!(d.session.snapshot().project, snapshot.project);
        restore.push(op.elapsed().as_nanos());
    }
    assert_eq!(d.session.history_len(), count);
    checkpoint(
        "cross_branch",
        start.elapsed().as_nanos(),
        &mut restore,
        stats(&d),
    );
    let start = Instant::now();
    let before = d.session.snapshot();
    d.draft(DraftAction::Begin {
        expected: before.state,
    });
    let mut draft_times = Vec::new();
    for step in 10001..=10300 {
        let op = Instant::now();
        d.draft(DraftAction::Replace {
            expected: d.draft_state(),
            commands: edits(step, docs),
        });
        draft_times.push(op.elapsed().as_nanos());
    }
    d.draft(DraftAction::Cancel {
        expected: d.draft_state(),
    });
    assert_eq!(d.session.snapshot(), before);
    assert_eq!(d.session.history_len(), count);
    checkpoint(
        "draft_cancel",
        start.elapsed().as_nanos(),
        &mut draft_times,
        stats(&d),
    );
    drop((a, b, before, times, restore, draft_times, d));
    checkpoint("released", 0, &mut [], None);
}
