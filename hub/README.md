# SENNEP hub

Our shared workspace: tasks, parts, tools, shopping list, battery cell log, ideas, links, docs and notes.
Everything is plain files in this repo, so Mikkel edits through the browser and Claude edits the files directly.

## Open it
1. Pull the repo as usual.
2. Open `hub/index.html` in **Chrome or Edge** (double-click is fine).
3. Click **Choose folder…** and pick the **repo root** (the folder with `hub/`, `components/`, `docs/`). Allow editing.
   Next time it is one click: **Reconnect**.
4. Edit. Changes save to the files automatically (~0.5 s). Commit and push with git when you are done.

If a file changed on disk since it was loaded (git pull, Claude), the hub asks before overwriting. Switching back to the
tab reloads files that changed.

## Where things live
| Tab | File |
|---|---|
| Overview | Summary of the files below |
| Tasks | `hub/data/tasks.csv` (id;title;area;status;who;priority;depends_on;note) |
| Parts / Tools | `components/inventory.csv` (the parts list; Tools = category `Tools`). Tick `counted` after a physical check. Run `python components/build_inventory.py` afterwards to refresh `INVENTORY.md` |
| Shopping | `hub/data/shopping.csv` (tier: minimum / recommended / nice) |
| Battery cells | `hub/data/battery_cells.csv` (shows range and spread live) |
| Ideas | `hub/data/ideas.csv` |
| Links | `hub/data/links.csv` |
| Docs & notes | Every `.md` in the repo's doc folders (view + edit). New notes go to `hub/notes/`, attachments (diagrams, photos, PDFs) to `hub/files/` |

## Rules for both of us
- CSV files use `;` as separator, UTF-8, one header row. Quotes are allowed when a value contains `;`.
- Do not rename columns without updating `hub/index.html` (the `VIEWS` list at the top of the script).
- Public repo: no receipts, addresses, order numbers or passwords in notes or attachments.
- Markdown rendering uses `marked` from cdnjs; offline, docs show as plain text.
