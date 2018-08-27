extern crate actix_web;
#[macro_use] extern crate askama;
extern crate chrono;
extern crate config;
#[macro_use] extern crate log;
extern crate rand;
extern crate redis;
#[macro_use] extern crate serde_json;
mod controller;
mod images;
mod rand_iter;
mod logger;

use actix_web::{server, App, HttpResponse, fs};
use actix_web::http::Method;
use actix_web::middleware::Logger;
use config::Config;
use controller::{AppParams, Controller, AppState};
use images::Endpoints;
use std::net::SocketAddr;

fn create_app(
	settings_file : &Config, 
	settings : AppParams) -> App<AppState>
{
	let mut endpoints = Endpoints::new();
	let table = settings_file.get_table("endpoints").unwrap();
	for key in table.keys()
	{
		let v = table.get(key);
		match v
		{
			None => (),
			Some(s) => {
				endpoints.add(key, s.clone().into_str().unwrap()).unwrap();
				debug!("loaded endpoint {}", key);
			}
		}
	}

	let state = AppState { endpoints: endpoints, settings: settings };

	App::with_state(state)
		.middleware(Logger::new("%{X-Real-IP}i \"%r\" %s %b \"%{User-Agent}i\" %Dms"))
		.resource(
			"/",
			|r| r.method(Method::GET).f(Controller::index))
		.resource(
			"/api/{endpoint}", 
			|r| r.method(Method::GET).f(Controller::get_endpoint))
		.handler("/assets", fs::StaticFiles::new("assets").unwrap())
		.default_resource(|r| {
			r.route().f(|_| HttpResponse::NotFound().body("Not found."))
		})
}

fn main()
{
	let mut settings_file = Config::default();

	settings_file
		.merge(config::File::with_name("Settings")).unwrap()
		.merge(config::Environment::with_prefix("SHIBE")).unwrap();

	let default_level = "trace";
	let level = 
		settings_file
		.get_str("log_level")
		.unwrap_or(default_level.to_owned())
		.parse::<log::Level>()
		.unwrap();

	let console_enabled = settings_file.get_bool("log_console").unwrap();
	let conn_str = settings_file.get_str("redis_url").unwrap();
	let channel = settings_file.get_str("log_channel").unwrap();
	let client = redis::Client::open(conn_str.as_str()).unwrap();
	let conn = client.get_connection().unwrap();

	logger::init(conn, channel, console_enabled, level).unwrap();

	info!("connected to redis on {}", conn_str);
	info!("starting server");

	let settings = AppParams {
		http_url: settings_file.get_str("base_http_url").unwrap(),
		https_url: settings_file.get_str("base_https_url").unwrap(),
	};

	let port : u16 = settings_file.get_int("port").unwrap_or(6767) as u16;
	
	server::new(move || create_app(&settings_file, settings.clone()))
		.bind(SocketAddr::from(([127, 0, 0, 1], port)))
		.unwrap()
		.run();
}