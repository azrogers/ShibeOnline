extern crate log;

use log::{Log, Level, Metadata, Record, SetLoggerError};
use redis::{Commands, Connection, RedisResult};
use std::sync::Mutex;
use chrono::Utc;

struct Logger 
{
	level: Level,
	conn: Mutex<Connection>,
	console: bool,
	channel: String,
}

impl Log for Logger 
{
	fn enabled(&self, metadata: &Metadata) -> bool
	{
		metadata.level() <= self.level
	}

	fn log(&self, record: &Record) 
	{
		if self.enabled(record.metadata()) && self.console
		{
			let msg = json!({
				"time": Utc::now().to_rfc3339(),
				"level": record.level().to_string(),
				"module": record.module_path().unwrap_or_default(),
				"line": record.line(),
				"args": record.args()
			});

			let conn = &*self.conn.lock().unwrap();
			//let res : RedisResult<()> = conn.publish(&self.channel, msg.to_string());
			let res : RedisResult<isize> = conn.lpush(&self.channel, msg.to_string());
			res.ok();

			println!(
				"{} {:<5} [{}] {}",
				Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
				record.level().to_string(),
				record.module_path().unwrap_or_default(),
				record.args());
		}
	}

	fn flush(&self) 
	{
	}
}

pub fn init(conn: Connection, channel: String, console: bool, level: Level) -> Result<(), SetLoggerError> 
{
	let logger = Logger { conn: Mutex::new(conn), console, level, channel };
	log::set_boxed_logger(Box::new(logger))?;
	log::set_max_level(level.to_level_filter());
	Ok(())
}