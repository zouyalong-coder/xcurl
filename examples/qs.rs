use serde::{Deserialize, Serialize};
use serde_qs;

#[derive(Debug, Deserialize, Serialize)]
struct Query {
    a: u8,
    b: u8,
    c: Vec<u8>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Conf {
    params: serde_json::Value,
}

fn main() {
    let c = r#"
    params:
        a: 1
        b[1]: 2
        b[2]: 3
    "#;
    let conf = serde_yaml::from_str::<Conf>(c).unwrap();
    println!("conf: {:?}", conf);
    println!("yaml: \n{}", serde_yaml::to_string(&conf).unwrap());
}
