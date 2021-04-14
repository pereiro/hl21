use crate::context::{Metrics, SyncContext};
use crate::http::http_post;
use crate::model::{Dig, TreasureList};
use rand::distributions::{Distribution, Uniform};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::time::Duration;
use url::Url;

pub struct Digger {
    url: Url,
    timeout: Duration,
    client: surf::Client,
    sync: SyncContext,
    min_depth: u64,
    max_depth: u64,
    min_depth_probability: u64,
}

impl Digger {
    pub fn new(
        url: Url,
        timeout: Duration,
        min_depth: u64,
        max_depth: u64,
        min_depth_probability: u64,
        sync: SyncContext,
    ) -> Digger {
        Digger {
            url,
            timeout,
            client: surf::Client::new(),
            min_depth,
            max_depth,
            sync,
            min_depth_probability,
        }
    }

    pub async fn start(self) {
        let mut rng = StdRng::from_entropy();
        let between = Uniform::from(0..100);
        loop {
            let tile = self.sync.tile_receiver.recv().await.unwrap();
            let mut dig = Dig::from_tile(tile, 0);

            while dig.amount > 0 && dig.depth <= self.max_depth {
                let mut license = self.sync.license_receiver.recv().await.unwrap();
                self.sync.digger_rate_limiter().await;
                dig.license_id = license.id;

                let treasures: TreasureList = match http_post(
                    &self.url,
                    self.timeout,
                    dig,
                    &self.client,
                    self.sync.clone(),
                )
                .await
                {
                    Ok(t) => {

                        self.sync
                            .metrics_sender
                            .send(Metrics::new_dig(true, dig.depth))
                            .await
                            .unwrap();

                        t
                    }
                    Err(e) => {

                        self.sync
                            .metrics_sender
                            .send(Metrics::new_dig(false, dig.depth))
                            .await
                            .unwrap();


                        if e.status == 404  || e.status == 422{
                            TreasureList::new()
                        } else {
                            // if e.status!=667 {
                            //     println!("digger error: {}", e.to_string());
                            // }
                            self.sync.license_sender.send(license).await.unwrap();
                            continue;
                        }
                    }
                };
                dig.depth += 1;
                license.dig_used += 1;
                if license.dig_used == license.dig_allowed {
                    self.sync.empty_license_sender.send(license).await.unwrap()
                } else {
                    self.sync.license_sender.send(license).await.unwrap()
                }
                for treasure in treasures.0.into_iter() {
                    dig.amount -= 1;
                    if dig.depth - 1 >= self.min_depth
                        && (dig.depth - 1 > self.min_depth
                            || (self.min_depth_probability >= 100
                                || between.sample(&mut rng) <= self.min_depth_probability))
                    {
                        self.sync.treasure_sender.send(treasure).await.unwrap();
                    }
                }
            }
        }
    }
}
