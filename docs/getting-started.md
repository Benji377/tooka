# Getting started

This walks through sorting PDF files out of your Downloads folder.

## 1. Write a rule

Create `pdfs.yaml`:

```yaml
id: pdfs
name: PDF documents
enabled: true
description: Move PDFs into Documents/PDFs
priority: 1
when:
  extensions: [pdf]
then:
  - action: move
    to: ~/Documents/PDFs
```

`tooka template` writes a larger, commented example to start from. All
options are explained in the [rule reference](rules.md).

## 2. Check and add it

```bash
tooka validate pdfs.yaml   # checks structure, patterns, dates and actions
tooka add pdfs.yaml        # stores the rule in Tooka's rules file
tooka list                 # shows all rules
```

The YAML file itself is no longer needed after `add`; Tooka keeps its own
copy. Use `tooka export pdfs` to get it back, `tooka toggle pdfs` to disable
or enable it, and `tooka remove pdfs` to delete it.

## 3. Preview

```bash
tooka sort --dry-run
```

By default Tooka sorts your Downloads folder; use `--source <folder>` for
another one. A dry run shows every planned step and changes nothing.

## 4. Sort

```bash
tooka sort
```

Tooka first checks the whole plan. If anything would go wrong, for example
a file with the same name already exists in `~/Documents/PDFs`, it stops
**before changing anything** and tells you why. Read
[Sorting](sorting.md) for the details and for how to handle such
conflicts.

## 5. Automate (optional)

```bash
tooka monitor add ~/Downloads --interval 3600
tooka monitor daemon run
```

See [Monitoring](monitoring.md).
