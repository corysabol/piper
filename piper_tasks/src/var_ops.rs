use pyo3::types::{
   PyDict,
};
use regex::{
    Regex,
    Captures,
};
use std::collections::HashMap;

/// Sets a variable in the shared context
pub fn set_var(args: &HashMap<String, String>, ctx: &PyDict) {
    let var_name = args.get("var");
    let var_value = args.get("val");
    ctx.set_item(var_name, var_value).unwrap();
}

pub fn interpolate_string(str: &String, ctx: &PyDict) -> String {
    let re = Regex::new(r"#\{(.*)\}").unwrap();
    let res = re.replace_all(str, |caps: &Captures| {
        format!("{}", ctx.get_item(&caps[1]).unwrap())
    });

    res.to_string()
}
