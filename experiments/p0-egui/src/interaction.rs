//! Isolated P0 gesture owner. Not the product Command API or project format.
//! Adapters submit values; no mutable document or draft references escape.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Snapshot {
    pub id: u64,
    pub parent: Option<u64>,
    pub position: [f64; 2],
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Command {
    Begin { expected_node: u64 },
    Move { session: u64, position: [f64; 2] },
    Commit { session: u64 },
    Cancel { session: u64 },
    Checkout { node: u64 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Started(u64),
    Updated,
    Committed(u64),
    Unchanged,
    Cancelled,
    CheckedOut,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Busy,
    Conflict,
    InvalidPosition,
    StaleSession,
    UnknownNode,
}
#[derive(Clone, Copy)]
struct Draft {
    session: u64,
    position: [f64; 2],
}
pub struct Owner {
    nodes: Vec<Snapshot>,
    current: usize,
    next_session: u64,
    draft: Option<Draft>,
}
impl Default for Owner {
    fn default() -> Self {
        Self {
            nodes: vec![Snapshot {
                id: 0,
                parent: None,
                position: [40., 40.],
            }],
            current: 0,
            next_session: 1,
            draft: None,
        }
    }
}
impl Owner {
    pub fn committed(&self) -> Snapshot {
        self.nodes[self.current]
    }
    pub fn visible_position(&self) -> [f64; 2] {
        self.draft.map_or(self.committed().position, |d| d.position)
    }
    pub fn snapshots(&self) -> &[Snapshot] {
        &self.nodes
    }
    pub fn execute(&mut self, command: Command) -> Result<Outcome, Error> {
        match command {
            Command::Begin { expected_node } => {
                if self.draft.is_some() {
                    return Err(Error::Busy);
                }
                if self.committed().id != expected_node {
                    return Err(Error::Conflict);
                }
                let session = self.next_session;
                self.next_session += 1;
                self.draft = Some(Draft {
                    session,
                    position: self.committed().position,
                });
                Ok(Outcome::Started(session))
            }
            Command::Checkout { node } => {
                if self.draft.is_some() {
                    return Err(Error::Busy);
                }
                let current = self
                    .nodes
                    .iter()
                    .position(|s| s.id == node)
                    .ok_or(Error::UnknownNode)?;
                self.current = current;
                Ok(Outcome::CheckedOut)
            }
            Command::Move { session, .. }
            | Command::Commit { session }
            | Command::Cancel { session } => {
                let draft = self
                    .draft
                    .filter(|d| d.session == session)
                    .ok_or(Error::StaleSession)?;
                match command {
                    Command::Move { position, .. } => {
                        if position.iter().any(|n| !n.is_finite() || n.abs() > 10000.) {
                            return Err(Error::InvalidPosition);
                        }
                        self.draft = Some(Draft { position, ..draft });
                        Ok(Outcome::Updated)
                    }
                    Command::Cancel { .. } => {
                        self.draft = None;
                        Ok(Outcome::Cancelled)
                    }
                    Command::Commit { .. } => {
                        self.draft = None;
                        if draft.position == self.committed().position {
                            return Ok(Outcome::Unchanged);
                        }
                        let id = self.nodes.len() as u64;
                        self.nodes.push(Snapshot {
                            id,
                            parent: Some(self.committed().id),
                            position: draft.position,
                        });
                        self.current = self.nodes.len() - 1;
                        Ok(Outcome::Committed(id))
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn begin(o: &mut Owner) -> u64 {
        match o
            .execute(Command::Begin {
                expected_node: o.committed().id,
            })
            .unwrap()
        {
            Outcome::Started(id) => id,
            _ => panic!(),
        }
    }
    #[test]
    fn many_draft_updates_commit_once_without_changing_published_snapshots() {
        let mut o = Owner::default();
        let original = o.committed();
        let session = begin(&mut o);
        for i in 0..300 {
            o.execute(Command::Move {
                session,
                position: [i as f64, 60.],
            })
            .unwrap();
            assert_eq!(o.committed(), original);
            assert_eq!(o.snapshots().len(), 1);
        }
        assert_eq!(
            o.execute(Command::Commit { session }),
            Ok(Outcome::Committed(1))
        );
        assert_eq!(o.committed().position, [299., 60.]);
        assert_eq!(o.snapshots()[0], original);
        assert_eq!(
            o.execute(Command::Commit { session }),
            Err(Error::StaleSession)
        );
    }
    #[test]
    fn invalid_update_rolls_back_and_cancel_or_noop_leave_no_nodes() {
        let mut o = Owner::default();
        let session = begin(&mut o);
        o.execute(Command::Move {
            session,
            position: [70., 80.],
        })
        .unwrap();
        for position in [[f64::NAN, 2.], [1., f64::INFINITY], [10001., 0.]] {
            assert_eq!(
                o.execute(Command::Move { session, position }),
                Err(Error::InvalidPosition)
            );
            assert_eq!(o.visible_position(), [70., 80.]);
        }
        assert_eq!(
            o.execute(Command::Cancel { session }),
            Ok(Outcome::Cancelled)
        );
        assert_eq!(o.visible_position(), [40., 40.]);
        assert_eq!(o.snapshots().len(), 1);
        let session = begin(&mut o);
        assert_eq!(
            o.execute(Command::Commit { session }),
            Ok(Outcome::Unchanged)
        );
        assert_eq!(o.snapshots().len(), 1);
    }
    #[test]
    fn stale_and_busy_requests_cannot_modify_a_different_session() {
        let mut o = Owner::default();
        assert_eq!(
            o.execute(Command::Begin { expected_node: 9 }),
            Err(Error::Conflict)
        );
        let first = begin(&mut o);
        assert_eq!(
            o.execute(Command::Begin { expected_node: 0 }),
            Err(Error::Busy)
        );
        assert_eq!(o.execute(Command::Checkout { node: 0 }), Err(Error::Busy));
        o.execute(Command::Cancel { session: first }).unwrap();
        let second = begin(&mut o);
        for command in [
            Command::Cancel { session: first },
            Command::Commit { session: first },
            Command::Move {
                session: first,
                position: [90., 90.],
            },
        ] {
            assert_eq!(o.execute(command), Err(Error::StaleSession));
        }
        o.execute(Command::Commit { session: second }).unwrap();
        assert_eq!(o.snapshots().len(), 1);
    }
    #[test]
    fn checkout_and_branch_preserve_original_descendants() {
        let mut o = Owner::default();
        for position in [[80., 40.], [120., 40.]] {
            let session = begin(&mut o);
            o.execute(Command::Move { session, position }).unwrap();
            o.execute(Command::Commit { session }).unwrap();
        }
        let old = o.snapshots().to_vec();
        o.execute(Command::Checkout { node: 0 }).unwrap();
        let session = begin(&mut o);
        o.execute(Command::Move {
            session,
            position: [40., 100.],
        })
        .unwrap();
        o.execute(Command::Commit { session }).unwrap();
        assert_eq!(&o.snapshots()[..3], old.as_slice());
        assert_eq!(o.committed().parent, Some(0));
        o.execute(Command::Checkout { node: 2 }).unwrap();
        assert_eq!(o.visible_position(), [120., 40.]);
        assert_eq!(
            o.execute(Command::Checkout { node: 99 }),
            Err(Error::UnknownNode)
        );
        assert_eq!(o.committed(), old[2]);
    }
}
