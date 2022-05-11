use axum::{routing::get, routing::post, Extension, Json, Router};
#[allow(unused_imports)]
use axum_macros::debug_handler;
use config::Config;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio;

#[allow(dead_code)]
#[tokio::main]
async fn main() {
    // load configuration from ./config.toml
    let config: Configuration = Config::builder()
        // Add in settings from `./config.toml`
        .add_source(config::File::with_name("config.toml"))
        .build()
        .unwrap()
        .try_deserialize::<Configuration>()
        .unwrap();

    //  make sure to reuse this for performance instead of re instantiating
    let client = reqwest::Client::new();

    // gives handles acces to config values via Extension as an extractor
    let shared_state = Arc::new(State {
        config: config,
        client: client,
    });

    #[allow(unused_must_use)]
    // conveniance/readability function for assembling an api request from config values and the users search term
    // TODO: handle errors
    async fn search_sonarr(
        client: Client,
        config: Configuration,
        search_phrase: String,
    ) -> Result<SonarrResponse, reqwest::Error> {
        let response = client
            //TODO: refactor the building of url and search term for better understanding
            .get(format!(
                "{}/api/series/lookup?term={}&apikey={}",
                config.sonarr_url,
                search_phrase.replace(" ", "%20"),
                config.sonarr_api_key
            ))
            .send()
            .await?
            // desirialise the json response into SonarrResponse
            .json::<SonarrResponse>()
            .await;

        return response;
    }
    async fn search_sonarr_handler(
        Json(payload): Json<Search>,
        Extension(state): Extension<Arc<State>>,
    ) -> Json<SonarrResponse> {
        // return response as json
        Json(
            search_sonarr(
                state.client.clone(),
                state.config.clone(),
                payload.search_phrase,
            )
            .await
            .unwrap(),
        )
    }
    #[derive(Deserialize, Serialize, Debug)]
    struct Search {
        search_phrase: String,
    }

    // build application routes
    let app = Router::new()
        .route("/", get(search_sonarr_handler))
        .layer(Extension(shared_state));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
#[derive(Debug)]
struct State {
    config: Configuration,
    client: Client,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Configuration {
    pub sonarr_url: String,
    pub radarr_url: String,
    pub lidarr_url: String,
    pub sonarr_api_key: String,
    pub radarr_api_key: String,
    pub lidarr_api_key: String,
}
// these are generated with https://typegen.vestera.as/
pub type SonarrResponse = Vec<SonarrResponse2>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SonarrResponse2 {
    pub title: String,
    pub sort_title: String,
    pub season_count: i64,
    pub status: String,
    pub overview: Option<String>,
    pub images: Vec<Images>,
    pub remote_poster: Option<String>,
    pub seasons: Vec<Season>,
    pub year: i64,
    pub profile_id: i64,
    pub language_profile_id: i64,
    pub season_folder: bool,
    pub monitored: bool,
    pub use_scene_numbering: bool,
    pub runtime: i64,
    pub tvdb_id: i64,
    pub first_aired: Option<String>,
    pub series_type: String,
    pub clean_title: String,
    pub title_slug: String,
    pub genres: Vec<String>,
    pub tags: Vec<Value>,
    pub added: String,
    pub ratings: Ratings,
    pub network: Option<String>,
    pub air_time: Option<String>,
    pub imdb_id: Option<String>,
    pub certification: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Images {
    pub cover_type: String,
    pub url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Season {
    pub season_number: i64,
    pub monitored: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ratings {
    pub votes: i64,
    pub value: f64,
}
