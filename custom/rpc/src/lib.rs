use jsonrpc_core::Result;
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_api::HeaderT;
use sc_client_api::AuxStore;
use sp_runtime::traits::Zero;
use sp_runtime::traits::One;
use sp_blockchain::{HeaderBackend, HeaderMetadata, Error as BlockChainError};
use std::sync::Arc;
use sp_api::Decode;
use hex;
use prost::Message;

mod api {
    include!(concat!(env!("OUT_DIR"), "/api.v1.rs"));
}

#[rpc]
pub trait SpamCheckApi {
	#[rpc(name = "spamCheck")]	
	fn is_valid(&self, input: String) -> Result<bool>;
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

impl<C, System: frame_system::Config + Sync + Send, Block> SpamCheckApi for SpamCheck<C, System, Block>
where
	Block: sp_runtime::traits::Block<Header = System::Header, Hash = System::Hash>,
	C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + AuxStore +
		HeaderMetadata<Block, Error=BlockChainError> + Send + Sync + 'static,
{
	fn is_valid(&self, input: String) -> Result<bool> {

		let raw: Vec<u8> = match hex::decode(&input[..]) {
			Ok(bytes) => bytes,
			Err(e) => {
				println!("Error while decoding hex: {:?}", e);
				return Ok(false);
			}
		};

		let mut len: usize = 0;
		let mut idx = 0;
		while raw[idx] >= 128 {
			len |= (raw[idx] as usize & 127) << (idx * 7);
			idx += 1;
		}
		len |= (raw[idx] as usize & 127) << (idx * 7);
		idx += 1;
		//println!("block decoding {} {:?}", input, &raw[..]);
		if raw.len() < idx+len {
			println!("incomplete msg: expectedLength: {}, actualLenght: {}", idx+len, raw.len() );
			return Ok(false);
		}
		let block_response = match api::BlockResponse::decode(&raw[idx..idx+len]) {
			Ok(resp) => resp,
			Err(e) => {
				println!("Error while decoding block response: {:?}", e);
				return Ok(false);
			}
		};

		if block_response.blocks.len() == 0 || block_response.blocks[0].header.is_empty() {
			return Ok(false);
		}

		let header = match Block::Header::decode(&mut block_response.blocks[0].header.as_ref()) {
			Ok(h) => h,
			Err(e) => {
				println!("Error while decoding block header: {:?}", e);
				return Ok(false);
			}
		};
		//println!("header: {:?}", header);
		let n = header.number().clone();
		Ok(n > System::BlockNumber::zero()
			&& self.client.hash(n - System::BlockNumber::one()).unwrap_or_else(|_| Some(header.hash())).unwrap_or_else(|| header.hash()) ==  *header.parent_hash())
	}
}