use xcurl::config::YamlConf;

fn main() {
    let content = include_str!("../fixtures/test.yaml");
    let conf = YamlConf::from_yaml(content).unwrap();
    println!("Conf: {:?}", conf);
    println!("Yaml: {}", conf.to_yaml().unwrap());
}
