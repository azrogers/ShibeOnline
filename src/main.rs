extern crate actix_web;
#[macro_use] extern crate askama;
extern crate config;
#[macro_use] extern crate log;
extern crate log4rs;
extern crate rand;
mod controller;
mod images;
mod rand_iter;

use actix_web::{server, App};
use actix_web::http::Method;
use actix_web::middleware::Logger;
use config::Config;
use controller::{AppParams, Controller};
use images::Endpoints;
use std::net::SocketAddr;

fn create_app(settings : AppParams) -> App
{
	let mut endpoints = Endpoints::new();
	endpoints.add("shibes", "content/shibes/*").unwrap();
	let controller = Controller::new(endpoints, settings).unwrap();

	App::new()
		.middleware(Logger::default())
		.resource(
			"/api/{endpoint}", 
			move |r| {
				r.method(Method::GET).f(move |req| controller.get_endpoint(&req))
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
	
	server::new(move || create_app(settings.clone()))
		.bind(SocketAddr::from(([127, 0, 0, 1], port)))
		.unwrap()
		.run();
}