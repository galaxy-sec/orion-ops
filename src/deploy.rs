use crate::{
    error::RunResult,
    resource::ResourceNode,
    system::SysModelSpec,
    task::{SetupTaskBuilder, TaskHandle},
};
use orion_exchange::vars::VarCollection;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InfraConfig {
    artifact_repo: String,
    code_repo: String,
}
#[derive(Clone, Debug)]
pub struct CustomModelConf {
    model_spec: SysModelSpec,
    //infra_conf: InfraConfig,
    #[allow(dead_code)]
    res_node: ResourceNode,
    #[allow(dead_code)]
    run_vars: VarCollection,
}

impl CustomModelConf {
    pub fn make_task(&self) -> RunResult<TaskHandle> {
        self.model_spec.make_setup_task()
    }

    pub fn new(model_spec: SysModelSpec, res_node: ResourceNode, run_vars: VarCollection) -> Self {
        Self {
            model_spec,
            res_node,
            run_vars,
        }
    }
}

pub struct DeployJob {}

#[cfg(test)]
mod tests {
    use super::CustomModelConf;
    use crate::{error::RunResult, resource::ResourceNode, system::tests::gateway_spec};
    use orion_exchange::vars::{VarCollection, VarType};

    #[test]
    fn test_seup() -> RunResult<()> {
        let spec = gateway_spec()?;
        let res_node = ResourceNode::localhost(2, 4);
        let run_vars = VarCollection::define(vec![VarType::from(("IP", "127.0.0.1"))]);
        let deploy_plan = CustomModelConf::new(spec, res_node, run_vars);
        let job = deploy_plan.make_task()?;
        job.exec()?;
        //job.report();
        Ok(())
    }
}
