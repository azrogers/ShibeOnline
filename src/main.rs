extern crate actix_web;
#[macro_use] extern crate askama;
extern crate config;
#[macro_use] extern crate log;
extern crate log4rs;
extern crate rand;
mod controller;
mod images;
mod rand_iter;

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
		.middleware(Logger::default())
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
	log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
	info!("starting server");

	let mut settings_file = Config::default();

	settings_file
		.merge(config::File::with_name("Settings")).unwrap()
		.merge(config::Environment::with_prefix("SHIBE")).unwrap();

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