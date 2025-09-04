# Side-by-Side Git Diff View

This document describes the new side-by-side Git diff view feature implemented for Zed.

## Overview

The side-by-side diff view provides a VS Code-like experience for viewing Git differences, with the old version on the left and the new version on the right, with synchronized scrolling and character-level highlighting.

## Features

- **Two-Column Layout**: Left pane shows the old version, right pane shows the new version
- **Synchronized Scrolling**: Both panes scroll together to maintain alignment
- **Character-Level Highlighting**: Shows exactly which characters changed (like VS Code)
- **Keyboard Navigation**: Support for `Ctrl+W+H` and `Ctrl+W+L` to switch between panes
- **Git Integration**: Works with existing Git panel and diff actions

## Usage

### Opening Side-by-Side Diff View

1. **From Command Palette**:

   - Open command palette (`Cmd+Shift+P` on macOS, `Ctrl+Shift+P` on Linux/Windows)
   - Search for "Open Side-by-Side Diff" or "git::OpenSideBySideDiff"

2. **Keyboard Shortcut**:

   - `Ctrl+G S` - Opens side-by-side diff for the current file

3. **From Git Panel**:
   - The feature integrates with the existing Git panel workflow

### Navigation

- **Switch to Left Pane**: `Ctrl+W H`
- **Switch to Right Pane**: `Ctrl+W L`
- **Toggle Between Panes**: `Ctrl+W W`

### Keyboard Shortcuts

| Shortcut   | Action                                  |
| ---------- | --------------------------------------- |
| `Ctrl+G S` | Open side-by-side diff for current file |
| `Ctrl+W H` | Switch to left pane (old version)       |
| `Ctrl+W L` | Switch to right pane (new version)      |
| `Ctrl+W W` | Toggle between left and right panes     |

## Implementation Details

### Architecture

The side-by-side diff view is implemented as a new `SideBySideDiffView` struct in the `git_ui` crate:

- **Left Editor**: Shows the old version of the file
- **Right Editor**: Shows the new version of the file
- **Synchronized Scrolling**: Both editors scroll together via event subscriptions
- **Diff Highlighting**: Uses the existing `BufferDiff` system for highlighting changes

### Key Components

1. **`SideBySideDiffView`**: Main view struct that manages both editors
2. **`FocusedPane`**: Enum to track which pane is currently focused
3. **Synchronized Scrolling**: Event-driven scrolling synchronization
4. **Git Integration**: Actions and keybindings integrated with existing Git system

### Files Modified

- `crates/git_ui/src/side_by_side_diff_view.rs` - Main implementation
- `crates/git_ui/src/git_ui.rs` - Integration with Git actions
- `assets/keymaps/*.json` - Keyboard shortcuts for all platforms

## Future Enhancements

- **Character-Level Diff Highlighting**: More granular highlighting of individual character changes
- **Inline Diff View**: Option to show changes inline rather than side-by-side
- **Three-Way Merge**: Support for three-way merges during conflict resolution
- **Diff Statistics**: Show statistics about changes (lines added/removed)
- **Diff Navigation**: Navigate between hunks with keyboard shortcuts

## Testing

The implementation includes basic tests in the `side_by_side_diff_view.rs` file. To run tests:

```bash
cargo test side_by_side_diff_view
```

## Contributing

When contributing to this feature:

1. Follow the existing code style and patterns
2. Add tests for new functionality
3. Update documentation as needed
4. Ensure keyboard shortcuts work across all platforms
5. Test with various Git scenarios (modified, added, deleted files)

## Related Issues

This implementation addresses the feature request in:

- [GitHub Discussion #26770](https://github.com/zed-industries/zed/discussions/26770)

The feature provides the side-by-side diff view that users have been requesting, similar to VS Code's diff view experience.
