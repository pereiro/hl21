use crate::context::{Metrics, SyncContext};
use crate::http::http_post;
use crate::model::{License, MoneyList};
use rand::distributions::{Distribution, Uniform};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::time::Duration;
use url::Url;

pub struct Attorney {
    url: Url,
    timeout: Duration,
    license_min_cost: u64,
    license_max_cost: u64,
    free_license_probability: u64,
    client: surf::Client,
    sync: SyncContext,
}

impl Attorney {
    pub fn new(
        url: Url,
        timeout: Duration,
        license_min_cost: u64,
        license_max_cost: u64,
        free_license_probability: u64,
        sync: SyncContext,
    ) -> Attorney {
        Attorney {
            url,
            timeout,
            license_min_cost,
            license_max_cost,
            free_license_probability,
            client: surf::Client::new(),
            sync,
        }
    }
    pub async fn start(self) {
        let mut rng = StdRng::from_entropy();
        let between = Uniform::from(0..100);
        loop {
            let mut license = self.sync.empty_license_receiver.recv().await.unwrap();
            let payload: MoneyList;
            if self.free_license_probability <= 0
                || (self.free_license_probability < 100
                    && between.sample(&mut rng) > self.free_license_probability)
            {
                payload = match self.sync.cash_receiver.try_recv() {
                    Ok(cash) => {
                        if cash.len() == 0 {
                            cash
                        } else {
                            let split = cash.get_optimal_list(
                                self.license_max_cost as usize,
                                self.license_min_cost as usize,
                            );
                            if split.exchange.len() > 0 {
                                self.sync.cash_sender.send(split.exchange).await.unwrap();
                            }
                            split.money
                        }
                    }
                    Err(_) => MoneyList::new(),
                };
            } else {
                payload = MoneyList::new();
            }
            let mut good_license = true;
            loop {
                self.sync.attorney_rate_limiter().await;
                license = match http_post(
                    &self.url,
                    self.timeout,
                    payload.clone(),
                    &self.client,
                    self.sync.clone(),
                )
                .await
                {
                    Ok(l) => l,
                    Err(e) => {

                        let metric = Metrics::new_license(0, payload.len() as u64);

                        if e.status == 402 {
                            self.sync
                                .empty_license_sender
                                .send(License::new())
                                .await
                                .unwrap();
                            good_license = false;
                            break;
                        }

                        self.sync.metrics_sender.send(metric).await.unwrap();


                        //self.sync.empty_license_sender.send(license).await.unwrap();
                        //self.sync.cash_sender.send(payload).await.unwrap();
                        continue;
                    }
                };
                break;
            }
            if good_license {

                let metric = Metrics::new_license(license.dig_allowed, payload.len() as u64);
                match self.sync.license_sender.send(license).await {
                    Ok(_) => {

                        self.sync.metrics_sender.send(metric).await.unwrap();

                    }
                    Err(e) => println!("attorney error - {}", e.to_string()),
                }
            }
        }
    }
}
