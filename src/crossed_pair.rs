use crate::bindings::flash_bots_uniswap_query::FlashBotsUniswapQuery;
use ethers::prelude::*;

#[derive(Debug)]
pub struct CrossedPairManager<'a, M>
where
    M: Middleware,
{
    flash_query_contract: &'a FlashBotsUniswapQuery<M>,
    markets: Vec<TokenMarket<'a>>,
}

impl<'a, M> CrossedPairManager<'a, M>
where
    M: Middleware,
{
    pub fn new(
        grouped_pairs: &'a Vec<(H160, Vec<H160>)>,
        flash_query_contract: &'a FlashBotsUniswapQuery<M>,
    ) -> Self {
        let pairs = grouped_pairs
            .into_iter()
            .map(|(token, pairs)| TokenMarket {
                token,
                pairs: pairs
                    .to_vec()
                    .into_iter()
                    .map(|address| Pair {
                        address,
                        reserve: None,
                    })
                    .collect::<Vec<Pair>>(),
            })
            .collect::<Vec<TokenMarket>>();
        Self {
            markets: pairs,
            flash_query_contract,
        }
    }

    pub async fn update_reserve(&mut self) {
        let reserves = self
            .get_all_pair_addresses()
            .iter()
            .map(|pair| pair.address)
            .collect::<Vec<H160>>();

        let reserves = self
            .flash_query_contract
            .get_reserves_by_pairs(reserves)
            .call()
            .await
            .unwrap();

        for (new_reserve, pair) in std::iter::zip(&reserves, self.get_all_pair_addresses()) {
            let updated_reserve = Reserve {
                reserve0: new_reserve[0],
                reserve1: new_reserve[1],
                block_timestamp_last: new_reserve[2],
            };

            pair.reserve = Some(updated_reserve);
        }
    }

    fn get_all_pair_addresses(&mut self) -> Vec<&mut Pair> {
        self.markets
            .iter_mut()
            .flat_map(|token_market| &mut token_market.pairs)
            .collect::<Vec<&mut Pair>>()
    }

    pub fn find_arbitrage_opportunities(&self) {
        for market in &self.markets {
            market.find_arbitrage_opportunity();
        }
        ()
    }
}

#[derive(Debug)]
pub struct TokenMarket<'a> {
    token: &'a H160,
    pairs: Vec<Pair>,
}

impl<'a> TokenMarket<'a> {
    pub fn find_arbitrage_opportunity(&self) {
        dbg!(self.token);
        println!("------------------------------------------------------------------------------------------");
        for pair in &self.pairs {
            let reserve = pair.reserve.as_ref().unwrap();

            let ether: U256 = U256::from_dec_str("1000000000000000000").unwrap();

            let buy_price = reserve.token_out_for_ether(ether);

            let sell_price = reserve.token_in_for_ether(ether);
            dbg!(buy_price);
            dbg!(sell_price);
            let timestamp = pair.reserve.as_ref().unwrap().block_timestamp_last;
            dbg!(timestamp);
            println!("------------------------------------------------------------------------------------------")
        }
        println!("------------------------------------------------------------------------------------------");
        println!("");
    }
}

#[derive(Debug)]
pub struct Pair {
    address: H160,
    reserve: Option<Reserve>,
}

#[derive(Debug)]
pub struct Reserve {
    reserve0: U256,
    reserve1: U256,
    block_timestamp_last: U256,
}

impl Reserve {
    pub fn token_out_for_ether(&self, amount: U256) -> U256 {
        let amount = amount / 1000 * 997;
        let numerator = self.reserve0 * amount;
        let denominator = self.reserve1 + amount;
        numerator / denominator
    }

    pub fn token_in_for_ether(&self, amount: U256) -> U256 {
        let numerator = self.reserve0 * amount;
        if self.reserve1 < amount {
            return self.reserve0;
        }
        let denominator = (self.reserve1 - amount) * 997 / 1000;
        numerator / denominator
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn is_flatten() {}
}
