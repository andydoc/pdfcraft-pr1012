# Local roadmap (andydoc/pdfcraft-pr1012)

Wishlist items for this fork. Kept apart from `ROADMAP.md` and `docs/roadmap.md` so upstream
merges do not conflict. Not scheduled, not estimated.

## Views > Tools > Crop and Perspective

A crop-and-perspective tool like the one in PhotoCraft: drag the four corners of a quad over the
picture, then straighten it to a rectangle (keystone and skew correction) and crop to it.

Open question on scope, to decide before building:

- **Image import only:** offered when a photo or scan is brought in (Create PDF from image, or
  the scan import path), before it becomes a page.
- **Any page:** offered from Views > Tools on whatever page is showing, correcting the page's
  scanned image (or a rasterised copy of the page) in place.

Starting points in the existing code:

- Scan correction (M10 #1012) already deskews up to 12 degrees and crops paper edges, in the
  Enhance scanned file path; a perspective warp would sit beside it.
- Edit on a scanned page deliberately runs with rotate, deskew and crop off
  (`scan_edit_settings` in `crates/ui-egui/src/ocr_ui.rs`), so a manual tool must be explicit,
  undoable and never run on opening a page.

Added 2026-10-29.
