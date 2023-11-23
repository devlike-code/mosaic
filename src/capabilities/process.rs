use std::{collections::HashMap, sync::Arc};

use itertools::Itertools;

use crate::{
    internals::{
        default_vals, self_val, Logging, Mosaic, MosaicCRUD, MosaicIO, Tile, TileFieldGetter,
        Value, S32,
    },
    iterators::{tile_deletion::TileDeletion, tile_getters::TileGetters},
};

use super::GroupingCapability;

pub trait ProcessCapability: GroupingCapability {
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

    fn add_process_result(&self, process: &Tile, result: &Tile) -> anyhow::Result<()>;
}

impl ProcessCapability for Arc<Mosaic> {
    fn create_process(&self, name: &str, params: &[&str]) -> anyhow::Result<Tile> {
        let process = self.new_object("Process", self_val(Value::S32(name.into())));

        self.group(name, &process, &[]);
        let process_desc = self.get_group_owner_descriptor(name, &process).unwrap();

        for &param in params {
            let param_obj = self.new_extension(
                &process_desc,
                "ProcessParameter",
                self_val(Value::S32(param.into())),
            );
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

        let binding = process.get("self").as_s32().to_string();
        let group_name = binding.as_str();

        let param = self
            .get_group_members(group_name, process)
            .filter(|t| t.get("self").as_s32() == param_name.into())
            .collect_vec();

        if param.len() != 1 {
            return format!("Process {:?} doesn't have exactly one member named {}. Cannot pass parameter {:?}.", process, param_name, value).to_error();
        }

        let param = param.first().unwrap();
        param.clone().into_iter().get_arrows_from().delete();
        self.new_arrow(
            param,
            value,
            "ParameterBinding",
            self_val(Value::S32(param_name.into())),
        );

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

        let binding = process.get("self").as_s32().to_string();
        let group_name = binding.as_str();

        let param = self
            .get_group_members(group_name, process)
            .filter(|t| t.get("self").as_s32() == param_name.into())
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
            .into_iter()
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

        let binding = process.get("self").as_s32().to_string();
        let group_name = binding.as_str();

        Ok(self
            .get_group_members(group_name, process)
            .map(|t| {
                (
                    t.get("self").as_s32(),
                    t.into_iter()
                        .get_arrows_from()
                        .get_targets()
                        .collect_vec()
                        .first()
                        .cloned(),
                )
            })
            .collect::<HashMap<_, _>>())
    }

    fn add_process_result(&self, process: &Tile, result: &Tile) -> anyhow::Result<()> {
        if process.component != "Process".into() {
            return format!("Tile {:?} does not represent a process; use `create_process(name: &str) -> Tile` to make one.", process).to_error();
        }

        let binding = process.get("self").as_s32().to_string();
        let group_name = binding.as_str();
        let process_desc = self
            .get_group_owner_descriptor(group_name, process)
            .unwrap();

        let result_ext = self.new_extension(&process_desc, "ProcessResult", default_vals());
        self.add_group_member(group_name, &process_desc, &result_ext)?;

        self.new_arrow(&result_ext, result, "ResultBinding", default_vals());

        Ok(())
    }
}
