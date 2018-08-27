extern crate num;
extern crate serde_json;

use actix_web::{HttpRequest, HttpResponse};
use actix_web::dev::HttpResponseBuilder;
use actix_web::http::ContentEncoding;
use askama::Template;
use images::{Endpoints, ImageManager};

pub struct AppParams
{
	pub http_url: String,
	pub https_url: String
}

impl AppParams
{
	pub fn clone(&self) -> Self
	{
		Self 
		{ 
			http_url: self.http_url.clone(), 
			https_url: self.https_url.clone() 
		}
	}
}

pub struct AppState
{
	pub endpoints: Endpoints,
	pub settings: AppParams
}

pub struct Controller { }

struct ApiParams
{
	count: usize,
	urls: bool,
	https_urls: bool
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a>
{
	dog: &'a str
}

// gets the value of the query string as a usize, or returns the default
fn get_query_usize(req: &HttpRequest<AppState>, key: &'static str, default: usize) -> usize
{
	let query = req.query();
	match query.get(key) {
		Some(s) => s.parse::<usize>().unwrap_or(default),
		None => default
	}
}

fn get_query_bool(req: &HttpRequest<AppState>, key: &'static str, default: bool) -> bool
{
	let query = req.query();
	match query.get(key) {
		Some(s) => s == "true",
		None => default
	}
}

fn ok(content: String) -> HttpResponse
{
	HttpResponse::Ok()
		.content_encoding(ContentEncoding::Auto)
		.content_type("application/json")
		.body(content)
}

fn error(mut res : HttpResponseBuilder, message: &'static str) -> HttpResponse
{
	error!("sending error response: {}", message);
	res
		.content_encoding(ContentEncoding::Auto)
		.content_type("application/json")
		.body(message)
}

impl Controller
{
	pub fn index(req: &HttpRequest<AppState>) -> HttpResponse
	{
		let im = req.state().endpoints.get("shibes").unwrap();
		let params = ApiParams { count: 1, https_urls: true, urls: true };
		let dog_rand = Controller::get_images_for(&req, &im, "shibes", &params);
		let none = String::from("");
		let index = IndexTemplate { dog: dog_rand.first().unwrap_or(&none) };
		match index.render() {
			Ok(html) => HttpResponse::Ok().body(html),
			Err(_e) => error(HttpResponse::InternalServerError(), "Failed to render template.")
		}
	}

	pub fn get_endpoint(req: &HttpRequest<AppState>) -> HttpResponse
	{
		let options = Controller::parse_api_params(&req);
		let endpoint = req.match_info().get("endpoint").unwrap_or("unknown");
		debug!("request to endpoint {}", endpoint);

		match req.state().endpoints.get(endpoint) {
			Some(im) => Controller::serialize_images(&req, &im, endpoint, &options),
			None => error(HttpResponse::NotFound(), "Invalid endpoint.")
		}
	}

	fn parse_api_params(req: &HttpRequest<AppState>) -> ApiParams
	{
		let count = num::clamp(get_query_usize(&req, "count", 1), 1, 100);
		ApiParams { 
			count: count, 
			urls: get_query_bool(&req, "urls", true),
			https_urls: get_query_bool(&req, "httpsUrls", true)
		}
	}

	fn handle_url(settings: &AppParams, options: &ApiParams, endpoint: &str, file: &str) -> String
	{
		if !options.urls 
		{
			// split on the last period and take the first item (the filename without extension)
			file.splitn(2, ".").next().unwrap().to_owned()
		}
		else if options.https_urls
		{
			let mut base = settings.https_url.clone();
			base.push_str(endpoint);
			base.push('/');
			base.push_str(file);
			base
		}
		else
		{
			let mut base = settings.http_url.clone();
			base.push_str(endpoint);
			base.push('/');
			base.push_str(file);
			base
		}
	}

	fn get_images_for(
		req: &HttpRequest<AppState>, 
		im: &ImageManager, 
		endpoint: &str, 
		options: &ApiParams) -> Vec<String>
	{
		return im.get_rand_iter(options.count)
			.map(|f| Controller::handle_url(&req.state().settings, &options, &endpoint, &f))
			.collect();
	}

	fn serialize_images(
		req: &HttpRequest<AppState>,
		im: &ImageManager, 
		endpoint: &str, 
		options: &ApiParams) -> HttpResponse
	{
		let rand = Controller::get_images_for(&req, &im, &endpoint, &options);
		match serde_json::to_string(&rand) {
			Ok(json) => ok(json),
			Err(_e) => error(HttpResponse::InternalServerError(), "Couldn't serialize images.")
		}
	}
}