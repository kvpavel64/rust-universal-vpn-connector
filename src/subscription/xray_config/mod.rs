//! Strongly typed representation of Xray-core JSON configuration documents.
//!
//! This schema models the Xray-style configuration supplied by some subscription
//! providers. It is intentionally separate from the normalized sing-box [`Node`](crate::Node)
//! model: conversion between engine formats must be explicit.

mod inbound;
mod observatory;
mod outbound;
mod root;
mod routing;

pub use inbound::{RootInboundsItem, RootInboundsItemSettings, RootInboundsItemSniffing};
pub use observatory::{RootBurstObservatory, RootBurstObservatoryPingConfig};
pub use outbound::*;
pub use root::{FileReadResult, JsonResult, Nodes, Root, RootDns, RootLog};
pub use routing::*;
