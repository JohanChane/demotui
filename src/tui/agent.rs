use anyhow::Result;
use crate::config::CoreType;
use std::collections::HashMap;

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum KeyValue<K> {
    Simple(K),
    WithDesc { action: K, desc: String },
}

pub fn extract_keymap_with_descs<K: serde::de::DeserializeOwned>(
    map: serde_yml::Mapping,
) -> Result<(HashMap<crate::tui::Key, K>, HashMap<crate::tui::Key, String>)> {
    let mut agent = HashMap::new();
    let mut descs = HashMap::new();
    for (key_val, value_val) in map {
        let key: crate::tui::Key = serde_yml::from_value(key_val)?;
        match serde_yml::from_value::<KeyValue<K>>(value_val) {
            Ok(KeyValue::Simple(action)) => {
                agent.insert(key, action);
            }
            Ok(KeyValue::WithDesc { action, desc }) => {
                agent.insert(key, action);
                descs.insert(key, desc);
            }
            Err(e) => return Err(e.into()),
        }
    }
    Ok((agent, descs))
}

pub fn init() -> Result<()> {
    let path = crate::config::keymap_path();

    if !path.exists() {
        generate_default_keymap(&path)?;
    }

    let file = std::fs::File::open(path)?;
    let mut value: serde_yml::Mapping = serde_yml::from_reader(file)?;

    let (mut common, core_specific) = split_sections(&mut value);

    if let Some(mut core_map) = core_specific {
        merge_mappings(&mut common, &mut core_map);
    }

    super::tab::prelude::agent_init(&mut common)?;

    Ok(())
}

fn generate_default_keymap(path: &std::path::Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, DEFAULT_KEYMAP_YAML)?;
    Ok(())
}

const DEFAULT_KEYMAP_YAML: &str = include_str!("keymap_default.yaml");

fn split_sections(
    value: &mut serde_yml::Mapping,
) -> (serde_yml::Mapping, Option<serde_yml::Mapping>) {
    let mihomo = take_mapping(value, "mihomo");
    let singbox = take_mapping(value, "sing-box");

    let core_type = crate::config::CONFIG.core_type();
    let core_specific = match core_type {
        CoreType::Mihomo => mihomo,
        CoreType::Singbox => singbox,
    };

    (value.clone(), core_specific)
}

fn take_mapping(value: &mut serde_yml::Mapping, key: &str) -> Option<serde_yml::Mapping> {
    let entry = value.remove(key)?;
    match entry {
        serde_yml::Value::Mapping(m) => Some(m),
        _ => None,
    }
}

fn merge_mappings(base: &mut serde_yml::Mapping, override_map: &mut serde_yml::Mapping) {
    for (key, val) in override_map.iter() {
        if let Some(serde_yml::Value::Mapping(base_map)) = base.get_mut(key) {
            if let serde_yml::Value::Mapping(override_inner) = val {
                merge_mappings(base_map, &mut override_inner.clone());
                continue;
            }
        }
        base.insert(key.clone(), val.clone());
    }
}

pub fn get(value: &mut serde_yml::Mapping, idx: &str) -> Result<serde_yml::Mapping> {
    let Some(maybe_map) = value.remove(idx) else {
        anyhow::bail!("Does not contain `{idx}` section")
    };
    let serde_yml::Value::Mapping(map) = maybe_map else {
        anyhow::bail!("Section `{idx}` is not mapping")
    };
    Ok(map)
}

pub fn check_duplicate_keys(section: &str, map: &serde_yml::Mapping) {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    for key in map.keys() {
        if let Ok(k) = serde_yml::from_value::<crate::tui::Key>(key.clone()) {
            if !seen.insert(k) {
                log::warn!("duplicate key `{k}` in [{section}] keymap — later binding overwrites earlier");
            }
        }
    }
}

#[test]
fn example() -> anyhow::Result<()> {
    use std::collections::HashMap;
    use crate::tui::Key;

    #[derive(serde::Deserialize, Debug)]
    enum K {
        Select,
    }

    let str = r#"
file:
  profile:
    ? code: Enter
      shift: false
      ctrl: false
      alt: false
      super_: false
    : Select
"#;
    let value =
        serde_yml::from_str::<serde_yml::Mapping>(str)?["file"]["profile"].clone();
    let keymap: HashMap<Key, K> = serde_yml::from_value(value)?;
    println!("{:?}", keymap);
    assert!(matches!(
        keymap.get(&Key { code: crossterm::event::KeyCode::Enter, shift: false, ctrl: false, alt: false, super_: false }),
        Some(K::Select)
    ));
    Ok(())
}

#[test]
fn test_section_merge_core_overrides_common() {
    let yaml = r#"
connections:
  ? code: Char('j')
    shift: false
    ctrl: false
    alt: false
    super_: false
  : MoveDown
mihomo:
  connections:
    ? code: Char('k')
      shift: false
      ctrl: false
      alt: false
      super_: false
    : MoveUp
"#;
    let mut value: serde_yml::Mapping = serde_yml::from_str(yaml).unwrap();

    // Simulate mihomo being the active core
    let mut common = value.clone();
    let mihomo_section = take_mapping(&mut common, "mihomo");
    // Remove sing-box section too
    common.remove("sing-box");

    assert!(mihomo_section.is_some(), "mihomo section should be extracted");

    if let Some(mut core_specific) = mihomo_section {
        merge_mappings(&mut common, &mut core_specific);
    }

    // After merge, common should have connections from mihomo
    let connections = common.get("connections").expect("connections should exist");
    assert!(connections.is_mapping(), "connections should be a mapping");
}

#[test]
fn test_no_keymap_wrapper_needed() {
    let yaml = r#"
connections:
  ? code: Char('j')
    shift: false
    ctrl: false
    alt: false
    super_: false
  : MoveDown
"#;
    let mut value: serde_yml::Mapping = serde_yml::from_str(yaml).unwrap();
    // Top-level directly has "connections" - no "keymap" wrapper needed
    assert!(value.contains_key("connections"));
    assert!(!value.contains_key("keymap"));
}

#[test]
fn test_take_mapping_removes_key() {
    let yaml = r#"
mihomo:
  foo: bar
common:
  baz: qux
"#;
    let mut value: serde_yml::Mapping = serde_yml::from_str(yaml).unwrap();
    let mihomo = take_mapping(&mut value, "mihomo");
    assert!(mihomo.is_some());
    assert!(!value.contains_key("mihomo"), "mihomo should be removed");
    assert!(value.contains_key("common"), "common should remain");
}

#[test]
fn test_profile_key_deserialization_string_variants() -> anyhow::Result<()> {
    use std::collections::HashMap;
    use crate::tui::Key as TuiKey;
    use crate::tui::tab::files::profile::Key;

    let yaml = r#"
? code: Enter
  shift: false
  ctrl: false
  alt: false
  super_: false
: Select
? code: Up
  shift: false
  ctrl: false
  alt: false
  super_: false
: MoveUp
? code: Down
  shift: false
  ctrl: false
  alt: false
  super_: false
: MoveDown
"#;
    let value: serde_yml::Mapping = serde_yml::from_str(yaml)?;
    let keymap: HashMap<TuiKey, Key> =
        serde_yml::from_value(serde_yml::Value::Mapping(value))?;
    assert_eq!(keymap.len(), 3);
    Ok(())
}

#[test]
fn test_profile_key_with_action_mapping_no_crash() -> anyhow::Result<()> {
    use std::collections::HashMap;
    use crate::tui::Key as TuiKey;
    use crate::tui::tab::files::profile::Key;

    let yaml = r#"
? code: !Char e
  shift: false
  ctrl: false
  alt: false
  super_: false
: Action: Edit
? code: !Char i
  shift: false
  ctrl: false
  alt: false
  super_: false
: Action: Add
"#;
    let value: serde_yml::Mapping = serde_yml::from_str(yaml)?;
    let keymap: HashMap<TuiKey, Key> =
        serde_yml::from_value(serde_yml::Value::Mapping(value))?;
    assert_eq!(keymap.len(), 2);
    let e_key = TuiKey {
        code: crossterm::event::KeyCode::Char('e'),
        shift: false,
        ctrl: false,
        alt: false,
        super_: false,
    };
    let i_key = TuiKey {
        code: crossterm::event::KeyCode::Char('i'),
        shift: false,
        ctrl: false,
        alt: false,
        super_: false,
    };
    assert!(matches!(keymap.get(&e_key), Some(Key::Action(_))));
    assert!(matches!(keymap.get(&i_key), Some(Key::Action(_))));
    Ok(())
}

#[test]
fn test_template_key_deserialization() -> anyhow::Result<()> {
    use std::collections::HashMap;
    use crate::tui::Key as TuiKey;
    use crate::tui::tab::files::template::Key;

    let yaml = r#"
? code: Enter
  shift: false
  ctrl: false
  alt: false
  super_: false
: Action: Generate
? code: Left
  shift: false
  ctrl: false
  alt: false
  super_: false
: Switch
"#;
    let value: serde_yml::Mapping = serde_yml::from_str(yaml)?;
    let keymap: HashMap<TuiKey, Key> =
        serde_yml::from_value(serde_yml::Value::Mapping(value))?;
    assert_eq!(keymap.len(), 2);
    let enter_key = TuiKey {
        code: crossterm::event::KeyCode::Enter,
        shift: false,
        ctrl: false,
        alt: false,
        super_: false,
    };
    assert!(matches!(keymap.get(&enter_key), Some(Key::Action(_))));
    Ok(())
}

#[test]
fn test_default_keymap_parses_as_empty_mapping() -> anyhow::Result<()> {
    let value: serde_yml::Mapping = serde_yml::from_str(DEFAULT_KEYMAP_YAML)?;
    assert!(value.is_empty(), "default keymap should be empty (comments + {{}})");
    Ok(())
}

#[test]
fn test_no_duplicate_keys_in_default_agents() {
    use std::collections::HashSet;
    let mut violations = Vec::new();

    macro_rules! check {
        ($name:expr, $agent:expr) => {{
            let agent = $agent;
            let mut seen = HashSet::new();
            for key in agent.keys() {
                if !seen.insert(*key) {
                    violations.push(format!("{}: duplicate key `{key}`", $name));
                }
            }
        }};
    }

    check!("connections", crate::tui::tab::connections::agent());
    check!("file/profile", crate::tui::tab::files::profile::agent());
    check!("file/template", crate::tui::tab::files::template::agent());
    check!("srvctl", crate::tui::tab::srvctl::agent());
    check!("settings", crate::tui::tab::settings::agent());
    check!("logs", crate::tui::tab::logs::agent());

    if !violations.is_empty() {
        panic!("duplicate keys in default agents:\n{}", violations.join("\n"));
    }
}

#[test]
fn test_default_agents_are_populated() {
    use std::collections::HashMap;
    use crate::tui::Key as TuiKey;

    // Verify default agents from mod_agent! are non-empty (no YAML dependency)
    assert!(!crate::tui::tab::connections::agent().is_empty());
    assert!(!crate::tui::tab::files::profile::agent().is_empty());
    assert!(!crate::tui::tab::files::template::agent().is_empty());
    assert!(!crate::tui::tab::srvctl::agent().is_empty());
    assert!(!crate::tui::tab::settings::agent().is_empty());
    assert!(!crate::tui::tab::logs::agent().is_empty());
}

#[test]
fn test_empty_keymap_skips_all_sections() -> anyhow::Result<()> {
    let mut value: serde_yml::Mapping = serde_yml::from_str(DEFAULT_KEYMAP_YAML)?;
    // Empty keymap: get() should return Err for all sections
    assert!(get(&mut value.clone(), "connections").is_err());
    assert!(get(&mut value.clone(), "srvctl").is_err());
    assert!(get(&mut value.clone(), "settings").is_err());
    assert!(get(&mut value.clone(), "logs").is_err());
    assert!(get(&mut value.clone(), "file").is_err());
    Ok(())
}

#[test]
fn test_keyvalue_deserialization_simple() -> anyhow::Result<()> {
    use crate::tui::Key as TuiKey;

    let yaml = r#"
connections:
  ? code: Enter
    shift: false
    ctrl: false
    alt: false
    super_: false
  : MoveUp
  ? code: !Char k
    shift: false
    ctrl: false
    alt: false
    super_: false
  : MoveDown
"#;
    let value: serde_yml::Mapping = serde_yml::from_str(yaml)?;

    if let Ok(map) = get(&mut value.clone(), "connections") {
        let (keys, _descs) = extract_keymap_with_descs::<crate::tui::tab::connections::Key>(map)?;
        assert_eq!(keys.len(), 2);
        assert!(_descs.is_empty());
    }

    Ok(())
}

#[test]
fn test_keyvalue_deserialization_with_desc() -> anyhow::Result<()> {
    use crate::tui::Key as TuiKey;

    let yaml = r#"
connections:
  ? code: Enter
    shift: false
    ctrl: false
    alt: false
    super_: false
  :
    action: MoveUp
    desc: Move cursor up
  ? code: !Char k
    shift: false
    ctrl: false
    alt: false
    super_: false
  :
    action: MoveDown
    desc: Move cursor down
"#;
    let value: serde_yml::Mapping = serde_yml::from_str(yaml)?;

    if let Ok(map) = get(&mut value.clone(), "connections") {
        let (keys, descs) = extract_keymap_with_descs::<crate::tui::tab::connections::Key>(map)?;
        assert_eq!(keys.len(), 2);
        assert_eq!(descs.len(), 2);
        let mut desc_vals: Vec<_> = descs.values().collect();
        desc_vals.sort();
        assert_eq!(desc_vals, vec!["Move cursor down", "Move cursor up"]);
    }

    Ok(())
}

#[test]
fn test_keyvalue_deserialization_mixed_simple_and_desc() -> anyhow::Result<()> {
    use crate::tui::Key as TuiKey;

    let yaml = r#"
connections:
  ? code: Enter
    shift: false
    ctrl: false
    alt: false
    super_: false
  : MoveUp
  ? code: !Char k
    shift: false
    ctrl: false
    alt: false
    super_: false
  :
    action: MoveDown
    desc: Move down
"#;
    let value: serde_yml::Mapping = serde_yml::from_str(yaml)?;

    if let Ok(map) = get(&mut value.clone(), "connections") {
        let (keys, descs) = extract_keymap_with_descs::<crate::tui::tab::connections::Key>(map)?;
        assert_eq!(keys.len(), 2);
        assert_eq!(descs.len(), 1);
        assert_eq!(descs.values().next().unwrap(), "Move down");
    }

    Ok(())
}
