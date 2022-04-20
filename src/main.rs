use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    process,
};
use yaml_rust::{Yaml, YamlLoader};

const BASE_URL: &str = "https://github.com/autowarefoundation/autoware";
const BASE_RAW_URL: &str =
    "https://raw.githubusercontent.com/autowarefoundation/autoware/main/autoware.repos";

fn main() -> Result<(), Box<dyn Error>> {
    let mut visited = BTreeSet::new();
    let mut graph = BTreeMap::new();
    load_url(BASE_URL, BASE_RAW_URL, &mut graph, &mut visited)?;
    print_graph(&graph);

    Ok(())
}

fn load_url(
    url: &str,
    raw_url: &str,
    graph: &mut BTreeMap<String, Vec<String>>,
    visited: &mut BTreeSet<String>,
) -> Result<(), Box<dyn Error>> {
    if visited.contains(raw_url) {
        return Ok(());
    }

    eprintln!("opening {raw_url}");
    let out = process::Command::new("curl").args(&[raw_url]).output()?;
    let stdout = std::str::from_utf8(&out.stdout)?;
    if stdout == "404: Not Found" {
        return Ok(());
    }

    visited.insert(raw_url.to_string());

    analyze_yaml(url, raw_url, stdout, graph, visited)?;

    Ok(())
}

fn analyze_yaml(
    from_url: &str,
    from_raw_url: &str,
    yaml_str: &str,
    graph: &mut BTreeMap<String, Vec<String>>,
    visited: &mut BTreeSet<String>,
) -> Result<(), Box<dyn Error>> {
    let yml = YamlLoader::load_from_str(yaml_str)?;
    if yml.is_empty() {
        return Err(format!("{from_raw_url} was empty").into());
    }

    if let Yaml::Hash(repositories) = &yml[0] {
        let repos = repositories
            .get(&Yaml::String("repositories".to_string()))
            .ok_or("repositories was not found")?;
        if let Yaml::Hash(entries) = repos {
            for (k, v) in entries {
                match (k, v) {
                    (Yaml::String(key), Yaml::Hash(val)) => {
                        let ty = val
                            .get(&Yaml::String("type".to_string()))
                            .ok_or("type was not found")?;
                        let url = val
                            .get(&Yaml::String("url".to_string()))
                            .ok_or("url was not found")?;
                        let ver = val
                            .get(&Yaml::String("version".to_string()))
                            .ok_or("version was not found")?;

                        match (ty, url, ver) {
                            (Yaml::String(_ty), Yaml::String(url), Yaml::String(ver)) => {
                                if let Some(urls) = graph.get_mut(from_url) {
                                    urls.push(url.clone());
                                } else {
                                    graph.insert(from_url.to_string(), vec![url.clone()]);
                                }

                                let raw_url = to_raw_url(url, ver);
                                load_url(url, &raw_url, graph, visited)?;
                            }
                            _ => {
                                eprintln!("{key} : {:?}, {:?}, {:?}", ty, url, ver);
                                return Err("invalid entry".into());
                            }
                        }
                    }
                    _ => {
                        eprintln!("{:?} : {:?}", k, v);
                        return Err("invalid entry".into());
                    }
                }
            }
        }
    } else {
        return Err(format!("{from_raw_url} was invalid format").into());
    }

    Ok(())
}

fn to_raw_url(url: &str, ver: &str) -> String {
    url.replace(".git", "")
        .replace("github", "raw.githubusercontent")
        + "/"
        + ver
        + "/build_depends.repos"
}

fn print_graph(graph: &BTreeMap<String, Vec<String>>) {
    println!("graph LR;");
    for (from, tos) in graph {
        let from = from.replace("https://github.com/", "");
        for to in tos {
            let to = to.replace("https://github.com/", "");
            println!("    {from}-->{to};");
        }
    }
}