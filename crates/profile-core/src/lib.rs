use roxmltree::{Document, Node};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
#[cfg(feature = "native")]
use std::fs::{self, File, OpenOptions};
#[cfg(feature = "native")]
use std::io::{Read, Write};
use std::ops::Range;
#[cfg(feature = "native")]
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
#[cfg(feature = "native")]
use std::sync::Mutex;
#[cfg(feature = "native")]
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_BYTES: u64 = 32 * 1024 * 1024;
const MAX_COINS: u64 = i32::MAX as u64;
#[cfg(feature = "native")]
static SAVE_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog {
    pub version: String,
    pub source: String,
    pub entries: Vec<Entry>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub description: String,
    pub achievement: Option<String>,
    pub unlock: Option<String>,
    pub discovery: Option<String>,
    pub expansion: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSnapshot {
    pub path: String,
    pub name: String,
    pub coins: u64,
    pub revision: String,
    pub unlocked_ids: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeRequest {
    pub path: String,
    pub revision: String,
    pub selection: Vec<String>,
    pub coins: Option<u64>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FieldChange {
    pub field: String,
    pub added: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Preview {
    pub changes: Vec<FieldChange>,
    pub coins_before: u64,
    pub coins_after: u64,
    pub total_additions: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveResult {
    pub profile: ProfileSnapshot,
    pub backup_path: Option<String>,
    pub preview: Preview,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditedProfile {
    pub profile: ProfileSnapshot,
    pub preview: Preview,
    pub xml: String,
}

static CATALOG: LazyLock<Catalog> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../../catalog.json"))
        .expect("bundled catalog must be valid")
});

static CATALOG_BY_ID: LazyLock<HashMap<&'static str, &'static Entry>> = LazyLock::new(|| {
    catalog()
        .entries
        .iter()
        .map(|entry| (entry.id.as_str(), entry))
        .collect()
});

pub fn catalog() -> &'static Catalog {
    &CATALOG
}

struct Field {
    whole: Range<usize>,
    content: Range<usize>,
    self_closing: bool,
    opening_end: usize,
}
struct Profile {
    xml: String,
    achievements: HashSet<String>,
    discoveries: HashSet<String>,
    unlocks: HashSet<String>,
    coins: u64,
    fields: [Field; 4],
}

fn field(node: Node<'_, '_>, xml: &str) -> Result<Field, String> {
    let whole = node.range();
    let bytes = xml.as_bytes();
    let mut quote = None;
    let mut end = whole.start;
    while end < whole.end {
        match (quote, bytes[end]) {
            (Some(q), b) if q == b => quote = None,
            (None, b'\'' | b'"') => quote = Some(bytes[end]),
            (None, b'>') => break,
            _ => {}
        }
        end += 1;
    }
    let self_closing = end > whole.start && bytes[end - 1] == b'/';
    let content_end = if self_closing {
        end + 1
    } else {
        whole.start
            + xml[whole.clone()]
                .rfind("</")
                .ok_or("Missing closing XML element")?
    };
    Ok(Field {
        whole,
        content: end + 1..content_end,
        self_closing,
        opening_end: end,
    })
}

fn text(node: Node<'_, '_>) -> Result<String, String> {
    if node.children().any(|n| n.is_element()) {
        return Err(format!(
            "{} must not contain child elements",
            node.tag_name().name()
        ));
    }
    Ok(node
        .children()
        .filter(|n| n.is_text())
        .filter_map(|n| n.text())
        .collect())
}

fn tokens(node: Node<'_, '_>) -> Result<HashSet<String>, String> {
    Ok(text(node)?.split_whitespace().map(str::to_owned).collect())
}

fn parse(xml: String) -> Result<Profile, String> {
    // roxmltree disables DTD parsing by default. Keep that policy explicit.
    let doc = Document::parse_with_options(
        &xml,
        roxmltree::ParsingOptions {
            allow_dtd: false,
            ..Default::default()
        },
    )
    .map_err(|e| format!("Invalid profile XML: {e}"))?;
    let declaration = xml.trim_start_matches('\u{feff}').trim_start();
    if declaration.starts_with("<?xml") {
        let end = declaration.find("?>").ok_or("Invalid XML declaration")?;
        let decl = &declaration[..end];
        if let Some(start) = decl.find("encoding") {
            let tail = decl[start + 8..].trim_start();
            let tail = tail
                .strip_prefix('=')
                .ok_or("Invalid XML encoding declaration")?
                .trim_start();
            let quote = tail
                .chars()
                .next()
                .ok_or("Invalid XML encoding declaration")?;
            if quote != '\'' && quote != '"' {
                return Err("Invalid XML encoding declaration".into());
            }
            let encoding = tail[1..].split(quote).next().unwrap_or("");
            if !encoding.eq_ignore_ascii_case("utf-8") {
                return Err("Only UTF-8 profile XML is supported".into());
            }
        }
    }
    let root = doc.root_element();
    if root.tag_name().name() != "UserProfile" || root.tag_name().namespace().is_some() {
        return Err("Expected an unnamespaced UserProfile XML root".into());
    }
    let mut nodes = Vec::new();
    for name in ["achievementsList", "discoveredPickups", "coins", "stats"] {
        let matching: Vec<_> = root
            .children()
            .filter(|n| n.is_element() && n.tag_name().name() == name)
            .collect();
        if matching.len() != 1 || matching[0].tag_name().namespace().is_some() {
            return Err(format!(
                "Profile must contain exactly one direct {name} field"
            ));
        }
        nodes.push(matching[0]);
    }
    let achievements = tokens(nodes[0])?;
    let discoveries = tokens(nodes[1])?;
    let coin_text = text(nodes[2])?;
    let coins = if coin_text.trim().is_empty() {
        0
    } else {
        let value = coin_text.trim();
        if !value.bytes().all(|b| b.is_ascii_digit()) {
            return Err("Profile coins must be a nonnegative integer".into());
        }
        value
            .parse::<u64>()
            .map_err(|_| "Profile coin balance is out of range")?
    };
    if coins > MAX_COINS {
        return Err("Profile coin balance exceeds 2147483647".into());
    }
    let mut unlocks = HashSet::new();
    for node in nodes[3].children().filter(|n| {
        n.is_element() && n.tag_name().name() == "unlock" && n.tag_name().namespace().is_none()
    }) {
        let value = text(node)?;
        if !value.trim().is_empty() {
            unlocks.insert(value.trim().to_owned());
        }
    }
    let fields = [
        field(nodes[0], &xml)?,
        field(nodes[1], &xml)?,
        field(nodes[2], &xml)?,
        field(nodes[3], &xml)?,
    ];
    drop(doc);
    Ok(Profile {
        xml,
        achievements,
        discoveries,
        unlocks,
        coins,
        fields,
    })
}

fn parse_bytes(bytes: Vec<u8>) -> Result<Profile, String> {
    if bytes.len() as u64 > MAX_BYTES {
        return Err("Profile exceeds the 32 MiB limit".into());
    }
    let xml = String::from_utf8(bytes).map_err(|_| "Only UTF-8 profile XML is supported")?;
    parse(xml)
}

#[cfg(feature = "native")]
fn read_profile(path: &Path) -> Result<(Profile, fs::Permissions), String> {
    let file = File::open(path).map_err(|e| format!("Cannot open {}: {e}", path.display()))?;
    let metadata = file
        .metadata()
        .map_err(|e| format!("Cannot inspect profile: {e}"))?;
    if !metadata.is_file() {
        return Err("Profile path must be a regular file".into());
    }
    if metadata.len() > MAX_BYTES {
        return Err("Profile exceeds the 32 MiB limit".into());
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("Cannot read profile: {e}"))?;
    Ok((parse_bytes(bytes)?, metadata.permissions()))
}

fn revision(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn snapshot_label(path: &str, name: &str, p: &Profile) -> ProfileSnapshot {
    let unlocked_ids = catalog()
        .entries
        .iter()
        .filter(|e| {
            e.achievement
                .as_ref()
                .is_none_or(|v| p.achievements.contains(v))
                && e.unlock.as_ref().is_none_or(|v| p.unlocks.contains(v))
                && e.discovery
                    .as_ref()
                    .is_none_or(|v| p.discoveries.contains(v))
        })
        .map(|e| e.id.clone())
        .collect();
    ProfileSnapshot {
        path: path.to_owned(),
        name: name.to_owned(),
        coins: p.coins,
        revision: revision(p.xml.as_bytes()),
        unlocked_ids,
    }
}

#[cfg(feature = "native")]
fn snapshot(path: &Path, p: &Profile) -> ProfileSnapshot {
    snapshot_label(
        &path.to_string_lossy(),
        &path.file_name().unwrap_or_default().to_string_lossy(),
        p,
    )
}

pub fn inspect_profile_bytes(name: &str, bytes: Vec<u8>) -> Result<ProfileSnapshot, String> {
    let profile = parse_bytes(bytes)?;
    Ok(snapshot_label(name, name, &profile))
}

pub fn preview_profile_bytes(bytes: Vec<u8>, args: &ChangeRequest) -> Result<Preview, String> {
    let profile = parse_bytes(bytes)?;
    check_revision(
        &profile,
        &args.revision,
        "Profile revision does not match the supplied bytes.",
    )?;
    plan(&profile, args)
}

pub fn edit_profile_bytes(bytes: Vec<u8>, args: &ChangeRequest) -> Result<EditedProfile, String> {
    let profile = parse_bytes(bytes)?;
    check_revision(
        &profile,
        &args.revision,
        "Profile revision does not match the supplied bytes.",
    )?;
    let preview = plan(&profile, args)?;
    let edited = if preview.total_additions == 0 && preview.coins_before == preview.coins_after {
        profile
    } else {
        render(&profile, &preview)?
    };
    Ok(EditedProfile {
        profile: snapshot_label(&args.path, &args.path, &edited),
        preview,
        xml: edited.xml,
    })
}

#[cfg(feature = "native")]
pub fn open_profile(path: &Path) -> Result<ProfileSnapshot, String> {
    let path = fs::canonicalize(path).map_err(|e| format!("Cannot locate profile: {e}"))?;
    let (profile, _) = read_profile(&path)?;
    Ok(snapshot(&path, &profile))
}
fn check_revision(p: &Profile, expected: &str, message: &str) -> Result<(), String> {
    if revision(p.xml.as_bytes()) != expected {
        return Err(message.into());
    }
    Ok(())
}

fn plan(p: &Profile, args: &ChangeRequest) -> Result<Preview, String> {
    let coins_after = args.coins.unwrap_or(p.coins);
    if coins_after > MAX_COINS {
        return Err("Coins must be an integer between 0 and 2147483647".into());
    }
    let mut additions: [Vec<String>; 3] = Default::default();
    let existing = [&p.achievements, &p.discoveries, &p.unlocks];
    let mut added: [HashSet<&str>; 3] = Default::default();
    for id in &args.selection {
        let entry = CATALOG_BY_ID
            .get(id.as_str())
            .ok_or_else(|| format!("Unknown catalog selection: {id}"))?;
        for (index, value) in [&entry.achievement, &entry.discovery, &entry.unlock]
            .into_iter()
            .enumerate()
        {
            if let Some(value) = value {
                if !existing[index].contains(value) && added[index].insert(value.as_str()) {
                    additions[index].push(value.clone());
                }
            }
        }
    }
    let total_additions = additions.iter().map(Vec::len).sum();
    let changes = ["achievementsList", "discoveredPickups", "stats.unlock"]
        .into_iter()
        .zip(additions)
        .filter(|(_, added)| !added.is_empty())
        .map(|(field, added)| FieldChange {
            field: field.into(),
            added,
        })
        .collect();
    Ok(Preview {
        changes,
        coins_before: p.coins,
        coins_after,
        total_additions,
    })
}

#[cfg(feature = "native")]
pub fn preview_changes(args: &ChangeRequest) -> Result<Preview, String> {
    let (p, _) = read_profile(Path::new(&args.path))?;
    check_revision(
        &p,
        &args.revision,
        "Profile changed on disk. Reopen it before previewing or saving.",
    )?;
    plan(&p, args)
}

fn escape(value: &str) -> std::borrow::Cow<'_, str> {
    if !value.bytes().any(|b| matches!(b, b'&' | b'<' | b'>')) {
        return std::borrow::Cow::Borrowed(value);
    }
    let mut escaped = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            _ => escaped.push(c),
        }
    }
    std::borrow::Cow::Owned(escaped)
}
fn patch(p: &Profile, index: usize, content: String, replace: bool) -> (Range<usize>, String) {
    let field = &p.fields[index];
    if field.self_closing {
        let name = ["achievementsList", "discoveredPickups", "coins", "stats"][index];
        let mut value = p.xml[field.whole.start..field.opening_end - 1].to_owned();
        value.push('>');
        value.push_str(&content);
        value.push_str("</");
        value.push_str(name);
        value.push('>');
        (field.whole.clone(), value)
    } else if replace {
        (field.content.clone(), content)
    } else {
        (field.content.end..field.content.end, content)
    }
}
fn render(p: &Profile, preview: &Preview) -> Result<Profile, String> {
    let mut patches = Vec::new();
    for change in &preview.changes {
        let (index, prefix, suffix) = match change.field.as_str() {
            "achievementsList" => (0, " ", ""),
            "discoveredPickups" => (1, " ", ""),
            "stats.unlock" => (3, "<unlock>", "</unlock>"),
            _ => unreachable!(),
        };
        let capacity = change
            .added
            .iter()
            .map(|value| value.len() + prefix.len() + suffix.len())
            .sum();
        let mut content = String::with_capacity(capacity);
        for value in &change.added {
            content.push_str(prefix);
            content.push_str(&escape(value));
            content.push_str(suffix);
        }
        patches.push(patch(p, index, content, false));
    }
    if preview.coins_before != preview.coins_after {
        patches.push(patch(p, 2, preview.coins_after.to_string(), true));
    }
    patches.sort_by_key(|(range, _)| range.start);
    let mut xml =
        String::with_capacity(p.xml.len() + patches.iter().map(|(_, s)| s.len()).sum::<usize>());
    let mut cursor = 0;
    for (range, value) in patches {
        xml.push_str(&p.xml[cursor..range.start]);
        xml.push_str(&value);
        cursor = range.end;
    }
    xml.push_str(&p.xml[cursor..]);
    if xml.len() as u64 > MAX_BYTES {
        return Err("Edited profile would exceed the 32 MiB limit".into());
    }
    parse(xml)
}

#[cfg(feature = "native")]
fn sync_directory(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    File::open(path)
        .and_then(|f| f.sync_all())
        .map_err(|e| format!("Cannot sync profile directory: {e}"))?;
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}
#[cfg(feature = "native")]
fn backup(
    path: &Path,
    parent: &Path,
    p: &Profile,
    permissions: &fs::Permissions,
) -> Result<PathBuf, String> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    for counter in 0..1000 {
        let mut name = path
            .file_name()
            .ok_or("Profile has no filename")?
            .to_os_string();
        name.push(format!(
            ".backup-{stamp}-{}-{counter}.xml",
            std::process::id()
        ));
        let target = parent.join(name);
        let mut file = match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&target)
        {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(format!("Cannot create backup; original unchanged: {e}")),
        };
        let result = (|| {
            file.write_all(p.xml.as_bytes())?;
            file.set_permissions(permissions.clone())?;
            file.sync_all()
        })();
        drop(file);
        if let Err(e) = result {
            let _ = fs::remove_file(&target);
            return Err(format!("Cannot finish backup; original unchanged: {e}"));
        }
        sync_directory(parent)?;
        return Ok(target);
    }
    Err("Cannot create a unique backup; original unchanged".into())
}

#[cfg(feature = "native")]
pub fn save_changes(args: &ChangeRequest) -> Result<SaveResult, String> {
    let _guard = SAVE_LOCK
        .lock()
        .map_err(|_| "Profile save lock is unavailable")?;
    let path = fs::canonicalize(&args.path).map_err(|e| format!("Cannot locate profile: {e}"))?;
    let (p, permissions) = read_profile(&path)?;
    check_revision(
        &p,
        &args.revision,
        "Profile changed on disk. Reopen it before previewing or saving.",
    )?;
    let preview = plan(&p, args)?;
    if preview.total_additions == 0 && preview.coins_before == preview.coins_after {
        return Ok(SaveResult {
            profile: snapshot(&path, &p),
            backup_path: None,
            preview,
        });
    }
    let rendered = render(&p, &preview)?;
    let parent = path.parent().ok_or("Profile has no parent directory")?;
    let mut temp = tempfile::NamedTempFile::new_in(parent)
        .map_err(|e| format!("Cannot create replacement: {e}"))?;
    temp.write_all(rendered.xml.as_bytes())
        .map_err(|e| format!("Cannot write replacement: {e}"))?;
    temp.as_file()
        .set_permissions(permissions.clone())
        .map_err(|e| format!("Cannot preserve profile permissions: {e}"))?;
    temp.as_file()
        .sync_all()
        .map_err(|e| format!("Cannot sync replacement: {e}"))?;
    let backup_path = backup(&path, parent, &p, &permissions)?;
    let (latest, _) = read_profile(&path)?;
    check_revision(
        &latest,
        &args.revision,
        "Profile changed on disk. Reopen it before previewing or saving.",
    )?;
    // tempfile uses atomic rename on Unix and MOVEFILE_REPLACE_EXISTING on Windows.
    temp.persist(&path).map_err(|e| {
        format!(
            "Cannot replace profile; backup is {}: {e}",
            backup_path.display()
        )
    })?;
    sync_directory(parent).map_err(|e| {
        format!(
            "Profile saved with backup {}, but durability confirmation failed: {e}",
            backup_path.display()
        )
    })?;
    Ok(SaveResult {
        profile: snapshot(&path, &rendered),
        backup_path: Some(backup_path.to_string_lossy().into_owned()),
        preview,
    })
}
