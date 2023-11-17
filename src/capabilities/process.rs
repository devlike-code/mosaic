use std::{collections::HashMap, sync::Arc};

use itertools::Itertools;

use crate::{
    internals::{Logging, Mosaic, MosaicCRUD, Tile, TileCommit, Value, S32},
    iterators::{
        deletion::DeleteTiles, get_arrows_from::GetArrowsFromTiles, get_targets::GetTargets,
    },
};

use super::GroupingCapability;

trait ProcessCapability: GroupingCapability {
    fn create_process(&self, name: &str, params: &[&str]) -> anyhow::Result<Tile>;
    fn pass_process_parameter(
        &self,
        process: &Tile,
        param_name: &str,
        value: &Tile,
    ) -> anyhow::Result<()>;

    fn get_process_parameter_value(
        &self,
        process: &Tile,
        param_name: &str,
    ) -> anyhow::Result<Option<Tile>>;

    fn get_process_parameter_values(
        &self,
        process: &Tile,
    ) -> anyhow::Result<HashMap<S32, Option<Tile>>>;
}

impl ProcessCapability for Arc<Mosaic> {
    fn create_process(&self, name: &str, params: &[&str]) -> anyhow::Result<Tile> {
        let mut process = self.new_object("Process");
        process["self"] = Value::S32(name.into());
        process.commit(Arc::clone(self))?;

        self.group(name, &process, &[]);

        for &param in params {
            let mut param_obj = self.new_object("ProcessParameter");
            param_obj["self"] = Value::S32(param.into());
            param_obj.commit(Arc::clone(self))?;

            self.add_group_member(name, &process, &param_obj)?;
        }

        Ok(process)
    }

    fn pass_process_parameter(
        &self,
        process: &Tile,
        param_name: &str,
        value: &Tile,
    ) -> anyhow::Result<()> {
        if process.component != "Process".into() {
            return format!("Tile {:?} does not represent a process; use `create_process(name: &str) -> Tile` to make one.", process).to_error();
        }

        let binding = process["self"].as_s32().to_string();
        let group_name = binding.as_str();

        let param = self
            .get_group_members(group_name, process)
            .filter(|t| t["self"].as_s32() == param_name.into())
            .collect_vec();

        if param.len() != 1 {
            return format!("Process {:?} doesn't have exactly one member named {}. Cannot pass parameter {:?}.", process, param_name, value).to_error();
        }

        let param = param.first().unwrap();
        param.iter_with(self).get_arrows_from().delete();
        let mut binding = self.new_arrow(param, value, "ParameterBinding");
        binding["self"] = Value::S32(param_name.into());
        self.commit(&binding)?;

        Ok(())
    }

    fn get_process_parameter_value(
        &self,
        process: &Tile,
        param_name: &str,
    ) -> anyhow::Result<Option<Tile>> {
        if process.component != "Process".into() {
            return format!("Tile {:?} does not represent a process; use `create_process(name: &str) -> Tile` to make one.", process).to_error();
        }

        let binding = process["self"].as_s32().to_string();
        let group_name = binding.as_str();

        let param = self
            .get_group_members(group_name, process)
            .filter(|t| t["self"].as_s32() == param_name.into())
            .collect_vec();

        if param.len() != 1 {
            return format!(
                "Process {:?} doesn't have exactly one member named {}. Cannot get parameter.",
                process, param_name
            )
            .to_error();
        }

        Ok(param
            .first()
            .unwrap()
            .clone()
            .iter_with(self)
            .get_arrows_from()
            .get_targets()
            .collect_vec()
            .first()
            .cloned())
    }

    fn get_process_parameter_values(
        &self,
        process: &Tile,
    ) -> anyhow::Result<HashMap<S32, Option<Tile>>> {
        if process.component != "Process".into() {
            return format!("Tile {:?} does not represent a process; use `create_process(name: &str) -> Tile` to make one.", process).to_error();
        }

        let binding = process["self"].as_s32().to_string();
        let group_name = binding.as_str();

        Ok(self
            .get_group_members(group_name, process)
            .map(|t| {
                (
                    t["self"].as_s32(),
                    t.iter_with(self)
                        .get_arrows_from()
                        .get_targets()
                        .collect_vec()
                        .first()
                        .cloned(),
                )
            })
            .collect::<HashMap<_, _>>())
    }
}

#[cfg(test)]
mod process_tests {
    use std::sync::Arc;

    use crate::internals::{Logging, Mosaic, MosaicTypelevelCRUD, Tile, TileCommit, Value};

    use super::ProcessCapability;

    #[test]
    fn test_processes() {
        let mosaic = Mosaic::new();
        mosaic.new_type("Number: u32;").unwrap();

        let add = mosaic.create_process("add", &["a", "b"]).unwrap();

        let mut x = mosaic.new_object("Number");
        x["self"] = Value::U32(7);
        mosaic.commit(&x).unwrap();

        let mut y = mosaic.new_object("Number");
        y["self"] = Value::U32(5);
        mosaic.commit(&y).unwrap();

        mosaic.pass_process_parameter(&add, "a", &x).unwrap();
        mosaic.pass_process_parameter(&add, "b", &y).unwrap();

        fn do_add(mosaic: &Arc<Mosaic>, add_instance: &Tile) -> anyhow::Result<u32> {
            let args = mosaic.get_process_parameter_values(add_instance)?;
            let a = args.get(&"a".into()).unwrap();
            let b = args.get(&"b".into()).unwrap();

            match (&a, &b) {
                (Some(a), Some(b)) => Ok(a["self"].as_u32() + b["self"].as_u32()),
                _ => "Can't do add :(".to_error(),
            }
        }

        assert_eq!(12, do_add(&mosaic, &add).unwrap());
    }
}
