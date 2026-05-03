use anyhow::Context;

use super::{PROXY_GROUPS, PROXY_PROVIDERS};

/// located in
/// ``` yaml
/// clashtui:
///     uses: ..
/// ```
#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
struct Config {
    /// use which profiles to generate this template,
    /// program will look up these in database,
    /// and **skip** it if not found
    ///
    /// ### Note
    /// there will be **NO** error if not even a single profile is found
    uses: Vec<String>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
struct PGparam {
    /// create one [PGitem] for each related providers,
    /// with name remap to `{name}-{provider_name}`
    ///
    /// e.g. `At-pvd0`
    ///
    /// these are actually prefixs
    providers: Vec<String>,
}
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
struct PGitem {
    /// maybe generated
    name: String,
    #[serde(rename = "use")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// maybe generated
    us_: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// maybe generated
    proxies: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// will be removed, not in final output
    tpl_param: Option<PGparam>,
    #[serde(rename = "type")]
    /// not cared, just keep this
    __type: String,
    /// not cared, just keep this
    #[serde(flatten)]
    __others: serde_yml::Value,
}
#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct PPitem {
    /// maybe generated
    url: Option<String>,
    /// maybe generated
    path: Option<String>,
    #[serde(flatten)]
    /// may contain `tpl_param` as maker,
    /// remove that before final output
    others: serde_yml::Mapping,
    #[serde(rename = "type")]
    /// not cared, just keep this
    __type: String,
}

/*
pub(super) fn gen_template(
    mut tpl: serde_yml::Mapping,
    tpl_name: &str,
    name_urls: Vec<(String, String)>,
) -> anyhow::Result<serde_yml::Mapping> {
    let local_urls: Vec<_> = if let Some(cfg) = tpl.remove("clashtui") {
        let Config { uses } = serde_yml::from_value(cfg)?;
        name_urls
            .into_iter()
            .filter(|(name, _)| uses.contains(name))
            .map(|(_, url)| url)
            .collect()
    } else {
        Default::default()
    };

    // proxy-groups
    let pgs = tpl
        .remove(PROXY_GROUPS)
        .ok_or(anyhow::anyhow!("{PROXY_GROUPS} not found"))?;
    let pgs: Vec<PGitem> = serde_yml::from_value(pgs)?;
    // proxy-providers
    let pps = tpl
        .remove(PROXY_PROVIDERS)
        .ok_or(anyhow::anyhow!("{PROXY_PROVIDERS} not found"))?;
    let pps: HashMap<String, PPitem> = serde_yml::from_value(pps)?;
    // proxy-providers name as key, proxy-providers as value
    let mut extended_proxy_providers = HashMap::new();
    let mut extended_proxy_groups = Vec::new();

    for (pp_name, mut pp) in pps {
        // remove marker
        if pp.others.remove("tpl_param").is_some() {
            // in this iteration, pp is read only after url and path are set
            // so cloning is not needed
            for (idx, url) in local_urls.iter().enumerate() {
                // let mut pp = pp.clone();
                let proxy_provider_name = format!("{pp_name}{idx}");
                pp.url = Some(url.clone());
                pp.path = Some(format!(
                    "proxy-providers/tpl/{tpl_name}/{proxy_provider_name}.yaml"
                ));
                extended_proxy_providers.insert(proxy_provider_name, serde_yml::to_value(&pp)?);
            }
        } else {
            extended_proxy_providers.insert(pp_name, serde_yml::to_value(&pp)?);
        }
    }
    // list to handle 'proxies' section
    let mut proxies_to_do = vec![];
    for mut pg in pgs {
        let ref_mut_vec = if pg
            .proxies
            .as_ref()
            .is_some_and(|v| v.iter().any(|s| s.starts_with('<') && s.ends_with('>')))
        {
            &mut proxies_to_do
        } else {
            &mut extended_proxy_groups
        };
        if let Some(param) = pg.tpl_param.take() {
            let PGparam { providers } = param;
            let pg_name = std::mem::take(&mut pg.name);
            for pp_prefix in providers {
                for pp_name in extended_proxy_providers.keys() {
                    if pp_name.starts_with(&pp_prefix) {
                        let mut new_pg = pg.clone();
                        new_pg.name = format!("{pg_name}-{pp_name}");
                        new_pg.us_ = Some(vec![pp_name.clone()]);
                        ref_mut_vec.push(new_pg);
                    }
                }
            }
        } else {
            ref_mut_vec.push(pg);
        }
    }

    for mut pg in proxies_to_do {
        let proxies = pg.proxies.take().unwrap();
        let mut keep_list: Vec<String> = proxies
            .iter()
            .filter(|s| !(s.starts_with('<') && s.ends_with('>')))
            .cloned()
            .collect();
        let rebind_list: Vec<String> = proxies
            .into_iter()
            .filter(|s| s.starts_with('<') && s.ends_with('>'))
            .map(|s| s.chars().skip(1).take(s.len() - 2).collect())
            .collect();
        let proxies = {
            for pg in &extended_proxy_groups {
                if rebind_list.iter().any(|s| pg.name.starts_with(s)) {
                    keep_list.push(pg.name.clone());
                }
            }
            keep_list
        };
        pg.proxies = Some(proxies);
        extended_proxy_groups.push(pg);
    }

    for pg in &mut extended_proxy_groups {
        if let Some(pp_names) = pg.us_.take() {
            let mut new_pp_names = vec![];
            for pp_name in pp_names {
                if pp_name.starts_with('<') && pp_name.ends_with('>') {
                    let pp_name: String = pp_name.chars().skip(1).take(pp_name.len() - 2).collect();
                    new_pp_names.extend(
                        extended_proxy_providers
                            .keys()
                            .filter(|s| s.starts_with(&pp_name))
                            .cloned(),
                    );
                } else {
                    new_pp_names.push(pp_name);
                }
            }
            pg.us_ = Some(new_pp_names);
        }
    }

    tpl.insert(
        PROXY_PROVIDERS.into(),
        serde_yml::to_value(&extended_proxy_providers)?,
    );
    tpl.insert(
        PROXY_GROUPS.into(),
        serde_yml::to_value(extended_proxy_groups)?,
    );
    Ok(tpl)
}
*/

pub(super) fn gen_template(
    mut tpl: serde_yml::Mapping,
    tpl_name: &str,
    name_urls: Vec<(String, String)>,
) -> anyhow::Result<serde_yml::Mapping> {
    // Ordering guarantee:
    // - proxy-providers use serde_yml::Mapping (backed by indexmap::IndexMap),
    //   which preserves insertion order. Template entries are expanded IN PLACE
    //   (replaced by provider0, provider1, ... at their original position).
    // - proxy-groups use serde_yml::Sequence (Vec), preserving push order.
    //   Non-template groups stay where they are; template groups expand in place.
    // - The clone of tpl into out_parsed_yaml preserves all top-level key order.
    use std::collections::HashMap;
    use std::path::Path;

    let mut out_parsed_yaml = tpl.clone();

    let local_urls: Vec<_> = if let Some(cfg) = tpl.remove("clashtui") {
        let Config { uses } = serde_yml::from_value(cfg)?;
        name_urls
            .into_iter()
            .filter(|(name, _)| uses.contains(name))
            .map(|(_, url)| url)
            .collect()
    } else {
        Default::default()
    };

    // ## proxy-providers
    // e.g. {provider: [provider0, provider1, ...]}
    let mut pp_names: HashMap<String, Vec<String>> = HashMap::new(); // proxy-provider names
    let mut new_proxy_providers = serde_yml::Mapping::new();
    let pp_mapping = if let Some(serde_yml::Value::Mapping(pp_mapping)) = tpl.get(PROXY_PROVIDERS) {
        pp_mapping
    } else {
        anyhow::bail!("Failed to parse `proxy-providers`");
    };

    for (pp_key, pp_value) in pp_mapping {
        if pp_value.get("tpl_param").is_none() {
            new_proxy_providers.insert(pp_key.clone(), pp_value.clone());
            continue;
        }

        let pp = pp_value
            .as_mapping()
            .ok_or_else(|| anyhow::anyhow!("Failed to parse `proxy-providers` value"))?;

        for (i, url) in local_urls.iter().enumerate() {
            let mut new_pp = pp.clone();
            new_pp.remove("tpl_param");
            // name: e.g. provider0, provider1, ...
            let pp_key_str = pp_key
                .as_str()
                .with_context(|| "Proxy-provider key is not a string")?;
            let the_pp_name = format!("{}{}", pp_key_str, i);
            pp_names
                .entry(pp_key_str.to_string())
                .or_default()
                .push(the_pp_name.clone());

            new_pp.insert(
                serde_yml::Value::String("url".to_string()),
                serde_yml::Value::String(url.clone()),
            );
            let tpl_name_no_ext = Path::new(tpl_name)
                .file_stem()
                .unwrap_or_else(|| Path::new(tpl_name).as_os_str())
                .to_str()
                .unwrap_or(tpl_name);
            new_pp.insert(
                serde_yml::Value::String("path".to_string()),
                serde_yml::Value::String(format!(
                    "proxy-providers/tpl/{}/{}.yaml",
                    tpl_name_no_ext, the_pp_name
                )),
            );
            new_proxy_providers.insert(
                serde_yml::Value::String(the_pp_name.clone()),
                serde_yml::Value::Mapping(new_pp.clone()),
            );
        }
    }
    out_parsed_yaml[PROXY_PROVIDERS] = serde_yml::Value::Mapping(new_proxy_providers);

    // ## proxy-groups
    // e.g. {Auto: [Auto-provider0, Auto-provider1, ...], Select: [Select-provider0, ...]}
    let mut pg_names: HashMap<String, Vec<String>> = HashMap::new();
    let mut new_proxy_groups = serde_yml::Sequence::new();
    let pg_value = if let Some(serde_yml::Value::Sequence(pg_value)) = tpl.get(PROXY_GROUPS) {
        pg_value
    } else {
        anyhow::bail!("Failed to parse `proxy-groups`.");
    };

    for the_pg_value in pg_value {
        if the_pg_value.get("tpl_param").is_none() {
            new_proxy_groups.push(the_pg_value.clone());
            continue;
        }

        let the_pg = if let serde_yml::Value::Mapping(the_pg) = the_pg_value {
            the_pg
        } else {
            anyhow::bail!("Failed to parse `proxy-groups` value");
        };

        let mut new_pg = the_pg.clone();
        new_pg.remove("tpl_param");

        let provider_keys = if let Some(serde_yml::Value::Sequence(provider_keys)) =
            the_pg["tpl_param"].get("providers")
        {
            provider_keys
        } else {
            anyhow::bail!("Failed to parse `providers` in `tpl_param`");
        };

        for the_provider_key in provider_keys {
            let the_pk_str = if let serde_yml::Value::String(the_pk_str) = the_provider_key {
                the_pk_str
            } else {
                anyhow::bail!("Failed to parse string in `providers`")
            };

            let names = if let Some(names) = pp_names.get(the_pk_str) {
                names
            } else {
                continue;
            };

            let the_pg_name =
                if let Some(serde_yml::Value::String(the_pg_name)) = the_pg_value.get("name") {
                    the_pg_name
                } else {
                    anyhow::bail!("Failed to parse `name` in `proxy-groups`");
                };

            for n in names {
                // new_pg_name: e.g. Auto-provider0, Auto-provider1, Select-provider0, ...
                let new_pg_name = format!("{}-{}", the_pg_name, n); // proxy-group
                // names

                pg_names
                    .entry(the_pg_name.clone())
                    .or_default()
                    .push(new_pg_name.clone());

                new_pg["name"] = serde_yml::Value::String(new_pg_name.clone());
                new_pg.insert(
                    serde_yml::Value::String("use".to_string()),
                    serde_yml::Value::Sequence(vec![serde_yml::Value::String(n.clone())]),
                );

                new_proxy_groups.push(serde_yml::Value::Mapping(new_pg.clone()));
            }
        }
    }
    out_parsed_yaml[PROXY_GROUPS] = serde_yml::Value::Sequence(new_proxy_groups);

    // ### replace special keys in group-providers
    // e.g. <provider> => provider0, provider1
    // e.g. <Auto> => Auto-provider0, Auto-provider1
    // e.g. <Select> => Select-provider0, Select-provider1
    //
    // Ordering: this iterates over the pg_sequence in the order it was constructed
    // above, which preserves the original template ordering.
    let pg_sequence = if let Some(serde_yml::Value::Sequence(pg_sequence)) =
        out_parsed_yaml.get_mut(PROXY_GROUPS)
    {
        pg_sequence
    } else {
        anyhow::bail!("Failed to parse `proxy-groups`");
    };

    for the_pg_seq in pg_sequence {
        if let Some(providers) = the_pg_seq.get("use") {
            let prov_seq = providers
                .as_sequence()
                .with_context(|| "`use` field is not a sequence")?;
            let mut new_providers = Vec::new();
            for p in prov_seq {
                let p_str = p
                    .as_str()
                    .with_context(|| "Non-string value in `use` list")?;
                if p_str.starts_with('<') && p_str.ends_with('>') {
                    let trimmed_p_str = p_str.trim_matches(|c| c == '<' || c == '>');
                    let provider_names = pp_names
                        .get(trimmed_p_str)
                        .with_context(|| "Can't find the proxy-provider name.")?;
                    new_providers.extend(provider_names.iter().cloned());
                } else {
                    new_providers.push(p_str.to_string());
                }
            }
            the_pg_seq["use"] = serde_yml::Value::Sequence(
                new_providers
                    .into_iter()
                    .map(serde_yml::Value::String)
                    .collect(),
            );
        }

        if let Some(serde_yml::Value::Sequence(groups)) = the_pg_seq.get("proxies") {
            let mut new_groups = Vec::new();
            for g in groups {
                let g_str = g
                    .as_str()
                    .with_context(|| "Non-string value in `proxies` list")?;
                if g_str.starts_with('<') && g_str.ends_with('>') {
                    let trimmed_g_str = g_str.trim_matches(|c| c == '<' || c == '>');
                    let group_names = pg_names
                        .get(trimmed_g_str)
                        .with_context(|| "Can't find the proxy-group name.")?;
                    new_groups.extend(group_names.iter().cloned());
                } else {
                    new_groups.push(g_str.to_string());
                }
            }
            the_pg_seq["proxies"] = serde_yml::Value::Sequence(
                new_groups
                    .into_iter()
                    .map(serde_yml::Value::String)
                    .collect(),
            );
        }
    }

    // ## add `clashtui` section
    out_parsed_yaml["clashtui"] = serde_yml::Value::Null;

    Ok(out_parsed_yaml)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn testdata_path(name: &str) -> std::path::PathBuf {
        let manifest_dir = std::env!("CARGO_MANIFEST_DIR");
        std::path::PathBuf::from(manifest_dir)
            .join("src/functions/file/template/testdata")
            .join(name)
    }

    fn load_yaml<P: AsRef<std::path::Path>>(
        path: P,
    ) -> anyhow::Result<serde_yml::Mapping> {
        let file = std::fs::File::open(path)?;
        Ok(serde_yml::from_reader(file)?)
    }

    fn two_urls() -> Vec<(String, String)> {
        vec![
            ("sub1".to_string(), "https://example.com/sub1.yaml".to_string()),
            ("sub2".to_string(), "https://example.com/sub2.yaml".to_string()),
        ]
    }

    fn one_url() -> Vec<(String, String)> {
        vec![
            ("sub1".to_string(), "https://example.com/sub1.yaml".to_string()),
        ]
    }

    #[test]
    fn test_simple_expansion() {
        let tpl = load_yaml(testdata_path("simple_tpl.yaml")).unwrap();
        let expected = serde_yml::from_reader::<_, serde_yml::Value>(
            std::fs::File::open(testdata_path("simple_tpl_output.yaml")).unwrap(),
        )
        .unwrap();

        let result = gen_template(tpl, "simple_tpl.yaml", two_urls()).unwrap();
        let result_value = serde_yml::to_value(result).unwrap();

        assert_eq!(result_value, expected);
    }

    #[test]
    fn test_multi_provider_expansion() {
        let tpl = load_yaml(testdata_path("multi_provider_tpl.yaml")).unwrap();
        let expected = serde_yml::from_reader::<_, serde_yml::Value>(
            std::fs::File::open(testdata_path("multi_provider_tpl_output.yaml")).unwrap(),
        )
        .unwrap();

        let result = gen_template(tpl, "multi_provider_tpl.yaml", two_urls()).unwrap();
        let result_value = serde_yml::to_value(result).unwrap();

        assert_eq!(result_value, expected);
    }

    #[test]
    fn test_no_tpl_param_passthrough() {
        let tpl = load_yaml(testdata_path("no_tpl_param_tpl.yaml")).unwrap();
        let expected = serde_yml::from_reader::<_, serde_yml::Value>(
            std::fs::File::open(testdata_path("no_tpl_param_tpl_output.yaml")).unwrap(),
        )
        .unwrap();

        let result = gen_template(tpl, "no_tpl_param_tpl.yaml", two_urls()).unwrap();
        let result_value = serde_yml::to_value(result).unwrap();

        assert_eq!(result_value, expected);
    }

    #[test]
    fn test_empty_uses() {
        let tpl = load_yaml(testdata_path("empty_uses_tpl.yaml")).unwrap();
        let expected = serde_yml::from_reader::<_, serde_yml::Value>(
            std::fs::File::open(testdata_path("empty_uses_tpl_output.yaml")).unwrap(),
        )
        .unwrap();

        let result = gen_template(tpl, "empty_uses_tpl.yaml", vec![]).unwrap();
        let result_value = serde_yml::to_value(result).unwrap();

        assert_eq!(result_value, expected);
    }

    #[test]
    fn test_ordering_preserved_proxy_groups() {
        let tpl = load_yaml(testdata_path("simple_tpl.yaml")).unwrap();
        let result = gen_template(tpl, "simple_tpl.yaml", one_url()).unwrap();

        let groups = result
            .get(PROXY_GROUPS)
            .and_then(|v| v.as_sequence())
            .unwrap();

        // Expected order: Select, Auto-pvd0, Direct
        let names: Vec<&str> = groups
            .iter()
            .filter_map(|g| g.get("name").and_then(|n| n.as_str()))
            .collect();
        assert_eq!(names, vec!["Select", "Auto-pvd0", "Direct"]);
    }

    #[test]
    fn test_ordering_preserved_proxy_providers() {
        let tpl = load_yaml(testdata_path("simple_tpl.yaml")).unwrap();
        let result = gen_template(tpl, "simple_tpl.yaml", one_url()).unwrap();

        let providers = result
            .get(PROXY_PROVIDERS)
            .and_then(|v| v.as_mapping())
            .unwrap();

        // Template entry pvd is expanded IN PLACE at its original position
        // (pvd was first → pvd0 is first), then static (passthrough) follows
        let keys: Vec<&str> = providers.keys().filter_map(|k| k.as_str()).collect();
        assert_eq!(keys, vec!["pvd0", "static"]);
    }

    #[test]
    fn test_angle_bracket_provider_placeholder() {
        let tpl = load_yaml(testdata_path("multi_provider_tpl.yaml")).unwrap();
        let result = gen_template(tpl, "multi_provider_tpl.yaml", two_urls()).unwrap();

        let groups = result
            .get(PROXY_GROUPS)
            .and_then(|v| v.as_sequence())
            .unwrap();

        let all_in_one = groups
            .iter()
            .find(|g| g.get("name").and_then(|n| n.as_str()) == Some("AllInOne"))
            .unwrap();

        let uses: Vec<&str> = all_in_one
            .get("use")
            .and_then(|v| v.as_sequence())
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str())
            .collect();

        assert_eq!(uses, vec!["pvd0", "pvd1", "pvd20", "pvd21"]);
    }

    #[test]
    fn test_angle_bracket_group_placeholder() {
        let tpl = load_yaml(testdata_path("multi_provider_tpl.yaml")).unwrap();
        let result = gen_template(tpl, "multi_provider_tpl.yaml", two_urls()).unwrap();

        let groups = result
            .get(PROXY_GROUPS)
            .and_then(|v| v.as_sequence())
            .unwrap();

        let select = groups
            .iter()
            .find(|g| g.get("name").and_then(|n| n.as_str()) == Some("Select"))
            .unwrap();

        let proxies: Vec<&str> = select
            .get("proxies")
            .and_then(|v| v.as_sequence())
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str())
            .collect();

        assert_eq!(
            proxies,
            vec!["DIRECT", "Auto-pvd0", "Auto-pvd1", "Fallback-pvd20", "Fallback-pvd21"]
        );
    }

    #[test]
    fn test_missing_proxy_providers_section() {
        let tpl = load_yaml(testdata_path("missing_pp_tpl.yaml")).unwrap();
        let result = gen_template(tpl, "missing_pp_tpl.yaml", one_url());
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_proxy_groups_section() {
        let tpl = load_yaml(testdata_path("missing_pg_tpl.yaml")).unwrap();
        let result = gen_template(tpl, "missing_pg_tpl.yaml", one_url());
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_tpl_param_providers_key() {
        let tpl = load_yaml(testdata_path("missing_providers_key_tpl.yaml")).unwrap();
        let result = gen_template(tpl, "missing_providers_key_tpl.yaml", one_url());
        assert!(result.is_err());
    }

    #[test]
    fn test_placeholder_to_nonexistent_target() {
        let tpl = load_yaml(testdata_path("bad_placeholder_tpl.yaml")).unwrap();
        let result = gen_template(tpl, "bad_placeholder_tpl.yaml", one_url());
        assert!(result.is_err());
    }

    #[test]
    fn test_clashtui_marker_added() {
        let tpl = load_yaml(testdata_path("simple_tpl.yaml")).unwrap();
        let result = gen_template(tpl, "simple_tpl.yaml", one_url()).unwrap();

        assert!(result.contains_key("clashtui"));
        assert_eq!(result.get("clashtui"), Some(&serde_yml::Value::Null));
    }

    #[test]
    fn test_path_generation_format() {
        let tpl = load_yaml(testdata_path("simple_tpl.yaml")).unwrap();
        let result = gen_template(tpl, "simple_tpl.yaml", one_url()).unwrap();

        let providers = result
            .get(PROXY_PROVIDERS)
            .and_then(|v| v.as_mapping())
            .unwrap();

        let pvd0 = providers
            .get(&serde_yml::Value::String("pvd0".to_string()))
            .and_then(|v| v.as_mapping())
            .unwrap();

        let path = pvd0
            .get(&serde_yml::Value::String("path".to_string()))
            .and_then(|v| v.as_str())
            .unwrap();

        assert_eq!(path, "proxy-providers/tpl/simple_tpl/pvd0.yaml");
    }
}
