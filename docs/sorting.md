# Sorting

```bash
tooka sort                 # sort the default source folder
tooka sort --source ~/Desktop --rules pdfs,photos
tooka sort --dry-run       # only show what would happen
```

## Which files are sorted

Tooka looks at every regular file below the source folder, including
sub-folders. It does **not** touch:

- symbolic links,
- files that are already inside a `move`/`copy` destination of one of your
  rules that lies within the source folder (so sorting twice does not move
  sorted files again),
- files whose names are not valid Unicode (rare; only possible on Linux).
  They are listed as warnings.

Each file is handled by the first matching rule, see
[Which rule wins](rules.md#which-rule-wins).

## How a sort runs

A sort is **all or nothing**: either every file ends up where your rules
say, or nothing changes.

1. **Plan.** Tooka works out every step for every file without changing
   anything. Any problem stops the sort here: a destination that already
   exists (see [`on_conflict`](rules.md#conflicts-on_conflict)), two files
   that would end up at the same place, a rename template producing an
   invalid name, or a command that does not exist.
2. **Do.** Folders are created and files are moved, copied and renamed.
   Every step is recorded in a journal on disk *before* it happens. If a
   step fails (for example a file was deleted or a new file appeared in the
   meantime) or you press Ctrl+C, Tooka undoes everything it did so far.
3. **Finish.** Only now the steps that cannot be undone run, file by file:
   deleting files (`delete`), removing the originals of moves to another
   drive, and running commands (`execute`). If one of them fails, it is
   reported, but the sort is not undone: your files are already sorted.

Tooka never overwrites a file. Before it moves or copies a file, it
reserves the destination name; if something already exists there, that
step fails and the sort is undone.

Moving files to another drive or partition works: the file is copied, the
copy is flushed to disk, and the original is removed in step 3 only if it
did not change in the meantime.

## Interrupted sorts

If Tooka is killed or the computer loses power during step 2, the journal
stays behind. The next `tooka sort` (and the monitoring daemon) refuses to
run and asks you to run:

```bash
tooka recover
```

It puts every file back where it was before the sort. Recovery is careful:
it only removes files that it can prove are Tooka's own (an empty
placeholder, or a copy whose original is still there and unchanged). If a
file was changed in the meantime, it is kept, and recover tells you which
files to look at. Running it again is safe.

If the interruption happened during step 3, all files are already sorted;
`tooka recover` lists the deletions and commands that may not have run, so
you can check them yourself.

## Reports

```bash
tooka sort --report pdf --output ~/reports
```

Writes `tooka_report.pdf` (or `.json`, `.csv`) listing every file, the rule
that matched and what happened to it. Without `--output`, the report goes
into the current folder.

## Logs

Every file operation is logged. See [Configuration](configuration.md#logs).
