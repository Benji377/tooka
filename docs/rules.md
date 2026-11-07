# Rules

A rule says **which files** it applies to (`when`) and **what happens** to
them (`then`). Rules are written in YAML, checked with `tooka validate`,
and added with `tooka add`. `tooka template` writes a commented example.

```yaml
id: photos                    # unique; used by toggle/remove/export/--rules
name: Photos by date
enabled: true
description: Sort camera pictures into dated folders   # optional
priority: 5                   # higher runs first, see "Which rule wins"
when:
  extensions: [jpg, jpeg]
then:
  - action: move
    to: ~/Pictures/sorted
  - action: rename
    to: "{{metadata.EXIF:DateTime|date:%Y-%m-%d}}_{{filename}}.jpg"
on_conflict: abort            # optional: abort (default), skip or rename
```

A file may contain one rule, or several rules in a list under `rules:`:

```yaml
# my rules
rules:
  - id: pdfs
    ...
  - id: photos
    ...
```

## Which rule wins

Each file is handled by **at most one rule**: the first enabled rule that
matches, after sorting by `priority` (highest first). Rules with the same
priority keep their order: the order in the rules file, or with
`tooka sort --rules b,a` the order given there. Files that match no rule are
left alone.

## Conditions (`when`)

All fields are optional. By default **all** conditions that are set must
match. With `any: true`, **one** matching condition is enough. A rule
without conditions matches every file.

| Field | Matches when | Example |
|---|---|---|
| `filename` | the file name (with extension) matches a [regular expression](https://docs.rs/regex/latest/regex/#syntax) | `^invoice_.*\.pdf$` |
| `extensions` | the extension is in the list (without dot; upper/lower case matters) | `[jpg, JPG, png]` |
| `path` | the full path matches a glob pattern | `**/Downloads/tmp/*` |
| `size_kb` | the size is within `min`/`max` in KiB (both inclusive, either optional) | `{min: 100, max: 5000}` |
| `mime_type` | the type guessed from the extension matches; `type/*` matches a family | `image/*`, `application/pdf` |
| `created_date` | the creation day is within `from`/`to` (inclusive, either optional) | `{from: "-30d", to: now}` |
| `modified_date` | the modification day is within `from`/`to` | `{from: "2024-01-01"}` |
| `is_symlink` | the file is (not) a symbolic link; symlinks are never sorted, so only `false` is useful | `false` |
| `metadata` | every listed EXIF field exists and, if `value` is given, matches the glob | `[{key: "EXIF:Make", value: "Google"}]` |

**Dates** can be `YYYY-MM-DD`, RFC 3339 (`2025-06-20T10:00:00Z`), `now`, or
relative to now: `-7d`, `+2w`, `-1m` (30 days), `-1y` (365 days), `-12h`,
`-30s`. Dates are compared by **calendar day** (UTC): `from: "-12h"` matches
everything from the start of that day. Not every filesystem records
creation dates; files without one never match `created_date`.

**EXIF keys** are `EXIF:` followed by the tag name, for example
`EXIF:Make`, `EXIF:Model`, `EXIF:DateTimeOriginal`, `EXIF:LensModel`
(upper/lower case does not matter). Note that `EXIF:DateTime` in a condition
is the *last modified* tag; use `EXIF:DateTimeOriginal` for when a photo was
taken.

`tooka validate` reports invalid regular expressions, glob patterns and
dates, so a typo does not silently make a rule match nothing.

## Actions (`then`)

Actions run in order, each on the result of the previous one: after `move`,
a `rename` renames the moved file. After `copy`, the following actions work
on **the copy**; the original stays where it is.

Order rules (checked by `tooka validate`):

- `execute` must come after all `move`, `copy` and `rename` actions,
- `delete` must be the last action.

### `move` and `copy`

```yaml
- action: move           # or: copy
  to: ~/Documents/Invoices
  preserve_structure: false   # optional, default false
```

`to` is a folder; it is created if needed.

| `to` starts with | Resolved from | Example |
|---|---|---|
| `~` | your home folder | `~/Documents` |
| `.` | the folder Tooka is started in | `./sorted`, `../archive` (also `.hidden`!) |
| `/` or a drive (`C:\`) | absolute path | `/data/photos` |
| anything else | treated as absolute (a `/` is added) | `Documents` = `/Documents` |

With `preserve_structure: true` the file keeps its sub-folders relative to
the source folder: `Downloads/2024/a.pdf` → `Invoices/2024/a.pdf`.

Moving to another drive works; see [Sorting](sorting.md#how-a-sort-runs).

### `rename`

```yaml
- action: rename
  to: "{{filename}}-scanned.pdf"
```

`to` is the new file name (no folders). Placeholders:

| Placeholder | Value |
|---|---|
| `{{filename}}` | current name **without** extension (add the extension yourself) |
| `{{metadata.modified}}` | modification time |
| `{{metadata.created}}` | creation time (if the filesystem records it) |
| `{{metadata.size}}` | size in bytes |
| `{{metadata.EXIF:DateTime}}` | when a photo was **taken** (EXIF `DateTimeOriginal`) |
| `{{metadata.EXIF:<Tag>}}` | any other EXIF field, e.g. `{{metadata.EXIF:Model}}` |

Add `|date:FORMAT` to format a date with
[chrono format codes](https://docs.rs/chrono/latest/chrono/format/strftime/index.html),
e.g. `{{metadata.modified|date:%Y-%m}}`. Unknown placeholders and missing
metadata become empty text. If the result is empty or not a valid file name
on your system, the sort stops before anything happens.

### `delete`

```yaml
- action: delete
  trash: true
```

Files go to the trash by default. Write `trash: false` to delete them
permanently instead.

Deleting happens at the very end of a sort, and only for files that did not
change during the sort.

### `execute`

```yaml
- action: execute
  command: notify-send
  args: ["Sorted", "{{path}}"]
```

Runs a program directly (not through a shell, so no pipes or `*`).
`{{path}}` in `args` is replaced with the file's path after the preceding
actions, e.g. its new location after a `move`. Commands run at the very end
of a sort; a failing command is reported but does not undo the sort. A
command that cannot be found stops the sort before anything happens (on
Windows, where this cannot always be checked, it is a warning).

### `skip`

```yaml
- action: skip
```

Does nothing. Use it as the only action of a high-priority rule to protect
files from lower-priority rules. (Actions listed after `skip` still run.)

## Conflicts (`on_conflict`)

What happens when a destination already exists, or two files would end up
at the same place:

| Value | Behavior |
|---|---|
| `abort` (default) | Nothing is changed; the sort lists every conflict. |
| `skip` | That file is left where it is (reported as a warning); the others are sorted. |
| `rename` | A free name is used: `report (1).pdf`, `report (2).pdf`, ... |

Tooka never overwrites a file, whatever the setting. For folders sorted by
the [monitoring daemon](monitoring.md), `skip` or `rename` is usually the
better choice: with `abort`, one conflicting file blocks every run.
