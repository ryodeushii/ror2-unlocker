use ror2_profile_core::{
    edit_profile_bytes, inspect_profile_bytes, preview_profile_bytes, ChangeRequest,
};

const EMPTY: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><UserProfile><achievementsList/><discoveredPickups/><coins/><stats/></UserProfile>";

fn request(xml: &str) -> ChangeRequest {
    let profile = inspect_profile_bytes("uploaded.xml", xml.as_bytes().to_vec()).unwrap();
    ChangeRequest {
        path: profile.path,
        revision: profile.revision,
        selection: vec![],
        coins: None,
    }
}

#[test]
fn labels_are_not_filesystem_paths_and_noop_preserves_original_bytes() {
    let xml = format!(
        "\u{feff}{}\r\n",
        EMPTY.replace("<coins/>", "<coins>0007</coins>")
    );
    let label = "nonexistent/../uploaded.xml";
    let original = inspect_profile_bytes(label, xml.as_bytes().to_vec()).unwrap();
    assert_eq!(original.path, label);
    assert_eq!(original.name, label);
    let args = ChangeRequest {
        path: label.into(),
        revision: original.revision.clone(),
        selection: vec![],
        coins: Some(7),
    };
    let edited = edit_profile_bytes(xml.as_bytes().to_vec(), &args).unwrap();
    assert_eq!(edited.xml.as_bytes(), xml.as_bytes());
    assert_eq!(edited.profile.revision, original.revision);
    assert_eq!(edited.profile.path, label);
    assert_eq!(edited.profile.name, label);
    assert_eq!(edited.preview.total_additions, 0);
    assert_eq!(edited.preview.coins_before, edited.preview.coins_after);
}

#[test]
fn editing_reports_actual_dependencies_and_preserves_unrelated_xml() {
    let xml = "<UserProfile>\r\n<!-- keep --><future a='&amp;'>&lt;data/&gt;</future><achievementsList>FutureAchievement</achievementsList><discoveredPickups>UnknownPickup</discoveredPickups><coins>7</coins><stats><stat name='career'>123</stat><unlock>Future.Unlock</unlock><!--inside--></stats></UserProfile>";
    let mut args = request(xml);
    args.selection = vec![
        "unlock:Characters.Croco".into(),
        "achievement:BeatArena".into(),
    ];
    args.coins = Some(2147483647);
    let preview = preview_profile_bytes(xml.as_bytes().to_vec(), &args).unwrap();
    let edited = edit_profile_bytes(xml.as_bytes().to_vec(), &args).unwrap();
    assert_eq!(edited.preview, preview);
    assert_eq!(preview.total_additions, 2);
    let expected = xml
        .replace(
            "FutureAchievement</achievementsList>",
            "FutureAchievement BeatArena</achievementsList>",
        )
        .replace(
            "<!--inside--></stats>",
            "<!--inside--><unlock>Characters.Croco</unlock></stats>",
        )
        .replace("<coins>7</coins>", "<coins>2147483647</coins>");
    assert_eq!(edited.xml, expected);
    assert_eq!(edited.profile.coins, 2147483647);
    assert!(edited
        .profile
        .unlocked_ids
        .contains(&"unlock:Characters.Croco".into()));
    assert_ne!(edited.profile.revision, args.revision);
    let inspected = inspect_profile_bytes("uploaded.xml", edited.xml.into_bytes()).unwrap();
    assert_eq!(edited.profile.revision, inspected.revision);
    assert_eq!(edited.profile.unlocked_ids, inspected.unlocked_ids);
}

#[test]
fn changed_source_bytes_reject_stale_preview_and_edit() {
    let args = request(EMPTY);
    let changed = EMPTY.replace("<coins/>", "<coins>2</coins>");
    assert!(preview_profile_bytes(changed.as_bytes().to_vec(), &args).is_err());
    assert!(edit_profile_bytes(changed.into_bytes(), &args).is_err());
}

#[test]
fn invalid_coins_and_unknown_selections_are_rejected_by_preview_and_edit() {
    let mut args = request(EMPTY);
    args.coins = Some(2147483648);
    assert!(preview_profile_bytes(EMPTY.as_bytes().to_vec(), &args).is_err());
    assert!(edit_profile_bytes(EMPTY.as_bytes().to_vec(), &args).is_err());
    args.coins = None;
    args.selection = vec!["unknown:selection".into()];
    assert!(preview_profile_bytes(EMPTY.as_bytes().to_vec(), &args).is_err());
    assert!(edit_profile_bytes(EMPTY.as_bytes().to_vec(), &args).is_err());
    for coins in ["-1", "1.5", "2147483648"] {
        let xml = EMPTY.replace("<coins/>", &format!("<coins>{coins}</coins>"));
        assert!(inspect_profile_bytes("invalid.xml", xml.into_bytes()).is_err());
    }
}

#[test]
fn all_byte_operations_reject_invalid_utf8_and_schema() {
    let args = request(EMPTY);
    let malformed = [
        vec![0xff, 0xfe],
        b"<UserProfile>".to_vec(),
        EMPTY
            .replace("</UserProfile>", "<coins/></UserProfile>")
            .into_bytes(),
        EMPTY.replace("UTF-8", "ISO-8859-1").into_bytes(),
        EMPTY
            .replace(
                "<achievementsList/>",
                "<achievementsList><child/></achievementsList>",
            )
            .into_bytes(),
        format!("<!DOCTYPE UserProfile [<!ENTITY injected 'x'>]>{EMPTY}").into_bytes(),
    ];
    for bytes in malformed {
        assert!(inspect_profile_bytes("invalid.xml", bytes.clone()).is_err());
        assert!(preview_profile_bytes(bytes.clone(), &args).is_err());
        assert!(edit_profile_bytes(bytes, &args).is_err());
    }
}

#[test]
fn input_and_rendered_output_enforce_the_same_size_limit() {
    const LIMIT: usize = 32 * 1024 * 1024;
    let mut bytes = EMPTY.as_bytes().to_vec();
    bytes.resize(LIMIT, b' ');
    let original = inspect_profile_bytes("large.xml", bytes.clone()).unwrap();
    let mut args = ChangeRequest {
        path: original.path,
        revision: original.revision,
        selection: vec![],
        coins: None,
    };
    assert_eq!(
        edit_profile_bytes(bytes.clone(), &args).unwrap().xml.len(),
        LIMIT
    );
    args.coins = Some(1);
    assert!(edit_profile_bytes(bytes.clone(), &args).is_err());
    bytes.push(b' ');
    assert!(inspect_profile_bytes("large.xml", bytes.clone()).is_err());
    assert!(preview_profile_bytes(bytes.clone(), &args).is_err());
    assert!(edit_profile_bytes(bytes, &args).is_err());
}
