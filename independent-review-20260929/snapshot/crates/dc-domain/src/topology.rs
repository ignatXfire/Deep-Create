//! C-01 part of the single global topology revision contract.
//! C-03 must compare and update this head in the SAME transaction as the graph
//! and event. This module neither commits nor supplies an authority receipt.

use crate::{
    ids::{EntityId, RowVersion},
    work::{ValidatedGraph, WorkError},
};

/// Native store snapshot, never an IPC proposal or proof of a successful commit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TopologyHead {
    pub installation_id: EntityId,
    pub revision: RowVersion,
}

/// Binds the successor to the exact immutable validated candidate, not merely
/// an integer supplied alongside a different graph.
#[derive(Debug)]
pub struct PreparedTopologyChange<'a> {
    expected: TopologyHead,
    successor: TopologyHead,
    graph: ValidatedGraph<'a>,
}

impl<'a> PreparedTopologyChange<'a> {
    pub fn expected(&self) -> TopologyHead {
        self.expected
    }

    pub fn successor(&self) -> TopologyHead {
        self.successor
    }

    pub fn graph(&self) -> &ValidatedGraph<'a> {
        &self.graph
    }
}

/// Caller supplies the head read with the complete affected graph. The sole
/// writer must still check it at commit; preparation is not a transferable lock.
pub fn prepare_topology_change(
    expected: TopologyHead,
    graph: ValidatedGraph<'_>,
) -> Result<PreparedTopologyChange<'_>, WorkError> {
    if expected.revision != graph.candidate().expected_graph_version
        || expected.revision != graph.candidate().current_graph_version
    {
        return Err(WorkError::GraphVersionConflict);
    }
    let revision = expected
        .revision
        .checked_add(1)
        .ok_or(WorkError::VersionOverflow)?;
    Ok(PreparedTopologyChange {
        expected,
        successor: TopologyHead {
            revision,
            ..expected
        },
        graph,
    })
}

/// Use inside the writer transaction immediately before the conditional update.
/// A successful return alone is NOT a committed graph or checkpoint receipt.
pub fn check_topology_precondition(
    plan: &PreparedTopologyChange<'_>,
    current: TopologyHead,
) -> Result<(), WorkError> {
    if plan.expected != current {
        return Err(WorkError::GraphVersionConflict);
    }
    Ok(())
}
