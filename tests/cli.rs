//! End-to-end tests that run the real `tooka` binary.
//!
//! Every test gets its own temporary folder and points Tooka at it through
//! `TOOKA_CONFIG_DIR`, `TOOKA_DATA_DIR` and `TOOKA_SOURCE_FOLDER`, so the
//! tests never touch the configuration of the machine running them.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    thread,
    time::{Duration, Instant},
};
use tempfile::TempDir;

struct Tooka {
    _tmp: TempDir,
    root: PathBuf,
}

impl Tooka {
    fn new() -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let root = std::path::absolute(tmp.path()).unwrap();
        for dir in ["config", "data", "src", "home"] {
            fs::create_dir_all(root.join(dir)).unwrap();
        }
        Self { _tmp: tmp, root }
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_tooka"));
        cmd.args(args)
            .current_dir(&self.root)
            .env("TOOKA_CONFIG_DIR", self.root.join("config"))
            .env("TOOKA_DATA_DIR", self.root.join("data"))
            .env("TOOKA_SOURCE_FOLDER", self.root.join("src"))
            .env("HOME", self.root.join("home"))
            .env("NO_COLOR", "1");
        cmd
    }

    fn run(&self, args: &[&str]) -> Output {
        self.command(args).output().unwrap()
    }

    /// Runs tooka, asserts success and returns stdout + stderr.
    fn ok(&self, args: &[&str]) -> String {
        let out = self.run(args);
        let text = text(&out);
        assert!(out.status.success(), "tooka {args:?} failed:\n{text}");
        text
    }

    /// Runs tooka, asserts failure and returns stdout + stderr.
    fn fails(&self, args: &[&str]) -> String {
        let out = self.run(args);
        let text = text(&out);
        assert!(!out.status.success(), "tooka {args:?} should fail:\n{text}");
        text
    }

    fn path(&self, rel: &str) -> PathBuf {
        self.root.join(rel)
    }

    fn write(&self, rel: &str, content: &str) -> PathBuf {
        let path = self.path(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, content).unwrap();
        path
    }

    fn read(&self, rel: &str) -> String {
        fs::read_to_string(self.path(rel)).unwrap()
    }

    /// Path as it should appear inside a YAML file (forward slashes work on
    /// every platform and need no escaping).
    fn yaml_path(&self, rel: &str) -> String {
        self.path(rel).to_string_lossy().replace('\\', "/")
    }

    fn add_rule(&self, yaml: &str) {
        let file = self.write("rule-to-add.yaml", yaml);
        self.ok(&["add", file.to_str().unwrap()]);
        fs::remove_file(file).unwrap();
    }
}

fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn pdf_rule(t: &Tooka, extra: &str) -> String {
    format!(
        "id: pdfs\nname: PDFs\nenabled: true\ndescription: null\npriority: 1\nwhen:\n  extensions: [pdf]\nthen:\n- action: move\n  to: {}\n{extra}",
        t.yaml_path("dest/pdfs")
    )
}

#[test]
fn rule_management_and_sorting_workflow() {
    let t = Tooka::new();

    // Template: created once, never overwritten.
    t.ok(&["template", "--output", "template.yaml"]);
    assert!(t.read("template.yaml").contains("id: example_rule"));
    assert!(
        t.fails(&["template", "--output", "template.yaml"])
            .contains("already exists")
    );

    // Add, list, validate.
    let rule = t.write("pdfs.yaml", &pdf_rule(&t, ""));
    t.ok(&["validate", rule.to_str().unwrap()]);
    t.ok(&["add", rule.to_str().unwrap()]);
    assert!(t.ok(&["list"]).contains("pdfs"));

    t.write("src/a.pdf", "first");
    t.write("src/notes.txt", "keep");

    // Dry run changes nothing.
    let out = t.ok(&["sort", "--dry-run"]);
    assert!(out.contains("a.pdf"), "{out}");
    assert!(t.path("src/a.pdf").exists());
    assert!(!t.path("dest").exists());

    // Real sort with a JSON report.
    t.ok(&["sort", "--report", "json", "--output", "reports"]);
    assert_eq!(t.read("dest/pdfs/a.pdf"), "first");
    assert!(!t.path("src/a.pdf").exists());
    assert_eq!(t.read("src/notes.txt"), "keep");
    assert!(t.read("reports/tooka_report.json").contains("\"move\""));

    // The destination exists now: the next sort stops without changing anything.
    t.write("src/a.pdf", "second");
    let out = t.fails(&["sort"]);
    assert!(out.contains("Nothing was changed"), "{out}");
    assert_eq!(t.read("src/a.pdf"), "second");
    assert_eq!(t.read("dest/pdfs/a.pdf"), "first");

    // Export never overwrites either.
    t.ok(&["export", "pdfs", "--output", "exported.yaml"]);
    assert!(t.read("exported.yaml").contains("id: pdfs"));
    t.fails(&["export", "pdfs", "--output", "exported.yaml"]);

    // Disabled rules are not applied.
    t.ok(&["toggle", "pdfs"]);
    assert!(t.ok(&["list"]).contains("Disabled"));
    t.fails(&["sort"]);

    t.ok(&["remove", "pdfs"]);
    assert!(!t.ok(&["list"]).contains("pdfs"));
}

#[test]
fn on_conflict_rename_keeps_both_files() {
    let t = Tooka::new();
    t.add_rule(&pdf_rule(&t, "on_conflict: rename\n"));
    t.write("dest/pdfs/a.pdf", "old");
    t.write("src/a.pdf", "new");

    t.ok(&["sort"]);
    assert_eq!(t.read("dest/pdfs/a.pdf"), "old");
    assert_eq!(t.read("dest/pdfs/a (1).pdf"), "new");
}

#[test]
fn reports_in_all_formats() {
    let t = Tooka::new();
    t.add_rule(&pdf_rule(&t, ""));
    for (i, format) in ["json", "csv", "pdf"].iter().enumerate() {
        t.write(&format!("src/{i}.pdf"), "x");
        t.ok(&["sort", "--report", format, "--output", "reports"]);
        let report = fs::read(t.path(&format!("reports/tooka_report.{format}"))).unwrap();
        assert!(!report.is_empty(), "{format} report is empty");
    }
    assert!(
        fs::read(t.path("reports/tooka_report.pdf"))
            .unwrap()
            .starts_with(b"%PDF-")
    );
}

#[test]
fn invalid_rules_are_rejected() {
    let t = Tooka::new();
    let bad = t.write(
        "bad.yaml",
        "id: bad\nname: Bad\nenabled: true\ndescription: null\npriority: 1\nwhen:\n  filename: '([unclosed'\nthen:\n- action: skip\n",
    );
    let out = t.fails(&["validate", bad.to_str().unwrap()]);
    assert!(
        out.contains("([unclosed"),
        "validate checks patterns: {out}"
    );
    let out = t.fails(&["add", bad.to_str().unwrap()]);
    assert!(out.contains("([unclosed"), "{out}");

    let broken = t.write("broken.yaml", "id: x\nwhen: [unclosed\n");
    let out = t.fails(&["validate", broken.to_str().unwrap()]);
    assert!(
        out.contains("line 2"),
        "errors point at the location: {out}"
    );
}

#[test]
fn recover_restores_files_after_an_interrupted_sort() {
    let t = Tooka::new();
    assert!(t.ok(&["recover"]).contains("Nothing to recover"));

    // State after a crash: a.txt was moved to dest/a.txt, journal left behind.
    let from = t.path("src/a.txt");
    let to = t.write("dest/a.txt", "content");
    let json_path = |p: &Path| serde_json_string(&p.to_string_lossy());
    let journal = format!(
        concat!(
            "{{\"op\":\"begin\",\"version\":1,\"started\":\"\",\"source\":{src}}}\n",
            "{{\"op\":\"dir_create\",\"path\":{dest}}}\n",
            "{{\"op\":\"claim\",\"path\":{to}}}\n",
            "{{\"op\":\"move\",\"from\":{from},\"to\":{to}}}\n",
        ),
        src = json_path(&t.path("src")),
        dest = json_path(&t.path("dest")),
        from = json_path(&from),
        to = json_path(&to),
    );
    t.write("data/sort-journal.jsonl", &journal);

    let out = t.fails(&["sort"]);
    assert!(out.contains("tooka recover"), "{out}");

    t.ok(&["recover"]);
    assert_eq!(fs::read_to_string(&from).unwrap(), "content");
    assert!(!t.path("dest").exists());
    assert!(!t.path("data/sort-journal.jsonl").exists());
}

/// Minimal JSON string encoding for paths (escapes backslashes and quotes).
fn serde_json_string(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Files written by Tooka v1.1.2 must keep working.
#[test]
fn files_from_v1_1_2_still_load() {
    let t = Tooka::new();
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/v1.1.2");
    let root = t.root.to_string_lossy().replace('\\', "/");
    for (fixture, target) in [
        ("tooka.yaml", "config/tooka.yaml"),
        ("monitoring.yml", "config/monitoring.yml"),
        ("rules.yaml", "data/rules.yaml"),
    ] {
        let content = fs::read_to_string(fixtures.join(fixture))
            .unwrap()
            .replace("@ROOT@", &root);
        t.write(target, &content);
    }

    assert!(t.ok(&["config", "--show"]).contains(&root));

    let list = t.ok(&["list"]);
    for id in ["photos", "docs", "junk", "ignore"] {
        assert!(list.contains(id), "rule {id} missing:\n{list}");
    }

    let status = t.ok(&["monitor", "status"]);
    assert!(status.contains("Success"), "{status}");
    assert!(status.contains("Failed"), "{status}");

    // Saving with the new version and loading again still works.
    t.ok(&["toggle", "docs"]);
    t.ok(&["list"]);
    t.ok(&["monitor", "add", "src", "--interval", "60"]);
    let status = t.ok(&["monitor", "status"]);
    assert!(status.contains("Failed"), "{status}");

    // The fixture rules are usable for sorting: matching (including the
    // metadata condition of "photos") and rename templates run.
    t.write("src/IMG_1.jpg", "not really a photo");
    t.write("src/download.part", "partial");
    // Nothing matches ("ignore" skips the .part file, the fake photo has no
    // EXIF data), but every condition and action of the fixture is evaluated.
    let out = t.ok(&["sort", "--dry-run"]);
    assert!(out.contains("nothing was changed"), "{out}");
    assert!(t.path("src/IMG_1.jpg").exists());
}

#[test]
fn monitor_folders_and_daemon_lifecycle() {
    let t = Tooka::new();
    t.add_rule(&pdf_rule(&t, ""));
    fs::create_dir_all(t.path("watched")).unwrap();
    t.write("watched/a.pdf", "x");

    t.ok(&["monitor", "add", "watched", "--interval", "3600"]);
    let list = t.ok(&["monitor", "list"]);
    assert!(
        list.contains(&t.path("watched").to_string_lossy().to_string()),
        "stored as absolute path: {list}"
    );
    t.fails(&["monitor", "add", "watched"]);

    assert!(
        t.ok(&["monitor", "daemon", "status"])
            .contains("not running")
    );

    let mut daemon = t
        .command(&["monitor", "daemon", "run", "--check-interval", "3600"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let stop = Stopper(&mut daemon);

    wait_for(|| {
        t.ok(&["monitor", "daemon", "status"])
            .contains("is running")
    });
    // The daemon sorts due folders right away.
    wait_for(|| t.path("dest/pdfs/a.pdf").exists());
    assert!(
        t.fails(&["monitor", "daemon", "run"])
            .contains("already running")
    );

    t.ok(&["monitor", "daemon", "stop"]);
    assert!(
        t.ok(&["monitor", "daemon", "status"])
            .contains("not running")
    );
    drop(stop);

    t.ok(&["monitor", "remove", "watched"]);
    assert!(t.ok(&["monitor", "list"]).contains("No folders"));
}

/// Kills the daemon if the test fails before stopping it.
struct Stopper<'a>(&'a mut Child);

impl Drop for Stopper<'_> {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn wait_for(mut condition: impl FnMut() -> bool) {
    let start = Instant::now();
    while !condition() {
        assert!(
            start.elapsed() < Duration::from_secs(20),
            "condition not met in time"
        );
        thread::sleep(Duration::from_millis(100));
    }
}

#[test]
fn completions_are_generated() {
    let t = Tooka::new();
    assert!(t.ok(&["completions", "bash"]).contains("tooka"));
}
