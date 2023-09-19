// crates.io
use parity_scale_codec::Decode;
use reqwest::{
	header::{HeaderMap, CONTENT_TYPE},
	Client, ClientBuilder,
};
use serde::Deserialize;
use subrpcer::state;
// commission-board
use crate::*;

pub struct Api {
	client: Client,
}
impl Api {
	pub fn new() -> Self {
		Self {
			client: ClientBuilder::new()
				.default_headers(HeaderMap::from_iter([(
					CONTENT_TYPE,
					"application/json".parse().unwrap(),
				)]))
				.build()
				.unwrap(),
		}
	}

	pub async fn collators(&self) -> Result<Vec<String>> {
		let json = state::get_storage(
			0,
			"0xcec5070d609dd3497f72bde07fc96ba088dcde934c658227ee1dfafcd6e16903",
			None::<()>,
		);
		let result = self
			.client
			.post("https://darwinia-rpc.darwiniacommunitydao.xyz")
			.json(&json)
			.send()
			.await?
			.json::<JsonrpcResult>()
			.await?;
		let collators =
			<Vec<[u8; 20]>>::decode(&mut &*array_bytes::hex2bytes_unchecked(result.result))
				.unwrap()
				.into_iter()
				.map(|v| array_bytes::bytes2hex("0x", v))
				.collect();

		Ok(collators)
	}

	pub async fn commission_history_of(&self, who: &str) -> Result<CommissionHistory> {
		let body = format!(
			"{{\
				\"query\":\"{{\
						events(\
						where:{{\
							name_eq:\\\"DarwiniaStaking.CommissionUpdated\\\",\
							args_jsonContains:\\\"{{\\\\\\\"who\\\\\\\":\\\\\\\"{}\\\\\\\"}}\\\"\
						}},\
						limit:5,\
						orderBy:block_height_DESC\
					){{\
						args \
						block{{height}}\
					}}\
				}}\"\
			}}",
			who
		);
		let response = self
			.client
			.post("https://darwinia.explorer.subsquid.io/graphql")
			.body(body)
			.send()
			.await?
			.json::<Data>()
			.await?;

		Ok(CommissionHistory {
			who: who.into(),
			commissions: response
				.data
				.events
				.into_iter()
				.map(|d| (d.block.height, d.args.commission))
				.collect(),
		})
	}
}

#[derive(Debug, Deserialize)]
struct JsonrpcResult {
	result: String,
}

#[derive(Debug, Deserialize)]
struct Data {
	data: DataInner,
}
#[derive(Debug, Deserialize)]
struct DataInner {
	events: Vec<Event>,
}
#[derive(Debug, Deserialize)]
struct Event {
	args: Commission,
	block: Block,
}
#[derive(Debug, Deserialize)]
struct Commission {
	commission: u32,
}
#[derive(Debug, Deserialize)]
struct Block {
	height: u32,
}

#[derive(Debug)]
pub struct CommissionHistory {
	pub who: String,
	pub commissions: Vec<(u32, u32)>,
}
impl CommissionHistory {
	pub fn commissions(&self) -> String {
		if self.commissions.is_empty() {
			return "Never changed, amazing!".into();
		}

		self.commissions
			.iter()
			.map(|(h, c)| format!("({h},{}%)", *c as f64 / 10000000.))
			.collect::<Vec<_>>()
			.join(", ")
	}

	pub fn reputation(self) -> &'static str {
		for w in self.commissions.windows(3) {
			let (h_0, _) = &w[0];
			let Some((h_2, _)) = w.get(2) else { return "Good" };

			if h_0 - h_2 <= 7200 {
				return "Bad";
			}
		}

		"Good"
	}
}
