//! The parallel folder walk, the search of one file, and the skip count.
//!
//! `scan` walks each root with the `ignore` crate, searches each regular file
//! with `grep-searcher`, and counts the entries that the walk skips.
//! `scan_input` searches the piped text with the same searcher, and
//! `scan_current_folder` walks `.` for a search with no path (FEATURE-14).

use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use grep_matcher::Matcher;
use grep_searcher::{BinaryDetection, Searcher, SearcherBuilder, Sink, SinkFinish, SinkMatch};
use ignore::{DirEntry, WalkBuilder, WalkState};

use super::Source;
use super::filter::FileFilter;

/// The options of a scan. `ScanOptions::default()` is what jag uses.
#[derive(Clone, Debug)]
pub struct ScanOptions {
    /// The count of threads. 0: the `ignore` crate picks the count.
    pub threads: usize,
    /// Read the global git ignore file.
    pub global_ignore: bool,
    /// Read the ignore files of parent folders.
    pub parent_ignore: bool,
    /// The globs of --include and --exclude (FEATURE-13).
    pub files: FileFilter,
    /// Keep a `FileMatches` with no line for each searched file with no
    /// match (FEATURE-13, `-c`).
    pub keep_files_with_no_match: bool,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            threads: 0,
            global_ignore: true,
            parent_ignore: true,
            files: FileFilter::default(),
            keep_files_with_no_match: false,
        }
    }
}

/// One line that matches.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatchLine {
    /// The number of the line. The first line is 1.
    pub line_number: u64,
    /// The line, with no line terminator.
    pub text: Vec<u8>,
}

/// The matching lines of one file, or of the piped text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileMatches {
    /// For a file, the root joined with the rest of the path, as the walk
    /// gives it. For the piped text, `Source::StandardInput`.
    pub source: Source,
    /// The matching lines, sorted by line number.
    pub lines: Vec<MatchLine>,
}

/// The count of entries that jag skipped, by reason.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SkipCounts {
    pub binary: u64,
    pub hidden: u64,
    pub ignored: u64,
}

impl SkipCounts {
    /// The count of all skipped entries.
    pub fn total(&self) -> u64 {
        self.binary + self.hidden + self.ignored
    }

    /// The notice for the user. `None` when jag skipped nothing.
    pub fn notice(&self) -> Option<String> {
        if self.total() == 0 {
            return None;
        }
        Some(format!(
            "jag: skipped {} files and folders ({} binary, {} hidden, {} ignored)",
            self.total(),
            self.binary,
            self.hidden,
            self.ignored
        ))
    }
}

/// The reason for a warning.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WarningKind {
    /// An error with an `io::ErrorKind`.
    Read(io::ErrorKind),
    /// A bad rule or line in an ignore file.
    IgnoreRule,
    /// Any other error of the `ignore` crate.
    Other,
    /// A path of the command line that does not exist (FEATURE-14,
    /// decision 7).
    Missing,
    /// An error of the read of standard input (FEATURE-14, decision 12).
    StandardInput(io::ErrorKind),
}

/// A problem that did not stop the walk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WalkWarning {
    /// `None` when the error has no path.
    pub path: Option<PathBuf>,
    pub kind: WarningKind,
}

impl WalkWarning {
    /// The text of the warning, with no `jag: ` in front. It holds no error
    /// text from a crate.
    pub fn text(&self) -> String {
        match (&self.kind, &self.path) {
            (WarningKind::Read(kind), Some(path)) => {
                format!("cannot read {}: {kind}", path.display())
            }
            (WarningKind::Read(kind), None) => format!("cannot read a file: {kind}"),
            (WarningKind::IgnoreRule, Some(path)) => {
                format!(
                    "a rule in {} is not valid; jag uses the other rules",
                    path.display()
                )
            }
            (WarningKind::IgnoreRule, None) => {
                "an ignore rule is not valid; jag uses the other rules".to_string()
            }
            (WarningKind::Other, Some(path)) => format!("cannot walk {}", path.display()),
            (WarningKind::Other, None) => "cannot walk a folder".to_string(),
            (WarningKind::Missing, Some(path)) => {
                format!("the path {} does not exist", path.display())
            }
            (WarningKind::Missing, None) => "a path does not exist".to_string(),
            (WarningKind::StandardInput(kind), _) => {
                format!("cannot read standard input: {kind}")
            }
        }
    }

    /// One line for the error stream of the text mode.
    pub fn message(&self) -> String {
        format!("jag: {}", self.text())
    }
}

/// The result of a scan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScanReport {
    /// Only the files with a match (with `keep_files_with_no_match`, also each
    /// searched file with no match and no binary data), in the order of the paths on the command
    /// line, then in the byte order of the path. A path that the command line
    /// gives two times, or a path under another path, gives its files at each
    /// place of the root, as `grep` does.
    pub files: Vec<FileMatches>,
    pub skipped: SkipCounts,
    /// By the place of the first root that holds the path (a warning with no
    /// path, or with a path under no root, takes place 0), then by path, then
    /// by message.
    pub warnings: Vec<WalkWarning>,
}

impl ScanReport {
    /// A report with no file, no skip, and no warning.
    pub fn empty() -> ScanReport {
        ScanReport {
            files: Vec::new(),
            skipped: SkipCounts::default(),
            warnings: Vec::new(),
        }
    }

    /// True when a path of the command line does not exist (FEATURE-14,
    /// decisions 7 and 20).
    pub fn path_missing(&self) -> bool {
        self.warnings
            .iter()
            .any(|warning| warning.kind == WarningKind::Missing)
    }
}

/// The state that the threads of the walk share.
#[derive(Default)]
struct Shared {
    binary: AtomicU64,
    hidden_children: AtomicU64,
    children: AtomicU64,
    yielded: AtomicU64,
    yielded_hidden: AtomicU64,
    /// Each file with the root that the walk found it under.
    files: Mutex<Vec<(PathBuf, FileMatches)>>,
    warnings: Mutex<Vec<WalkWarning>>,
    /// Paths of errors for entries at depth 1 or more.
    error_paths: Mutex<Vec<PathBuf>>,
    /// Paths of the folders that the walk yielded.
    folder_paths: Mutex<HashSet<PathBuf>>,
}

/// Walk each root, search each file, and count what jag skips.
pub fn scan<M>(roots: &[PathBuf], matcher: &M, options: &ScanOptions) -> ScanReport
where
    M: Matcher + Sync,
{
    let Some((first, rest)) = roots.split_first() else {
        return ScanReport::empty();
    };
    let mut builder = WalkBuilder::new(first);
    for root in rest {
        builder.add(root);
    }
    builder
        .follow_links(false)
        .threads(options.threads)
        .git_global(options.global_ignore)
        .parents(options.parent_ignore);

    let shared = Shared::default();
    builder.build_parallel().run(|| {
        let mut searcher = searcher();
        let shared = &shared;
        Box::new(move |result| {
            match result {
                Ok(entry) => visit_entry(shared, matcher, options, &mut searcher, &entry),
                Err(error) => visit_error(shared, &error),
            }
            WalkState::Continue
        })
    });
    finish(shared, roots)
}

/// Search the piped text with the searcher of a file, so that the binary
/// rule and the byte order mark rule are the same (FEATURE-14, decision 11).
/// Text with no byte gives no file and no warning (decision 3). With
/// `keep_no_match` (`-c`), text with no match gives a file with no line, also
/// when it has no byte (FEATURE-13, decision 20).
pub fn scan_input<M: Matcher>(bytes: &[u8], matcher: &M, keep_no_match: bool) -> ScanReport {
    let mut report = ScanReport::empty();
    let mut sink = LineSink::default();
    match searcher().search_reader(matcher, bytes, &mut sink) {
        Err(error) => report.warnings.push(WalkWarning {
            path: None,
            kind: WarningKind::StandardInput(error.kind()),
        }),
        Ok(()) if sink.binary => report.skipped.binary = 1,
        Ok(()) if keep_no_match || !sink.lines.is_empty() => report.files.push(FileMatches {
            source: Source::StandardInput,
            lines: sink.lines,
        }),
        Ok(()) => {}
    }
    report
}

/// Walk `.` for a search with no path, and remove the `./` at the start of
/// each path: the user typed no root (FEATURE-14, decision 5).
pub fn scan_current_folder<M>(matcher: &M, options: &ScanOptions) -> ScanReport
where
    M: Matcher + Sync,
{
    without_current_folder_prefix(scan(&[PathBuf::from(".")], matcher, options))
}

/// Remove the first `.` of each path of a scan of `.`, then sort the two
/// lists again with the keys of `order_files` and `finish`.
fn without_current_folder_prefix(mut report: ScanReport) -> ScanReport {
    for file in &mut report.files {
        if let Source::File(path) = &mut file.source {
            *path = without_current_folder(path);
        }
    }
    for warning in &mut report.warnings {
        if let Some(path) = &mut warning.path {
            *path = without_current_folder(path);
        }
    }
    report
        .files
        .sort_by(|a, b| a.source.label().cmp(b.source.label()));
    sort_warnings(&mut report.warnings, &[]);
    report
}

/// `./a.txt` gives `a.txt`. The path `.` alone, and a path that does not
/// start with `.`, stay as they are.
fn without_current_folder(path: &Path) -> PathBuf {
    match path.strip_prefix(".") {
        Ok(rest) if !rest.as_os_str().is_empty() => rest.to_path_buf(),
        _ => path.to_path_buf(),
    }
}

/// The searcher of a file and of the piped text.
fn searcher() -> Searcher {
    SearcherBuilder::new()
        .binary_detection(BinaryDetection::quit(b'\0'))
        .line_number(true)
        .build()
}

fn visit_entry<M: Matcher>(
    shared: &Shared,
    matcher: &M,
    options: &ScanOptions,
    searcher: &mut Searcher,
    entry: &DirEntry,
) {
    if let Some(error) = entry.error() {
        push(&shared.warnings, make_warning(error));
    }
    if entry.depth() >= 1 {
        shared.yielded.fetch_add(1, Ordering::Relaxed);
        if is_hidden(entry.file_name()) {
            shared.yielded_hidden.fetch_add(1, Ordering::Relaxed);
        }
    }
    let Some(file_type) = entry.file_type() else {
        return;
    };
    if file_type.is_dir() {
        count_children(shared, entry.path());
        if let Ok(mut folders) = shared.folder_paths.lock() {
            folders.insert(entry.path().to_path_buf());
        }
    } else if file_type.is_file() && options.files.allows(entry.file_name()) {
        // The walk counted the entry above, so a file that a glob skips is
        // not in the skip notice (FEATURE-13, decision 27).
        let path = entry.path();
        let root = path.ancestors().nth(entry.depth()).unwrap_or(path);
        search_file(
            shared,
            matcher,
            searcher,
            root,
            path,
            options.keep_files_with_no_match,
        );
    }
}

fn visit_error(shared: &Shared, error: &ignore::Error) {
    push(&shared.warnings, make_warning(error));
    if error.depth().is_some_and(|depth| depth >= 1)
        && let Some(path) = error_path(error)
    {
        push(&shared.error_paths, path);
    }
}

/// Read the children of a folder to count the entries that the walk skips.
fn count_children(shared: &Shared, folder: &Path) {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return;
    };
    let mut children = 0;
    let mut hidden = 0;
    for entry in entries.flatten() {
        children += 1;
        if is_hidden(&entry.file_name()) {
            hidden += 1;
        }
    }
    shared.children.fetch_add(children, Ordering::Relaxed);
    shared.hidden_children.fetch_add(hidden, Ordering::Relaxed);
}

fn search_file<M: Matcher>(
    shared: &Shared,
    matcher: &M,
    searcher: &mut Searcher,
    root: &Path,
    path: &Path,
    keep_no_match: bool,
) {
    let mut sink = LineSink::default();
    match searcher.search_path(matcher, path, &mut sink) {
        Err(error) => push(
            &shared.warnings,
            WalkWarning {
                path: Some(path.to_path_buf()),
                kind: WarningKind::Read(error.kind()),
            },
        ),
        Ok(()) if sink.binary => {
            shared.binary.fetch_add(1, Ordering::Relaxed);
        }
        Ok(()) if keep_no_match || !sink.lines.is_empty() => push(
            &shared.files,
            (
                root.to_path_buf(),
                FileMatches {
                    source: Source::File(path.to_path_buf()),
                    lines: sink.lines,
                },
            ),
        ),
        Ok(()) => {}
    }
}

/// Keeps each matching line. A file with binary data gives no line.
#[derive(Default)]
struct LineSink {
    lines: Vec<MatchLine>,
    binary: bool,
}

impl Sink for LineSink {
    type Error = io::Error;

    fn matched(&mut self, _searcher: &Searcher, mat: &SinkMatch<'_>) -> Result<bool, io::Error> {
        let mut text = mat.bytes();
        text = text.strip_suffix(b"\n").unwrap_or(text);
        text = text.strip_suffix(b"\r").unwrap_or(text);
        self.lines.push(MatchLine {
            line_number: mat.line_number().unwrap_or(0),
            text: text.to_vec(),
        });
        Ok(true)
    }

    fn binary_data(&mut self, _searcher: &Searcher, _offset: u64) -> Result<bool, io::Error> {
        self.binary = true;
        self.lines.clear();
        Ok(false)
    }

    fn finish(&mut self, _searcher: &Searcher, finish: &SinkFinish) -> Result<(), io::Error> {
        if finish.binary_byte_offset().is_some() {
            self.binary = true;
            self.lines.clear();
        }
        Ok(())
    }
}

/// Combine the counters and the lists after the walk.
fn finish(shared: Shared, roots: &[PathBuf]) -> ScanReport {
    let mut yielded = shared.yielded.into_inner();
    let mut yielded_hidden = shared.yielded_hidden.into_inner();
    let folder_paths = shared.folder_paths.into_inner().unwrap_or_default();
    // An entry that the crate could not read gives an error and no entry.
    for path in shared.error_paths.into_inner().unwrap_or_default() {
        if folder_paths.contains(&path) {
            continue;
        }
        yielded += 1;
        if path.file_name().is_some_and(is_hidden) {
            yielded_hidden += 1;
        }
    }
    let children = shared.children.into_inner();
    let hidden = shared
        .hidden_children
        .into_inner()
        .saturating_sub(yielded_hidden);
    let ignored = children.saturating_sub(yielded).saturating_sub(hidden);

    let files = order_files(shared.files.into_inner().unwrap_or_default(), roots);
    let mut warnings = shared.warnings.into_inner().unwrap_or_default();
    sort_warnings(&mut warnings, roots);
    ScanReport {
        files,
        skipped: SkipCounts {
            binary: shared.binary.into_inner(),
            hidden,
            ignored,
        },
        warnings,
    }
}

/// Sort the warnings by the place of their root, then by path, then by
/// message.
fn sort_warnings(warnings: &mut [WalkWarning], roots: &[PathBuf]) {
    warnings.sort_by_cached_key(|warning| {
        (
            warning
                .path
                .as_deref()
                .map_or(0, |path| warning_place(roots, path)),
            warning.path.as_deref().map(|path| path_key(path).to_vec()),
            warning.message(),
        )
    });
}

fn push<T>(list: &Mutex<Vec<T>>, item: T) {
    if let Ok(mut list) = list.lock() {
        list.push(item);
    }
}

/// Sort the files by the place of their root on the command line, then by the
/// bytes of the path. A root that the command line gives two times has two
/// places. The walk gives its files two times, and the copies of one file take
/// the places of that root, one each. The copies are equal, so the order of
/// the threads does not change the report.
fn order_files(found: Vec<(PathBuf, FileMatches)>, roots: &[PathBuf]) -> Vec<FileMatches> {
    let mut places: HashMap<&Path, Vec<usize>> = HashMap::new();
    for (place, root) in roots.iter().enumerate() {
        places.entry(root.as_path()).or_default().push(place);
    }
    let mut found = found;
    found.sort_by(|(root_a, a), (root_b, b)| {
        (root_a, a.source.label()).cmp(&(root_b, b.source.label()))
    });
    let mut placed: Vec<(usize, FileMatches)> = Vec::with_capacity(found.len());
    let mut copy = 0;
    for (index, (root, file)) in found.iter().enumerate() {
        let same =
            index > 0 && found[index - 1].0 == *root && found[index - 1].1.source == file.source;
        copy = if same { copy + 1 } else { 0 };
        let own = places.get(root.as_path()).map_or(&[][..], Vec::as_slice);
        // A root that the table does not know goes to place 0.
        let place = if own.is_empty() {
            0
        } else {
            own[copy % own.len()]
        };
        placed.push((place, file.clone()));
    }
    placed.sort_by(|(place_a, a), (place_b, b)| {
        (place_a, a.source.label()).cmp(&(place_b, b.source.label()))
    });
    placed.into_iter().map(|(_, file)| file).collect()
}

/// The place of the first root, in the order of the command line, that holds
/// `path`. A path under no root takes place 0.
fn warning_place(roots: &[PathBuf], path: &Path) -> usize {
    roots
        .iter()
        .position(|root| path.starts_with(root))
        .unwrap_or(0)
}

fn path_key(path: &Path) -> &[u8] {
    path.as_os_str().as_encoded_bytes()
}

fn is_hidden(name: &OsStr) -> bool {
    name.as_encoded_bytes().first() == Some(&b'.')
}

/// The path of the first `WithPath`, or the `child` of a loop.
fn error_path(error: &ignore::Error) -> Option<PathBuf> {
    match error {
        ignore::Error::WithPath { path, .. } => Some(path.clone()),
        ignore::Error::Loop { child, .. } => Some(child.clone()),
        ignore::Error::WithDepth { err, .. } | ignore::Error::WithLineNumber { err, .. } => {
            error_path(err)
        }
        ignore::Error::Partial(errors) if errors.len() == 1 => error_path(&errors[0]),
        _ => None,
    }
}

/// True when the error is a bad rule or a bad line in an ignore file.
fn is_rule_error(error: &ignore::Error) -> bool {
    match error {
        ignore::Error::WithLineNumber { .. }
        | ignore::Error::Glob { .. }
        | ignore::Error::UnrecognizedFileType(_)
        | ignore::Error::InvalidDefinition => true,
        ignore::Error::WithPath { err, .. } | ignore::Error::WithDepth { err, .. } => {
            is_rule_error(err)
        }
        ignore::Error::Partial(errors) => errors.iter().any(is_rule_error),
        ignore::Error::Loop { .. } | ignore::Error::Io(_) => false,
    }
}

/// An error at depth 0 is an error of a root: the walk reads the metadata of
/// each root, and follows a symbolic link. `NotFound` and `NotADirectory`
/// there mean that the path does not exist (FEATURE-14, decision 7). Another
/// error of a root, for example `PermissionDenied`, is a read error
/// (decision 10).
fn make_warning(error: &ignore::Error) -> WalkWarning {
    let kind = if let Some(io_error) = error.io_error() {
        let missing = matches!(
            io_error.kind(),
            io::ErrorKind::NotFound | io::ErrorKind::NotADirectory
        );
        if missing && error.depth() == Some(0) {
            WarningKind::Missing
        } else {
            WarningKind::Read(io_error.kind())
        }
    } else if error.is_partial() || is_rule_error(error) {
        WarningKind::IgnoreRule
    } else {
        WarningKind::Other
    };
    WalkWarning {
        path: error_path(error),
        kind,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn warning(path: Option<&str>, kind: WarningKind) -> WalkWarning {
        WalkWarning {
            path: path.map(PathBuf::from),
            kind,
        }
    }

    // `finish` looks up each error path in the set of the folders. A search of
    // a large tree has many of both, so the time must not grow with their
    // product (FEATURE-16, CRITERION-65).
    #[test]
    fn finish_with_many_folders_and_many_error_paths_is_fast() {
        let shared = Shared::default();
        shared.children.store(50_000, Ordering::Relaxed);
        shared.yielded.store(40_000, Ordering::Relaxed);
        for number in 0..40_000 {
            if let Ok(mut folders) = shared.folder_paths.lock() {
                folders.insert(PathBuf::from(format!("/tree/d{number:05}")));
            }
        }
        // 2,000 error paths are folders, and 2,000 are not.
        for number in 0..2_000 {
            push(
                &shared.error_paths,
                PathBuf::from(format!("/tree/d{number:05}")),
            );
            push(
                &shared.error_paths,
                PathBuf::from(format!("/tree/x{number:05}")),
            );
        }
        let started = std::time::Instant::now();
        let report = finish(shared, &[]);
        assert!(started.elapsed() < std::time::Duration::from_secs(1));
        // Only the 2,000 error paths that are not folders count as entries.
        assert_eq!(report.skipped.ignored, 50_000 - 42_000);
    }

    // No real folder gives two warnings with the same path and different
    // kinds, so this test gives made-up warnings to `finish`.
    #[test]
    fn warnings_with_the_same_path_come_in_the_order_of_their_message() {
        let read = WarningKind::Read(io::ErrorKind::PermissionDenied);
        let shared = Shared::default();
        for made_up in [
            warning(Some("b"), WarningKind::Other),
            warning(Some("a"), WarningKind::Other),
            warning(Some("a"), read),
            warning(None, WarningKind::Other),
            warning(Some("a"), WarningKind::IgnoreRule),
        ] {
            push(&shared.warnings, made_up);
        }
        let report = finish(shared, &[]);
        // "jag: a rule in a ..." < "jag: cannot read a: ..." < "jag: cannot walk a"
        assert_eq!(
            report.warnings,
            [
                warning(None, WarningKind::Other),
                warning(Some("a"), WarningKind::IgnoreRule),
                warning(Some("a"), read),
                warning(Some("a"), WarningKind::Other),
                warning(Some("b"), WarningKind::Other),
            ]
        );
    }

    fn found(root: &str, path: &str) -> (PathBuf, FileMatches) {
        (
            PathBuf::from(root),
            FileMatches {
                source: Source::File(PathBuf::from(path)),
                lines: Vec::new(),
            },
        )
    }

    fn order(found: Vec<(PathBuf, FileMatches)>, roots: &[&str]) -> Vec<String> {
        let roots: Vec<PathBuf> = roots.iter().map(PathBuf::from).collect();
        order_files(found, &roots)
            .iter()
            .map(|file| String::from_utf8_lossy(file.source.label()).into_owned())
            .collect()
    }

    // Decision 6 of FEATURE-14: a path that the command line gives two times
    // has two places, and the copies of a file take them one each.
    #[test]
    fn a_path_that_is_given_two_times_gives_its_files_at_each_place() {
        let got = order(
            vec![found("b", "b"), found("a", "a"), found("b", "b")],
            &["b", "a", "b"],
        );
        assert_eq!(got, ["b", "a", "b"]);
    }

    #[test]
    fn a_path_under_another_path_gives_its_files_at_the_place_of_each_root() {
        // `d d/sub d`, with the files d/a and d/sub/c.
        let roots = ["d", "d/sub", "d"];
        let got = order(
            vec![
                found("d", "d/a"),
                found("d", "d/a"),
                found("d", "d/sub/c"),
                found("d", "d/sub/c"),
                found("d/sub", "d/sub/c"),
            ],
            &roots,
        );
        assert_eq!(got, ["d/a", "d/sub/c", "d/sub/c", "d/a", "d/sub/c"]);
    }

    #[test]
    fn a_file_inside_a_root_is_ordered_by_its_bytes() {
        let got = order(
            vec![found("d", "d/b"), found("d", "d/a"), found("d", "d/B")],
            &["d"],
        );
        assert_eq!(got, ["d/B", "d/a", "d/b"]);
    }

    #[test]
    fn a_warning_takes_the_place_of_the_first_root_that_holds_its_path() {
        let roots = [PathBuf::from("b"), PathBuf::from("a"), PathBuf::from("a/x")];
        assert_eq!(warning_place(&roots, Path::new("b/f")), 0);
        assert_eq!(warning_place(&roots, Path::new("a/x/f")), 1);
        assert_eq!(warning_place(&roots, Path::new("/elsewhere")), 0);
    }

    #[test]
    fn a_warning_with_no_root_above_it_comes_with_place_0() {
        let shared = Shared::default();
        for made_up in [
            warning(Some("a/f"), WarningKind::Other),
            warning(Some("/up"), WarningKind::Other),
            warning(None, WarningKind::Other),
            warning(Some("b/f"), WarningKind::Other),
        ] {
            push(&shared.warnings, made_up);
        }
        let roots = [PathBuf::from("b"), PathBuf::from("a")];
        let report = finish(shared, &roots);
        let got: Vec<_> = report.warnings.iter().map(|w| w.path.clone()).collect();
        assert_eq!(
            got,
            [
                None,
                Some(PathBuf::from("/up")),
                Some(PathBuf::from("b/f")),
                Some(PathBuf::from("a/f")),
            ]
        );
    }

    /// The error that the walk gives for a root that it cannot read:
    /// `Error::Io(err).with_depth(depth).with_path(path)`.
    fn root_error(kind: io::ErrorKind, depth: usize, path: &str) -> ignore::Error {
        ignore::Error::WithPath {
            path: PathBuf::from(path),
            err: Box::new(ignore::Error::WithDepth {
                depth,
                err: Box::new(ignore::Error::Io(io::Error::from(kind))),
            }),
        }
    }

    // Decision 7 of FEATURE-14.
    #[test]
    fn a_root_that_does_not_exist_gives_the_warning_of_a_missing_path() {
        let warning = make_warning(&root_error(io::ErrorKind::NotFound, 0, "nofile"));
        assert_eq!(warning, self::warning(Some("nofile"), WarningKind::Missing));
        assert_eq!(warning.text(), "the path nofile does not exist");
        assert_eq!(warning.message(), "jag: the path nofile does not exist");
        let warning = make_warning(&root_error(io::ErrorKind::NotADirectory, 0, "a.txt/x"));
        assert_eq!(warning.kind, WarningKind::Missing);
    }

    // Decision 10 of FEATURE-14.
    #[test]
    fn other_errors_of_the_walk_stay_read_warnings() {
        let warning = make_warning(&root_error(io::ErrorKind::NotFound, 1, "dir/gone"));
        assert_eq!(warning.kind, WarningKind::Read(io::ErrorKind::NotFound));
        let warning = make_warning(&root_error(io::ErrorKind::PermissionDenied, 0, "locked"));
        assert_eq!(
            warning.kind,
            WarningKind::Read(io::ErrorKind::PermissionDenied)
        );
    }

    #[test]
    fn a_report_knows_when_a_path_is_missing() {
        let mut report = ScanReport::empty();
        assert!(!report.path_missing());
        report.warnings.push(warning(
            Some("x"),
            WarningKind::Read(io::ErrorKind::NotFound),
        ));
        assert!(!report.path_missing());
        report
            .warnings
            .push(warning(Some("nofile"), WarningKind::Missing));
        assert!(report.path_missing());
    }

    #[test]
    fn a_read_error_of_standard_input_has_its_own_text() {
        let warning = warning(None, WarningKind::StandardInput(io::ErrorKind::Other));
        assert_eq!(
            warning.message(),
            "jag: cannot read standard input: other error"
        );
    }

    fn matcher() -> grep_regex::RegexMatcher {
        grep_regex::RegexMatcher::new("needle").expect("a valid pattern")
    }

    // Decision 3 of FEATURE-14.
    #[test]
    fn piped_text_with_no_byte_gives_an_empty_report() {
        assert_eq!(scan_input(b"", &matcher(), false), ScanReport::empty());
    }

    /// The line numbers and the texts of a report.
    fn lines(report: &ScanReport) -> Vec<(u64, Vec<u8>)> {
        report
            .files
            .iter()
            .flat_map(|file| file.lines.iter())
            .map(|line| (line.line_number, line.text.clone()))
            .collect()
    }

    // CRITERION-35 and decision 11 of FEATURE-14: the piped text gives the
    // lines of a file with the same bytes.
    #[test]
    fn piped_text_gives_the_lines_of_a_file_with_the_same_bytes() {
        let mut utf16_little = vec![0xFF, 0xFE];
        let mut utf16_big = vec![0xFE, 0xFF];
        for unit in "x\nneedle é\n".encode_utf16() {
            utf16_little.extend(unit.to_le_bytes());
            utf16_big.extend(unit.to_be_bytes());
        }
        let texts: [&[u8]; 4] = [
            b"x\nneedle here\r\nneedle\n",
            b"\xEF\xBB\xBFneedle\nx\n",
            &utf16_little,
            &utf16_big,
        ];
        for text in texts {
            let dir = tempfile::tempdir().expect("temporary folder");
            let file = dir.path().join("a.txt");
            std::fs::write(&file, text).expect("write file");
            let from_file = scan(&[file], &matcher(), &ScanOptions::default());
            let from_pipe = scan_input(text, &matcher(), false);
            assert!(!from_pipe.files.is_empty());
            assert_eq!(lines(&from_pipe), lines(&from_file));
            assert_eq!(from_pipe.files[0].source, Source::StandardInput);
            assert!(from_pipe.warnings.is_empty());
        }
    }

    // Decision 11 of FEATURE-14: the binary rule of a file.
    #[test]
    fn binary_piped_text_gives_no_file_and_one_skip() {
        let report = scan_input(b"needle\0\n", &matcher(), false);
        assert!(report.files.is_empty());
        assert_eq!(
            report.skipped,
            SkipCounts {
                binary: 1,
                ..SkipCounts::default()
            }
        );
    }

    // Decision 5 of FEATURE-14.
    #[test]
    fn a_scan_of_the_current_folder_gives_paths_with_no_dot() {
        let file = |path: &str| FileMatches {
            source: Source::File(PathBuf::from(path)),
            lines: Vec::new(),
        };
        let report = ScanReport {
            files: vec![file("./sub/b.txt"), file("./a.txt")],
            skipped: SkipCounts::default(),
            warnings: vec![
                warning(Some("./x"), WarningKind::Other),
                warning(Some("."), WarningKind::Other),
                warning(Some("/up/.ignore"), WarningKind::IgnoreRule),
            ],
        };
        let report = without_current_folder_prefix(report);
        let names: Vec<_> = report
            .files
            .iter()
            .map(|file| String::from_utf8_lossy(file.source.label()).into_owned())
            .collect();
        assert_eq!(names, ["a.txt", "sub/b.txt"]);
        let paths: Vec<_> = report.warnings.iter().map(|w| w.path.clone()).collect();
        assert_eq!(
            paths,
            [
                Some(PathBuf::from(".")),
                Some(PathBuf::from("/up/.ignore")),
                Some(PathBuf::from("x")),
            ]
        );
    }
}
