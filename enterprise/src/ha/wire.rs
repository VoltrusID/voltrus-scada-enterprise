//! Messages of the node link and their framing.
//!
//! One frame is a 4 byte big-endian length followed by a MessagePack body.
//! Frames larger than [`MAX_FRAME`] are refused before they are read.

use std::io;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use super::store::{Batch, Cell, SnapshotTable};

/// Bumped when a message changes shape.
pub const PROTOCOL_VERSION: u32 = 1;

/// Largest accepted frame. A full-sync chunk is about 1 MiB; a batch of the
/// biggest change sets a few MiB.
pub const MAX_FRAME: usize = 64 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WireRole {
    Primary,
    Standby,
    /// Wants to be primary but may not act: it lost its lease or its peer and witness.
    Fenced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hello {
    pub proto: u32,
    pub node_id: String,
    pub role: WireRole,
    pub epoch: u64,
    /// Standby: position in the primary's log of `epoch`.
    pub applied_seq: u64,
    /// Standby: its data is not a clean copy, a full sync is required.
    pub needs_full_sync: bool,
    /// Primary: highest sequence number it has assigned.
    pub max_seq: u64,
    pub version: String,
    pub now_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Msg {
    // Authentication (see `link`): both bound to the TLS session.
    AuthInit {
        node_id: String,
        nonce: Vec<u8>,
        mac: Vec<u8>,
    },
    AuthReply {
        node_id: String,
        nonce: Vec<u8>,
        mac: Vec<u8>,
    },

    Hello(Hello),

    // Primary to standby.
    PlanIncremental {
        from_seq: u64,
    },
    PlanFull {
        snapshot_seq: u64,
        tables: Vec<SnapshotTable>,
    },
    Chunk {
        table: String,
        cols: Vec<String>,
        key: Vec<String>,
        rows: Vec<Vec<Cell>>,
    },
    FullEnd {
        snapshot_seq: u64,
    },
    Batch(Batch),
    Heartbeat {
        epoch: u64,
        max_seq: u64,
        lag_ms: u64,
        /// Device pollers running on the primary, so a takeover starts the same.
        pollers: Vec<String>,
        now_ms: i64,
    },
    /// The primary has stopped acting and the standby has every change through
    /// `final_seq`: take over.
    Handoff {
        final_seq: u64,
    },

    // Standby to primary.
    Ack {
        seq: u64,
    },
    HeartbeatAck {
        applied_seq: u64,
        now_ms: i64,
    },
    /// Run a full sync now.
    ResyncRequest,
    /// Hand the active role to me.
    SwitchoverRequest,
    HandoffRefused {
        reason: String,
    },
    Promoted {
        epoch: u64,
    },

    // Either direction.
    Reject {
        reason: String,
    },

    // Witness (arbiter) exchange.
    LeaseAcquire {
        node_id: String,
        /// The epoch the caller currently holds or has seen.
        epoch: u64,
        ttl_ms: u64,
    },
    LeaseRelease {
        node_id: String,
        epoch: u64,
    },
    LeaseReply {
        granted: bool,
        epoch: u64,
        holder: String,
        remaining_ms: u64,
    },
}

pub async fn write_msg<W: AsyncWrite + Unpin>(w: &mut W, msg: &Msg) -> io::Result<()> {
    let body = rmp_serde::to_vec_named(msg).map_err(|e| io::Error::other(e.to_string()))?;
    if body.len() > MAX_FRAME {
        return Err(io::Error::other("frame too large"));
    }
    let mut frame = Vec::with_capacity(4 + body.len());
    frame.extend_from_slice(&(body.len() as u32).to_be_bytes());
    frame.extend_from_slice(&body);
    w.write_all(&frame).await?;
    w.flush().await
}

pub async fn read_msg<R: AsyncRead + Unpin>(r: &mut R) -> io::Result<Msg> {
    let len = r.read_u32().await? as usize;
    if len > MAX_FRAME {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "frame too large",
        ));
    }
    let mut body = vec![0u8; len];
    r.read_exact(&mut body).await?;
    rmp_serde::from_slice(&body)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn messages_round_trip_through_the_framing() {
        let (mut a, mut b) = tokio::io::duplex(1 << 16);
        let batch = Batch {
            seq_from: 4,
            seq_to: 9,
            tables: vec![crate::ha::store::TableChanges {
                name: "t".into(),
                cols: vec!["id".into(), "v".into(), "r".into(), "b".into(), "n".into()],
                key: vec!["id".into()],
                rows: vec![vec![
                    Cell::Int(-7),
                    Cell::Text("x".into()),
                    Cell::Real(1.0),
                    Cell::Blob(vec![0, 1, 255]),
                    Cell::Null,
                ]],
                deletes: vec![vec![Cell::Text("k".into())]],
                delete_runs: vec![(1, 5)],
            }],
        };
        write_msg(&mut a, &Msg::Batch(batch.clone())).await.unwrap();
        write_msg(&mut a, &Msg::Ack { seq: 9 }).await.unwrap();
        match read_msg(&mut b).await.unwrap() {
            Msg::Batch(got) => assert_eq!(got, batch, "1.0 stays a REAL, 7 stays an INTEGER"),
            other => panic!("unexpected {other:?}"),
        }
        assert!(matches!(
            read_msg(&mut b).await.unwrap(),
            Msg::Ack { seq: 9 }
        ));
    }

    #[tokio::test]
    async fn an_oversized_frame_is_refused_before_it_is_read() {
        let (mut a, mut b) = tokio::io::duplex(64);
        a.write_all(&((MAX_FRAME as u32) + 1).to_be_bytes())
            .await
            .unwrap();
        let err = read_msg(&mut b).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }
}
