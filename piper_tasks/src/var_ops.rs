use mlua::prelude::*;
use regex::{
    Regex,
    Captures,
};
use std::collections::HashMap;

/// Sets a variable in the shared context
pub fn set_var(args: &HashMap<String, String>, lua: &Lua, ctx: &LuaTable) {
    let var_name = args.get("var").unwrap();
    let var_value = args.get("val").unwrap();
    let globals = lua.globals();
    ctx.set(var_name.clone(),var_value.clone());
    globals.set("ctx", ctx);
}

pub fn interpolate_string(str: &String, ctx: &LuaTable) -> String {
    let re = Regex::new(r"#\{(.*)\}").unwrap();
    let res = re.replace_all(str, |caps: &Captures| {
        format!("{}", ctx.get::<String>(&caps[1]).unwrap())
    });

    res.to_string()
}
