use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process::exit;
use std::result::Result;

extern crate reqwest;
extern crate serde;
use serde_json;

#[derive(Debug)]
struct Consul {
    kv_prefix: String,
    host: String,
    token: String,
}

#[derive(Debug)]
struct KvString {
    key: String,
    value: serde_json::Value,
}

impl Consul {
    fn store_kv(&self, key: String, value: serde_json::Value) {
        let key_value = format!("{}/{}", self.kv_prefix, key);
        let request_uri = format!("{}/v1/kv/{}", self.host, &key_value);
        let tmp_value = format!("{}", value);

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "x-consul-token",
            reqwest::header::HeaderValue::from_str(&self.token).unwrap(),
        );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();
        let res = client.put(&request_uri).body(tmp_value).send().unwrap();

        if !res.status().is_success() {
            eprintln!(
                "stroing [{}] failed, status_code {}",
                key_value,
                res.status()
            );
        }
    }
}

fn get_consul_info() -> Result<Consul, Box<dyn Error>> {
    let consul_kv_prefix = match env::var("CONSUL_KV_PREFIX") {
        Ok(prefix) => prefix,
        _ => String::from("salt-shared"),
    };
    let consul_host = match env::var("CONSUL_HOST") {
        Ok(host) => host,
        _ => String::from("http://127.0.0.1:8500"),
    };

    let consul_token = env::var("CONSUL_TOKEN")?;

    Ok(Consul {
        kv_prefix: consul_kv_prefix,
        host: consul_host,
        token: consul_token,
    })
}

fn print_help(prog_name: &str) {
    eprintln!(
                "Usage: CONSUL_TOKEN=$consul_token CONSUL_KV_PREFIX='salt-shared' CONSUL_HOST=$consul_http_host {} /path/to/saltstack-pillar.json",
                &prog_name
            );
    eprintln!("CONSUL_KV_PREFIX default to salt-shared");
    eprintln!("CONSUL_host default to http://127.0.0.1:8500");
}

fn main() {
    let all_args: Vec<_> = env::args().collect();
    let prog_name = &all_args[0];

    if all_args.len() != 2 {
        print_help(&prog_name);
        exit(1);
    }

    if &all_args[1] == "-h" || &all_args[1] == "--help" || &all_args[1] == "help" {
        print_help(&prog_name);
        exit(0);
    }

    let consul = match get_consul_info() {
        Ok(consul) => consul,
        _ => {
            print_help(&prog_name);
            panic!("");
        }
    };

    let pillar_file = env::args()
        .skip(1)
        .next()
        .expect("you must give the path of pillar file to read");

    println!(
        "consul info\nhost: {}\nkv_prefix: {}",
        &consul.host, &consul.kv_prefix
    );
    do_work(consul, &pillar_file);
}

fn do_work(consul: Consul, pillar_file: &str) {
    let pillar_file_path = Path::new(pillar_file);

    if !pillar_file_path.exists() {
        panic!("{:?} is not exists", pillar_file_path);
    }

    let kv = parse_pillar_file_as_hashmap(pillar_file_path).unwrap();

    for item in kv {
        consul.store_kv(item.key, item.value);
    }
}

fn parse_pillar_file_as_hashmap(file_name: &Path) -> Result<Vec<KvString>, Box<dyn Error>> {
    let mut file = File::open(file_name)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let json_value: serde_json::Value = serde_json::from_str(&contents)?;

    Ok(flat_json("", &json_value))
}

fn flat_json(old_key: &str, data: &serde_json::Value) -> Vec<KvString> {
    // change pillar json to kv string

    let mut results = Vec::new();
    if data.is_object() {
        for entry in data.as_object().unwrap() {
            let (key, value) = entry;
            // if value is a object, will continue to parsing
            if value.is_object() {
                let new_str: String;
                // strip local prefix in pillar return
                if old_key == "" || old_key == "local" {
                    new_str = format!("{}", key);
                } else {
                    new_str = format!("{}/{}", old_key, key);
                }
                let mut sub_kv = flat_json(&new_str, value);
                results.append(&mut sub_kv);
            } else {
                // string and array will be stop to parse and store
                let kv_key = format!("{}/{}", old_key, key);
                // change key and valut to owned will save a lot of time to add lifetime parameters
                let tmp_kv = KvString {
                    key: kv_key.to_owned(),
                    value: value.to_owned(),
                };
                results.push(tmp_kv);
            }
        }
    }

    return results;
}
