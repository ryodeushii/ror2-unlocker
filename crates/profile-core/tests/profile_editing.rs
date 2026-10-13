#![cfg(feature = "native")]

use ror2_profile_core::{catalog, open_profile, preview_changes, save_changes, ChangeRequest};
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use tempfile::TempDir;

fn fixture(xml: &str) -> (TempDir, PathBuf, ChangeRequest) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("profile.xml");
    fs::write(&path, xml).unwrap();
    let profile = open_profile(&path).unwrap();
    let args = ChangeRequest {
        path: profile.path,
        revision: profile.revision,
        selection: vec![],
        coins: None,
    };
    (directory, path, args)
}

const EMPTY: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><UserProfile><achievementsList/><discoveredPickups/><coins/><stats/></UserProfile>";

#[test]
fn self_closing_fields_expand_and_dependencies_are_deduplicated() {
    let (_dir, path, mut args) = fixture(EMPTY);
    args.selection = vec![
        "unlock:Characters.Croco".into(),
        "achievement:BeatArena".into(),
    ];
    let preview = preview_changes(&args).unwrap();
    assert_eq!(preview.total_additions, 2);
    assert_eq!(preview.changes[0].added, ["BeatArena"]);
    assert_eq!(preview.changes[1].added, ["Characters.Croco"]);
    save_changes(&args).unwrap();
    let loaded = open_profile(&path).unwrap();
    assert!(loaded
        .unlocked_ids
        .contains(&"unlock:Characters.Croco".into()));
    assert!(loaded
        .unlocked_ids
        .contains(&"achievement:BeatArena".into()));
}

#[test]
fn empty_normal_fields_and_whitespace_coins_are_supported() {
    let xml = "<UserProfile><achievementsList></achievementsList><discoveredPickups> </discoveredPickups><coins> \n </coins><stats>\n</stats></UserProfile>";
    let (_dir, path, mut args) = fixture(xml);
    args.coins = Some(2147483647);
    assert_eq!(save_changes(&args).unwrap().profile.coins, 2147483647);
    assert_eq!(
        fs::read_to_string(path).unwrap(),
        xml.replace("<coins> \n </coins>", "<coins>2147483647</coins>")
    );
}

#[test]
fn unrelated_escaped_fields_and_unknown_tokens_are_preserved_byte_for_byte() {
    let xml = "<UserProfile><!-- keep --><controller>&lt;data value=&quot;x&amp;y&quot;/&gt;</controller><achievementsList>FutureAchievement</achievementsList><discoveredPickups>UnknownPickup</discoveredPickups><coins>7</coins><stats><stat name=\"career\">123</stat><unlock>Future.Unlock</unlock><!--inside--></stats><future a='&amp;'>raw</future></UserProfile>";
    let (_dir, path, mut args) = fixture(xml);
    args.selection = vec!["unlock:Characters.Croco".into()];
    save_changes(&args).unwrap();
    let expected = xml
        .replace(
            "FutureAchievement</achievementsList>",
            "FutureAchievement BeatArena</achievementsList>",
        )
        .replace(
            "<!--inside--></stats>",
            "<!--inside--><unlock>Characters.Croco</unlock></stats>",
        );
    assert_eq!(fs::read_to_string(path).unwrap(), expected);
}

#[test]
fn duplicate_important_fields_are_rejected() {
    for field in ["achievementsList", "discoveredPickups", "coins", "stats"] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.xml");
        fs::write(
            &path,
            EMPTY.replace("</UserProfile>", &format!("<{field}/></UserProfile>")),
        )
        .unwrap();
        assert!(open_profile(&path).is_err());
    }
}

#[test]
fn invalid_xml_dtd_encodings_and_nested_tokens_are_rejected() {
    for xml in [
        "<UserProfile>",
        "<!DOCTYPE UserProfile [<!ENTITY injected 'x'>]><UserProfile/>",
        "<?xml version='1.0' encoding='ISO-8859-1'?><UserProfile><achievementsList/><discoveredPickups/><coins/><stats/></UserProfile>",
        "<Other><achievementsList/><discoveredPickups/><coins/><stats/></Other>",
        "<UserProfile><achievementsList><child/></achievementsList><discoveredPickups/><coins/><stats/></UserProfile>",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.xml");
        fs::write(&path, xml).unwrap();
        assert!(open_profile(&path).is_err(), "accepted {xml}");
    }
}

#[test]
fn coin_limits_are_checked_without_clamping() {
    let (_dir, path, mut args) = fixture(EMPTY);
    args.coins = Some(2147483648);
    assert!(preview_changes(&args).is_err());
    assert!(save_changes(&args).is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), EMPTY);
    args.coins = Some(2147483647);
    let saved = save_changes(&args).unwrap();
    args.revision = saved.profile.revision;
    args.coins = Some(0);
    assert_eq!(save_changes(&args).unwrap().profile.coins, 0);
    for balance in ["-1", "2147483648", "1.5"] {
        fs::write(
            &path,
            EMPTY.replace("<coins/>", &format!("<coins>{balance}</coins>")),
        )
        .unwrap();
        assert!(open_profile(&path).is_err());
    }
}

#[test]
fn populated_item_dependencies_are_granted_with_discovery() {
    let item = catalog()
        .entries
        .iter()
        .find(|entry| {
            entry.kind == "items" && entry.achievement.is_some() && entry.unlock.is_some()
        })
        .unwrap();
    let (_dir, _path, mut args) = fixture(EMPTY);
    args.selection = vec![item.id.clone()];
    let preview = preview_changes(&args).unwrap();
    assert_eq!(preview.total_additions, 3);
    let saved = save_changes(&args).unwrap();
    assert!(saved.profile.unlocked_ids.contains(&item.id));
}

#[test]
fn partial_state_is_not_reported_as_complete() {
    let xml = EMPTY.replace(
        "<achievementsList/>",
        "<achievementsList>BeatArena</achievementsList>",
    );
    let (_dir, path, mut args) = fixture(&xml);
    assert!(!open_profile(&path)
        .unwrap()
        .unlocked_ids
        .contains(&"unlock:Characters.Croco".into()));
    args.selection = vec!["unlock:Characters.Croco".into()];
    assert_eq!(preview_changes(&args).unwrap().total_additions, 1);
}

#[test]
fn stale_preview_and_save_leave_external_changes_untouched() {
    let (_dir, path, mut args) = fixture(EMPTY);
    args.coins = Some(9);
    let external = EMPTY.replace("<coins/>", "<coins>2</coins>");
    fs::write(&path, &external).unwrap();
    assert!(preview_changes(&args).is_err());
    assert!(save_changes(&args).is_err());
    assert_eq!(fs::read_to_string(path).unwrap(), external);
}

#[test]
fn backup_is_exact_original_and_successive_backups_are_unique() {
    let (_dir, path, mut args) = fixture(EMPTY);
    args.coins = Some(10);
    let first = save_changes(&args).unwrap();
    let first_backup = first.backup_path.unwrap();
    assert_eq!(fs::read(&first_backup).unwrap(), EMPTY.as_bytes());
    let intermediate = fs::read(&path).unwrap();
    args.revision = first.profile.revision;
    args.coins = Some(20);
    let second = save_changes(&args).unwrap();
    let second_backup = second.backup_path.unwrap();
    assert_ne!(first_backup, second_backup);
    assert_eq!(fs::read(second_backup).unwrap(), intermediate);
    assert_eq!(fs::read(first_backup).unwrap(), EMPTY.as_bytes());
}

#[test]
fn noop_never_writes_or_creates_backup() {
    let (dir, path, args) = fixture(EMPTY);
    let saved = save_changes(&args).unwrap();
    assert!(saved.backup_path.is_none());
    assert_eq!(saved.preview.total_additions, 0);
    assert_eq!(fs::read_to_string(path).unwrap(), EMPTY);
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn arbitrary_client_xml_identifiers_are_rejected() {
    let (_dir, path, mut args) = fixture(EMPTY);
    args.selection = vec!["Characters.Invented</unlock><injected/>".into()];
    assert!(save_changes(&args).is_err());
    assert_eq!(fs::read_to_string(path).unwrap(), EMPTY);
}

#[cfg(unix)]
#[test]
fn replacement_preserves_permissions() {
    use std::os::unix::fs::PermissionsExt;
    let (_dir, path, mut args) = fixture(EMPTY);
    fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
    args.coins = Some(1);
    save_changes(&args).unwrap();
    assert_eq!(
        fs::metadata(path).unwrap().permissions().mode() & 0o777,
        0o640
    );
}

#[test]
fn oversized_and_non_utf8_profiles_are_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.xml");
    let mut file = fs::File::create(&path).unwrap();
    file.write_all(EMPTY.as_bytes()).unwrap();
    file.write_all(b"<!--").unwrap();
    std::io::copy(&mut std::io::repeat(b' ').take(32 * 1024 * 1024), &mut file).unwrap();
    file.write_all(b"-->").unwrap();
    drop(file);
    assert!(open_profile(&path).is_err());
    fs::write(&path, [0xff, 0xfe, b'<', b'>']).unwrap();
    assert!(open_profile(&path).is_err());
}
