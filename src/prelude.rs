#![allow(unused)]
use crate::{
    error::{Kind, Result},
    {err_new, err_new_io},
};
use std::{collections::HashMap, fmt::Debug, fs::File, path::Path};

pub const RESOURCE: &str = "../resources";

pub fn debug_print<T: Debug>(s: T) {
    #[cfg(debug_assertions)]
    dbg!(s);
}

pub fn read_json<P, T>(file: P) -> Result<Vec<T>>
where
    P: AsRef<Path>,
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    let file = File::open(file.as_ref()).map_err(|e| err_new_io!(e))?;
    Ok(serde_json::from_reader(file).map_err(|e| err_new!(Kind::Other, &e.to_string()))?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        println!("{:?}", std::fs::canonicalize(RESOURCE));
    }
}
