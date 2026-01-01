//! Substrate Node Template CLI library.
#![warn(missing_docs)]

mod benchmarking;
mod chain_spec;
mod cli;
mod command;
// TODO: Phase 2.3 - Full Frontier RPC integration
// mod eth;
mod rpc;
mod service;

fn main() -> sc_cli::Result<()> {
	command::run()
}
