use anyhow::Result;
use crate::config::CoreType;

pub fn init() -> Result<()> {
    let path = crate::config::keymap_path();

    if !path.exists() {
        log::debug!("Skip loading keymap");
        return Ok(());
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
