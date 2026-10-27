use std::sync::Arc;

use anyhow::bail;
use sqrilizz_parse::conditions::ConditionKind;
use sqrilizz_parse::parse::Parsed;
use sqrilizz_parse::vars::{HashMapVariableStore, ReservedConstantVariables, VariableStore};
use sqrilizz_pkg::RecommendedPackage;
use sqrilizz_pkg::properties::PackageProperties;
use sqrilizz_pkg::script_eval::{
	AddonInstructionData, ScriptEvalConfig, ScriptEvaluator as ScriptEvaluatorTrait,
};
use sqrilizz_shared::pkg::{ArcPkgReq, PackageID};

use crate::io::paths::Paths;

use super::conditions::eval_condition;
use super::{
	EvalData, EvalInput, EvalPermissions, MAX_NOTICE_CHARACTERS, MAX_NOTICE_INSTRUCTIONS,
	RequiredPackage, Routine, create_valid_addon_request,
};

struct SharedData<'a> {
	eval: EvalData,
	#[allow(dead_code)]
	paths: &'a Paths,
}

/// Evaluate a script package
pub async fn eval_script_package(
	req: &ArcPkgReq,
	parsed: &Parsed,
	routine: Routine,
	properties: Arc<PackageProperties>,
	input: EvalInput,
	paths: &Paths,
) -> anyhow::Result<EvalData> {
	let mut eval = EvalData::new(input, req.clone(), properties, &routine);

	eval.vars.set_reserved_constants(ReservedConstantVariables {
		mc_version: eval.input.constants.version.as_deref(),
	});

	let reason = eval.reason;
	let mut data = SharedData { eval, paths };

	sqrilizz_pkg::script_eval::eval_script_package(
		parsed,
		&mut ScriptEvaluator,
		&mut data,
		&ScriptEvalConfig { reason },
	)
	.await?;

	Ok(data.eval)
}

struct ScriptEvaluator;

#[async_trait::async_trait]
impl ScriptEvaluatorTrait for ScriptEvaluator {
	type Shared<'a> = SharedData<'a>;
	type VariableStore = HashMapVariableStore;

	fn get_variable_store<'a>(
		&self,
		shared: &'a mut Self::Shared<'_>,
	) -> &'a mut Self::VariableStore {
		&mut shared.eval.vars
	}

	fn eval_condition(
		&mut self,
		shared: &mut Self::Shared<'_>,
		condition: &ConditionKind,
	) -> anyhow::Result<bool> {
		eval_condition(condition, &shared.eval)
	}

	fn add_addon(
		&mut self,
		shared: &mut Self::Shared<'_>,
		addon: AddonInstructionData,
	) -> anyhow::Result<()> {
		if shared
			.eval
			.addon_reqs
			.iter()
			.any(|x| x.addon.id == addon.id)
		{
			bail!("Duplicate addon id '{}'", addon.id);
		}

		let addon_req =
			create_valid_addon_request(addon, shared.eval.req.clone(), &shared.eval.input)?;
		shared.eval.addon_reqs.push(addon_req);

		Ok(())
	}

	fn add_bundled(&mut self, shared: &mut Self::Shared<'_>, pkg: PackageID) -> anyhow::Result<()> {
		shared.eval.bundled.push(pkg);
		Ok(())
	}

	fn add_command(
		&mut self,
		shared: &mut Self::Shared<'_>,
		command: Vec<String>,
	) -> anyhow::Result<()> {
		match shared.eval.input.params.perms {
			EvalPermissions::Elevated => {}
			_ => bail!("Insufficient permissions to run the 'cmd' instruction"),
		}
		shared.eval.commands.push(command);
		Ok(())
	}

	fn add_compat(
		&mut self,
		shared: &mut Self::Shared<'_>,
		compat: (PackageID, PackageID),
	) -> anyhow::Result<()> {
		shared.eval.compats.push(compat);
		Ok(())
	}

	fn add_conflict(
		&mut self,
		shared: &mut Self::Shared<'_>,
		pkg: PackageID,
	) -> anyhow::Result<()> {
		shared.eval.conflicts.push(pkg);
		Ok(())
	}

	fn add_dependency(
		&mut self,
		shared: &mut Self::Shared<'_>,
		dep: Vec<RequiredPackage>,
	) -> anyhow::Result<()> {
		shared.eval.deps.push(dep);
		Ok(())
	}

	fn add_extension(
		&mut self,
		shared: &mut Self::Shared<'_>,
		pkg: PackageID,
	) -> anyhow::Result<()> {
		shared.eval.extensions.push(pkg);
		Ok(())
	}

	fn add_inclusion(
		&mut self,
		shared: &mut Self::Shared<'_>,
		pkg: PackageID,
	) -> anyhow::Result<()> {
		shared.eval.inclusions.push(pkg);
		Ok(())
	}

	fn add_notice(&mut self, shared: &mut Self::Shared<'_>, notice: String) -> anyhow::Result<()> {
		if shared.eval.notices.len() > MAX_NOTICE_INSTRUCTIONS {
			bail!("Max number of notice instructions was exceded (>{MAX_NOTICE_INSTRUCTIONS})");
		}
		if notice.len() > MAX_NOTICE_CHARACTERS {
			bail!("Notice message is too long (>{MAX_NOTICE_CHARACTERS})");
		}
		shared.eval.notices.push(notice);
		Ok(())
	}

	fn add_recommendation(
		&mut self,
		shared: &mut Self::Shared<'_>,
		pkg: RecommendedPackage,
	) -> anyhow::Result<()> {
		shared.eval.recommendations.push(pkg);
		Ok(())
	}

	async fn run_custom(
		&mut self,
		_shared: &mut Self::Shared<'_>,
		command: String,
		_args: Vec<String>,
	) -> anyhow::Result<()> {
		Err(anyhow::anyhow!(
			"Custom instruction '{}' is not supported without plugins",
			command
		))
	}
}
