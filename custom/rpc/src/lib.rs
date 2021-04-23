use jsonrpc_core::Result;
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_api::HeaderT;
use sc_client_api::AuxStore;
use sp_runtime::traits::Zero;
use sp_runtime::traits::One;
use sp_blockchain::{HeaderBackend, HeaderMetadata, Error as BlockChainError};
use sp_runtime::{traits::Block as BlockT};
use std::sync::Arc;

#[rpc]
pub trait SpamCheckApi<BlockHeader> {
	#[rpc(name = "spamCheck")]	
	fn is_valid(&self, header: BlockHeader) -> Result<bool>;
}

pub struct SpamCheck<C, S, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
	_unused: std::marker::PhantomData<S>,
}

impl<C, S, M> SpamCheck<C, S, M> {
    pub fn new(client: Arc<C>) -> Self {
        Self { client, _marker: Default::default(), _unused: Default::default() }
    }
}

impl<C, System: frame_system::Config + Sync + Send, Block> SpamCheckApi<<Block as BlockT>::Header> for SpamCheck<C, System, Block>
where
	Block: sp_runtime::traits::Block<Header = System::Header, Hash = System::Hash>,
	C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + AuxStore +
		HeaderMetadata<Block, Error=BlockChainError> + Send + Sync + 'static,
{
	fn is_valid(&self, header: <Block as BlockT>::Header) -> Result<bool> {
		let n = header.number().clone();
		Ok(n > System::BlockNumber::zero()
			&& self.client.hash(n - System::BlockNumber::one()).unwrap_or_else(|_| Some(header.hash())).unwrap_or_else(|| header.hash()) ==  *header.parent_hash())
	}
}