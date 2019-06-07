use std::collections::HashMap;
use actix::prelude::*;

use crate::chain::{self, Chain};
use crate::node::{NodeDetails, connector::Initialize};
use crate::feed::connector::FeedConnector;
use crate::util::DenseMap;

pub struct Aggregator {
    chains: HashMap<Box<str>, Addr<Chain>>,
    feeds: DenseMap<FeedConnector>,
}

impl Aggregator {
    pub fn new() -> Self {
        Aggregator {
            chains: HashMap::new(),
            feeds: DenseMap::new(),
        }
    }

    /// Get an address to the chain actor by name. If the address is not found,
    /// or the address is disconnected (actor dropped), create a new one.
    pub fn lazy_chain(&mut self, chain: Box<str>, ctx: &mut <Self as Actor>::Context) -> &Addr<Chain> {
        let connected = self.chains.get(&chain).map(|addr| addr.connected()).unwrap_or(false);

        if !connected {
            let addr = Chain::new(ctx.address().recipient(), chain.clone()).start();

            self.chains.insert(chain.clone(), addr);
        }

        &self.chains[&chain]
    }
}

impl Actor for Aggregator {
    type Context = Context<Self>;
}

/// Message sent from the NodeConnector to the Aggregator upon getting all node details
#[derive(Message)]
pub struct AddNode {
    pub node: NodeDetails,
    pub chain: Box<str>,
    pub rec: Recipient<Initialize>,
}

/// Message sent from the Chain to the Aggregator when the Chain loses all nodes
#[derive(Message)]
pub struct DropChain(pub Box<str>);

/// Message sent from the FeedConnector to the Aggregator when subscribing to a new chain
#[derive(Message)]
pub struct Subscribe {
    pub chain: Box<str>,
    pub feed: Addr<FeedConnector>,
}

impl Handler<AddNode> for Aggregator {
    type Result = ();

    fn handle(&mut self, msg: AddNode, ctx: &mut Self::Context) {
        let AddNode { node, chain, rec } = msg;

        self.lazy_chain(chain, ctx).do_send(chain::AddNode {
            node,
            rec,
        });
    }
}

impl Handler<DropChain> for Aggregator {
    type Result = ();

    fn handle(&mut self, msg: DropChain, _: &mut Self::Context) {
        let DropChain(chain) = msg;

        self.chains.remove(&chain);

        info!("Dropped chain [{}] from the aggregator", chain);
    }
}