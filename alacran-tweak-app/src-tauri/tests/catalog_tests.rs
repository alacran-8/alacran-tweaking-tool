use alacran_tweak_lib::catalog::*;

fn text(s: &str) -> Text {
    Text { tr: s.into(), en: s.into() }
}

fn reg_op() -> Op {
    Op::RegSet {
        hive: Hive::Hklm,
        path: "SOFTWARE\\X".into(),
        name: "V".into(),
        data: RegData::Dword(1),
    }
}

fn tweak(id: &str) -> Tweak {
    Tweak {
        id: id.into(),
        category: "game".into(),
        title: text("t"),
        description: text("d"),
        risk: Risk::Safe,
        in_one_click: false,
        new: false,
        reboot: Reboot::None,
        one_way: false,
        min_build: 19044,
        max_build: None,
        vendor: None,
        laptop: None,
        caution_note: None,
        ops: vec![reg_op()],
        options: vec![],
        default_option: None,
        auto_option: None,
    }
}

fn errs_for(t: Vec<Tweak>) -> Vec<String> {
    validate(&Catalog { tweaks: t })
}

#[test]
fn embedded_catalog_is_valid() {
    let c = load_embedded();
    assert!(!c.tweaks.is_empty());
    let e = validate(&c);
    assert!(e.is_empty(), "{:?}", e);
}

#[test]
fn caution_never_in_one_click() {
    let mut t = tweak("bad_caution");
    t.risk = Risk::Caution;
    t.in_one_click = true;
    let e = errs_for(vec![t]);
    assert!(e.iter().any(|m| m.contains("caution") && m.contains("bad_caution")), "{:?}", e);
}

#[test]
fn duplicate_ids_rejected() {
    let e = errs_for(vec![tweak("a"), tweak("a")]);
    assert!(e.iter().any(|m| m.contains("duplicate")), "{:?}", e);
}

#[test]
fn missing_translation_rejected() {
    let mut t = tweak("a");
    t.title.en = String::new();
    let e = errs_for(vec![t]);
    assert!(e.iter().any(|m| m.contains("translation")), "{:?}", e);
}

#[test]
fn one_way_required_for_appx_and_cleanup() {
    let mut t = tweak("a");
    t.ops = vec![Op::AppxRemove { pattern: "*Foo*".into() }];
    assert!(errs_for(vec![t.clone()]).iter().any(|m| m.contains("one_way")));
    t.ops = vec![Op::Cleanup { target: CleanupTarget::UserTemp }];
    assert!(errs_for(vec![t.clone()]).iter().any(|m| m.contains("one_way")));
    t.one_way = true;
    assert!(errs_for(vec![t]).is_empty());
}

#[test]
fn options_default_must_exist() {
    let mut t = tweak("a");
    t.ops = vec![];
    t.options = vec![TweakOption { id: "x".into(), label: text("x"), ops: vec![reg_op()] }];
    t.default_option = Some("missing".into());
    let e = errs_for(vec![t.clone()]);
    assert!(e.iter().any(|m| m.contains("default_option")), "{:?}", e);
    t.default_option = Some("x".into());
    assert!(errs_for(vec![t]).is_empty());
}

#[test]
fn unknown_category_rejected() {
    let mut t = tweak("a");
    t.category = "bogus".into();
    let e = errs_for(vec![t]);
    assert!(e.iter().any(|m| m.contains("category")), "{:?}", e);
}

#[test]
fn min_build_and_empty_path_and_hex_rejected() {
    let mut t = tweak("a");
    t.min_build = 18000;
    assert!(errs_for(vec![t]).iter().any(|m| m.contains("min_build")));

    let mut t = tweak("b");
    t.ops = vec![Op::RegSet {
        hive: Hive::Hkcu,
        path: " ".into(),
        name: "V".into(),
        data: RegData::Binary("abc".into()),
    }];
    let e = errs_for(vec![t]);
    assert!(e.iter().any(|m| m.contains("empty path")), "{:?}", e);
    assert!(e.iter().any(|m| m.contains("hex")), "{:?}", e);
}

#[test]
fn serde_shapes() {
    let op: Op = serde_json::from_str(
        r#"{"op":"reg_set","hive":"hklm","path":"P","name":"N","data":{"type":"dword","value":1}}"#,
    )
    .unwrap();
    assert_eq!(op, reg_op_with("P", "N"));
}

fn reg_op_with(path: &str, name: &str) -> Op {
    Op::RegSet { hive: Hive::Hklm, path: path.into(), name: name.into(), data: RegData::Dword(1) }
}

#[test]
fn unknown_fields_rejected_in_tweak_text_and_option() {
    let mut v = serde_json::to_value(tweak("a")).unwrap();
    v["laptpo"] = serde_json::json!(false);
    assert!(serde_json::from_value::<Tweak>(v).is_err(), "typo in a tweak field");

    let mut v = serde_json::to_value(tweak("a")).unwrap();
    v["title"]["de"] = serde_json::json!("x");
    assert!(serde_json::from_value::<Tweak>(v).is_err(), "typo in a text field");

    let mut t = tweak("a");
    t.options = vec![TweakOption { id: "o".into(), label: text("l"), ops: vec![] }];
    let mut v = serde_json::to_value(t).unwrap();
    v["options"][0]["opts"] = serde_json::json!([]);
    assert!(serde_json::from_value::<Tweak>(v).is_err(), "typo in an option field");
}

#[test]
fn every_category_has_at_least_one_one_click_tweak_except_privacy() {
    let c = load_embedded();
    for cat in CATEGORIES.iter().filter(|c| **c != "privacy") {
        assert!(
            c.tweaks.iter().any(|t| t.category == *cat && t.in_one_click),
            "category '{cat}' has no one-click tweak"
        );
    }
}
