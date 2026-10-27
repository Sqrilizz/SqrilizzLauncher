use sqrilizz_instance::logs::{list_logs, read_log};
use sqrilizz_shared::output::SqrilizzOutput;

use crate::{instance::Instance, io::paths::Paths};

impl Instance {
	/// Get the list of log IDs for this instance
	pub async fn get_logs(
		&self,
		_paths: &Paths,
		_o: &mut impl SqrilizzOutput,
	) -> anyhow::Result<Vec<String>> {
		if let Some(inst_dir) = &self.dir {
			let logs_dir = inst_dir.join("logs");
			list_logs(&logs_dir)
		} else {
			Ok(Vec::new())
		}
	}

	/// Get the contents of a specific log file
	pub async fn get_log(
		&self,
		log_id: &str,
		_paths: &Paths,
		_o: &mut impl SqrilizzOutput,
	) -> anyhow::Result<String> {
		if let Some(inst_dir) = &self.dir {
			let logs_dir = inst_dir.join("logs");
			read_log(&logs_dir.join(log_id))
		} else {
			Ok(String::new())
		}
	}
}
