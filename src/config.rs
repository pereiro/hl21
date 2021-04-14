use envconfig::Envconfig;
use std::env;
use std::fmt::{Display, Formatter, Result};
use url::Url;

#[derive(Envconfig, Copy, Clone)]
pub struct Config {
    #[envconfig(from = "ATTORNEYS_NUM", default = "8")]
    pub attorneys_num: u64,
    #[envconfig(from = "DIGGERS_NUM", default = "8")]
    pub diggers_num: u64,
    #[envconfig(from = "ACCOUNTANT_NUM", default = "8")]
    pub accountant_num: u64,

    #[envconfig(from = "SEARCH_BINARY_ENABLED", default = "true")]
    pub search_binary_enabled: bool,
    #[envconfig(from = "SEARCH_INITIAL_ARRAY_SIZE", default = "31")]
    pub search_initial_array_size: u64,
    #[envconfig(from = "SEARCH_MIN_AMOUNT", default = "1")]
    pub search_min_amount: u64,
    #[envconfig(from = "SEARCH_TO_FLAT_THRESHOLD", default = "31")]
    pub search_to_flat_threshold: u64,
    #[envconfig(from = "SEARCH_FLAT_SIZE", default = "3")]
    pub search_flat_size: u64,
    #[envconfig(from = "SEARCH_EXPLORERS_NUM", default = "8")]
    pub search_explorers_num: u64,

    #[envconfig(from = "DIGGER_MIN_DEPTH", default = "3")]
    pub digger_min_depth: u64,
    #[envconfig(from = "DIGGER_MAX_DEPTH", default = "10")]
    pub digger_max_depth: u64,
    #[envconfig(from = "DIGGER_MIN_DEPTH_PROBABILITY", default = "100")]
    pub digger_min_depth_probability: u64,

    #[envconfig(from = "ATTORNEY_LICENSE_MIN_COST", default = "1")]
    pub attorney_license_min_cost: u64,
    #[envconfig(from = "ATTORNEY_LICENSE_MAX_COST", default = "1")]
    pub attorney_license_max_cost: u64,
    #[envconfig(from = "ATTORNEY_FREE_LICENSE_PROBABILITY", default = "60")]
    pub attorney_free_license_probability: u64,
    #[envconfig(from = "ATTORNEY_HTTP_TIMEOUT_MS", default = "120")]
    pub attorney_http_timeout_ms: u64,

    #[envconfig(from = "ACCOUNTANT_HTTP_TIMEOUT_MS", default = "100")]
    pub accountant_http_timeout_ms: u64,

    #[envconfig(from = "AREA_CHAN_CAP", default = "5")]
    pub area_chan_cap: usize,
    #[envconfig(from = "TILE_CHAN_CAP", default = "5")]
    pub tile_chan_cap: usize,
    #[envconfig(from = "LICENSE_CHAN_CAP", default = "30")]
    pub license_chan_cap: usize,
    #[envconfig(from = "EMPTY_LICENSE_CHAN_CAP", default = "10")]
    pub empty_license_chan_cap: usize,
    #[envconfig(from = "TREASURE_CHAN_CAP", default = "100")]
    pub treasure_chan_cap: usize,

    #[envconfig(from = "STATIST_DISPLAY_TICK", default = "10")]
    pub statist_display_tick: u64,

    #[envconfig(from = "MAX_RPS", default = "1000")]
    pub max_rps: u32,
    #[envconfig(from = "EXPLORE_PHASE1_RPS", default = "650")]
    pub explore_phase1_rps: u32,
    #[envconfig(from = "ACCOUNTANT_PHASE1_RPS", default = "100")]
    pub accountant_phase1_rps: u32,
    #[envconfig(from = "DIGGER_PHASE1_RPS", default = "400")]
    pub digger_phase1_rps: u32,
    #[envconfig(from = "ATTORNEY_PHASE1_RPS", default = "400")]
    pub attorney_phase1_rps: u32,

    #[envconfig(from = "EXPLORE_PHASE2_RPS", default = "1")]
    pub explore_phase2_rps: u32,
    #[envconfig(from = "ACCOUNTANT_PHASE2_RPS", default = "400")]
    pub accountant_phase2_rps: u32,
    #[envconfig(from = "DIGGER_PHASE2_RPS", default = "1")]
    pub digger_phase2_rps: u32,
    #[envconfig(from = "ATTORNEY_PHASE2_RPS", default = "1")]
    pub attorney_phase2_rps: u32,

    #[envconfig(from = "ENABLE_PHASED", default = "false")]
    pub enable_phased: bool,
    #[envconfig(from = "PHASE2_START", default = "450")]
    pub phase2_start: u64,

    #[envconfig(from = "HTTP_TIMEOUT_MS", default = "500")]
    pub http_timeout_ms: u64,

    #[envconfig(from = "WORLD_SIZE", default = "3500")]
    pub world_size: u64,
}

impl Config {
    pub fn get_url(&self) -> Url {
        let mut url = Url::parse("http://localhost:8000").unwrap();
        url.set_scheme("http").unwrap();
        url.set_port(Some(8000)).unwrap();
        let host = match env::var("ADDRESS") {
            Ok(h) => h,
            Err(_) => "localhost".to_string(),
        };
        url.set_host(Some(host.as_str())).unwrap();
        url
    }
    pub fn get_explore_url(&self) -> Url {
        self.get_url().join("explore").unwrap()
    }
    pub fn get_dig_url(&self) -> Url {
        self.get_url().join("dig").unwrap()
    }
    pub fn get_licenses_url(&self) -> Url {
        self.get_url().join("licenses").unwrap()
    }
    pub fn get_cash_url(&self) -> Url {
        self.get_url().join("cash").unwrap()
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "[e={},d={},at={},ac={}][depth={}-{}({})][bin={},array_size={},min={},fthres={},fsize={}][lic={}-{}({}%][ht={},aht={}][ph={}({})]",
            self.search_explorers_num,
            self.diggers_num,
            self.attorneys_num,
            self.accountant_num,
            self.digger_min_depth,
            self.digger_max_depth,
            self.digger_min_depth_probability,
            self.search_binary_enabled,
            self.search_initial_array_size,
            self.search_min_amount,
            self.search_to_flat_threshold,
            self.search_flat_size,
            self.attorney_license_min_cost,
            self.attorney_license_max_cost,
            self.attorney_free_license_probability,
            self.http_timeout_ms,
            self.attorney_http_timeout_ms,
            self.enable_phased,
            self.phase2_start
        )
    }
}
