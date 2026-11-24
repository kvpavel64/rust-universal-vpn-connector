//! Strongly typed representation of Xray-core JSON configuration documents.
//!
//! This schema models the Xray-style configuration supplied by some subscription
//! providers. It is intentionally separate from the normalized sing-box [`Node`](crate::Node)
//! model: conversion between engine formats must be explicit.

mod root;
mod inbound;
mod outbound;
mod routing;
mod observatory;

pub use root::{FileReadResult, JsonResult, Nodes, Root, RootDns, RootLog};
pub use inbound::{RootInboundsItem, RootInboundsItemSettings, RootInboundsItemSniffing};
pub use outbound::*;
pub use routing::*;
pub use observatory::{RootBurstObservatory, RootBurstObservatoryPingConfig};
